use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};
use tokio::sync::mpsc;
use crate::modules::audio::SensitiveAudio;
use zeroize::{Zeroize, ZeroizeOnDrop};

// Security: Protected Transcript
#[derive(Zeroize, ZeroizeOnDrop, Debug)]
pub struct SensitiveTranscript(String);

impl SensitiveTranscript {
    pub fn new(s: String) -> Self { Self(s) }
    pub fn as_str(&self) -> &str { &self.0 }
}

pub struct InferenceEngine {
    base_path: std::path::PathBuf,
}

impl InferenceEngine {
    pub fn new(app_data_dir: std::path::PathBuf) -> Self {
        Self {
            base_path: app_data_dir,
        }
    }

    pub async fn start_processing_loop(
        &self, 
        mut rx: mpsc::Receiver<SensitiveAudio>,
        model_filename: String
    ) -> SensitiveTranscript {
        let mut samples_buffer = Vec::new();
        let model_path = self.base_path.join(&model_filename);
        
        println!("[DEBUG] Inference Engine: Capturing audio stream...");

        while let Some(chunk) = rx.recv().await {
            samples_buffer.extend_from_slice(chunk.as_slice());
        }

        if samples_buffer.len() < 1600 {
            return SensitiveTranscript::new("".to_string());
        }

        println!("[DEBUG] Inference Engine: Processing {} samples with {}...", samples_buffer.len(), model_filename);

        if !model_path.exists() {
            println!("[ERROR] Whisper model not found at {:?}", model_path);
            return SensitiveTranscript::new(format!("Error: AI model not found. Please download {} in settings.", model_filename));
        }

        let ctx = WhisperContext::new_with_params(
            &model_path.to_string_lossy(),
            WhisperContextParameters::default()
        ).expect("failed to load model");

        let mut state = ctx.create_state().expect("failed to create state");
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        // Whisper expects 16kHz mono. AudioEngine provides this.
        state.full(params, &samples_buffer).expect("failed to run model");

        let mut result = String::new();
        let num_segments = state.full_n_segments().expect("failed to get segments");
        for i in 0..num_segments {
            if let Ok(segment) = state.full_get_segment_text(i) {
                result.push_str(&segment);
            }
        }

        println!("[DEBUG] Whisper Result: \"{}\"", result.trim());
        SensitiveTranscript::new(result.trim().to_string())
    }
}