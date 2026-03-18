use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use crate::modules::os_integration::OSIntegration;
use crate::modules::inference::SensitiveTranscript;

const OLLAMA_URL: &str = "http://localhost:11434/api/generate";
const MODEL: &str = "llama3.2:3b";

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    system: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

pub struct ContextEngine;

impl ContextEngine {
    // Security: Validate Localhost
    fn validate_endpoint() -> Result<()> {
        if !OLLAMA_URL.starts_with("http://127.0.0.1") && !OLLAMA_URL.starts_with("http://localhost") {
            return Err(anyhow!("Security Violation: External API endpoints are strictly forbidden."));
        }
        Ok(())
    }

    pub fn determine_system_prompt() -> String {
        let app = OSIntegration::get_active_app_name();
        let base_prompt = "You are a text refinement engine. Correct grammar, remove filler words (uhm, ah), and fix punctuation.";

        match app.as_str() {
            "Cursor" | "Code" | "VSCodium" => {
                format!("{} You are in a CODE EDITOR. Format the output as a concise code comment or documentation string. Do not add markdown code blocks.", base_prompt)
            },
            "WhatsApp" | "Telegram" | "Discord" | "Signal" => {
                format!("{} You are in a CHAT APP. Keep the tone casual, use appropriate emojis, and keep it short.", base_prompt)
            },
            _ => format!("{} Return ONLY the corrected text without preamble.", base_prompt),
        }
    }

    pub async fn refine_text(transcript: &SensitiveTranscript) -> Result<String> {
        Self::validate_endpoint()?;
        
        let system_prompt = Self::determine_system_prompt();
        let client = reqwest::Client::new();
        
        let res = client.post(OLLAMA_URL)
            .json(&OllamaRequest {
                model: MODEL.to_string(),
                prompt: transcript.as_str().to_string(),
                system: system_prompt,
                stream: false,
            })
            .send()
            .await?
            .json::<OllamaResponse>()
            .await?;

        Ok(res.response.trim().to_string())
    }
}