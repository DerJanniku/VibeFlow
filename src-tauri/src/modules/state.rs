use std::sync::Arc;
use parking_lot::Mutex;
use tokio::sync::mpsc;
use crate::modules::{inference::InferenceEngine, audio::SensitiveAudio};
use tauri_plugin_global_shortcut::{Modifiers, Code};

#[allow(dead_code)]
pub struct AppState {
    pub is_recording: Arc<Mutex<bool>>,
    pub tx_audio: Arc<Mutex<Option<mpsc::Sender<SensitiveAudio>>>>,
    pub inference_engine: Arc<InferenceEngine>,
    pub amplitude: Arc<Mutex<f32>>, 
    pub selected_device: Arc<Mutex<Option<String>>>,
    pub hotkey_modifiers: Arc<Mutex<Modifiers>>,
    pub hotkey_code: Arc<Mutex<Code>>,
    pub selected_model: Arc<Mutex<String>>,
}
