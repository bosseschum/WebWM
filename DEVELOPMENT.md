# WebWM Development Guide

This guide explains how to work on and extend WebWM.

## 🏗️ Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    User Configuration                    │
│                                                          │
│  desktop.xml  ──┐                                        │
│  style.css    ──┼──→  Config Parser  ──→  Unified Config│
│  config.js    ──┘                                        │
└─────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────┐
│                   Compositor Core                        │
│                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │   Window     │  │    Layout    │  │    Input     │  │
│  │  Management  │  │   Engine     │  │   Handler    │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
│                                                          │
│  ┌──────────────────────────────────────────────────┐  │
│  │           Smithay (Wayland Protocol)             │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────┐
│                    Rendering Pipeline                    │
│                                                          │
│  ┌────────────┐  ┌────────────┐  ┌──────────────────┐  │
│  │   Window   │  │    CSS     │  │   Decoration    │  │
│  │ Compositor │→ │   Styler   │→ │    Renderer      │  │
│  └────────────┘  └────────────┘  └──────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## 📂 Project Structure

```
webwm/
├── src/
│   ├── main.rs              # Entry point & CLI
│   ├── config/
│   │   ├── mod.rs           # Config module
│   │   ├── xml_parser.rs    # XML → DesktopConfig
│   │   ├── css_parser.rs    # CSS → StyleSheet
│   │   └── js_runtime.rs    # JS execution & API
│   ├── compositor/
│   │   ├── mod.rs           # Compositor initialization
│   │   ├── wayland.rs       # Wayland protocol handling
│   │   └── surface.rs       # Surface management
│   ├── layout/
│   │   ├── mod.rs           # Layout trait & manager
│   │   ├── tiling.rs        # Tiling layouts
│   │   └── floating.rs      # Floating layout
│   ├── render/
│   │   ├── mod.rs           # Render pipeline
│   │   ├── window.rs        # Window rendering
│   │   ├── decoration.rs    # CSS-styled decorations
│   │   └── ui.rs            # Bar/panel rendering
│   ├── input/
│   │   ├── mod.rs           # Input handling
│   │   ├── keyboard.rs      # Keyboard events
│   │   └── pointer.rs       # Mouse events
│   └── state.rs             # Global compositor state
├── config/
│   ├── desktop.xml          # Example structure
│   ├── style.css            # Example styling
│   └── config.js            # Example logic
├── tests/
│   ├── parser_tests.rs      # Config parser tests
│   └── integration/         # Integration tests
└── docs/
    ├── API.md               # JavaScript API reference
    ├── XML_REFERENCE.md     # XML element reference
    └── CSS_REFERENCE.md     # Supported CSS properties
```

## 🔧 Development Setup

### Prerequisites

```bash
# Arch Linux
sudo pacman -S wayland wayland-protocols libxkbcommon mesa \
               libinput libudev0 libseat pkg-config

# Ubuntu/Debian
sudo apt install libwayland-dev wayland-protocols \
                 libxkbcommon-dev libgles-dev \
                 libinput-dev libudev-dev libseat-dev \
                 pkg-config

# Fedora
sudo dnf install wayland-devel wayland-protocols-devel \
                 libxkbcommon-devel mesa-libGLES-devel \
                 libinput-devel systemd-devel libseat-devel \
                 pkg-config
```

### Building

```bash
# Debug build (faster compilation)
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- config
```

## 🛠️ Current Implementation Status

### ✅ Completed
- [x] Project structure
- [x] XML parser (desktop.xml)
- [x] CSS parser (style.css)
- [x] JavaScript runtime integration
- [x] Configuration loading system
- [x] Basic data structures
- [x] Example configuration files

### 🚧 In Progress
- [ ] Smithay compositor integration
- [ ] Event loop implementation
- [ ] Window management
- [ ] Layout algorithms

### 📋 TODO
- [ ] Window rendering with CSS styles
- [ ] Input event handling
- [ ] Keybinding dispatch
- [ ] IPC for live reload
- [ ] Bar/panel rendering
- [ ] Workspace management
- [ ] Animation system
- [ ] Hot reload support

## 🎯 Next Development Steps

### Phase 1: Core Compositor (Priority)

1. **Initialize Smithay Compositor**
   - Set up Wayland socket
   - Handle client connections
   - Implement basic surface management

2. **Window Management**
   - Create/destroy windows
   - Focus management
   - Window mapping/unmapping

3. **Basic Layout Engine**
   - Implement tiling algorithm
   - Window positioning
   - Resize handling

### Phase 2: Styling & Rendering

1. **CSS Style Application**
   - Apply border colors from CSS
   - Implement border-width
   - Handle border-radius (if possible)

