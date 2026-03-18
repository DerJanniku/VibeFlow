use anyhow::{Result, anyhow};
use cpal::traits::{DeviceTrait, HostTrait};
use std::sync::Arc;
use tokio::sync::mpsc;
use webrtc_vad::{Vad, VadMode};
use zeroize::{Zeroize, ZeroizeOnDrop};
use std::fmt;

// Audio constants
const SAMPLE_RATE: u32 = 16000;
const CHUNK_SIZE_MS: usize = 30;
const FRAME_SIZE: usize = (SAMPLE_RATE as usize * CHUNK_SIZE_MS) / 1000;

// Security: Protected Audio Buffer
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SensitiveAudio(Vec<f32>);

impl fmt::Debug for SensitiveAudio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<Protected Audio Data: {} samples>", self.0.len())
    }
}

impl SensitiveAudio {
    pub fn new(data: Vec<f32>) -> Self {
        Self(data)
    }
    pub fn as_slice(&self) -> &[f32] {
        &self.0
    }
}

pub struct AudioEngine;

impl AudioEngine {
    pub fn start_stream(
        tx: mpsc::Sender<SensitiveAudio>, 
        is_recording: Arc<parking_lot::Mutex<bool>>,
        amplitude: Arc<parking_lot::Mutex<f32>>
    ) -> Result<cpal::Stream> {
        let host = cpal::default_host();
        let device = host.default_input_device().ok_or_else(|| anyhow!("No input device"))?;
        let config = device.default_input_config()?;
        
        let stream_config: cpal::StreamConfig = config.clone().into();
        let err_fn = |err| eprintln!("Stream error: {}", err);
        let tx_clone = tx.clone();
        
        let internal_buffer = Arc::new(parking_lot::Mutex::new(Vec::with_capacity(FRAME_SIZE * 2)));
        let buf_ref = internal_buffer.clone();

        let mut _vad = Vad::new_with_rate_and_mode(
            webrtc_vad::SampleRate::Rate16kHz, 
            VadMode::Aggressive
        );

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if *is_recording.lock() {
                        // Calculate RMS for GUI
                        let rms = (data.iter().map(|&x| x * x).sum::<f32>() / data.len() as f32).sqrt();
                        *amplitude.lock() = rms;

                        let mut buf = buf_ref.lock();
                        buf.extend_from_slice(data);
                        
                        if let Err(_) = tx_clone.try_send(SensitiveAudio::new(data.to_vec())) {
                        }
                    } else {
                        *amplitude.lock() = 0.0;
                    }
                },
                err_fn,
                None
            )?,
            _ => return Err(anyhow!("Unsupported sample format (F32 required for this pipeline)")),
        };

        Ok(stream)
    }

    pub async fn simulate_recording(tx: mpsc::Sender<SensitiveAudio>, duration_ms: u64) {
        let chunks = duration_ms / 30; 
        for _ in 0..chunks {
            let noise: Vec<f32> = (0..FRAME_SIZE).map(|_| 0.001).collect();
            let _ = tx.send(SensitiveAudio::new(noise)).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
        }
    }
}