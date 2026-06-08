//! Mikrofon-Aufnahme via `cpal` → 16 kHz mono. Ersetzt `AudioRecorder.swift`.
//!
//! Der cpal-Input-Stream ist auf Windows (WASAPI) nicht `Send`, daher läuft er
//! auf einem dedizierten Thread, der über ein Atomic-Flag gestoppt wird.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Sample;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

const TARGET_RATE: u32 = 16_000;

#[derive(Default)]
struct Shared {
    /// Roh aufgenommene Mono-Samples (in Quell-Samplerate).
    samples: Mutex<Vec<f32>>,
    src_rate: AtomicU32,
    /// Aktueller Pegel (RMS, 0..1) für die Waveform-Anzeige.
    level: Mutex<f32>,
    error: Mutex<Option<String>>,
}

pub struct Recorder {
    recording: Arc<AtomicBool>,
    shared: Arc<Shared>,
    handle: Option<JoinHandle<()>>,
    started_at: Option<Instant>,
}

/// Ergebnis einer Aufnahme: 16-kHz-Mono-Samples + Dauer in Sekunden.
pub struct Recording {
    pub samples_16k: Vec<f32>,
    pub duration_secs: f64,
}

impl Default for Recorder {
    fn default() -> Self {
        Self::new()
    }
}

impl Recorder {
    pub fn new() -> Self {
        Self {
            recording: Arc::new(AtomicBool::new(false)),
            shared: Arc::new(Shared::default()),
            handle: None,
            started_at: None,
        }
    }

    pub fn is_recording(&self) -> bool {
        self.recording.load(Ordering::Acquire)
    }

    pub fn level(&self) -> f32 {
        *self.shared.level.lock().unwrap()
    }

    /// Startet die Aufnahme auf einem eigenen Thread.
    pub fn start(&mut self) -> Result<(), String> {
        if self.is_recording() {
            return Ok(());
        }
        // Zustand zurücksetzen.
        self.shared.samples.lock().unwrap().clear();
        *self.shared.level.lock().unwrap() = 0.0;
        *self.shared.error.lock().unwrap() = None;
        self.recording.store(true, Ordering::Release);

        let recording = self.recording.clone();
        let shared = self.shared.clone();
        // Der Capture-Thread meldet hierueber, sobald der Stream wirklich laeuft.
        let (ready_tx, ready_rx) = mpsc::channel::<Result<(), String>>();

        let handle = std::thread::spawn(move || {
            if let Err(e) = capture_loop(&recording, &shared, ready_tx) {
                *shared.error.lock().unwrap() = Some(e);
                recording.store(false, Ordering::Release);
            }
        });

        self.handle = Some(handle);

        // Warten, bis das Mikrofon tatsaechlich aufnimmt, BEVOR die UI „Aufnahme"
        // signalisiert. Sonst geht der Wortanfang im WASAPI-Device-Open
        // (~100-300 ms) verloren, weil der Nutzer schon spricht.
        match ready_rx.recv_timeout(Duration::from_secs(3)) {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                // Setup im Capture-Thread fehlgeschlagen (z. B. kein Mikrofon).
                self.recording.store(false, Ordering::Release);
                if let Some(h) = self.handle.take() {
                    let _ = h.join();
                }
                return Err(e);
            }
            Err(_) => {
                // Timeout: lieber mit Mini-Verzug aufnehmen als gar nicht.
            }
        }

        self.started_at = Some(Instant::now());
        Ok(())
    }

    /// Stoppt die Aufnahme und liefert die 16-kHz-Mono-Samples + Dauer.
    pub fn stop(&mut self) -> Result<Recording, String> {
        if !self.is_recording() && self.handle.is_none() {
            return Err("Es läuft keine Aufnahme.".into());
        }
        self.recording.store(false, Ordering::Release);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
        let duration_secs = self
            .started_at
            .take()
            .map(|t| t.elapsed().as_secs_f64())
            .unwrap_or(0.0);

        if let Some(err) = self.shared.error.lock().unwrap().take() {
            return Err(err);
        }

        let src_rate = self.shared.src_rate.load(Ordering::Acquire).max(1);
        let raw = std::mem::take(&mut *self.shared.samples.lock().unwrap());
        let samples_16k = resample_mono(&raw, src_rate, TARGET_RATE);

        Ok(Recording {
            samples_16k,
            duration_secs,
        })
    }

    /// Bricht eine laufende Aufnahme ab (Samples verwerfen).
    pub fn discard(&mut self) {
        self.recording.store(false, Ordering::Release);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
        self.shared.samples.lock().unwrap().clear();
        self.started_at = None;
    }
}

fn capture_loop(
    recording: &Arc<AtomicBool>,
    shared: &Arc<Shared>,
    ready_tx: mpsc::Sender<Result<(), String>>,
) -> Result<(), String> {
    // Device öffnen + Stream bauen + starten. Erst danach ist das Mikrofon live.
    let stream = match setup_stream(shared) {
        Ok(s) => s,
        Err(e) => {
            let _ = ready_tx.send(Err(e.clone()));
            return Err(e);
        }
    };

    if let Err(e) = stream
        .play()
        .map_err(|e| format!("Aufnahme-Start fehlgeschlagen: {e}"))
    {
        let _ = ready_tx.send(Err(e.clone()));
        return Err(e);
    }

    // Stream läuft jetzt wirklich → grünes Licht für start().
    let _ = ready_tx.send(Ok(()));

    while recording.load(Ordering::Acquire) {
        std::thread::sleep(Duration::from_millis(10));
    }
    drop(stream);
    Ok(())
}

