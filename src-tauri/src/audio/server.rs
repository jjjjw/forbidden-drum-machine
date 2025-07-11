use crate::audio::AudioSystem;
use std::collections::HashMap;

/// Global audio server that manages multiple audio systems with smooth switching
pub struct AudioServer {
    /// Registered systems by name
    systems: HashMap<String, Box<dyn AudioSystem>>,
    
    /// Currently active system
    current_system: Option<String>,
    
    /// System being faded out during transition
    fading_system: Option<String>,
    
    /// Fade counter for smooth transitions (in samples)
    fade_counter: u32,
    
    /// Fade duration in samples (e.g., 1024 samples for quick fade)
    fade_duration: u32,
    
    /// Sample rate
    sample_rate: f32,
}

impl AudioServer {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            systems: HashMap::new(),
            current_system: None,
            fading_system: None,
            fade_counter: 0,
            fade_duration: 1024, // Quick fade ~23ms at 44.1kHz
            sample_rate,
        }
    }
    
    /// Add a system to the server
    pub fn add_system(&mut self, name: String, mut system: Box<dyn AudioSystem>) {
        system.set_sample_rate(self.sample_rate);
        self.systems.insert(name, system);
    }
    
    /// Switch to a different system with fade-out
    pub fn switch_to_system(&mut self, name: &str) -> Result<(), String> {
        if !self.systems.contains_key(name) {
            return Err(format!("System '{}' not found", name));
        }
        
        // Start fade if we have a current system
        if let Some(current) = &self.current_system {
            if current != name {
                self.fading_system = Some(current.clone());
                self.fade_counter = 0;
            }
        }
        
        // Set new current system
        self.current_system = Some(name.to_string());
        Ok(())
    }
    
    /// Get the name of the current system
    pub fn get_current_system(&self) -> Option<&str> {
        self.current_system.as_deref()
    }
    
    /// Process audio through the system(s) with crossfading
    pub fn process_stereo(&mut self, left_in: f32, right_in: f32) -> (f32, f32) {
        let mut output = (0.0, 0.0);
        
        // Process current system
        if let Some(current_name) = &self.current_system {
            if let Some(current_system) = self.systems.get_mut(current_name) {
                output = current_system.process_stereo(left_in, right_in);
            }
        }
        
        // Process fading system if active
        if let Some(fading_name) = &self.fading_system {
            if let Some(fading_system) = self.systems.get_mut(fading_name) {
                let fading_output = fading_system.process_stereo(left_in, right_in);
                
                // Calculate fade coefficient (1.0 to 0.0)
                let fade_progress = self.fade_counter as f32 / self.fade_duration as f32;
                let fade_coefficient = (1.0 - fade_progress).max(0.0);
                
                // Mix fading output with current output
                output.0 += fading_output.0 * fade_coefficient;
                output.1 += fading_output.1 * fade_coefficient;
                
                // Update fade counter
                self.fade_counter += 1;
                
                // Stop fading when complete
                if self.fade_counter >= self.fade_duration {
                    self.fading_system = None;
                    self.fade_counter = 0;
                }
            }
        }
        
        output
    }
    
    
    /// Send a set sequence command to a specific system
    pub fn send_set_sequence(
        &mut self,
        system_name: &str,
        sequence_config: &serde_json::Value,
    ) -> Result<(), String> {
        if let Some(system) = self.systems.get_mut(system_name) {
            system.set_sequence(sequence_config)
        } else {
            Err(format!("System '{}' not found", system_name))
        }
    }
    
    /// Set sample rate for all systems
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        // Update fade duration to maintain the same time duration
        self.fade_duration = (0.023 * sample_rate) as u32; // ~23ms fade
        
        for system in self.systems.values_mut() {
            system.set_sample_rate(sample_rate);
        }
    }
    
    /// Get list of registered system names
    pub fn get_system_names(&self) -> Vec<&str> {
        self.systems.keys().map(|s| s.as_str()).collect()
    }
    
    /// Send a node event to a specific system
    pub fn send_node_event(
        &mut self,
        system_name: &str,
        node_name: &str,
        event_name: &str,
        parameter: f32,
    ) -> Result<(), String> {
        if let Some(system) = self.systems.get_mut(system_name) {
            // Check if this is actually a system event (when node_name is "system")
            if node_name == "system" {
                let event = crate::events::SystemEvent::from_string(event_name, parameter)?;
                system.handle_system_event(event)
            } else {
                let node = crate::events::NodeName::from_string(node_name)?;
                let event = crate::events::NodeEvent::from_string(event_name, parameter)?;
                system.handle_node_event(node, event)
            }
        } else {
            Err(format!("System '{}' not found", system_name))
        }
    }

    /// Send a system event to a specific system
    pub fn send_system_event(
        &mut self,
        system_name: &str,
        event_name: &str,
        parameter: f32,
    ) -> Result<(), String> {
        if let Some(system) = self.systems.get_mut(system_name) {
            let event = crate::events::SystemEvent::from_string(event_name, parameter)?;
            system.handle_system_event(event)
        } else {
            Err(format!("System '{}' not found", system_name))
        }
    }
}