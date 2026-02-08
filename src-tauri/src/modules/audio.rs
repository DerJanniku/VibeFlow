use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait};
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::fmt;
use std::sync::Arc;
use tokio::sync::mpsc;
// VAD & Noise Suppression would be integrated here
// For Phase 1, we will implement the Ring Buffer logic first as the Core "Rewind" mechanic.
// The VAD integration with 'tract' requires loading an ONNX model file.
// To keep Phase 1 pure "Core Audio Refactor", we will implement the Ring Buffer and prepare the struct for VAD.

// Audio constants
const SAMPLE_RATE: u32 = 16000;
const PRE_RECORD_SECONDS: usize = 3;
// 16000 samples/sec * 3 sec = 48000 samples
const RING_BUFFER_SIZE: usize = (SAMPLE_RATE as usize) * PRE_RECORD_SECONDS;

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
        let host = if cfg!(target_os = "linux") {
            println!("[DEBUG] Linux detected - checking audio backends...");
            
            // 1. Try Jack (Preferred for Pro Audio)
            let jack_host = cpal::host_from_id(cpal::HostId::Jack).ok();
            let mut use_jack = false;
            
            if let Some(ref h) = jack_host {
                 // Verify if Jack server is actually running by probing devices
                 match h.input_devices() {
                     Ok(mut devices) => {
                         if devices.next().is_some() {
                             println!("[DEBUG] Audio Host: Jack is ACTIVE and has devices.");
                             use_jack = true;
                         } else {
                             println!("[DEBUG] Audio Host: Jack found but has NO devices (Server likely down).");
                         }
                     },
                     Err(e) => {
                         println!("[DEBUG] Audio Host: Jack probe failed: {}", e);
                     }
                 }
            }

            if use_jack {
                jack_host.unwrap()
            } else {
                println!("[DEBUG] Audio Host: Falling back to System Default (ALSA/Pulse)");
                cpal::default_host()
            }
        } else {
            cpal::default_host()
        };

        // 3. Define the Stream Builder (Closure)
        let build_stream_fn = |device: &cpal::Device| -> Result<cpal::Stream> {
            let name = device.name().unwrap_or("unknown".to_string());
            println!("[DEBUG] Trying device: {}", name);
            
            // SMART CONFIG SELECTION (Universal)
            let supported_configs = device.supported_input_configs()?;
            let mut selected_config = None;
            
            for config_range in supported_configs {
                 let entry = config_range.with_max_sample_rate();
                 if selected_config.is_none() {
                     selected_config = Some(entry);
                     continue;
                 }
                 let current = selected_config.as_ref().unwrap();
                 if entry.sample_format() == cpal::SampleFormat::F32 && current.sample_format() != cpal::SampleFormat::F32 {
                     selected_config = Some(entry);
                 }
            }

            let config = selected_config
                .ok_or_else(|| anyhow!("No supported input config found"))?;
                
            let source_sample_rate = config.sample_rate().0 as f32;
            let source_channels = config.channels() as usize;
            let target_sample_rate = 16000.0;
            
            let tx_clone = tx.clone();
            let amp_clone = amplitude.clone();
            let is_rec_clone = is_recording.clone();
            let mut resample_buffer = Vec::new();
            let mut last_sample_pos = 0.0;
            let mut ring_buffer: VecDeque<f32> = VecDeque::with_capacity(RING_BUFFER_SIZE);
            let mut was_recording = false;
            let err_fn = |err| println!("[ERROR] Audio stream error: {}", err);

            match config.sample_format() {
                cpal::SampleFormat::F32 => {
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[f32], _: &_| {
                             Self::process_audio_chunk(
                                 data, &tx_clone, &is_rec_clone, &amp_clone, 
                                 source_channels, source_sample_rate, target_sample_rate, 
                                 &mut resample_buffer, &mut last_sample_pos, &mut ring_buffer, &mut was_recording
                            );
                        },
                        err_fn, None
                    ).map_err(|e| anyhow!(e))
                },
                cpal::SampleFormat::I16 => {
                    device.build_input_stream(
                        &config.into(),
                        move |data: &[i16], _: &_| {
                            let float_data: Vec<f32> = data.iter().map(|&x| x as f32 / 32768.0).collect();
                             Self::process_audio_chunk(
                                 &float_data, &tx_clone, &is_rec_clone, &amp_clone, 
                                 source_channels, source_sample_rate, target_sample_rate, 
                                 &mut resample_buffer, &mut last_sample_pos, &mut ring_buffer, &mut was_recording
                            );
                        },
                        err_fn, None
                    ).map_err(|e| anyhow!(e))
                },
                cpal::SampleFormat::U16 => {
                     device.build_input_stream(
                        &config.into(),
                        move |data: &[u16], _: &_| {
                            let float_data: Vec<f32> = data.iter().map(|&x| (x as f32 - 32768.0) / 32768.0).collect();
                             Self::process_audio_chunk(
                                 &float_data, &tx_clone, &is_rec_clone, &amp_clone, 
                                 source_channels, source_sample_rate, target_sample_rate, 
                                 &mut resample_buffer, &mut last_sample_pos, &mut ring_buffer, &mut was_recording
                            );
                        },
                        err_fn, None
                    ).map_err(|e| anyhow!(e))
                }
                _ => Err(anyhow!("Unsupported sample format"))
            }
        };

        // 4. Execute Device Selection Strategy (STRICT OS SEPARATION)
        
        #[cfg(target_os = "linux")]
        {
            println!("[DEBUG] Using LINUX-Specific Device Selection Strategy (Auto-Healing)");
            // A. Specific Device
            if let Some(name) = device_name {
                 let device = host.input_devices()?
                    .find(|x| x.name().map(|n| n == name).unwrap_or(false))
                    .ok_or(anyhow!("Device not found"))?;
                return build_stream_fn(&device);
            }
            // B. Default
            println!("[DEBUG] Trying Default Device...");
            if let Some(default_device) = host.default_input_device() {
                if let Ok(stream) = build_stream_fn(&default_device) {
                    println!("[DEBUG] Host Default worked.");
                    return Ok(stream);
                }
            }
            // C. Auto-Heal (Loop All)
            println!("[DEBUG] Default failed. Starting Auto-Healing...");
            if let Ok(devices) = host.input_devices() {
                for device in devices {
                    if let Ok(stream) = build_stream_fn(&device) {
                         println!("[DEBUG] Auto-Healing connected to: {:?}", device.name().unwrap_or_default());
                         return Ok(stream);
                    }
                }
            }
            Err(anyhow!("CRITICAL: No working audio device found on Linux."))
        }

        #[cfg(target_os = "windows")]
        {
            println!("[DEBUG] Using WINDOWS-Specific Device Selection Strategy (Standard)");
            // A. Specific Device
            if let Some(name) = device_name {
                 let device = host.input_devices()?
                    .find(|x| x.name().map(|n| n == name).unwrap_or(false))
                    .ok_or(anyhow!("Device not found"))?;
                return build_stream_fn(&device);
            }
            // B. Default Only (Standard behavior)
            let device = host.default_input_device()
                .ok_or_else(|| anyhow!("No default input device found"))?;
            build_stream_fn(&device)
        }

        #[cfg(target_os = "macos")]
        {
             // MacOS Strategy (Similar to Windows)
            let device = host.default_input_device()
                .ok_or_else(|| anyhow!("No default input device found"))?;
            build_stream_fn(&device)
        }
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
                } else if name.starts_with("sysdefault:CARD=") || name.starts_with("default:CARD=")
                {
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

    fn process_audio_chunk(
        data: &[f32],
        tx: &mpsc::Sender<SensitiveAudio>,
        is_rec: &Arc<Mutex<bool>>,
        amp: &Arc<Mutex<f32>>,
        channels: usize,
        src_rate: f32,
        dst_rate: f32,
        resample_buf: &mut Vec<f32>,
        last_pos: &mut f32,
        ring_buf: &mut VecDeque<f32>,
        was_rec: &mut bool
    ) {
        let recording_now = *is_rec.lock();

        // 1. Amplitude (RMS)
        let sum: f32 = data.iter().map(|&x| x * x).sum();
        let rms = if !data.is_empty() {
            (sum / data.len() as f32).sqrt()
        } else {
            0.0
        };
        *amp.lock() = rms;

        // 2. Downmix
        let mut mono_data = Vec::with_capacity(data.len() / channels);
        for chunk in data.chunks_exact(channels) {
            let sum: f32 = chunk.iter().sum();
            mono_data.push(sum / channels as f32);
        }

        // 3. Resample
        let mut processed_chunk = Vec::new();
        let ratio = src_rate / dst_rate;

        for sample in mono_data {
            resample_buf.push(sample);

            while *last_pos < resample_buf.len() as f32 - 1.0 {
                let idx = *last_pos as usize;
                let frac = *last_pos - idx as f32;

                let s1 = resample_buf[idx];
                let s2 = resample_buf[idx + 1];
                let interpolated = s1 + (s2 - s1) * frac;

                processed_chunk.push(interpolated);
                *last_pos += ratio;
            }
        }

        let drain_amt = last_pos.floor() as usize;
        if drain_amt > 0 && resample_buf.len() > drain_amt {
            resample_buf.drain(0..drain_amt);
            *last_pos -= drain_amt as f32;
        }

        if !processed_chunk.is_empty() {
             // --- REWIND LOGIC ---
             for &sample in &processed_chunk {
                 if ring_buf.len() >= RING_BUFFER_SIZE {
                     ring_buf.pop_front();
                 }
                 ring_buf.push_back(sample);
             }

             if recording_now && !*was_rec {
                 let history: Vec<f32> = ring_buf.iter().cloned().collect();
                 let _ = tx.try_send(SensitiveAudio::new(history));
             }

             if recording_now {
                 let _ = tx.try_send(SensitiveAudio::new(processed_chunk));
             }

             *was_rec = recording_now;
        }
    }
}
