use smithay::utils::{Physical, Rectangle};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::compositor::workspace::WorkspaceManager;
use crate::config::StyleSheet;
use crate::config::{BarConfig, Position, Widget};

#[derive(Debug, Clone)]
pub struct Bar {
    pub config: BarConfig,
    pub geometry: Rectangle<i32, Physical>,
}

impl Bar {
    pub fn new(config: BarConfig, output_width: i32) -> Self {
        let height = config.height as i32;
        let width = output_width;

        let geometry = match config.position {
            Position::Top => Rectangle::from_loc_and_size((0, 0), (width, height)),
            Position::Bottom => Rectangle::from_loc_and_size((0, 1080 - height), (width, height)),
            Position::Left => Rectangle::from_loc_and_size((0, 0), (height, 1080)),
            Position::Right => Rectangle::from_loc_and_size((width - height, 0), (height, 1080)),
        };

        Self { config, geometry }
    }

    pub fn height(&self) -> i32 {
        match self.config.position {
            Position::Top | Position::Bottom => self.config.height as i32,
            Position::Left | Position::Right => self.config.height as i32,
        }
    }

    pub fn is_horizontal(&self) -> bool {
        matches!(self.config.position, Position::Top | Position::Bottom)
    }
}

pub struct BarRenderer {
    pub bars: Vec<Bar>,
}

impl BarRenderer {
    pub fn new(bar_configs: Vec<BarConfig>, output_width: i32) -> Self {
        let bars = bar_configs
            .into_iter()
            .map(|config| Bar::new(config, output_width))
            .collect();

        Self { bars }
    }

    pub fn render_bars(
        &self,
        workspace_manager: &WorkspaceManager,
        focused_window_title: Option<String>,
        stylesheet: Option<&StyleSheet>,
    ) -> Vec<BarElement> {
        let mut elements = Vec::new();

        for bar in &self.bars {
            elements.extend(self.render_bar(
                bar,
                workspace_manager,
                focused_window_title.clone(),
                stylesheet,
            ));
        }

        elements
    }

    fn render_bar(
        &self,
        bar: &Bar,
        workspace_manager: &WorkspaceManager,
        focused_window_title: Option<String>,
        stylesheet: Option<&StyleSheet>,
    ) -> Vec<BarElement> {
        let mut elements = Vec::new();
        let mut x_offset = 16; // Left padding

        // Get colors from stylesheet or use defaults
        let (bg_color, text_color) = if let Some(ss) = stylesheet {
            let bg = ss
                .get_color(&bar.config.class, "background")
                .map(|c| c.to_rgba_f32())
                .unwrap_or([0.11, 0.11, 0.18, 0.95]);
            let fg = ss
                .get_color(&bar.config.class, "color")
                .map(|c| c.to_rgba_f32())
                .unwrap_or([0.8, 0.83, 0.96, 1.0]);
            (bg, fg)
        } else {
            ([0.11, 0.11, 0.18, 0.95], [0.8, 0.83, 0.96, 1.0])
        };

        // Background
        elements.push(BarElement::Rectangle {
            geometry: bar.geometry,
            color: bg_color,
        });

        // Render widgets
        for widget in &bar.config.widgets {
            let widget_elements = self.render_widget(
                widget,
                workspace_manager,
                focused_window_title.as_ref(),
                &mut x_offset,
                bar.geometry.loc.y + 5,
                text_color,
                stylesheet,
            );
            elements.extend(widget_elements);
        }

        elements
    }

    fn render_widget(
        &self,
        widget: &Widget,
        workspace_manager: &WorkspaceManager,
        focused_window_title: Option<&String>,
        x_offset: &mut i32,
        y: i32,
        text_color: [f32; 4],
        stylesheet: Option<&StyleSheet>,
    ) -> Vec<BarElement> {
        match widget {
            Widget::Workspaces { display: _ } => {
                self.render_workspaces(workspace_manager, x_offset, y, text_color, stylesheet)
            }
            Widget::WindowTitle { max_width } => {
                self.render_window_title(focused_window_title, x_offset, y, *max_width, text_color)
            }
            Widget::Clock { format } => self.render_clock(format, x_offset, y, text_color),
            Widget::SystemTray => {
                // TODO: Implement system tray
                Vec::new()
            }
            Widget::Spacer { flex } => {
                *x_offset += 100 * (*flex as i32); // Simple spacer
                Vec::new()
            }
        }
    }

