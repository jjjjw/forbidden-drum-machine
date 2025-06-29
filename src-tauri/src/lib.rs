mod audio;
mod audio_output;
mod commands;
mod events;
mod sequencing;

use audio_output::AudioOutput;
use commands::{AudioCommand, AudioCommandQueue};
use events::AudioEventQueue;
use once_cell::sync::Lazy;

// Global command queue for UI -> Audio communication
static COMMAND_QUEUE: Lazy<AudioCommandQueue> = Lazy::new(|| AudioCommandQueue::new());

// Global event queue for Audio -> UI communication
static EVENT_QUEUE: Lazy<AudioEventQueue> = Lazy::new(|| AudioEventQueue::new());

// Audio output handle
static mut AUDIO_OUTPUT: Option<AudioOutput> = None;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
#[allow(static_mut_refs)]
fn start_audio(app_handle: tauri::AppHandle) -> Result<String, String> {
    unsafe {
        if AUDIO_OUTPUT.is_some() {
            AUDIO_OUTPUT = None; // Stop existing audio first
            std::thread::sleep(std::time::Duration::from_millis(100)); // Let it clean up
        }

        let command_receiver = COMMAND_QUEUE.receiver();
        let event_sender = EVENT_QUEUE.sender();
        let event_receiver = EVENT_QUEUE.receiver();
        match AudioOutput::new(command_receiver, event_sender, event_receiver, app_handle) {
            Ok(output) => {
                AUDIO_OUTPUT = Some(output);
                Ok("Audio started successfully".to_string())
            }
            Err(e) => Err(format!("Failed to start audio: {}", e)),
        }
    }
}

#[tauri::command]
fn stop_audio() -> Result<String, String> {
    let sender = COMMAND_QUEUE.sender();
    sender.send(AudioCommand::SetPaused(true));
    Ok("Audio paused".to_string())
}

#[tauri::command]
fn resume_audio() -> Result<String, String> {
    let sender = COMMAND_QUEUE.sender();
    sender.send(AudioCommand::SetPaused(false));
    Ok("Audio resumed".to_string())
}

#[tauri::command]
fn set_bpm(bpm: f32) -> Result<(), String> {
    let sender = COMMAND_QUEUE.sender();
    sender.send(AudioCommand::SetBpm(bpm));
    Ok(())
}

#[tauri::command]
fn set_kick_pattern(pattern: Vec<bool>) -> Result<(), String> {
    if pattern.len() != 16 {
        return Err("Pattern must be exactly 16 steps".to_string());
    }

    let mut array_pattern = [false; 16];
    array_pattern.copy_from_slice(&pattern);

    let sender = COMMAND_QUEUE.sender();
    sender.send(AudioCommand::SetKickPattern(array_pattern));
    Ok(())
}

#[tauri::command]
fn set_clap_pattern(pattern: Vec<bool>) -> Result<(), String> {
    if pattern.len() != 16 {
        return Err("Pattern must be exactly 16 steps".to_string());
    }

    let mut array_pattern = [false; 16];
    array_pattern.copy_from_slice(&pattern);

    let sender = COMMAND_QUEUE.sender();
    sender.send(AudioCommand::SetClapPattern(array_pattern));
    Ok(())
}

#[tauri::command]
fn set_delay_send(send: f32) -> Result<(), String> {
    let sender = COMMAND_QUEUE.sender();
    sender.send(AudioCommand::SetDelaySend(send));
    Ok(())
}

#[tauri::command]
fn set_reverb_send(send: f32) -> Result<(), String> {
    let sender = COMMAND_QUEUE.sender();
    sender.send(AudioCommand::SetReverbSend(send));
    Ok(())
}

#[tauri::command]
fn set_delay_freeze(freeze: bool) -> Result<(), String> {
    let sender = COMMAND_QUEUE.sender();
    sender.send(AudioCommand::SetDelayFreeze(freeze));
    Ok(())
}

#[tauri::command]
fn set_kick_attack(attack: f32) -> Result<(), String> {
    let sender = COMMAND_QUEUE.sender();
    sender.send(AudioCommand::SetKickAmpAttack(attack));
    Ok(())
}

#[tauri::command]
fn set_kick_release(release: f32) -> Result<(), String> {
    let sender = COMMAND_QUEUE.sender();
    sender.send(AudioCommand::SetKickAmpRelease(release));
    Ok(())
}

#[tauri::command]
fn set_clap_density(density: f32) -> Result<(), String> {
    let sender = COMMAND_QUEUE.sender();
    sender.send(AudioCommand::SetClapDensity(density));
    Ok(())
}

#[tauri::command]
fn set_kick_loop_bias(bias: f32) -> Result<(), String> {
    let sender = COMMAND_QUEUE.sender();
    sender.send(AudioCommand::SetKickLoopBias(bias));
    Ok(())
}

#[tauri::command]
fn set_clap_loop_bias(bias: f32) -> Result<(), String> {
    let sender = COMMAND_QUEUE.sender();
    sender.send(AudioCommand::SetClapLoopBias(bias));
    Ok(())
}

#[tauri::command]
fn generate_kick_pattern() -> Result<(), String> {
    let sender = COMMAND_QUEUE.sender();
    sender.send(AudioCommand::GenerateKickPattern);
    Ok(())
}

#[tauri::command]
fn generate_clap_pattern() -> Result<(), String> {
    let sender = COMMAND_QUEUE.sender();
    sender.send(AudioCommand::GenerateClapPattern);
    Ok(())
}

#[tauri::command]
fn set_kick_volume(volume: f32) -> Result<(), String> {
    let sender = COMMAND_QUEUE.sender();
    sender.send(AudioCommand::SetKickVolume(volume));
    Ok(())
}

#[tauri::command]
fn set_clap_volume(volume: f32) -> Result<(), String> {
    let sender = COMMAND_QUEUE.sender();
    sender.send(AudioCommand::SetClapVolume(volume));
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            start_audio,
            stop_audio,
            resume_audio,
            set_bpm,
            set_kick_pattern,
            set_clap_pattern,
            set_delay_send,
            set_reverb_send,
            set_delay_freeze,
            set_kick_attack,
            set_kick_release,
            set_clap_density,
            set_kick_loop_bias,
            set_clap_loop_bias,
            generate_kick_pattern,
            generate_clap_pattern,
            set_kick_volume,
            set_clap_volume
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
