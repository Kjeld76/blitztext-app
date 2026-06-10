//! Sprech-Overlay: kleines, rahmenloses Fenster oben-zentral am oberen
//! Bildschirmrand. Erscheint, sobald das Mikrofon wirklich aufnimmt
//! (phase == "recording"), und verschwindet sonst.
//!
//! Position oben statt unten, weil das Windows-Lautstärke-Flyout (ausgelöst
//! beim Aktivieren mancher Headsets, z. B. Yealink-DECT) unten-zentral
//! erscheint und das Overlay sonst verdecken würde.
//!
//! Wichtig: Das Fenster darf NIE den Fokus stehlen, sonst landet das
//! anschließende Einfügen im Overlay statt in der Ziel-App. Daher wird unter
//! Windows `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW` gesetzt.

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

const LABEL: &str = "overlay";
const WIDTH: f64 = 264.0;
const HEIGHT: f64 = 60.0;

/// Erstellt das Overlay-Fenster (versteckt) beim App-Start.
pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let window = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("overlay.html".into()))
        .title("Blitztext Overlay")
        .inner_size(WIDTH, HEIGHT)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .visible(false)
        .build()?;

    #[cfg(windows)]
    apply_no_activate(&window);

    Ok(())
}

/// Zeigt/versteckt das Overlay passend zur Aufnahme-Phase.
pub fn update(app: &AppHandle, phase: &str) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };
    if phase == "recording" {
        position(&window);
        if let Err(e) = window.show() {
            eprintln!("Overlay konnte nicht angezeigt werden: {e}");
        }
    } else if let Err(e) = window.hide() {
        eprintln!("Overlay konnte nicht versteckt werden: {e}");
    }
}

/// Zentriert das Overlay horizontal und setzt es knapp unter den oberen
/// Rand des Arbeitsbereichs (Hauptbildschirm).
fn position(window: &tauri::WebviewWindow) {
    let size = window
        .outer_size()
        .unwrap_or(tauri::PhysicalSize::new(WIDTH as u32, HEIGHT as u32));
    let scale = window.scale_factor().unwrap_or(1.0);
    let margin = (12.0 * scale) as i32;

    #[cfg(windows)]
    if let Some(wa) = work_area() {
        let x = wa.left + ((wa.right - wa.left) - size.width as i32) / 2;
        let y = wa.top + margin;
        let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
        return;
    }

    // Fallback (Nicht-Windows / kein Arbeitsbereich): Monitor-Maße nutzen.
    if let Ok(Some(monitor)) = window.current_monitor() {
        let m_pos = monitor.position();
        let m_size = monitor.size();
        let x = m_pos.x + ((m_size.width as i32) - size.width as i32) / 2;
        let y = m_pos.y + margin;
        let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
    }
}

/// Arbeitsbereich des primären Bildschirms (ohne Taskleiste), physische Pixel.
#[cfg(windows)]
fn work_area() -> Option<windows::Win32::Foundation::RECT> {
    use windows::Win32::Foundation::RECT;
    use windows::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPI_GETWORKAREA, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
    };
    let mut rect = RECT::default();
    let ok = unsafe {
        SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut rect as *mut _ as *mut core::ffi::c_void),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    };
    ok.ok().map(|_| rect)
}

/// Verhindert Fokus-Diebstahl und blendet das Fenster aus Alt-Tab/Taskleiste aus.
#[cfg(windows)]
fn apply_no_activate(window: &tauri::WebviewWindow) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    };
    if let Ok(raw) = window.hwnd() {
        // Tauris `windows`-Version kann von unserer abweichen → rohen Wert
        // entpacken und in unseren HWND verpacken.
        let hwnd = HWND(raw.0 as *mut core::ffi::c_void);
        unsafe {
            let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            let extra = (WS_EX_NOACTIVATE.0 as isize) | (WS_EX_TOOLWINDOW.0 as isize);
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex | extra);
        }
    }
}
