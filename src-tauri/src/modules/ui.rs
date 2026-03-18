use tray_icon::{TrayIconBuilder, menu::{Menu, MenuItem}};
use notify_rust::Notification;
use anyhow::Result;

pub struct UiManager;

impl UiManager {
    pub fn init_tray() -> Result<tray_icon::TrayIcon> {
        let tray_menu = Menu::new();
        let quit_i = MenuItem::new("Quit VibeFlow", true, None);
        tray_menu.append(&quit_i)?;

        // In a real scenario, we'd load an icon. For now, we assume system default or handle error.
        // Creating a tray icon usually requires an Icon struct.
        // We will skip actual icon loading to avoid complex image dependency handling in this prompt,
        // but the infrastructure is here.
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("VibeFlow - Ready")
            .build()?;
            
        Ok(tray_icon)
    }

    pub fn send_notification(title: &str, body: &str) {
        let _ = Notification::new()
            .summary(title)
            .body(body)
            .show();
    }
}
