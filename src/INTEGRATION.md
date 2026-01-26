# WebWM Compositor Integration Guide

This guide explains how the Smithay compositor integration works in WebWM.

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         main.rs                             │
│  • Parses command line arguments                            │
│  • Loads configuration                                      │
│  • Initializes compositor                                   │
│  • Runs event loop                                          │
└─────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┴───────────────┐
              ▼                               ▼
┌──────────────────────────┐    ┌──────────────────────────┐
│   compositor/mod.rs      │    │     backend.rs           │
│                          │    │                          │
│  • WebWMCompositor       │◄───│  • WebWMBackend          │
│  • Window management     │    │  • Winit integration     │
│  • Layout algorithms     │    │  • Rendering pipeline    │
│  • Wayland protocols     │    │  • Input handling        │
│  • Style application     │    │  • Output management     │
└──────────────────────────┘    └──────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Smithay Framework                         │
│  • Wayland protocol implementation                          │
│  • Buffer management                                        │
│  • Input event processing                                   │
│  • Rendering abstractions                                   │
└─────────────────────────────────────────────────────────────┘
```

## 📦 Key Components

### 1. WebWMCompositor (compositor/mod.rs)

The core compositor state that implements all Smithay traits:

- **CompositorHandler** - Handles surface commits
- **XdgShellHandler** - Manages XDG shell surfaces (windows)
- **ShmHandler** - Shared memory buffers
- **SeatHandler** - Input device management
- **DataDeviceHandler** - Clipboard/drag-and-drop

**Key responsibilities:**
- Window lifecycle (create, destroy, focus)
- Layout management (tiling, floating)
- Applying window rules from config
- Extracting CSS styles for rendering

### 2. WebWMBackend (backend.rs)

Handles the rendering backend and input:

- **Winit Backend** - Creates a window on your existing desktop (for development)
- **Rendering Loop** - 60 FPS render timer
- **Input Processing** - Keyboard and mouse events
- **Damage Tracking** - Only redraws changed areas

**Key responsibilities:**
- Initialize graphics backend
- Render windows with decorations
- Process input events
- Manage outputs/monitors

### 3. Main Event Loop (main.rs)

Coordinates everything:

- **Wayland Socket** - Listens for client connections
- **Render Timer** - Triggers redraws at 60 FPS
- **Input Sources** - Processes keyboard/mouse
- **Display Dispatch** - Handles Wayland protocol messages

## 🔄 Data Flow

### Window Creation Flow

```
1. Client creates XDG surface
   ↓
2. XdgShellHandler::new_toplevel() called
   ↓
3. WebWMCompositor::add_window()
   ↓
4. Apply window rules from config
   ↓
5. Add to Space (spatial management)
   ↓
6. Relayout all windows
   ↓
7. Send configure to client with size
```

### Rendering Flow

```
1. Timer fires (every 16ms)
   ↓
2. For each window:
   - Get location from Space
   - Collect render elements
   - Apply CSS styles (border color, etc.)
   ↓
3. Damage tracking determines what changed
   ↓
4. Render to framebuffer
   ↓
5. Submit frame
   ↓
6. Send frame callbacks to clients
```

### Input Flow

```
1. Winit receives input event
   ↓
2. WinitEvent::Input dispatched
   ↓
3. WebWMCompositor::handle_winit_event()
   ↓
4. Match against keybindings
   ↓
5. Execute action (spawn, close, focus, etc.)
   ↓
6. Forward to focused client
```

## 🎨 CSS Integration

### How Styles Are Applied

```rust
// In compositor/mod.rs
pub fn get_border_color(&self, window: &Window, focused: bool) -> [f32; 4] {
    if let Some(ref stylesheet) = self.stylesheet {
        // Try to get color from CSS
        let selector = if focused { "window:focus" } else { "window" };
        
        if let Some(color) = stylesheet.get_color(selector, "border-color") {
            return color.to_rgba_f32();
        }
    }

    // Fallback to config theme
    parse_hex_color(&self.config.theme.border_focused)
}
```

### Supported CSS Properties (Currently)

- `border-color` - Window border color
- `border-width` - Window border thickness
- CSS variables (via `var(--name)`)

### Future CSS Properties

- `border-radius` - Rounded corners
- `box-shadow` - Drop shadows
- `background` - Window backgrounds
- `opacity` - Window transparency
- `transition` - Animations

## 🔌 Smithay Delegates

WebWM implements these Smithay protocol handlers:

```rust
delegate_compositor!(WebWMCompositor);
delegate_xdg_shell!(WebWMCompositor);
delegate_shm!(WebWMCompositor);
delegate_seat!(WebWMCompositor);
delegate_data_device!(WebWMCompositor);
delegate_output!(WebWMCompositor);
```

Each delegate requires implementing specific traits that handle Wayland protocol messages.

## 🎯 Layout Algorithms

### Tiling Layout

```rust
fn layout_tiling(&mut self, output_size: Size, gaps: i32) {
    let window_count = self.windows.len();
    let available_height = output_size.h - (gaps * (window_count + 1));
    let window_height = available_height / window_count;
    let available_width = output_size.w - (gaps * 2);

    for (i, window) in self.windows.iter().enumerate() {
        let x = gaps;
        let y = gaps + (i * (window_height + gaps));
        
        self.space.map_element(window.clone(), (x, y), false);
        
        window.configure_size((available_width, window_height));
    }
}
```

### Floating Layout

```rust
fn layout_floating(&mut self, output_size: Size) {
    for (i, window) in self.windows.iter().enumerate() {
        let offset = i * 30; // Cascade windows
        let x = (output_size.w - 800) / 2 + offset;
        let y = (output_size.h - 600) / 2 + offset;
        
        self.space.map_element(window.clone(), (x, y), false);
        window.configure_size((800, 600));
    }
}
```

## 🐛 Debugging

### Enable Logging

```bash
# All logs
RUST_LOG=debug cargo run

