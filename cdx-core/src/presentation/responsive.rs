//! Responsive presentation layer.
//!
//! The responsive presentation adapts content to different viewport sizes
//! using breakpoints and responsive styles.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::style::{CssValue, Style};

/// Responsive presentation for adaptive screen layouts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Responsive {
    /// Format version.
    pub version: String,

    /// Presentation type (always "responsive").
    #[serde(rename = "type")]
    pub presentation_type: String,

    /// Default settings.
    pub defaults: ResponsiveDefaults,

    /// Style definitions with breakpoint overrides.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub styles: HashMap<String, ResponsiveStyle>,
}

impl Default for Responsive {
    fn default() -> Self {
        Self::new()
    }
}

impl Responsive {
    /// Create a new responsive presentation with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: crate::SPEC_VERSION.to_string(),
            presentation_type: "responsive".to_string(),
            defaults: ResponsiveDefaults::default(),
            styles: HashMap::new(),
        }
    }

    /// Create a responsive presentation with custom breakpoints.
    #[must_use]
    pub fn with_breakpoints(breakpoints: Vec<Breakpoint>) -> Self {
        Self {
            version: crate::SPEC_VERSION.to_string(),
            presentation_type: "responsive".to_string(),
            defaults: ResponsiveDefaults {
                breakpoints,
                ..Default::default()
            },
            styles: HashMap::new(),
        }
    }

    /// Add a responsive style definition.
    #[must_use]
    pub fn with_style(mut self, name: impl Into<String>, style: ResponsiveStyle) -> Self {
        self.styles.insert(name.into(), style);
        self
    }

    /// Create a presentation with standard mobile/tablet/desktop breakpoints.
    #[must_use]
    pub fn with_standard_breakpoints() -> Self {
        Self::with_breakpoints(Breakpoint::standard())
    }
}

/// Default settings for responsive presentation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponsiveDefaults {
    /// Root font size for rem calculations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_font_size: Option<CssValue>,

    /// Viewport breakpoints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub breakpoints: Vec<Breakpoint>,

    /// Container max-width.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<CssValue>,

    /// Base content padding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<CssValue>,

    /// Default font family.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,

    /// Default line height.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_height: Option<CssValue>,
}

impl Default for ResponsiveDefaults {
    fn default() -> Self {
        Self {
            root_font_size: Some(CssValue::String("16px".to_string())),
            breakpoints: Breakpoint::standard(),
            max_width: Some(CssValue::String("1200px".to_string())),
            padding: Some(CssValue::String("16px".to_string())),
            font_family: None,
            line_height: Some(CssValue::Number(1.6)),
        }
    }
}

/// A viewport width breakpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Breakpoint {
    /// Breakpoint name (e.g., "mobile", "tablet", "desktop").
    pub name: String,

    /// Minimum viewport width (inclusive).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_width: Option<String>,

    /// Maximum viewport width (inclusive).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<String>,
}

impl Breakpoint {
    /// Create a new breakpoint with a name and min/max widths.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        min_width: Option<String>,
        max_width: Option<String>,
    ) -> Self {
        Self {
            name: name.into(),
            min_width,
            max_width,
        }
    }

    /// Create a mobile breakpoint (max-width: 599px).
    #[must_use]
    pub fn mobile() -> Self {
        Self {
            name: "mobile".to_string(),
            min_width: None,
            max_width: Some("599px".to_string()),
        }
    }

    /// Create a tablet breakpoint (600px - 1023px).
    #[must_use]
    pub fn tablet() -> Self {
        Self {
            name: "tablet".to_string(),
            min_width: Some("600px".to_string()),
            max_width: Some("1023px".to_string()),
        }
    }

    /// Create a desktop breakpoint (min-width: 1024px).
    #[must_use]
    pub fn desktop() -> Self {
        Self {
            name: "desktop".to_string(),
            min_width: Some("1024px".to_string()),
            max_width: None,
        }
    }

    /// Create a large desktop breakpoint (min-width: 1440px).
    #[must_use]
    pub fn large_desktop() -> Self {
        Self {
            name: "large-desktop".to_string(),
            min_width: Some("1440px".to_string()),
            max_width: None,
        }
    }

    /// Get standard breakpoints (mobile, tablet, desktop).
    #[must_use]
    pub fn standard() -> Vec<Self> {
        vec![Self::mobile(), Self::tablet(), Self::desktop()]
    }

    /// Convert to a CSS media query.
    #[must_use]
    pub fn to_media_query(&self) -> String {
        match (&self.min_width, &self.max_width) {
            (Some(min), Some(max)) => {
                format!("@media (min-width: {min}) and (max-width: {max})")
            }
            (Some(min), None) => format!("@media (min-width: {min})"),
            (None, Some(max)) => format!("@media (max-width: {max})"),
            (None, None) => "@media all".to_string(),
        }
    }
}

