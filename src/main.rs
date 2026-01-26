use std::sync::Arc;
use smithay::{
    backend::winit::{self, WinitEvent},
    reexports::{
        calloop::EventLoop,
        wayland_server::{Display, DisplayHandle},
    },
    wayland::{
        compositor::{CompositorClientState, CompositorState},
        shell::xdg::{XdgShellState, XdgToplevelSurfaceData},
    },
};

mod config;
mod state;

use state::WebWMState;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize event loop
    let mut event_loop = EventLoop::try_new()?;
    
    // Create Wayland display
    let display = Display::new()?;
    let dh = display.handle();
    
    // For prototype, use winit backend (creates a window on your existing desktop)
    let (backend, mut input) = winit::init()?;
    
    // Load web-based configuration
    let config = config::load_config("./config")?;
    println!("Loaded configuration:");
    println!("  Keybindings: {}", config.keybindings.len());
    println!("  Window rules: {}", config.window_rules.len());
    
    // Create compositor state
    let mut state = WebWMState::new(
        &mut event_loop,
        &dh,
        backend.clone(),
        config,
    );
    
    // Main event loop
    event_loop.run(
        None,
        &mut state,
        |state| {
            // Render frame
            state.render();
        },
    )?;
    
    Ok(())
}
