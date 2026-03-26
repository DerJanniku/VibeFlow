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
            log::error!("[ERROR] Whisper model not found at {:?}", model_path);
            return SensitiveTranscript::new(format!(
                "Error: AI model not found. Please download {} in settings.",
                model_filename
            ));
        }

        let ctx = match WhisperContext::new_with_params(
            &model_path,
            WhisperContextParameters::default(),
        ) {
            Ok(c) => c,
            Err(e) => {
                log::error!("[ERROR] Failed to load Whisper model: {}", e);
                return SensitiveTranscript::new(format!(
                    "Error: Failed to load AI model ({}). It might be corrupted.",
                    e
                ));
            }
        };

        let mut state = match ctx.create_state() {
            Ok(s) => s,
            Err(e) => {
                log::error!("[ERROR] Failed to create Whisper state: {}", e);
                return SensitiveTranscript::new(format!(
                    "Error: Failed to initialize AI state ({}).",
                    e
                ));
            }
        };

        log::debug!(
            "[DEBUG] Inference Loop Started for model: {}",
            model_filename
        );

        let mut samples_buffer = Vec::new();
        let mut full_transcript = String::new();
        let chunk_limit = 16000 * 30; // Hard limit 30s to prevent RAM explosion
        let mut last_inference_time = Instant::now();
        let inference_interval = Duration::from_millis(100); // 100ms for INSTANT feedback (User requested "fast as fuck")

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
                        log::warn!("[WARN] Buffer overflow protection. Flushing.");
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
                        log::debug!("[DEBUG] Silence detected. Finalizing transcription...");
                        let text = self.run_inference(&mut state, &samples_buffer);
                        full_transcript.push_str(&text);

                        // Clear Ghost Text on finish
                        let _ = app_handle.emit("transcript_partial", "");

                        break;
                    }
                }
            }
        }

        // FINAL HALLUCINATION CHECK
        let final_text = full_transcript.trim();
        if final_text.eq_ignore_ascii_case("you")
            || final_text.eq_ignore_ascii_case("thank you")
            || final_text.is_empty()
        {
            log::debug!("[DEBUG] Discarding hallucination: '{}'", final_text);
            return SensitiveTranscript::new(String::new());
        }

        log::info!("[DEBUG] Whisper Final Result: \"{}\"", final_text);
        SensitiveTranscript::new(final_text.to_string())
    }

    fn run_inference(&self, state: &mut whisper_rs::WhisperState, samples: &[f32]) -> String {
        // ENERGY GATE (VAD Lite)
        // If audio is silence, don't run heavy inference
        let sum: f32 = samples.iter().map(|&x| x * x).sum();
        let rms = if !samples.is_empty() {
            (sum / samples.len() as f32).sqrt()
        } else {
            0.0
        };

        // LOG RMS to see if we are cutting off speech
        log::debug!("[DEBUG] Inference RMS: {:.5} (Threshold: 0.0003)", rms);

        if rms < 0.0003 {
            // Threshold for silence (LOWERED for Q9 Mic)
            // log::debug!("[DEBUG] Skipping inference (Silence, RMS={:.5})", rms);
            return String::new();
        }

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("de")); // German Support Requested
        params.set_translate(false); // Transcribe, don't translate to English
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        // Anti-Hallucination Params
        params.set_no_speech_thold(0.6); // Default is 0.6, consider raising?
        params.set_logprob_thold(-1.0);

        // Performance Optimization: UNLIMITED POWER
        // User requested "fast as fuck" -> Use ALL available cores
        let threads = num_cpus::get() as i32;
        params.set_n_threads(threads);

        if let Err(e) = state.full(params, samples) {
            log::error!("[ERROR] Whisper Inference Failed: {}", e);
            return String::new();
        }

        let mut result = String::new();
        let num_segments = state.full_n_segments();
        let mut full_raw = String::new();

        // Deduplication Logic
        let mut last_segment = String::new();

        for i in 0..num_segments {
            if let Some(seg_obj) = state.get_segment(i) {
                let segment = match seg_obj.to_str_lossy() {
                    Ok(s) => s.into_owned(),
                    Err(_) => continue,
                };
                full_raw.push_str(&segment);
                let clean_seg = segment.trim();

                // Filter common hallucinations
                if clean_seg.eq_ignore_ascii_case("you") || clean_seg.eq_ignore_ascii_case("thanks")
                {
                    continue;
                }
                // Filter repeats
                if clean_seg == last_segment {
                    continue;
                }

                result.push_str(&segment);
                last_segment = clean_seg.to_string();
            }
        }

        log::debug!(
            "[DEBUG] RAW WHISPER: '{}' -> FILTERED: '{}'",
            full_raw.trim(),
            result.trim()
        );
        result
    }
}
