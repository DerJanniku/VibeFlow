use crate::modules::audio::AudioEngine;
use crate::modules::state::AppState;
use tauri::{AppHandle, Emitter, Manager, State};

#[tauri::command]
pub fn get_audio_devices() -> Result<Vec<String>, String> {
    AudioEngine::list_input_devices().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_audio_device(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
) -> Result<(), String> {
    *state.selected_device.lock() = Some(name.clone());

    // Persist to config.json
    let mut config = load_config(&app);
    config["device"] = serde_json::json!(name);
    save_config(&app, &config)?;

    Ok(())
}

#[tauri::command]
pub fn get_audio_device(state: State<'_, AppState>) -> Option<String> {
    state.selected_device.lock().clone()
}

#[tauri::command]
pub fn save_hotkey(app: AppHandle, modifiers: Vec<String>, code: String) -> Result<(), String> {
    use tauri_plugin_global_shortcut::{Code, Modifiers};
    let state = app.state::<AppState>();

    let mut m = Modifiers::empty();
    for mod_str in modifiers.clone() {
        match mod_str.to_uppercase().as_str() {
            "CTRL" | "CONTROL" => m |= Modifiers::CONTROL,
            "SHIFT" => m |= Modifiers::SHIFT,
            "ALT" => m |= Modifiers::ALT,
            "SUPER" | "COMMAND" | "META" => m |= Modifiers::SUPER,
            _ => {}
        }
    }

    // Map string representation to tauri_plugin_global_shortcut::Code
    let c = match code.to_uppercase().as_str() {
        "SPACE" => Code::Space,
        "F9" => Code::F9,
        "F10" => Code::F10,
        "F11" => Code::F11,
        "F12" => Code::F12,
        "A" => Code::KeyA,
        "B" => Code::KeyB,
        "C" => Code::KeyC,
        "D" => Code::KeyD,
        "E" => Code::KeyE,
        "F" => Code::KeyF,
        "G" => Code::KeyG,
        "H" => Code::KeyH,
        "I" => Code::KeyI,
        "J" => Code::KeyJ,
        "K" => Code::KeyK,
        "L" => Code::KeyL,
        "M" => Code::KeyM,
        "N" => Code::KeyN,
        "O" => Code::KeyO,
        "P" => Code::KeyP,
        "Q" => Code::KeyQ,
        "R" => Code::KeyR,
        "S" => Code::KeyS,
        "T" => Code::KeyT,
        "U" => Code::KeyU,
        "V" => Code::KeyV,
        "W" => Code::KeyX,
        "X" => Code::KeyX,
        "Y" => Code::KeyY,
        "Z" => Code::KeyZ,
        _ => Code::F9,
    };

    *state.hotkey_modifiers.lock() = m;
    *state.hotkey_code.lock() = c;

    // Persist to config.json
    let mut config = load_config(&app);
    config["hotkey"] = serde_json::json!({
        "modifiers": modifiers,
        "code": code
    });
    save_config(&app, &config)?;

    crate::re_register_shortcut(&app).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_hotkey(state: State<'_, AppState>) -> String {
    let mods = state.hotkey_modifiers.lock();
    let code = state.hotkey_code.lock();

    let mut label = String::new();
    if mods.contains(tauri_plugin_global_shortcut::Modifiers::CONTROL) {
        label.push_str("Ctrl + ");
    }
    if mods.contains(tauri_plugin_global_shortcut::Modifiers::SHIFT) {
        label.push_str("Shift + ");
    }
    if mods.contains(tauri_plugin_global_shortcut::Modifiers::ALT) {
        label.push_str("Alt + ");
    }
    if mods.contains(tauri_plugin_global_shortcut::Modifiers::SUPER) {
        label.push_str("Super + ");
    }

    label.push_str(&format!("{:?}", *code));
    label
}

#[tauri::command]
pub async fn download_model(app: AppHandle, tier: String) -> Result<(), String> {
    use futures_util::StreamExt;
    use tokio::io::AsyncWriteExt;

    let (model_url, filename) = match tier.as_str() {
        "realfast" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin",
            "ggml-tiny.en.bin",
        ),
        "fast" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin",
            "ggml-base.en.bin",
        ),
        "standard" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin",
            "ggml-small.en.bin",
        ),
        "pro" => (
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin",
            "ggml-large-v3-turbo.bin",
        ),
        _ => return Err("Invalid model tier selected".to_string()),
    };

    let model_path = app.path().app_data_dir().unwrap().join(filename);

    // Save preference to app config (simulated for now by updating state)
    let state = app.state::<AppState>();
    *state.selected_model.lock() = filename.to_string();

    if model_path.exists() {
        println!("[DEBUG] Model {} already exists", filename);
        app.emit("download-progress", 100)
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    if let Some(parent) = model_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    println!("[DEBUG] Starting real model download from {}", model_url);

    let response = reqwest::get(model_url).await.map_err(|e| e.to_string())?;
    let total_size = response.content_length().unwrap_or(0);

    let mut file = tokio::fs::File::create(&model_path)
        .await
        .map_err(|e: std::io::Error| e.to_string())?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| e.to_string())?;
        file.write_all(&chunk)
            .await
            .map_err(|e: std::io::Error| e.to_string())?;
        downloaded += chunk.len() as u64;

        if total_size > 0 {
            let progress = (downloaded as f32 / total_size as f32 * 100.0) as u32;
            let _ = app.emit("download-progress", progress);
        }
    }

    println!("[DEBUG] Model downloaded to {:?}", model_path);
    app.emit("download-progress", 100)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_selected_model(state: tauri::State<AppState>) -> String {
    state.selected_model.lock().clone()
}

/// Load config from JSON file
fn load_config(app: &AppHandle) -> serde_json::Value {
    let config_path = app.path().app_data_dir().unwrap().join("config.json");
    if config_path.exists() {
        if let Ok(data) = std::fs::read_to_string(&config_path) {
            if let Ok(json) = serde_json::from_str(&data) {
                return json;
            }
        }
    }
    serde_json::json!({})
}

/// Save config to JSON file
fn save_config(app: &AppHandle, config: &serde_json::Value) -> Result<(), String> {
    let config_path = app.path().app_data_dir().unwrap().join("config.json");
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(config_path, data).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_onboarding_status(app: AppHandle) -> bool {
    let config = load_config(&app);
    let onboarded = config
        .get("onboarded")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    println!("[DEBUG] Onboarding check: onboarded = {}", onboarded);
    onboarded
}

#[tauri::command]
pub fn complete_onboarding(app: AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();

    let current_hotkey_label = get_hotkey(state.clone());
    let hotkey_parts = current_hotkey_label
        .split(" + ")
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let code = hotkey_parts.last().cloned().unwrap_or("Space".to_string());
    let modifiers = if hotkey_parts.len() > 1 {
        hotkey_parts[0..hotkey_parts.len() - 1].to_vec()
    } else {
        vec![]
    };

    // Build config object with all settings
    let config = serde_json::json!({
        "onboarded": true,
        "model": state.selected_model.lock().clone(),
        "device": state.selected_device.lock().clone(),
        "hotkey": {
            "modifiers": modifiers,
            "code": code
        },
        "version": "0.3.0"
    });

    save_config(&app, &config)?;
    println!("[DEBUG] Onboarding completed and saved to config.json");
    Ok(())
}