    fn render_workspaces(
        &self,
        workspace_manager: &WorkspaceManager,
        x_offset: &mut i32,
        y: i32,
        text_color: [f32; 4],
        stylesheet: Option<&StyleSheet>,
    ) -> Vec<BarElement> {
        let mut elements = Vec::new();
        let active_id = workspace_manager.active_workspace_id();

        for workspace in workspace_manager.all_workspaces() {
            let is_active = workspace.id == active_id;
            let has_windows = !workspace.is_empty();

            // Get colors from stylesheet
            let (bg_color, fg_color) = if let Some(ss) = stylesheet {
                if is_active {
                    let bg = ss
                        .get_color("workspace.active", "background")
                        .map(|c| c.to_rgba_f32())
                        .unwrap_or([0.54, 0.71, 0.98, 1.0]); // Blue
                    let fg = ss
                        .get_color("workspace.active", "color")
                        .map(|c| c.to_rgba_f32())
                        .unwrap_or([0.11, 0.11, 0.18, 1.0]); // Dark
                    (bg, fg)
                } else if has_windows {
                    let bg = ss
                        .get_color("workspace", "background")
                        .map(|c| c.to_rgba_f32())
                        .unwrap_or([0.19, 0.20, 0.27, 1.0]); // Gray
                    (bg, text_color)
                } else {
                    ([0.0, 0.0, 0.0, 0.0], text_color) // Transparent
                }
            } else {
                if is_active {
                    ([0.54, 0.71, 0.98, 1.0], [0.11, 0.11, 0.18, 1.0])
                } else if has_windows {
                    ([0.19, 0.20, 0.27, 1.0], text_color)
                } else {
                    ([0.0, 0.0, 0.0, 0.0], text_color)
                }
            };

            let width = 40;
            let height = 20;

            // Background box
            if bg_color[3] > 0.0 {
                elements.push(BarElement::Rectangle {
                    geometry: Rectangle::from_loc_and_size((*x_offset, y), (width, height)),
                    color: bg_color,
                });
            }

            // Workspace number/name
            let text = if workspace.name.len() <= 3 {
                workspace.name.clone()
            } else {
                workspace.id.to_string()
            };

            elements.push(BarElement::Text {
                position: (*x_offset + 12, y + 3),
                text,
                color: fg_color,
                size: 13,
            });

            // Window count indicator (small dot if has windows)
            if has_windows && !is_active {
                elements.push(BarElement::Circle {
                    center: (*x_offset + width - 8, y + height - 8),
                    radius: 3,
                    color: text_color,
                });
            }

            *x_offset += width + 8;
        }

        elements
    }

    fn render_window_title(
        &self,
        title: Option<&String>,
        x_offset: &mut i32,
        y: i32,
        max_width: Option<u32>,
        text_color: [f32; 4],
    ) -> Vec<BarElement> {
        if let Some(title) = title {
            let display_title = if let Some(max) = max_width {
                if title.len() > max as usize {
                    format!("{}...", &title[..max as usize - 3])
                } else {
                    title.clone()
                }
            } else {
                title.clone()
            };

            let text_width = display_title.len() * 8; // Approximate
            let result = vec![BarElement::Text {
                position: (*x_offset, y + 3),
                text: display_title,
                color: text_color,
                size: 13,
            }];

            *x_offset += text_width as i32 + 16;
            result
        } else {
            Vec::new()
        }
    }

    fn render_clock(
        &self,
        format: &str,
        x_offset: &mut i32,
        y: i32,
        text_color: [f32; 4],
    ) -> Vec<BarElement> {
        let time_str = format_time(format);

        let text_width = time_str.len() * 8;
        let result = vec![BarElement::Text {
            position: (*x_offset, y + 3),
            text: time_str,
            color: text_color,
            size: 13,
        }];

        *x_offset += text_width as i32 + 16;
        result
    }
}

#[derive(Debug, Clone)]
pub enum BarElement {
    Rectangle {
        geometry: Rectangle<i32, Physical>,
        color: [f32; 4],
    },
    Circle {
        center: (i32, i32),
        radius: i32,
        color: [f32; 4],
    },
    Text {
        position: (i32, i32),
        text: String,
        color: [f32; 4],
        size: u32,
    },
}

fn format_time(format: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Simple time formatting (not full strftime support)
    let total_seconds = now;
    let hours = (total_seconds / 3600) % 24;
    let minutes = (total_seconds / 60) % 60;
    let seconds = total_seconds % 60;

    // Get date components (approximate)
    let days_since_epoch = total_seconds / 86400;
    let year = 1970 + (days_since_epoch / 365);
    let day_of_year = days_since_epoch % 365;
    let month = (day_of_year / 30) + 1;
    let day = (day_of_year % 30) + 1;

    let weekdays = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let weekday = weekdays[(days_since_epoch % 7) as usize];

    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let month_name = months[(month.saturating_sub(1) % 12) as usize];

    // Replace format specifiers
    format
        .replace("%H", &format!("{:02}", hours))
        .replace("%M", &format!("{:02}", minutes))
        .replace("%S", &format!("{:02}", seconds))
        .replace("%d", &format!("{:02}", day))
        .replace("%m", &format!("{:02}", month))
        .replace("%Y", &year.to_string())
        .replace("%y", &format!("{:02}", year % 100))
        .replace("%a", weekday)
        .replace("%b", month_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_formatting() {
        let time = format_time("%H:%M");
        assert!(time.contains(":"));

        let time = format_time("%H:%M:%S");
        assert_eq!(time.matches(":").count(), 2);
    }

    #[test]
    fn test_bar_geometry() {
        let config = BarConfig {
            id: "test".to_string(),
            position: Position::Top,
            height: 30,
            class: "bar".to_string(),
            widgets: vec![],
        };

        let bar = Bar::new(config, 1920);
        assert_eq!(bar.geometry.loc.y, 0);
        assert_eq!(bar.geometry.size.h, 30);
    }
}
