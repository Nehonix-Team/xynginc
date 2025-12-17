#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

curl -L -o xynginc https://github.com/Nehonix-Team/xynginc/releases/latest/download/xynginc && chmod +x xynginc && sudo mv xynginc /usr/local/bin/ && xynginc --version