use smithay::backend::input::{
    InputEvent, KeyState, KeyboardKeyEvent, PointerAxisEvent, 
    PointerButtonEvent, PointerMotionEvent, Axis, InputBackend,
};
use smithay::input::{
    keyboard::{keysyms, ModifiersState},
    pointer::{AxisFrame, ButtonEvent, MotionEvent},
};
use smithay::utils::{Logical, Point, SERIAL_COUNTER};
use std::process::Command;

use crate::config::Action;
use crate::compositor::WebWMCompositor;

// Key modifier flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub super_key: bool,
}

impl Modifiers {
    pub fn from_smithay(mods: &ModifiersState) -> Self {
        Self {
            ctrl: mods.ctrl,
            alt: mods.alt,
            shift: mods.shift,
            super_key: mods.logo,
        }
    }

    pub fn matches(&self, binding_mods: &[String]) -> bool {
        let mut required = Self {
            ctrl: false,
            alt: false,
            shift: false,
            super_key: false,
        };

        for modifier in binding_mods {
            match modifier.to_lowercase().as_str() {
                "ctrl" | "control" => required.ctrl = true,
                "alt" | "mod1" => required.alt = true,
                "shift" => required.shift = true,
                "super" | "mod4" | "logo" => required.super_key = true,
                _ => {}
            }
        }

        self.ctrl == required.ctrl
            && self.alt == required.alt
            && self.shift == required.shift
            && self.super_key == required.super_key
    }
}

