#[cfg(target_os = "linux")]
use crate::modules::linux_paste::LinuxPaste;

use active_win_pos_rs::get_active_window;
use anyhow::Result;
use arboard::Clipboard;
use enigo::{Enigo, Key as EnigoKey, KeyboardControllable};
use std::env;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct OSIntegration;

// Mock Clipboard for Testing
lazy_static::lazy_static! {
    static ref MOCK_CLIPBOARD: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
}

impl OSIntegration {
    pub fn get_active_app_name() -> String {
        #[cfg(target_os = "linux")]
        {
            // On Linux, active_win_pos_rs can crash on Wayland.
            "Linux App".to_string()
        }
        #[cfg(not(target_os = "linux"))]
        {
            match get_active_window() {
                Ok(window) => window.app_name,
                Err(_) => "Unknown".to_string(),
            }
        }
    }

    pub fn paste_text(text: &str) -> Result<()> {
        // Runtime Mock Check
        if env::var("VIBEFLOW_TEST_MODE").is_ok() {
            *MOCK_CLIPBOARD.lock().unwrap() = text.to_string();
            return Ok(());
        }

        println!("[DEBUG] paste_text: {}", text);

        #[cfg(target_os = "linux")]
        {
            return LinuxPaste::paste_text(text).map_err(|e| {
                println!("[LINUX] Paste failed: {}", e);
                e
            });
        }

        #[cfg(target_os = "windows")]
        {
            // WINDOWS STRATEGY: Enigo (Reliable on Windows)
            let result = std::panic::catch_unwind(|| {
                let mut clipboard = Clipboard::new()
                    .map_err(|e| anyhow::anyhow!("Clipboard init failed: {}", e))?;
                let original_content = clipboard.get_text().unwrap_or_default();
                clipboard
                    .set_text(text.to_owned())
                    .map_err(|e| anyhow::anyhow!("Clipboard set failed: {}", e))?;
                thread::sleep(Duration::from_millis(200));

                let mut enigo = Enigo::new();
                enigo.key_down(EnigoKey::Control);
                enigo.key_click(EnigoKey::Layout('v'));
                enigo.key_up(EnigoKey::Control);

                thread::sleep(Duration::from_millis(200));
                let _ = clipboard.set_text(original_content);
                Ok::<(), anyhow::Error>(())
            });
            match result {
                Ok(inner) => inner,
                Err(_) => Err(anyhow::anyhow!("Paste operation panicked")),
            }
        }

        #[cfg(target_os = "macos")]
        {
            Ok(())
        }
    }

    pub fn execute_command(command: crate::modules::llm::Command) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            use crate::modules::llm::Command;
            let key_sequence = match command {
                Command::Delete => "ctrl+shift+Left BackSpace",
                Command::Bold => "ctrl+b",
                Command::Italic => "ctrl+i",
                Command::SelectAll => "ctrl+a",
                Command::Enter => "Return",
            };

            return LinuxPaste::execute_command(key_sequence);
        }

        #[cfg(target_os = "windows")]
        {
            // WINDOWS STRATEGY: Enigo
            let result = std::panic::catch_unwind(|| {
                let mut enigo = Enigo::new();
                match command {
                    crate::modules::llm::Command::Delete => {
                        enigo.key_down(EnigoKey::Control);
                        enigo.key_down(EnigoKey::Shift);
                        enigo.key_click(EnigoKey::LeftArrow);
                        enigo.key_up(EnigoKey::Shift);
                        enigo.key_up(EnigoKey::Control);
                        enigo.key_click(EnigoKey::Backspace);
                    }
                    crate::modules::llm::Command::Bold => {
                        enigo.key_down(EnigoKey::Control);
                        enigo.key_click(EnigoKey::Layout('b'));
                        enigo.key_up(EnigoKey::Control);
                    }
                    crate::modules::llm::Command::Italic => {
                        enigo.key_down(EnigoKey::Control);
                        enigo.key_click(EnigoKey::Layout('i'));
                        enigo.key_up(EnigoKey::Control);
                    }
                    crate::modules::llm::Command::SelectAll => {
                        enigo.key_down(EnigoKey::Control);
                        enigo.key_click(EnigoKey::Layout('a'));
                        enigo.key_up(EnigoKey::Control);
                    }
                    crate::modules::llm::Command::Enter => {
                        enigo.key_click(EnigoKey::Return);
                    }
                }
            });
            match result {
                Ok(_) => Ok(()),
                Err(_) => Err(anyhow::anyhow!("Command execution panicked")),
            }
        }

        #[cfg(target_os = "macos")]
        {
            Ok(())
        }
    }
}
