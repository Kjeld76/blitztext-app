//! Transkription. Default: lokal via `whisper-rs` (whisper.cpp, large-v3).
//! Optional: Online-STT (OpenAI-kompatibel) — portiert aus `TranscriptionService.swift`.

use crate::settings::{self, GatewaySettings};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

// MARK: - Modell-Registry

pub struct ModelInfo {
    pub name: &'static str,
    pub display_name: &'static str,
    pub file_name: &'static str,
    pub url: &'static str,
    pub approx_mb: u32,
}

/// Verfügbare lokale Modelle. `large-v3` = höchste Genauigkeit (Nutzerwunsch),
/// `large-v3-turbo` als schnellere Alternative für reine CPU.
pub const MODELS: &[ModelInfo] = &[
    ModelInfo {
        name: "ggml-large-v3",
        display_name: "Large v3 (höchste Genauigkeit)",
        file_name: "ggml-large-v3.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin",
        approx_mb: 3100,
    },
    ModelInfo {
        name: "ggml-large-v3-turbo",
        display_name: "Large v3 Turbo (schnell)",
        file_name: "ggml-large-v3-turbo.bin",
        url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin",
        approx_mb: 1600,
    },
];

pub fn model_info(name: &str) -> &'static ModelInfo {
    MODELS
        .iter()
        .find(|m| m.name == name)
        .unwrap_or(&MODELS[0])
}

pub fn model_path(name: &str) -> PathBuf {
    settings::models_dir().join(model_info(name).file_name)
}

pub fn is_installed(name: &str) -> bool {
    model_path(name).is_file()
}

// MARK: - Modell-Download (mit Fortschritt 0..1)

pub async fn download_model<F>(name: &str, mut progress: F) -> Result<(), String>
where
    F: FnMut(f64),
{
    use futures_util::StreamExt;

    let info = model_info(name);
    let dir = settings::models_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Ordner-Fehler: {e}"))?;
    let target = model_path(name);
    let tmp = target.with_extension("download");

    let client = reqwest::Client::new();
    let resp = client
        .get(info.url)
        .send()
        .await
        .map_err(|e| format!("Download-Fehler: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("Download fehlgeschlagen: HTTP {}", resp.status()));
    }
    let total = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut file = tokio::fs::File::create(&tmp)
        .await
        .map_err(|e| format!("Datei-Fehler: {e}"))?;
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download-Abbruch: {e}"))?;
        tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
            .await
            .map_err(|e| format!("Schreibfehler: {e}"))?;
        downloaded += chunk.len() as u64;
        if total > 0 {
            progress((downloaded as f64 / total as f64).clamp(0.0, 1.0));
        }
    }
    tokio::io::AsyncWriteExt::flush(&mut file)
        .await
        .map_err(|e| format!("Schreibfehler: {e}"))?;
    drop(file);
    std::fs::rename(&tmp, &target).map_err(|e| format!("Umbenennen fehlgeschlagen: {e}"))?;
    progress(1.0);
    Ok(())
}

// MARK: - Lokale Transkription (whisper-rs), mit Kontext-Cache

struct ContextCache {
    model_name: String,
    ctx: Arc<WhisperContext>,
}

fn cache() -> &'static Mutex<Option<ContextCache>> {
    static CACHE: OnceLock<Mutex<Option<ContextCache>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(None))
}

fn load_context(model_name: &str) -> Result<Arc<WhisperContext>, String> {
    {
        let guard = cache().lock().unwrap();
        if let Some(c) = guard.as_ref() {
            if c.model_name == model_name {
                return Ok(c.ctx.clone());
            }
        }
    }
    let path = model_path(model_name);
    if !path.is_file() {
        return Err(format!(
            "Lokales Modell fehlt: {}. Bitte in den Einstellungen herunterladen.",
            model_info(model_name).display_name
        ));
    }
    let ctx = WhisperContext::new_with_params(&path, WhisperContextParameters::default())
        .map_err(|e| format!("Modell laden fehlgeschlagen: {e}"))?;
    let arc = Arc::new(ctx);
    *cache().lock().unwrap() = Some(ContextCache {
        model_name: model_name.to_string(),
        ctx: arc.clone(),
    });
    Ok(arc)
}

/// Lädt das Modell vorab in den Cache (Prewarm), blockierend.
pub fn prewarm(model_name: &str) -> Result<(), String> {
    load_context(model_name).map(|_| ())
}

/// Transkribiert 16-kHz-Mono-Samples lokal. Blockierend — via spawn_blocking aufrufen.
pub fn transcribe_local(
    samples_16k: &[f32],
    language: &str,
    custom_terms: &[String],
    model_name: &str,
) -> Result<String, String> {
    let ctx = load_context(model_name)?;
    let mut state = ctx
        .create_state()
        .map_err(|e| format!("Whisper-State-Fehler: {e}"))?;

    // Mit GPU (CUDA) Beam-Search (genauer); auf reiner CPU Greedy (schneller).
    #[cfg(feature = "cuda")]
    let strategy = SamplingStrategy::BeamSearch {
        beam_size: 5,
        patience: -1.0,
    };
    #[cfg(not(feature = "cuda"))]
    let strategy = SamplingStrategy::Greedy { best_of: 1 };

    let mut params = FullParams::new(strategy);
    if !language.trim().is_empty() {
        params.set_language(Some(language));
    }
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    let prompt;
    if !custom_terms.is_empty() {
        prompt = format!("Eigennamen und Begriffe: {}", custom_terms.join(", "));
        params.set_initial_prompt(&prompt);
    }

    state
        .full(params, samples_16k)
        .map_err(|e| format!("Transkription fehlgeschlagen: {e}"))?;

    let num = state.full_n_segments();
    let mut text = String::new();
    for i in 0..num {
        if let Some(seg) = state.get_segment(i) {
            if let Ok(s) = seg.to_str_lossy() {
                text.push_str(&s);
            }
        }
    }
    Ok(text.trim().to_string())
}

// MARK: - Optionale Online-Transkription (OpenAI-kompatibel)

pub async fn transcribe_online(
    wav_path: &std::path::Path,
    gateway: &GatewaySettings,
    api_key: Option<String>,
    custom_terms: &[String],
    language: &str,
) -> Result<String, String> {
    if gateway.stt_base_url.trim().is_empty() {
        return Err("Kein Online-STT-Endpunkt konfiguriert.".into());
    }
    let bytes = tokio::fs::read(wav_path)
        .await
        .map_err(|e| format!("Audio-Datei-Fehler: {e}"))?;

    let part = reqwest::multipart::Part::bytes(bytes)
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| e.to_string())?;
    let mut form = reqwest::multipart::Form::new()
        .part("file", part)
        .text("model", gateway.stt_model.clone())
        .text("response_format", "text");
    if !custom_terms.is_empty() {
        form = form.text(
            "prompt",
            format!("Eigennamen und Begriffe: {}", custom_terms.join(", ")),
        );
    }
    if !language.trim().is_empty() {
        form = form.text("language", language.trim().to_string());
    }

    let url = format!(
        "{}/audio/transcriptions",
        gateway.stt_base_url.trim_end_matches('/')
    );
    let client = reqwest::Client::new();
    let mut req = client.post(url).multipart(form);
    if let Some(key) = api_key {
        req = req.header("Authorization", format!("Bearer {key}"));
    }
    let resp = req.send().await.map_err(|e| format!("Netzwerkfehler: {e}"))?;
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("STT-Fehler: HTTP {} {}", status.as_u16(), body));
    }
    Ok(body.trim().to_string())
}
