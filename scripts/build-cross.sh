#!/bin/bash

# Navigate to the Go source directory
cd core-go || exit 1

# Ensure bin directory exists
mkdir -p ../bin

echo "Building for Linux..."
env GOOS=linux GOARCH=amd64 go build -o ../bin/xynginc-linux-x64
env GOOS=linux GOARCH=arm64 go build -o ../bin/xynginc-linux-arm64

echo "Building for macOS (Darwin)..."
env GOOS=darwin GOARCH=amd64 go build -o ../bin/xynginc-darwin-x64
env GOOS=darwin GOARCH=arm64 go build -o ../bin/xynginc-darwin-arm64

echo "Building for Windows..."
env GOOS=windows GOARCH=amd64 go build -o ../bin/xynginc-win32-x64.exe
env GOOS=windows GOARCH=arm64 go build -o ../bin/xynginc-win32-arm64.exe

echo "✨ All cross-platform binaries have been built successfully in the bin/ directory."
