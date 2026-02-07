#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod modules {
    pub mod audio;
    pub mod inference;
    pub mod llm;
    pub mod os_integration;
    pub mod commands;
    pub mod state;
}

use tauri::{Manager, Emitter, AppHandle};
use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent};
use tauri::menu::{Menu, MenuItem};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState, Code, Modifiers, ShortcutEvent, Shortcut};
use std::sync::Arc;
use tokio::sync::mpsc;
use parking_lot::Mutex;
use rodio::{OutputStream, Sink, Source};
use modules::{audio::AudioEngine, inference::InferenceEngine, llm::ContextEngine, os_integration::OSIntegration, state::AppState};
// use modules::audio::SensitiveAudio; (unused)

#[tokio::main]
async fn main() {
    let is_recording = Arc::new(Mutex::new(false));
    let tx_audio = Arc::new(Mutex::new(None));
    let amplitude = Arc::new(Mutex::new(0.0));
    let selected_device = Arc::new(Mutex::new(None));
    let hotkey_modifiers = Arc::new(Mutex::new(Modifiers::empty()));
    let hotkey_code = Arc::new(Mutex::new(Code::F9));
    let selected_model = Arc::new(Mutex::new("ggml-base.en.bin".to_string()));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![])))
        .plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(handle_shortcut).build())
        .invoke_handler(tauri::generate_handler![
            modules::commands::get_audio_devices,
            modules::commands::set_audio_device,
            modules::commands::get_audio_device,
            modules::commands::save_hotkey,
            modules::commands::get_hotkey,
            modules::commands::download_model,
            modules::commands::get_selected_model,
            modules::commands::get_onboarding_status,
            modules::commands::complete_onboarding
        ])
        .setup(|app| {
            let app_data = app.path().app_data_dir()?;
            let inference_engine = Arc::new(InferenceEngine::new(app_data));

            // DerJannik Branding
            println!(r#"
    __      ___ _          ______ _               
    \ \    / (_) |        |  ____| |              
     \ \  / / _| |__   ___| |__  | | _____      __
      \ \/ / | | '_ \ / _ \  __| | |/ _ \ \ /\ / /
       \  /  | | |_) |  __/ |    | | (_) \ V  V / 
        \/   |_|_.__/ \___|_|    |_|\___/ \_/\_/  
                                    by DerJannik
            "#);
            println!("[INFO] Made by DerJannik | https://de.fiverr.com/s/xXgY29x");
            println!("[INFO] VibeFlow Professional initialized.");
            
            let mods_val = *hotkey_modifiers.lock();
            let code_val = *hotkey_code.lock();

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
            let app_handle = app.handle().clone();
            let show_item = MenuItem::with_id(app, "show", "Show VibeFlow", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .menu_on_left_click(false)
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        },
                        "quit" => {
                            app.exit(0);
                        },
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Handle window close -> hide to tray
            if let Some(window) = app.get_webview_window("main") {
                let win = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win.hide();
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

    if shortcut.matches(mods, code) && event.state() == ShortcutState::Pressed {
        let recording = { *state.is_recording.lock() };
        println!("[DEBUG] Shortcut triggered: Recording={}", recording);
        
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
    println!("[DEBUG] Hotkey re-registered: {:?} + {:?}", mods, code);
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
    if *recording_guard { return; } 
    *recording_guard = true;

    play_feedback_sound(880.0);
    println!(">>> VibeFlow: Recording Toggle ON");

    // Show overlay
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.show();
    }

    if let Some(window) = app.get_webview_window("main") {
        let _ = app.emit("status", "Recording");
    }

    let (tx, rx) = mpsc::channel(100);
    *state.tx_audio.lock() = Some(tx.clone());

    let is_rec_clone = state.is_recording.clone();
    let amp_clone = state.amplitude.clone();
    let device_clone = state.selected_device.lock().clone();
    let app_handle = app.clone();

    std::thread::spawn(move || {
        let stream = AudioEngine::start_stream(tx, is_rec_clone.clone(), amp_clone.clone(), device_clone);
        if let Ok(s) = stream {
             use cpal::traits::StreamTrait;
             let _ = s.play();
             while *is_rec_clone.lock() {
                 let amp = *amp_clone.lock();
                 let _ = app_handle.emit("amplitude", amp);
                 std::thread::sleep(std::time::Duration::from_millis(30));
             }
        }
    });

    let engine = state.inference_engine.clone();
    let app_handle_2 = app.clone();
    let model_filename = state.selected_model.lock().clone();
    
    tauri::async_runtime::spawn(async move {
        println!("[DEBUG] Starting transcription loop with model: {}...", model_filename);
        let transcript = engine.start_processing_loop(rx, model_filename).await;
        println!("[DEBUG] Transcription complete: \"{}\"", transcript.as_str());
        
        let _ = app_handle_2.emit("status", "Processing");
        let refined = ContextEngine::refine_text(&transcript).await.unwrap_or_default();
        println!("[DEBUG] Refined text: \"{}\"", refined);
        
        // Emit for the UI test area
        let _ = app_handle_2.emit("transcript", &refined);
        
        let _ = OSIntegration::paste_text(&refined);
        let _ = app_handle_2.emit("status", "Ready");
    });
}

fn stop_recording(app: &AppHandle) {
    let state = app.state::<AppState>();
    let mut recording_guard = state.is_recording.lock();
    if !*recording_guard { return; }
    *recording_guard = false;
    *state.tx_audio.lock() = None;
    
    // Hide overlay
    if let Some(overlay) = app.get_webview_window("overlay") {
        let _ = overlay.hide();
    }
    
    play_feedback_sound(440.0);
    println!(">>> VibeFlow: Recording Toggle OFF");
}
