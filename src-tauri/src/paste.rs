//! Ergebnis in die Zwischenablage legen und ins fokussierte Fenster einfügen.
//! Ersetzt `AppState.performPaste` (CGEvent Cmd+V) durch Clipboard + enigo Strg+V.
//! Unter Windows ist dafür keine Sonderberechtigung nötig.
//!
//! Viele Terminals (z. B. Windows Terminal) mappen Strg+V nicht aufs Einfügen —
//! dort landet der Text sonst nur in der Zwischenablage. Im Modus `Auto` wird
//! das Vordergrundfenster geprüft und in erkannten Terminals stattdessen
//! Strg+Umschalt+V gesendet. Klassisches conhost bleibt bewusst bei Strg+V
//! (versteht kein Strg+Umschalt+V; Strg+V funktioniert dort standardmäßig).

use crate::settings::PasteShortcutMode;
use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

/// Prozessnamen bekannter Terminals, die Strg+Umschalt+V zum Einfügen nutzen.
#[cfg(windows)]
const TERMINAL_PROCESSES: [&str; 6] = [
    "windowsterminal.exe",
    "openconsole.exe",
    "wezterm-gui.exe",
    "alacritty.exe",
    "mintty.exe",
    "hyper.exe",
];

/// Fensterklasse des Windows Terminal.
#[cfg(windows)]
const TERMINAL_WINDOW_CLASS: &str = "CASCADIA_HOSTING_WINDOW_CLASS";

/// Schreibt Text in die Zwischenablage (bleibt als Fallback erhalten).
pub fn set_clipboard(text: &str) -> Result<(), String> {
    let mut cb = Clipboard::new().map_err(|e| format!("Zwischenablage-Fehler: {e}"))?;
    cb.set_text(text.to_string())
        .map_err(|e| format!("Zwischenablage-Fehler: {e}"))
}

/// Legt den Text in die Zwischenablage und simuliert anschließend das
/// Einfüge-Kürzel (Strg+V bzw. Strg+Umschalt+V, siehe `PasteShortcutMode`).
/// Blockierend (enigo/Stream sind nicht `Send`) — via spawn_blocking aufrufen.
pub fn paste_text(text: &str, mode: PasteShortcutMode) -> Result<(), String> {
    set_clipboard(text)?;
    // Kurze Verzögerung, damit der Fokus sicher beim Zielfenster liegt.
    std::thread::sleep(std::time::Duration::from_millis(120));

    let with_shift = match mode {
        PasteShortcutMode::CtrlV => false,
        PasteShortcutMode::CtrlShiftV => true,
        PasteShortcutMode::Auto => foreground_is_terminal(),
    };

    let mut enigo =
        Enigo::new(&Settings::default()).map_err(|e| format!("Eingabe-Simulation-Fehler: {e}"))?;
    enigo
        .key(Key::Control, Direction::Press)
        .map_err(|e| format!("Tastendruck-Fehler: {e}"))?;
    if with_shift {
        enigo
            .key(Key::Shift, Direction::Press)
            .map_err(|e| format!("Tastendruck-Fehler: {e}"))?;
    }
    enigo
        .key(Key::Unicode('v'), Direction::Click)
        .map_err(|e| format!("Tastendruck-Fehler: {e}"))?;
    if with_shift {
        enigo
            .key(Key::Shift, Direction::Release)
            .map_err(|e| format!("Tastendruck-Fehler: {e}"))?;
    }
    enigo
        .key(Key::Control, Direction::Release)
        .map_err(|e| format!("Tastendruck-Fehler: {e}"))?;
    Ok(())
}

/// Prüft, ob das Vordergrundfenster zu einem bekannten Terminal gehört
/// (Fensterklasse oder Prozessname des Fenster-Prozesses).
#[cfg(windows)]
fn foreground_is_terminal() -> bool {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetClassNameW, GetForegroundWindow, GetWindowThreadProcessId,
    };

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return false;
        }

        // 1) Fensterklasse (Windows Terminal hat eine stabile eigene Klasse).
        let mut class_buf = [0u16; 256];
        let class_len = GetClassNameW(hwnd, &mut class_buf);
        if class_len > 0 {
            let class = String::from_utf16_lossy(&class_buf[..class_len as usize]);
            if class == TERMINAL_WINDOW_CLASS {
                return true;
            }
        }

        // 2) Prozessname des Fenster-Prozesses gegen die Allowlist.
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return false;
        }
        let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return false;
        };
        let mut name_buf = [0u16; 1024];
        let mut len = name_buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(name_buf.as_mut_ptr()),
            &mut len,
        );
        let _ = CloseHandle(process);
        if ok.is_err() {
            return false;
        }
        let path = String::from_utf16_lossy(&name_buf[..len as usize]);
        let exe = path
            .rsplit(['\\', '/'])
            .next()
            .unwrap_or(&path)
            .to_ascii_lowercase();
        TERMINAL_PROCESSES.contains(&exe.as_str())
    }
}

#[cfg(not(windows))]
fn foreground_is_terminal() -> bool {
    false
}
