//! Persistente Einstellungen — portiert aus
//! `BlitztextMac/Features/Workflows/WorkflowProtocol.swift` (gleiche JSON-Keys)
//! und erweitert um Windows-spezifische Felder (konfigurierbare Hotkeys,
//! OpenAI-kompatibler LLM-Gateway).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// MARK: - Enums (RawValues identisch zur Swift-Variante)

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum HotkeyMode {
    #[default]
    Hold,
    Toggle,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[serde(rename_all = "camelCase")]
pub enum WorkflowType {
    Transcription,
    LocalTranscription,
    TextImprover,
    DampfAblassen,
    EmojiText,
}

impl WorkflowType {
    pub const ALL: [WorkflowType; 5] = [
        WorkflowType::Transcription,
        WorkflowType::LocalTranscription,
        WorkflowType::TextImprover,
        WorkflowType::DampfAblassen,
        WorkflowType::EmojiText,
    ];

    /// Anzeigename (aus WorkflowProtocol.swift:18-26)
    pub fn display_name(&self) -> &'static str {
        match self {
            WorkflowType::Transcription => "Blitztext",
            WorkflowType::LocalTranscription => "Blitztext Lokal",
            WorkflowType::TextImprover => "Blitztext+",
            WorkflowType::DampfAblassen => "Blitztext $%&!",
            WorkflowType::EmojiText => "Blitztext :)",
        }
    }

    /// Stabiler String-Key (== Swift rawValue, für JSON/Hotkey-Map)
    pub fn key(&self) -> &'static str {
        match self {
            WorkflowType::Transcription => "transcription",
            WorkflowType::LocalTranscription => "localTranscription",
            WorkflowType::TextImprover => "textImprover",
            WorkflowType::DampfAblassen => "dampfAblassen",
            WorkflowType::EmojiText => "emojiText",
        }
    }

    /// Default-Hotkey unter Windows (keine fn-Taste verfügbar).
    pub fn default_hotkey(&self) -> &'static str {
        match self {
            WorkflowType::Transcription => "Ctrl+Alt+B",
            WorkflowType::LocalTranscription => "Ctrl+Alt+L",
            WorkflowType::TextImprover => "Ctrl+Alt+P",
            WorkflowType::DampfAblassen => "Ctrl+Alt+R",
            WorkflowType::EmojiText => "Ctrl+Alt+E",
        }
    }

    /// Workflows, die nach der Transkription noch einen LLM-Schritt brauchen.
    pub fn needs_llm(&self) -> bool {
        matches!(
            self,
            WorkflowType::TextImprover | WorkflowType::DampfAblassen | WorkflowType::EmojiText
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum TextTone {
    Formal,
    #[default]
    Neutral,
    Casual,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "lowercase")]
pub enum EmojiDensity {
    Wenig,
    #[default]
    Mittel,
    Viel,
}

// MARK: - Settings-Structs (Keys identisch zu WorkflowProtocol.swift)

fn default_local_model() -> String {
    "ggml-large-v3".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    #[serde(default)]
    pub hotkey_mode: HotkeyMode,
    #[serde(default)]
    pub has_seen_onboarding: bool,
    #[serde(default)]
    pub secure_local_mode_enabled: bool,
    #[serde(default = "default_local_model")]
    pub selected_local_transcription_model_name: String,
    #[serde(default)]
    pub has_auto_selected_fast_local_model: bool,
    /// Pre-Roll: Mikrofon läuft warm (Ringpuffer), damit kein Wortanfang
    /// verloren geht und der Start ohne spürbaren Verzug erfolgt. Standard aus,
    /// da das Mikrofon dann dauerhaft offen ist (Windows zeigt es als aktiv).
    #[serde(default)]
    pub preroll_enabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            hotkey_mode: HotkeyMode::Hold,
            has_seen_onboarding: false,
            secure_local_mode_enabled: false,
            selected_local_transcription_model_name: default_local_model(),
            has_auto_selected_fast_local_model: false,
            preroll_enabled: false,
        }
    }
}

