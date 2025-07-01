mod audio;
mod audio_output;
mod commands;
mod events;
mod sequencing;

use audio_output::AudioOutput;
use commands::{AudioCommand, AudioCommandQueue};
use events::{AudioEvent, AudioEventQueue};
use std::sync::Mutex;
use tauri::{Emitter, Manager, State};
use std::process::ExitCode;
use std::time::Duration;

// App state containing only thread-safe communication channels
struct AppAudioState {
    command_queue: AudioCommandQueue,
    event_queue: AudioEventQueue,
}

type AppState = Mutex<AppAudioState>;

/// Starts the event emitter background process that forwards audio events to the frontend
fn start_event_emitter(event_receiver: crate::events::AudioEventReceiver, app_handle: tauri::AppHandle) {
    std::thread::spawn(move || {
        loop {
            event_receiver.process_events(|event| {
                match event {
                    AudioEvent::KickStepChanged(step) => {
                        let _ = app_handle.emit("kick_step_changed", step);
                    },
                    AudioEvent::ClapStepChanged(step) => {
                        let _ = app_handle.emit("clap_step_changed", step);
                    },
                    AudioEvent::ModulatorValues(delay, size, decay) => {
                        let _ = app_handle.emit("modulator_values", (delay, size, decay));
                    },
                    AudioEvent::KickPatternGenerated(pattern) => {
                        let _ = app_handle.emit("kick_pattern_generated", pattern.to_vec());
                    },
                    AudioEvent::ClapPatternGenerated(pattern) => {
                        let _ = app_handle.emit("clap_pattern_generated", pattern.to_vec());
                    },
                }
            });
            
            // Small sleep to avoid busy waiting
            std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
        }
    });
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn stop_audio(state: State<'_, AppState>) -> Result<String, String> {
    let app_state = state.lock().unwrap();
    let sender = app_state.command_queue.sender();
    sender.send(AudioCommand::SetPaused(true));
    Ok("Audio paused".to_string())
}

#[tauri::command]
fn resume_audio(state: State<'_, AppState>) -> Result<String, String> {
    let app_state = state.lock().unwrap();
    let sender = app_state.command_queue.sender();
    sender.send(AudioCommand::SetPaused(false));
    Ok("Audio resumed".to_string())
}

#[tauri::command]
fn set_bpm(bpm: f32, state: State<'_, AppState>) -> Result<(), String> {
    let app_state = state.lock().unwrap();
    let sender = app_state.command_queue.sender();
    sender.send(AudioCommand::SetBpm(bpm));
    Ok(())
}

#[tauri::command]
fn set_kick_pattern(pattern: Vec<bool>, state: State<'_, AppState>) -> Result<(), String> {
    if pattern.len() != 16 {
        return Err("Pattern must be exactly 16 steps".to_string());
    }

    let mut array_pattern = [false; 16];
    array_pattern.copy_from_slice(&pattern);

    let app_state = state.lock().unwrap();
    let sender = app_state.command_queue.sender();
    sender.send(AudioCommand::SetKickPattern(array_pattern));
    Ok(())
}

#[tauri::command]
fn set_clap_pattern(pattern: Vec<bool>, state: State<'_, AppState>) -> Result<(), String> {
    if pattern.len() != 16 {
        return Err("Pattern must be exactly 16 steps".to_string());
    }

    let mut array_pattern = [false; 16];
    array_pattern.copy_from_slice(&pattern);

    let app_state = state.lock().unwrap();
    let sender = app_state.command_queue.sender();
    sender.send(AudioCommand::SetClapPattern(array_pattern));
    Ok(())
}

#[tauri::command]
fn set_delay_send(send: f32, state: State<'_, AppState>) -> Result<(), String> {
    let app_state = state.lock().unwrap();
    let sender = app_state.command_queue.sender();
    sender.send(AudioCommand::SetDelaySend(send));
    Ok(())
}

#[tauri::command]
fn set_reverb_send(send: f32, state: State<'_, AppState>) -> Result<(), String> {
    let app_state = state.lock().unwrap();
    let sender = app_state.command_queue.sender();
    sender.send(AudioCommand::SetReverbSend(send));
    Ok(())
}

#[tauri::command]
fn set_delay_freeze(freeze: bool, state: State<'_, AppState>) -> Result<(), String> {
    let app_state = state.lock().unwrap();
    let sender = app_state.command_queue.sender();
    sender.send(AudioCommand::SetDelayFreeze(freeze));
    Ok(())
}

#[tauri::command]
fn set_kick_attack(attack: f32, state: State<'_, AppState>) -> Result<(), String> {
    let app_state = state.lock().unwrap();
    let sender = app_state.command_queue.sender();
    sender.send(AudioCommand::SetKickAmpAttack(attack));
    Ok(())
}

#[tauri::command]
fn set_kick_release(release: f32, state: State<'_, AppState>) -> Result<(), String> {
    let app_state = state.lock().unwrap();
    let sender = app_state.command_queue.sender();
    sender.send(AudioCommand::SetKickAmpRelease(release));
    Ok(())
}

#[tauri::command]
fn set_clap_density(density: f32, state: State<'_, AppState>) -> Result<(), String> {
    let app_state = state.lock().unwrap();
    let sender = app_state.command_queue.sender();
    sender.send(AudioCommand::SetClapDensity(density));
    Ok(())
}

#[tauri::command]
fn set_kick_loop_bias(bias: f32, state: State<'_, AppState>) -> Result<(), String> {
    let app_state = state.lock().unwrap();
    let sender = app_state.command_queue.sender();
    sender.send(AudioCommand::SetKickLoopBias(bias));
    Ok(())
}

#[tauri::command]
fn set_clap_loop_bias(bias: f32, state: State<'_, AppState>) -> Result<(), String> {
    let app_state = state.lock().unwrap();
    let sender = app_state.command_queue.sender();
    sender.send(AudioCommand::SetClapLoopBias(bias));
    Ok(())
}

#[tauri::command]
fn generate_kick_pattern(state: State<'_, AppState>) -> Result<(), String> {
    let app_state = state.lock().unwrap();
    let sender = app_state.command_queue.sender();
    sender.send(AudioCommand::GenerateKickPattern);
    Ok(())
}

#[tauri::command]
fn generate_clap_pattern(state: State<'_, AppState>) -> Result<(), String> {
    let app_state = state.lock().unwrap();
    let sender = app_state.command_queue.sender();
    sender.send(AudioCommand::GenerateClapPattern);
    Ok(())
}

#[tauri::command]
fn set_kick_volume(volume: f32, state: State<'_, AppState>) -> Result<(), String> {
    let app_state = state.lock().unwrap();
    let sender = app_state.command_queue.sender();
    sender.send(AudioCommand::SetKickVolume(volume));
    Ok(())
}

#[tauri::command]
fn set_clap_volume(volume: f32, state: State<'_, AppState>) -> Result<(), String> {
    let app_state = state.lock().unwrap();
    let sender = app_state.command_queue.sender();
    sender.send(AudioCommand::SetClapVolume(volume));
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> ExitCode {
    // Initialize audio system in run() scope
    let command_queue = AudioCommandQueue::new();
    let event_queue = AudioEventQueue::new();
    
    let command_receiver = command_queue.receiver();
    let event_sender = event_queue.sender();
    let event_receiver = event_queue.receiver();

    // Create AudioOutput - it will live for the duration of run()
    let _audio_output = match AudioOutput::new(command_receiver, event_sender) {
        Ok(output) => {
            println!("Audio system initialized successfully - drum machine is paused by default");
            output
        }
        Err(e) => {
            eprintln!("Failed to initialize audio system: {}", e);
            eprintln!("This is likely due to missing audio drivers or hardware");
            return ExitCode::FAILURE;
        }
    };


    let result = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
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
        .setup(move |app| {
            let app_handle = app.handle().clone();
            
            // Start event emitter background process
            start_event_emitter(event_receiver, app_handle);

            // Manage only the communication channels
            app.manage(Mutex::new(AppAudioState {
                command_queue,
                event_queue,
            }));
            
            Ok(())
        })
        .run(tauri::generate_context!());

    // When we get here, the Tauri app has shut down
    // AudioOutput (_audio_output) will be dropped and cleaned up properly
    
    match result {
        Ok(_) => {
            println!("Application exited normally");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Application error: {}", e);
            ExitCode::FAILURE
        }
    }
}
