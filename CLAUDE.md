# EnvMesh - P2P Environment Variable Sync

## Overview

EnvMesh is a cross-platform peer-to-peer mesh network for synchronizing environment variables across multiple machines. Built with Tauri 2.0, Rust, and libp2p, it provides a secure, decentralized solution for managing environment variables without relying on cloud services.

## Architecture

### Technology Stack
- **Frontend**: Tauri 2.0 (native webview - 3-10MB installer)
- **Backend**: Rust
- **P2P Networking**: libp2p with gossipsub protocol
- **Storage**: SQLite with encryption
- **Conflict Resolution**: CRDT (Conflict-free Replicated Data Types)
- **Encryption**: AES-256-GCM with Argon2 key derivation

### System Design

```
┌─────────────────────────────────────┐
│       System Tray Native App        │
│  (Tauri - Rust backend + Web UI)    │
├─────────────────────────────────────┤
│  ┌──────────────────────────────┐  │
│  │   UI Layer (HTML/CSS/JS)     │  │
│  │  - List env vars             │  │
│  │  - Add/Edit/Delete           │  │
│  │  - Peer discovery UI         │  │
│  │  - Sync status               │  │
│  └──────────────────────────────┘  │
│               ↕                     │
│  ┌──────────────────────────────┐  │
│  │   Rust Backend               │  │
│  │  - libp2p gossipsub mesh     │  │
│  │  - CRDT for conflict-free    │  │
│  │  - SQLite encrypted storage  │  │
│  │  - Shell integration         │  │
│  └──────────────────────────────┘  │
└─────────────────────────────────────┘
           ↕ (P2P encrypted)
    [Other machines on network]
```

## Module Architecture

### `src-tauri/src/`

#### `main.rs`
- Tauri application entry point
- System tray setup and event handling
- Window management
- Command handler registration

#### `p2p.rs`
- libp2p mesh networking implementation
- Gossipsub protocol for P2P sync
- Peer discovery (mDNS for local, DHT for internet-wide)
- Noise protocol for encrypted transport
- Message broadcasting and receiving

#### `storage.rs`
- SQLite database management
- Encrypted local storage
- CRUD operations for environment variables
- Schema: `(key, value, timestamp, machine_id, deleted)`
- Change tracking for synchronization

#### `crypto.rs`
- AES-256-GCM encryption/decryption
- Argon2 password-based key derivation
- Secure random nonce generation
- Data protection at rest and in transit

#### `api.rs`
- Tauri command handlers
- Frontend ↔ Backend communication
- API endpoints:
  - `get_env_var(key)` - Retrieve variable
  - `set_env_var(key, value)` - Store variable
  - `delete_env_var(key)` - Remove variable
  - `list_env_vars()` - List all variables
  - `get_peers()` - Show connected peers
  - `trigger_sync()` - Force synchronization

#### `cli.rs`
- Command-line interface using clap
- Subcommands: get, set, delete, list, export, peers, sync, daemon
- Shell integration support (bash, zsh, PowerShell)

## Features

### Core Features
- 🔐 **End-to-End Encryption**: All data encrypted with AES-256-GCM
- 🌐 **P2P Mesh Network**: Decentralized sync using libp2p gossipsub
- 🔍 **Auto Peer Discovery**: mDNS for local network, DHT for internet-wide
- 💾 **Local-First**: Works offline, syncs when connected
- 🖥️ **Cross-Platform**: Windows, macOS, Linux support
- 🎨 **Native UI**: System tray application (lightweight)
- 🛠️ **CLI Tool**: Command-line interface for automation
- 🔄 **CRDT Sync**: Conflict-free replicated data types for reliable merging

### Security
- No central server - fully decentralized
- Master password required for database access
- Perfect forward secrecy with Noise protocol
- All communication encrypted using TLS
- Environment variables encrypted at rest

## Development

### Prerequisites
- Rust 1.70+ (https://rustup.rs)
- System dependencies for Tauri (platform-specific)

### Build Instructions

```bash
# Navigate to Rust backend
cd src-tauri

# Check compilation
cargo check

# Build debug version
cargo build

# Build release version (optimized)
cargo build --release

# Run the application
cargo run
```

### Project Structure

```
envmesh/
├── src-tauri/          # Rust backend
│   ├── src/
│   │   ├── main.rs     # Tauri app entry
│   │   ├── p2p.rs      # libp2p networking
│   │   ├── storage.rs  # SQLite storage
│   │   ├── crypto.rs   # Encryption
│   │   ├── api.rs      # Tauri commands
│   │   └── cli.rs      # CLI interface
│   ├── Cargo.toml      # Dependencies
│   └── tauri.conf.json # Tauri configuration
├── ui/                 # Frontend
│   ├── index.html
│   ├── styles.css
│   └── app.js
├── README.md          # User documentation
└── CLAUDE.md          # This file
```

### Dependencies

Key Rust crates:
- `tauri` - Native application framework
- `libp2p` - P2P networking with gossipsub, mDNS, Kad DHT
- `tokio` - Async runtime
- `rusqlite` - SQLite database
- `aes-gcm` - AES-256-GCM encryption
- `argon2` - Password hashing
- `automerge` - CRDT implementation
- `clap` - CLI argument parsing
- `serde` - Serialization

## Usage

### GUI Application

Start the system tray application:
```bash
./envmesh daemon
```

The app appears in your system tray with options to:
- View all environment variables
- Add new variables
- See connected peers
- Trigger manual sync

### CLI Commands

```bash
# Set an environment variable
envmesh set AWS_KEY=your-secret-key

# Get a variable
envmesh get AWS_KEY

# List all variables
envmesh list

# Export for shell (add to .bashrc/.zshrc)
eval "$(envmesh export)"

# View connected peers
envmesh peers

# Force sync
envmesh sync
```

### Shell Integration

Add to `.bashrc` or `.zshrc`:
```bash
# Load environment variables from envmesh
eval "$(envmesh export)"
```

For PowerShell:
```powershell
envmesh export --shell powershell | Invoke-Expression
```

## Configuration

### Data Storage

- **Linux**: `~/.local/share/envmesh/`
- **macOS**: `~/Library/Application Support/envmesh/`
- **Windows**: `%APPDATA%\envmesh\`

### Files

- `envmesh.db` - Encrypted SQLite database
- `config.toml` - Configuration file
- `peer_key` - libp2p peer identity

## Standards & Conventions

### Code Style
Follow the global standards defined in `~/.claude/CLAUDE.md`:
- Functional programming principles where applicable
- Type safety with strict TypeScript/Rust patterns
- Immutability by default
- Clear, descriptive naming
- Comprehensive error handling

### Rust-Specific
- Use `Result<T, E>` for error handling (no panics in production)
- Leverage type system for compile-time safety
- Prefer `Option<T>` over nullable types
- Use traits for abstraction and composability
- Follow Rust API guidelines

### Testing
- Unit tests for core logic (crypto, storage, sync)
- Integration tests for P2P networking
- End-to-end tests for CLI and GUI

## Roadmap

- [x] Project structure setup
- [x] Core modules (storage, crypto, p2p, API)
- [ ] Complete libp2p integration
- [ ] CRDT implementation
- [ ] CLI functionality
- [ ] GUI completion
- [ ] Cross-platform builds
- [ ] Automated tests
- [ ] Mobile support (iOS/Android)

## Contributing

When working on this project:
1. Read this CLAUDE.md first
2. Follow global standards in `~/.claude/CLAUDE.md`
3. Run `cargo check` before committing
4. Write tests for new features
5. Update documentation as needed

## License

MIT License
