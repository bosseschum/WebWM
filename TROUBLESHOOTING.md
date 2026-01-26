# WebWM Troubleshooting Guide

Common issues and how to fix them.

## 🔨 Build Issues

### Error: "failed to run custom build command for `smithay`"

**Cause:** Missing system dependencies

**Solution:**
```bash
# Arch/Manjaro
sudo pacman -S wayland wayland-protocols libxkbcommon mesa libinput libseat

# Ubuntu/Debian
sudo apt install libwayland-dev wayland-protocols libxkbcommon-dev \
                 libgles-dev libinput-dev libudev-dev libseat-dev \
                 pkg-config

# Fedora
sudo dnf install wayland-devel wayland-protocols-devel libxkbcommon-devel \
                 mesa-libGLES-devel libinput-devel systemd-devel libseat-devel
```

### Error: "could not find `winit` in `smithay::backend`"

**Cause:** Smithay version mismatch

**Solution:** Ensure Cargo.toml has correct features:
```toml
smithay = { version = "0.3", features = ["backend_winit", "renderer_gl"] }
```

### Error: "package `rquickjs` cannot be built"

**Cause:** Missing build tools

**Solution:**
```bash
# Install build essentials
sudo apt install build-essential  # Debian/Ubuntu
sudo pacman -S base-devel         # Arch
sudo dnf install gcc gcc-c++      # Fedora
```

## 🖥️ Runtime Issues

### Compositor window doesn't appear

**Possible causes:**
1. Already running in a non-X11/Wayland environment
2. Graphics driver issues
3. Winit can't create window

**Solution:**
```bash
# Check you're in a graphical environment
echo $DISPLAY      # Should show something like :0
echo $WAYLAND_DISPLAY  # Might be empty or show wayland-0

# Try with debug logging
RUST_LOG=debug ./target/release/webwm

# Check graphics capabilities
glxinfo | grep "OpenGL"  # Should show GPU info
```

### Clients can't connect

**Error:** `No such file or directory (os error 2)`

**Cause:** Wrong socket name

**Solution:**
```bash
# WebWM prints the socket name when starting:
# "Wayland socket: wayland-1"

# Use exactly that name:
WAYLAND_DISPLAY=wayland-1 alacritty

# Don't include /run/user/ or the full path
```

### Windows appear but don't render

**Cause:** Rendering pipeline issue

**Solution:**
```bash
# Check logs for render errors
RUST_LOG=webwm::backend=debug ./target/release/webwm

# Verify compositor is rendering:
# Should see "Render frame" messages in logs
```

### Keyboard input not working

**Cause:** Seat not properly initialized or focus not set

**Solution:**
```bash
# Check if seat was created
RUST_LOG=webwm::compositor=debug ./target/release/webwm
# Should see: "Seat created: seat-0"

# Click on the window to ensure it has focus
```

## ⚙️ Configuration Issues

### "Failed to parse XML"

**Common causes:**
- Missing closing tags
- Invalid XML structure
- Special characters not escaped

**Solution:**
```bash
# Validate XML syntax
xmllint --noout config/desktop.xml

# Common fixes:
# - Ensure all tags are closed: <bar></bar>
# - Self-closing tags need /: <spacer />
# - Escape special chars: &lt; &gt; &amp;
```

### "CSS parse error"

**Common causes:**
- Missing semicolons
- Invalid color values
- Typos in property names

**Solution:**
```bash
# Test CSS separately
# Create test.html with your CSS and open in browser

# Common fixes:
# - Colors must be valid: #123456 not #12345
# - Properties need semicolons: color: red;
# - Check for typos: border-color not boder-color
```

### "JavaScript evaluation error"

**Common causes:**
- Syntax errors in config.js
- Calling undefined functions
- Using unsupported JS features

**Solution:**
```bash
# Quick syntax check
node config/config.js  # Will catch basic syntax errors

# Check for typos in WM API calls
# Correct: wm.spawn('alacritty')
# Wrong: wm.Spawn('alacritty')  # Capital S

# Supported features: ES6, but no browser APIs
# Don't use: document, window, fetch, setTimeout
```

### Configuration not being applied

**Cause:** Config file not found or in wrong location

**Solution:**
```bash
# Check file locations
ls -la config/
# Should show: desktop.xml, style.css, config.js

# Specify config dir explicitly
./target/release/webwm /full/path/to/config

# Verify config is loaded
./target/release/webwm config
# Should show your keybindings and rules
```

## 🎨 Styling Issues

### CSS colors not applying to windows

**Cause:** Selector not matching or property not implemented

**Current status:**
- ✅ Implemented: `border-color`, `border-width`
- ❌ Not yet: `border-radius`, `box-shadow`, `background`

