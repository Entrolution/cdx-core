//! Typography configuration for advanced text rendering.

use serde::{Deserialize, Serialize};

/// Typography configuration for a presentation layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypographyConfig {
    /// Line numbering configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_numbering: Option<LineNumbering>,

    /// Baseline grid alignment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub baseline_grid: Option<BaselineGrid>,

    /// Hyphenation settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hyphenation: Option<HyphenationConfig>,
}

impl TypographyConfig {
    /// Create a new empty typography configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            line_numbering: None,
            baseline_grid: None,
            hyphenation: None,
        }
    }

    /// Set line numbering.
    #[must_use]
    pub fn with_line_numbering(mut self, ln: LineNumbering) -> Self {
        self.line_numbering = Some(ln);
        self
    }

    /// Set baseline grid.
    #[must_use]
    pub fn with_baseline_grid(mut self, bg: BaselineGrid) -> Self {
        self.baseline_grid = Some(bg);
        self
    }

    /// Set hyphenation.
    #[must_use]
    pub fn with_hyphenation(mut self, h: HyphenationConfig) -> Self {
        self.hyphenation = Some(h);
        self
    }
}

impl Default for TypographyConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Line numbering configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineNumbering {
    /// Whether line numbering is enabled.
    pub enabled: bool,

    /// Starting line number (default: 1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<u32>,

    /// Show number every N lines (default: 1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval: Option<u32>,

    /// Which side to display numbers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side: Option<LineNumberingSide>,

    /// When to restart numbering.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restart: Option<LineNumberingRestart>,
}

impl LineNumbering {
    /// Create enabled line numbering with defaults.
    #[must_use]
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            start: None,
            interval: None,
            side: None,
            restart: None,
        }
    }

    /// Set the starting number.
    #[must_use]
    pub const fn with_start(mut self, start: u32) -> Self {
        self.start = Some(start);
        self
    }

    /// Set the numbering interval.
    #[must_use]
    pub const fn with_interval(mut self, interval: u32) -> Self {
        self.interval = Some(interval);
        self
    }
}

/// Side for line numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LineNumberingSide {
    /// Display on the left margin.
    Left,
    /// Display on the right margin.
    Right,
}

/// When to restart line numbering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LineNumberingRestart {
    /// Restart at each page.
    Page,
    /// Restart at each section.
    Section,
    /// Never restart (continuous numbering).
    #[serde(rename = "none")]
    NoRestart,
}

/// Baseline grid configuration for vertical rhythm.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaselineGrid {
    /// Whether baseline grid snapping is enabled.
    pub enabled: bool,

    /// Line height for the grid (e.g., "14pt").
    pub line_height: String,

    /// Vertical offset from the top (e.g., "10pt").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<String>,
}

impl BaselineGrid {
    /// Create a baseline grid with the given line height.
    #[must_use]
    pub fn new(line_height: impl Into<String>) -> Self {
        Self {
            enabled: true,
            line_height: line_height.into(),
            offset: None,
        }
    }
}

/// Hyphenation configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HyphenationConfig {
    /// Whether hyphenation is enabled.
    pub enabled: bool,

    /// Language for hyphenation rules (BCP 47 tag).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Minimum word length to hyphenate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_word_length: Option<u32>,

    /// Minimum characters before a hyphen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_before: Option<u32>,

    /// Minimum characters after a hyphen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_after: Option<u32>,

    /// Maximum consecutive hyphenated lines.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_consecutive: Option<u32>,
}

impl HyphenationConfig {
    /// Create enabled hyphenation with defaults.
    #[must_use]
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            language: None,
            min_word_length: None,
            min_before: None,
            min_after: None,
            max_consecutive: None,
        }
    }

    /// Set the language.
    #[must_use]
    pub fn with_language(mut self, lang: impl Into<String>) -> Self {
        self.language = Some(lang.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typography_config_serde() {
        let config = TypographyConfig::new()
            .with_line_numbering(LineNumbering::enabled().with_interval(5))
            .with_hyphenation(HyphenationConfig::enabled().with_language("en-US"));

        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: TypographyConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, config);
    }

    #[test]
    fn test_line_numbering_serde() {
        let ln = LineNumbering::enabled().with_start(10).with_interval(5);
        let json = serde_json::to_string(&ln).unwrap();
        assert!(json.contains("\"start\":10"));
        assert!(json.contains("\"interval\":5"));

        let parsed: LineNumbering = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ln);
    }

    #[test]
    fn test_line_numbering_side_serde() {
        let side = LineNumberingSide::Left;
        let json = serde_json::to_string(&side).unwrap();
        assert_eq!(json, "\"left\"");

        let right: LineNumberingSide = serde_json::from_str("\"right\"").unwrap();
        assert_eq!(right, LineNumberingSide::Right);
    }

    #[test]
    fn test_line_numbering_restart_serde() {
        let restart = LineNumberingRestart::NoRestart;
        let json = serde_json::to_string(&restart).unwrap();
        assert_eq!(json, "\"none\"");

        let page: LineNumberingRestart = serde_json::from_str("\"page\"").unwrap();
        assert_eq!(page, LineNumberingRestart::Page);
    }

    #[test]
    fn test_baseline_grid_serde() {
        let grid = BaselineGrid::new("14pt");
        let json = serde_json::to_string(&grid).unwrap();
        assert!(json.contains("\"lineHeight\":\"14pt\""));

        let parsed: BaselineGrid = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, grid);
    }

    #[test]
    fn test_hyphenation_serde() {
        let hyph = HyphenationConfig::enabled().with_language("de");
        let json = serde_json::to_string(&hyph).unwrap();
        assert!(json.contains("\"language\":\"de\""));

        let parsed: HyphenationConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, hyph);
    }

    #[test]
    fn test_typography_defaults() {
        let json = "{}";
        let config: TypographyConfig = serde_json::from_str(json).unwrap();
        assert!(config.line_numbering.is_none());
        assert!(config.baseline_grid.is_none());
        assert!(config.hyphenation.is_none());
    }
}
