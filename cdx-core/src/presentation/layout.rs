//! Layout configuration for multi-column and grid layouts.

use serde::{Deserialize, Serialize};

/// Multi-column layout configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnLayout {
    /// Number of columns.
    pub columns: u32,

    /// Gap between columns (e.g., "20px", "1em").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gap: Option<String>,

    /// Whether to balance content across columns.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub balance: Option<bool>,
}

impl ColumnLayout {
    /// Create a new column layout.
    #[must_use]
    pub const fn new(columns: u32) -> Self {
        Self {
            columns,
            gap: None,
            balance: None,
        }
    }

    /// Set the column gap.
    #[must_use]
    pub fn with_gap(mut self, gap: impl Into<String>) -> Self {
        self.gap = Some(gap.into());
        self
    }

    /// Enable column balancing.
    #[must_use]
    pub const fn balanced(mut self) -> Self {
        self.balance = Some(true);
        self
    }
}

/// CSS Grid layout configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridLayout {
    /// Column definition (e.g., "12", "1fr 2fr", "repeat(3, 1fr)").
    pub columns: String,

    /// Row definition (e.g., "auto 1fr auto").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows: Option<String>,

    /// Gap between grid cells (e.g., "10px").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gap: Option<String>,

    /// Named grid areas.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub areas: Option<Vec<GridArea>>,
}

impl GridLayout {
    /// Create a new grid layout.
    #[must_use]
    pub fn new(columns: impl Into<String>) -> Self {
        Self {
            columns: columns.into(),
            rows: None,
            gap: None,
            areas: None,
        }
    }

    /// Set grid rows.
    #[must_use]
    pub fn with_rows(mut self, rows: impl Into<String>) -> Self {
        self.rows = Some(rows.into());
        self
    }

    /// Set the grid gap.
    #[must_use]
    pub fn with_gap(mut self, gap: impl Into<String>) -> Self {
        self.gap = Some(gap.into());
        self
    }

    /// Set named grid areas.
    #[must_use]
    pub fn with_areas(mut self, areas: Vec<GridArea>) -> Self {
        self.areas = Some(areas);
        self
    }
}

/// A named area within a grid layout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridArea {
    /// Area name.
    pub name: String,

    /// Column span (e.g., "1 / 13", "span 6").
    pub column: String,

    /// Row span (e.g., "1 / 3", "span 2").
    pub row: String,
}

impl GridArea {
    /// Create a new grid area.
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        column: impl Into<String>,
        row: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            column: column.into(),
            row: row.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_layout_serde() {
        let layout = ColumnLayout::new(2).with_gap("20px").balanced();
        let json = serde_json::to_string_pretty(&layout).unwrap();
        assert!(json.contains("\"columns\": 2"));
        assert!(json.contains("\"gap\": \"20px\""));
        assert!(json.contains("\"balance\": true"));

        let parsed: ColumnLayout = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, layout);
    }

    #[test]
    fn test_grid_layout_serde() {
        let layout = GridLayout::new("1fr 2fr")
            .with_rows("auto 1fr")
            .with_gap("10px");

        let json = serde_json::to_string(&layout).unwrap();
        assert!(json.contains("\"columns\":\"1fr 2fr\""));

        let parsed: GridLayout = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, layout);
    }

    #[test]
    fn test_grid_area_serde() {
        let area = GridArea::new("header", "1 / 13", "1 / 2");
        let json = serde_json::to_string(&area).unwrap();
        assert!(json.contains("\"name\":\"header\""));
        assert!(json.contains("\"column\":\"1 / 13\""));

        let parsed: GridArea = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, area);
    }

    #[test]
    fn test_grid_with_areas() {
        let layout = GridLayout::new("repeat(12, 1fr)").with_areas(vec![
            GridArea::new("header", "1 / 13", "1 / 2"),
            GridArea::new("sidebar", "1 / 4", "2 / 3"),
            GridArea::new("content", "4 / 13", "2 / 3"),
        ]);

        let json = serde_json::to_string_pretty(&layout).unwrap();
        let parsed: GridLayout = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, layout);
        assert_eq!(parsed.areas.unwrap().len(), 3);
    }

    #[test]
    fn test_column_layout_defaults() {
        let json = r#"{"columns": 3}"#;
        let layout: ColumnLayout = serde_json::from_str(json).unwrap();
        assert_eq!(layout.columns, 3);
        assert!(layout.gap.is_none());
        assert!(layout.balance.is_none());
    }
}
