//! Transkriptions-Qualität — portiert aus
//! `BlitztextMac/Services/TranscriptionQualityService.swift`.

pub const MINIMUM_RECORDING_DURATION: f64 = 0.3;

pub fn should_reject_recording(duration: f64) -> bool {
    duration < MINIMUM_RECORDING_DURATION
}

pub fn cleaned_transcript(text: &str) -> String {
    text.trim().to_string()
}

/// Heuristik gegen Whisper-Halluzinationen bei (zu) kurzen Aufnahmen.
pub fn is_likely_artifact(text: &str, recording_duration: f64) -> bool {
    let cleaned = cleaned_transcript(text);
    if cleaned.is_empty() {
        return true;
    }

    let words = cleaned.split_whitespace().count();
    let letters = cleaned.chars().filter(|c| c.is_alphabetic()).count();
    let char_count = cleaned.chars().count();

    if letters == 0 {
        return true;
    }

    if recording_duration < 0.55 && (words >= 5 || char_count >= 32) {
        return true;
    }

    if recording_duration < 0.8 && char_count >= 56 {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zu_kurze_aufnahmen_werden_abgelehnt() {
        assert!(should_reject_recording(0.0));
        assert!(should_reject_recording(0.29));
        assert!(!should_reject_recording(MINIMUM_RECORDING_DURATION));
        assert!(!should_reject_recording(5.0));
    }

    #[test]
    fn cleaned_transcript_trimmt_nur() {
        assert_eq!(cleaned_transcript("  Hallo Welt \n"), "Hallo Welt");
        assert_eq!(cleaned_transcript("   \n\t"), "");
        // Innenliegender Whitespace bleibt unangetastet.
        assert_eq!(cleaned_transcript("a  b"), "a  b");
    }

    #[test]
    fn leerer_oder_buchstabenloser_text_ist_artefakt() {
        assert!(is_likely_artifact("", 3.0));
        assert!(is_likely_artifact("   ", 3.0));
        assert!(is_likely_artifact("... !!! 123", 3.0));
    }

    #[test]
    fn normaler_satz_bei_normaler_dauer_ist_kein_artefakt() {
        assert!(!is_likely_artifact(
            "Das ist ein ganz normaler diktierter Satz.",
            3.0
        ));
        // Kurze, plausible Aeusserung bei kurzer Aufnahme.
        assert!(!is_likely_artifact("Ja, genau.", 0.5));
    }

    #[test]
    fn viel_text_bei_sehr_kurzer_aufnahme_ist_artefakt() {
        // < 0,55 s: ab 5 Woertern bzw. 32 Zeichen unplausibel.
        assert!(is_likely_artifact("eins zwei drei vier fuenf", 0.54));
        assert!(is_likely_artifact(
            "Abcdefghijklmnopqrstuvwxyzabcdef",
            0.54
        ));
        // Gleiche Texte knapp ueber der Schwelle sind ok.
        assert!(!is_likely_artifact("eins zwei drei vier fuenf", 0.55));
        // 4 Woerter, < 32 Zeichen: auch bei sehr kurzer Aufnahme ok.
        assert!(!is_likely_artifact("eins zwei drei vier", 0.54));
    }

    #[test]
    fn sehr_viel_text_bei_kurzer_aufnahme_ist_artefakt() {
        let long = "Untertitel im Auftrag des ZDF fuer funk, zweitausendzwanzig."; // > 56 Zeichen
        assert!(long.chars().count() >= 56);
        assert!(is_likely_artifact(long, 0.79));
        assert!(!is_likely_artifact(long, 0.8));
    }
}
