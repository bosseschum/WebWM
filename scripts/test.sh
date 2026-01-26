#!/bin/bash

# WebWM Test Script
# This script helps test the configuration parser

set -e

echo "=========================================="
echo "  WebWM Configuration Parser Test"
echo "=========================================="
echo ""

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Cargo not found. Please install Rust."
    exit 1
fi

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create test config directory if it doesn't exist
if [ ! -d "config" ]; then
    echo -e "${YELLOW}Creating config directory...${NC}"
    mkdir -p config
fi

# Check if config files exist
if [ ! -f "config/desktop.xml" ] || [ ! -f "config/style.css" ] || [ ! -f "config/config.js" ]; then
    echo -e "${YELLOW}Config files not found. Please ensure the following files exist:${NC}"
    echo "  • config/desktop.xml"
    echo "  • config/style.css"
    echo "  • config/config.js"
    echo ""
    echo "You can copy them from the artifacts in this conversation."
    exit 1
fi

echo -e "${GREEN}✓ Configuration files found${NC}"
echo ""

# Build the project
echo -e "${BLUE}Building WebWM...${NC}"
cargo build --release 2>&1 | tail -n 5

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Build successful${NC}"
else
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi

echo ""
echo "=========================================="
echo "  Running Configuration Parser"
echo "=========================================="
echo ""

# Run the parser
./target/release/webwm config

echo ""
echo "=========================================="
echo "  Test Complete!"
echo "=========================================="
echo ""
echo "Next steps:"
echo "  1. Review the parsed configuration above"
echo "  2. Modify config files and re-run to test"
echo "  3. Use --save-json to export parsed config"
echo ""
echo "Examples:"
echo "  ./target/release/webwm config --save-json"
echo "  ./target/release/webwm /path/to/other/config"
echo ""
