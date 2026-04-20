use serde::de;
use serde::{Deserialize, Deserializer};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

pub type TemplateId = String;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct NodeTemplate {
    pub id: TemplateId,
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub size: NodeSize,
    #[serde(default)]
    pub style: NodeStyle,
    #[serde(default)]
    pub ports: Vec<PortTemplate>,
    #[serde(default)]
    pub properties: Vec<PropertyTemplate>,
}

impl NodeTemplate {
    pub fn from_json_str(source: &str) -> Result<Self, TemplateError> {
        let template: Self = serde_json::from_str(source)?;
        template.validate()?;
        Ok(template)
    }

    pub fn validate(&self) -> Result<(), TemplateError> {
        validate_id("template id", &self.id)?;
        validate_present("template title", &self.title)?;
        self.size.validate()?;

        let mut port_ids = BTreeSet::new();
        for port in &self.ports {
            validate_id("port id", &port.id)?;
            validate_present("port label", &port.label)?;

            if !port_ids.insert(port.id.as_str()) {
                return Err(TemplateError::Invalid(format!(
                    "duplicate port id '{}'",
                    port.id
                )));
            }

            if port.capacity_per_tick == Some(0) {
                return Err(TemplateError::Invalid(format!(
                    "port '{}' has zero capacity_per_tick",
                    port.id
                )));
            }
        }

        let mut property_ids = BTreeSet::new();
        for property in &self.properties {
            validate_id("property id", &property.id)?;
            validate_present("property label", &property.label)?;

            if !property_ids.insert(property.id.as_str()) {
                return Err(TemplateError::Invalid(format!(
                    "duplicate property id '{}'",
                    property.id
                )));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct NodeSize {
    pub width: f32,
    pub header_height: f32,
    pub row_height: f32,
    pub padding: f32,
    pub port_radius: f32,
}

impl Default for NodeSize {
    fn default() -> Self {
        Self {
            width: 240.0,
            header_height: 36.0,
            row_height: 28.0,
            padding: 12.0,
            port_radius: 6.0,
        }
    }
}

impl NodeSize {
    fn validate(self) -> Result<(), TemplateError> {
        validate_positive("size.width", self.width)?;
        validate_positive("size.header_height", self.header_height)?;
        validate_positive("size.row_height", self.row_height)?;
        validate_positive("size.padding", self.padding)?;
        validate_positive("size.port_radius", self.port_radius)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(default)]
pub struct NodeStyle {
    pub header_color: Color,
    pub body_color: Color,
    pub border_color: Color,
    pub text_color: Color,
    pub item_port_color: Color,
    pub energy_port_color: Color,
    pub signal_port_color: Color,
}

impl Default for NodeStyle {
    fn default() -> Self {
        Self {
            header_color: Color::rgb(54, 75, 89),
            body_color: Color::rgb(24, 28, 32),
            border_color: Color::rgb(93, 105, 116),
            text_color: Color::rgb(235, 238, 241),
            item_port_color: Color::rgb(109, 185, 129),
            energy_port_color: Color::rgb(235, 183, 74),
            signal_port_color: Color::rgb(111, 164, 219),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        parse_hex_color(&value).map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PortTemplate {
    pub id: String,
    pub label: String,
    pub direction: PortDirection,
    #[serde(default)]
    pub kind: PortKind,
    #[serde(default)]
    pub item: Option<String>,
    #[serde(default)]
    pub capacity_per_tick: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortDirection {
    Input,
    Output,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortKind {
    Item,
    Energy,
    Signal,
}

impl Default for PortKind {
    fn default() -> Self {
        Self::Item
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PropertyTemplate {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub value: TemplateValue,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum TemplateValue {
    Bool(bool),
    Number(f64),
    Text(String),
}

impl Default for TemplateValue {
    fn default() -> Self {
        Self::Text(String::new())
    }
}

#[derive(Debug)]
pub enum TemplateError {
    Json(serde_json::Error),
    Invalid(String),
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(f, "invalid node template json: {error}"),
            Self::Invalid(message) => write!(f, "invalid node template: {message}"),
        }
    }
}

impl Error for TemplateError {}

impl From<serde_json::Error> for TemplateError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

fn validate_id(label: &str, value: &str) -> Result<(), TemplateError> {
    validate_present(label, value)?;

    if value.chars().all(|character| {
        character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
    }) {
        Ok(())
    } else {
        Err(TemplateError::Invalid(format!(
            "{label} '{value}' must use lowercase snake_case"
        )))
    }
}

fn validate_present(label: &str, value: &str) -> Result<(), TemplateError> {
    if value.trim().is_empty() {
        Err(TemplateError::Invalid(format!("{label} cannot be empty")))
    } else {
        Ok(())
    }
}

fn validate_positive(label: &str, value: f32) -> Result<(), TemplateError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(TemplateError::Invalid(format!(
            "{label} must be a positive finite number"
        )))
    }
}

fn parse_hex_color(value: &str) -> Result<Color, String> {
    let Some(hex) = value.strip_prefix('#') else {
        return Err(format!("color '{value}' must start with #"));
    };

    match hex.len() {
        6 => Ok(Color::rgb(
            parse_hex_byte(hex, 0)?,
            parse_hex_byte(hex, 2)?,
            parse_hex_byte(hex, 4)?,
        )),
        8 => Ok(Color::rgba(
            parse_hex_byte(hex, 0)?,
            parse_hex_byte(hex, 2)?,
            parse_hex_byte(hex, 4)?,
            parse_hex_byte(hex, 6)?,
        )),
        _ => Err(format!("color '{value}' must use #RRGGBB or #RRGGBBAA")),
    }
}

fn parse_hex_byte(hex: &str, start: usize) -> Result<u8, String> {
    u8::from_str_radix(&hex[start..start + 2], 16)
        .map_err(|_| format!("invalid hex color byte '{}'", &hex[start..start + 2]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sample_node_template() {
        let template = NodeTemplate::from_json_str(include_str!(
            "../../../assets/node_templates/ore_source.json"
        ))
        .expect("sample template should parse");

        assert_eq!(template.id, "ore_source");
        assert_eq!(template.ports.len(), 4);
        assert_eq!(template.style.header_color, Color::rgb(42, 92, 67));
    }

    #[test]
    fn rejects_duplicate_ports() {
        let source = r##"
        {
            "id": "bad_node",
            "title": "Bad Node",
            "ports": [
                { "id": "ore", "label": "Ore", "direction": "input" },
                { "id": "ore", "label": "Ore", "direction": "output" }
            ]
        }
        "##;

        let error = NodeTemplate::from_json_str(source).unwrap_err();
        assert!(error.to_string().contains("duplicate port id"));
    }

    #[test]
    fn parses_rgba_hex_color() {
        let color = parse_hex_color("#10203040").unwrap();
        assert_eq!(color, Color::rgba(16, 32, 48, 64));
    }
}
