# EnvMesh CLI Usage

## Overview

EnvMesh provides a headless daemon and CLI interface perfect for servers, WSL, and headless environments where the GUI isn't available.

## Architecture

```
┌─────────────────────────────────────┐
│         envmesh-daemon              │
│  (Background process, no GUI)       │
│  - libp2p P2P networking            │
│  - SQLite storage                   │
│  - Unix socket IPC                  │
└─────────────────────────────────────┘
                ↕ Unix Socket
┌─────────────────────────────────────┐
│         envmesh-cli                 │
│  (Command-line interface)           │
│  - get, set, delete commands        │
│  - list, peers, export              │
│  - Shell integration                │
└─────────────────────────────────────┘
```

## Quick Start

### 1. Start the Daemon

```bash
# Start in foreground
./envmesh-daemon

# Or start in background
./envmesh-daemon &

# Or use nohup for persistent background
nohup ./envmesh-daemon > ~/envmesh.log 2>&1 &
```

The daemon will:
- Create database at `~/.local/share/envmesh/envmesh.db`
- Create Unix socket at `~/.local/share/envmesh/daemon.sock`
- Start P2P networking on random port
- Listen for CLI connections

### 2. Use the CLI

```bash
# Set environment variables
envmesh-cli set AWS_KEY=your-secret-key
envmesh-cli set DB_HOST localhost
envmesh-cli set API_PORT 8080

# Get a variable
envmesh-cli get AWS_KEY

# List all variables
envmesh-cli list

# Delete a variable
envmesh-cli delete OLD_VAR

# Export for shell integration
eval "$(envmesh-cli export)"

# View connected peers
envmesh-cli peers

# Trigger manual sync
envmesh-cli sync

# Shutdown daemon
envmesh-cli shutdown
```

## Shell Integration

### Bash/Zsh

Add to `~/.bashrc` or `~/.zshrc`:

```bash
# Load EnvMesh variables
if command -v envmesh-cli &> /dev/null; then
    eval "$(envmesh-cli export 2>/dev/null || true)"
fi
```

### Fish

Add to `~/.config/fish/config.fish`:

```fish
# Load EnvMesh variables
if command -v envmesh-cli > /dev/null
    envmesh-cli export --shell fish | source
end
```

### PowerShell

Add to your PowerShell profile:

```powershell
# Load EnvMesh variables
if (Get-Command envmesh-cli -ErrorAction SilentlyContinue) {
    envmesh-cli export --shell powershell | Invoke-Expression
}
```

## Systemd Service (Linux)

Create `/etc/systemd/system/envmesh.service`:

```ini
[Unit]
Description=EnvMesh P2P Environment Variable Sync
After=network.target

[Service]
Type=simple
User=%i
ExecStart=/usr/local/bin/envmesh-daemon
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl enable envmesh
sudo systemctl start envmesh
sudo systemctl status envmesh
```

## WSL Usage

Perfect for WSL where GUI doesn't work:

```bash
# Add to ~/.bashrc
if grep -qi microsoft /proc/version; then
    # Start daemon if not running
    if ! pgrep -x envmesh-daemon > /dev/null; then
        nohup envmesh-daemon > ~/envmesh.log 2>&1 &
    fi

    # Load variables
    eval "$(envmesh-cli export 2>/dev/null || true)"
fi
```

## Command Reference

### envmesh-cli set

Set an environment variable.

```bash
# Format 1: KEY=value
envmesh-cli set AWS_KEY=secret123

# Format 2: KEY value (with space)
envmesh-cli set AWS_KEY secret123

# Values with spaces (use quotes)
envmesh-cli set "MESSAGE=Hello World"
```

### envmesh-cli get

Get an environment variable value.

```bash
envmesh-cli get AWS_KEY
# Output: secret123

# Use in scripts
DB_HOST=$(envmesh-cli get DB_HOST)
```

### envmesh-cli list

List all environment variables.

```bash
envmesh-cli list
# Output:
# AWS_KEY=secret123
# DB_HOST=localhost
# API_PORT=8080
```

### envmesh-cli delete

Delete an environment variable.