pub struct InputHandler {
    pub pointer_location: Point<f64, Logical>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            pointer_location: (0.0, 0.0).into(),
        }
    }

    pub fn process_input_event<B: InputBackend>(
        &mut self,
        event: InputEvent<B>,
        compositor: &mut WebWMCompositor,
    ) {
        match event {
            InputEvent::Keyboard { event } => {
                self.handle_keyboard(event, compositor);
            }
            InputEvent::PointerMotion { event } => {
                self.handle_pointer_motion(event, compositor);
            }
            InputEvent::PointerButton { event } => {
                self.handle_pointer_button(event, compositor);
            }
            InputEvent::PointerAxis { event } => {
                self.handle_pointer_axis(event, compositor);
            }
            _ => {}
        }
    }

    fn handle_keyboard<B: InputBackend>(
        &mut self,
        event: impl KeyboardKeyEvent<B>,
        compositor: &mut WebWMCompositor,
    ) {
        let keycode = event.key_code();
        let state = event.state();

        // Only process key press (not release)
        if state != KeyState::Pressed {
            return;
        }

        // Get keyboard to translate keycode to keysym
        if let Some(keyboard) = compositor.seat.get_keyboard() {
            // Get current modifiers
            let modifiers = keyboard.modifier_state();
            let mods = Modifiers::from_smithay(&modifiers);

            // Get the keysym for this keycode
            let keysym = keyboard
                .with_xkb_state(compositor, |state| {
                    state.key_get_one_sym(keycode)
                });

            println!("Key pressed: keycode={}, keysym={:?}, mods={:?}", 
                     keycode, keysym, mods);

            // Check if this matches any keybinding
            if self.check_keybindings(keysym, mods, compositor) {
                // Keybinding handled, don't forward to client
                return;
            }

            // Forward to focused window
            keyboard.input::<(), _>(
                compositor,
                keycode,
                state,
                SERIAL_COUNTER.next_serial(),
                0,
                |_, _, _| {
                    smithay::input::keyboard::FilterResult::Forward
                },
            );
        }
    }

    fn check_keybindings(
        &mut self,
        keysym: u32,
        mods: Modifiers,
        compositor: &mut WebWMCompositor,
    ) -> bool {
        // Convert keysym to key name
        let key_name = keysym_to_string(keysym);

        println!("Checking keybinding: {} with mods {:?}", key_name, mods);

        // Check each configured keybinding
        for binding in &compositor.config.keybindings.clone() {
            if binding.key.to_lowercase() == key_name.to_lowercase()
                && mods.matches(&binding.modifiers)
            {
                println!("Matched keybinding: {:?}", binding);
                self.execute_action(&binding.action, compositor);
                return true;
            }
        }

        false
    }

    fn execute_action(&mut self, action: &Action, compositor: &mut WebWMCompositor) {
        match action {
            Action::Spawn { command } => {
                println!("Spawning: {}", command);
                Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .spawn()
                    .map_err(|e| eprintln!("Failed to spawn '{}': {}", command, e))
                    .ok();
            }

            Action::Close => {
                println!("Closing focused window");
                if let Some(window) = compositor.workspace_manager.focused_window() {
                    if let Some(toplevel) = window.toplevel() {
                        toplevel.send_close();
                    }
                }
            }

            Action::Focus { direction } => {
                println!("Focusing: {}", direction);
                self.focus_direction(direction, compositor);
            }

            Action::Move { workspace } => {
                println!("Moving window to workspace: {}", workspace);
                if let Some(window) = compositor.workspace_manager.focused_window().cloned() {
                    compositor.workspace_manager.move_window_to_workspace(window, *workspace);
                    compositor.relayout();
                }
            }

            Action::SwitchWorkspace { workspace } => {
                println!("Switching to workspace: {}", workspace);
                compositor.workspace_manager.switch_to_workspace(*workspace);
                compositor.relayout();
            }

            Action::ToggleFloating => {
                println!("Toggling floating mode");
                // TODO: Implement floating toggle
            }

            Action::Custom { js } => {
                println!("Executing custom JS: {}", js);
                // TODO: Execute JavaScript callback
            }
        }
    }

    fn focus_direction(&mut self, direction: &str, compositor: &mut WebWMCompositor) {
        match direction {
            "up" | "left" => {
                compositor.workspace_manager.focus_prev_window();
            }
            "down" | "right" => {
                compositor.workspace_manager.focus_next_window();
            }
            _ => return,
        };

        // Update keyboard focus
        if let Some(window) = compositor.workspace_manager.focused_window() {
            if let Some(keyboard) = compositor.seat.get_keyboard() {
                if let Some(surface) = window.wl_surface() {
                    keyboard.set_focus(
                        compositor,
                        Some(surface.clone()),
                        SERIAL_COUNTER.next_serial(),
                    );
                    
                    let workspace = compositor.workspace_manager.active_workspace();
                    let window_idx = workspace.focused_window_idx.unwrap_or(0);
                    println!("Focused window {} in workspace {}", window_idx, workspace.id);
                }
            }
        }
    }

    fn handle_pointer_motion<B: InputBackend>(
        &mut self,
        event: impl PointerMotionEvent<B>,
        compositor: &mut WebWMCompositor,
    ) {
        let delta = event.delta();
        self.pointer_location.x += delta.0;
        self.pointer_location.y += delta.1;

        // Clamp to output bounds
        let output_size = (1920.0, 1080.0); // TODO: Get from actual output
        self.pointer_location.x = self.pointer_location.x.max(0.0).min(output_size.0);
        self.pointer_location.y = self.pointer_location.y.max(0.0).min(output_size.1);

        // Update pointer focus based on location
        let surface_under = compositor.space.element_under(self.pointer_location);
        
        if let Some(pointer) = compositor.seat.get_pointer() {
            if let Some((window, location)) = surface_under {
                if let Some(surface) = window.wl_surface() {
                    let surface_location = self.pointer_location - location.to_f64();
                    
                    pointer.motion(
                        compositor,
                        Some((surface.clone(), surface_location)),
                        &MotionEvent {
                            location: self.pointer_location,
                            serial: SERIAL_COUNTER.next_serial(),
                            time: 0,
                        },
                    );
                }
            } else {
                pointer.motion(
                    compositor,
                    None,
                    &MotionEvent {
                        location: self.pointer_location,
                        serial: SERIAL_COUNTER.next_serial(),
                        time: 0,
                    },
                );
            }
        }
    }

    fn handle_pointer_button<B: InputBackend>(
        &mut self,
        event: impl PointerButtonEvent<B>,
        compositor: &mut WebWMCompositor,
    ) {
        let button = event.button_code();
        let state = event.state();

        println!("Pointer button: {} {:?}", button, state);

        // On button press, focus the window under cursor
        if state == KeyState::Pressed {
            let surface_under = compositor.space.element_under(self.pointer_location);
            
            if let Some((window, _)) = surface_under {
                if let Some(keyboard) = compositor.seat.get_keyboard() {
                    if let Some(surface) = window.wl_surface() {
                        keyboard.set_focus(
                            compositor,
                            Some(surface.clone()),
                            SERIAL_COUNTER.next_serial(),
                        );
                        println!("Focused window under cursor");
                    }
                }
            }
        }

        // Forward button event to client
        if let Some(pointer) = compositor.seat.get_pointer() {
            pointer.button(
                compositor,
                &ButtonEvent {
                    button,
                    state: state.into(),
                    serial: SERIAL_COUNTER.next_serial(),
                    time: 0,
                },
            );
        }
    }

    fn handle_pointer_axis<B: InputBackend>(
        &mut self,
        event: impl PointerAxisEvent<B>,
        compositor: &mut WebWMCompositor,
    ) {
        if let Some(pointer) = compositor.seat.get_pointer() {
            let horizontal = event.amount(Axis::Horizontal)
                .unwrap_or(0.0);
            let vertical = event.amount(Axis::Vertical)
                .unwrap_or(0.0);

            let frame = AxisFrame::new(0)
                .value(Axis::Horizontal, horizontal)
                .value(Axis::Vertical, vertical);

            pointer.axis(compositor, frame);
        }
    }
}

