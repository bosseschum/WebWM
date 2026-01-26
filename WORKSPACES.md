# WebWM Workspace Management Guide

WebWM now has full workspace support! This guide explains how to use and configure workspaces.

## 🎯 What Are Workspaces?

Workspaces (also called virtual desktops) let you organize windows into separate groups:
- **Workspace 1**: Browser and email
- **Workspace 2**: Code editor and terminals  
- **Workspace 3**: Music and chat apps
- etc.

Only windows in the **active workspace** are visible and rendered.

## 📋 Default Configuration

WebWM creates **9 workspaces** by default (numbered 1-9).

Each workspace can have its own:
- Layout mode (tiling, floating, monocle)
- Windows
- Focus state

## ⌨️ Keybindings

### Switch to Workspace

```javascript
// config.js
keybind('Super+1', () => wm.switchToWorkspace(1));
keybind('Super+2', () => wm.switchToWorkspace(2));
// ... up to Super+9
```

**Or use a loop:**
```javascript
for (let i = 1; i <= 9; i++) {
  keybind(`Super+${i}`, () => wm.switchToWorkspace(i));
}
```

### Move Window to Workspace

```javascript
// Move focused window to workspace N
keybind('Super+Shift+1', () => wm.moveToWorkspace(1));
keybind('Super+Shift+2', () => wm.moveToWorkspace(2));
// ... etc

// Or with loop:
for (let i = 1; i <= 9; i++) {
  keybind(`Super+Shift+${i}`, () => wm.moveToWorkspace(i));
}
```

### Cycle Through Workspaces

```javascript
// Next workspace
keybind('Super+Tab', () => wm.cycleWorkspaceNext());

// Previous workspace
keybind('Super+Shift+Tab', () => wm.cycleWorkspacePrev());
```

## 🎨 Configuring Workspaces in XML

Define workspaces in `desktop.xml`:

```xml
<workspaces>
  <workspace id="1" name="main" layout="tiling">
    <split-ratio>0.6</split-ratio>
  </workspace>
  
  <workspace id="2" name="web" layout="tiling" />
  
  <workspace id="3" name="code" layout="tiling" />
  
  <workspace id="4" name="media" layout="floating" />
  
  <workspace id="5" name="fullscreen" layout="monocle" />
</workspaces>
```

### Workspace Attributes

- **id** (required): Workspace number (1-9)
- **name** (optional): Human-readable name (default: same as id)
- **layout** (optional): `tiling`, `floating`, or `monocle` (default: `tiling`)

### Layout Modes

**Tiling**: Windows automatically arranged in non-overlapping tiles
**Floating**: Windows cascade with manual positioning
**Monocle**: Fullscreen focused window, others hidden

## 🎯 Window Rules with Workspaces

Assign apps to specific workspaces automatically:

### In XML

```xml
<window-rules>
  <rule app-id="firefox" workspace="2" />
  <rule app-id="code" workspace="3" />
  <rule app-id="spotify" workspace="4" floating="true" />
</window-rules>
```

### In JavaScript

```javascript
onWindowCreate((window) => {
  if (window.appId === 'firefox') {
    window.moveToWorkspace(2);
  }
  
  if (window.appId === 'discord') {
    window.moveToWorkspace(4);
  }
});
```

## 🔧 JavaScript API

### Switch Workspace

```javascript
wm.switchToWorkspace(workspace_number);

// Example
wm.switchToWorkspace(2);  // Switch to workspace 2
```

### Move Window

```javascript
wm.moveToWorkspace(workspace_number);

// Moves the currently focused window
```

### Cycle Workspaces

```javascript
wm.cycleWorkspaceNext();   // Go to next workspace
wm.cycleWorkspacePrev();   // Go to previous workspace
```

## 📊 Workspace State

Each workspace independently tracks:
- **Windows**: List of windows on that workspace
- **Focus**: Which window is focused in that workspace
- **Layout**: Tiling, floating, or monocle mode

When you switch workspaces:
1. Current workspace windows are hidden
2. New workspace windows are shown
3. Layout is recalculated for new workspace
4. Focus is restored to previously focused window

## 🎮 Advanced Usage

### Per-Workspace Layout

```xml
<workspace id="1" layout="tiling" />
<workspace id="2" layout="tiling" />
<workspace id="3" layout="tiling" />
<workspace id="4" layout="floating" />
<workspace id="5" layout="monocle" />
```

- Workspaces 1-3: Tiling (for productivity)
- Workspace 4: Floating (for casual apps)
- Workspace 5: Monocle (for focus/fullscreen)

