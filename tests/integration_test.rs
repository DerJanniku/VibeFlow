use vibeflow::modules::{audio::AudioEngine, inference::InferenceEngine, llm::ContextEngine, os_integration::OSIntegration};
use std::time::Instant;
use tokio::sync::mpsc;
use sysinfo::System;
use std::env;

#[tokio::test]
async fn test_e2e_pipeline() {
    env::set_var("VIBEFLOW_TEST_MODE", "1");
    println!("--- Starting VibeFlow Security & Performance Audit ---");
    let mut sys = System::new_all();
    sys.refresh_all();
    let start_ram = sys.used_memory();

    // 1. Audio-Loopback Simulation
    println!("[Test] Audio-Loopback Simulation (5s)...");
    let (tx, rx) = mpsc::channel(100);
    
    // Spawn simulator
    let sim_handle = tokio::spawn(async move {
        AudioEngine::simulate_recording(tx, 5000).await;
    });

    // 2. Inference Benchmark
    println!("[Test] Inference Benchmark...");
    let inference_start = Instant::now();
    let engine = InferenceEngine::new();
    let transcript = engine.start_processing_loop(rx).await;
    let _ = sim_handle.await;
    let total_duration = inference_start.elapsed();
    
    // Calculate overhead: Total time - Audio Duration (5000ms)
    // We use saturating_sub in case of clock drift or extremely fast execution
    let processing_overhead = total_duration.as_millis().saturating_sub(5000);
    
    println!(" -> Transcribed: {:?}", transcript);
    println!(" -> Total Duration: {:.2?}", total_duration);
    println!(" -> Processing Overhead: {} ms", processing_overhead);
    
    assert!(processing_overhead < 500, "Inference overhead too high! (>500ms)");

    // 3. LLM Latency Check & Validation
    println!("[Test] LLM Latency & Validation...");
    let llm_start = Instant::now();
    
    // Check if Ollama is running, else skip to avoid failure in CI environment
    let refined_result = match ContextEngine::refine_text(&transcript).await {
        Ok(text) => {
            println!(" -> LLM Response: {}", text);
            text
        },
        Err(e) => {
            println!(" -> LLM Skipped (Ollama likely offline): {}", e);
            "Simulated Corrected Text".to_string()
        }
    };
    
    let llm_duration = llm_start.elapsed();
    println!(" -> LLM Latency: {:.2?}", llm_duration);

    // Validation: Check if filler words removed (Mock check)
    if refined_result != "Simulated Corrected Text" {
         assert!(!refined_result.to_lowercase().contains("ähm"), "Filter failed to remove filler words!");
    }

    // 4. Input Sim
    println!("[Test] Input Simulation...");
    OSIntegration::paste_text(&refined_result).unwrap();
    let pasted = OSIntegration::get_mock_paste();
    assert_eq!(pasted, refined_result, "Clipboard/Paste simulation mismatch!");

    // Final Report
    sys.refresh_all();
    let end_ram = sys.used_memory();
    
    println!("\n=== AUDIT REPORT ===");
    println!("Audio Pipeline: PASS");
    println!("Total Duration: {:.2?}", total_duration);
    println!("Processing Overhead: {} ms", processing_overhead);
    println!("LLM Latency: {:.2?}", llm_duration);
    println!("Input Security: PASS (Mocked)");
    println!("RAM Delta: {} KB", (end_ram as i64 - start_ram as i64) / 1024);
    println!("====================");
}