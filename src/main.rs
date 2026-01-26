mod config;
mod compositor;
mod backend;

use config::Config;
use compositor::{WebWMCompositor, ClientState};
use backend::WebWMBackend;

use smithay::reexports::{
    wayland_server::{Display, DisplayHandle},
    calloop::timer::{Timer, TimeoutAction},
};
use std::env;
use std::time::Duration;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    println!("===========================================");
    println!("  WebWM - Web-Tech Wayland Compositor");
    println!("  Version 0.2.0");
    println!("===========================================\n");
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    
    // Check if running in config-only mode
    if args.len() > 1 && args[1] == "config" {
        return run_config_mode(&args);
    }
    
    // Normal compositor mode
    run_compositor()?;
    
    Ok(())
}

fn run_config_mode(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running in configuration validation mode\n");
    
    let config_dir = args.get(2)
        .map(|s| s.as_str())
        .unwrap_or("./config");
    
    println!("Step 1: Loading Configuration");
    println!("-------------------------------------------");
    let config = config::load_config(config_dir)?;
    
    println!("\nConfiguration Summary:");
    println!("  • Keybindings: {}", config.keybindings.len());
    println!("  • Window rules: {}", config.window_rules.len());
    println!("  • Layout mode: {}", config.layout.default_mode);
    println!("  • Gaps: {}px", config.layout.gaps);
    println!("  • Border width: {}px", config.layout.border_width);
    
    if let Some(ref desktop) = config.desktop {
        println!("\nDesktop Configuration:");
        println!("  • Bars: {}", desktop.bars.len());
        println!("  • Workspaces: {}", desktop.workspaces.len());
        println!("  • Animations: {}", if desktop.animations.enabled { "enabled" } else { "disabled" });
    }
    
    if let Some(ref stylesheet) = config.stylesheet {
        println!("\nStylesheet:");
        println!("  • CSS Rules: {}", stylesheet.rules.len());
        println!("  • CSS Variables: {}", stylesheet.variables.len());
        
        println!("\nTheme Colors:");
        println!("  • Background: {}", config.theme.background);
        println!("  • Border (focused): {}", config.theme.border_focused);
        println!("  • Border (normal): {}", config.theme.border_normal);
    }
    
    println!("\n-------------------------------------------");
    println!("Step 2: Display Keybindings");
    println!("-------------------------------------------");
    for (i, kb) in config.keybindings.iter().enumerate() {
        let modifiers = if kb.modifiers.is_empty() {
            String::new()
        } else {
            format!("{}+", kb.modifiers.join("+"))
        };
        
        let action = match &kb.action {
            config::Action::Spawn { command } => format!("spawn '{}'", command),
            config::Action::Close => "close window".to_string(),
            config::Action::Focus { direction } => format!("focus {}", direction),
            config::Action::Move { workspace } => format!("move to workspace {}", workspace),
            config::Action::ToggleFloating => "toggle floating".to_string(),
            config::Action::Custom { js } => format!("execute JS: {}", js),
        };
        
        println!("  {}. {}{} → {}", i + 1, modifiers, kb.key, action);
    }
    
    if args.iter().any(|arg| arg == "--save-json") {
        let json_path = format!("{}/parsed_config.json", config_dir);
        config::save_config_json(&config, &json_path)?;
        println!("\n✓ Saved parsed configuration to: {}", json_path);
    }
    
    println!("\n===========================================");
    println!("Configuration is valid!");
    println!("===========================================\n");
    
    Ok(())
}

fn run_compositor() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting compositor...\n");
    
    // Get config directory
    let config_dir = env::args()
        .nth(1)
        .unwrap_or_else(|| "./config".to_string());
    
    // Load configuration
    println!("Loading configuration from: {}", config_dir);
    let config = config::load_config(&config_dir)?;
    
    println!("✓ Configuration loaded");
    println!("  • {} keybindings", config.keybindings.len());
    println!("  • {} window rules", config.window_rules.len());
    println!("  • Layout: {}", config.layout.default_mode);
    println!("  • Gaps: {}px", config.layout.gaps);
    println!();
    
    // Create event loop first
    let event_loop = smithay::reexports::calloop::EventLoop::try_new()?;
    
    // Create Wayland display
    let mut display = Display::<WebWMCompositor>::new()?;
    
    // Create compositor
    println!("Initializing compositor...");
    let mut compositor = WebWMCompositor::new(
        &mut display,
        event_loop.handle(),
        config,
    );
    println!("✓ Compositor initialized");
    
    // Initialize backend with event handler
    println!("Initializing backend...");
    let mut backend = WebWMBackend::new(&event_loop, |event, compositor| {
        let input_handler = &mut InputHandler::new(); // Temporary, will fix
        compositor.handle_winit_event(event, input_handler);
    })?;
    println!("✓ Backend initialized (winit)");
    
    // Add output to space
    compositor.space.map_output(&backend.output, (0, 0));
    
    // Get the Wayland socket name
    let socket = smithay::wayland::socket::ListeningSocketSource::new_auto()?;
    let socket_name = socket.socket_name().to_string_lossy().into_owned();
    
    println!("\n===========================================");
    println!("  WebWM is running!");
    println!("===========================================");
    println!("\nWayland socket: {}", socket_name);
    println!("\nTo connect a client, run:");
    println!("  WAYLAND_DISPLAY={} alacritty", socket_name);
    println!("  WAYLAND_DISPLAY={} weston-terminal", socket_name);
    println!("\nKeybindings active:");
    for kb in compositor.config.keybindings.iter().take(5) {
        let mods = if kb.modifiers.is_empty() {
            String::new()
        } else {
            format!("{}+", kb.modifiers.join("+"))
        };
        println!("  {}{}", mods, kb.key);
    }
    println!("\nPress Ctrl+C to exit");
    println!("===========================================\n");
    
    // Insert socket into event loop
    event_loop
        .handle()
        .insert_source(socket, {
            let dh = display.handle();
            move |client_stream, _, _| {
                if let Err(e) = dh.insert_client(
                    client_stream,
                    Arc::new(ClientState {
                        compositor_state: Default::default(),
                    }),
                ) {
                    eprintln!("Error accepting client: {}", e);
                }
            }
        })?;
    
    // Add periodic rendering
    let timer = Timer::from_duration(Duration::from_millis(16)); // ~60 FPS
    event_loop
        .handle()
        .insert_source(timer, |_, _, _| {
            TimeoutAction::ToDuration(Duration::from_millis(16))
        })?;
    
    // Run event loop
    event_loop.run(
        Duration::from_millis(16),
        &mut compositor,
        |compositor| {
            // Dispatch Wayland events
            display.dispatch_clients(compositor).unwrap();
            display.flush_clients().unwrap();
            
            // Render frame
            if let Err(e) = backend.render(compositor) {
                eprintln!("Render error: {:?}", e);
            }
        },
    )?;
    
    Ok(())
}

impl WebWMCompositor {
    fn render_frame(&mut self) {
        // This will be called by the backend
        // The actual rendering is handled in backend.rs
    }
}
