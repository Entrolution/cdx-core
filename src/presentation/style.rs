//! Styling types for presentation layers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A map of style names to style definitions.
pub type StyleMap = HashMap<String, Style>;

/// CSS-like style properties.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Style {
    // Typography
    /// Font family stack.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,

    /// Font size with units.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<CssValue>,

    /// Font weight (100-900 or keyword).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<FontWeight>,

    /// Font style (normal, italic).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_style: Option<String>,

    /// Line height (unitless ratio or with units).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_height: Option<CssValue>,

    /// Letter spacing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub letter_spacing: Option<CssValue>,

    /// Text alignment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_align: Option<TextAlign>,

    /// Text decoration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_decoration: Option<String>,

    /// Text transform.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_transform: Option<String>,

    /// Text color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,

    // Spacing
    /// Top margin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub margin_top: Option<CssValue>,

    /// Right margin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub margin_right: Option<CssValue>,

    /// Bottom margin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub margin_bottom: Option<CssValue>,

    /// Left margin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub margin_left: Option<CssValue>,

    /// Top padding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding_top: Option<CssValue>,

    /// Right padding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding_right: Option<CssValue>,

    /// Bottom padding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding_bottom: Option<CssValue>,

    /// Left padding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding_left: Option<CssValue>,

    // Borders
    /// Border width.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_width: Option<CssValue>,

    /// Border style.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_style: Option<String>,

    /// Border color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_color: Option<Color>,

    // Background
    /// Background color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_color: Option<Color>,

    // Layout
    /// Width.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<CssValue>,

    /// Height.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<CssValue>,

    /// Maximum width.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<CssValue>,

    /// Maximum height.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_height: Option<CssValue>,

    // Page breaks (for print)
    /// Page break before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_break_before: Option<String>,

    /// Page break after.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_break_after: Option<String>,

    // Inheritance
    /// Style to inherit from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,
}

/// CSS value with units.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CssValue {
    /// Numeric value (for unitless values like line-height).
    Number(f32),
    /// String value with units (e.g., "16px", "1.5em").
    String(String),
}

impl CssValue {
    /// Create a pixel value.
    #[must_use]
    pub fn px(value: f32) -> Self {
        Self::String(format!("{value}px"))
    }

    /// Create a point value.
    #[must_use]
    pub fn pt(value: f32) -> Self {
        Self::String(format!("{value}pt"))
    }

    /// Create an em value.
    #[must_use]
    pub fn em(value: f32) -> Self {
        Self::String(format!("{value}em"))
    }

    /// Create a rem value.
    #[must_use]
    pub fn rem(value: f32) -> Self {
        Self::String(format!("{value}rem"))
    }

    /// Create a percentage value.
    #[must_use]
    pub fn percent(value: f32) -> Self {
        Self::String(format!("{value}%"))
    }

    /// Create an inch value.
    #[must_use]
    pub fn inch(value: f32) -> Self {
        Self::String(format!("{value}in"))
    }
}

impl From<f32> for CssValue {
    fn from(value: f32) -> Self {
        Self::Number(value)
    }
}

impl From<&str> for CssValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}

/// Font weight.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FontWeight {
    /// Numeric weight (100-900).
    Number(u16),
    /// Keyword (normal, bold, etc.).
    Keyword(String),
}

impl FontWeight {
    /// Normal weight (400).
    #[must_use]
    pub fn normal() -> Self {
        Self::Number(400)
    }

    /// Bold weight (700).
    #[must_use]
    pub fn bold() -> Self {
        Self::Number(700)
    }
}

/// Text alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextAlign {
    /// Left alignment.
    Left,
    /// Center alignment.
    Center,
    /// Right alignment.
    Right,
    /// Justified alignment.
    Justify,
}

/// Color value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Color {
    /// Named color or hex value.
    Named(String),
}

impl Color {
    /// Create a hex color.
    #[must_use]
    pub fn hex(value: impl Into<String>) -> Self {
        Self::Named(value.into())
    }

    /// Black color.
    #[must_use]
    pub fn black() -> Self {
        Self::Named("black".to_string())
    }

    /// White color.
    #[must_use]
    pub fn white() -> Self {
        Self::Named("white".to_string())
    }
}

impl From<&str> for Color {
    fn from(value: &str) -> Self {
        Self::Named(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_default() {
        let style = Style::default();
        assert!(style.font_family.is_none());
        assert!(style.font_size.is_none());
    }

    #[test]
    fn test_css_value_units() {
        assert!(matches!(CssValue::px(16.0), CssValue::String(s) if s == "16px"));
        assert!(matches!(CssValue::em(1.5), CssValue::String(s) if s == "1.5em"));
        assert!(matches!(CssValue::percent(100.0), CssValue::String(s) if s == "100%"));
    }

    #[test]
    fn test_style_serialization() {
        let style = Style {
            font_family: Some("Georgia, serif".to_string()),
            font_size: Some(CssValue::px(16.0)),
            font_weight: Some(FontWeight::bold()),
            color: Some(Color::hex("#333")),
            ..Default::default()
        };

        let json = serde_json::to_string_pretty(&style).unwrap();
        assert!(json.contains("\"fontFamily\": \"Georgia, serif\""));
        assert!(json.contains("\"fontSize\": \"16px\""));
        assert!(json.contains("\"fontWeight\": 700"));
    }

    #[test]
    fn test_style_deserialization() {
        let json = r##"{
            "fontFamily": "system-ui, sans-serif",
            "fontSize": "1rem",
            "lineHeight": 1.6,
            "marginBottom": "1em",
            "color": "#333333"
        }"##;

        let style: Style = serde_json::from_str(json).unwrap();
        assert_eq!(style.font_family, Some("system-ui, sans-serif".to_string()));
        assert!(matches!(style.line_height, Some(CssValue::Number(n)) if (n - 1.6).abs() < 0.001));
    }
}