**Workaround:**
```css
/* Use these working properties */
window {
  border-color: #89b4fa;  /* ✅ Works */
  border-width: 2px;       /* ✅ Works */
}

window:focus {
  border-color: #f38ba8;  /* ✅ Works */
}

/* These don't work yet */
window {
  border-radius: 8px;     /* ❌ Not implemented */
  box-shadow: 0 4px 12px rgba(0,0,0,0.3);  /* ❌ Not implemented */
}
```

### CSS variables not working

**Cause:** Variable reference syntax

**Solution:**
```css
/* Correct */
:root {
  --my-color: #123456;
}

window {
  border-color: var(--my-color);  /* ✅ Works */
}

/* Wrong */
window {
  border-color: --my-color;  /* ❌ Missing var() */
}
```

## 🐛 Crashes and Panics

### "thread 'main' panicked at 'Failed to...'"

**General approach:**
1. Read the full error message
2. Check the backtrace with `RUST_BACKTRACE=1`
3. Enable debug logging with `RUST_LOG=debug`

```bash
RUST_BACKTRACE=1 RUST_LOG=debug ./target/release/webwm
```

### Segmentation fault

**Possible causes:**
- Graphics driver issue
- NULL pointer in rendering
- Smithay internal issue

**Solution:**
```bash
# Run with debugger
rust-gdb ./target/release/webwm
(gdb) run
# When it crashes:
(gdb) bt  # Show backtrace

# Check graphics setup
glxinfo | grep "direct rendering"  # Should be "Yes"
```

### Compositor hangs

**Cause:** Event loop blocked or infinite loop

**Solution:**
```bash
# Kill with SIGTERM first (graceful)
killall -TERM webwm

# If that doesn't work, force kill
killall -KILL webwm

# Check for infinite loops in logs
RUST_LOG=trace ./target/release/webwm 2>&1 | grep -A 5 "hang"
```

## 🔍 Debugging Tips

### Enable All Logging

```bash
RUST_LOG=trace WAYLAND_DEBUG=1 ./target/release/webwm 2>&1 | tee webwm.log
```

This creates a log file with:
- All Rust debug messages (RUST_LOG=trace)
- All Wayland protocol messages (WAYLAND_DEBUG=1)
- Saved to webwm.log for analysis

### Test Minimal Configuration

Create `config/minimal.xml`:
```xml
<desktop>
  <workspace id="1" layout="tiling" />
</desktop>
```

Create `config/minimal.css`:
```css
window {
  border-color: #ff0000;
}
```

Create `config/minimal.js`:
```javascript
console.log('Config loaded');
```

### Verify Individual Components

```bash
# Test config parsing only
./target/release/webwm config

# Test with minimal config
./target/release/webwm config/minimal

# Test compositor without config
# (will use defaults)
rm -rf config/*
./target/release/webwm
```

## 📋 Checklist for Bug Reports

When reporting issues, include:

- [ ] WebWM version (`cargo pkgid`)
- [ ] Operating system and version
- [ ] Graphics card and driver
- [ ] Full error message
- [ ] Configuration files (desktop.xml, style.css, config.js)
- [ ] Steps to reproduce
- [ ] Output with `RUST_LOG=debug`
- [ ] Wayland protocol log if relevant (`WAYLAND_DEBUG=1`)

Example bug report template:
```markdown
**WebWM Version:** 0.2.0
**OS:** Arch Linux (kernel 6.6.1)
**GPU:** NVIDIA RTX 3060 (driver 535.129.03)
**Issue:** Windows not rendering

**Steps to reproduce:**
1. cargo build --release
2. ./target/release/webwm
3. WAYLAND_DISPLAY=wayland-1 alacritty
4. Window appears blank

**Logs:**
[paste debug logs here]

**Config:**
[attach or paste config files]
```

## 💡 Performance Issues

### Compositor using too much CPU

**Cause:** Rendering every frame even when nothing changes

**Current behavior:** WebWM renders at 60 FPS constantly

**Future improvement:** Damage tracking will reduce this

**Temporary workaround:** N/A (will be fixed in damage tracking update)

### High memory usage

**Cause:** Not properly cleaning up destroyed windows

**Check:**
```bash
# Monitor memory
watch -n 1 'ps aux | grep webwm'

# Check for memory leaks
valgrind ./target/debug/webwm
```

## 🆘 Getting Help

If you're still stuck:

1. **Check existing issues:** https://github.com/yourusername/webwm/issues
2. **Search discussions:** Look for similar problems
3. **Ask on Matrix/Discord:** [Link when available]
4. **Create an issue:** Use the bug report template above

**Before asking:**
- Read this guide completely
- Try with minimal config
- Enable debug logging
- Check for typos in config files

---

**Most issues are config-related!** Double-check your XML, CSS, and JS files carefully.
