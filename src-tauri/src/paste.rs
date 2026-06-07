//! Ergebnis in die Zwischenablage legen und ins fokussierte Fenster einfügen.
//! Ersetzt `AppState.performPaste` (CGEvent Cmd+V) durch Clipboard + enigo Strg+V.
//! Unter Windows ist dafür keine Sonderberechtigung nötig.

use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

/// Schreibt Text in die Zwischenablage (bleibt als Fallback erhalten).
pub fn set_clipboard(text: &str) -> Result<(), String> {
    let mut cb = Clipboard::new().map_err(|e| format!("Zwischenablage-Fehler: {e}"))?;
    cb.set_text(text.to_string())
        .map_err(|e| format!("Zwischenablage-Fehler: {e}"))
}

/// Legt den Text in die Zwischenablage und simuliert anschließend Strg+V.
/// Blockierend (enigo/Stream sind nicht `Send`) — via spawn_blocking aufrufen.
pub fn paste_text(text: &str) -> Result<(), String> {
    set_clipboard(text)?;
    // Kurze Verzögerung, damit der Fokus sicher beim Zielfenster liegt.
    std::thread::sleep(std::time::Duration::from_millis(120));

    let mut enigo =
        Enigo::new(&Settings::default()).map_err(|e| format!("Eingabe-Simulation-Fehler: {e}"))?;
    enigo
        .key(Key::Control, Direction::Press)
        .map_err(|e| format!("Tastendruck-Fehler: {e}"))?;
    enigo
        .key(Key::Unicode('v'), Direction::Click)
        .map_err(|e| format!("Tastendruck-Fehler: {e}"))?;
    enigo
        .key(Key::Control, Direction::Release)
        .map_err(|e| format!("Tastendruck-Fehler: {e}"))?;
    Ok(())
}
