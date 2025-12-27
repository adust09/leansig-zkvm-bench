#!/bin/bash
# RISC Zero Prerequisites Check Script
# Verifies all requirements for RISC Zero development

set -e

echo "=== RISC Zero Prerequisites Check ==="
echo

# Check Rust
echo "Checking Rust installation..."
if command -v rustc &> /dev/null; then
    echo "✓ Rust: $(rustc --version)"
else
    echo "✗ Rust not found. Install via: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check Cargo
echo "Checking Cargo..."
if command -v cargo &> /dev/null; then
    echo "✓ Cargo: $(cargo --version)"
else
    echo "✗ Cargo not found"
    exit 1
fi

# Check rustup
echo "Checking rustup..."
if command -v rustup &> /dev/null; then
    echo "✓ rustup: $(rustup --version 2>/dev/null | head -1)"
else
    echo "✗ rustup not found. RISC Zero requires Rust installed via rustup.rs"
    exit 1
fi

# Check rzup
echo "Checking rzup..."
if command -v rzup &> /dev/null; then
    echo "✓ rzup: $(rzup --version 2>/dev/null || echo 'installed')"
else
    echo "○ rzup not found. Install via: curl -L https://risczero.com/install | bash"
fi

# Check cargo-risczero
echo "Checking cargo-risczero..."
if cargo risczero --version &> /dev/null; then
    echo "✓ cargo-risczero: $(cargo risczero --version)"
else
    echo "○ cargo-risczero not found. Install via: rzup install"
fi

# Check RAM
echo
echo "Checking system resources..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    RAM_GB=$(($(sysctl -n hw.memsize) / 1024 / 1024 / 1024))
else
    RAM_GB=$(($(grep MemTotal /proc/meminfo | awk '{print $2}') / 1024 / 1024))
fi

if [ "$RAM_GB" -ge 16 ]; then
    echo "✓ RAM: ${RAM_GB}GB (recommended: 16GB+)"
else
    echo "⚠ RAM: ${RAM_GB}GB (recommended: 16GB+ for local proving)"
fi

echo
echo "=== Prerequisites check complete ==="
