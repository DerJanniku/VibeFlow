use anyhow::{Result, anyhow};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::Arc;
use tokio::sync::mpsc;
use parking_lot::Mutex;
use std::fmt;

// Audio constants
const SAMPLE_RATE: u32 = 16000;
const CHUNK_SIZE_MS: usize = 30;
const FRAME_SIZE: usize = (SAMPLE_RATE as usize * CHUNK_SIZE_MS) / 1000;

// Security: Protected Audio Buffer
#[derive(zeroize::Zeroize, zeroize::ZeroizeOnDrop)]
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
        is_recording: Arc<Mutex<bool>>,
        amplitude: Arc<Mutex<f32>>,
        device_name: Option<String>,
    ) -> Result<cpal::Stream> {
        let host = cpal::default_host();
        let device = if let Some(name) = device_name {
            println!("[DEBUG] Attempting to open device: \"{}\"", name);
            host.input_devices()?
                .find(|x| x.name().map(|n| n == name).unwrap_or(false))
                .or_else(|| {
                    println!("[DEBUG] Device \"{}\" not found by name, trying fallback search...", name);
                    host.input_devices().ok().and_then(|mut d| d.find(|x| {
                        let n = x.name().unwrap_or_default().to_uppercase();
                        n.contains("Q9") || n.contains("GENERIC") || n.contains("USB")
                    }))
                })
                .ok_or_else(|| anyhow!("Device not found"))?
        } else {
            println!("[DEBUG] Using default input device");
            host.default_input_device()
                .ok_or_else(|| anyhow!("No default input device found"))?
        };

        let config = device.default_input_config()?;
        println!("[DEBUG] RAW Audio Device Config: {:?}", config);
        
        let source_sample_rate = config.sample_rate().0 as f32;
        let source_channels = config.channels() as usize;
        let target_sample_rate = 16000.0;
        
        let tx_clone = tx.clone();
        let amp_clone = amplitude.clone();
        let is_rec_clone = is_recording.clone();
        
        // State for resampling
        let mut resample_buffer = Vec::new();
        let mut last_sample_pos = 0.0;

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _: &_| {
                        if !*is_rec_clone.lock() { return; }
                        
                        // 1. Calculate amplitude for visualization (RMS)
                        let sum: f32 = data.iter().map(|&x| x * x).sum();
                        let rms = if !data.is_empty() { (sum / data.len() as f32).sqrt() } else { 0.0 };
                        *amp_clone.lock() = rms;

                        // 2. Downmix to Mono
                        let mut mono_data = Vec::with_capacity(data.len() / source_channels);
                        for chunk in data.chunks_exact(source_channels) {
                            let sum: f32 = chunk.iter().sum();
                            mono_data.push(sum / source_channels as f32);
                        }

                        // 3. Resample to 16kHz (Linear Interpolation)
                        // This is a simple but effective live resampler
                        let mut processed_chunk = Vec::new();
                        let ratio = source_sample_rate / target_sample_rate;
                        
                        for sample in mono_data {
                            resample_buffer.push(sample);
                            
                            while last_sample_pos < resample_buffer.len() as f32 - 1.0 {
                                let idx = last_sample_pos as usize;
                                let frac = last_sample_pos - idx as f32;
                                
                                // Linear interpolation
                                let s1 = resample_buffer[idx];
                                let s2 = resample_buffer[idx + 1];
                                let interpolated = s1 + (s2 - s1) * frac;
                                
                                processed_chunk.push(interpolated);
                                last_sample_pos += ratio;
                            }
                        }

                        // Keep buffer small (only what we need for next interp)
                        let drain_amt = last_sample_pos.floor() as usize;
                        if drain_amt > 0 && resample_buffer.len() > drain_amt {
                            resample_buffer.drain(0..drain_amt);
                            last_sample_pos -= drain_amt as f32;
                        }

                        if !processed_chunk.is_empty() {
                            let _ = tx_clone.try_send(SensitiveAudio::new(processed_chunk));
                        }
                    },
                    |err| println!("[ERROR] Audio stream error: {}", err),
                    None
                )?
            },
            _ => return Err(anyhow!("Unsupported sample format. Only F32 is supported by the current driver.")),
        };

        Ok(stream)
    }

    pub fn list_input_devices() -> Result<Vec<String>> {
        let host = cpal::default_host();
        let devices = match host.input_devices() {
            Ok(d) => d,
            Err(e) => {
                println!("[ERROR] Failed to get input devices: {}", e);
                return Ok(vec!["Default Microphone".to_string()]);
            }
        };
        
        let mut names = Vec::new();
        println!("[DEBUG] CPAL Host: {}", host.id().name());
        
        for device in devices {
            if let Ok(name) = device.name() {
                println!("[DEBUG] RAW DEVICE NAME: \"{}\"", name);
                
                let mut display_name = name.clone();
                if name.to_uppercase().contains("Q9") {
                    display_name = "Q9 Microphone 🎙️".to_string();
                } else if name.starts_with("sysdefault:CARD=") || name.starts_with("default:CARD=") {
                    display_name = name
                        .replace("sysdefault:CARD=", "")
                        .replace("default:CARD=", "")
                        .split(',')
                        .next()
                        .unwrap_or(&name)
                        .to_string();
                }

                if !names.contains(&display_name) {
                    names.push(display_name);
                }
            } else {
                println!("[DEBUG] Could not query name for a device");
            }
        }
        
        if names.is_empty() {
            names.push("Default Microphone".to_string());
        }
        
        println!("[DEBUG] Final selection for UI: {:?}", names);
        Ok(names)
    }
}