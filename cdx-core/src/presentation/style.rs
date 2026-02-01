//! Styling types for presentation layers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A map of style names to style definitions.
pub type StyleMap = HashMap<String, Style>;

/// Writing mode for text direction.
///
/// Controls the direction in which text flows within a block.
/// This is particularly important for CJK (Chinese, Japanese, Korean)
/// languages which can be written vertically.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WritingMode {
    /// Horizontal text, top-to-bottom block flow (default).
    /// Used for Latin, Cyrillic, Arabic, Hebrew scripts.
    #[default]
    HorizontalTb,

    /// Vertical text, right-to-left block flow.
    /// Traditional Chinese, Japanese, Korean.
    VerticalRl,

    /// Vertical text, left-to-right block flow.
    /// Used for Mongolian script.
    VerticalLr,

    /// Sideways text, right-to-left (90° clockwise rotation).
    SidewaysRl,

    /// Sideways text, left-to-right (90° counter-clockwise rotation).
    SidewaysLr,
}

/// 2D transform for element positioning.
///
/// Transforms allow rotation, scaling, skewing, and translation
/// of elements in paginated and precise layouts.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transform {
    /// Rotation angle (e.g., "90deg", "-45deg", "0.5rad").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotate: Option<String>,

    /// Scale factor (uniform or non-uniform).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale: Option<Scale>,

    /// Skew along X-axis (angle).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skew_x: Option<String>,

    /// Skew along Y-axis (angle).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skew_y: Option<String>,

    /// Translation along X-axis (length).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub translate_x: Option<String>,

    /// Translation along Y-axis (length).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub translate_y: Option<String>,

    /// 2D transformation matrix [a, b, c, d, tx, ty].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matrix: Option<[f64; 6]>,

    /// Transform origin point.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<TransformOrigin>,
}

impl Transform {
    /// Create a rotation transform.
    #[must_use]
    pub fn rotate(angle: impl Into<String>) -> Self {
        Self {
            rotate: Some(angle.into()),
            ..Default::default()
        }
    }

    /// Create a uniform scale transform.
    #[must_use]
    pub fn scale_uniform(factor: f64) -> Self {
        Self {
            scale: Some(Scale::Uniform(factor)),
            ..Default::default()
        }
    }

    /// Create a non-uniform scale transform.
    #[must_use]
    pub fn scale_xy(x: f64, y: f64) -> Self {
        Self {
            scale: Some(Scale::NonUniform { x, y }),
            ..Default::default()
        }
    }

    /// Create a translation transform.
    #[must_use]
    pub fn translate(x: impl Into<String>, y: impl Into<String>) -> Self {
        Self {
            translate_x: Some(x.into()),
            translate_y: Some(y.into()),
            ..Default::default()
        }
    }

    /// Set the transform origin.
    #[must_use]
    pub fn with_origin(mut self, origin: TransformOrigin) -> Self {
        self.origin = Some(origin);
        self
    }
}

/// Scale factor for transforms.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Scale {
    /// Uniform scaling (same factor for X and Y).
    Uniform(f64),
    /// Non-uniform scaling (different factors for X and Y).
    NonUniform {
        /// X scale factor.
        x: f64,
        /// Y scale factor.
        y: f64,
    },
}

/// Transform origin point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TransformOrigin {
    /// Keyword origin (e.g., "center", "top left").
    Keyword(String),
    /// Explicit coordinate origin.
    Point {
        /// X coordinate.
        x: String,
        /// Y coordinate.
        y: String,
    },
}

impl TransformOrigin {
    /// Center origin.
    #[must_use]
    pub fn center() -> Self {
        Self::Keyword("center".to_string())
    }

    /// Top-left origin.
    #[must_use]
    pub fn top_left() -> Self {
        Self::Keyword("top left".to_string())
    }

    /// Custom point origin.
    #[must_use]
    pub fn point(x: impl Into<String>, y: impl Into<String>) -> Self {
        Self::Point {
            x: x.into(),
            y: y.into(),
        }
    }
}

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

    // Writing mode
    /// Writing mode for text direction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub writing_mode: Option<WritingMode>,

    // Stacking
    /// Z-index for stacking order.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub z_index: Option<i32>,

    // Background images
    /// Background image URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_image: Option<String>,

    /// Background size.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_size: Option<String>,

    /// Background position.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_position: Option<String>,

    /// Background repeat.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_repeat: Option<String>,

    // Visual effects
    /// Element opacity (0.0 to 1.0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,

    /// Border radius.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<CssValue>,

    /// Box shadow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub box_shadow: Option<String>,
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

    #[test]
    fn test_writing_mode_serialization() {
        let mode = WritingMode::VerticalRl;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"vertical-rl\"");

        let mode = WritingMode::HorizontalTb;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"horizontal-tb\"");
    }

    #[test]
    fn test_writing_mode_deserialization() {
        let mode: WritingMode = serde_json::from_str("\"vertical-lr\"").unwrap();
        assert_eq!(mode, WritingMode::VerticalLr);

        let mode: WritingMode = serde_json::from_str("\"sideways-rl\"").unwrap();
        assert_eq!(mode, WritingMode::SidewaysRl);
    }

    #[test]
    fn test_transform_rotate() {
        let t = Transform::rotate("45deg");
        assert_eq!(t.rotate, Some("45deg".to_string()));
        assert!(t.scale.is_none());
    }

    #[test]
    fn test_transform_scale_uniform() {
        let t = Transform::scale_uniform(2.0);
        assert!(matches!(t.scale, Some(Scale::Uniform(s)) if (s - 2.0).abs() < 0.001));
    }

    #[test]
    fn test_transform_scale_xy() {
        let t = Transform::scale_xy(1.5, 2.0);
        if let Some(Scale::NonUniform { x, y }) = t.scale {
            assert!((x - 1.5).abs() < 0.001);
            assert!((y - 2.0).abs() < 0.001);
        } else {
            panic!("Expected NonUniform scale");
        }
    }

    #[test]
    fn test_transform_translate() {
        let t = Transform::translate("10px", "20px");
        assert_eq!(t.translate_x, Some("10px".to_string()));
        assert_eq!(t.translate_y, Some("20px".to_string()));
    }

    #[test]
    fn test_transform_origin() {
        let t = Transform::rotate("90deg").with_origin(TransformOrigin::center());
        assert!(matches!(t.origin, Some(TransformOrigin::Keyword(ref k)) if k == "center"));
    }

    #[test]
    fn test_transform_serialization() {
        let t = Transform {
            rotate: Some("45deg".to_string()),
            scale: Some(Scale::Uniform(1.5)),
            origin: Some(TransformOrigin::center()),
            ..Default::default()
        };
        let json = serde_json::to_string(&t).unwrap();
        assert!(json.contains("\"rotate\":\"45deg\""));
        assert!(json.contains("\"scale\":1.5"));
        assert!(json.contains("\"origin\":\"center\""));
    }

    #[test]
    fn test_style_with_new_properties() {
        let style = Style {
            writing_mode: Some(WritingMode::VerticalRl),
            z_index: Some(10),
            opacity: Some(0.8),
            border_radius: Some(CssValue::px(8.0)),
            background_image: Some("url('bg.png')".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string_pretty(&style).unwrap();
        assert!(json.contains("\"writingMode\": \"vertical-rl\""));
        assert!(json.contains("\"zIndex\": 10"));
        assert!(json.contains("\"opacity\": 0.8"));
        assert!(json.contains("\"borderRadius\": \"8px\""));
        assert!(json.contains("\"backgroundImage\": \"url('bg.png')\""));
    }
}
