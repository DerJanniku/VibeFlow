use anyhow::Result;
use cpal::traits::StreamTrait;
use vibeflow::modules::{
    audio::AudioEngine, 
    inference::InferenceEngine, 
    llm::ContextEngine, 
    ui::UiManager, 
    os_integration::OSIntegration,
    gui::{self, AppStatus}
};
use rdev::{listen, EventType, Key};
use std::sync::Arc;
use tokio::sync::mpsc;
use parking_lot::Mutex;

fn main() -> Result<()> {
    println!("--- VibeFlow Pro | Cyberpunk Speech Overlay ---");

    // Shared State for GUI
    let app_status = Arc::new(Mutex::new(AppStatus::Ready));
    let amplitude = Arc::new(Mutex::new(0.0f32));
    let gui_visible = Arc::new(Mutex::new(false));

    let status_clone = app_status.clone();
    let amp_clone = amplitude.clone();
    let vis_clone = gui_visible.clone();

    // Start Processing Loop in a dedicated Tokio thread
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            if let Err(e) = run_processing_loop(status_clone, amp_clone, vis_clone).await {
                eprintln!("Processing Loop Error: {}", e);
            }
        });
    });

    // GUI must run on the main thread for most OS
    gui::run_gui(app_status, amplitude, gui_visible);

    Ok(())
}

use rodio::{OutputStream, Sink, Source};

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

async fn run_processing_loop(
    status: Arc<Mutex<AppStatus>>, 
    amplitude: Arc<Mutex<f32>>,
    visible: Arc<Mutex<bool>>
) -> Result<()> {
    let _tray = match UiManager::init_tray() {
        Ok(t) => Some(t),
        Err(_) => None,
    };

    let inference_engine = Arc::new(InferenceEngine::new());
    let is_recording = Arc::new(Mutex::new(false));
    let (tx_cmd, mut rx_cmd) = mpsc::channel(10);
    
    let tx_clone = tx_cmd.clone();
    std::thread::spawn(move || {
        // Toggle Logic State
        let mut _last_press = std::time::Instant::now(); 
        listen(move |event| {
            if let EventType::KeyPress(Key::F9) = event.event_type {
                // Simple debounce or just strict toggle on Press
                let _ = tx_clone.blocking_send("TOGGLE");
            }
        }).unwrap();
    });

    let mut current_tx_audio = None;
    let mut current_stream = None;
    let mut processing_handle = None;

    while let Some(cmd) = rx_cmd.recv().await {
        match cmd {
            "TOGGLE" => {
                let mut recording_guard = is_recording.lock();
                if *recording_guard {
                    // STOP RECORDING
                    *recording_guard = false;
                    *status.lock() = AppStatus::Processing;
                    
                    // Feedback: Low pitch for stop
                    play_feedback_sound(440.0);

                    current_tx_audio = None;
                    if let Some(stream) = current_stream.take() {
                        drop(stream);
                    }

                    if let Some(handle) = processing_handle.take() {
                        let transcript = handle.await?;
                        
                        let refined = ContextEngine::refine_text(&transcript).await?;
                        OSIntegration::paste_text(&refined)?;
                    }

                    *status.lock() = AppStatus::Ready;
                    *visible.lock() = false;
                } else {
                    // START RECORDING
                    *recording_guard = true;
                    *status.lock() = AppStatus::Recording;
                    *visible.lock() = true;

                    // Feedback: High pitch for start
                    play_feedback_sound(880.0);

                    let (tx_audio, rx_audio) = mpsc::channel(100);
                    current_tx_audio = Some(tx_audio);
                    
                    let stream_res = AudioEngine::start_stream(
                        current_tx_audio.as_ref().unwrap().clone(), 
                        is_recording.clone(),
                        amplitude.clone()
                    );
                    
                    match stream_res {
                        Ok(stream) => {
                            stream.play()?;
                            current_stream = Some(stream);
                        },
                        Err(_) => {
                            // Reset if failed
                            *recording_guard = false;
                            continue;
                        }
                    }

                    let engine = inference_engine.clone();
                    processing_handle = Some(tokio::spawn(async move {
                        engine.start_processing_loop(rx_audio).await
                    }));
                }
            },
            _ => {}
        }
    }
    Ok(())
}
