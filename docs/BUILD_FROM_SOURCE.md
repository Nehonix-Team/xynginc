# Build from Source Guide

This guide explains how to compile XyNginC (XNCP) from source code. This is useful if you are using a non-standard architecture (like ARM64/Raspberry Pi), want to contribute to development, or simply prefer to build your own binaries.

## Prerequisites

To build XNCP, you need the **Go** (Golang) programming language.

### 1. Install Go

You can download and install Go from the official website: [https://go.dev/dl/](https://go.dev/dl/)

**Ubuntu/Debian/Kali:**

```bash
sudo apt update
sudo apt install golang-go
```

**CentOS/RHEL:**

```bash
sudo yum install golang
```

Verify the installation:

```bash
go version
```

### 2. Verify System Dependencies

XyNginC interacts with the system directly, but the Go compiler statically links most of what it needs. However, the system where you _run_ the binary must support standard Linux syscalls.

## Building XyNginC

### 1. Clone the Repository

```bash
git clone https://github.com/Nehonix-Team/xynginc.git
cd xynginc
```

### 2. Compile the Project

Navigate to the `core-go` directory where the Go code resides:

```bash
cd core-go
```

Download dependencies (if any) and build the project:

```bash
go mod tidy
go build -o xynginc
```

This process generates a statically linked binary named `xynginc` in the current directory.

### 3. Locate the Binary

Once the build completes successfully, the binary will be located at:

```bash
./xynginc
```

## Installation

To install your custom-built binary, simply move it to your system's binary path:

```bash
sudo mv ./xynginc /usr/local/bin/
```

### Verify Installation

Check that the installed version works properly:

```bash
xynginc check
```

## Cross-Compilation (Advanced)

One of Go's greatest strengths is effortless cross-compilation. If you are building on x86_64 for an ARM target (like a Raspberry Pi), you can easily specify the OS and Architecture via environment variables:

```bash
# Build for 64-bit ARM Linux
GOOS=linux GOARCH=arm64 go build -o xynginc-arm64
```

Simply transfer the `xynginc-arm64` binary to your destination machine and move it to `/usr/local/bin/xynginc`.