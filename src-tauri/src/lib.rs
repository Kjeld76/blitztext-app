//! Blitztext für Windows — Tauri-Backend.

mod audio;
mod credentials;
mod llm;
mod paste;
mod prompts;
mod quality;
mod settings;
mod transcription;
mod workflows;

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Mutex;

use settings::{HotkeyMode, SettingsContainer, WorkflowType};
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_autostart::{ManagerExt, MacosLauncher};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use workflows::Engine;

// MARK: - Managed State

pub struct AppStateInner {
    pub settings: SettingsContainer,
}
pub type SharedState = Mutex<AppStateInner>;
pub type EngineState = Mutex<Engine>;
/// Hotkey-ID (Shortcut::id()) → Workflow.
pub type HotkeyMap = Mutex<HashMap<u32, WorkflowType>>;
pub type EscapeShortcut = Mutex<Option<Shortcut>>;
/// Globaler Aktiv-Schalter: false = pausiert (Hotkeys abgemeldet, App bleibt im Tray).
pub type ActiveState = Mutex<bool>;

// MARK: - Hotkey-Dispatch

fn dispatch_shortcut(app: &AppHandle, shortcut: &Shortcut, state: ShortcutState) {
    // Escape (nur während Toggle-Aufnahme registriert) → abbrechen.
    let esc = app.state::<EscapeShortcut>().lock().unwrap().clone();
    if let Some(esc) = esc {
        if shortcut.id() == esc.id() {
            if state == ShortcutState::Pressed {
                cancel_recording(app);
            }
            return;
        }
    }

    let workflow = app
        .state::<HotkeyMap>()
        .lock()
        .unwrap()
        .get(&shortcut.id())
        .copied();
    let Some(workflow) = workflow else { return };

    let mode = app.state::<SharedState>().lock().unwrap().settings.app.hotkey_mode;
    match mode {
        HotkeyMode::Hold => match state {
            ShortcutState::Pressed => {
                start_recording_internal(app, workflow);
            }
            ShortcutState::Released => {
                stop_recording_internal(app);
            }
        },
        HotkeyMode::Toggle => {
            if state == ShortcutState::Pressed {
                let busy = app.state::<EngineState>().lock().unwrap().is_busy();
                if busy {
                    stop_recording_internal(app);
                } else if start_recording_internal(app, workflow) {
                    register_escape(app);
                }
            }
        }
    }
}

/// Startet die Aufnahme; gibt true zurück, wenn tatsächlich gestartet wurde.
fn start_recording_internal(app: &AppHandle, workflow: WorkflowType) -> bool {
    let engine = app.state::<EngineState>();
    let started = engine.lock().unwrap().begin(workflow).is_ok();
    if started {
        workflows::set_status(app, "recording", Some(workflow), None);
    }
    started
}

fn stop_recording_internal(app: &AppHandle) {
    let engine = app.state::<EngineState>();
    let res = engine.lock().unwrap().end();
    let (workflow, recording) = match res {
        Ok(x) => x,
        Err(_) => return,
    };
    unregister_escape(app);
    let settings = app.state::<SharedState>().lock().unwrap().settings.clone();
    workflows::set_status(app, "processing", Some(workflow), Some("Wird transkribiert …".into()));
    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        workflows::run_pipeline(app2, workflow, recording, settings).await;
    });
}

fn cancel_recording(app: &AppHandle) {
    app.state::<EngineState>().lock().unwrap().cancel();
    unregister_escape(app);
    workflows::set_status(app, "idle", None, None);
}

fn register_escape(app: &AppHandle) {
    let esc = app.state::<EscapeShortcut>().lock().unwrap().clone();
    if let Some(esc) = esc {
        let _ = app.global_shortcut().register(esc);
    }
}

fn unregister_escape(app: &AppHandle) {
    let esc = app.state::<EscapeShortcut>().lock().unwrap().clone();
    if let Some(esc) = esc {
        let _ = app.global_shortcut().unregister(esc);
    }
}

/// Registriert alle konfigurierten Hotkeys neu (nur wenn aktiv; sonst alles abmelden).
fn apply_hotkeys(app: &AppHandle) {
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();

    let map_state = app.state::<HotkeyMap>();
    let mut map = map_state.lock().unwrap();
    map.clear();

    // Pausiert? Dann keine Hotkeys registrieren.
    if !*app.state::<ActiveState>().lock().unwrap() {
        *app.state::<EscapeShortcut>().lock().unwrap() = None;
        return;
    }

    let settings = app.state::<SharedState>().lock().unwrap().settings.clone();

    for workflow in WorkflowType::ALL {
        let s = settings.hotkey_for(workflow);
        match Shortcut::from_str(&s) {
            Ok(sc) => {
                if gs.register(sc).is_ok() {
                    map.insert(sc.id(), workflow);
                } else {
                    eprintln!("Hotkey konnte nicht registriert werden: {s}");
                }
            }
            Err(_) => eprintln!("Ungültiger Hotkey: {s}"),
        }
    }

    *app.state::<EscapeShortcut>().lock().unwrap() = Shortcut::from_str("Escape").ok();
}

// MARK: - Commands: Settings

