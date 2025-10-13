# EnvMesh Quick Start for WSL

## 5-Minute Setup

### Step 1: Build (if not already built)

```bash
cd ~/RustroverProjects/envmesh/src-tauri
cargo build --release
```

### Step 2: Start the Daemon

```bash
cd ~/RustroverProjects/envmesh/src-tauri
./target/release/envmesh-daemon &
```

You should see:
```
🚀 EnvMesh Daemon Starting...
📁 Database: /home/user/.local/share/envmesh/envmesh.db
🔌 Socket: /home/user/.local/share/envmesh/daemon.sock
✓ Storage initialized
✓ P2P node initialized

📡 Daemon running. Use 'envmesh-cli' to interact.
```

### Step 3: Use the CLI

```bash
# Set some variables
./target/release/envmesh-cli set AWS_KEY=secret123
./target/release/envmesh-cli set DB_HOST=localhost
./target/release/envmesh-cli set API_PORT=8080

# List them
./target/release/envmesh-cli list

# Get a specific one
./target/release/envmesh-cli get AWS_KEY

# Export for shell
./target/release/envmesh-cli export
```

### Step 4: Shell Integration (Optional)

Add to `~/.bashrc`:

```bash
# EnvMesh daemon auto-start and variable loading
if grep -qi microsoft /proc/version; then
    # Check if daemon is running
    if ! pgrep -f envmesh-daemon > /dev/null; then
        echo "Starting EnvMesh daemon..."
        nohup ~/RustroverProjects/envmesh/src-tauri/target/release/envmesh-daemon \
            > ~/.envmesh.log 2>&1 &
        sleep 1
    fi

    # Load variables
    if command -v ~/RustroverProjects/envmesh/src-tauri/target/release/envmesh-cli &> /dev/null; then
        eval "$(~/RustroverProjects/envmesh/src-tauri/target/release/envmesh-cli export 2>/dev/null || true)"
    fi
fi
```

Then reload:
```bash
source ~/.bashrc
```

### Step 5: Verify It Works

```bash
# Check variables are loaded
echo $AWS_KEY
echo $DB_HOST

# Add a new variable
envmesh-cli set TEST=works

# In a new terminal (or after reload)
echo $TEST  # Should output: works
```

## Common Commands

```bash
# Set a variable
envmesh-cli set KEY=value

# Get a variable
envmesh-cli get KEY

# List all variables
envmesh-cli list

# Delete a variable
envmesh-cli delete KEY

# View connected peers
envmesh-cli peers

# Shutdown daemon
envmesh-cli shutdown
```

## Stopping the Daemon

```bash
# Graceful shutdown
envmesh-cli shutdown

# Or kill the process
pkill -f envmesh-daemon
```

## Troubleshooting

### "Daemon not running" error

```bash
# Start it
./target/release/envmesh-daemon &

# Or check if it's already running
pgrep -f envmesh-daemon
```

### Variables not persisting

Make sure you added the shell integration to `~/.bashrc` and reloaded it.

### Check daemon logs

```bash
# If started with nohup
tail -f ~/.envmesh.log

# If started manually
# Check the terminal where you started it
```

## What's Next?

- **Multi-Machine Sync**: Start daemon on another machine on same network
- **Automation**: Use in scripts with `envmesh-cli`
- **See full docs**: `CLI_USAGE.md` for complete command reference

## Why CLI Mode?

✅ Works in WSL (GUI doesn't)
✅ Perfect for servers
✅ Great for automation
✅ Low memory footprint (~10-20MB)
✅ Fast startup
✅ Easy scripting

Enjoy! 🚀