/// A style with optional breakpoint overrides.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ResponsiveStyle {
    /// Base style (applied at all sizes).
    #[serde(flatten)]
    pub base: Style,

    /// Breakpoint-specific style overrides.
    /// Key is the breakpoint name, value is the style overrides.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub breakpoints: HashMap<String, Style>,
}

impl ResponsiveStyle {
    /// Create a new responsive style with a base style.
    #[must_use]
    pub fn new(base: Style) -> Self {
        Self {
            base,
            breakpoints: HashMap::new(),
        }
    }

    /// Add a breakpoint override.
    #[must_use]
    pub fn with_breakpoint(mut self, name: impl Into<String>, style: Style) -> Self {
        self.breakpoints.insert(name.into(), style);
        self
    }

    /// Add a mobile-specific override.
    #[must_use]
    pub fn with_mobile(self, style: Style) -> Self {
        self.with_breakpoint("mobile", style)
    }

    /// Add a tablet-specific override.
    #[must_use]
    pub fn with_tablet(self, style: Style) -> Self {
        self.with_breakpoint("tablet", style)
    }

    /// Add a desktop-specific override.
    #[must_use]
    pub fn with_desktop(self, style: Style) -> Self {
        self.with_breakpoint("desktop", style)
    }

    /// Get the style for a specific breakpoint, merging with base.
    ///
    /// Returns the base style merged with breakpoint overrides.
    #[must_use]
    pub fn style_for_breakpoint(&self, breakpoint: &str) -> Style {
        let mut merged = self.base.clone();

        if let Some(override_style) = self.breakpoints.get(breakpoint) {
            // Merge override into base (override takes precedence)
            merge_styles(&mut merged, override_style);
        }

        merged
    }
}