// Convert keysym to human-readable string
fn keysym_to_string(keysym: u32) -> String {
    match keysym {
        keysyms::KEY_Return => "Return".to_string(),
        keysyms::KEY_Escape => "Escape".to_string(),
        keysyms::KEY_BackSpace => "BackSpace".to_string(),
        keysyms::KEY_Tab => "Tab".to_string(),
        keysyms::KEY_space => "space".to_string(),
        
        // Letters
        keysyms::KEY_a => "a".to_string(),
        keysyms::KEY_b => "b".to_string(),
        keysyms::KEY_c => "c".to_string(),
        keysyms::KEY_d => "d".to_string(),
        keysyms::KEY_e => "e".to_string(),
        keysyms::KEY_f => "f".to_string(),
        keysyms::KEY_g => "g".to_string(),
        keysyms::KEY_h => "h".to_string(),
        keysyms::KEY_i => "i".to_string(),
        keysyms::KEY_j => "j".to_string(),
        keysyms::KEY_k => "k".to_string(),
        keysyms::KEY_l => "l".to_string(),
        keysyms::KEY_m => "m".to_string(),
        keysyms::KEY_n => "n".to_string(),
        keysyms::KEY_o => "o".to_string(),
        keysyms::KEY_p => "p".to_string(),
        keysyms::KEY_q => "q".to_string(),
        keysyms::KEY_r => "r".to_string(),
        keysyms::KEY_s => "s".to_string(),
        keysyms::KEY_t => "t".to_string(),
        keysyms::KEY_u => "u".to_string(),
        keysyms::KEY_v => "v".to_string(),
        keysyms::KEY_w => "w".to_string(),
        keysyms::KEY_x => "x".to_string(),
        keysyms::KEY_y => "y".to_string(),
        keysyms::KEY_z => "z".to_string(),
        
        // Numbers
        keysyms::KEY_1 => "1".to_string(),
        keysyms::KEY_2 => "2".to_string(),
        keysyms::KEY_3 => "3".to_string(),
        keysyms::KEY_4 => "4".to_string(),
        keysyms::KEY_5 => "5".to_string(),
        keysyms::KEY_6 => "6".to_string(),
        keysyms::KEY_7 => "7".to_string(),
        keysyms::KEY_8 => "8".to_string(),
        keysyms::KEY_9 => "9".to_string(),
        keysyms::KEY_0 => "0".to_string(),
        
        // Function keys
        keysyms::KEY_F1 => "F1".to_string(),
        keysyms::KEY_F2 => "F2".to_string(),
        keysyms::KEY_F3 => "F3".to_string(),
        keysyms::KEY_F4 => "F4".to_string(),
        keysyms::KEY_F5 => "F5".to_string(),
        keysyms::KEY_F6 => "F6".to_string(),
        keysyms::KEY_F7 => "F7".to_string(),
        keysyms::KEY_F8 => "F8".to_string(),
        keysyms::KEY_F9 => "F9".to_string(),
        keysyms::KEY_F10 => "F10".to_string(),
        keysyms::KEY_F11 => "F11".to_string(),
        keysyms::KEY_F12 => "F12".to_string(),
        
        // Arrow keys
        keysyms::KEY_Left => "Left".to_string(),
        keysyms::KEY_Right => "Right".to_string(),
        keysyms::KEY_Up => "Up".to_string(),
        keysyms::KEY_Down => "Down".to_string(),
        
        _ => format!("Unknown({})", keysym),
    }
}
