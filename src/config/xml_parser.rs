use roxmltree::{Document, Node};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DesktopConfig {
    pub bars: Vec<BarConfig>,
    pub workspaces: Vec<WorkspaceConfig>,
    pub window_rules: Vec<WindowRuleConfig>,
    pub layout: LayoutSettings,
    pub animations: AnimationSettings,
}

#[derive(Debug, Clone)]
pub struct BarConfig {
    pub id: String,
    pub position: Position,
    pub height: u32,
    pub class: String,
    pub widgets: Vec<Widget>,
}

#[derive(Debug, Clone)]
pub enum Position {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub enum Widget {
    Workspaces { display: String },
    WindowTitle { max_width: Option<u32> },
    SystemTray,
    Clock { format: String },
    Spacer { flex: u32 },
}

#[derive(Debug, Clone)]
pub struct WorkspaceConfig {
    pub id: u32,
    pub name: String,
    pub layout: String,
    pub split_ratio: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct WindowRuleConfig {
    pub app_id: Option<String>,
    pub title: Option<String>,
    pub workspace: Option<u32>,
    pub floating: Option<bool>,
    pub sticky: Option<bool>,
    pub class: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LayoutSettings {
    pub gaps_outer: u32,
    pub gaps_inner: u32,
    pub split_mode: String,
    pub master_ratio: f32,
    pub floating_default_width: u32,
    pub floating_default_height: u32,
    pub center_new_windows: bool,
}

#[derive(Debug, Clone)]
pub struct AnimationSettings {
    pub enabled: bool,
    pub window_open_duration: u32,
    pub window_close_duration: u32,
    pub workspace_switch_duration: u32,
}

pub fn parse_desktop_xml(xml_content: &str) -> Result<DesktopConfig, String> {
    let doc = Document::parse(xml_content)
        .map_err(|e| format!("XML parse error: {}", e))?;
    
    let root = doc.root_element();
    
    if root.tag_name().name() != "desktop" {
        return Err("Root element must be <desktop>".to_string());
    }
    
    let mut config = DesktopConfig {
        bars: Vec::new(),
        workspaces: Vec::new(),
        window_rules: Vec::new(),
        layout: LayoutSettings::default(),
        animations: AnimationSettings::default(),
    };
    
    for child in root.children().filter(|n| n.is_element()) {
        match child.tag_name().name() {
            "bar" => {
                if let Some(bar) = parse_bar(&child) {
                    config.bars.push(bar);
                }
            }
            "workspaces" => {
                config.workspaces = parse_workspaces(&child);
            }
            "window-rules" => {
                config.window_rules = parse_window_rules(&child);
            }
            "layout" => {
                config.layout = parse_layout(&child);
            }
            "animations" => {
                config.animations = parse_animations(&child);
            }
            _ => {}
        }
    }
    
    Ok(config)
}

fn parse_bar(node: &Node) -> Option<BarConfig> {
    let id = node.attribute("id")?.to_string();
    let position = match node.attribute("position")? {
        "top" => Position::Top,
        "bottom" => Position::Bottom,
        "left" => Position::Left,
        "right" => Position::Right,
        _ => Position::Top,
    };
    let height = node.attribute("height")
        .and_then(|h| h.parse().ok())
        .unwrap_or(30);
    let class = node.attribute("class")
        .unwrap_or("bar")
        .to_string();
    
    let mut widgets = Vec::new();
    for child in node.children().filter(|n| n.is_element()) {
        if let Some(widget) = parse_widget(&child) {
            widgets.push(widget);
        }
    }
    
    Some(BarConfig {
        id,
        position,
        height,
        class,
        widgets,
    })
}

fn parse_widget(node: &Node) -> Option<Widget> {
    match node.tag_name().name() {
        "workspaces" => Some(Widget::Workspaces {
            display: node.attribute("display")
                .unwrap_or("icons")
                .to_string(),
        }),
        "window-title" => Some(Widget::WindowTitle {
            max_width: node.attribute("max-width")
                .and_then(|w| w.parse().ok()),
        }),
        "system-tray" => Some(Widget::SystemTray),
        "clock" => Some(Widget::Clock {
            format: node.attribute("format")
                .unwrap_or("%H:%M")
                .to_string(),
        }),
        "spacer" => Some(Widget::Spacer {
            flex: node.attribute("flex")
                .and_then(|f| f.parse().ok())
                .unwrap_or(1),
        }),
        _ => None,
    }
}

fn parse_workspaces(node: &Node) -> Vec<WorkspaceConfig> {
    node.children()
        .filter(|n| n.is_element() && n.tag_name().name() == "workspace")
        .filter_map(|ws| {
            let id = ws.attribute("id")?.parse().ok()?;
            let name = ws.attribute("name")
                .unwrap_or(&format!("{}", id))
                .to_string();
            let layout = ws.attribute("layout")
                .unwrap_or("tiling")
                .to_string();
            
            let split_ratio = ws.children()
                .find(|n| n.tag_name().name() == "split-ratio")
                .and_then(|n| n.text())
                .and_then(|t| t.parse().ok());
            
            Some(WorkspaceConfig {
                id,
                name,
                layout,
                split_ratio,
            })
        })
        .collect()
}

fn parse_window_rules(node: &Node) -> Vec<WindowRuleConfig> {
    node.children()
        .filter(|n| n.is_element() && n.tag_name().name() == "rule")
        .map(|rule| {
            WindowRuleConfig {
                app_id: rule.attribute("app-id").map(|s| s.to_string()),
                title: rule.attribute("title").map(|s| s.to_string()),
                workspace: rule.attribute("workspace")
                    .and_then(|w| w.parse().ok()),
                floating: rule.attribute("floating")
                    .and_then(|f| f.parse().ok()),
                sticky: rule.attribute("sticky")
                    .and_then(|s| s.parse().ok()),
                class: rule.attribute("class").map(|s| s.to_string()),
            }
        })
        .collect()
}

fn parse_layout(node: &Node) -> LayoutSettings {
    let mut settings = LayoutSettings::default();
    
    for child in node.children().filter(|n| n.is_element()) {
        match child.tag_name().name() {
            "tiling" => {
                if let Some(gaps) = child.children()
                    .find(|n| n.tag_name().name() == "gaps") {
                    settings.gaps_outer = gaps.attribute("outer")
                        .and_then(|g| g.parse().ok())
                        .unwrap_or(10);
                    settings.gaps_inner = gaps.attribute("inner")
                        .and_then(|g| g.parse().ok())
                        .unwrap_or(10);
                }
                
                if let Some(ratio) = child.children()
                    .find(|n| n.tag_name().name() == "master-ratio")
                    .and_then(|n| n.text())
                    .and_then(|t| t.parse().ok()) {
                    settings.master_ratio = ratio;
                }
            }
            "floating" => {
                if let Some(size) = child.children()
                    .find(|n| n.tag_name().name() == "default-size") {
                    settings.floating_default_width = size.attribute("width")
                        .and_then(|w| w.parse().ok())
                        .unwrap_or(800);
                    settings.floating_default_height = size.attribute("height")
                        .and_then(|h| h.parse().ok())
                        .unwrap_or(600);
                }
                
                if let Some(center) = child.children()
                    .find(|n| n.tag_name().name() == "center-new-windows")
                    .and_then(|n| n.text())
                    .and_then(|t| t.parse().ok()) {
                    settings.center_new_windows = center;
                }
            }
            _ => {}
        }
    }
    
    settings
}

fn parse_animations(node: &Node) -> AnimationSettings {
    AnimationSettings {
        enabled: node.attribute("enabled")
            .and_then(|e| e.parse().ok())
            .unwrap_or(true),
        window_open_duration: parse_duration(
            node.children()
                .find(|n| n.tag_name().name() == "window-open")
                .and_then(|n| n.attribute("duration"))
        ),
        window_close_duration: parse_duration(
            node.children()
                .find(|n| n.tag_name().name() == "window-close")
                .and_then(|n| n.attribute("duration"))
        ),
        workspace_switch_duration: parse_duration(
            node.children()
                .find(|n| n.tag_name().name() == "workspace-switch")
                .and_then(|n| n.attribute("duration"))
        ),
    }
}

fn parse_duration(duration_str: Option<&str>) -> u32 {
    duration_str
        .and_then(|s| s.trim_end_matches("ms").parse().ok())
        .unwrap_or(200)
}

impl Default for LayoutSettings {
    fn default() -> Self {
        Self {
            gaps_outer: 10,
            gaps_inner: 10,
            split_mode: "auto".to_string(),
            master_ratio: 0.55,
            floating_default_width: 800,
            floating_default_height: 600,
            center_new_windows: true,
        }
    }
}

impl Default for AnimationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            window_open_duration: 200,
            window_close_duration: 150,
            workspace_switch_duration: 250,
        }
    }
}
