use cssparser::Color as CssColor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleSheet {
    pub rules: Vec<StyleRule>,
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleRule {
    pub selector: String,
    pub properties: HashMap<String, StyleValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StyleValue {
    Color(Color),
    Length(f32, LengthUnit),
    String(String),
    Number(f32),
    Keyword(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LengthUnit {
    Px,
    Percent,
    Em,
    Rem,
}

pub fn parse_css(css_content: &str) -> Result<StyleSheet, String> {
    let mut stylesheet = StyleSheet {
        rules: Vec::new(),
        variables: HashMap::new(),
    };

    // Simple CSS parser - in production, use a full CSS parser
    // For now, we'll do basic parsing

    let lines: Vec<&str> = css_content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with("/*") {
            i += 1;
            continue;
        }

        // Parse CSS variables
        if line.starts_with(":root") {
            i += 1;
            while i < lines.len() {
                let var_line = lines[i].trim();
                if var_line == "}" {
                    break;
                }
                if let Some((key, value)) = parse_css_variable(var_line) {
                    stylesheet.variables.insert(key, value);
                }
                i += 1;
            }
            i += 1;
            continue;
        }

        // Parse regular CSS rules
        if line.contains("{") {
            let selector = line.trim_end_matches('{').trim().to_string();
            i += 1;

            let mut properties = HashMap::new();
            while i < lines.len() {
                let prop_line = lines[i].trim();
                if prop_line == "}" {
                    break;
                }
                if let Some((prop, value)) = parse_css_property(prop_line, &stylesheet.variables) {
                    properties.insert(prop, value);
                }
                i += 1;
            }

            stylesheet.rules.push(StyleRule {
                selector,
                properties,
            });
        }

        i += 1;
    }

    Ok(stylesheet)
}

fn parse_css_variable(line: &str) -> Option<(String, String)> {
    if !line.contains(":") {
        return None;
    }

    let parts: Vec<&str> = line.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }

    let key = parts[0].trim().to_string();
    let value = parts[1].trim().trim_end_matches(';').trim().to_string();

    Some((key, value))
}

fn parse_css_property(
    line: &str,
    variables: &HashMap<String, String>,
) -> Option<(String, StyleValue)> {
    if !line.contains(":") {
        return None;
    }

    let parts: Vec<&str> = line.splitn(2, ':').collect();
    if parts.len() != 2 {
        return None;
    }

    let property = parts[0].trim().to_string();
    let mut value_str = parts[1].trim().trim_end_matches(';').trim();

    // Resolve CSS variables
    let resolved_value;
    if value_str.starts_with("var(") && value_str.ends_with(")") {
        let var_name = value_str
            .trim_start_matches("var(")
            .trim_end_matches(")")
            .trim();
        resolved_value = variables
            .get(var_name)
            .map(|v| v.as_str())
            .unwrap_or(value_str);
        value_str = resolved_value;
    }

    let value = parse_css_value(value_str)?;

    Some((property, value))
}

fn parse_css_value(value_str: &str) -> Option<StyleValue> {
    let value = value_str.trim();

    // Try to parse as color
    if value.starts_with("#") || value.starts_with("rgb") || value.starts_with("rgba") {
        if let Some(color) = parse_color(value) {
            return Some(StyleValue::Color(color));
        }
    }

    // Try to parse as length
    if value.ends_with("px") {
        if let Ok(num) = value.trim_end_matches("px").parse::<f32>() {
            return Some(StyleValue::Length(num, LengthUnit::Px));
        }
    }

    if value.ends_with("%") {
        if let Ok(num) = value.trim_end_matches("%").parse::<f32>() {
            return Some(StyleValue::Length(num, LengthUnit::Percent));
        }
    }

    if value.ends_with("em") {
        if let Ok(num) = value.trim_end_matches("em").parse::<f32>() {
            return Some(StyleValue::Length(num, LengthUnit::Em));
        }
    }

    // Try to parse as number
    if let Ok(num) = value.parse::<f32>() {
        return Some(StyleValue::Number(num));
    }

    // Otherwise, treat as string/keyword
    Some(StyleValue::String(value.to_string()))
}

fn parse_color(color_str: &str) -> Option<Color> {
    let color = color_str.trim();

    // Parse hex colors
    if color.starts_with("#") {
        return parse_hex_color(color);
    }

    // Parse rgb/rgba
    if color.starts_with("rgb(") || color.starts_with("rgba(") {
        return parse_rgb_color(color);
    }

    // Parse named colors
    parse_named_color(color)
}

fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches("#");

    let (r, g, b) = match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            (r, g, b)
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            (r, g, b)
        }
        _ => return None,
    };

    Some(Color { r, g, b, a: 1.0 })
}