# Specific modules
RUST_LOG=webwm::compositor=debug cargo run

# Wayland protocol
WAYLAND_DEBUG=1 cargo run
```

### Common Issues

**Problem:** "Failed to initialize backend"
**Solution:** Make sure you have graphics drivers installed

**Problem:** Clients can't connect
**Solution:** Check the socket name is correct: `WAYLAND_DISPLAY=wayland-1 alacritty`

**Problem:** Windows not rendering
**Solution:** Check `Space::map_element()` is called and render loop is running

**Problem:** Input not working
**Solution:** Verify seat has keyboard/pointer added

## 🔧 Extending the Compositor

### Adding a New Layout Mode

1. Add mode to config:
```rust
// In config.rs
pub enum LayoutMode {
    Tiling,
    Floating,
    Monocle, // NEW
}
```

2. Implement layout algorithm:
```rust
// In compositor/mod.rs
fn layout_monocle(&mut self, output_size: Size) {
    // Fullscreen the focused window
    if let Some(focused) = self.windows.first() {
        self.space.map_element(focused.clone(), (0, 0), false);
        focused.configure_size((output_size.w, output_size.h));
    }
}
```

3. Call from relayout:
```rust
match self.config.layout.default_mode.as_str() {
    "tiling" => self.layout_tiling(size, gaps),
    "floating" => self.layout_floating(size),
    "monocle" => self.layout_monocle(size), // NEW
    _ => self.layout_tiling(size, gaps),
}
```

### Adding Input Handling

```rust
// In compositor/mod.rs
impl WebWMCompositor {
    pub fn handle_key_press(&mut self, keycode: u32, mods: ModifiersState) {
        // Check keybindings
        for kb in &self.config.keybindings {
            if self.matches_keybinding(keycode, mods, kb) {
                self.execute_action(&kb.action);
                return;
            }
        }
    }
    
    fn execute_action(&mut self, action: &Action) {
        match action {
            Action::Spawn { command } => {
                std::process::Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .spawn()
                    .ok();
            }
            Action::Close => {
                if let Some(window) = self.windows.first() {
                    window.toplevel().unwrap().send_close();
                }
            }
            // ... handle other actions
        }
    }
}
```

### Adding Custom Decorations

```rust
// In backend.rs - rendering
fn render_decorations(&mut self, window: &Window) {
    let border_color = self.compositor.get_border_color(window, focused);
    let border_width = self.compositor.get_border_width();
    
    // Draw border rectangle
    renderer.draw_rectangle(
        location,
        size,
        border_color,
        border_width,
    );
    
    // Draw title bar if configured
    if let Some(title_bar) = self.get_title_bar_config() {
        renderer.draw_title_bar(window, title_bar);
    }
}
```

## 📚 Resources

### Smithay Documentation
- [Smithay Book](https://smithay.github.io/book/)
- [API Docs](https://docs.rs/smithay/)
- [Examples](https://github.com/Smithay/smithay/tree/master/anvil)

### Wayland Protocol
- [Wayland Book](https://wayland-book.com/)
- [Protocol Spec](https://wayland.freedesktop.org/docs/html/)
- [XDG Shell](https://wayland.app/protocols/xdg-shell)

### Similar Compositors
- **Anvil** - Smithay's reference compositor
- **Sway** - i3-compatible Wayland compositor
- **Cosmic** - System76's Rust compositor

## 🚀 Next Steps

1. **Test the compositor:**
   ```bash
   cargo build --release
   ./target/release/webwm
   ```

2. **Connect a client:**
   ```bash
   WAYLAND_DISPLAY=wayland-1 alacritty
   ```

3. **Implement keybindings** - Map keyboard input to actions

4. **Add workspace support** - Multiple virtual desktops

5. **Implement decorations** - Title bars, buttons

6. **Add animations** - Use CSS transition timings

---

**The compositor is now functional!** You can create windows, they'll be laid out according to your config, and styles from CSS are applied.
