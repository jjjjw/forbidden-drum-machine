use crossbeam::queue::SegQueue;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum AudioEvent {
    KickStepChanged(u8),
    ClapStepChanged(u8),
    ModulatorValues(f32, f32, f32), // delay_time, reverb_size, reverb_decay
    KickPatternGenerated([bool; 16]),
    ClapPatternGenerated([bool; 16]),
}

/// Lock-free event queue for audio -> UI communication
/// Uses a single-producer, single-consumer queue from crossbeam
pub struct AudioEventQueue {
    queue: Arc<SegQueue<AudioEvent>>,
}

impl AudioEventQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(SegQueue::new()),
        }
    }

    /// Get a handle for sending events (for audio thread)
    pub fn sender(&self) -> AudioEventSender {
        AudioEventSender {
            queue: Arc::clone(&self.queue),
        }
    }

    /// Get a handle for receiving events (for UI thread)
    pub fn receiver(&self) -> AudioEventReceiver {
        AudioEventReceiver {
            queue: Arc::clone(&self.queue),
        }
    }
}

/// Sender handle for audio thread
#[derive(Clone)]
pub struct AudioEventSender {
    queue: Arc<SegQueue<AudioEvent>>,
}

impl AudioEventSender {
    /// Send an event to the UI thread (non-blocking)
    pub fn send(&self, event: AudioEvent) {
        self.queue.push(event);
    }
}

/// Receiver handle for UI thread
pub struct AudioEventReceiver {
    queue: Arc<SegQueue<AudioEvent>>,
}

impl AudioEventReceiver {
    /// Process all pending events, emitting them via Tauri
    /// This should be called once per audio buffer
    pub fn process_events<F>(&self, mut emit_event: F)
    where
        F: FnMut(AudioEvent),
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

impl Default for AudioEventQueue {
    fn default() -> Self {
        Self::new()
    }
}