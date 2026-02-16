//! Table of contents configuration.

use serde::{Deserialize, Serialize};

/// Table of contents configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TocConfig {
    /// Custom title for the table of contents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Which heading levels to include (e.g., [1, 2, 3]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub levels: Option<Vec<u32>>,

    /// Whether to show page numbers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_numbers: Option<bool>,

    /// Leader style between title and page number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leaders: Option<TocLeaders>,
}

impl TocConfig {
    /// Create a new table of contents configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: None,
            levels: None,
            page_numbers: None,
            leaders: None,
        }
    }

    /// Set the TOC title.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set which heading levels to include.
    #[must_use]
    pub fn with_levels(mut self, levels: Vec<u32>) -> Self {
        self.levels = Some(levels);
        self
    }

    /// Enable page numbers.
    #[must_use]
    pub const fn with_page_numbers(mut self) -> Self {
        self.page_numbers = Some(true);
        self
    }

    /// Set the leader style.
    #[must_use]
    pub const fn with_leaders(mut self, leaders: TocLeaders) -> Self {
        self.leaders = Some(leaders);
        self
    }
}

impl Default for TocConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Leader style for table of contents entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TocLeaders {
    /// Dotted leaders (. . . . .).
    Dots,
    /// Dashed leaders (- - - - -).
    Dashes,
    /// No leaders.
    #[serde(rename = "none")]
    NoLeaders,
    /// Solid line leaders.
    Solid,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toc_config_serde() {
        let config = TocConfig::new()
            .with_title("Table of Contents")
            .with_levels(vec![1, 2, 3])
            .with_page_numbers()
            .with_leaders(TocLeaders::Dots);

        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("\"title\": \"Table of Contents\""));
        assert!(json.contains("\"pageNumbers\": true"));
        assert!(json.contains("\"leaders\": \"dots\""));

        let parsed: TocConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, config);
    }

    #[test]
    fn test_toc_leaders_serde() {
        assert_eq!(serde_json::to_string(&TocLeaders::Dots).unwrap(), "\"dots\"");
        assert_eq!(
            serde_json::to_string(&TocLeaders::Dashes).unwrap(),
            "\"dashes\""
        );
        assert_eq!(
            serde_json::to_string(&TocLeaders::NoLeaders).unwrap(),
            "\"none\""
        );
        assert_eq!(
            serde_json::to_string(&TocLeaders::Solid).unwrap(),
            "\"solid\""
        );
    }

    #[test]
    fn test_toc_defaults() {
        let json = "{}";
        let config: TocConfig = serde_json::from_str(json).unwrap();
        assert!(config.title.is_none());
        assert!(config.levels.is_none());
        assert!(config.page_numbers.is_none());
        assert!(config.leaders.is_none());
    }
}
