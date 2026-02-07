use anyhow::Result;
use active_win_pos_rs::get_active_window;
use arboard::Clipboard;
use enigo::{Enigo, KeyboardControllable, Key as EnigoKey};
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::env;

pub struct OSIntegration;

// Mock Clipboard for Testing
lazy_static::lazy_static! {
    static ref MOCK_CLIPBOARD: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
}

impl OSIntegration {
    pub fn get_active_app_name() -> String {
        match get_active_window() {
            Ok(window) => window.app_name,
            Err(_) => "Unknown".to_string(),
        }
    }

    pub fn paste_text(text: &str) -> Result<()> {
        // Runtime Mock Check for Integration Tests
        if env::var("VIBEFLOW_TEST_MODE").is_ok() {
            *MOCK_CLIPBOARD.lock().unwrap() = text.to_string();
            return Ok(());
        }

        let mut clipboard = Clipboard::new()?;
        
        // Security: Save previous state
        let original_content = clipboard.get_text().unwrap_or_default();
        
        // Set new content
        clipboard.set_text(text.to_owned())?;
        
        let mut enigo = Enigo::new();
        enigo.key_down(EnigoKey::Control);
        enigo.key_click(EnigoKey::Layout('v'));
        enigo.key_up(EnigoKey::Control);
        
        // Security: Restore logic after 500ms
        thread::sleep(Duration::from_millis(500));
        
        // Restore
        let _ = clipboard.set_text(original_content);
        
        Ok(())
    }

    pub fn get_mock_paste() -> String {
        MOCK_CLIPBOARD.lock().unwrap().clone()
    }
}
