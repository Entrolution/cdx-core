//! Paginated presentation layer.

use serde::{Deserialize, Serialize};

use super::style::StyleMap;

/// Paginated presentation for print/PDF output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Paginated {
    /// Format version.
    pub version: String,

    /// Presentation type (always "paginated").
    #[serde(rename = "type")]
    pub presentation_type: String,

    /// Default page settings.
    pub defaults: PaginatedDefaults,

    /// Page definitions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pages: Vec<Page>,

    /// Style definitions.
    #[serde(default, skip_serializing_if = "StyleMap::is_empty")]
    pub styles: StyleMap,
}

impl Default for Paginated {
    fn default() -> Self {
        Self {
            version: crate::SPEC_VERSION.to_string(),
            presentation_type: "paginated".to_string(),
            defaults: PaginatedDefaults::default(),
            pages: Vec::new(),
            styles: StyleMap::new(),
        }
    }
}

/// Default settings for paginated presentation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedDefaults {
    /// Page size.
    pub page_size: PageSize,

    /// Page orientation.
    pub orientation: Orientation,

    /// Page margins.
    pub margins: Margins,
}

impl Default for PaginatedDefaults {
    fn default() -> Self {
        Self {
            page_size: PageSize::letter(),
            orientation: Orientation::Portrait,
            margins: Margins::default(),
        }
    }
}

/// Standard page sizes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PageSize {
    /// Named page size.
    Named(StandardPageSize),
    /// Custom page size with dimensions.
    Custom {
        /// Page width with units (e.g., "8in").
        width: String,
        /// Page height with units (e.g., "10in").
        height: String,
    },
}

impl PageSize {
    /// US Letter size (8.5 x 11 in).
    #[must_use]
    pub fn letter() -> Self {
        Self::Named(StandardPageSize::Letter)
    }

    /// US Legal size (8.5 x 14 in).
    #[must_use]
    pub fn legal() -> Self {
        Self::Named(StandardPageSize::Legal)
    }

    /// A4 size (210 x 297 mm).
    #[must_use]
    pub fn a4() -> Self {
        Self::Named(StandardPageSize::A4)
    }

    /// A5 size (148 x 210 mm).
    #[must_use]
    pub fn a5() -> Self {
        Self::Named(StandardPageSize::A5)
    }
}

impl Default for PageSize {
    fn default() -> Self {
        Self::letter()
    }
}

/// Standard named page sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StandardPageSize {
    /// US Letter (8.5 x 11 in).
    Letter,
    /// US Legal (8.5 x 14 in).
    Legal,
    /// A4 (210 x 297 mm).
    A4,
    /// A5 (148 x 210 mm).
    A5,
}

/// Page orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    /// Portrait orientation (taller than wide).
    #[default]
    Portrait,
    /// Landscape orientation (wider than tall).
    Landscape,
}

/// Page margins.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Margins {
    /// Top margin.
    pub top: String,
    /// Right margin.
    pub right: String,
    /// Bottom margin.
    pub bottom: String,
    /// Left margin.
    pub left: String,
}

impl Default for Margins {
    fn default() -> Self {
        Self {
            top: "1in".to_string(),
            right: "1in".to_string(),
            bottom: "1in".to_string(),
            left: "1in".to_string(),
        }
    }
}

impl Margins {
    /// Create margins with all sides equal.
    #[must_use]
    pub fn all(value: impl Into<String>) -> Self {
        let v = value.into();
        Self {
            top: v.clone(),
            right: v.clone(),
            bottom: v.clone(),
            left: v,
        }
    }
}

/// A page in a paginated presentation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Page {
    /// Page number (1-indexed).
    pub number: u32,

    /// Elements on this page.
    #[serde(default)]
    pub elements: Vec<PageElement>,
}

/// An element positioned on a page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageElement {
    /// Reference to content block ID.
    pub block_ref: String,

    /// Position and size.
    pub position: Position,

    /// Style name to apply.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,

    /// Overflow behavior.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overflow: Option<String>,
}

/// Element position on a page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    /// X position from left.
    pub x: String,
    /// Y position from top.
    pub y: String,
    /// Width (or "auto").
    pub width: String,
    /// Height (or "auto").
    pub height: String,
}

impl Position {
    /// Create a position with auto height.
    #[must_use]
    pub fn new(x: impl Into<String>, y: impl Into<String>, width: impl Into<String>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            width: width.into(),
            height: "auto".to_string(),
        }
    }
}

/// Flow element for automatic text flow across pages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowElement {
    /// Element type (always "flow").
    #[serde(rename = "type")]
    pub element_type: String,

    /// Block references to flow.
    pub block_refs: Vec<String>,

    /// Number of columns.
    #[serde(default = "default_columns")]
    pub columns: u32,

    /// Starting page number.
    pub start_page: u32,

    /// Flow regions on each page.
    pub regions: Vec<FlowRegion>,
}

fn default_columns() -> u32 {
    1
}

/// A region where content can flow.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlowRegion {
    /// Page number for this region.
    pub page: u32,
    /// Position of the region.
    pub position: Position,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paginated_default() {
        let p = Paginated::default();
        assert_eq!(p.presentation_type, "paginated");
        assert_eq!(p.defaults.page_size, PageSize::letter());
        assert_eq!(p.defaults.orientation, Orientation::Portrait);
    }

    #[test]
    fn test_margins() {
        let m = Margins::all("0.5in");
        assert_eq!(m.top, "0.5in");
        assert_eq!(m.right, "0.5in");
        assert_eq!(m.bottom, "0.5in");
        assert_eq!(m.left, "0.5in");
    }

    #[test]
    fn test_serialization() {
        let p = Paginated::default();
        let json = serde_json::to_string_pretty(&p).unwrap();
        assert!(json.contains("\"type\": \"paginated\""));
        assert!(json.contains("\"pageSize\": \"letter\""));
    }

    #[test]
    fn test_custom_page_size() {
        let size = PageSize::Custom {
            width: "8in".to_string(),
            height: "10in".to_string(),
        };
        let json = serde_json::to_string(&size).unwrap();
        assert!(json.contains("\"width\":\"8in\""));
    }
}
