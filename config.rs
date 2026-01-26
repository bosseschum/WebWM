use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub keybindings: Vec<Keybinding>,
    pub window_rules: Vec<WindowRule>,
    pub layout: LayoutConfig,
    pub theme: ThemeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybinding {
    pub key: String,
    pub modifiers: Vec<String>,
    pub action: Action,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Action {
    Spawn { command: String },
    Close,
    Focus { direction: String },
    Move { workspace: u32 },
    ToggleFloating,
    Custom { js: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowRule {
    pub app_id: String,
    pub workspace: Option<u32>,
    pub floating: Option<bool>,
    pub css_class: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub default_mode: String,
    pub gaps: u32,
    pub border_width: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub border_focused: String,
    pub border_normal: String,
    pub background: String,
}

pub fn load_config(config_dir: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = Path::new(config_dir);
    
    // Load config.json (generated from XML/CSS/JS processing)
    let config_file = config_path.join("config.json");
    
    if !config_file.exists() {
        // Return default config
        return Ok(default_config());
    }
    
    let config_str = fs::read_to_string(config_file)?;
    let config: Config = serde_json::from_str(&config_str)?;
    
    Ok(config)
}

fn default_config() -> Config {
    Config {
        keybindings: vec![
            Keybinding {
                key: "Return".to_string(),
                modifiers: vec!["Super".to_string()],
                action: Action::Spawn {
                    command: "alacritty".to_string(),
                },
            },
            Keybinding {
                key: "q".to_string(),
                modifiers: vec!["Super".to_string()],
                action: Action::Close,
            },
        ],
        window_rules: vec![],
        layout: LayoutConfig {
            default_mode: "tiling".to_string(),
            gaps: 10,
            border_width: 2,
        },
        theme: ThemeConfig {
            border_focused: "#4c7899".to_string(),
            border_normal: "#333333".to_string(),
            background: "#1e1e1e".to_string(),
        },
    }
}

// This will eventually parse XML/CSS/JS and convert to Config
pub fn parse_web_config(
    xml: &str,
    css: &str,
    js: &str,
) -> Result<Config, Box<dyn std::error::Error>> {
    // Placeholder: In full implementation, this would:
    // 1. Parse XML structure (desktop.xml)
    // 2. Parse CSS for theming (style.css)
    // 3. Evaluate JS for keybindings and rules (config.js)
    // 4. Combine into Config struct
    
    println!("Parsing web config...");
    println!("  XML length: {} bytes", xml.len());
    println!("  CSS length: {} bytes", css.len());
    println!("  JS length: {} bytes", js.len());
    
    // For now, return default
    Ok(default_config())
}
