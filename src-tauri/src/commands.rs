use crossbeam::queue::SegQueue;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum AudioCommand {
    SendNodeEvent {
        system_name: String,
        node_name: String,
        event_name: String,
        parameter: f32,
    },
    SwitchSystem(String),
    SetSequence {
        system_name: String,
        sequence_data: serde_json::Value,
    },
}

/// Lock-free command queue for audio parameter changes
/// Uses a multiple-producer, single-consumer queue from crossbeam
pub struct AudioCommandQueue {
    queue: Arc<SegQueue<AudioCommand>>,
}

impl AudioCommandQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(SegQueue::new()),
        }
    }

    /// Get a handle for sending commands (for UI thread)
    pub fn sender(&self) -> AudioCommandSender {
        AudioCommandSender {
            queue: Arc::clone(&self.queue),
        }
    }

    /// Get a handle for receiving commands (for audio thread)
    pub fn receiver(&self) -> AudioCommandReceiver {
        AudioCommandReceiver {
            queue: Arc::clone(&self.queue),
        }
    }
}

/// Sender handle for UI thread
#[derive(Clone)]
pub struct AudioCommandSender {
    queue: Arc<SegQueue<AudioCommand>>,
}

impl AudioCommandSender {
    /// Send a command to the audio thread (non-blocking)
    pub fn send(&self, command: AudioCommand) {
        self.queue.push(command);
    }

    /// Try to send a command, returns true if successful
    pub fn try_send(&self, command: AudioCommand) -> bool {
        self.queue.push(command);
        true // SegQueue::push is always successful (unless out of memory)
    }
}

/// Receiver handle for audio thread
pub struct AudioCommandReceiver {
    queue: Arc<SegQueue<AudioCommand>>,
}

impl AudioCommandReceiver {
    /// Process all pending commands, applying them to the drum machine
    /// This should be called at the start of each audio block
    pub fn process_commands<F>(&self, mut apply_command: F)
    where
        F: FnMut(AudioCommand),
    {
        // Process up to 64 commands per audio block to avoid spending too much time
        // in command processing during the audio callback
        for _ in 0..64 {
            if let Some(command) = self.queue.pop() {
                apply_command(command);
            } else {
                break;
            }
        }
    }

    /// Check if there are pending commands
    pub fn has_commands(&self) -> bool {
        !self.queue.is_empty()
    }
}

impl Default for AudioCommandQueue {
    fn default() -> Self {
        Self::new()
    }
}
