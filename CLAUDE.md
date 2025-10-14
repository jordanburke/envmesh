# EnvMesh - Client-Server Environment Variable Sync

## Overview

EnvMesh is a cross-platform client-server application for synchronizing environment variables across multiple machines. Built with Tauri 2.0, Rust, and WebSocket, it provides a secure solution for managing environment variables with automatic failover and LAN server capabilities.

## Architecture

### Technology Stack
- **Frontend**: Tauri 2.0 (native webview - 3-10MB installer)
- **Backend**: Rust
- **Networking**: WebSocket (tokio-tungstenite) for client-server communication
- **Storage**: SQLite with encryption
- **Encryption**: AES-256-GCM with Argon2 key derivation
- **IPC**: Unix domain sockets (Linux/macOS), TCP localhost (Windows)

### Unified Binary Architecture

**Key Concept**: ALL nodes run the same binary. Behavior is determined by configuration, not separate codebases.

```
┌─────────────────────────────────────────────────────────┐
│                   UNIFIED BINARY                        │
│           (envmesh - runs everywhere)                   │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Configuration determines role:                        │
│                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐ │
│  │ Cloud Server │  │  LAN Server  │  │   Client    │ │
│  │              │  │              │  │             │ │
│  │ mode:        │  │ mode:        │  │ mode:       │ │
│  │ server-      │  │ auto         │  │ client-only │ │
│  │ preferred    │  │ (failover)   │  │             │ │
│  │              │  │              │  │             │ │
│  │ Runs on VPS  │  │ Elected from │  │ Never       │ │
│  │ Always       │  │ clients when │  │ becomes     │ │
│  │ server       │  │ cloud down   │  │ server      │ │
│  └──────────────┘  └──────────────┘  └─────────────┘ │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Three-Tier Failover System

```
Client Connection Strategy:

