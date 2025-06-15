# Forbidden Drum Machine

## Installation

1. **Install frontend dependencies**:
   ```bash
   npm install
   ```

2. **Install Rust dependencies** (handled automatically during build):
   ```bash
   cd src-tauri
   cargo build
   cd ..
   ```

## Development

### Running in Development Mode

Start the development server with hot-reloading:

```bash
npm run tauri dev
```

This will:
- Start the Vite development server for the React frontend
- Compile the Rust backend
- Launch the Tauri application with hot-reloading enabled

### Building for Production

Create optimized production builds:

```bash
npm run tauri build
```

The built application will be available in `src-tauri/target/release/bundle/`.

## Testing

### Running All Tests

```bash
# Test Rust backend
cd src-tauri
cargo test

# Test frontend (none yet, but can be added)
cd ..
npm test
```
