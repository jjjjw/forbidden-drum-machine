mod audio;
mod audio_output;
mod commands;
mod events;
mod sequencing;

use audio_output::AudioOutput;
use commands::{ClientCommand, ClientCommandQueue};
use events::{ServerEvent, ServerEventQueue};
use std::process::ExitCode;
use std::sync::Mutex;
use std::time::Duration;
use sysinfo::{Pid, System};
use tauri::{Emitter, Manager, State};

// App state containing only thread-safe communication channels
struct AppAudioState {
    command_queue: ClientCommandQueue,
}

type AppState = Mutex<AppAudioState>;

/// Starts the event emitter background process that forwards audio events to the frontend
fn start_event_emitter(
    event_receiver: crate::events::ServerEventReceiver,
    app_handle: tauri::AppHandle,
) {
    std::thread::spawn(move || {
        loop {
            event_receiver.process_events(|event| match event {
                ServerEvent::KickStepChanged(step) => {
                    let _ = app_handle.emit("kick_step_changed", step);
                }
                ServerEvent::ClapStepChanged(step) => {
                    let _ = app_handle.emit("clap_step_changed", step);
                }
                ServerEvent::ModulatorValues(delay, size, decay) => {
                    let _ = app_handle.emit("modulator_values", (delay, size, decay));
                }
                ServerEvent::KickPatternGenerated(pattern) => {
                    let _ = app_handle.emit("kick_pattern_generated", pattern.to_vec());
                }
                ServerEvent::ClapPatternGenerated(pattern) => {
                    let _ = app_handle.emit("clap_pattern_generated", pattern.to_vec());
                }
            });

            // Small sleep to avoid busy waiting
            std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
        }
    });
}

/// Starts CPU usage monitoring that reports every 10 seconds
fn start_cpu_monitor(app_handle: tauri::AppHandle) {
    std::thread::spawn(move || {
        let mut system = System::new();
        let current_pid = Pid::from_u32(std::process::id());

        loop {
            // Refresh process information
            system.refresh_processes();

            if let Some(process) = system.process(current_pid) {
                let cpu_usage = process.cpu_usage();
                let memory_usage = process.memory() / 1024 / 1024; // Convert to MB

                println!("CPU Usage: {:.1}%, Memory: {} MB", cpu_usage, memory_usage);

                // Emit to frontend
                let _ = app_handle.emit(
                    "cpu_usage",
                    serde_json::json!({
                        "cpu_percent": cpu_usage,
                        "memory_mb": memory_usage
                    }),
                );
            }

            // Sleep for 10 seconds
            std::thread::sleep(Duration::from_secs(10));
        }
    });
}

#[tauri::command]
fn send_audio_event(
    system_name: String,
    node_name: String,
    event_name: String,
    parameter: f32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let app_state = state.lock().unwrap();
    let sender = app_state.command_queue.sender();
    let client_event =
        crate::events::ClientEvent::new(&system_name, &node_name, &event_name, parameter);
    sender.send(ClientCommand::SendClientEvent(client_event));
    Ok(())
}

#[tauri::command]
fn switch_audio_system(system_name: String, state: State<'_, AppState>) -> Result<(), String> {
    let app_state = state.lock().unwrap();
    let sender = app_state.command_queue.sender();
    sender.send(ClientCommand::SwitchSystem(system_name));
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> ExitCode {
    // Initialize audio system in run() scope
    let command_queue = ClientCommandQueue::new();
    let event_queue = ServerEventQueue::new();

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
            send_audio_event,
            switch_audio_system
        ])
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // Start event emitter background process
            start_event_emitter(event_receiver, app_handle.clone());

            // Start CPU monitoring
            start_cpu_monitor(app_handle);

            // Manage only the communication channels
            app.manage(Mutex::new(AppAudioState { command_queue }));

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
