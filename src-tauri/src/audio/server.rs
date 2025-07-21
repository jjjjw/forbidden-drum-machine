use crate::audio::AudioSystem;
use std::collections::HashMap;

/// Global audio server that manages multiple audio systems
pub struct AudioServer {
    /// Registered systems by name
    systems: HashMap<String, Box<dyn AudioSystem>>,

    /// Currently active system
    current_system: Option<String>,

    /// Sample rate
    sample_rate: f32,
}

impl AudioServer {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            systems: HashMap::new(),
            current_system: None,
            sample_rate,
        }
    }

    /// Add a system to the server
    pub fn add_system(&mut self, name: String, mut system: Box<dyn AudioSystem>) {
        system.set_sample_rate(self.sample_rate);
        self.systems.insert(name, system);
    }

    /// Switch to a different system immediately
    pub fn switch_to_system(&mut self, name: &str) -> Result<(), String> {
        if !self.systems.contains_key(name) {
            return Err(format!("System '{}' not found", name));
        }

        // Set new current system
        self.current_system = Some(name.to_string());
        Ok(())
    }

    /// Get the name of the current system
    pub fn get_current_system(&self) -> Option<&str> {
        self.current_system.as_deref()
    }

    /// Process audio into interleaved stereo buffer
    pub fn generate(&mut self, data: &mut [f32]) {
        // Process current system if one is selected
        if let Some(current_name) = &self.current_system {
            if let Some(current_system) = self.systems.get_mut(current_name) {
                // Let the system generate audio directly into the buffer
                current_system.generate(data);
            } else {
                // No system found, output silence
                data.fill(0.0);
            }
        } else {
            // No active system, output silence
            data.fill(0.0);
        }
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
            let node = crate::events::NodeName::from_string(node_name)?;
            let event = crate::events::NodeEvent::from_string(event_name, parameter)?;
            system.handle_node_event(node, event)
        } else {
            Err(format!("System '{}' not found", system_name))
        }
    }
}
