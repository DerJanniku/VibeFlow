// linux_paste.rs
use anyhow::{anyhow, Result};
use arboard::Clipboard;
use std::process::Command;
use std::thread;
use std::time::Duration;

pub struct LinuxPaste;

impl LinuxPaste {
    pub fn paste_text(text: &str) -> Result<()> {
        // session type
        let session = Self::get_session_type();
        println!("[LinuxPaste] Detected session: {}", session);

        
        Self::copy_to_clipboard(text)?;

        thread::sleep(Duration::from_millis(50));

        match session.as_str() {
            "wayland" => Self::paste_wayland(),
            _ => Self::paste_x11(),
        }
    }

    fn get_session_type() -> String {
        std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "x11".to_string())
    }

    fn copy_to_clipboard(text: &str) -> Result<()> {
        let mut clipboard = Clipboard::new()
            .map_err(|e| anyhow!("Failed to initialize clipboard: {}", e))?;
        
        clipboard.set_text(text.to_owned())
            .map_err(|e| anyhow!("Failed to copy to clipboard: {}", e))?;
        
        println!("[LinuxPaste] Text copied to clipboard");
        Ok(())
    }

    fn paste_x11() -> Result<()> {
        if !Self::check_command("xdotool") {
            return Err(anyhow!("xdotool not found. Please install it (sudo pacman -S xdotool)"));
        }

        let output = Command::new("xdotool")
            .arg("key")
            .arg("ctrl+v")
            .output()?;

        if output.status.success() {
            println!("[LinuxPaste] xdotool paste successful");
            Ok(())
        } else {
            Err(anyhow!("xdotool failed: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }

    fn paste_wayland() -> Result<()> {
        if Self::check_command("wtype") {
            return Self::paste_wtype();
        }

        if Self::check_command("ydotool") {
            return Self::paste_ydotool();
        }

        Err(anyhow!("No Wayland paste tool found. Install wtype or ydotool"))
    }

    fn paste_wtype() -> Result<()> {
        let output = Command::new("wtype")
            .args(["-M", "ctrl", "-P", "v", "-m", "ctrl"])
            .output()?;

        if output.status.success() {
            println!("[LinuxPaste] wtype paste successful");
            Ok(())
        } else {
            Err(anyhow!("wtype failed: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }

    fn paste_ydotool() -> Result<()> {
        let output = Command::new("ydotool")
            .arg("key")
            .arg("29:56")  // ctrl+v
            .output()?;

        if output.status.success() {
            println!("[LinuxPaste] ydotool paste successful");
            Ok(())
        } else {
            Err(anyhow!("ydotool failed: {}", String::from_utf8_lossy(&output.stderr)))
        }
    }

    fn check_command(cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

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
            .arg("key")
            .arg(keys)
            .spawn()?;

        Ok(())
    }

    fn execute_command_wayland(keys: &str) -> Result<()> {
        if Self::check_command("wtype") {
            for key in keys.split_whitespace() {
                let mapped = match key {
                    "ctrl+shift+Left" => vec!["-M", "ctrl", "-M", "shift", "-P", "Left"],
                    "BackSpace" => vec!["-P", "BackSpace"],
                    "ctrl+b" => vec!["-M", "ctrl", "-P", "b"],
                    "ctrl+i" => vec!["-M", "ctrl", "-P", "i"],
                    "ctrl+a" => vec!["-M", "ctrl", "-P", "a"],
                    "Return" => vec!["-P", "Return"],
                    _ => return Err(anyhow!("Unsupported key sequence: {}", key)),
                };
                
                Command::new("wtype").args(&mapped).spawn()?;
                thread::sleep(Duration::from_millis(10));
            }
            Ok(())
        } else {
            Err(anyhow!("wtype not found"))
        }
    }
}