/// Merge source style into destination, source values take precedence.
fn merge_styles(dest: &mut Style, source: &Style) {
    if source.font_family.is_some() {
        dest.font_family.clone_from(&source.font_family);
    }
    if source.font_size.is_some() {
        dest.font_size.clone_from(&source.font_size);
    }
    if source.font_weight.is_some() {
        dest.font_weight.clone_from(&source.font_weight);
    }
    if source.font_style.is_some() {
        dest.font_style.clone_from(&source.font_style);
    }
    if source.line_height.is_some() {
        dest.line_height.clone_from(&source.line_height);
    }
    if source.letter_spacing.is_some() {
        dest.letter_spacing.clone_from(&source.letter_spacing);
    }
    if source.text_align.is_some() {
        dest.text_align = source.text_align;
    }
    if source.text_decoration.is_some() {
        dest.text_decoration.clone_from(&source.text_decoration);
    }
    if source.text_transform.is_some() {
        dest.text_transform.clone_from(&source.text_transform);
    }
    if source.color.is_some() {
        dest.color.clone_from(&source.color);
    }
    if source.margin_top.is_some() {
        dest.margin_top.clone_from(&source.margin_top);
    }
    if source.margin_right.is_some() {
        dest.margin_right.clone_from(&source.margin_right);
    }
    if source.margin_bottom.is_some() {
        dest.margin_bottom.clone_from(&source.margin_bottom);
    }
    if source.margin_left.is_some() {
        dest.margin_left.clone_from(&source.margin_left);
    }
    if source.padding_top.is_some() {
        dest.padding_top.clone_from(&source.padding_top);
    }
    if source.padding_right.is_some() {
        dest.padding_right.clone_from(&source.padding_right);
    }
    if source.padding_bottom.is_some() {
        dest.padding_bottom.clone_from(&source.padding_bottom);
    }
    if source.padding_left.is_some() {
        dest.padding_left.clone_from(&source.padding_left);
    }
    if source.border_width.is_some() {
        dest.border_width.clone_from(&source.border_width);
    }
    if source.border_style.is_some() {
        dest.border_style.clone_from(&source.border_style);
    }
    if source.border_color.is_some() {
        dest.border_color.clone_from(&source.border_color);
    }
    if source.background_color.is_some() {
        dest.background_color.clone_from(&source.background_color);
    }
    if source.width.is_some() {
        dest.width.clone_from(&source.width);
    }
    if source.height.is_some() {
        dest.height.clone_from(&source.height);
    }
    if source.max_width.is_some() {
        dest.max_width.clone_from(&source.max_width);
    }
    if source.max_height.is_some() {
        dest.max_height.clone_from(&source.max_height);
    }
    if source.page_break_before.is_some() {
        dest.page_break_before.clone_from(&source.page_break_before);
    }
    if source.page_break_after.is_some() {
        dest.page_break_after.clone_from(&source.page_break_after);
    }
    if source.extends.is_some() {
        dest.extends.clone_from(&source.extends);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::style::FontWeight;

    #[test]
    fn test_responsive_default() {
        let r = Responsive::default();
        assert_eq!(r.presentation_type, "responsive");
        assert_eq!(r.defaults.breakpoints.len(), 3);
    }

    #[test]
    fn test_breakpoint_constructors() {
        let mobile = Breakpoint::mobile();
        assert_eq!(mobile.name, "mobile");
        assert!(mobile.min_width.is_none());
        assert_eq!(mobile.max_width, Some("599px".to_string()));

        let tablet = Breakpoint::tablet();
        assert_eq!(tablet.name, "tablet");
        assert_eq!(tablet.min_width, Some("600px".to_string()));
        assert_eq!(tablet.max_width, Some("1023px".to_string()));

        let desktop = Breakpoint::desktop();
        assert_eq!(desktop.name, "desktop");
        assert_eq!(desktop.min_width, Some("1024px".to_string()));
        assert!(desktop.max_width.is_none());
    }

    #[test]
    fn test_standard_breakpoints() {
        let breakpoints = Breakpoint::standard();
        assert_eq!(breakpoints.len(), 3);
        assert_eq!(breakpoints[0].name, "mobile");
        assert_eq!(breakpoints[1].name, "tablet");
        assert_eq!(breakpoints[2].name, "desktop");
    }

    #[test]
    fn test_media_query_generation() {
        assert_eq!(
            Breakpoint::mobile().to_media_query(),
            "@media (max-width: 599px)"
        );
        assert_eq!(
            Breakpoint::tablet().to_media_query(),
            "@media (min-width: 600px) and (max-width: 1023px)"
        );
        assert_eq!(
            Breakpoint::desktop().to_media_query(),
            "@media (min-width: 1024px)"
        );
    }

    #[test]
    fn test_responsive_style_with_breakpoints() {
        let base_style = Style {
            font_size: Some(CssValue::String("16px".to_string())),
            ..Default::default()
        };

        let mobile_style = Style {
            font_size: Some(CssValue::String("14px".to_string())),
            ..Default::default()
        };

        let style = ResponsiveStyle::new(base_style).with_mobile(mobile_style);

        assert!(style.breakpoints.contains_key("mobile"));
    }

    #[test]
    fn test_style_for_breakpoint() {
        let base = Style {
            font_size: Some(CssValue::String("16px".to_string())),
            font_weight: Some(FontWeight::Number(400)),
            ..Default::default()
        };

        let mobile_override = Style {
            font_size: Some(CssValue::String("14px".to_string())),
            ..Default::default()
        };

        let style = ResponsiveStyle::new(base).with_mobile(mobile_override);

        let merged = style.style_for_breakpoint("mobile");
        assert_eq!(
            merged.font_size,
            Some(CssValue::String("14px".to_string()))
        );
        assert_eq!(merged.font_weight, Some(FontWeight::Number(400)));

        // Base style for unknown breakpoint
        let base_only = style.style_for_breakpoint("desktop");
        assert_eq!(
            base_only.font_size,
            Some(CssValue::String("16px".to_string()))
        );
    }

    #[test]
    fn test_serialization() {
        let r = Responsive::with_standard_breakpoints();
        let json = serde_json::to_string_pretty(&r).unwrap();
        assert!(json.contains("\"type\": \"responsive\""));
        assert!(json.contains("\"mobile\""));
        assert!(json.contains("\"tablet\""));
        assert!(json.contains("\"desktop\""));
    }

    #[test]
    fn test_deserialization() {
        let json = r#"{
            "version": "0.1",
            "type": "responsive",
            "defaults": {
                "rootFontSize": "16px",
                "breakpoints": [
                    {"name": "mobile", "maxWidth": "599px"},
                    {"name": "tablet", "minWidth": "600px", "maxWidth": "1023px"},
                    {"name": "desktop", "minWidth": "1024px"}
                ],
                "maxWidth": "1200px",
                "lineHeight": 1.6
            },
            "styles": {
                "heading1": {
                    "fontSize": "2.5rem",
                    "fontWeight": 700,
                    "breakpoints": {
                        "mobile": {
                            "fontSize": "1.75rem"
                        }
                    }
                }
            }
        }"#;

        let r: Responsive = serde_json::from_str(json).unwrap();
        assert_eq!(r.presentation_type, "responsive");
        assert_eq!(r.defaults.breakpoints.len(), 3);
        assert!(r.styles.contains_key("heading1"));

        let h1_style = r.styles.get("heading1").unwrap();
        assert!(h1_style.breakpoints.contains_key("mobile"));
    }

    #[test]
    fn test_round_trip() {
        let base = Style {
            font_size: Some(CssValue::String("2rem".to_string())),
            font_weight: Some(FontWeight::Number(700)),
            margin_bottom: Some(CssValue::String("1rem".to_string())),
            ..Default::default()
        };

        let mobile = Style {
            font_size: Some(CssValue::String("1.5rem".to_string())),
            ..Default::default()
        };

        let heading_style = ResponsiveStyle::new(base).with_mobile(mobile);

        let responsive = Responsive::with_standard_breakpoints().with_style("heading1", heading_style);

        let json = serde_json::to_string(&responsive).unwrap();
        let parsed: Responsive = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.defaults.breakpoints.len(), 3);
        assert!(parsed.styles.contains_key("heading1"));
    }
}