fn parse_rgb_color(rgb: &str) -> Option<Color> {
    let is_rgba = rgb.starts_with("rgba");
    let content = rgb
        .trim_start_matches("rgb(")
        .trim_start_matches("rgba(")
        .trim_end_matches(")");

    let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();

    if (!is_rgba && parts.len() != 3) || (is_rgba && parts.len() != 4) {
        return None;
    }

    let r = parts[0].parse::<u8>().ok()?;
    let g = parts[1].parse::<u8>().ok()?;
    let b = parts[2].parse::<u8>().ok()?;
    let a = if is_rgba {
        parts[3].parse::<f32>().ok()?
    } else {
        1.0
    };

    Some(Color { r, g, b, a })
}

fn parse_named_color(name: &str) -> Option<Color> {
    // Basic named colors
    match name.to_lowercase().as_str() {
        "black" => Some(Color {
            r: 0,
            g: 0,
            b: 0,
            a: 1.0,
        }),
        "white" => Some(Color {
            r: 255,
            g: 255,
            b: 255,
            a: 1.0,
        }),
        "red" => Some(Color {
            r: 255,
            g: 0,
            b: 0,
            a: 1.0,
        }),
        "green" => Some(Color {
            r: 0,
            g: 255,
            b: 0,
            a: 1.0,
        }),
        "blue" => Some(Color {
            r: 0,
            g: 0,
            b: 255,
            a: 1.0,
        }),
        "transparent" => Some(Color {
            r: 0,
            g: 0,
            b: 0,
            a: 0.0,
        }),
        _ => None,
    }
}

impl StyleSheet {
    pub fn get_styles_for_selector(&self, selector: &str) -> HashMap<String, StyleValue> {
        let mut styles = HashMap::new();

        for rule in &self.rules {
            if self.selector_matches(&rule.selector, selector) {
                styles.extend(rule.properties.clone());
            }
        }

        styles
    }

    fn selector_matches(&self, rule_selector: &str, target: &str) -> bool {
        // Simple matching - in production, implement full CSS selector matching

        // Exact match
        if rule_selector == target {
            return true;
        }

        // Class match
        if rule_selector.starts_with(".") && target.contains(&rule_selector[1..]) {
            return true;
        }

        // Pseudo-class match (e.g., window:focus)
        if rule_selector.contains(":") {
            let parts: Vec<&str> = rule_selector.split(':').collect();
            if parts.len() == 2 && target.starts_with(parts[0]) {
                // Would need to check actual state (focused, etc.)
                return false; // Placeholder
            }
        }

        false
    }

    pub fn get_color(&self, selector: &str, property: &str) -> Option<Color> {
        let styles = self.get_styles_for_selector(selector);
        match styles.get(property)? {
            StyleValue::Color(c) => Some(*c),
            _ => None,
        }
    }

    pub fn get_length(&self, selector: &str, property: &str) -> Option<f32> {
        let styles = self.get_styles_for_selector(selector);
        match styles.get(property)? {
            StyleValue::Length(l, LengthUnit::Px) => Some(*l),
            _ => None,
        }
    }
}

impl Color {
    pub fn to_rgba_f32(&self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a,
        ]
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}
