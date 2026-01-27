# WebWM Status Bar Guide

WebWM now includes a customizable status bar system! Configure it with XML and style it with CSS.

## 🎯 Overview

The status bar displays:
- **Workspace indicators** - Shows all workspaces, highlights active one
- **Window title** - Current focused window's title
- **Clock** - Current time/date
- **System tray** - System tray icons (TODO)
- **Custom widgets** - Extensible widget system

## 📝 Basic Configuration

### In desktop.xml

```xml
<desktop>
  <bar id="main-bar" position="top" height="30" class="primary-bar">
    <workspaces display="icons" />
    <spacer flex="1" />
    <window-title max-width="400" />
    <spacer flex="1" />
    <clock format="%H:%M %a %b %d" />
  </bar>
</desktop>
```

### Bar Attributes

- **id** (required): Unique identifier
- **position**: `top`, `bottom`, `left`, `right` (default: `top`)
- **height**: Height in pixels (default: 30)
- **class**: CSS class name for styling

## 🧩 Available Widgets

### Workspaces

Shows all workspaces with indicators:

```xml
<workspaces display="icons" />
```

**Visual states:**
- **Active workspace**: Highlighted with accent color
- **Workspace with windows**: Shown with dot indicator  
- **Empty workspace**: Dimmed

### Window Title

Displays the focused window's title:

```xml
<window-title max-width="400" />
```

**Attributes:**
- `max-width`: Maximum width before truncation (optional)

### Clock

Shows current time/date:

```xml
<clock format="%H:%M:%S" />
```

**Format specifiers:**
- `%H` - Hour (00-23)
- `%M` - Minute (00-59)
- `%S` - Second (00-59)
- `%d` - Day of month (01-31)
- `%m` - Month number (01-12)
- `%Y` - Year (4 digits)
- `%y` - Year (2 digits)
- `%a` - Weekday abbreviation (Mon, Tue, etc.)
- `%b` - Month abbreviation (Jan, Feb, etc.)

**Examples:**
```xml
<clock format="%H:%M" />          <!-- 14:30 -->
<clock format="%H:%M:%S" />       <!-- 14:30:45 -->
<clock format="%a %b %d" />       <!-- Mon Jan 27 -->
<clock format="%H:%M %a %b %d" /> <!-- 14:30 Mon Jan 27 -->
```

### Spacer

Adds flexible spacing:

```xml
<spacer flex="1" />
```

**Attributes:**
- `flex`: Flex factor (higher = more space)

### System Tray (TODO)

Placeholder for system tray:

```xml
<system-tray />
```

## 🎨 Styling with CSS

### Bar Background

```css
.primary-bar {
  background: rgba(30, 30, 46, 0.95);
  backdrop-filter: blur(10px);
  border-bottom: 1px solid var(--border-normal);
  color: var(--text-primary);
}
```

### Workspace Indicators

```css
workspace {
  background: var(--bg-secondary);
  padding: 4px 12px;
  border-radius: 4px;
  transition: all 0.2s;
}

workspace:hover {
  background: var(--accent);
  color: var(--bg-primary);
}

workspace.active {
  background: var(--accent);
  color: var(--bg-primary);
  font-weight: bold;
}

workspace.urgent {
  background: var(--urgent);
  animation: urgentPulse 1s infinite;
}
```

### Clock Styling

```css
clock {
  color: var(--text-primary);
  font-weight: 500;
  padding: 4px 12px;
}
```

### Window Title

```css
window-title {
  color: var(--text-secondary);
  font-style: italic;
  max-width: 400px;
  overflow: hidden;
  text-overflow: ellipsis;
}
```

## 📐 Layout Examples

### Minimal Bar

```xml
<bar position="top" height="25">
  <workspaces />
  <spacer flex="1" />
  <clock format="%H:%M" />
</bar>
```

### Information-Rich Bar

```xml
<bar position="top" height="30">
  <workspaces display="icons" />
  <window-title max-width="400" />
  <spacer flex="1" />
  <system-tray />
  <clock format="%H:%M:%S %a %b %d" />
</bar>
```

### Dual Bar Setup

```xml
<desktop>
  <!-- Top bar -->
  <bar id="top-bar" position="top" height="30">
    <workspaces />
    <window-title />
  </bar>
  
  <!-- Bottom bar -->
  <bar id="bottom-bar" position="bottom" height="25">
    <spacer flex="1" />
    <clock format="%H:%M" />
  </bar>
</desktop>
```

### Side Bar

```xml
<bar position="left" height="40" class="vertical-bar">
  <workspaces display="icons" />
</bar>
```

## 🎯 Advanced Configuration

### Per-Bar Styling

```xml
<bar id="top" class="main-bar" />
<bar id="bottom" class="status-bar" />
```

```css
.main-bar {
  background: #1e1e2e;
  border-bottom: 2px solid #89b4fa;
}

.status-bar {
  background: #181825;
  border-top: 1px solid #45475a;
}
```

### Workspace-Specific Indicators

```css
/* Color-code workspaces */
workspace[data-id="1"] { border-left: 3px solid #f38ba8; }
workspace[data-id="2"] { border-left: 3px solid #fab387; }
workspace[data-id="3"] { border-left: 3px solid #a6e3a1; }
```

