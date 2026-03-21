#!/bin/bash

# XyNginC Multi-Arch Build Script
# This script compiles the Go binary for different Linux architectures
# to match the naming convention used by the post-install downloader.

set -e

PROJECT_ROOT=$(pwd)
CORE_DIR="$PROJECT_ROOT/core-go"
RELEASE_DIR="$PROJECT_ROOT/releases"

echo "🚀 Starting multi-arch build for XyNginC..."

# Create release directory
mkdir -p "$RELEASE_DIR"

cd "$CORE_DIR"

# Ensure dependencies are up to date
go mod tidy

# Define targets: GOOS/GOARCH/Suffix
TARGETS=(
    "linux/amd64/x64"
    "linux/arm64/arm64"
    "linux/386/ia32"
)

for target in "${TARGETS[@]}"; do
    IFS="/" read -r OS ARCH SUFFIX <<< "$target"
    
    BINARY_NAME="xynginc-linux-$SUFFIX"
    echo "📦 Building $BINARY_NAME (OS=$OS ARCH=$ARCH)..."
    
    # Compile
    GOOS=$OS GOARCH=$ARCH go build -ldflags="-s -w" -o "$RELEASE_DIR/$BINARY_NAME"
    
    echo "   ✅ Done: $BINARY_NAME"
done

echo ""
echo "🎉 All binaries built successfully in: $RELEASE_DIR"
ls -lh "$RELEASE_DIR"
