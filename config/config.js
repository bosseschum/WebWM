// WebWM Configuration
// This file defines keybindings, window rules, and custom behavior

// Compositor API (would be injected by the Rust backend)
const wm = {
  spawn: (cmd) => console.log(`Spawn: ${cmd}`),
  close: () => console.log('Close window'),
  focus: (dir) => console.log(`Focus: ${dir}`),
  moveToWorkspace: (ws) => console.log(`Move to workspace: ${ws}`),
  toggleFloating: () => console.log('Toggle floating'),
  reload: () => console.log('Reload config'),
};

// ============================================================================
// KEYBINDINGS
// ============================================================================

// Modifier key (Super/Mod4)
const MOD = 'Super';
const ALT = 'Alt';
const SHIFT = 'Shift';
const CTRL = 'Control';

// Application launchers
keybind(`${MOD}+Return`, () => wm.spawn('alacritty'));
keybind(`${MOD}+d`, () => wm.spawn('rofi -show drun'));
keybind(`${MOD}+w`, () => wm.spawn('firefox'));
keybind(`${MOD}+e`, () => wm.spawn('thunar'));

// Window management
keybind(`${MOD}+q`, () => wm.close());
keybind(`${MOD}+f`, () => wm.toggleFloating());
keybind(`${MOD}+m`, () => wm.toggleMaximize());

// Focus management
keybind(`${MOD}+h`, () => wm.focus('left'));
keybind(`${MOD}+j`, () => wm.focus('down'));
keybind(`${MOD}+k`, () => wm.focus('up'));
keybind(`${MOD}+l`, () => wm.focus('right'));

// Move windows
keybind(`${MOD}+${SHIFT}+h`, () => wm.moveWindow('left'));
keybind(`${MOD}+${SHIFT}+j`, () => wm.moveWindow('down'));
keybind(`${MOD}+${SHIFT}+k`, () => wm.moveWindow('up'));
keybind(`${MOD}+${SHIFT}+l`, () => wm.moveWindow('right'));

// Workspace switching
for (let i = 1; i <= 9; i++) {
  keybind(`${MOD}+${i}`, () => wm.switchToWorkspace(i));
  keybind(`${MOD}+${SHIFT}+${i}`, () => wm.moveToWorkspace(i));
}

// Layout controls
keybind(`${MOD}+t`, () => wm.setLayout('tiling'));
keybind(`${MOD}+space`, () => wm.cycleLayout());

// System
keybind(`${MOD}+${SHIFT}+r`, () => wm.reload());
keybind(`${MOD}+${SHIFT}+e`, () => wm.exit());

// ============================================================================
// WINDOW RULES
// ============================================================================

// Rule: Firefox always goes to workspace 2
onWindowCreate((window) => {
  if (window.appId === 'firefox') {
    window.moveToWorkspace(2);
    window.addClass('browser');
  }
});

// Rule: Picture-in-Picture windows are always floating and sticky
onWindowCreate((window) => {
  if (window.title?.includes('Picture-in-Picture')) {
    window.setFloating(true);
    window.setSticky(true);
    window.resize(480, 270);
  }
});

// Rule: Terminal windows get specific size when floating
onWindowCreate((window) => {
  if (window.appId === 'alacritty' && window.isFloating) {
    window.resize(1000, 600);
    window.center();
  }
});

// Rule: Spotify to workspace 4, floating
onWindowCreate((window) => {
  if (window.appId === 'spotify') {
    window.moveToWorkspace(4);
    window.setFloating(true);
    window.addClass('media');
  }
});

// ============================================================================
// CUSTOM BEHAVIORS
// ============================================================================

// Auto-focus windows on mouse hover (with delay)
let hoverTimeout = null;
onMouseEnter((window) => {
  hoverTimeout = setTimeout(() => {
    window.focus();
  }, 200);
});

onMouseLeave(() => {
  if (hoverTimeout) {
    clearTimeout(hoverTimeout);
    hoverTimeout = null;
  }
});

// Smart gaps: disable gaps when only one window
onLayoutChange((workspace) => {
  if (workspace.windows.length === 1) {
    workspace.setGaps(0);
  } else {
    workspace.setGaps(10);
  }
});

// Urgent window notification
onWindowUrgent((window) => {
  notify({
    title: 'Window needs attention',
    body: `${window.title || window.appId} on workspace ${window.workspace}`,
    timeout: 3000,
  });
});

// ============================================================================
// STARTUP COMMANDS
// ============================================================================

onStartup(() => {
  // Start compositor services
  wm.spawn('wl-paste --watch cliphist store'); // Clipboard manager
  wm.spawn('mako'); // Notification daemon
  wm.spawn('waybar'); // Optional: external bar
  
  // Set wallpaper
  wm.spawn('swaybg -i ~/Pictures/wallpaper.jpg -m fill');
  
  // Auto-start applications
  wm.spawn('firefox');
  wm.spawn('discord');
  
  console.log('WebWM started successfully!');
});

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

function keybind(combo, action) {
  // This would register with the Rust backend
  // For now, just log it
  console.log(`Registered keybinding: ${combo}`);
}

function onWindowCreate(handler) {
  // Hook into window creation events
  console.log('Registered window creation handler');
}

function onMouseEnter(handler) {
  console.log('Registered mouse enter handler');
}

function onMouseLeave(handler) {
  console.log('Registered mouse leave handler');
}

function onLayoutChange(handler) {
  console.log('Registered layout change handler');
}

function onWindowUrgent(handler) {
  console.log('Registered urgent window handler');
}

function onStartup(handler) {
  console.log('Registered startup handler');
  // Execute immediately in this context
  handler();
}

function notify(options) {
  console.log('Notification:', options);
}
