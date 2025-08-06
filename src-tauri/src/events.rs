use crossbeam::queue::SegQueue;
use std::sync::Arc;

/// Client event - sent from frontend to backend
#[derive(Debug, Clone)]
pub struct ClientEvent {
    /// Target system (e.g., "drum_machine", "euclidean", "auditioner")
    pub system: String,
    /// Target node within system (e.g., "kick", "clap", "system") 
    pub node: String,
    /// Event name (e.g., "trigger", "set_gain", "set_bpm")
    pub event: String,
    /// Event parameter (for booleans: 0.0 = false, 1.0 = true)
    pub parameter: f32,
}

impl ClientEvent {
    pub fn new(system: &str, node: &str, event: &str, parameter: f32) -> Self {
        Self {
            system: system.to_string(),
            node: node.to_string(),
            event: event.to_string(),
            parameter,
        }
    }

    /// Get parameter as boolean (0.0 = false, non-zero = true)
    pub fn as_bool(&self) -> bool {
        self.parameter != 0.0
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
}

impl Default for ServerEventQueue {
    fn default() -> Self {
        Self::new()
    }
}