#[tauri::command]
fn get_settings(state: tauri::State<'_, SharedState>) -> SettingsContainer {
    state.lock().unwrap().settings.clone()
}

#[tauri::command]
fn save_settings(app: AppHandle, new_settings: SettingsContainer) -> Result<(), String> {
    settings::save(&new_settings).map_err(|e| e.to_string())?;
    let launch = new_settings.windows.launch_at_login;
    app.state::<SharedState>().lock().unwrap().settings = new_settings;
    apply_hotkeys(&app);
    apply_autostart(&app, launch);
    apply_prewarm(&app);
    Ok(())
}

// MARK: - Commands: Credentials

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct CredentialStatus {
    llm_configured: bool,
    llm_masked: String,
    stt_configured: bool,
    stt_masked: String,
}

#[tauri::command]
fn credential_status() -> CredentialStatus {
    CredentialStatus {
        llm_configured: credentials::has(credentials::keys::LLM_GATEWAY_TOKEN),
        llm_masked: credentials::masked(credentials::keys::LLM_GATEWAY_TOKEN),
        stt_configured: credentials::has(credentials::keys::STT_API_KEY),
        stt_masked: credentials::masked(credentials::keys::STT_API_KEY),
    }
}

#[tauri::command]
fn set_credential(key: String, value: String) -> Result<(), String> {
    match key.as_str() {
        credentials::keys::LLM_GATEWAY_TOKEN | credentials::keys::STT_API_KEY => {
            credentials::set(&key, &value)
        }
        _ => Err(format!("Unbekannter Schlüssel: {key}")),
    }
}

// MARK: - Commands: LLM-Gateway

#[tauri::command]
async fn test_llm_connection(state: tauri::State<'_, SharedState>) -> Result<String, String> {
    let gateway = state.lock().unwrap().settings.windows.gateway.clone();
    llm::test_connection(&gateway).await
}

// MARK: - Commands: Modelle

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelMeta {
    name: String,
    display_name: String,
    installed: bool,
    approx_mb: u32,
}

#[tauri::command]
fn list_models() -> Vec<ModelMeta> {
    transcription::MODELS
        .iter()
        .map(|m| ModelMeta {
            name: m.name.to_string(),
            display_name: m.display_name.to_string(),
            installed: transcription::is_installed(m.name),
            approx_mb: m.approx_mb,
        })
        .collect()
}

#[tauri::command]
async fn download_model(app: AppHandle, name: String) -> Result<(), String> {
    let name_for_event = name.clone();
    let app2 = app.clone();
    let result = transcription::download_model(&name, move |p| {
        let _ = app2.emit(
            "model-progress",
            serde_json::json!({ "name": name_for_event, "progress": p }),
        );
    })
    .await;
    let installed = transcription::is_installed(&name);
    let _ = app.emit(
        "model-progress",
        serde_json::json!({ "name": name, "progress": 1.0, "done": installed, "error": result.as_ref().err() }),
    );
    result
}

// MARK: - Commands: Autostart

fn apply_autostart(app: &AppHandle, enabled: bool) {
    let al = app.autolaunch();
    let _ = if enabled { al.enable() } else { al.disable() };
}

#[tauri::command]
fn get_autostart(app: AppHandle) -> bool {
    app.autolaunch().is_enabled().unwrap_or(false)
}

// MARK: - Commands: manuelle Steuerung + Status

#[tauri::command]
fn start_recording(app: AppHandle, workflow: String) -> Result<(), String> {
    let wf = WorkflowType::ALL
        .into_iter()
        .find(|w| w.key() == workflow)
        .ok_or_else(|| format!("Unbekannter Workflow: {workflow}"))?;
    start_recording_internal(&app, wf);
    Ok(())
}

#[tauri::command]
fn stop_recording(app: AppHandle) {
    stop_recording_internal(&app);
}

#[tauri::command]
fn cancel(app: AppHandle) {
    cancel_recording(&app);
}

#[tauri::command]
fn audio_level(state: tauri::State<'_, EngineState>) -> f32 {
    state.lock().unwrap().level()
}

/// Während der Hotkey-Erfassung im UI die globalen Shortcuts pausieren,
/// damit die Tastenkombination das Eingabefeld erreicht (sonst fängt sie
/// der systemweite Hotkey ab). `active=false` registriert sie neu.
#[tauri::command]
fn set_hotkey_capture(app: AppHandle, active: bool) {
    if active {
        let _ = app.global_shortcut().unregister_all();
    } else {
        apply_hotkeys(&app);
    }
}

// MARK: - Fenster-Helfer

fn show_main_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}

fn toggle_main_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        if win.is_visible().unwrap_or(false) {
            let _ = win.hide();
        } else {
            let _ = win.show();
            let _ = win.set_focus();
        }
    }
}

/// Fenster ins Tray verstecken (vom „Schließen"-Button im UI aufgerufen).
#[tauri::command]
fn hide_window(app: AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.hide();
    }
}

/// Tray-Icon-Varianten: farbig (aktiv) und Graustufen (pausiert).
struct TrayIcons {
    active: Image<'static>,
    paused: Image<'static>,
}

