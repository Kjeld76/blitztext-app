//! Sichere Zugangsdaten über den Windows Credential Manager (`keyring`-Crate).
//! Ersetzt `BlitztextMac/Services/KeychainService.swift` (macOS Keychain).

use keyring::Entry;

const SERVICE: &str = "de.blitztext.win";

/// Bekannte Schlüssel-Namen.
pub mod keys {
    /// Token/Key für den OpenAI-kompatiblen LLM-Gateway (z. B. auth2api).
    pub const LLM_GATEWAY_TOKEN: &str = "llm_gateway_token";
    /// Optionaler API-Key für einen Online-STT-Endpunkt.
    pub const STT_API_KEY: &str = "stt_api_key";
}

fn entry(key: &str) -> Result<Entry, String> {
    Entry::new(SERVICE, key).map_err(|e| e.to_string())
}

/// Speichert einen Wert. Leerer Wert => löschen.
pub fn set(key: &str, value: &str) -> Result<(), String> {
    let e = entry(key)?;
    if value.is_empty() {
        // Nicht vorhanden ist ok.
        match e.delete_credential() {
            Ok(_) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(err) => Err(err.to_string()),
        }
    } else {
        e.set_password(value).map_err(|e| e.to_string())
    }
}

/// Lädt einen Wert (None, wenn nicht gesetzt).
pub fn get(key: &str) -> Option<String> {
    let e = entry(key).ok()?;
    match e.get_password() {
        Ok(v) if !v.is_empty() => Some(v),
        _ => None,
    }
}

/// Ist ein nicht-leerer Wert hinterlegt?
pub fn has(key: &str) -> bool {
    get(key).is_some()
}

/// Maskierte Anzeige (analog AppState.apiKeyDisplayValue).
pub fn masked(key: &str) -> String {
    match get(key) {
        Some(v) if v.chars().count() > 8 => {
            let prefix: String = v.chars().take(4).collect();
            format!("{prefix} ••••••••")
        }
        Some(_) => "••••••••".to_string(),
        None => String::new(),
    }
}
