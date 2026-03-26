// linux_paste.rs — Linux text-insertion for X11 and Wayland (including Hyprland)
//
// Strategy:
//  Wayland  → 1. wl-copy (wl-clipboard) sets clipboard
//               2a. wtype  — direct keystroke simulation (preferred, no ydotool daemon needed)
//               2b. ydotool — fallback (requires ydotoold daemon)
//  X11      → xdotool (xdotool type --clearmodifiers or key ctrl+v)
//
// For Hyprland users the recommended tools are:
//   wl-clipboard  (wl-copy / wl-paste)  — pacman -S wl-clipboard
//   wtype                               — pacman -S wtype
//
use anyhow::{anyhow, Result};
use arboard::Clipboard;
use std::process::Command;
use std::thread;
use std::time::Duration;

pub struct LinuxPaste;

/// Which Wayland compositor / session are we running under?
#[derive(Debug, PartialEq)]
enum WaylandCompositor {
    Hyprland,
    Other,
}

impl LinuxPaste {
    pub fn paste_text(text: &str) -> Result<()> {
        let session = Self::get_session_type();
        log::debug!("[LinuxPaste] session={}", session);

        match session.as_str() {
            "wayland" => Self::paste_wayland(text),
            _ => Self::paste_x11(text),
        }
    }

    fn get_session_type() -> String {
        std::env::var("XDG_SESSION_TYPE")
            .unwrap_or_else(|_| "x11".to_string())
            .to_lowercase()
    }

