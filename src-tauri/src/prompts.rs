//! LLM-System-Prompts — wörtlich portiert aus
//! `BlitztextMac/Services/LLMService.swift:159-208`.

use crate::settings::{EmojiDensity, TextImprovementSettings, TextTone};

/// Leichte Korrektur eines lokalen Whisper-Transkripts (Ctrl+Alt+B).
/// Nur Fehlerkorrektur — KEIN inhaltliches Umschreiben.
pub fn build_correction_system_prompt(custom_terms: &[String]) -> String {
    let mut prompt = String::from(
        "Du erhaeltst ein automatisch erzeugtes Sprach-Transkript. Korrigiere ausschliesslich \
         offensichtliche Erkennungsfehler, Zeichensetzung sowie Gross- und Kleinschreibung. \
         Aendere NICHT die Wortwahl, den Stil, den Satzbau oder den Inhalt; fuege nichts hinzu \
         und lasse nichts weg. Gib NUR den korrigierten Text zurueck, ohne Erklaerungen.",
    );
    if !custom_terms.is_empty() {
        prompt.push_str(&format!(
            "\n\nAchte besonders auf die korrekte Schreibweise dieser Eigennamen und Fachbegriffe: {}",
            custom_terms.join(", ")
        ));
    }
    prompt
}

/// Emoji-Dichte-Prompt (LLMService.swift:159-171)
pub fn build_emoji_system_prompt(density: EmojiDensity) -> String {
    let density_instruction = match density {
        EmojiDensity::Wenig => "Setze nur vereinzelt Emojis ein, maximal 1-2 pro Absatz.",
        EmojiDensity::Mittel => {
            "Setze regelmaessig passende Emojis ein, etwa alle 1-2 Saetze."
        }
        EmojiDensity::Viel => "Setze grosszuegig Emojis ein, gerne mehrere pro Satz.",
    };

    format!(
        "Du erhaeltst ein gesprochenes Transkript. Gib den Text moeglichst originalgetreu zurueck, \
         aber fuege passende Emojis ein. {density_instruction} Korrigiere offensichtliche Sprach- \
         und Grammatikfehler. Behalte den Stil und die Bedeutung bei. Gib NUR den Text mit Emojis \
         zurueck, keine Erklaerungen."
    )
}

/// Textverbesserer-Prompt (LLMService.swift:173-208)
pub fn build_improvement_system_prompt(settings: &TextImprovementSettings) -> String {
    if !settings.system_prompt.is_empty() {
        let mut prompt = settings.system_prompt.clone();
        if !settings.custom_terms.is_empty() {
            prompt.push_str(&format!(
                "\n\nWichtig: Diese Eigennamen und Fachbegriffe muessen exakt so geschrieben \
                 werden: {}",
                settings.custom_terms.join(", ")
            ));
        }
        return prompt;
    }

    let mut prompt = String::from(
        "Du bist ein Lektor und Schreibassistent. Verbessere den folgenden Text:\n\
         - Korrigiere Rechtschreibung und Grammatik\n\
         - Verbessere die Formulierung und den Lesefluss\n\
         - Behalte die urspruengliche Bedeutung bei\n\
         - Gib NUR den verbesserten Text zurueck, keine Erklaerungen",
    );

    match settings.tone {
        TextTone::Formal => prompt.push_str("\n- Verwende einen formellen, professionellen Ton"),
        TextTone::Neutral => prompt.push_str("\n- Verwende einen neutralen, klaren Ton"),
        TextTone::Casual => prompt.push_str("\n- Verwende einen lockeren, natuerlichen Ton"),
    }

    if !settings.custom_terms.is_empty() {
        prompt.push_str(&format!(
            "\n\nWichtig: Diese Eigennamen und Fachbegriffe muessen exakt so geschrieben werden: {}",
            settings.custom_terms.join(", ")
        ));
    }

    if !settings.context.is_empty() {
        prompt.push_str(&format!("\n\nKontext: {}", settings.context));
    }

    prompt
}
