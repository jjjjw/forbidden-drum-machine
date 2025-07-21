use crossbeam::queue::SegQueue;
use std::sync::Arc;

// Audio node events for individual instruments and effects
#[derive(Debug, Clone)]
pub enum NodeEvent {
    // Common events
    Trigger,
    SetGain(f32),
    
    // Instrument events
    SetBaseFrequency(f32),
    SetFrequencyModAmount(f32),
    SetAmpAttack(f32),
    SetAmpRelease(f32),
    SetFreqAttack(f32),
    SetFreqRelease(f32),
    
    // Delay events
    SetDelaySeconds(f32),
    SetFeedback(f32),
    SetFreeze(bool),
    SetHighpassFreq(f32),
    SetLowpassFreq(f32),
    
    // Reverb events
    SetSize(f32),
    SetDamping(f32),
    SetModulationDepth(f32),
    
    // System events (when node_name is "system")
    SetBpm(f32),
    SetPaused(bool),
    SetMarkovDensity(f32),
    SetKickLoopBias(f32),
    SetClapLoopBias(f32),
    GenerateKickPattern,
    GenerateClapPattern,
    SetDelaySend(f32),
    SetReverbSend(f32),
    SetDelayReturn(f32),
    SetReverbReturn(f32),
}

impl NodeEvent {
    pub fn from_string(event_name: &str, parameter: f32) -> Result<Self, String> {
        match event_name {
            "trigger" => Ok(NodeEvent::Trigger),
            "set_gain" => Ok(NodeEvent::SetGain(parameter)),
            "set_base_frequency" => Ok(NodeEvent::SetBaseFrequency(parameter)),
            "set_frequency_mod_amount" => Ok(NodeEvent::SetFrequencyModAmount(parameter)),
            "set_amp_attack" => Ok(NodeEvent::SetAmpAttack(parameter)),
            "set_amp_release" => Ok(NodeEvent::SetAmpRelease(parameter)),
            "set_freq_attack" => Ok(NodeEvent::SetFreqAttack(parameter)),
            "set_freq_release" => Ok(NodeEvent::SetFreqRelease(parameter)),
            "set_delay_seconds" => Ok(NodeEvent::SetDelaySeconds(parameter)),
            "set_feedback" => Ok(NodeEvent::SetFeedback(parameter)),
            "set_freeze" => Ok(NodeEvent::SetFreeze(parameter != 0.0)),
            "set_highpass_freq" => Ok(NodeEvent::SetHighpassFreq(parameter)),
            "set_lowpass_freq" => Ok(NodeEvent::SetLowpassFreq(parameter)),
            "set_size" => Ok(NodeEvent::SetSize(parameter)),
            "set_damping" => Ok(NodeEvent::SetDamping(parameter)),
            "set_modulation_depth" => Ok(NodeEvent::SetModulationDepth(parameter)),
            // System events
            "set_bpm" => Ok(NodeEvent::SetBpm(parameter)),
            "set_paused" => Ok(NodeEvent::SetPaused(parameter != 0.0)),
            "set_markov_density" => Ok(NodeEvent::SetMarkovDensity(parameter)),
            "set_kick_loop_bias" => Ok(NodeEvent::SetKickLoopBias(parameter)),
            "set_clap_loop_bias" => Ok(NodeEvent::SetClapLoopBias(parameter)),
            "generate_kick_pattern" => Ok(NodeEvent::GenerateKickPattern),
            "generate_clap_pattern" => Ok(NodeEvent::GenerateClapPattern),
            "set_delay_send" => Ok(NodeEvent::SetDelaySend(parameter)),
            "set_reverb_send" => Ok(NodeEvent::SetReverbSend(parameter)),
            "set_delay_return" => Ok(NodeEvent::SetDelayReturn(parameter)),
            "set_reverb_return" => Ok(NodeEvent::SetReverbReturn(parameter)),
            _ => Err(format!("Unknown node event: {}", event_name))
        }
    }
}


// Audio node names
#[derive(Debug, Clone, PartialEq)]
pub enum NodeName {
    Kick,
    Clap,
    Delay,
    Reverb,
    System,
}

impl NodeName {
    pub fn from_string(name: &str) -> Result<Self, String> {
        match name {
            "kick" => Ok(NodeName::Kick),
            "clap" => Ok(NodeName::Clap),
            "delay" => Ok(NodeName::Delay),
            "reverb" => Ok(NodeName::Reverb),
            "system" => Ok(NodeName::System),
            _ => Err(format!("Unknown node name: {}", name))
        }
    }
}

// Audio system names
#[derive(Debug, Clone, PartialEq)]
pub enum SystemName {
    DrumMachine,
    Auditioner,
}

impl SystemName {
    pub fn from_string(name: &str) -> Result<Self, String> {
        match name {
            "drum_machine" => Ok(SystemName::DrumMachine),
            "auditioner" => Ok(SystemName::Auditioner),
            _ => Err(format!("Unknown system name: {}", name))
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            SystemName::DrumMachine => "drum_machine",
            SystemName::Auditioner => "auditioner",
        }
    }
}

// Server events for audio -> UI communication
#[derive(Debug, Clone)]
pub enum ServerEvent {
    KickStepChanged(u8),
    ClapStepChanged(u8),
    ModulatorValues(f32, f32, f32), // delay_time, reverb_size, reverb_decay
    KickPatternGenerated([bool; 16]),
    ClapPatternGenerated([bool; 16]),
}

/// Lock-free event queue for audio -> UI communication
/// Uses a single-producer, single-consumer queue from crossbeam
pub struct ServerEventQueue {
    queue: Arc<SegQueue<ServerEvent>>,
}

impl ServerEventQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(SegQueue::new()),
        }
    }

    /// Get a handle for sending events (for audio thread)
    pub fn sender(&self) -> ServerEventSender {
        ServerEventSender {
            queue: Arc::clone(&self.queue),
        }
    }

    /// Get a handle for receiving events (for UI thread)
    pub fn receiver(&self) -> ServerEventReceiver {
        ServerEventReceiver {
            queue: Arc::clone(&self.queue),
        }
    }
}

/// Sender handle for audio thread
#[derive(Clone)]
pub struct ServerEventSender {
    queue: Arc<SegQueue<ServerEvent>>,
}

impl ServerEventSender {
    /// Send an event to the UI thread (non-blocking)
    pub fn send(&self, event: ServerEvent) {
        self.queue.push(event);
    }
}

/// Receiver handle for UI thread
pub struct ServerEventReceiver {
    queue: Arc<SegQueue<ServerEvent>>,
}

impl ServerEventReceiver {
    /// Process all pending events, emitting them via Tauri
    /// This should be called once per audio buffer
    pub fn process_events<F>(&self, mut emit_event: F)
    where
        F: FnMut(ServerEvent),
    {
        // Process all available events
        while let Some(event) = self.queue.pop() {
            emit_event(event);
        }
    }

    /// Check if there are pending events
    pub fn has_events(&self) -> bool {
        !self.queue.is_empty()
    }
}

impl Default for ServerEventQueue {
    fn default() -> Self {
        Self::new()
    }
}