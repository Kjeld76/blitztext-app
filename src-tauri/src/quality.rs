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