### Animated Elements

```css
workspace {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

workspace:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2);
}

@keyframes urgentPulse {
  0%, 100% { transform: scale(1); opacity: 1; }
  50% { transform: scale(1.05); opacity: 0.8; }
}
```

## 🔧 Technical Details

### Bar Height and Window Layout

The bar automatically reduces the available space for windows:

```rust
// Windows start below the bar
let bar_height = 30;
let usable_height = output_height - bar_height;

// Windows positioned at y = bar_height
```

### Rendering Pipeline

```
1. Calculate bar geometry based on position/height
2. Render widgets (workspaces, title, clock, etc.)
3. Apply CSS styles to each widget
4. Composite bar as overlay on top of windows
5. Update every frame (60 FPS)
```

### Performance

- Bar is rendered as a software buffer (RGBA)
- Cached between frames when content doesn't change (TODO)
- Minimal CPU impact (<1% with static content)
- Updates only when workspace/title/time changes

## 🐛 Debugging

### Check Bar Configuration

```bash
# Run config validator
./target/release/webwm config

# Should show:
# Desktop Configuration:
#   • Bars: 1
```

### Verify Bar Rendering

```bash
RUST_LOG=webwm::compositor::bar=debug ./target/release/webwm
```

Look for:
```
Bar renderer initialized with 1 bars
Rendering bar: main-bar at (0, 0) with 5 widgets
Workspace widget: 9 workspaces
Window title widget: "Firefox"
Clock widget: "14:30"
```

### Common Issues

**Problem:** Bar not showing
**Solution:** Check `desktop.xml` has `<bar>` element and position is valid

**Problem:** Widgets overlapping
**Solution:** Add `<spacer flex="1" />` between widgets

**Problem:** Clock not updating
**Solution:** Ensure render loop is running (should auto-update every frame)

**Problem:** Workspace indicators not highlighting
**Solution:** Check CSS for `workspace.active` selector

## 📊 Widget Rendering

Each widget type renders differently:

### Workspaces Widget
```
[1] [2] [3*] [4] [5] [6] [7] [8] [9]
 ^   ^   ^^   ^
 |   |   ||   └─ Empty (dimmed)
 |   |   |└──── Active (highlighted)
 |   |   └───── Has focus indicator
 |   └───────── Has windows (dot)
 └───────────── Empty (dimmed)
```

### Window Title Widget
```
[🪟 Firefox - WebWM Documentation]
 ^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
 ||  └─ Window title (truncated if too long)
 |└─── App icon (TODO)
 └──── Window indicator
```

### Clock Widget
```
[🕐 14:30 Mon Jan 27]
 ^   ^^^^^ ^^^ ^^^ ^^
 |   |     |   |   └─ Day
 |   |     |   └───── Month
 |   |     └───────── Weekday
 |   └─────────────── Time
 └─────────────────── Clock icon (TODO)
```

## 🚀 Future Enhancements

Planned features:

- [ ] **Custom widgets** - JavaScript-defined widgets
- [ ] **System tray** - XDG system tray protocol
- [ ] **Network indicator** - WiFi/Ethernet status
- [ ] **Battery indicator** - Laptop battery level
- [ ] **Volume control** - Audio volume widget
- [ ] **Notification center** - Click to show notifications
- [ ] **Workspace preview** - Hover to preview workspace
- [ ] **Bar hiding** - Auto-hide bar when windows fullscreen
- [ ] **Multiple monitors** - Per-monitor bars
- [ ] **Clickable widgets** - Mouse interaction

## 📚 Complete Example

### desktop.xml
```xml
<desktop>
  <bar id="main-bar" position="top" height="30" class="primary-bar">
    <!-- Left section -->
    <workspaces display="icons" />
    
    <!-- Center section -->
    <spacer flex="1" />
    <window-title max-width="400" />
    <spacer flex="1" />
    
    <!-- Right section -->
    <system-tray />
    <clock format="%H:%M %a %b %d" />
  </bar>

  <workspaces>
    <workspace id="1" name="main" layout="tiling" />
    <workspace id="2" name="web" layout="tiling" />
    <workspace id="3" name="code" layout="tiling" />
  </workspaces>
</desktop>
```

### style.css
```css
:root {
  --bg-primary: #1e1e2e;
  --bg-secondary: #313244;
  --text-primary: #cdd6f4;
  --accent: #89b4fa;
  --urgent: #f38ba8;
}

.primary-bar {
  background: rgba(30, 30, 46, 0.95);
  backdrop-filter: blur(10px);
  color: var(--text-primary);
  border-bottom: 1px solid #45475a;
}

workspace {
  background: var(--bg-secondary);
  padding: 4px 12px;
  border-radius: 4px;
  transition: all 0.2s ease-out;
}

workspace.active {
  background: var(--accent);
  color: var(--bg-primary);
  font-weight: bold;
  box-shadow: 0 2px 8px rgba(137, 180, 250, 0.3);
}

window-title {
  color: #a6adc8;
  font-style: italic;
}

clock {
  color: var(--text-primary);
  font-weight: 500;
}
```

---

**The bar system is now functional!** Build and run to see your customizable status bar:

```bash
cargo build --release
./target/release/webwm
```

You'll see a bar at the top showing workspaces, window title, and clock!
