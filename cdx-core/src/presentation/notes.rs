//! Footnote and endnote configuration.

use serde::{Deserialize, Serialize};

use super::Style;

/// Footnotes configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FootnotesConfig {
    /// Numbering scheme (e.g., "decimal", "lower-alpha", "lower-roman", "symbols").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numbering: Option<String>,

    /// Where footnotes are placed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<FootnotePosition>,

    /// Separator line configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub separator: Option<FootnoteSeparator>,

    /// Style for footnote text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<Style>,
}

impl FootnotesConfig {
    /// Create a new footnotes configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            numbering: None,
            position: None,
            separator: None,
            style: None,
        }
    }

    /// Set the numbering scheme.
    #[must_use]
    pub fn with_numbering(mut self, numbering: impl Into<String>) -> Self {
        self.numbering = Some(numbering.into());
        self
    }

    /// Set the footnote position.
    #[must_use]
    pub const fn with_position(mut self, position: FootnotePosition) -> Self {
        self.position = Some(position);
        self
    }

    /// Set the separator configuration.
    #[must_use]
    pub fn with_separator(mut self, separator: FootnoteSeparator) -> Self {
        self.separator = Some(separator);
        self
    }
}

impl Default for FootnotesConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Where footnotes are placed in the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FootnotePosition {
    /// At the bottom of each page.
    PageBottom,
    /// At the end of each section.
    SectionEnd,
    /// At the end of the document.
    DocumentEnd,
}

/// Configuration for the footnote separator line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FootnoteSeparator {
    /// Width of the separator (e.g., "33%", "100px").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,

    /// Line style (e.g., "solid", "dashed", "dotted").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,

    /// Margin above and below (e.g., "8px").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub margin: Option<String>,
}

impl FootnoteSeparator {
    /// Create a new separator configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            width: None,
            style: None,
            margin: None,
        }
    }

    /// Set the separator width.
    #[must_use]
    pub fn with_width(mut self, width: impl Into<String>) -> Self {
        self.width = Some(width.into());
        self
    }

    /// Set the line style.
    #[must_use]
    pub fn with_style(mut self, style: impl Into<String>) -> Self {
        self.style = Some(style.into());
        self
    }
}

impl Default for FootnoteSeparator {
    fn default() -> Self {
        Self::new()
    }
}

/// Endnotes configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndnotesConfig {
    /// Title for the endnotes section.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Numbering scheme (e.g., "decimal", "lower-roman").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numbering: Option<String>,

    /// Whether to restart numbering per chapter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub per_chapter: Option<bool>,
}

impl EndnotesConfig {
    /// Create a new endnotes configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: None,
            numbering: None,
            per_chapter: None,
        }
    }

    /// Set the endnotes section title.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the numbering scheme.
    #[must_use]
    pub fn with_numbering(mut self, numbering: impl Into<String>) -> Self {
        self.numbering = Some(numbering.into());
        self
    }

    /// Enable per-chapter numbering.
    #[must_use]
    pub const fn per_chapter(mut self) -> Self {
        self.per_chapter = Some(true);
        self
    }
}

impl Default for EndnotesConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_footnotes_config_serde() {
        let config = FootnotesConfig::new()
            .with_numbering("lower-roman")
            .with_position(FootnotePosition::PageBottom)
            .with_separator(
                FootnoteSeparator::new()
                    .with_width("33%")
                    .with_style("solid"),
            );

        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("\"numbering\": \"lower-roman\""));
        assert!(json.contains("\"position\": \"pageBottom\""));

        let parsed: FootnotesConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, config);
    }

    #[test]
    fn test_footnote_position_serde() {
        assert_eq!(
            serde_json::to_string(&FootnotePosition::PageBottom).unwrap(),
            "\"pageBottom\""
        );
        assert_eq!(
            serde_json::to_string(&FootnotePosition::SectionEnd).unwrap(),
            "\"sectionEnd\""
        );
        assert_eq!(
            serde_json::to_string(&FootnotePosition::DocumentEnd).unwrap(),
            "\"documentEnd\""
        );
    }

    #[test]
    fn test_endnotes_config_serde() {
        let config = EndnotesConfig::new()
            .with_title("Notes")
            .with_numbering("decimal")
            .per_chapter();

        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("\"title\": \"Notes\""));
        assert!(json.contains("\"perChapter\": true"));

        let parsed: EndnotesConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, config);
    }

    #[test]
    fn test_footnote_separator_serde() {
        let sep = FootnoteSeparator::new()
            .with_width("50%")
            .with_style("dashed");
        let json = serde_json::to_string(&sep).unwrap();
        assert!(json.contains("\"width\":\"50%\""));

        let parsed: FootnoteSeparator = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, sep);
    }

    #[test]
    fn test_footnotes_defaults() {
        let json = "{}";
        let config: FootnotesConfig = serde_json::from_str(json).unwrap();
        assert!(config.numbering.is_none());
        assert!(config.position.is_none());
        assert!(config.separator.is_none());
        assert!(config.style.is_none());
    }

    #[test]
    fn test_endnotes_defaults() {
        let json = "{}";
        let config: EndnotesConfig = serde_json::from_str(json).unwrap();
        assert!(config.title.is_none());
        assert!(config.numbering.is_none());
        assert!(config.per_chapter.is_none());
    }
}
