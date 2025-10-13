#!/bin/bash
# EnvMesh Installation Script

set -e

echo "🚀 Installing EnvMesh..."
echo ""

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust not found. Install from https://rustup.rs"
    exit 1
fi

# Build binaries if needed
if [ ! -f "src-tauri/target/release/envmesh-daemon" ] || [ ! -f "src-tauri/target/release/envmesh-cli" ]; then
    echo "📦 Building EnvMesh (this may take a few minutes)..."
    cd src-tauri
    cargo build --release --bin envmesh-daemon --bin envmesh-cli
    cd ..
    echo "✓ Build complete"
else
    echo "✓ Binaries already built"
fi

# Install to ~/.local/bin
echo "📥 Installing binaries..."
mkdir -p ~/.local/bin
cp src-tauri/target/release/envmesh-daemon ~/.local/bin/
cp src-tauri/target/release/envmesh-cli ~/.local/bin/
chmod +x ~/.local/bin/envmesh-daemon ~/.local/bin/envmesh-cli
echo "✓ Binaries installed to ~/.local/bin"

# Ensure ~/.local/bin is in PATH
if ! echo "$PATH" | grep -q "$HOME/.local/bin"; then
    echo "📝 Adding ~/.local/bin to PATH..."
    echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
    echo "✓ Added to ~/.bashrc"
fi

# Add shell integration
if ! grep -q "envmesh-daemon" ~/.bashrc; then
    echo "📝 Adding shell integration..."
    cat >> ~/.bashrc << 'EOF'

# EnvMesh - P2P Environment Variable Sync
pgrep -f envmesh-daemon > /dev/null || envmesh-daemon > /dev/null 2>&1 &
eval "$(envmesh-cli export 2>/dev/null)"
EOF
    echo "✓ Shell integration added to ~/.bashrc"
else
    echo "⚠️  EnvMesh already configured in ~/.bashrc, skipping"
fi

echo ""
echo "✅ Installation complete!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Next Steps:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "1. Reload your shell:"
echo "   source ~/.bashrc"
echo ""
echo "2. Start using EnvMesh:"
echo "   envmesh-cli set MY_VAR=hello"
echo "   envmesh-cli list"
echo "   envmesh-cli get MY_VAR"
echo ""
echo "3. In new terminals, variables load automatically!"
echo ""
echo "For more info, see:"
echo "  - README.md for general usage"
echo "  - CLI_USAGE.md for CLI details"
echo "  - QUICKSTART_WSL.md for WSL setup"
echo ""