    fn detect_compositor() -> WaylandCompositor {
        if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            WaylandCompositor::Hyprland
        } else {
            WaylandCompositor::Other
        }
    }

    // ------------------------------------------------------------------
    // Wayland
    // ------------------------------------------------------------------

    fn paste_wayland(text: &str) -> Result<()> {
        let compositor = Self::detect_compositor();
        log::debug!("[LinuxPaste] Wayland compositor: {:?}", compositor);

        // 1. Put text into the Wayland clipboard.
        //    Prefer wl-copy (wl-clipboard) over arboard — more reliable on Wayland.
        if Self::check_command("wl-copy") {
            Self::copy_with_wl_copy(text)?;
        } else {
            // arboard fallback (works on some Wayland setups via XWayland)
            Self::copy_to_clipboard(text)?;
        }

        // Small delay so the target application can detect the clipboard change.
        thread::sleep(Duration::from_millis(60));

        // 2. Simulate Ctrl+V in the focused window.
        if Self::check_command("wtype") {
            return Self::paste_wtype_ctrl_v();
        }

        if Self::check_command("ydotool") {
            return Self::paste_ydotool();
        }

        Err(anyhow!(
            "No Wayland key-simulation tool found.\n\
             Install one of:\n  \
             • wtype  (recommended for Hyprland): sudo pacman -S wtype\n  \
             • ydotool (needs ydotoold daemon):    sudo pacman -S ydotool"
        ))
    }

    fn copy_with_wl_copy(text: &str) -> Result<()> {
        let mut child = Command::new("wl-copy")
            .arg("--")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("Failed to spawn wl-copy: {}", e))?;

        if let Some(stdin) = child.stdin.take() {
            use std::io::Write;
            let mut stdin = stdin;
            stdin
                .write_all(text.as_bytes())
                .map_err(|e| anyhow!("wl-copy write failed: {}", e))?;
        }

        let status = child
            .wait()
            .map_err(|e| anyhow!("wl-copy wait failed: {}", e))?;

        if status.success() {
            log::debug!("[LinuxPaste] wl-copy clipboard set");
            Ok(())
        } else {
            Err(anyhow!("wl-copy exited with status {}", status))
        }
    }

    fn paste_wtype_ctrl_v() -> Result<()> {
        // wtype -M ctrl -P v -m ctrl  →  press Ctrl+V, then release Ctrl
        let output = Command::new("wtype")
            .args(["-M", "ctrl", "-P", "v", "-m", "ctrl"])
            .output()
            .map_err(|e| anyhow!("Failed to run wtype: {}", e))?;

        if output.status.success() {
            log::debug!("[LinuxPaste] wtype Ctrl+V successful");
            Ok(())
        } else {
            Err(anyhow!(
                "wtype failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    fn paste_ydotool() -> Result<()> {
        // keycodes: 29 = Left Ctrl, 47 = v
        let output = Command::new("ydotool")
            .args(["key", "29:1", "47:1", "47:0", "29:0"])
            .output()
            .map_err(|e| anyhow!("Failed to run ydotool: {}", e))?;

        if output.status.success() {
            log::debug!("[LinuxPaste] ydotool Ctrl+V successful");
            Ok(())
        } else {
            Err(anyhow!(
                "ydotool failed: {}\nMake sure ydotoold is running: sudo ydotoold &",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    // ------------------------------------------------------------------
    // X11
    // ------------------------------------------------------------------

    fn paste_x11(text: &str) -> Result<()> {
        // First set the clipboard via arboard (works fine on X11)
        Self::copy_to_clipboard(text)?;
        thread::sleep(Duration::from_millis(50));

        if !Self::check_command("xdotool") {
            return Err(anyhow!(
                "xdotool not found.\nInstall it: sudo pacman -S xdotool"
            ));
        }

        let output = Command::new("xdotool")
            .args(["key", "--clearmodifiers", "ctrl+v"])
            .output()
            .map_err(|e| anyhow!("Failed to run xdotool: {}", e))?;

        if output.status.success() {
            log::debug!("[LinuxPaste] xdotool Ctrl+V successful");
            Ok(())
        } else {
            Err(anyhow!(
                "xdotool failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    fn copy_to_clipboard(text: &str) -> Result<()> {
        let mut clipboard =
            Clipboard::new().map_err(|e| anyhow!("Failed to init clipboard: {}", e))?;
        clipboard
            .set_text(text.to_owned())
            .map_err(|e| anyhow!("Failed to set clipboard: {}", e))?;
        log::debug!("[LinuxPaste] arboard clipboard set");
        Ok(())
    }

    // ------------------------------------------------------------------
    // Key-command execution (for LLM magic commands)
    // ------------------------------------------------------------------

    pub fn execute_command(keys: &str) -> Result<()> {
        let session = Self::get_session_type();
        match session.as_str() {
            "wayland" => Self::execute_command_wayland(keys),
            _ => Self::execute_command_x11(keys),
        }
    }

    fn execute_command_x11(keys: &str) -> Result<()> {
        if !Self::check_command("xdotool") {
            return Err(anyhow!("xdotool not found"));
        }
        Command::new("xdotool")
            .args(["key", "--clearmodifiers", keys])
            .spawn()
            .map_err(|e| anyhow!("xdotool spawn failed: {}", e))?;
        Ok(())
    }

    fn execute_command_wayland(keys: &str) -> Result<()> {
        if !Self::check_command("wtype") {
            return Err(anyhow!(
                "wtype not found. Install: sudo pacman -S wtype"
            ));
        }

        // Map VibeFlow key-sequence strings to wtype arguments
        let args: Vec<&str> = match keys {
            "ctrl+shift+Left BackSpace" => {
                vec!["-M", "ctrl", "-M", "shift", "-P", "Left", "-m", "shift", "-m", "ctrl", "-P", "BackSpace"]
            }
            "BackSpace" => vec!["-P", "BackSpace"],
            "ctrl+b"    => vec!["-M", "ctrl", "-P", "b", "-m", "ctrl"],
            "ctrl+i"    => vec!["-M", "ctrl", "-P", "i", "-m", "ctrl"],
            "ctrl+a"    => vec!["-M", "ctrl", "-P", "a", "-m", "ctrl"],
            "Return"    => vec!["-P", "Return"],
            other => return Err(anyhow!("Unsupported key sequence for wtype: {}", other)),
        };

        let output = Command::new("wtype")
            .args(&args)
            .output()
            .map_err(|e| anyhow!("wtype spawn failed: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(anyhow!(
                "wtype failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    // ------------------------------------------------------------------
    // Utility
    // ------------------------------------------------------------------

    fn check_command(cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
