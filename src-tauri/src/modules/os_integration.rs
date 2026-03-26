// os_integration.rs — Cross-platform text insertion and active-window detection.
//
// Linux strategy:
//   • Hyprland: `hyprctl activewindow -j` (JSON) — zero extra deps, always available on Hyprland
//   • X11:      `xdotool getactivewindow getwindowname` — available when xdotool is installed
//   • Fallback: return a generic string so context-detection still works
//
// Windows/macOS: active-win-pos-rs (excluded from Linux build — see Cargo.toml)

#[cfg(target_os = "linux")]
use crate::modules::linux_paste::LinuxPaste;

// active-win-pos-rs is only compiled on Windows/macOS
#[cfg(not(target_os = "linux"))]
use active_win_pos_rs::get_active_window;

use anyhow::Result;
use std::env;
use std::sync::{Arc, Mutex};

#[cfg(target_os = "windows")]
use arboard::Clipboard;
#[cfg(target_os = "windows")]
use enigo::{Enigo, Key as EnigoKey, KeyboardControllable};
#[cfg(target_os = "windows")]
use std::thread;
#[cfg(target_os = "windows")]
use std::time::Duration;

pub struct OSIntegration;

// Mock clipboard used in test mode
lazy_static::lazy_static! {
    static ref MOCK_CLIPBOARD: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
}

impl OSIntegration {
    pub fn get_active_app_name() -> String {
        #[cfg(target_os = "linux")]
        {
            // Try Hyprland first (most users on Arch who use this app)
            if let Some(name) = Self::get_active_app_hyprland() {
                return name;
            }
            // Fallback to xdotool on X11
            if let Some(name) = Self::get_active_app_xdotool() {
                return name;
            }
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

    /// Hyprland: `hyprctl activewindow -j` returns JSON like:
    /// { "class": "kitty", "title": "~", ... }
    #[cfg(target_os = "linux")]
    fn get_active_app_hyprland() -> Option<String> {
        // Only attempt if we are running under Hyprland
        if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_err() {
            return None;
        }

        let output = std::process::Command::new("hyprctl")
            .args(["activewindow", "-j"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let json: serde_json::Value =
            serde_json::from_slice(&output.stdout).ok()?;

        // "class" holds the application name (e.g. "kitty", "code", "firefox")
        json.get("class")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// X11 fallback via xdotool
    #[cfg(target_os = "linux")]
    fn get_active_app_xdotool() -> Option<String> {
        let output = std::process::Command::new("xdotool")
            .args(["getactivewindow", "getwindowname"])
            .output()
            .ok()?;

        if output.status.success() {
            let name = String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
        None
    }

    pub fn paste_text(text: &str) -> Result<()> {
        // Test-mode shortcut
        if env::var("VIBEFLOW_TEST_MODE").is_ok() {
            *MOCK_CLIPBOARD.lock().unwrap() = text.to_string();
            return Ok(());
        }

        log::debug!("[OS] paste_text: {}", text);

        #[cfg(target_os = "linux")]
        {
            return LinuxPaste::paste_text(text).map_err(|e| {
                log::warn!("[Linux] Paste failed: {}", e);
                e
            });
        }

        #[cfg(target_os = "windows")]
        {
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
            return match result {
                Ok(inner) => inner,
                Err(_) => Err(anyhow::anyhow!("Paste operation panicked")),
            };
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
                Command::Delete    => "ctrl+shift+Left BackSpace",
                Command::Bold      => "ctrl+b",
                Command::Italic    => "ctrl+i",
                Command::SelectAll => "ctrl+a",
                Command::Enter     => "Return",
            };
            return LinuxPaste::execute_command(key_sequence);
        }

        #[cfg(target_os = "windows")]
        {
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
            return match result {
                Ok(_) => Ok(()),
                Err(_) => Err(anyhow::anyhow!("Command execution panicked")),
            };
        }

        #[cfg(target_os = "macos")]
        {
            Ok(())
        }
    }
}