### Conditional Window Placement

```javascript
onWindowCreate((window) => {
  // Browsers go to workspace 2
  if (window.appId.includes('firefox') || 
      window.appId.includes('chrome')) {
    window.moveToWorkspace(2);
  }
  
  // IDEs go to workspace 3
  if (window.appId === 'code' || 
      window.appId === 'intellij') {
    window.moveToWorkspace(3);
  }
  
  // Communication apps go to workspace 4
  if (window.appId === 'discord' || 
      window.appId === 'slack') {
    window.moveToWorkspace(4);
    window.setFloating(true);
  }
});
```

### Workspace-Specific Keybindings

```javascript
// Only show notification on workspace 1
keybind('Super+n', () => {
  if (getCurrentWorkspace() === 1) {
    notify({ title: 'On workspace 1!' });
  }
});
```

## 🐛 Debugging

### Check Active Workspace

When WebWM is running, watch the logs:

```
Switching to workspace 2
Window added to workspace 2: 1 total windows in workspace
Relayout: 1 windows in tiling mode on workspace 2 (gaps: 10px)
```

### Verify Workspace Switching

```bash
# Start WebWM
./target/release/webwm

# In another terminal, launch apps
WAYLAND_DISPLAY=wayland-1 alacritty  # Appears on workspace 1

# Press Super+2 in WebWM window
# Window disappears (now on workspace 2)

# Launch another app
WAYLAND_DISPLAY=wayland-1 alacritty  # Appears on workspace 2

# Press Super+1
# First window reappears
```

### Common Issues

**Problem:** "Workspace 5 does not exist"
**Solution:** Make sure the workspace is defined in desktop.xml or use default workspaces (1-9)

**Problem:** Window doesn't move to workspace
**Solution:** Check window rule `app-id` matches actual app ID. Use `WAYLAND_DEBUG=1` to see app IDs.

**Problem:** Can't switch to workspace
**Solution:** Verify keybinding is registered. Check logs for "Matched keybinding" messages.

## 📈 Performance

Workspaces improve performance by:
- Only rendering windows in active workspace
- Reducing layout calculations
- Isolating window management

With 20 windows across 4 workspaces:
- **Without workspaces**: Rendering 20 windows every frame
- **With workspaces**: Rendering ~5 windows per workspace

## 🎯 Future Enhancements

Planned features:
- [ ] Workspace persistence (remember windows after restart)
- [ ] Named workspaces (use names instead of numbers)
- [ ] Workspace-specific gaps and borders
- [ ] Workspace indicators in bar
- [ ] Scratchpad workspace
- [ ] Workspace groups/tags

## 📚 Examples

### Minimal Setup

```javascript
// Just workspace switching
for (let i = 1; i <= 5; i++) {
  keybind(`Super+${i}`, () => wm.switchToWorkspace(i));
}
```

### Power User Setup

```javascript
// Workspace switching (1-9)
for (let i = 1; i <= 9; i++) {
  keybind(`Super+${i}`, () => wm.switchToWorkspace(i));
  keybind(`Super+Shift+${i}`, () => wm.moveToWorkspace(i));
}

// Cycling
keybind('Super+Tab', () => wm.cycleWorkspaceNext());
keybind('Super+Shift+Tab', () => wm.cycleWorkspacePrev());

// Auto-assign apps
onWindowCreate((window) => {
  const rules = {
    'firefox': 2,
    'code': 3,
    'spotify': 4,
    'discord': 4,
    'gimp': 5,
  };
  
  const workspace = rules[window.appId];
  if (workspace) {
    window.moveToWorkspace(workspace);
  }
});
```

### Workspace + Focus Management

```javascript
// Switch workspace and focus first window
function switchAndFocus(ws) {
  wm.switchToWorkspace(ws);
  wm.focus('down');  // Focus first window
}

keybind('Super+1', () => switchAndFocus(1));
keybind('Super+2', () => switchAndFocus(2));
// ... etc
```

## 🎨 Styling Workspace Indicators (TODO)

Future feature - style workspace indicators in the bar:

```css
workspace {
  background: var(--bg-secondary);
  padding: 4px 12px;
  transition: all 0.2s;
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

---

**Workspaces are now fully functional!** Try them out:

```bash
cargo build --release
./target/release/webwm

# Connect some clients
WAYLAND_DISPLAY=wayland-1 alacritty
# Press Super+2
WAYLAND_DISPLAY=wayland-1 alacritty
# Press Super+1 to go back
```
