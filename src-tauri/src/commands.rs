use crossbeam::queue::SegQueue;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum ClientCommand {
    SendClientEvent(crate::events::ClientEvent),
    SwitchSystem(String),
}

/// Lock-free command queue for audio parameter changes
/// Uses a multiple-producer, single-consumer queue from crossbeam
pub struct ClientCommandQueue {
    queue: Arc<SegQueue<ClientCommand>>,
}

impl ClientCommandQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(SegQueue::new()),
        }
    }

    /// Get a handle for sending commands (for UI thread)
    pub fn sender(&self) -> ClientCommandSender {
        ClientCommandSender {
            queue: Arc::clone(&self.queue),
        }
    }

    /// Get a handle for receiving commands (for audio thread)
    pub fn receiver(&self) -> ClientCommandReceiver {
        ClientCommandReceiver {
            queue: Arc::clone(&self.queue),
        }
    }
}

/// Sender handle for UI thread
#[derive(Clone)]
pub struct ClientCommandSender {
    queue: Arc<SegQueue<ClientCommand>>,
}

impl ClientCommandSender {
    /// Send a command to the audio thread (non-blocking)
    pub fn send(&self, command: ClientCommand) {
        self.queue.push(command);
    }
}

/// Receiver handle for audio thread
pub struct ClientCommandReceiver {
    queue: Arc<SegQueue<ClientCommand>>,
}

impl ClientCommandReceiver {
    /// Process all pending commands, applying them to the drum machine
    /// This should be called at the start of each audio block
    pub fn process_commands<F>(&self, mut apply_command: F)
    where
        F: FnMut(ClientCommand),
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
}

impl Default for ClientCommandQueue {
    fn default() -> Self {
        Self::new()
    }
}
