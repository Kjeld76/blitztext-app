//! OpenAI-kompatibler Chat-Client für den LLM-Gateway (auth2api/Ollama/OpenAI …).
//! Portiert aus `BlitztextMac/Services/LLMService.swift`, aber mit
//! konfigurierbarer `base_url` + Modell + Token statt hartem OpenAI-Endpunkt.

use crate::credentials;
use crate::settings::GatewaySettings;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize)]
struct ChatMessage {
    role: &'static str,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f64,
}

#[derive(Deserialize)]
struct ChoiceMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct Choice {
    message: Option<ChoiceMessage>,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Option<Vec<Choice>>,
}

#[derive(Deserialize)]
struct ApiErrorBody {
    error: Option<ApiErrorDetail>,
}

#[derive(Deserialize)]
struct ApiErrorDetail {
    message: Option<String>,
}

fn chat_url(base_url: &str) -> String {
    format!("{}/chat/completions", base_url.trim_end_matches('/'))
}

/// Ein Chat-Completion-Aufruf gegen den Gateway.
pub async fn complete(
    gateway: &GatewaySettings,
    model: &str,
    system_prompt: &str,
    user_text: &str,
    temperature: f64,
) -> Result<String, String> {
    let payload = ChatRequest {
        model: model.to_string(),
        messages: vec![
            ChatMessage {
                role: "system",
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: "user",
                content: user_text.to_string(),
            },
        ],
        temperature,
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Client-Fehler: {e}"))?;

    let mut req = client
        .post(chat_url(&gateway.base_url))
        .json(&payload)
        .header("Accept", "application/json");

    // Token optional — viele lokale Gateways (auth2api, Ollama) brauchen keinen.
    if let Some(token) = credentials::get(credentials::keys::LLM_GATEWAY_TOKEN) {
        req = req.header("Authorization", format!("Bearer {token}"));
    }

    let resp = req.send().await.map_err(|e| {
        format!(
            "Verbindungsproblem zum LLM-Gateway ({}). Läuft auth2api/Docker? Details: {e}",
            gateway.base_url
        )
    })?;

    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();

    if !status.is_success() {
        let msg = serde_json::from_str::<ApiErrorBody>(&body)
            .ok()
            .and_then(|b| b.error)
            .and_then(|e| e.message)
            .unwrap_or_else(|| format!("Status {}", status.as_u16()));
        return Err(format!("Fehler vom LLM-Gateway: {msg}"));
    }

    let parsed: ChatResponse =
        serde_json::from_str(&body).map_err(|e| format!("Antwort nicht lesbar: {e}"))?;

    let content = parsed
        .choices
        .and_then(|c| c.into_iter().next())
        .and_then(|c| c.message)
        .and_then(|m| m.content)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "Keine Antwort erhalten. Bitte nochmal versuchen.".to_string())?;

    Ok(content)
}

/// Schneller Verbindungstest (für den „Verbindung testen"-Button).
pub async fn test_connection(gateway: &GatewaySettings) -> Result<String, String> {
    let reply = complete(
        gateway,
        &gateway.fast_model,
        "Antworte mit genau einem Wort: OK",
        "Sag OK",
        0.0,
    )
    .await?;
    Ok(reply)
}
