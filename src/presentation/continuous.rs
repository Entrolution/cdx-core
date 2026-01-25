//! Continuous presentation layer.

use serde::{Deserialize, Serialize};

use super::style::{CssValue, Style, StyleMap};

/// Continuous presentation for screen reading.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Continuous {
    /// Format version.
    pub version: String,

    /// Presentation type (always "continuous").
    #[serde(rename = "type")]
    pub presentation_type: String,

    /// Default settings.
    pub defaults: ContinuousDefaults,

    /// Content sections.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sections: Vec<Section>,

    /// Style definitions.
    #[serde(default, skip_serializing_if = "StyleMap::is_empty")]
    pub styles: StyleMap,
}

impl Default for Continuous {
    fn default() -> Self {
        Self {
            version: crate::SPEC_VERSION.to_string(),
            presentation_type: "continuous".to_string(),
            defaults: ContinuousDefaults::default(),
            sections: Vec::new(),
            styles: StyleMap::new(),
        }
    }
}

/// Default settings for continuous presentation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinuousDefaults {
    /// Maximum content width.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<CssValue>,

    /// Content padding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<CssValue>,

    /// Default font family.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,

    /// Default font size.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<CssValue>,

    /// Default line height.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_height: Option<CssValue>,
}

impl Default for ContinuousDefaults {
    fn default() -> Self {
        Self {
            max_width: Some(CssValue::String("800px".to_string())),
            padding: Some(CssValue::String("24px".to_string())),
            font_family: None,
            font_size: None,
            line_height: None,
        }
    }
}

/// A section of content in continuous presentation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    /// Block references in this section.
    pub block_refs: Vec<String>,

    /// Style name to apply.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,

    /// Inline style attributes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Style>,
}

impl Section {
    /// Create a new section with block references.
    #[must_use]
    pub fn new(block_refs: Vec<String>) -> Self {
        Self {
            block_refs,
            style: None,
            attributes: None,
        }
    }

    /// Create a section with a named style.
    #[must_use]
    pub fn with_style(block_refs: Vec<String>, style: impl Into<String>) -> Self {
        Self {
            block_refs,
            style: Some(style.into()),
            attributes: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_continuous_default() {
        let c = Continuous::default();
        assert_eq!(c.presentation_type, "continuous");
        assert!(matches!(
            c.defaults.max_width,
            Some(CssValue::String(ref s)) if s == "800px"
        ));
    }

    #[test]
    fn test_section() {
        let section = Section::with_style(vec!["block-1".to_string()], "intro");
        assert_eq!(section.block_refs, vec!["block-1"]);
        assert_eq!(section.style, Some("intro".to_string()));
    }

    #[test]
    fn test_serialization() {
        let c = Continuous::default();
        let json = serde_json::to_string_pretty(&c).unwrap();
        assert!(json.contains("\"type\": \"continuous\""));
        assert!(json.contains("\"maxWidth\": \"800px\""));
    }

    #[test]
    fn test_deserialization() {
        let json = r#"{
            "version": "0.1",
            "type": "continuous",
            "defaults": {
                "maxWidth": "720px",
                "padding": "20px",
                "fontFamily": "Georgia, serif",
                "fontSize": "18px",
                "lineHeight": 1.7
            },
            "styles": {
                "heading1": {
                    "fontSize": "2.5rem",
                    "fontWeight": 700
                }
            }
        }"#;

        let c: Continuous = serde_json::from_str(json).unwrap();
        assert_eq!(c.defaults.font_family, Some("Georgia, serif".to_string()));
        assert!(c.styles.contains_key("heading1"));
    }
}
