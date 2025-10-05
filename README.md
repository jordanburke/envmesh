# EnvMesh - Cross-Platform P2P Environment Variable Sync

A lightweight, secure, peer-to-peer mesh network for environment variable synchronization built with Rust and Tauri.

## Features

- 🔐 **End-to-End Encryption**: All data encrypted with AES-256-GCM
- 🌐 **P2P Mesh Network**: Using libp2p with gossipsub for decentralized sync
- 🔍 **Auto Peer Discovery**: mDNS for local network, DHT for internet-wide
- 💾 **Local-First**: Works offline, syncs when connected
- 🖥️ **Cross-Platform**: Windows, macOS, Linux support
- 🎨 **Native UI**: System tray application with Tauri (3-10MB size)
- 🛠️ **CLI Tool**: Command-line interface for automation
- 🔄 **CRDT Sync**: Conflict-free replicated data types for reliable merging

## Architecture

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

## Tech Stack

- **Frontend**: Tauri 2.0 (native webview, not Electron)
- **Backend**: Rust
- **P2P**: libp2p with gossipsub
- **Storage**: SQLite with encryption
- **CRDT**: Automerge for conflict resolution
- **Encryption**: AES-256-GCM with Argon2 key derivation

## Installation

### Prerequisites

- Rust 1.70+ (install from https://rustup.rs)
- Node.js (optional, for UI development)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/jordanburke/envmesh.git
cd envmesh

# Build the application
cd src-tauri
cargo build --release

# The binary will be in target/release/envmesh
```

## Usage

### GUI Application

```bash
# Start the GUI application
./envmesh daemon
```

The app will appear in your system tray. Click the icon to:
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

Add to your `.bashrc` or `.zshrc`:

```bash
# Load environment variables from envmesh
eval "$(envmesh export)"
```

For PowerShell, add to your profile:

```powershell
envmesh export --shell powershell | Invoke-Expression
```

## Configuration

The app stores data in:
- **Linux**: `~/.local/share/envmesh/`
- **macOS**: `~/Library/Application Support/envmesh/`
- **Windows**: `%APPDATA%\envmesh\`

### Files

- `envmesh.db`: Encrypted SQLite database
- `config.toml`: Configuration file
- `peer_key`: libp2p peer identity

## Security

- All environment variables are encrypted at rest using AES-256-GCM
- P2P communication uses Noise protocol for encryption
- Master password required for database access
- No central server - fully decentralized

## Development

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
│   └── Cargo.toml
└── ui/                 # Frontend
    ├── index.html
    ├── styles.css
    └── app.js
```

### Build for Release

```bash
cd src-tauri
cargo build --release --target x86_64-unknown-linux-gnu    # Linux
cargo build --release --target x86_64-pc-windows-msvc      # Windows
cargo build --release --target x86_64-apple-darwin         # macOS
```

### Run Tests

```bash
cd src-tauri
cargo test
```

## Roadmap

- [x] Project structure setup
- [x] Core modules (storage, crypto, p2p, API)
- [ ] Complete libp2p integration
- [ ] CRDT implementation
- [ ] CLI functionality
- [ ] GUI completion
- [ ] Cross-platform builds
- [ ] Documentation
- [ ] Automated tests
- [ ] Mobile support (iOS/Android)

## License

MIT License - see LICENSE file for details

## Contributing

Contributions welcome! Please open an issue or PR.

## Author

Jordan Burke
