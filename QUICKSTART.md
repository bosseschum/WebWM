# WebWM Quick Start Guide

Get up and running with WebWM in 5 minutes!

## 🚀 Installation

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Install Dependencies

**Arch Linux:**
```bash
sudo pacman -S wayland wayland-protocols libxkbcommon mesa libinput libseat
```

**Ubuntu/Debian:**
```bash
sudo apt install libwayland-dev wayland-protocols libxkbcommon-dev \
                 libgles-dev libinput-dev libudev-dev libseat-dev
```

**Fedora:**
```bash
sudo dnf install wayland-devel wayland-protocols-devel libxkbcommon-devel \
                 mesa-libGLES-devel libinput-devel systemd-devel libseat-devel
```

### 3. Clone and Build

```bash
git clone https://github.com/yourusername/webwm
cd webwm
cargo build --release
```

## 📝 Configuration Basics

WebWM uses three files for configuration:

### 1. desktop.xml - Structure

Defines your desktop layout, bars, workspaces, and window rules.

```xml
<?xml version="1.0" encoding="UTF-8"?>
<desktop>
  <!-- Top bar -->
  <bar id="main-bar" position="top" height="30">
    <workspaces />
    <window-title />
    <clock format="%H:%M" />
  </bar>

  <!-- Workspaces -->
  <workspaces>
    <workspace id="1" name="main" layout="tiling" />
    <workspace id="2" name="web" layout="tiling" />
    <workspace id="3" name="code" layout="tiling" />
  </workspaces>

  <!-- Window rules -->
  <window-rules>
    <rule app-id="firefox" workspace="2" />
    <rule app-id="code" workspace="3" />
  </window-rules>
</desktop>
```

### 2. style.css - Styling

Style your windows, bars, and UI elements just like a website!

```css
/* CSS Variables for easy theming */
:root {
  --bg-primary: #1e1e2e;
  --accent: #89b4fa;
  --border-focus: #89b4fa;
  --border-normal: #45475a;
}

/* Window styles */
window {
  border: 2px solid var(--border-normal);
  border-radius: 8px;
  transition: all 0.2s ease-out;
}

window:focus {
  border-color: var(--border-focus);
  box-shadow: 0 4px 20px rgba(137, 180, 250, 0.4);
}

/* Bar styles */
.primary-bar {
  background: rgba(30, 30, 46, 0.95);
  backdrop-filter: blur(10px);
}
```

### 3. config.js - Behavior

Define keybindings, window rules, and custom behaviors with JavaScript!

```javascript
// Keybindings
keybind('Super+Return', () => wm.spawn('alacritty'));
keybind('Super+d', () => wm.spawn('rofi -show drun'));
keybind('Super+q', () => wm.close());

// Focus navigation
keybind('Super+h', () => wm.focus('left'));
keybind('Super+j', () => wm.focus('down'));
keybind('Super+k', () => wm.focus('up'));
keybind('Super+l', () => wm.focus('right'));

// Workspace switching
for (let i = 1; i <= 9; i++) {
  keybind(`Super+${i}`, () => wm.switchToWorkspace(i));
}

// Window rules with custom logic
onWindowCreate((window) => {
  if (window.appId === 'firefox') {
    window.moveToWorkspace(2);
    window.addClass('browser');
  }
  
  if (window.title?.includes('Picture-in-Picture')) {
    window.setFloating(true);
    window.setSticky(true);
  }
});

// Startup commands
onStartup(() => {
  wm.spawn('swaybg -i ~/Pictures/wallpaper.jpg');
  wm.spawn('mako'); // notification daemon
});
```

## 🎨 Creating Your First Theme

### Step 1: Choose Colors

Pick a color scheme (e.g., from [Catppuccin](https://catppuccin.com/)):

```css
:root {
  --bg-primary: #1e1e2e;    /* Base background */
  --accent: #89b4fa;         /* Blue accent */
  --text-primary: #cdd6f4;   /* White text */
  --border-focus: #89b4fa;   /* Focused border */
  --border-normal: #45475a;  /* Normal border */
}
```

### Step 2: Style Windows

```css
window {
  border: 2px solid var(--border-normal);
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

window:focus {
  border-color: var(--border-focus);
  box-shadow: 0 4px 20px rgba(137, 180, 250, 0.4);
}
```

### Step 3: Add Animations

```css
window {
  transition: all 0.2s ease-out;
}

workspace {
  transition: all 0.2s ease-in-out;
}
```

### Step 4: Test It!

```bash
./target/release/webwm config
```

## 🎯 Common Tasks

### Add a Keybinding

In `config.js`:
```javascript
keybind('Super+b', () => wm.spawn('firefox'));
```

### Create a Window Rule

In `desktop.xml`:
```xml
<rule app-id="spotify" workspace="4" floating="true" />
```

Or in `config.js`:
```javascript
onWindowCreate((window) => {
  if (window.appId === 'spotify') {
    window.moveToWorkspace(4);
    window.setFloating(true);
  }
});
```

### Change Gap Size

In `style.css`:
```css
:root {
  --gap-size: 15px;  /* Change from 10px to 15px */
}

.tiling-container {
  gap: var(--gap-size);
}
```

Or in `desktop.xml`:
```xml
<tiling>
  <gaps outer="15" inner="15" />
</tiling>
```

### Add a Bar Widget

In `desktop.xml`:
```xml
<bar position="top">
  <workspaces />
  <spacer flex="1" />
  <system-tray />
  <clock format="%H:%M" />
</bar>
```

## 🐛 Troubleshooting

### Config not loading?

Check file locations:
```bash
ls -la config/
# Should show: desktop.xml, style.css, config.js
```

### Parse errors?

Run with verbose output:
```bash
RUST_LOG=debug ./target/release/webwm config
```

### JavaScript errors?

Check the JS syntax:
```bash
node config/config.js  # Quick syntax check
```

## 📚 Next Steps

1. **Read the full documentation**
   - [README.md](./README.md) - Project overview
   - [DEVELOPMENT.md](./DEVELOPMENT.md) - Development guide
   - [API.md](./docs/API.md) - JavaScript API reference

2. **Explore examples**
   - Check `examples/` directory for different configurations
   - Study other users' configs (when available)

3. **Join the community**
   - Report bugs and request features on GitHub
   - Share your configurations
   - Contribute to development

## 🎉 You're Ready!

You now have a web-tech-configured window manager! Customize it to your heart's content using familiar HTML/CSS/JS concepts.

**Pro Tips:**
- Use CSS variables for easy theme switching
- Keep JavaScript callbacks simple and fast
- Test changes incrementally
- Version control your config with git

**Happy customizing!** 🚀
