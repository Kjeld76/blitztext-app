//! Workflow-Orchestrierung — portiert aus den `*Workflow.swift` + `AppState.swift`.
//! Zweiphasig: Aufnahme → Transkription (lokal/online) → optional LLM → Paste.

use crate::audio::{Recorder, Recording};
use crate::settings::{self, SettingsContainer, WorkflowType};
use crate::{credentials, llm, paste, prompts, quality, transcription};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

/// Laufzeit-Zustand (getrennt von den persistenten Settings).
#[derive(Default)]
pub struct Engine {
    recorder: Recorder,
    active: Option<WorkflowType>,
    is_recording: bool,
    is_processing: bool,
}

impl Engine {
    pub fn is_busy(&self) -> bool {
        self.is_recording || self.is_processing
    }

    pub fn active(&self) -> Option<WorkflowType> {
        self.active
    }

    /// Startet die Aufnahme für einen Workflow.
    pub fn begin(&mut self, workflow: WorkflowType) -> Result<(), String> {
        if self.is_busy() {
            return Err("Es läuft bereits ein Vorgang.".into());
        }
        self.recorder.start()?;
        self.active = Some(workflow);
        self.is_recording = true;
        Ok(())
    }

    /// Beendet die Aufnahme und gibt Workflow + Aufnahme zurück.
    pub fn end(&mut self) -> Result<(WorkflowType, Recording), String> {
        if !self.is_recording {
            return Err("Es läuft keine Aufnahme.".into());
        }
        let workflow = self.active.ok_or("Kein aktiver Workflow.")?;
        let recording = self.recorder.stop()?;
        self.is_recording = false;
        self.is_processing = true;
        Ok((workflow, recording))
    }

    pub fn cancel(&mut self) {
        if self.is_recording {
            self.recorder.discard();
        }
        self.is_recording = false;
        self.is_processing = false;
        self.active = None;
    }

    pub fn finish_processing(&mut self) {
        self.is_processing = false;
        self.active = None;
    }

    pub fn level(&self) -> f32 {
        self.recorder.level()
    }
}

// MARK: - Status (Event an Frontend + Tray-Tooltip)

#[derive(Serialize, Clone)]
pub struct StatusEvent {
    pub phase: String,
    pub workflow: Option<String>,
    pub message: Option<String>,
}

pub fn set_status(app: &AppHandle, phase: &str, workflow: Option<WorkflowType>, message: Option<String>) {
    let evt = StatusEvent {
        phase: phase.to_string(),
        workflow: workflow.map(|w| w.key().to_string()),
        message: message.clone(),
    };
    let _ = app.emit("status", evt);

    // Tray-Tooltip aktualisieren.
    if let Some(tray) = app.tray_by_id("main") {
        let tip = match phase {
            "recording" => format!(
                "Blitztext – Aufnahme ({})",
                workflow.map(|w| w.display_name()).unwrap_or("")
            ),
            "processing" => message
                .clone()
                .unwrap_or_else(|| "Blitztext – verarbeite…".into()),
            "error" => format!("Blitztext – Fehler: {}", message.unwrap_or_default()),
            "done" => "Blitztext – fertig".into(),
            _ => "Blitztext".into(),
        };
        let _ = tray.set_tooltip(Some(&tip));
    }
}

// MARK: - Pipeline

/// Verarbeitet eine fertige Aufnahme asynchron bis zum Paste.
pub async fn run_pipeline(
    app: AppHandle,
    workflow: WorkflowType,
    recording: Recording,
    settings: SettingsContainer,
) {
    let result = process(&app, workflow, recording, &settings).await;

    match result {
        Ok(text) => {
            // Paste blockierend.
            let to_paste = text.clone();
            let paste_res =
                tokio::task::spawn_blocking(move || paste::paste_text(&to_paste)).await;
            match paste_res {
                Ok(Ok(())) => set_status(&app, "done", Some(workflow), None),
                Ok(Err(e)) => set_status(&app, "error", Some(workflow), Some(e)),
                Err(e) => set_status(&app, "error", Some(workflow), Some(e.to_string())),
            }
        }
        Err(e) => set_status(&app, "error", Some(workflow), Some(e)),
    }

    // Engine freigeben + nach kurzer Pause auf idle.
    if let Some(engine) = app.try_state::<crate::EngineState>() {
        engine.lock().unwrap().finish_processing();
    }
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    set_status(&app, "idle", None, None);
}

