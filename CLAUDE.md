# Forbidden Drum Machine - Development Guide

## Project Overview
Custom DSP and sequencing desktop application built with Tauri (Rust backend) and React (TypeScript frontend). Focus on FM synthesis and experimental audio processing techniques.

## Active Systems
The application currently has two main systems:
- **Auditioner**: Individual instrument testing and parameter tweaking
- **TranceRiff**: Chord-based sequencing with supersaw synthesis

## Style Preferences
- No emojis in code or documentation
- No semicolons in TypeScript/JavaScript code (Prettier configured with `semi: false`)

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
npm run typecheck

# Linting
npm run lint
npm run lint:fix

# Formatting
npm run format
npm run format:check

# Testing
npm run test
npm run test:run
npm run test:coverage
```

## Architecture

### Event System
The project uses a lock-free event queue system for real-time communication between frontend and backend:

1. **Frontend → Backend**: Commands sent via Tauri's `invoke()` API
   - Pushed to `ClientCommandQueue` (lock-free queue)
   - Processed by audio thread (up to 64 commands per buffer)
   - Routed to appropriate audio nodes/systems

2. **Backend → Frontend**: Events emitted from audio thread
   - Pushed to `ServerEventQueue`
   - Polled every 16ms and emitted via Tauri events
   - Frontend listeners update React state

3. **Key Event Types**:
   - `ClientEvent`: Audio node control (trigger, parameters, modulation)
   - `ServerEvent`: UI updates (step changes, patterns, modulator values)
   - `ClientCommand`: High-level commands (system switching, sequencing)

### Audio Systems
- **AuditionerSystem**: For testing individual sounds and tweaking parameters
- **TranceRiffSystem**: Chord-based sequencing with supersaw synthesis
- Real-time audio processing using CPAL
- Lock-free architecture prevents audio dropouts

### Key Files
- `src-tauri/src/events.rs`: Event definitions and parsing
- `src-tauri/src/audio/server.rs`: Audio thread and command processing
- `src-tauri/src/audio/systems/`: Audio system implementations (auditioner, trance_riff)
- `src-tauri/src/commands.rs`: Tauri command handlers
- `src/events.ts`: Frontend event type definitions (organized by System → Node → Events)
- `src/App.tsx`: Frontend event listeners and UI state

## DSP Resources
See `inspiration.gen` for DSP algorithm references and implementations.

## Testing
- Backend: `cargo test` in src-tauri directory
- Frontend: `npm run test` (Vitest with @testing-library/react)

## Event Invocation Patterns

### Frontend → Backend Audio Events
The frontend uses strongly-typed events organized by System → Node → Events. Always use the imported event constants:

```typescript
import { TranceRiff, Auditioner, SystemNames, NodeNames } from "../events"

// System switching
await invoke("switch_audio_system", {
  systemName: SystemNames.TranceRiff, // or SystemNames.Auditioner
})

// Audio events
await invoke("send_client_event", {
  systemName: SystemNames.TranceRiff,
  nodeName: NodeNames.System,
  eventName: TranceRiff.System.SetBpm,
  parameter: 120
})
```

**Current Event Structure:**
- `SystemNames.Auditioner` / `SystemNames.TranceRiff`
- `NodeNames.System` / `NodeNames.Kick` / `NodeNames.Supersaw` / etc.
- `TranceRiff.System.SetBpm` / `Auditioner.Kick.SetGain` / etc.

**Examples:**
- Set BPM: `TranceRiff.System.SetBpm`
- Pause/Resume: `TranceRiff.System.SetPaused` (1.0 = paused, 0.0 = playing)
- Trigger kick: `Auditioner.Kick.Trigger`
- Set supersaw gain: `TranceRiff.Supersaw.SetGain`

**Important:** Always use the typed event constants instead of hardcoded strings for better type safety and maintainability.

## Development Tooling

### Frontend Code Quality
The project includes comprehensive tooling for maintaining code quality:

- **Prettier**: Code formatting with `semi: false` configuration
  - Config: `.prettierrc`
  - Run: `npm run format` (fix) or `npm run format:check` (verify)

- **ESLint**: Code linting with TypeScript and React rules
  - Config: `eslint.config.js` (modern flat config)
  - Run: `npm run lint` (check) or `npm run lint:fix` (fix)

- **TypeScript**: Strict type checking
  - Run: `npm run typecheck`

- **Vitest**: Testing framework with React Testing Library
  - Setup: `src/test/setup.ts`
  - Run: `npm run test` (watch) or `npm run test:run` (single run)

### Pre-Development Checklist
Before working on the frontend, always run:
```bash
npm run typecheck && npm run lint && npm run format:check
```

## Important Notes
- Audio thread never blocks on UI operations
- Events are strongly typed for safety
- Maximum 64 commands processed per audio buffer to maintain real-time performance
- Event emitter runs at ~60 FPS for smooth UI updates
