# EnvMesh Configuration Examples

This directory contains example configuration files for different deployment scenarios.

## Configuration Files

### `cloud-server.toml`
**Use case:** Cloud VPS or always-on server

- Server mode: `server-preferred` (always tries to be a server)
- Listens on: `0.0.0.0` (all interfaces, publicly accessible)
- Cloud connection: Disabled
- LAN discovery: Disabled

**Deployment:**
```bash
# Copy to your cloud server
scp cloud-server.toml user@your-vps:~/.envmesh/config.toml

# Or run with --config flag
./envmesh-daemon --config cloud-server.toml
```

---

### `client.toml`
**Use case:** Local machines (laptops, desktops)

- Server mode: `auto` (can become LAN server if needed)
- Listens on: `127.0.0.1` (local only)
- Cloud connection: Enabled
- LAN discovery: Enabled

**Priority:**
1. Try cloud server first
2. Fall back to LAN server if found
3. Become LAN server if no others available

**Deployment:**
```bash
# Copy to your home directory
cp client.toml ~/.envmesh/config.toml

# Update cloud_url with your server address
sed -i 's|ws://cloud.envmesh.com:8765|ws://YOUR_SERVER:8765|' ~/.envmesh/config.toml
```

---

### `lan-only.toml`
**Use case:** Offline/air-gapped networks

- Server mode: `auto`
- Listens on: `0.0.0.0` (all local interfaces)
- Cloud connection: Disabled
- LAN discovery: Enabled

**Perfect for:**
- Home networks without internet
- Corporate intranets
- Development environments

---

### `client-only.toml`
**Use case:** Machines that should NEVER become servers

- Server mode: `client-only`
- Cloud connection: Enabled
- LAN discovery: Enabled (connects but never becomes server)

**Perfect for:**
- Low-power devices
- Machines with unstable connectivity
- When you want dedicated servers only

---

## Installation

### Quick Setup

```bash
# 1. Create config directory
mkdir -p ~/.envmesh

# 2. Copy desired config
cp config-examples/client.toml ~/.envmesh/config.toml

# 3. Edit as needed
nano ~/.envmesh/config.toml

# 4. Start daemon
./envmesh-daemon
```

### Custom Config Location

```bash
# Use --config flag to specify custom location
./envmesh-daemon --config /path/to/my-config.toml
```

---

## Configuration Format

### Full Example

```toml
[server]
# Mode: "auto", "server-preferred", or "client-only"
mode = "auto"

# Listen address: "127.0.0.1" (local) or "0.0.0.0" (all interfaces)
listen = "127.0.0.1"

# Port to listen on
port = 8765

[client]
# Cloud server URL
cloud_url = "ws://cloud.envmesh.com:8765"

# Enable/disable cloud connection
enable_cloud = true

# Enable/disable LAN discovery
enable_lan = true
```

---

## Deployment Scenarios

### Scenario 1: Home + Cloud Setup

**Cloud Server:**
```toml
[server]
mode = "server-preferred"
listen = "0.0.0.0"
port = 8765

[client]
enable_cloud = false
enable_lan = false
```

**Home Machines:**
```toml
[server]
mode = "auto"
listen = "127.0.0.1"
port = 8765

[client]
cloud_url = "ws://YOUR_VPS:8765"
enable_cloud = true
enable_lan = true
```

**Behavior:**
- Home machines connect to cloud
- If internet down: Home machines elect LAN server
- When internet returns: Reconnect to cloud

---

### Scenario 2: Pure LAN (No Cloud)

**All Machines:**
```toml
[server]
mode = "auto"
listen = "0.0.0.0"
port = 8765

[client]
enable_cloud = false
enable_lan = true
```

**Behavior:**
- Machines discover each other via mDNS
- One machine elected as LAN server
- Others connect to elected server

---

### Scenario 3: Dedicated LAN Server

**Server Machine:**
```toml
[server]
mode = "server-preferred"
listen = "0.0.0.0"
port = 8765

[client]
enable_cloud = false
enable_lan = false
```

**Client Machines:**
```toml
[server]
mode = "client-only"
listen = "127.0.0.1"
port = 8765

[client]
cloud_url = "ws://192.168.1.100:8765"  # Your LAN server
enable_cloud = true                    # Treat LAN server as "cloud"
enable_lan = false
```

---

## Troubleshooting

### Config Not Found

The daemon looks for config in these locations (in order):
1. Path specified with `--config` flag
2. `~/.envmesh/config.toml`
3. `~/.config/envmesh/config.toml`
4. Built-in defaults

### Check Current Config

```bash
# Daemon will print config on startup
./envmesh-daemon

# Look for output:
# ⚙️  Configuration:
#    Server mode: Auto
#    Listen address: 127.0.0.1:8765
#    ...
```

### Test Configuration

```bash
# Create a test config
cat > test-config.toml <<EOF
[server]
mode = "auto"
listen = "127.0.0.1"
port = 8765

[client]
cloud_url = "ws://localhost:8080"
enable_cloud = true
enable_lan = true
EOF

# Run with test config
./envmesh-daemon --config test-config.toml
```
