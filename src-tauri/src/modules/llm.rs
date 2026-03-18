use crate::modules::inference::SensitiveTranscript;
use crate::modules::os_integration::OSIntegration;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Clone)]
pub enum AppMode {
    Coding,
    Chat,
    Browser,
    Terminal,
    Default,
}

#[derive(Debug, Serialize, Clone)]
pub struct ContextInfo {
    pub app_name: String,
    pub mode: AppMode,
    pub system_prompt: String,
}

#[derive(Debug, Serialize, Clone)]
pub enum Command {
    Delete,
    Bold,
    Italic,
    SelectAll,
    Enter,
}

pub struct ContextEngine;

impl ContextEngine {
    // Security: Validate Localhost
    fn validate_endpoint() -> Result<()> {
        if !OLLAMA_URL.starts_with("http://127.0.0.1")
            && !OLLAMA_URL.starts_with("http://localhost")
        {
            return Err(anyhow!(
                "Security Violation: External API endpoints are strictly forbidden."
            ));
        }
        Ok(())
    }

    pub fn get_context() -> ContextInfo {
        let app_name = OSIntegration::get_active_app_name();
        // Normalize for matching
        let app_lower = app_name.to_lowercase();

        let base_prompt = "You are a text refinement engine. Correct grammar, remove filler words (uhm, ah), and fix punctuation.";

        let (mode, prompt_suffix) = if app_lower.contains("code")
            || app_lower.contains("cursor")
            || app_lower.contains("intellij")
            || app_lower.contains("rustrover")
            || app_lower.contains("vim")
            || app_lower.contains("neovim")
        {
            (AppMode::Coding, "You are in a CODE EDITOR. Format the output as a concise code comment or documentation string. Do not add markdown code blocks.")
        } else if app_lower.contains("whatsapp")
            || app_lower.contains("telegram")
            || app_lower.contains("discord")
            || app_lower.contains("slack")
            || app_lower.contains("signal")
        {
            (AppMode::Chat, "You are in a CHAT APP. Keep the tone casual, use appropriate emojis, and keep it short.")
        } else if app_lower.contains("chrome")
            || app_lower.contains("edge")
            || app_lower.contains("firefox")
            || app_lower.contains("brave")
            || app_lower.contains("arc")
        {
            (
                AppMode::Browser,
                "You are in a WEB BROWSER. Format as clear, searchable text or a summary.",
            )
        } else if app_lower.contains("terminal")
            || app_lower.contains("powershell")
            || app_lower.contains("cmd")
            || app_lower.contains("wezterm")
            || app_lower.contains("alacritty")
        {
            (AppMode::Terminal, "You are in a TERMINAL. Format the output as a shell command or a concise explanation. Do not wrap in markdown blocks if it's a command.")
        } else {
            (
                AppMode::Default,
                "Return ONLY the corrected text without preamble.",
            )
        };

        ContextInfo {
            app_name,
            mode,
            system_prompt: format!("{} {}", base_prompt, prompt_suffix),
        }
    }

    pub async fn refine_text(transcript: &SensitiveTranscript) -> Result<(String, Option<Command>)> {
        if let Err(_) = Self::validate_endpoint() {
            return Ok((transcript.as_str().to_string(), None));
        }

        let context = Self::get_context();
        let client = reqwest::Client::new();

        // Attempt Ollama with short timeout
        let res = client
            .post(OLLAMA_URL)
            .timeout(std::time::Duration::from_secs(2))
            .json(&OllamaRequest {
                model: MODEL.to_string(),
                prompt: transcript.as_str().to_string(),
                system: format!("{} If the user says 'delete that', 'bold that', 'italicize that', 'select all', or 'press enter', return ONLY the keyword: [DELETE], [BOLD], [ITALIC], [SELECT_ALL], or [ENTER]. Otherwise, just return the corrected text.", context.system_prompt),
                stream: false,
            })
            .send()
            .await;

        match res {
            Ok(response) => {
                if let Ok(data) = response.json::<OllamaResponse>().await {
                    let text = data.response.trim();
                    if text == "[DELETE]" { return Ok(("".to_string(), Some(Command::Delete))); }
                    if text == "[BOLD]" { return Ok(("".to_string(), Some(Command::Bold))); }
                    if text == "[ITALIC]" { return Ok(("".to_string(), Some(Command::Italic))); }
                    if text == "[SELECT_ALL]" { return Ok(("".to_string(), Some(Command::SelectAll))); }
                    if text == "[ENTER]" { return Ok(("".to_string(), Some(Command::Enter))); }
                    return Ok((text.to_string(), None));
                }
            }
            Err(_) => {
                println!("[WARNING] Ollama not available, returning raw transcript.");
            }
        }

        Ok((transcript.as_str().to_string(), None))
    }
}