fn default_language() -> String {
    "de".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TranscriptionSettings {
    #[serde(default = "default_language")]
    pub language: String,
}

impl Default for TranscriptionSettings {
    fn default() -> Self {
        Self {
            language: default_language(),
        }
    }
}

fn default_dampf_prompt() -> String {
    // Wörtlich aus WorkflowProtocol.swift:176
    "Du erhältst ein emotional gesprochenes Transkript. Erkenne zuerst das eigentliche Ziel, \
     Anliegen und den wahren Frust der Person. Formuliere daraus eine klare, respektvolle und \
     wirksame Nachricht, mit der die Person ihr Ziel eher erreicht. Bewahre relevante Fakten, \
     konkrete Probleme, Grenzen, Erwartungen und die nötige Dringlichkeit. Entferne Beleidigungen, \
     Drohungen, Sarkasmus, Unterstellungen und unnötige Eskalation. Wenn mehrere Vorwürfe genannt \
     werden, verdichte sie auf die entscheidenden Kernpunkte. Der Ton soll ruhig, menschlich, \
     bestimmt und lösungsorientiert sein. Gib NUR die fertige Nachricht zurück."
        .to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DampfAblassenSettings {
    #[serde(default = "default_dampf_prompt")]
    pub system_prompt: String,
    #[serde(default)]
    pub custom_name: String,
}

impl Default for DampfAblassenSettings {
    fn default() -> Self {
        Self {
            system_prompt: default_dampf_prompt(),
            custom_name: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct EmojiTextSettings {
    #[serde(default)]
    pub emoji_density: EmojiDensity,
    #[serde(default)]
    pub custom_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TextImprovementSettings {
    #[serde(default)]
    pub system_prompt: String,
    #[serde(default)]
    pub custom_terms: Vec<String>,
    #[serde(default)]
    pub context: String,
    #[serde(default)]
    pub tone: TextTone,
    #[serde(default)]
    pub custom_name: String,
}

// MARK: - Windows-spezifisch: LLM-Gateway (auth2api / OpenAI-kompatibel)

fn default_gateway_base_url() -> String {
    // Default-Annahme für einen lokal laufenden auth2api-Docker-Proxy.
    "http://localhost:8317/v1".to_string()
}
fn default_correction_model() -> String {
    "claude-haiku-4-5".to_string()
}
fn default_fast_model() -> String {
    "claude-sonnet-4-6".to_string()
}
fn default_strong_model() -> String {
    "claude-sonnet-4-6".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GatewaySettings {
    /// OpenAI-kompatible Basis-URL (z. B. auth2api `http://localhost:PORT/v1`).
    #[serde(default = "default_gateway_base_url")]
    pub base_url: String,
    /// Modell für die Transkriptions-Korrektur (Ctrl+Alt+B). Leer = keine Korrektur.
    #[serde(default = "default_correction_model")]
    pub correction_model: String,
    /// Modell für die „schnelle" Variante (improve/emoji).
    #[serde(default = "default_fast_model")]
    pub fast_model: String,
    /// Modell für die „starke" Variante (dampfAblassen / beruhigen).
    #[serde(default = "default_strong_model")]
    pub strong_model: String,
    /// Optionaler separater STT-Endpunkt (nur falls vorhanden). Leer = lokal.
    #[serde(default)]
    pub stt_base_url: String,
    /// Online-STT-Modellname (z. B. "whisper-1").
    #[serde(default)]
    pub stt_model: String,
}

impl Default for GatewaySettings {
    fn default() -> Self {
        Self {
            base_url: default_gateway_base_url(),
            correction_model: default_correction_model(),
            fast_model: default_fast_model(),
            strong_model: default_strong_model(),
            stt_base_url: String::new(),
            stt_model: String::new(),
        }
    }
}

fn default_hotkeys() -> HashMap<String, String> {
    WorkflowType::ALL
        .iter()
        .map(|w| (w.key().to_string(), w.default_hotkey().to_string()))
        .collect()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WindowsSettings {
    #[serde(default)]
    pub launch_at_login: bool,
    #[serde(default = "default_hotkeys")]
    pub hotkeys: HashMap<String, String>,
    #[serde(default)]
    pub gateway: GatewaySettings,
}

impl Default for WindowsSettings {
    fn default() -> Self {
        Self {
            launch_at_login: false,
            hotkeys: default_hotkeys(),
            gateway: GatewaySettings::default(),
        }
    }
}

// MARK: - Container (Form wie SettingsContainer in AppState.swift, + windows)

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SettingsContainer {
    #[serde(default)]
    pub app: AppSettings,
    #[serde(default)]
    pub transcription: TranscriptionSettings,
    #[serde(default)]
    pub text_improvement: TextImprovementSettings,
    #[serde(default)]
    pub dampf_ablassen: DampfAblassenSettings,
    #[serde(default)]
    pub emoji_text: EmojiTextSettings,
    /// Windows-Erweiterung (nicht in der Mac-App vorhanden).
    #[serde(default)]
    pub windows: WindowsSettings,
}

impl SettingsContainer {
    /// Resolvter Hotkey-String für einen Workflow (Fallback auf Default).
    pub fn hotkey_for(&self, workflow: WorkflowType) -> String {
        self.windows
            .hotkeys
            .get(workflow.key())
            .cloned()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| workflow.default_hotkey().to_string())
    }
}

// MARK: - Pfade & Persistenz (%APPDATA%\Blitztext\)

/// Basisverzeichnis der App-Daten: `%APPDATA%\Blitztext`.
pub fn app_support_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("Blitztext")
}

pub fn settings_path() -> PathBuf {
    app_support_dir().join("settings.json")
}

pub fn models_dir() -> PathBuf {
    app_support_dir().join("models")
}

pub fn tmp_dir() -> PathBuf {
    app_support_dir().join("tmp")
}

/// Lädt die Settings; fehlt die Datei oder ist sie kaputt, kommen Defaults.
pub fn load() -> SettingsContainer {
    let path = settings_path();
    match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
        Err(_) => SettingsContainer::default(),
    }
}

/// Speichert die Settings atomar nach `settings.json`.
pub fn save(container: &SettingsContainer) -> std::io::Result<()> {
    let dir = app_support_dir();
    std::fs::create_dir_all(&dir)?;
    let path = settings_path();
    let tmp = path.with_extension("json.tmp");
    let data = serde_json::to_vec_pretty(container)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(&tmp, data)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}
