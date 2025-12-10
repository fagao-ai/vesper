# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Vesper is a modern SSH tunnel manager built with Tauri (Rust backend) and Vue 3 + TypeScript frontend. It provides an intuitive interface for managing SSH connections and tunnels with cross-platform support.

## Architecture

### Frontend (Vue 3 + TypeScript)
- **Framework**: Vue 3 with Composition API
- **Language**: TypeScript
- **Build Tool**: Vite
- **Location**: `src/` directory
- **Key Files**:
  - `src/main.ts` - Application entry point
  - `src/App.vue` - Root Vue component

### Backend (Tauri + Rust)
- **Framework**: Tauri v2
- **Language**: Rust
- **Location**: `src-tauri/` directory
- **Key Files**:
  - `src-tauri/src/main.rs` - Main Rust entry point
  - `src-tauri/Cargo.toml` - Rust dependencies
  - `src-tauri/tauri.conf.json` - Tauri configuration

## Development Commands

### Frontend Development
```bash
# Start development server (frontend only)
npm run dev

# Build frontend for production
npm run build

# Preview built frontend
npm run preview
```

### Tauri Development
```bash
# Start Tauri development mode (runs both frontend and backend)
npm run tauri dev

# Build complete application for distribution
npm run tauri build
```

### General Development
```bash
# Install all dependencies (frontend + Tauri)
npm install
```

## Key Technologies

### Frontend Stack
- **Vue 3** - Progressive JavaScript framework
- **TypeScript** - Type-safe JavaScript
- **Vite** - Next-generation frontend build tool
- **Tauri APIs** - For native system integration

### Backend Stack
- **Tauri** - Cross-platform application framework
- **Rust** - System programming language
- **serde** - Serialization framework
- **tokio** - Async runtime (likely used for SSH operations)

## Project Structure

```
vesper/
├── src/                 # Frontend source code
│   ├── components/      # Vue components
│   ├── views/          # Page views
│   ├── stores/         # State management (Pinia)
│   ├── types/          # TypeScript definitions
│   ├── utils/          # Utility functions
│   └── assets/         # Static assets
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── commands.rs  # Tauri commands
│   │   ├── ssh.rs       # SSH connection logic
│   │   └── main.rs      # Main entry point
│   └── Cargo.toml       # Rust dependencies
├── public/             # Public assets
└── package.json        # Frontend dependencies and scripts
```

## Development Notes

- This is a Tauri application, so always use `npm run tauri dev` for full development experience
- Frontend builds can be tested independently with `npm run dev`
- TypeScript compilation is checked during builds with `vue-tsc --noEmit`
- The application is designed to manage SSH connections and tunnels
- Configuration files are stored in platform-specific locations (as detailed in README.md)