/// Erzeugt eine Graustufen-Variante (Alpha bleibt erhalten).
fn to_grayscale(img: &Image) -> Image<'static> {
    let (w, h) = (img.width(), img.height());
    let mut data = img.rgba().to_vec();
    for px in data.chunks_mut(4) {
        if px.len() == 4 {
            let y = ((px[0] as u32 * 299 + px[1] as u32 * 587 + px[2] as u32 * 114) / 1000) as u8;
            px[0] = y;
            px[1] = y;
            px[2] = y;
        }
    }
    Image::new_owned(data, w, h)
}

/// Aktiv/Pausiert umschalten (App bleibt im Tray laufen).
fn set_active_state(app: &AppHandle, active: bool) {
    *app.state::<ActiveState>().lock().unwrap() = active;
    if !active {
        app.state::<EngineState>().lock().unwrap().cancel();
    }
    apply_hotkeys(app); // registriert oder meldet ab, je nach Aktiv-Zustand
    if let Some(tray) = app.tray_by_id("main") {
        if let Some(icons) = app.try_state::<TrayIcons>() {
            let img = if active {
                icons.active.clone()
            } else {
                icons.paused.clone()
            };
            let _ = tray.set_icon(Some(img));
        }
        let _ = tray.set_tooltip(Some(if active {
            "Blitztext"
        } else {
            "Blitztext – pausiert"
        }));
    }
    let _ = app.emit("active", active);
    apply_prewarm(app);
    if active {
        workflows::set_status(app, "idle", None, None);
    }
}

/// Warmes Mikrofon (Pre-Roll) gemäß Einstellung + Aktiv-Zustand schalten.
fn apply_prewarm(app: &AppHandle) {
    let enabled = app
        .state::<SharedState>()
        .lock()
        .unwrap()
        .settings
        .app
        .preroll_enabled;
    let active = *app.state::<ActiveState>().lock().unwrap();
    if let Some(engine) = app.try_state::<EngineState>() {
        engine.lock().unwrap().set_prewarm(enabled && active);
    }
}

#[tauri::command]
fn set_active(app: AppHandle, active: bool) {
    set_active_state(&app, active);
}

#[tauri::command]
fn get_active(state: tauri::State<'_, ActiveState>) -> bool {
    *state.lock().unwrap()
}

// MARK: - App-Setup

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let initial = settings::load();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    dispatch_shortcut(app, shortcut, event.state);
                })
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .manage(Mutex::new(AppStateInner { settings: initial }))
        .manage(Mutex::new(Engine::default()))
        .manage::<HotkeyMap>(Mutex::new(HashMap::<u32, WorkflowType>::new()))
        .manage::<EscapeShortcut>(Mutex::new(None::<Shortcut>))
        .manage::<ActiveState>(Mutex::new(true))
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            credential_status,
            set_credential,
            test_llm_connection,
            list_models,
            download_model,
            get_autostart,
            start_recording,
            stop_recording,
            cancel,
            audio_level,
            set_hotkey_capture,
            hide_window,
            set_active,
            get_active,
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            // Tray-Menü
            let open = MenuItem::with_id(app, "open", "Öffnen", true, None::<&str>)?;
            let settings_item =
                MenuItem::with_id(app, "settings", "Einstellungen", true, None::<&str>)?;
            let toggle_active =
                MenuItem::with_id(app, "toggle_active", "Pausieren / Aktivieren", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Beenden", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &settings_item, &toggle_active, &quit])?;

            let _tray = TrayIconBuilder::with_id("main")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Blitztext")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show_main_window(app),
                    "settings" => {
                        show_main_window(app);
                        let _ = app.emit("navigate", "settings");
                    }
                    "toggle_active" => {
                        let cur = *app.state::<ActiveState>().lock().unwrap();
                        set_active_state(app, !cur);
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_main_window(tray.app_handle());
                    }
                })
                .build(app)?;

            // Tray-Icon-Varianten (farbig = aktiv, Graustufen = pausiert) vorbereiten.
            if let Some(base) = app.default_window_icon() {
                let active_icon =
                    Image::new_owned(base.rgba().to_vec(), base.width(), base.height());
                let paused_icon = to_grayscale(base);
                app.manage(TrayIcons {
                    active: active_icon,
                    paused: paused_icon,
                });
            }

            // Pre-Roll (warmes Mikrofon) gemäß Einstellung starten.
            apply_prewarm(&handle);

            // Hotkeys registrieren
            apply_hotkeys(&handle);

            // Autostart-Status mit Settings synchronisieren
            let launch = handle
                .state::<SharedState>()
                .lock()
                .unwrap()
                .settings
                .windows
                .launch_at_login;
            apply_autostart(&handle, launch);

            // Lokales Whisper-Modell im Hintergrund vorwärmen (falls vorhanden)
            let model = handle
                .state::<SharedState>()
                .lock()
                .unwrap()
                .settings
                .app
                .selected_local_transcription_model_name
                .clone();
            if transcription::is_installed(&model) {
                std::thread::spawn(move || {
                    let _ = transcription::prewarm(&model);
                });
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            // Schließen versteckt nur (Tray-App bleibt aktiv).
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
