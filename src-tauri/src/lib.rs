mod audio;
mod audio_output;

use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use audio::systems::DrumMachine;
use audio_output::AudioOutput;

// Global drum machine instance
static DRUM_MACHINE: Lazy<Arc<Mutex<DrumMachine>>> = Lazy::new(|| {
    Arc::new(Mutex::new(DrumMachine::new()))
});

static mut AUDIO_OUTPUT: Option<AudioOutput> = None;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
#[allow(static_mut_refs)]
fn start_audio() -> Result<String, String> {
    unsafe {
        if AUDIO_OUTPUT.is_some() {
            AUDIO_OUTPUT = None; // Stop existing audio first
            std::thread::sleep(std::time::Duration::from_millis(100)); // Let it clean up
        }
        
        match AudioOutput::new(DRUM_MACHINE.clone()) {
            Ok(output) => {
                AUDIO_OUTPUT = Some(output);
                Ok("Audio started successfully".to_string())
            }
            Err(e) => Err(format!("Failed to start audio: {}", e)),
        }
    }
}

#[tauri::command]
#[allow(static_mut_refs)]
fn stop_audio() -> Result<String, String> {
    unsafe {
        if AUDIO_OUTPUT.is_some() {
            AUDIO_OUTPUT = None;
            Ok("Audio stopped".to_string())
        } else {
            Ok("Audio was not running".to_string())
        }
    }
}

#[tauri::command] 
fn set_bpm(bpm: f32) -> Result<(), String> {
    if let Ok(mut drum_machine) = DRUM_MACHINE.try_lock() {
        drum_machine.set_bpm(bpm);
        Ok(())
    } else {
        Err("Could not access drum machine".to_string())
    }
}

#[tauri::command]
fn set_kick_pattern(pattern: Vec<bool>) -> Result<(), String> {
    if pattern.len() != 16 {
        return Err("Pattern must be exactly 16 steps".to_string());
    }
    
    if let Ok(mut drum_machine) = DRUM_MACHINE.try_lock() {
        let mut array_pattern = [false; 16];
        array_pattern.copy_from_slice(&pattern);
        drum_machine.set_kick_pattern(array_pattern);
        Ok(())
    } else {
        Err("Could not access drum machine".to_string())
    }
}

#[tauri::command]
fn set_snare_pattern(pattern: Vec<bool>) -> Result<(), String> {
    if pattern.len() != 16 {
        return Err("Pattern must be exactly 16 steps".to_string());
    }
    
    if let Ok(mut drum_machine) = DRUM_MACHINE.try_lock() {
        let mut array_pattern = [false; 16];
        array_pattern.copy_from_slice(&pattern);
        drum_machine.set_snare_pattern(array_pattern);
        Ok(())
    } else {
        Err("Could not access drum machine".to_string())
    }
}

#[tauri::command]
fn get_current_step() -> Result<u8, String> {
    if let Ok(drum_machine) = DRUM_MACHINE.try_lock() {
        Ok(drum_machine.get_current_step())
    } else {
        Err("Could not access drum machine".to_string())
    }
}

#[tauri::command]
fn get_modulator_values() -> Result<(f32, f32, f32), String> {
    if let Ok(drum_machine) = DRUM_MACHINE.try_lock() {
        let delay_time = drum_machine.get_current_delay_time();
        let reverb_size = drum_machine.get_current_reverb_size();
        let reverb_decay = drum_machine.get_current_reverb_decay();
        Ok((delay_time, reverb_size, reverb_decay))
    } else {
        Err("Could not access drum machine".to_string())
    }
}

#[tauri::command]
fn set_delay_send(send: f32) -> Result<(), String> {
    if let Ok(mut drum_machine) = DRUM_MACHINE.try_lock() {
        drum_machine.set_delay_send(send);
        Ok(())
    } else {
        Err("Could not access drum machine".to_string())
    }
}

#[tauri::command]
fn set_reverb_send(send: f32) -> Result<(), String> {
    if let Ok(mut drum_machine) = DRUM_MACHINE.try_lock() {
        drum_machine.set_reverb_send(send);
        Ok(())
    } else {
        Err("Could not access drum machine".to_string())
    }
}

#[tauri::command]
fn set_delay_freeze(freeze: bool) -> Result<(), String> {
    if let Ok(mut drum_machine) = DRUM_MACHINE.try_lock() {
        drum_machine.set_delay_freeze(freeze);
        Ok(())
    } else {
        Err("Could not access drum machine".to_string())
    }
}

#[tauri::command]
fn set_kick_attack(attack: f32) -> Result<(), String> {
    if let Ok(mut drum_machine) = DRUM_MACHINE.try_lock() {
        drum_machine.get_kick().set_amp_attack(attack);
        Ok(())
    } else {
        Err("Could not access drum machine".to_string())
    }
}

#[tauri::command]
fn set_kick_release(release: f32) -> Result<(), String> {
    if let Ok(mut drum_machine) = DRUM_MACHINE.try_lock() {
        drum_machine.get_kick().set_amp_release(release);
        Ok(())
    } else {
        Err("Could not access drum machine".to_string())
    }
}

#[tauri::command]
fn set_snare_attack(attack: f32) -> Result<(), String> {
    if let Ok(mut drum_machine) = DRUM_MACHINE.try_lock() {
        drum_machine.get_snare().set_amp_attack(attack);
        Ok(())
    } else {
        Err("Could not access drum machine".to_string())
    }
}

#[tauri::command]
fn set_snare_release(release: f32) -> Result<(), String> {
    if let Ok(mut drum_machine) = DRUM_MACHINE.try_lock() {
        drum_machine.get_snare().set_amp_release(release);
        Ok(())
    } else {
        Err("Could not access drum machine".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet, 
            start_audio, 
            stop_audio,
            set_bpm, 
            set_kick_pattern, 
            set_snare_pattern,
            get_current_step,
            get_modulator_values,
            set_delay_send,
            set_reverb_send,
            set_delay_freeze,
            set_kick_attack,
            set_kick_release,
            set_snare_attack,
            set_snare_release
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
