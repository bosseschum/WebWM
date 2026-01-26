mod config;
mod state;

use config::*;
use state::WebWMState;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("===========================================");
    println!("  WebWM - Web-Tech Wayland Compositor");
    println!("  Version 0.1.0 (Prototype)");
    println!("===========================================\n");
    
    // Get config directory from args or use default
    let config_dir = env::args()
        .nth(1)
        .unwrap_or_else(|| "./config".to_string());
    
    // Load configuration
    println!("Step 1: Loading Configuration");
    println!("-------------------------------------------");
    let config = config::load_config(&config_dir)?;
    
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
        
        // Show some extracted theme values
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
            Action::Spawn { command } => format!("spawn '{}'", command),
            Action::Close => "close window".to_string(),
            Action::Focus { direction } => format!("focus {}", direction),
            Action::Move { workspace } => format!("move to workspace {}", workspace),
            Action::ToggleFloating => "toggle floating".to_string(),
            Action::Custom { js } => format!("execute JS: {}", js),
        };
        
        println!("  {}. {}{} → {}", i + 1, modifiers, kb.key, action);
    }
    
    if config.keybindings.is_empty() {
        println!("  (No keybindings configured)");
    }
    
    println!("\n-------------------------------------------");
    println!("Step 3: Display Window Rules");
    println!("-------------------------------------------");
    for (i, rule) in config.window_rules.iter().enumerate() {
        print!("  {}. app_id='{}' →", i + 1, rule.app_id);
        
        if let Some(ws) = rule.workspace {
            print!(" workspace {}", ws);
        }
        if let Some(floating) = rule.floating {
            print!(" floating={}", floating);
        }
        if let Some(ref class) = rule.css_class {
            print!(" class='{}'", class);
        }
        println!();
    }
    
    if config.window_rules.is_empty() {
        println!("  (No window rules configured)");
    }
    
    // Show CSS selectors if available
    if let Some(ref stylesheet) = config.stylesheet {
        println!("\n-------------------------------------------");
        println!("Step 4: CSS Selectors");
        println!("-------------------------------------------");
        for (i, rule) in stylesheet.rules.iter().take(10).enumerate() {
            println!("  {}. {} ({} properties)", 
                     i + 1, 
                     rule.selector, 
                     rule.properties.len());
        }
        if stylesheet.rules.len() > 10 {
            println!("  ... and {} more rules", stylesheet.rules.len() - 10);
        }
    }
    
    println!("\n===========================================");
    println!("Configuration loaded successfully!");
    println!("===========================================\n");
    
    // For now, just validate config and exit
    // In a full implementation, we would:
    // 1. Initialize Wayland compositor
    // 2. Set up event loop
    // 3. Apply styles to windows
    // 4. Handle input events
    
    println!("Note: This is a prototype. Full compositor");
    println!("implementation coming in next iteration.");
    
    // Optionally save the parsed config as JSON for inspection
    if env::args().any(|arg| arg == "--save-json") {
        let json_path = format!("{}/parsed_config.json", config_dir);
        config::save_config_json(&config, &json_path)?;
        println!("\n✓ Saved parsed configuration to: {}", json_path);
    }
    
    Ok(())
}