```bash
envmesh-cli delete AWS_KEY
```

### envmesh-cli export

Export variables in shell format.

```bash
# Bash/Zsh (default)
envmesh-cli export
# Output: export AWS_KEY="secret123"

# PowerShell
envmesh-cli export --shell powershell
# Output: $env:AWS_KEY="secret123"

# Fish
envmesh-cli export --shell fish
# Output: set -gx AWS_KEY "secret123"

# Use with eval
eval "$(envmesh-cli export)"
```

### envmesh-cli peers

Show connected P2P peers.

```bash
envmesh-cli peers
# Output:
# 12D3KooW... @ /ip4/192.168.1.100/tcp/52341
# 12D3KooW... @ /ip4/10.0.0.50/tcp/45123
```

### envmesh-cli sync

Trigger manual synchronization with peers.

```bash
envmesh-cli sync
```

### envmesh-cli shutdown

Gracefully shutdown the daemon.

```bash
envmesh-cli shutdown
```

## Troubleshooting

### Daemon not running

```bash
# Check if daemon is running
pgrep -x envmesh-daemon

# Check socket exists
ls -la ~/.local/share/envmesh/daemon.sock

# Start daemon
./envmesh-daemon &
```

### Permission denied on socket

```bash
# Check socket permissions
ls -la ~/.local/share/envmesh/daemon.sock

# Remove old socket and restart
rm ~/.local/share/envmesh/daemon.sock
./envmesh-daemon &
```

### Variables not syncing

```bash
# Check peers
envmesh-cli peers

# Check daemon logs (if using nohup)
tail -f ~/envmesh.log

# Trigger manual sync
envmesh-cli sync
```

### View daemon logs

```bash
# If running with nohup
tail -f ~/envmesh.log

# If running with systemd
journalctl -u envmesh -f

# Check socket activity
strace -e trace=network ./envmesh-daemon
```

## Security Notes

- Unix socket is only accessible by the user (default permissions)
- Data is stored in user's home directory
- P2P communication uses libp2p Noise protocol encryption
- Variables are NOT encrypted at rest by default (add encryption if needed)

## Performance

- Daemon memory usage: ~10-20MB
- CLI command latency: <10ms (local Unix socket)
- P2P sync: Sub-second on local network
- Storage: SQLite, minimal overhead

## Comparison: GUI vs CLI

| Feature | GUI (envmesh) | CLI (envmesh-daemon + envmesh-cli) |
|---------|---------------|-----------------------------------|
| Display required | Yes | No |
| WSL support | No (crashes) | Yes ✓ |
| Server use | No | Yes ✓ |
| System tray | Yes | No |
| Shell integration | Via GUI | Native (`envmesh export`) |
| Automation | Via GUI | Easy (CLI scripts) |
| Memory usage | ~30-50MB | ~10-20MB |
| Binary size | ~15MB | ~10MB |

## Examples

### Backup and Restore

```bash
# Backup
envmesh-cli list > ~/envmesh-backup.env

# Restore
while IFS='=' read -r key value; do
    envmesh-cli set "$key=$value"
done < ~/envmesh-backup.env
```

### Conditional Variables

```bash
# Set based on hostname
if [ "$HOSTNAME" = "prod-server" ]; then
    envmesh-cli set ENV=production
else
    envmesh-cli set ENV=development
fi
```

### Script Integration

```bash
#!/bin/bash

# Ensure daemon is running
if ! pgrep -x envmesh-daemon > /dev/null; then
    echo "Starting EnvMesh daemon..."
    envmesh-daemon &
    sleep 1
fi

# Set variables
envmesh-cli set DB_HOST=localhost
envmesh-cli set DB_PORT=5432

# Use variables
DB_HOST=$(envmesh-cli get DB_HOST)
echo "Connecting to $DB_HOST"
```

## Future Enhancements

- [ ] Watch mode: `envmesh-cli watch` to monitor changes
- [ ] Import from .env files: `envmesh-cli import .env`
- [ ] Encrypted storage option
- [ ] Remote daemon connection (TCP instead of Unix socket)
- [ ] Web UI for monitoring
