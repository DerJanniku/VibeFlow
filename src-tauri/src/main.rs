#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod modules {
    pub mod audio;
    pub mod commands;
    pub mod inference;
    pub mod llm;
    pub mod linux_paste;
    pub mod os_integration;
    pub mod state;
}

use modules::{
    audio::AudioEngine, inference::InferenceEngine, llm::ContextEngine,
    os_integration::OSIntegration, state::AppState,
};
// active_win_pos_rs is excluded on Linux (crashes on Wayland) — see Cargo.toml target dep
use parking_lot::Mutex;
use rodio::{OutputStream, Sink, Source};
use std::sync::Arc;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_global_shortcut::{
    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutEvent, ShortcutState,
};
use tokio::sync::mpsc;
// use modules::audio::SensitiveAudio; (unused)

#[tauri::command]
fn ui_ready() {
    log::debug!("[DEBUG] >>> OVERLAY UI IS READY AND CONNECTED! <<<");
}

#[tokio::main]
async fn main() {
    // SECURITY: Global Panic Hook to capture exact abort reasons
    std::panic::set_hook(Box::new(|info| {
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            *s
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            &s[..]
        } else {
            "Unknown panic"
        };
        let location = info.location().map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column())).unwrap_or_else(|| "unknown location".to_string());
        log::error!("[CRITICAL PANIC] {} at {}", msg, location);
    }));

    // Start idle — recording begins only when the user presses the hotkey
    let is_recording = Arc::new(Mutex::new(false));
    let tx_audio = Arc::new(Mutex::new(None));
    let amplitude = Arc::new(Mutex::new(0.0));
    let selected_device = Arc::new(Mutex::new(None));
    let hotkey_modifiers = Arc::new(Mutex::new(Modifiers::CONTROL | Modifiers::SHIFT));
    let hotkey_code = Arc::new(Mutex::new(Code::Space));
    
    // Switch to Multilingual Model for German Support
    let selected_model = Arc::new(Mutex::new("ggml-base.bin".to_string()));

    // Resolve the app config directory in a cross-platform way before Tauri initialises.
    // Windows: %APPDATA%\com.derjanniku.vibeflow
    // Linux:   $XDG_CONFIG_HOME/com.derjanniku.vibeflow  (fallback: ~/.config/…)
    // macOS:   ~/Library/Application Support/com.derjanniku.vibeflow
    #[cfg(target_os = "windows")]
    let app_data = std::env::var("APPDATA")
        .ok()
        .map(|ad| std::path::PathBuf::from(ad).join("com.derjanniku.vibeflow"))
        .unwrap_or_default();

    // Tauri's app_data_dir on Linux = $XDG_DATA_HOME/… (default ~/.local/share/…)
    // Must match exactly so config.json and model files are in the same place.
    #[cfg(target_os = "linux")]
    let app_data = std::env::var("XDG_DATA_HOME")
        .ok()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| std::path::PathBuf::from(h).join(".local").join("share"))
                .unwrap_or_default()
        })
        .join("com.derjanniku.vibeflow");

    #[cfg(target_os = "macos")]
    let app_data = std::env::var("HOME")
        .ok()
        .map(|h| {
            std::path::PathBuf::from(h)
                .join("Library")
                .join("Application Support")
                .join("com.derjanniku.vibeflow")
        })
        .unwrap_or_default();

    let config_path = app_data.join("config.json");
    if config_path.exists() {
        if let Ok(data) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&data) {
                // Load all fields that are present — no version-based wipe.
                // Missing fields just keep their defaults.
                if let Some(m) = json.get("model").and_then(|v| v.as_str()) {
                    *selected_model.lock() = m.to_string();
                }
                if let Some(d) = json.get("device").and_then(|v| v.as_str()) {
                    *selected_device.lock() = Some(d.to_string());
                }
                if let Some(hk) = json.get("hotkey") {
                    if let Some(mods_arr) = hk.get("modifiers").and_then(|v| v.as_array()) {
                        let mut m = Modifiers::empty();
                        for mod_v in mods_arr {
                            if let Some(mod_str) = mod_v.as_str() {
                                match mod_str.to_uppercase().as_str() {
                                    "CTRL" | "CONTROL" => m |= Modifiers::CONTROL,
                                    "SHIFT" => m |= Modifiers::SHIFT,
                                    "ALT" => m |= Modifiers::ALT,
                                    "SUPER" | "COMMAND" | "META" => m |= Modifiers::SUPER,
                                    _ => {}
                                }
                            }
                        }
                        *hotkey_modifiers.lock() = m;
                    }
                    if let Some(code_str) = hk.get("code").and_then(|v| v.as_str()) {
                        if code_str.to_uppercase() == "SPACE" {
                            *hotkey_code.lock() = Code::Space;
                        }
                    }
                }
                log::info!("[INFO] Config loaded from {:?}", config_path);
            }
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new()
            .level(log::LevelFilter::Debug)
            .build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(handle_shortcut)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            modules::commands::get_audio_devices,
            modules::commands::set_audio_device,
            modules::commands::get_audio_device,
            modules::commands::save_hotkey,
            modules::commands::get_hotkey,
            modules::commands::download_model,
            modules::commands::get_selected_model,
            modules::commands::get_onboarding_status,
            modules::commands::complete_onboarding,
            ui_ready
        ])
        .setup(|app| {
            let app_data = app.path().app_data_dir()?;
            if !app_data.exists() {
                std::fs::create_dir_all(&app_data)?;
            }
            
            // Default to Multilingual Base
            let model_name = "ggml-base.bin";
            let dest_path = app_data.join(model_name);
            
            let mut force_restore = false;
            if dest_path.exists() {
                if let Ok(meta) = std::fs::metadata(&dest_path) {
                     if meta.len() < 1000000 { // < 1MB
                        log::warn!("[WARNING] Model file exists but is too small/corrupted. Forcing restore.");
                        force_restore = true;
                     }
                }
            }

            if (!dest_path.exists() || force_restore) && model_name == "ggml-base.en.bin" {
                log::info!("[INFO] Model not found or corrupted. Attempting to restore from bundle...");
                match app.path().resolve("resources/ggml-base.en.bin", tauri::path::BaseDirectory::Resource) {
                    Ok(resource_path) => {
                        if resource_path.exists() {
                            if let Err(e) = std::fs::copy(&resource_path, &dest_path) {
                                log::error!("[ERROR] Failed to copy bundled model: {}", e);
                            } else {
                                log::info!("[INFO] Bundled model installed successfully to {:?}", dest_path);
                            }
                        } else {
                            log::warn!("[WARNING] Bundled resource not found at {:?}", resource_path);
                        }
                    }
                    Err(e) => log::warn!("[WARNING] Could not resolve bundled resource path: {}", e),
                }
            }

            let inference_engine = Arc::new(InferenceEngine::new(app_data));

            // DerJannik Branding
            println!(
                r#"
    __      ___ _          ______ _               
    \ \    / (_) |        |  ____| |              
     \ \  / / _| |__   ___| |__  | | _____      __
      \ \/ / | | '_ \ / _ \  __| | |/ _ \ \ /\ / /
       \  /  | | |_) |  __/ |    | | (_) \ V  V / 
        \/   |_|_.__/ \___|_|    |_|\___/ \_/\_/  
                                    by DerJannik
            "#
            );
            log::info!("[INFO] Made by DerJanniku | https://de.fiverr.com/s/xXgY29x");
            log::info!("[INFO] VibeFlow initialized.");

            let mods_val = *hotkey_modifiers.lock();
            let code_val = *hotkey_code.lock();

            // --- CORE AUDIO REFACTOR: ALWAYS-ON STREAM ---
            let (tx, mut rx) = mpsc::channel(100);
            *tx_audio.lock() = Some(tx.clone());

            let is_rec_clone = is_recording.clone();
            let amp_clone = amplitude.clone();
            let device_clone = selected_device.lock().clone();
            let app_handle = app.handle().clone();

            std::thread::spawn(move || {
                let stream = AudioEngine::start_stream(
                    tx,
                    is_rec_clone.clone(),
                    amp_clone.clone(),
                    device_clone,
                );
                match stream {
                    Ok(s) => {
                        use cpal::traits::StreamTrait;
                        if let Err(e) = s.play() {
                            log::error!("[ERROR] Failed to play audio stream: {}", e);
                        } else {
                            log::debug!("[DEBUG] Audio Stream Started (Always-On Mode) - Listening to buffer...");
                            // Keep thread alive
                            loop {
                                let amp = *amp_clone.lock();
                                let _ = app_handle.emit("amplitude", amp);
                                std::thread::sleep(std::time::Duration::from_millis(15));
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("[CRITICAL] Failed to start audio stream: {}", e);
                    }
                }
            });

            // --- INFERENCE REFACTOR: LISTENER ---
            let engine = inference_engine.clone();
            let app_handle_2 = app.handle().clone();
            let model_filename = selected_model.lock().clone();

            tauri::async_runtime::spawn(async move {
                log::debug!(
                    "[DEBUG] Starting transcription loop with model: {}...",
                    model_filename
                );

                loop {
                    let (refined, command) = match engine
                        .start_processing_loop(&mut rx, &model_filename, &app_handle_2)
                        .await
                    {
                        transcript if !transcript.as_str().trim().is_empty() => {
                            let _ = app_handle_2.emit("status", "Processing");
                            match ContextEngine::refine_text(&transcript).await {
                                Ok((r, c)) => (r, c),
                                Err(_) => (transcript.as_str().to_string(), None),
                            }
                        }
                        _ => continue,
                    };

                    log::debug!("[DEBUG] Final Refined: \"{}\"", &refined);
                    let _ = app_handle_2.emit("transcript", &refined);

                    if let Some(cmd) = command {
                        let _ = OSIntegration::execute_command(cmd);
                    } else if !refined.is_empty() && !refined.starts_with("Error:") {
                        let _ = OSIntegration::paste_text(&refined);
                    }
                    let _ = app_handle_2.emit("status", "Ready");
                }
            });

            // --- CONTEXT WATCHER ---
            let app_handle_3 = app.handle().clone();
            std::thread::spawn(move || loop {
                let context = ContextEngine::get_context();
                let _ = app_handle_3.emit("context_update", &context);
                std::thread::sleep(std::time::Duration::from_secs(1));
            });

            let state = AppState {
                is_recording,
                tx_audio,
                inference_engine,
                amplitude,
                selected_device,
                hotkey_modifiers,
                hotkey_code,
                selected_model,
            };
            app.manage(state);

            let shortcut = Shortcut::new(Some(mods_val), code_val);
            let _ = app.global_shortcut().unregister_all();
            let _ = app.global_shortcut().register(shortcut);

            // System Tray Setup
            let _app_handle = app.handle().clone();
            let show_item = MenuItem::with_id(app, "show", "Show VibeFlow", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            let tray_icon = app.default_window_icon().cloned()
                .unwrap_or_else(|| {
                    log::warn!("[WARNING] Default window icon not found, using empty icon.");
                    tauri::image::Image::new(&[], 0, 0)
                });

            let tray_builder = TrayIconBuilder::new()
                .icon(tray_icon)
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                });
            
            // On Linux, Tray creation can fail if libappindicator is missing or tray isn't available
            match tray_builder.build(app) {
                Ok(_) => log::debug!("[DEBUG] System Tray initialized successfully."),
                Err(e) => log::warn!("[WARNING] System Tray failed to initialize (expected on some Linux environments): {}", e),
            }
            
            // Handle window close -> hide to tray ONLY if on Windows/Mac or if tray is actually there
            // For Linux, we default to showing the window initially but allowing standard close if tray fails.
            if let Some(window) = app.get_webview_window("main") {
                #[cfg(not(target_os = "linux"))]
                let win = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api: _close_api, .. } = event {
                        #[cfg(not(target_os = "linux"))]
                        {
                            _close_api.prevent_close();
                            let _ = win.hide();
                        }
                        // On Linux, we allow standard close unless we want to force hide?
                        // Let's keep it safe for now: Standard close on Linux.
                    }
                });
                let _ = window.show();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn handle_shortcut(app: &AppHandle, shortcut: &Shortcut, event: ShortcutEvent) {
    let state = app.state::<AppState>();
    let mods = *state.hotkey_modifiers.lock();
    let code = *state.hotkey_code.lock();

    log::debug!("[DEBUG] handle_shortcut event: {:?} for shortcut: {:?}", event, shortcut);
    if shortcut.matches(mods, code) && event.state() == ShortcutState::Pressed {
        let recording = { *state.is_recording.lock() };

        if recording {
            stop_recording(app);
        } else {
            start_recording(app);
        }
    }
}

