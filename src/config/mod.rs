mod css_parser;
mod js_runtime;
mod xml_parser;

pub use css_parser::*;
pub use js_runtime::*;
pub use xml_parser::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub desktop: Option<DesktopConfig>,
    pub stylesheet: Option<StyleSheet>,
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
    SwitchWorkspace { workspace: u32 },
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

pub fn load_config(
    config_dir: &str,
) -> Result<(Config, Option<JSRuntime>), Box<dyn std::error::Error>> {
    let config_path = Path::new(config_dir);

    println!("Loading configuration from: {}", config_dir);

    // Try to load web-based config (XML + CSS + JS)
    let xml_path = config_path.join("desktop.xml");
    let css_path = config_path.join("style.css");
    let js_path = config_path.join("config.js");

    if xml_path.exists() && css_path.exists() && js_path.exists() {
        println!("Found web-based configuration files");
        let (config, js_runtime) = load_web_config(&xml_path, &css_path, &js_path)?;
        return Ok((config, Some(js_runtime)));
    }

    // Fall back to JSON config
    let config_file = config_path.join("config.json");
    if config_file.exists() {
        println!("Loading JSON configuration");
        let config_str = fs::read_to_string(config_file)?;
        let config: Config = serde_json::from_str(&config_str)?;
        return Ok((config, None));
    }

    println!("No configuration found, using defaults");
    Ok((default_config(), None))
}

fn load_web_config(
    xml_path: &Path,
    css_path: &Path,
    js_path: &Path,
) -> Result<(Config, JSRuntime), Box<dyn std::error::Error>> {
    println!("Parsing web-based configuration...");

    // Load and parse XML
    let xml_content = fs::read_to_string(xml_path)?;
    let desktop_config = xml_parser::parse_desktop_xml(&xml_content)?;
    println!(
        "  ✓ Parsed desktop.xml: {} bars, {} workspaces, {} window rules",
        desktop_config.bars.len(),
        desktop_config.workspaces.len(),
        desktop_config.window_rules.len()
    );

    // Load and parse CSS
    let css_content = fs::read_to_string(css_path)?;
    let stylesheet = css_parser::parse_css(&css_content)?;
    println!(
        "  ✓ Parsed style.css: {} rules, {} variables",
        stylesheet.rules.len(),
        stylesheet.variables.len()
    );

    // Load and execute JavaScript
    let js_content = fs::read_to_string(js_path)?;

    // Create and initialize JavaScript runtime
    let js_runtime = JSRuntime::new()?;
    js_runtime.init_api()?;
    js_runtime.evaluate(&js_content)?;

    let js_keybindings = js_runtime.get_keybindings();
    println!(
        "  ✓ Executed config.js: {} keybindings registered",
        js_keybindings.len()
    );

    // Convert to unified Config structure
    let mut config = Config {
        keybindings: vec![],
        window_rules: vec![],
        layout: LayoutConfig {
            default_mode: desktop_config.layout.split_mode.clone(),
            gaps: desktop_config.layout.gaps_inner,
            border_width: 2, // Would extract from CSS
        },
        theme: extract_theme_from_css(&stylesheet),
        desktop: Some(desktop_config.clone()),
        stylesheet: Some(stylesheet),
    };

    // Convert JS keybindings to Config keybindings
    for js_kb in js_keybindings {
        config.keybindings.push(Keybinding {
            key: js_kb.key.clone(),
            modifiers: js_kb.modifiers.clone(),
            action: Action::Custom { js: js_kb.callback },
        });
    }

    // Convert XML window rules to Config window rules
    for xml_rule in &desktop_config.window_rules {
        if let Some(app_id) = &xml_rule.app_id {
            config.window_rules.push(WindowRule {
                app_id: app_id.clone(),
                workspace: xml_rule.workspace,
                floating: xml_rule.floating,
                css_class: xml_rule.class.clone(),
            });
        }
    }

    println!("Configuration loaded successfully!");
    Ok((config, js_runtime))
}

fn extract_theme_from_css(stylesheet: &StyleSheet) -> ThemeConfig {
    // Extract theme colors from CSS variables
    let border_focused = stylesheet
        .variables
        .get("--border-focus")
        .cloned()
        .unwrap_or("#4c7899".to_string());

    let border_normal = stylesheet
        .variables
        .get("--border-normal")
        .cloned()
        .unwrap_or("#333333".to_string());

    let background = stylesheet
        .variables
        .get("--bg-primary")
        .cloned()
        .unwrap_or("#1e1e1e".to_string());

    ThemeConfig {
        border_focused,
        border_normal,
        background,
    }
}

fn default_config() -> Config {
    Config {
        keybindings: vec![
            Keybinding {
                key: "Return".to_string(),
                modifiers: vec!["Super".to_string()],
                action: Action::Spawn {
                    command: "kitty".to_string(),
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
        desktop: None,
        stylesheet: None,
    }
}

pub fn save_config_json(config: &Config, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(config)?;
    fs::write(path, json)?;
    Ok(())
}
