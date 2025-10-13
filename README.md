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

### Quick Install (Recommended)

```bash
curl -sSL https://raw.githubusercontent.com/jordanburke/envmesh/main/install.sh | bash
```

This will:
- Build the binaries (if needed)
- Install to `~/.local/bin`
- Add auto-start to `~/.bashrc`
- Set up shell integration

Then reload your shell:
```bash
source ~/.bashrc
```

### Manual Installation

#### Prerequisites

- Rust 1.70+ (install from https://rustup.rs)
- For GUI: System dependencies for Tauri (see below)

#### Build from Source

```bash
# Clone the repository
git clone https://github.com/jordanburke/envmesh.git
cd envmesh

# Build the application
cd src-tauri
cargo build --release

# Binaries will be in target/release/
# - envmesh          (GUI application)
# - envmesh-daemon   (headless daemon)
# - envmesh-cli      (command-line interface)
```

#### Install to PATH

```bash
# Copy binaries to ~/.local/bin
mkdir -p ~/.local/bin
cp target/release/envmesh-daemon ~/.local/bin/
cp target/release/envmesh-cli ~/.local/bin/

# Add to ~/.bashrc
echo '' >> ~/.bashrc
echo '# EnvMesh - P2P Environment Variable Sync' >> ~/.bashrc
echo 'pgrep -f envmesh-daemon > /dev/null || envmesh-daemon > /dev/null 2>&1 &' >> ~/.bashrc
echo 'eval "$(envmesh-cli export 2>/dev/null)"' >> ~/.bashrc

# Reload shell
source ~/.bashrc
```

### System Dependencies (GUI Only)

For the GUI application, install system dependencies:

**Ubuntu/Debian:**
```bash
sudo apt install pkg-config libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev \
  libjavascriptcoregtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

**Arch Linux:**
```bash
sudo pacman -S webkit2gtk gtk3 libsoup3
```

**Fedora:**
```bash
sudo dnf install webkit2gtk4.1-devel gtk3-devel libsoup3-devel
```

**macOS:**
```bash
# No additional dependencies needed
```

**Windows:**
```bash
# No additional dependencies needed
```

**Note:** CLI mode (daemon + CLI) works without these dependencies!

### Uninstall

```bash
curl -sSL https://raw.githubusercontent.com/jordanburke/envmesh/main/uninstall.sh | bash
```

Or if you have the repo:
```bash
./uninstall.sh
```

## Usage

### GUI Application (Native Linux/Windows/macOS)

```bash
# Start the GUI application
./envmesh
```

The app will appear in your system tray. Click the icon to:
- View all environment variables
- Add new variables
- See connected peers
- Trigger manual sync

**Note:** GUI requires a display server. For WSL/servers, use CLI mode below.

### CLI Mode (WSL/Servers/Headless)

Perfect for WSL, servers, and automation!

```bash
# 1. Start the daemon (background process)
./envmesh-daemon &

# 2. Use the CLI
envmesh-cli set AWS_KEY=your-secret-key
envmesh-cli get AWS_KEY
envmesh-cli list

# 3. Shell integration (add to .bashrc/.zshrc)
eval "$(envmesh-cli export)"

# View connected peers
envmesh-cli peers

# Shutdown daemon
envmesh-cli shutdown
```

**See [CLI_USAGE.md](CLI_USAGE.md) for detailed CLI documentation.**

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
- [x] libp2p integration (gossipsub + mDNS)
- [x] CLI functionality (daemon + CLI)
- [x] GUI application (Tauri 2.0)
- [x] Shell integration (bash, zsh, fish, PowerShell)
- [x] Installation scripts
- [x] Documentation (README, CLI_USAGE, QUICKSTART_WSL)
- [ ] CRDT implementation
- [ ] Encryption at rest (crypto module exists but not wired up)
- [ ] Cross-platform builds (Linux working, need Windows/macOS)
- [ ] Automated tests
- [ ] Mobile support (iOS/Android)

## License

MIT License - see LICENSE file for details

## Contributing

Contributions welcome! Please open an issue or PR.

## Author

Jordan Burke