2. **Window Decorations**
   - Render title bars
   - Window buttons (close, minimize, maximize)
   - Apply CSS styles to decorations

3. **Bar Rendering**
   - Parse bar configuration from XML
   - Render bar widgets
   - Apply CSS styles to bar

### Phase 3: Input & Interaction

1. **Keyboard Input**
   - Capture keyboard events
   - Match against keybindings
   - Execute actions (JS callbacks or built-in)

2. **Mouse Input**
   - Window focus on click
   - Window dragging
   - Resize by dragging edges

3. **JavaScript Actions**
   - Execute JS callbacks for keybindings
   - Provide window object to JS handlers
   - Implement wm API methods

### Phase 4: Polish & Features

1. **Animations**
   - Window open/close animations
   - Workspace switching animations
   - Use CSS transition timings

2. **Hot Reload**
   - Watch config files for changes
   - Reload without restart
   - Apply new styles live

3. **Multi-Monitor**
   - Detect monitors
   - Per-monitor configuration
   - Window placement across monitors

## 🧪 Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_xml_parsing() {
        let xml = r#"
            <desktop>
                <bar position="top" />
            </desktop>
        "#;
        let config = parse_desktop_xml(xml).unwrap();
        assert_eq!(config.bars.len(), 1);
    }
}
```

### Integration Tests
```rust
// tests/integration/config_loading.rs
#[test]
fn test_full_config_loading() {
    let config = load_config("./test_configs/basic").unwrap();
    assert!(config.keybindings.len() > 0);
}
```

### Manual Testing
```bash
# Test with different configs
./target/release/webwm ./config
./target/release/webwm ./examples/minimal
./target/release/webwm ./examples/advanced

# Test with validation
./target/release/webwm --validate ./config

# Test hot reload
./target/release/webwm --watch ./config
```

## 🐛 Debugging Tips

### Enable Logging
```bash
# All debug output
RUST_LOG=debug cargo run

# Specific modules
RUST_LOG=webwm::config=debug,webwm::render=trace cargo run

# Wayland protocol debugging
WAYLAND_DEBUG=1 cargo run
```

### Common Issues

**Problem:** JS runtime errors
**Solution:** Check QuickJS error messages, ensure API is properly exposed

**Problem:** CSS not applying
**Solution:** Verify selector matching logic, check CSS parsing output

**Problem:** Keybindings not working
**Solution:** Check key name mapping, verify modifier parsing

## 📚 Resources

### Wayland Development
- [Wayland Book](https://wayland-book.com/)
- [Smithay Documentation](https://smithay.github.io/smithay/)
- [Wayland Protocol](https://wayland.freedesktop.org/docs/html/)

### Similar Projects to Study
- **Sway** - i3-compatible Wayland compositor (C)
- **Hyprland** - Modern tiling compositor (C++)
- **River** - Tiling Wayland compositor (Zig)
- **awesome** - X11 WM with Lua config

### Rust Resources
- [Rust Book](https://doc.rust-lang.org/book/)
- [rquickjs Documentation](https://docs.rs/rquickjs/)
- [cssparser Documentation](https://docs.rs/cssparser/)

## 🤝 Contributing

### Code Style
- Use `rustfmt` for formatting: `cargo fmt`
- Use `clippy` for linting: `cargo clippy`
- Write documentation comments for public APIs
- Add tests for new functionality

### Pull Request Process
1. Create feature branch from `main`
2. Implement changes with tests
3. Run `cargo test` and `cargo clippy`
4. Update documentation if needed
5. Submit PR with clear description

### Areas Needing Help
- [ ] Smithay compositor implementation
- [ ] Advanced CSS features (flexbox, grid)
- [ ] Animation system
- [ ] Multi-monitor support
- [ ] Accessibility features
- [ ] Documentation and examples

## 📝 API Documentation

See [API.md](./docs/API.md) for the complete JavaScript API reference.

### Quick Example
```javascript
// Keybinding with action
keybind('Super+Return', () => wm.spawn('alacritty'));

// Window rule with handler
onWindowCreate((window) => {
  if (window.appId === 'firefox') {
    window.moveToWorkspace(2);
    window.addClass('browser');
  }
});
```

## 🎨 Theming Guide

See [CSS_REFERENCE.md](./docs/CSS_REFERENCE.md) for supported properties.

### Quick Example
```css
window:focus {
  border: 2px solid var(--accent);
  box-shadow: 0 4px 20px rgba(137, 180, 250, 0.4);
  transition: all 0.2s ease-out;
}
```

---

**Questions?** Open an issue or start a discussion!