async fn process(
    app: &AppHandle,
    workflow: WorkflowType,
    recording: Recording,
    settings: &SettingsContainer,
) -> Result<String, String> {
    if quality::should_reject_recording(recording.duration_secs) {
        return Err("Keine Aufnahme erkannt.".into());
    }

    set_status(app, "processing", Some(workflow), Some("Wird transkribiert …".into()));
    let duration = recording.duration_secs;
    let language = settings.transcription.language.clone();
    let custom_terms = settings.text_improvement.custom_terms.clone();
    let vocab = if duration >= 0.9 { custom_terms.clone() } else { Vec::new() };

    // --- Phase 1: Transkription (lokal default, online nur falls konfiguriert) ---
    let force_local = matches!(workflow, WorkflowType::LocalTranscription)
        || settings.app.secure_local_mode_enabled;
    let online_available = !settings.windows.gateway.stt_base_url.trim().is_empty();

    let raw_text = if !force_local && online_available {
        let dir = settings::tmp_dir();
        let _ = std::fs::create_dir_all(&dir);
        let wav = dir.join("blitztext_rec.wav");
        crate::audio::write_wav_16k(&wav, &recording.samples_16k)?;
        let key = credentials::get(credentials::keys::STT_API_KEY);
        let r = transcription::transcribe_online(
            &wav,
            &settings.windows.gateway,
            key,
            &vocab,
            &language,
        )
        .await;
        let _ = std::fs::remove_file(&wav);
        r?
    } else {
        let model = settings.app.selected_local_transcription_model_name.clone();
        let samples = recording.samples_16k;
        let lang = language.clone();
        let vocab2 = vocab.clone();
        tokio::task::spawn_blocking(move || {
            transcription::transcribe_local(&samples, &lang, &vocab2, &model)
        })
        .await
        .map_err(|e| e.to_string())??
    };

    let cleaned = quality::cleaned_transcript(&raw_text);
    if quality::is_likely_artifact(&cleaned, duration) {
        return Err("Keine Aufnahme erkannt.".into());
    }

    let gateway = &settings.windows.gateway;

    // Blitztext Lokal (L): rein lokal, kein Cloud-Schritt.
    if matches!(workflow, WorkflowType::LocalTranscription) {
        return Ok(cleaned);
    }

    // Blitztext (B): lokale Transkription + leichte Claude-Korrektur.
    // Faellt bei nicht erreichbarem Gateway auf das rohe Transkript zurueck.
    if matches!(workflow, WorkflowType::Transcription) {
        if gateway.correction_model.trim().is_empty() {
            return Ok(cleaned);
        }
        set_status(app, "processing", Some(workflow), Some("Wird korrigiert …".into()));
        let system_prompt = prompts::build_correction_system_prompt(&custom_terms);
        return match llm::complete(gateway, &gateway.correction_model, &system_prompt, &cleaned, 0.0)
            .await
        {
            Ok(out) => {
                let corrected = quality::cleaned_transcript(&out);
                Ok(if corrected.is_empty() { cleaned } else { corrected })
            }
            Err(_) => Ok(cleaned),
        };
    }

    // --- Umschreib-Workflows (P/E/R): Transkript -> LLM ---
    let (system_prompt, model, temperature, status_msg): (String, String, f64, &str) = match workflow
    {
        WorkflowType::TextImprover => (
            prompts::build_improvement_system_prompt(&settings.text_improvement),
            gateway.fast_model.clone(),
            0.3,
            "Text wird verbessert …",
        ),
        WorkflowType::EmojiText => (
            prompts::build_emoji_system_prompt(settings.emoji_text.emoji_density),
            gateway.fast_model.clone(),
            0.3,
            "Emojis werden eingefügt …",
        ),
        WorkflowType::DampfAblassen => (
            settings.dampf_ablassen.system_prompt.clone(),
            gateway.strong_model.clone(),
            0.4,
            "Wird umformuliert …",
        ),
        _ => unreachable!(),
    };

    set_status(app, "processing", Some(workflow), Some(status_msg.into()));
    let out = llm::complete(gateway, &model, &system_prompt, &cleaned, temperature).await?;
    let cleaned_out = quality::cleaned_transcript(&out);

    if matches!(workflow, WorkflowType::DampfAblassen | WorkflowType::EmojiText)
        && cleaned_out == "KEINE_AUFNAHME_ERKANNT"
    {
        return Err("Keine Aufnahme erkannt.".into());
    }

    Ok(cleaned_out)
}
