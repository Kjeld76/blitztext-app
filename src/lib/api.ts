import { invoke } from "@tauri-apps/api/core";

// Spiegelt SettingsContainer (Rust) — camelCase Keys.
export type HotkeyMode = "hold" | "toggle";
export type TextTone = "formal" | "neutral" | "casual";
export type EmojiDensity = "wenig" | "mittel" | "viel";
export type PasteShortcutMode = "auto" | "ctrlV" | "ctrlShiftV";

export interface AppSettings {
  hotkeyMode: HotkeyMode;
  hasSeenOnboarding: boolean;
  secureLocalModeEnabled: boolean;
  selectedLocalTranscriptionModelName: string;
  hasAutoSelectedFastLocalModel: boolean;
  prerollEnabled: boolean;
}
export interface TranscriptionSettings {
  language: string;
}
export interface TextImprovementSettings {
  systemPrompt: string;
  customTerms: string[];
  context: string;
  tone: TextTone;
  customName: string;
}
export interface DampfAblassenSettings {
  systemPrompt: string;
  customName: string;
}
export interface EmojiTextSettings {
  emojiDensity: EmojiDensity;
  customName: string;
}
export interface GatewaySettings {
  baseUrl: string;
  correctionModel: string;
  fastModel: string;
  strongModel: string;
  sttBaseUrl: string;
  sttModel: string;
}
export interface WindowsSettings {
  launchAtLogin: boolean;
  hotkeys: Record<string, string>;
  gateway: GatewaySettings;
  pasteShortcut: PasteShortcutMode;
}
export interface SettingsContainer {
  app: AppSettings;
  transcription: TranscriptionSettings;
  textImprovement: TextImprovementSettings;
  dampfAblassen: DampfAblassenSettings;
  emojiText: EmojiTextSettings;
  windows: WindowsSettings;
}

export interface CredentialStatus {
  llmConfigured: boolean;
  llmMasked: string;
  sttConfigured: boolean;
  sttMasked: string;
}

export interface ModelMeta {
  name: string;
  displayName: string;
  installed: boolean;
  approxMb: number;
}

export interface StatusEvent {
  phase: "idle" | "recording" | "processing" | "done" | "error";
  workflow: string | null;
  message: string | null;
}

export const WORKFLOWS: { key: string; name: string; subtitle: string }[] = [
  { key: "transcription", name: "Blitztext", subtitle: "Sprache rein. Text raus." },
  { key: "localTranscription", name: "Blitztext Lokal", subtitle: "Nur lokal. Kein Server." },
  { key: "textImprover", name: "Blitztext+", subtitle: "Geschrieben sprechen." },
  { key: "dampfAblassen", name: "Blitztext $%&!", subtitle: "Frust rein. Entspannt raus." },
  { key: "emojiText", name: "Blitztext :)", subtitle: "Text rein. Emojis dazu." },
];

export const api = {
  getSettings: () => invoke<SettingsContainer>("get_settings"),
  saveSettings: (newSettings: SettingsContainer) =>
    invoke<void>("save_settings", { newSettings }),
  credentialStatus: () => invoke<CredentialStatus>("credential_status"),
  setCredential: (key: string, value: string) =>
    invoke<void>("set_credential", { key, value }),
  testLlmConnection: () => invoke<string>("test_llm_connection"),
  listModels: () => invoke<ModelMeta[]>("list_models"),
  downloadModel: (name: string) => invoke<void>("download_model", { name }),
  getAutostart: () => invoke<boolean>("get_autostart"),
  startRecording: (workflow: string) => invoke<void>("start_recording", { workflow }),
  stopRecording: () => invoke<void>("stop_recording"),
  cancel: () => invoke<void>("cancel"),
  audioLevel: () => invoke<number>("audio_level"),
  setHotkeyCapture: (active: boolean) => invoke<void>("set_hotkey_capture", { active }),
  hideWindow: () => invoke<void>("hide_window"),
  setActive: (active: boolean) => invoke<void>("set_active", { active }),
  getActive: () => invoke<boolean>("get_active"),
};
