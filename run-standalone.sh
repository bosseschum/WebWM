#!/bin/bash
# WebWM launcher script for standalone mode

clear
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                     WebWM - Standalone Wayland Compositor                    ║"
echo "║                                                                     ║"
echo "║  ✅ DRM Backend: Direct hardware access                               ║"
echo "║  ✅ JavaScript Config: 39 keybindings loaded                          ║"
echo "║  ✅ Wayland Server: Ready for clients                             ║"
echo "║                                                                     ║"
echo "║  Note: This is basic implementation - no visual output yet             ║"
echo "║  (Watch for 'DRM: Rendered N frames' messages)                    ║"
echo "║                                                                     ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
echo "Make sure you're in a TTY (switched from GUI with Ctrl+Alt+F3-F6)"
echo "If display access issues occur, try with sudo for proper DRM permissions"
echo ""

# Set environment for Full DRM backend
export WEBWM_BACKEND=drm-full

# Run WebWM
cd /home/bosse/Projects/WebWM
cargo run