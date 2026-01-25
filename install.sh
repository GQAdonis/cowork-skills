#!/bin/bash
# CoWork Skills - One-line installer
# Usage: curl -sSL https://raw.githubusercontent.com/ZhangHanDong/cowork-skills/main/install.sh | bash

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() { echo -e "${BLUE}→${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
warn() { echo -e "${YELLOW}!${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; exit 1; }

echo ""
echo "╭─────────────────────────────────────╮"
echo "│     CoWork Skills Installer         │"
echo "╰─────────────────────────────────────╯"
echo ""

# Check for Rust/Cargo
if ! command -v cargo &> /dev/null; then
    warn "Cargo not found. Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    success "Rust installed"
else
    success "Cargo found: $(cargo --version)"
fi

# Install cowork from crates.io or source
info "Installing cowork CLI..."

if cargo install cowork 2>/dev/null; then
    success "Installed cowork from crates.io"
else
    warn "crates.io install failed, building from source..."

    TEMP_DIR=$(mktemp -d)
    trap "rm -rf $TEMP_DIR" EXIT

    git clone --depth 1 https://github.com/ZhangHanDong/cowork-skills.git "$TEMP_DIR"
    cd "$TEMP_DIR/cli"
    cargo install --path .

    success "Installed cowork from source"
fi

# Verify installation
if command -v cowork &> /dev/null; then
    success "cowork installed: $(cowork --version 2>/dev/null || echo 'v0.1.0')"
else
    error "Installation failed. Please check your PATH includes ~/.cargo/bin"
fi

# Initialize built-in skills
info "Initializing built-in skills..."
cowork init --force
success "Built-in skills installed to ~/.claude/skills/"

echo ""
echo "╭─────────────────────────────────────╮"
echo "│     Installation Complete!          │"
echo "╰─────────────────────────────────────╯"
echo ""
echo "Next steps:"
echo "  cowork --help          # Show all commands"
echo "  cowork list            # List installed skills"
echo "  cowork install user/repo  # Install from GitHub"
echo ""
