# Forbidden Drum Machine - Development Guide

## Project Overview
Custom DSP and sequencing desktop application built with Tauri (Rust backend) and React (TypeScript frontend). Focus on FM synthesis and experimental audio processing techniques.

## Style Preferences
- No emojis in code or documentation

## Development Commands

### Backend (Rust)
```bash
# Format code
cd src-tauri
cargo fmt

# Run tests
cargo test

# Build
cargo build

# Development mode
cd ..
npm run tauri dev
```

### Frontend (TypeScript/React)
```bash
# Development
npm run dev

# Build
npm run build

# Type checking
npx tsc --noEmit
```

## Architecture

### Event System
The project uses a lock-free event queue system for real-time communication between frontend and backend:

1. **Frontend → Backend**: Commands sent via Tauri's `invoke()` API
   - Pushed to `AudioCommandQueue` (lock-free queue)
   - Processed by audio thread (up to 64 commands per buffer)
   - Routed to appropriate audio nodes/systems

2. **Backend → Frontend**: Events emitted from audio thread
   - Pushed to `ServerEventQueue` 
   - Polled every 16ms and emitted via Tauri events
   - Frontend listeners update React state

3. **Key Event Types**:
   - `NodeEvent`: Audio node control (trigger, parameters, modulation)
   - `ServerEvent`: UI updates (step changes, patterns, modulator values)
   - `AudioCommand`: High-level commands (system switching, sequencing)

### Audio Systems
- **DrumMachineSystem**: Main sequencer with Markov chain generation
- **AuditionerSystem**: For testing individual sounds
- Real-time audio processing using CPAL
- Lock-free architecture prevents audio dropouts

### Key Files
- `src-tauri/src/events.rs`: Event definitions and parsing
- `src-tauri/src/audio/server.rs`: Audio thread and command processing
- `src-tauri/src/commands.rs`: Tauri command handlers
- `src/App.tsx`: Frontend event listeners and UI state

## DSP Resources
See `inspiration.gen` for DSP algorithm references and implementations.

## Testing
- Backend: `cargo test` in src-tauri directory
- Frontend: Tests can be added using Vite's test setup

## Important Notes
- Audio thread never blocks on UI operations
- Events are strongly typed for safety
- Maximum 64 commands processed per audio buffer to maintain real-time performance
- Event emitter runs at ~60 FPS for smooth UI updates