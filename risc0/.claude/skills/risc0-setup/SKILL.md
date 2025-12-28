---
name: RISC Zero Setup
description: This skill should be used when the user asks to "install risc zero", "setup risc0", "configure zkvm environment", "install rzup", "setup risc zero toolchain", or mentions setting up a RISC Zero development environment.
version: 0.1.0
---

# RISC Zero Environment Setup

Set up a complete RISC Zero zkVM development environment.

## Purpose

Provide step-by-step guidance for installing and configuring RISC Zero toolchain, including prerequisites verification, toolchain installation, and troubleshooting common issues.

## Prerequisites Check

Before installing RISC Zero, verify the following prerequisites:

### Rust Installation

Check Rust installation via rustup (not Homebrew):

```bash
rustc --version
cargo --version
rustup --version
```

If Rust is installed via Homebrew, uninstall and reinstall via rustup.rs:

```bash
# Uninstall Homebrew Rust if present
brew uninstall rust

# Install via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### System Requirements

- **RAM**: Minimum 16GB for local proof generation
- **OS**: Linux (x86-64) or macOS (Arm64) recommended
- **Disk**: 2GB+ free space for toolchain

## Installation Workflow

### Step 1: Install rzup

Execute the RISC Zero toolchain installer:

```bash
curl -L https://risczero.com/install | bash
```

After installation, restart the terminal or source the shell configuration:

```bash
source ~/.bashrc  # or ~/.zshrc
```

### Step 2: Install Toolchain

Install the RISC Zero toolchain:

```bash
rzup install
```

For a specific version:

```bash
rzup install cargo-risczero <version>
```

### Step 3: Verify Installation

Confirm successful installation:

```bash
cargo risczero --version
rzup --version
```

## Update Workflow

Update to the latest RISC Zero version:

```bash
rzup update
```

Then update project dependencies:

```bash
cargo update
```

## Troubleshooting

### Common Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| `rzup: command not found` | PATH not updated | Restart terminal or source shell config |
| `rustup not found` | Rust not via rustup | Reinstall Rust via rustup.rs |
| Build errors | Old toolchain | Run `rzup update` |

### CI Environment Setup

For CI environments, pin specific versions for reproducibility:

```bash
rzup install cargo-risczero 1.0.0
```

## Additional Resources

### Reference Files

- **`references/troubleshooting.md`** - Detailed troubleshooting guide

### Scripts

- **`scripts/check-prerequisites.sh`** - Automated prerequisite verification
