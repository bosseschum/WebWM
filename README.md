# WebWM - Web-Tech Wayland Compositor Prototype

A Wayland compositor/window manager configured using web technologies: XML, CSS, and JavaScript.

## 🎯 Concept

WebWM allows you to design your desktop environment like you would a website:

- **XML** - Define structure (bars, panels, workspaces, window rules)
- **CSS** - Style everything (colors, borders, animations, layouts)
- **JavaScript** - Add logic (keybindings, window rules, custom behaviors)

## 🏗️ Architecture

```
┌─────────────────────────────────────┐
│   User Configuration Layer          │
│   (XML + CSS + JS)                  │
├─────────────────────────────────────┤
│   Config Parser & JS Runtime        │
│   (Converts web tech → Config)      │
├─────────────────────────────────────┤
│   Compositor Core (Rust + Smithay)  │
│   - Window management               │
│   - Wayland protocol                │
│   - Input handling                  │
├─────────────────────────────────────┤
│   Rendering Pipeline                │
│   - Native window compositing       │
│   - CSS-styled decorations          │
│   - XML-defined UI elements         │
└─────────────────────────────────────┘
```

## 📁 Project Structure

```
webwm/
├── src/
│   ├── main.rs          # Entry point
│   ├── config.rs        # Config parser
│   ├── state.rs         # Compositor state
│   ├── layout/          # Layout algorithms
│   ├── render/          # Rendering engine
│   └── input/           # Input handling
├── config/
│   ├── desktop.xml      # Structure definition
│   ├── style.css        # Visual styling
│   └── config.js        # Keybindings & logic
└── Cargo.toml
```

## 🚀 Current Status (v0.2.0 - Working Compositor!)

**Implemented:**
- ✅ Full Wayland compositor using Smithay
- ✅ XML parser for desktop.xml (structure definition)
- ✅ CSS parser for style.css (theming and styling)
- ✅ JavaScript runtime integration (rquickjs)
- ✅ Unified configuration system
- ✅ Window creation and destruction
- ✅ Tiling and floating layout algorithms
- ✅ CSS border colors applied to windows
- ✅ Window rules from config
- ✅ Rendering pipeline at 60 FPS
- ✅ Input event handling
- ✅ Winit backend for testing

**Currently Working:**
WebWM is now a functional Wayland compositor! You can:
- Run it and connect Wayland clients
- See windows automatically arranged in tiling or floating mode
- Window borders use colors from your CSS
- Gaps and layout configured from XML/CSS

**TODO:**
- ⏳ Keybinding execution (match input to config keybindings)
- ⏳ Workspace management
- ⏳ Window decorations (title bars, buttons)
- ⏳ Bar/panel rendering from XML
- ⏳ JavaScript callback execution for window events
- ⏳ More CSS properties (border-radius, shadows, etc.)
- ⏳ Animation system using CSS transitions
- ⏳ Multi-monitor support
- ⏳ IPC for live config reloading
- ⏳ DRM/KMS backend for running on TTY

## 🛠️ Building

### Prerequisites

```bash
# Arch/Manjaro
sudo pacman -S wayland wayland-protocols libxkbcommon mesa libinput libseat

# Ubuntu/Debian
sudo apt install libwayland-dev wayland-protocols libxkbcommon-dev \
                 libgles-dev libinput-dev libudev-dev libseat-dev

# Fedora
sudo dnf install wayland-devel wayland-protocols-devel libxkbcommon-devel \
                 mesa-libGLES-devel libinput-devel systemd-devel libseat-devel
```

### Compile & Run

```bash
# Clone the repo
git clone https://github.com/yourusername/webwm
cd webwm

# Build release version
cargo build --release

# Validate your configuration
./target/release/webwm config

# Run the compositor!
./target/release/webwm

# In another terminal, connect a client
WAYLAND_DISPLAY=wayland-1 alacritty
WAYLAND_DISPLAY=wayland-1 firefox
```

### What You'll See

When you run WebWM:
1. A window opens (using winit backend for testing)
2. The compositor starts listening for Wayland clients
3. Connect apps using the printed `WAYLAND_DISPLAY` value
4. Windows automatically tile according to your layout config
5. Border colors from your CSS are applied!

### Testing Your Configuration

```bash
# Just validate config without running compositor
./target/release/webwm config

# Save parsed config as JSON for inspection
./target/release/webwm config --save-json

# Run with debug logging
RUST_LOG=debug ./target/release/webwm

# Test with Wayland protocol debugging
WAYLAND_DEBUG=1 ./target/release/webwm
```

## 📝 Example Configuration

### desktop.xml
```xml
<desktop>
  <bar position="top" class="main-bar">
    <workspaces />
    <window-title />
    <clock format="%H:%M" />
  </bar>
  
  <workspace id="1" layout="tiling" />
</desktop>
```

### style.css
```css
window:focus {
  border: 2px solid #89b4fa;
  box-shadow: 0 4px 20px rgba(137, 180, 250, 0.4);
}
```

### config.js
```javascript
keybind('Super+Return', () => wm.spawn('alacritty'));

onWindowCreate((window) => {
  if (window.appId === 'firefox') {
    window.moveToWorkspace(2);
  }
});
```

## 🎨 Features (Planned)

- **Live Editing** - Change CSS, see updates in real-time
- **Animations** - CSS transitions for window movements
- **Flexbox/Grid** - Modern layout engines for tiling
- **Themes** - Swap CSS files for instant themes
- **Plugins** - NPM packages as WM extensions
- **Inspector** - DevTools-like debugging

## 🤝 Contributing

This is an early prototype! Contributions welcome:

1. **Core Compositor** - Improve Smithay integration
2. **Parsers** - XML/CSS parsing and application
3. **JS Runtime** - Embed and expose WM API
4. **Rendering** - CSS styling for windows
5. **Documentation** - Usage guides and examples

## 📚 Resources

- [Smithay Documentation](https://smithay.github.io/smithay/)
- [Wayland Protocol](https://wayland.freedesktop.org/docs/html/)
- [Similar Projects] - Hyprland, awesome, i3

## 📄 License

MIT

## 🎯 Roadmap

**Phase 1: Foundation** (Current)
- Basic compositor structure
- Config file parsing
- Example configurations

**Phase 2: Core Features**
- XML → UI rendering
- CSS → style application
- JS → behavior system

**Phase 3: Polish**
- Hot reloading
- Performance optimization
- Documentation

**Phase 4: Ecosystem**
- Plugin system
- Theme repository
- Community tools

---

**Note:** This is a prototype/proof-of-concept. It's not production-ready yet, but demonstrates the viability of web-tech-based compositor configuration.
