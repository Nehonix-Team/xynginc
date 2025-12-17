# Build from Source Guide

This guide explains how to compile XyNginC (XNCP) from source code. This is useful if you are using a non-standard architecture (like ARM64/Raspberry Pi), want to contribute to development, or simply prefer to build your own binaries.

## Prerequisites

To build XNCP, you need the **Rust** programming language and its package manager, **Cargo**.

### 1. Install Rust and Cargo

The recommended way to install Rust is via `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the on-screen instructions (default installation is usually fine). After installation, restart your shell or run:

```bash
source "$HOME/.cargo/env"
```

Verify the installation:

```bash
rustc --version
cargo --version
```

### 2. Install System Dependencies

XNCP relies on OpenSSL. You need to install the development packages for your system.

**Ubuntu/Debian/Kali:**

```bash
sudo apt update
sudo apt install build-essential libssl-dev pkg-config
```

**CentOS/RHEL:**

```bash
sudo yum groupinstall "Development Tools"
sudo yum install openssl-devel
```

## Building XyNginC

### 1. Clone the Repository

```bash
git clone https://github.com/Nehonix-Team/xynginc.git
cd xynginc
```

### 2. Compile the Project

Navigate to the core directory where the Rust code resides:

```bash
cd core
```

Build the project in release mode (optimized for performance):

```bash
cargo build --release
```

This process may take a few minutes as it downloads dependencies and compiles the code.

### 3. Locate the Binary

Once the build completes successfully, the binary will be located at:

```bash
./target/release/xynginc
```

## Installation

To install your custom-built binary, simply move it to your system's binary path:

```bash
sudo mv ./target/release/xynginc /usr/local/bin/
```

### Verify Installation

Check that the installed version matches your build:

```bash
xynginc --version
```

## Troubleshooting Build Issues

### "linker 'cc' not found"

This means you are missing the C compiler. Ensure you installed `build-essential` (Ubuntu) or "Development Tools" (CentOS).

### "openssl-sys" build failed

This usually means `libssl-dev` or `pkg-config` is missing. Re-run the system dependencies installation step.

### Cross-Compilation (Advanced)

If you are building on x86_64 for an ARM target (or vice-versa), you will need to install the appropriate cross-compilation target via rustup (e.g., `rustup target add aarch64-unknown-linux-gnu`) and use a cross-linker. This is outside the scope of this basic guide.