/// Öffnet das Standard-Mikrofon und baut den Capture-Stream (noch ohne `play`).
fn setup_stream(shared: &Arc<Shared>) -> Result<cpal::Stream, String> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| "Kein Mikrofon gefunden.".to_string())?;
    let config = device
        .default_input_config()
        .map_err(|e| format!("Mikrofon-Konfiguration fehlt: {e}"))?;

    let channels = config.channels() as usize;
    shared
        .src_rate
        .store(config.sample_rate().0, Ordering::Release);
    let sample_format = config.sample_format();
    let stream_config: cpal::StreamConfig = config.into();

    let shared_cb = shared.clone();
    let err_shared = shared.clone();
    let err_fn = move |e: cpal::StreamError| {
        *err_shared.error.lock().unwrap() = Some(format!("Audio-Fehler: {e}"));
    };

    let push = move |mono: &[f32]| {
        if mono.is_empty() {
            return;
        }
        let rms = (mono.iter().map(|s| s * s).sum::<f32>() / mono.len() as f32).sqrt();
        *shared_cb.level.lock().unwrap() = rms.min(1.0);
        shared_cb.samples.lock().unwrap().extend_from_slice(mono);
    };

    // Stream je nach Sample-Format bauen; Kanäle zu Mono mitteln.
    match sample_format {
        cpal::SampleFormat::F32 => build_stream::<f32>(&device, &stream_config, channels, push, err_fn),
        cpal::SampleFormat::I16 => build_stream::<i16>(&device, &stream_config, channels, push, err_fn),
        cpal::SampleFormat::U16 => build_stream::<u16>(&device, &stream_config, channels, push, err_fn),
        other => Err(format!("Nicht unterstütztes Audio-Format: {other:?}")),
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    channels: usize,
    mut push: impl FnMut(&[f32]) + Send + 'static,
    err_fn: impl FnMut(cpal::StreamError) + Send + 'static,
) -> Result<cpal::Stream, String>
where
    T: cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    device
        .build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                let mut mono = Vec::with_capacity(data.len() / channels.max(1));
                for frame in data.chunks(channels.max(1)) {
                    let sum: f32 = frame.iter().map(|s| f32::from_sample(*s)).sum();
                    mono.push(sum / channels as f32);
                }
                push(&mono);
            },
            err_fn,
            None,
        )
        .map_err(|e| format!("Stream-Aufbau fehlgeschlagen: {e}"))
}

/// Resampling von `src_rate` → `dst_rate` (mono) mit Anti-Aliasing.
/// Beim Heruntertasten wird zuerst ein Tiefpass angewandt, sonst falten sich
/// Frequenzen > Nyquist in das Sprachband und verschlechtern die Erkennung.
fn resample_mono(input: &[f32], src_rate: u32, dst_rate: u32) -> Vec<f32> {
    if input.is_empty() || src_rate == 0 {
        return Vec::new();
    }
    if src_rate == dst_rate {
        return input.to_vec();
    }

    let filtered = if dst_rate < src_rate {
        let cutoff = 0.45 * dst_rate as f32; // knapp unter Nyquist des Ziels
        low_pass_fir(input, src_rate as f32, cutoff)
    } else {
        input.to_vec()
    };

    let ratio = dst_rate as f64 / src_rate as f64;
    let out_len = (filtered.len() as f64 * ratio).round() as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src_pos = i as f64 / ratio;
        let idx = src_pos.floor() as usize;
        let frac = (src_pos - idx as f64) as f32;
        let a = filtered.get(idx).copied().unwrap_or(0.0);
        let b = filtered.get(idx + 1).copied().unwrap_or(a);
        out.push(a + (b - a) * frac);
    }
    out
}

/// Linear-phase Tiefpass-FIR (Hann-gefenstertes Sinc), Einheitsverstärkung bei DC.
fn low_pass_fir(input: &[f32], sample_rate: f32, cutoff_hz: f32) -> Vec<f32> {
    let taps: usize = 64; // -> 65 Koeffizienten, symmetrisch
    let fc = (cutoff_hz / sample_rate).min(0.5); // normierte Grenzfrequenz
    let m = taps as f32;
    let pi = std::f32::consts::PI;

    let mut h = vec![0f32; taps + 1];
    let mut sum = 0f32;
    for (n, hn) in h.iter_mut().enumerate() {
        let k = n as f32 - m / 2.0;
        let sinc = if k.abs() < 1e-6 {
            2.0 * fc
        } else {
            (2.0 * pi * fc * k).sin() / (pi * k)
        };
        let w = 0.5 - 0.5 * (2.0 * pi * n as f32 / m).cos(); // Hann
        *hn = sinc * w;
        sum += *hn;
    }
    for hn in h.iter_mut() {
        *hn /= sum;
    }

    let half = (h.len() / 2) as isize;
    let n = input.len();
    let mut out = vec![0f32; n];
    for (i, o) in out.iter_mut().enumerate() {
        let mut acc = 0f32;
        for (j, &coef) in h.iter().enumerate() {
            let idx = i as isize + j as isize - half;
            if idx >= 0 && (idx as usize) < n {
                acc += input[idx as usize] * coef;
            }
        }
        *o = acc;
    }
    out
}

/// Schreibt 16-kHz-Mono-Samples als 16-bit-PCM-WAV (für Online-STT).
pub fn write_wav_16k(path: &Path, samples_16k: &[f32]) -> Result<(), String> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: TARGET_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer =
        hound::WavWriter::create(path, spec).map_err(|e| format!("WAV-Fehler: {e}"))?;
    for &s in samples_16k {
        let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        writer.write_sample(v).map_err(|e| format!("WAV-Fehler: {e}"))?;
    }
    writer.finalize().map_err(|e| format!("WAV-Fehler: {e}"))?;
    Ok(())
}
