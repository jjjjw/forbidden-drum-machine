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
    /// Optional event parameter (for booleans: 0.0 = false, 1.0 = true)
    pub parameter: Option<f32>,
    /// Optional data payload for complex events (serialized JSON)
    pub data: Option<serde_json::Value>,
}

impl ClientEvent {
    /// Create a simple event with just a parameter
    pub fn new(system: &str, node: &str, event: &str, parameter: f32) -> Self {
        Self {
            system: system.to_string(),
            node: node.to_string(),
            event: event.to_string(),
            parameter: Some(parameter),
            data: None,
        }
    }

    /// Create an event with data payload
    pub fn with_data(system: &str, node: &str, event: &str, data: serde_json::Value) -> Self {
        Self {
            system: system.to_string(),
            node: node.to_string(),
            event: event.to_string(),
            parameter: None,
            data: Some(data),
        }
    }

    /// Create an event with both parameter and data
    pub fn with_param_and_data(system: &str, node: &str, event: &str, parameter: f32, data: serde_json::Value) -> Self {
        Self {
            system: system.to_string(),
            node: node.to_string(),
            event: event.to_string(),
            parameter: Some(parameter),
            data: Some(data),
        }
    }

    /// Create a trigger event (no parameter needed)
    pub fn trigger(system: &str, node: &str) -> Self {
        Self {
            system: system.to_string(),
            node: node.to_string(),
            event: "trigger".to_string(),
            parameter: None,
            data: None,
        }
    }

    /// Get parameter as boolean (0.0 = false, non-zero = true)
    pub fn as_bool(&self) -> bool {
        self.parameter.map(|p| p != 0.0).unwrap_or(false)
    }

    /// Get parameter value, defaulting to 0.0 if None
    pub fn param(&self) -> f32 {
        self.parameter.unwrap_or(0.0)
    }
}

/// Server event - sent from backend to frontend
/// Mirrors ClientEvent structure for symmetry
#[derive(Debug, Clone)]
pub struct ServerEvent {
    /// Source system (e.g., "drum_machine", "euclidean", "auditioner")
    pub system: String,
    /// Source node within system (e.g., "kick", "clap", "system")
    pub node: String,
    /// Event name (e.g., "step_changed", "pattern_generated", "modulator_values")
    pub event: String,
    /// Optional parameter value
    pub parameter: Option<f32>,
    /// Optional data payload for complex events (serialized JSON)
    pub data: Option<serde_json::Value>,
}

impl ServerEvent {
    /// Create a simple event with just a parameter
    pub fn new(system: &str, node: &str, event: &str, parameter: f32) -> Self {
        Self {
            system: system.to_string(),
            node: node.to_string(),
            event: event.to_string(),
            parameter: Some(parameter),
            data: None,
        }
    }

    /// Create an event with data payload
    pub fn with_data(system: &str, node: &str, event: &str, data: serde_json::Value) -> Self {
        Self {
            system: system.to_string(),
            node: node.to_string(),
            event: event.to_string(),
            parameter: None,
            data: Some(data),
        }
    }

    /// Create an event with both parameter and data
    pub fn with_param_and_data(system: &str, node: &str, event: &str, parameter: f32, data: serde_json::Value) -> Self {
        Self {
            system: system.to_string(),
            node: node.to_string(),
            event: event.to_string(),
            parameter: Some(parameter),
            data: Some(data),
        }
    }

    /// Get parameter value, defaulting to 0.0 if None
    pub fn param(&self) -> f32 {
        self.parameter.unwrap_or(0.0)
    }
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
