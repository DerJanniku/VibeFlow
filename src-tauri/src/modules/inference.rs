use crate::modules::audio::SensitiveAudio;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
use zeroize::{Zeroize, ZeroizeOnDrop};

// Security: Protected Transcript
#[derive(Zeroize, ZeroizeOnDrop, Debug)]
pub struct SensitiveTranscript(String);

impl SensitiveTranscript {
    pub fn new(s: String) -> Self {
        Self(s)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
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
        rx: &mut mpsc::Receiver<SensitiveAudio>,
        model_filename: &str,
        app_handle: &AppHandle,
    ) -> SensitiveTranscript {
        let model_path = self.base_path.join(model_filename);

        if !model_path.exists() {
            println!("[ERROR] Whisper model not found at {:?}", model_path);
            return SensitiveTranscript::new(format!(
                "Error: AI model not found. Please download {} in settings.",
                model_filename
            ));
        }

        let ctx = WhisperContext::new_with_params(
            &model_path.to_string_lossy(),
            WhisperContextParameters::default(),
        )
        .expect("failed to load model");

        let mut state = ctx.create_state().expect("failed to create state");

        println!(
            "[DEBUG] Inference Loop Started for model: {}",
            model_filename
        );

        let mut samples_buffer = Vec::new();
        let mut full_transcript = String::new();
        let chunk_limit = 16000 * 30; // Hard limit 30s to prevent RAM explosion
        let mut last_inference_time = Instant::now();
        let inference_interval = Duration::from_millis(300); // Snappier ghost text

        loop {
            // We use a short timeout to check for "Silence" (Conversation End)
            // But we also want to process *while* receiving if enough time passed.
            match tokio::time::timeout(Duration::from_millis(200), rx.recv()).await {
                Ok(Some(chunk)) => {
                    samples_buffer.extend_from_slice(chunk.as_slice());

                    // Live Streaming / Ghost Text Logic
                    if samples_buffer.len() > 3200
                        && last_inference_time.elapsed() > inference_interval
                    {
                        // Run partial inference for Ghost Text
                        // We clone the buffer to not block the collector?
                        // Actually whisper runs on CPU, so it will block this thread.
                        // Since this is inside `spawn`, it blocks only this async task.
                        // `rx` buffer will hold incoming audio.

                        let partial_text = self.run_inference(&mut state, &samples_buffer);
                        // Emit Ghost Text
                        let _ = app_handle.emit("transcript_partial", &partial_text);
                        // println!("[DEBUG] Ghost: {}", partial_text); // customized logging
                        last_inference_time = Instant::now();
                    }

                    // Safety Clean
                    if samples_buffer.len() > chunk_limit {
                        println!("[WARN] Buffer overflow protection. Flushing.");
                        break;
                    }
                }
                Ok(None) => {
                    break;
                }
                Err(_) => {
                    // Timeout = Silence detected (User stopped speaking for > 200ms)
                    // In "Always On" mode, audio.rs STOPS sending data when silence/unflagged.
                    // So this timeout means "Recording Session Ended".
                    if !samples_buffer.is_empty() {
                        println!("[DEBUG] Silence detected. Finalizing transcription...");
                        let text = self.run_inference(&mut state, &samples_buffer);
                        full_transcript.push_str(&text);

                        // Clear Ghost Text on finish
                        let _ = app_handle.emit("transcript_partial", "");

                        break;
                    }
                }
            }
        }

        println!(
            "[DEBUG] Whisper Final Result: \"{}\"",
            full_transcript.trim()
        );
        SensitiveTranscript::new(full_transcript.trim().to_string())
    }

    fn run_inference(&self, state: &mut whisper_rs::WhisperState, samples: &[f32]) -> String {
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        
        // Performance Optimization: Use multiple threads
        // We have 16 cores, let's use 6-8 for speed without lagging the UI
        let threads = std::cmp::min(8, num_cpus::get() as i32);
        params.set_n_threads(threads);

        if let Err(e) = state.full(params, samples) {
            println!("[ERROR] Whisper Inference Failed: {}", e);
            return String::new();
        }

        let mut result = String::new();
        let num_segments = state.full_n_segments().unwrap_or(0);
        for i in 0..num_segments {
            if let Ok(segment) = state.full_get_segment_text(i) {
                result.push_str(&segment);
            }
        }
        result
    }
}
