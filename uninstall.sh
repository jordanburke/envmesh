#!/bin/bash
# EnvMesh Uninstallation Script

set -e

echo "🗑️  Uninstalling EnvMesh..."
echo ""

# Stop daemon
if pgrep -f envmesh-daemon > /dev/null; then
    echo "⏹️  Stopping daemon..."
    pkill -f envmesh-daemon 2>/dev/null || true
    echo "✓ Daemon stopped"
fi

# Remove binaries
if [ -f ~/.local/bin/envmesh-daemon ] || [ -f ~/.local/bin/envmesh-cli ]; then
    echo "🗑️  Removing binaries..."
    rm -f ~/.local/bin/envmesh-daemon
    rm -f ~/.local/bin/envmesh-cli
    echo "✓ Binaries removed"
fi

# Remove from bashrc
if grep -q "# EnvMesh - P2P Environment Variable Sync" ~/.bashrc; then
    echo "📝 Removing from ~/.bashrc..."
    # Remove the comment and the next 2 lines
    sed -i '/# EnvMesh - P2P Environment Variable Sync/,+2d' ~/.bashrc
    echo "✓ Removed from ~/.bashrc"
fi

# Ask about data removal
echo ""
read -p "Remove EnvMesh data (~/.local/share/envmesh)? [y/N] " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    rm -rf ~/.local/share/envmesh
    echo "✓ Data removed"
else
    echo "⚠️  Data kept at ~/.local/share/envmesh"
fi

echo ""
echo "✅ EnvMesh uninstalled"
echo ""
echo "To remove completely:"
echo "  rm -rf ~/.local/share/envmesh"
echo ""