1. Try Cloud Server (ws://cloud.envmesh.com:8765)
   ├─ Success → Use cloud
   └─ Fail → Try Step 2

2. Try LAN Discovery (mDNS/broadcast)
   ├─ Found LAN server → Connect to it
   └─ No LAN server → Try Step 3

3. Become LAN Server (if allowed by ServerMode)
   ├─ ServerMode::Auto → Become server
   ├─ ServerMode::ServerPreferred → Become server
   └─ ServerMode::ClientOnly → Give up (offline mode)
```

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
│  │  - Connection status         │  │
│  │  - Sync status               │  │
│  └──────────────────────────────┘  │
│               ↕                     │
│  ┌──────────────────────────────┐  │
│  │   Rust Backend               │  │
│  │  - WebSocket client/server   │  │
│  │  - Automatic failover        │  │
│  │  - SQLite encrypted storage  │  │
│  │  - Shell integration         │  │
│  └──────────────────────────────┘  │
└─────────────────────────────────────┘
           ↕ (WebSocket)
    ┌──────────────────────┐
    │   Cloud/LAN Server   │
    │  (same binary!)      │
    └──────────────────────┘
```

## Module Architecture

### `src-tauri/src/`

#### `main.rs`
- Tauri application entry point
- System tray setup and event handling
- Window management
- Command handler registration

#### `node.rs`
- **Unified node** combining client + server functionality
- Types: `EnvMeshNode`, `NodeConfig`, `NodeMode`, `ServerMode`
- Three-tier failover logic: Cloud → LAN → Become Server
- Automatic reconnection and health monitoring

#### `client.rs`
- WebSocket client implementation
- Types: `SyncMessage`, `WebSocketClient`
- Methods: `connect()`, `send()`, `receive()`, `ping()`
- Handles connection to cloud or LAN servers

#### `server.rs`
- Embedded WebSocket server (runs when node becomes LAN server)
- Type: `EmbeddedServer`
- Methods: `start()`, `broadcast()`, `active_connections()`
- Spawns when client is elected as LAN coordinator

#### `config.rs`
- TOML configuration parsing
- Types: `Config`, `ServerConfig`, `ClientConfig`
- Converts config to `NodeConfig` for runtime use
- Default config locations: `~/.envmesh/config.toml` or system config dir

#### `election.rs`
- Leader election logic for LAN server role
- Types: `Election`, `ServerInfo`, `PeerId`
- Methods: `discover_lan_server()`, `should_become_server()`
- Uses mDNS for discovery (placeholder - to be implemented)

#### `health.rs`
- Health monitoring and auto-failback
- Type: `HealthMonitor`
- Methods: `start_monitoring()`, `is_cloud_healthy()`, `failover_to_lan()`, `failback_to_cloud()`
- Not yet wired up (planned feature)

#### `storage.rs`
- SQLite database management
- Encrypted local storage
- CRUD operations for environment variables
- Schema: `(key, value, timestamp, machine_id, deleted)`
- Change tracking for synchronization
- Type alias: `ChangeRecord = (String, String, i64, String, bool)`

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

#### `state.rs`
- Application state management
- Type: `AppState`
- Holds: `storage`, `node`, `machine_id`

### `src-tauri/src/bin/`

#### `daemon.rs`
- Headless daemon for servers and WSL
- IPC via Unix domain sockets (Linux/macOS) or TCP localhost:37842 (Windows)
- Accepts JSON commands: Get, Set, Delete, List, Peers, Sync, Shutdown
- Platform-specific handlers: `handle_connection_unix()`, `handle_connection_tcp()`

#### `cli.rs`
- Command-line interface using clap
- Subcommands: get, set, delete, list, export, peers, sync, shutdown
- Shell integration support (bash, zsh, PowerShell, fish)
- Connects to daemon via IPC (Unix sockets or TCP)
- Platform-specific connection logic

## Configuration

### ServerMode Options

```toml
[server]
mode = "auto"  # Options: "auto", "server-preferred", "client-only"
listen = "127.0.0.1"  # or "0.0.0.0" for cloud servers
port = 8765

[client]
enable_cloud = true
cloud_url = "ws://cloud.envmesh.com:8765"
enable_lan = true
```

### Example Configurations

**Cloud Server** (`config-examples/cloud-server.toml`):
```toml
[server]
mode = "server-preferred"
listen = "0.0.0.0"
port = 8765

[client]
enable_cloud = false
enable_lan = false
```

**Auto-Failover Client** (`config-examples/client.toml`):
```toml
[server]
mode = "auto"
listen = "127.0.0.1"
port = 8765

[client]
enable_cloud = true
cloud_url = "ws://cloud.envmesh.com:8765"
enable_lan = true
```

**Client-Only** (`config-examples/client-only.toml`):
```toml
[server]
mode = "client-only"

[client]
enable_cloud = true
cloud_url = "ws://cloud.envmesh.com:8765"
enable_lan = false
```

## Features

### Core Features
- 🔐 **End-to-End Encryption**: All data encrypted with AES-256-GCM
- 🌐 **Client-Server**: WebSocket-based sync with automatic failover
- 🔄 **Three-Tier Failover**: Cloud → LAN → Become Server
- 💾 **Local-First**: Works offline, syncs when connected
- 🖥️ **Cross-Platform**: Windows, macOS, Linux support
- 🎨 **Native UI**: System tray application (lightweight)
- 🛠️ **CLI Tool**: Command-line interface for automation
- ⚙️ **Unified Binary**: Same binary for all roles (cloud, LAN, client)
- 📝 **Configuration-Driven**: Behavior determined by TOML config

### Security
- Master password required for database access
- All communication encrypted using WebSocket over TLS (when configured)
- Environment variables encrypted at rest
- IPC via Unix sockets (secure) or TCP localhost (Windows)

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

# Format code
cargo fmt

# Lint code
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test

# Build debug version
cargo build

# Build release version (optimized)
cargo build --release

# Run the GUI application
cargo run

# Run the daemon
cargo run --bin envmesh-daemon

# Run the CLI
cargo run --bin envmesh-cli -- list
```

### Project Structure

```
envmesh/
├── src-tauri/          # Rust backend
│   ├── src/
│   │   ├── main.rs     # Tauri app entry
│   │   ├── node.rs     # Unified client/server node
│   │   ├── client.rs   # WebSocket client
│   │   ├── server.rs   # Embedded WebSocket server
│   │   ├── config.rs   # TOML configuration
│   │   ├── election.rs # Leader election
│   │   ├── health.rs   # Health monitoring
│   │   ├── storage.rs  # SQLite storage
│   │   ├── crypto.rs   # Encryption
│   │   ├── api.rs      # Tauri commands
│   │   ├── state.rs    # App state
│   │   └── lib.rs      # Library exports
│   ├── src/bin/
│   │   ├── daemon.rs   # Headless daemon
│   │   └── cli.rs      # CLI interface
│   ├── icons/          # Application icons
│   ├── Cargo.toml      # Dependencies
│   └── tauri.conf.json # Tauri configuration
├── config-examples/    # Example configurations
│   ├── cloud-server.toml
│   ├── client.toml
│   ├── lan-only.toml
│   ├── client-only.toml
│   └── README.md
├── .github/workflows/  # CI/CD
│   ├── ci.yml         # Test & lint
│   ├── release.yml    # Automated releases
│   └── README.md
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
- `tokio` - Async runtime
- `tokio-tungstenite` - WebSocket client/server
- `rusqlite` - SQLite database
- `aes-gcm` - AES-256-GCM encryption
- `argon2` - Password hashing
- `automerge` - CRDT implementation (planned)
- `clap` - CLI argument parsing
- `toml` - Configuration parsing
- `serde` - Serialization
- `uuid` - Machine ID generation
- `chrono` - Timestamp handling

## Usage

### GUI Application

Start the system tray application:
```bash
./envmesh
```

The app appears in your system tray with options to:
- View all environment variables
- Add new variables
- See connection status
- Trigger manual sync

### Daemon Mode

Run headless daemon (for servers/WSL):
```bash
./envmesh-daemon --config /path/to/config.toml
```

### CLI Commands

```bash
# Set an environment variable
envmesh-cli set AWS_KEY=your-secret-key

# Get a variable
envmesh-cli get AWS_KEY

# List all variables
envmesh-cli list

# Export for shell (add to .bashrc/.zshrc)
eval "$(envmesh-cli export)"

# View connected peers
envmesh-cli peers

# Force sync
envmesh-cli sync

# Shutdown daemon
envmesh-cli shutdown
```

### Shell Integration

Add to `.bashrc` or `.zshrc`:
```bash
# Load environment variables from envmesh
eval "$(envmesh-cli export)"
```

For PowerShell:
```powershell
envmesh-cli export --shell powershell | Invoke-Expression
```

## Data Storage

### Locations

- **Linux**: `~/.local/share/envmesh/`
- **macOS**: `~/Library/Application Support/envmesh/`
- **Windows**: `%APPDATA%\envmesh\`

### Files

- `envmesh.db` - Encrypted SQLite database
- `config.toml` - Configuration file (optional)
- `daemon.sock` - Unix socket for IPC (Linux/macOS)

## CI/CD

### GitHub Actions

**CI Workflow** (`.github/workflows/ci.yml`):
- Runs on: push to main/develop, pull requests
- Platforms: Linux, macOS, Windows
- Steps: format check, clippy, tests

**Release Workflow** (`.github/workflows/release.yml`):
- Triggers on: git tags `v*.*.*`
- Builds for: Linux, macOS (x86_64 + ARM64), Windows
- Creates: GitHub Release with binaries

### Running CI Locally

```bash
cd src-tauri

# Format check
cargo fmt -- --check

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Test
cargo test --all-features
```

## Standards & Conventions

### Code Style
Follow the global standards defined in `~/.claude/CLAUDE.md`:
- Functional programming principles where applicable
- Type safety with strict Rust patterns
- Immutability by default
- Clear, descriptive naming
- Comprehensive error handling

### Rust-Specific
- Use `Result<T, E>` for error handling (no panics in production)
- Leverage type system for compile-time safety
- Prefer `Option<T>` over nullable types
- Use traits for abstraction and composability
- Follow Rust API guidelines
- Platform-specific code with `#[cfg(unix)]` / `#[cfg(windows)]`

### Testing
- Unit tests for core logic (crypto, storage, client, server)
- Integration tests for node failover
- End-to-end tests for CLI and GUI
- Test file: `#[cfg(test)] mod tests { ... }`

## Roadmap

- [x] Project structure setup
- [x] Core modules (storage, crypto, node, client, server)
- [x] WebSocket client-server implementation
- [x] Three-tier failover system
- [x] Configuration system
- [x] CLI functionality (daemon + cli binaries)
- [x] Cross-platform IPC (Unix sockets + TCP)
- [x] GitHub Actions CI/CD
- [ ] Leader election implementation (mDNS)
- [ ] Health monitoring and auto-failback
- [ ] CRDT for conflict resolution
- [ ] GUI completion
- [ ] TLS/SSL support for WebSocket
- [ ] Automated installer builds
- [ ] Mobile support (iOS/Android)

## Architecture Decisions

### Why Client-Server Instead of P2P?

1. **Simplicity**: Easier to reason about and debug than mesh networking
2. **WAN Support**: Works over internet, not just LAN (P2P with mDNS is LAN-only)
3. **Reliability**: Central coordination point reduces split-brain scenarios
4. **Failover**: Three-tier strategy ensures network stays up even if cloud fails

### Why Unified Binary?

1. **Consistency**: Same code everywhere means fewer bugs
2. **Simplicity**: One build, one release, one deployment
3. **Flexibility**: Configuration determines role, not separate codebases
4. **Cost**: Cloud server is just a well-configured peer, not special

### Why WebSocket?

1. **Cross-Platform**: Works everywhere (unlike Unix domain sockets)
2. **Firewall-Friendly**: HTTP-based, usually allowed
3. **Bidirectional**: Full-duplex communication for real-time sync
4. **Standard**: Well-understood protocol with good tooling

## Contributing

When working on this project:
1. Read this CLAUDE.md first
2. Follow global standards in `~/.claude/CLAUDE.md`
3. Run `cargo fmt` before committing
4. Run `cargo clippy` and fix all warnings
5. Run `cargo test` and ensure all pass
6. Write tests for new features
7. Update documentation as needed
8. Use platform-specific code with `#[cfg(...)]` when necessary

## License

MIT License