pub fn re_register_shortcut(app: &AppHandle) -> Result<(), tauri_plugin_global_shortcut::Error> {
    let state = app.state::<AppState>();
    let mods = *state.hotkey_modifiers.lock();
    let code = *state.hotkey_code.lock();

    // Unregister all first to be safe
    let _ = app.global_shortcut().unregister_all();

    let shortcut = Shortcut::new(Some(mods), code);
    app.global_shortcut().register(shortcut)?;
    log::debug!("[DEBUG] Hotkey re-registered: {:?} + {:?}", mods, code);
    Ok(())
}

fn play_feedback_sound(frequency: f32) {
    std::thread::spawn(move || {
        let res = OutputStream::try_default();
        if let Ok((_stream, stream_handle)) = res {
            if let Ok(sink) = Sink::try_new(&stream_handle) {
                let source = rodio::source::SineWave::new(frequency)
                    .take_duration(std::time::Duration::from_millis(150))
                    .amplify(0.2);
                sink.append(source);
                sink.sleep_until_end();
            }
        }
    });
}

fn start_recording(app: &AppHandle) {
    let state = app.state::<AppState>();
    let mut recording_guard = state.is_recording.lock();
    if *recording_guard {
        return;
    }
    *recording_guard = true;

    play_feedback_sound(880.0);
    log::info!(">>> VibeFlow: Recording Toggle ON (Flag set to true)");

    // NEW LOGIC: We don't spawn a thread here anymore.
    // The thread is already running in main().
    // We just toggled the flag, which tells the loop in audio.rs to start pushing data to the channel.
    // AND it triggers the "Flush Ring Buffer" logic due to the flag flip.

    // Position overlay logic remains the same
    // Get active window to determine which monitor the user is looking at
    // active_win_pos_rs is not compiled on Linux (Wayland segfault) — see Cargo.toml
    #[cfg(not(target_os = "linux"))]
    let target_monitor = if let Ok(active_window) = active_win_pos_rs::get_active_window() {
        let active_center_x =
            active_window.position.x + (active_window.position.width / 2.0) as f64;
        let active_center_y =
            active_window.position.y + (active_window.position.height / 2.0) as f64;

        // Find monitor containing this point
        app.available_monitors().ok().and_then(|monitors| {
            monitors.into_iter().find(|m| {
                let pos = m.position();
                let size = m.size();
                let m_x = pos.x as f64;
                let m_y = pos.y as f64;
                let m_w = size.width as f64;
                let m_h = size.height as f64;

                active_center_x >= m_x
                    && active_center_x < (m_x + m_w)
                    && active_center_y >= m_y
                    && active_center_y < (m_y + m_h)
            })
        })
    } else {
        None
    };

    #[cfg(target_os = "linux")]
    let target_monitor: Option<tauri::Monitor> = None;

    // Fallback to main window or overlay's current
    let final_monitor = target_monitor
        .or_else(|| {
            let main_window = app.get_webview_window("main");
            main_window.and_then(|w| w.current_monitor().ok().flatten())
        })
        .or_else(|| {
            app.get_webview_window("overlay")
                .and_then(|w| w.current_monitor().ok().flatten())
        });

    if let Some(overlay) = app.get_webview_window("overlay") {
        if let Some(monitor) = final_monitor {
            log::debug!(
                "[DEBUG] Positioning overlay on monitor: {:?}",
                monitor.name()
            );
            let size = monitor.size();
            let win_size = overlay
                .outer_size()
                .unwrap_or(tauri::PhysicalSize::new(200, 120));
            let x = monitor.position().x + (size.width as i32 - win_size.width as i32) / 2;
            let y = monitor.position().y + size.height as i32 - win_size.height as i32 - 40;
            // On Wayland, set_position is restricted by the compositor; log failure but don't crash.
            if let Err(e) = overlay.set_position(tauri::PhysicalPosition::new(x, y)) {
                log::debug!("[DEBUG] set_position not supported by compositor (Wayland): {}. Using default position.", e);
            }
        } else {
            log::error!("[ERROR] Could not detect any monitor for overlay positioning.");
        }

        // Show the window as early as possible to ensure it's mapped by the compositor
        let _ = overlay.show();

        // set_shadow is only meaningful on Windows/macOS
        #[cfg(not(target_os = "linux"))]
        let _ = overlay.set_shadow(false);

        // Prevent the overlay from stealing input focus or blocking clicks
        if let Err(e) = overlay.set_ignore_cursor_events(true) {
            log::debug!("[DEBUG] set_ignore_cursor_events failed (may not be supported): {}", e);
        }
    }

    if let Some(_window) = app.get_webview_window("main") {
        let _ = app.emit("status", "Recording");
    }
}

fn stop_recording(app: &AppHandle) {
    let state = app.state::<AppState>();
    let mut recording_guard = state.is_recording.lock();
    if !*recording_guard {
        return;
    }
    *recording_guard = false;
    // *state.tx_audio.lock() = None; // DON'T CLOSE CHANNEL! It stays open.

    // Hide overlay
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.hide();
    }

    play_feedback_sound(440.0);
    log::info!(">>> VibeFlow: Recording Toggle OFF (Flag set to false)");
}
