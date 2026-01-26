//! Precise layout presentation layer.
//!
//! Precise layouts provide exact coordinates for every element, enabling
//! pixel-perfect reproduction regardless of rendering implementation.
//! They are **required** for FROZEN and PUBLISHED documents.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::paginated::Margins;

/// Precise layout for a specific page format.
///
/// Precise layouts store exact positions for all elements, ensuring
/// identical rendering across different implementations. This is
/// required for documents in FROZEN or PUBLISHED state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreciseLayout {
    /// Format version.
    pub version: String,

    /// Presentation type (always "precise").
    pub presentation_type: String,

    /// Target page format name (e.g., "letter", "a4", "legal", "custom").
    pub target_format: String,

    /// Exact page dimensions.
    pub page_size: PrecisePageSize,

    /// Hash of content when this layout was generated.
    /// Used to detect staleness when content changes.
    pub content_hash: String,

    /// Timestamp when this layout was generated.
    pub generated_at: DateTime<Utc>,

    /// Optional page template for headers/footers/margins.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_template: Option<PageTemplate>,

    /// Page definitions with precise element positions.
    pub pages: Vec<PrecisePage>,

    /// Font metrics for exact text reproduction.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub fonts: HashMap<String, FontMetrics>,
}

impl PreciseLayout {
    /// Create a new precise layout for US Letter format.
    #[must_use]
    pub fn new_letter(content_hash: impl Into<String>) -> Self {
        Self {
            version: crate::SPEC_VERSION.to_string(),
            presentation_type: "precise".to_string(),
            target_format: "letter".to_string(),
            page_size: PrecisePageSize::letter(),
            content_hash: content_hash.into(),
            generated_at: Utc::now(),
            page_template: None,
            pages: Vec::new(),
            fonts: HashMap::new(),
        }
    }

    /// Create a new precise layout for A4 format.
    #[must_use]
    pub fn new_a4(content_hash: impl Into<String>) -> Self {
        Self {
            version: crate::SPEC_VERSION.to_string(),
            presentation_type: "precise".to_string(),
            target_format: "a4".to_string(),
            page_size: PrecisePageSize::a4(),
            content_hash: content_hash.into(),
            generated_at: Utc::now(),
            page_template: None,
            pages: Vec::new(),
            fonts: HashMap::new(),
        }
    }

    /// Create a new precise layout for US Legal format.
    #[must_use]
    pub fn new_legal(content_hash: impl Into<String>) -> Self {
        Self {
            version: crate::SPEC_VERSION.to_string(),
            presentation_type: "precise".to_string(),
            target_format: "legal".to_string(),
            page_size: PrecisePageSize::legal(),
            content_hash: content_hash.into(),
            generated_at: Utc::now(),
            page_template: None,
            pages: Vec::new(),
            fonts: HashMap::new(),
        }
    }

    /// Check if this layout is stale (content has changed).
    #[must_use]
    pub fn is_stale(&self, current_content_hash: &str) -> bool {
        self.content_hash != current_content_hash
    }

    /// Add a page to this layout.
    pub fn add_page(&mut self, page: PrecisePage) {
        self.pages.push(page);
    }

    /// Set the page template.
    pub fn with_template(mut self, template: PageTemplate) -> Self {
        self.page_template = Some(template);
        self
    }

    /// Add font metrics.
    pub fn with_font(mut self, name: impl Into<String>, metrics: FontMetrics) -> Self {
        self.fonts.insert(name.into(), metrics);
        self
    }
}

/// Exact page dimensions for precise layouts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrecisePageSize {
    /// Page width with units (e.g., "8.5in", "210mm").
    pub width: String,
    /// Page height with units (e.g., "11in", "297mm").
    pub height: String,
}

impl PrecisePageSize {
    /// US Letter size (8.5 x 11 in).
    #[must_use]
    pub fn letter() -> Self {
        Self {
            width: "8.5in".to_string(),
            height: "11in".to_string(),
        }
    }

    /// US Legal size (8.5 x 14 in).
    #[must_use]
    pub fn legal() -> Self {
        Self {
            width: "8.5in".to_string(),
            height: "14in".to_string(),
        }
    }

    /// A4 size (210 x 297 mm).
    #[must_use]
    pub fn a4() -> Self {
        Self {
            width: "210mm".to_string(),
            height: "297mm".to_string(),
        }
    }

    /// A5 size (148 x 210 mm).
    #[must_use]
    pub fn a5() -> Self {
        Self {
            width: "148mm".to_string(),
            height: "210mm".to_string(),
        }
    }

    /// Custom page size.
    #[must_use]
    pub fn custom(width: impl Into<String>, height: impl Into<String>) -> Self {
        Self {
            width: width.into(),
            height: height.into(),
        }
    }
}

/// Page template for headers, footers, and margins.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageTemplate {
    /// Page margins.
    pub margins: Margins,

    /// Header region.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub header: Option<PageRegion>,

    /// Footer region.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub footer: Option<PageRegion>,
}

impl Default for PageTemplate {
    fn default() -> Self {
        Self {
            margins: Margins::default(),
            header: None,
            footer: None,
        }
    }
}

impl PageTemplate {
    /// Create a template with default margins and no header/footer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set custom margins.
    #[must_use]
    pub fn with_margins(mut self, margins: Margins) -> Self {
        self.margins = margins;
        self
    }

    /// Set header region.
    #[must_use]
    pub fn with_header(mut self, header: PageRegion) -> Self {
        self.header = Some(header);
        self
    }

    /// Set footer region.
    #[must_use]
    pub fn with_footer(mut self, footer: PageRegion) -> Self {
        self.footer = Some(footer);
        self
    }
}

/// Header or footer region.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageRegion {
    /// Content template. Supports placeholders:
    /// - `{pageNumber}` - Current page number
    /// - `{totalPages}` - Total page count
    pub content: String,

    /// Y position from top of page.
    pub y: String,

    /// Style name to apply.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
}

impl PageRegion {
    /// Create a new page region.
    #[must_use]
    pub fn new(content: impl Into<String>, y: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            y: y.into(),
            style: None,
        }
    }

    /// Create a page number footer.
    #[must_use]
    pub fn page_number_footer(y: impl Into<String>) -> Self {
        Self {
            content: "Page {pageNumber} of {totalPages}".to_string(),
            y: y.into(),
            style: Some("footer".to_string()),
        }
    }

    /// Set style name.
    #[must_use]
    pub fn with_style(mut self, style: impl Into<String>) -> Self {
        self.style = Some(style.into());
        self
    }
}

/// A page in a precise layout.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrecisePage {
    /// Page number (1-indexed).
    pub number: u32,

    /// Precisely positioned elements on this page.
    #[serde(default)]
    pub elements: Vec<PrecisePageElement>,
}

impl PrecisePage {
    /// Create a new page with the given number.
    #[must_use]
    pub fn new(number: u32) -> Self {
        Self {
            number,
            elements: Vec::new(),
        }
    }

    /// Add an element to this page.
    pub fn add_element(&mut self, element: PrecisePageElement) {
        self.elements.push(element);
    }

    /// Add an element and return self for chaining.
    #[must_use]
    pub fn with_element(mut self, element: PrecisePageElement) -> Self {
        self.elements.push(element);
        self
    }
}

/// A precisely positioned element on a page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrecisePageElement {
    /// Reference to content block ID.
    pub block_id: String,

    /// Horizontal position from left edge.
    pub x: String,

    /// Vertical position from top edge.
    pub y: String,

    /// Element width.
    pub width: String,

    /// Element height.
    pub height: String,

    /// True if this element continues to the next page.
    #[serde(default, skip_serializing_if = "is_false")]
    pub continues: bool,

    /// True if this element is continued from the previous page.
    #[serde(default, skip_serializing_if = "is_false")]
    pub continuation: bool,

    /// Line-level precision for legal documents.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lines: Vec<LineInfo>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

impl PrecisePageElement {
    /// Create a new element with precise positioning.
    #[must_use]
    pub fn new(
        block_id: impl Into<String>,
        x: impl Into<String>,
        y: impl Into<String>,
        width: impl Into<String>,
        height: impl Into<String>,
    ) -> Self {
        Self {
            block_id: block_id.into(),
            x: x.into(),
            y: y.into(),
            width: width.into(),
            height: height.into(),
            continues: false,
            continuation: false,
            lines: Vec::new(),
        }
    }

    /// Mark this element as continuing to the next page.
    #[must_use]
    pub fn continues(mut self) -> Self {
        self.continues = true;
        self
    }

    /// Mark this element as a continuation from the previous page.
    #[must_use]
    pub fn continuation(mut self) -> Self {
        self.continuation = true;
        self
    }

    /// Add line-level precision information.
    #[must_use]
    pub fn with_lines(mut self, lines: Vec<LineInfo>) -> Self {
        self.lines = lines;
        self
    }
}

/// Line-level precision for legal documents.
///
/// Optional - only needed for legal/court documents where line numbers
/// are referenced (e.g., "page 7, line 23").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineInfo {
    /// Line number (1-indexed within the block).
    pub number: u32,

    /// Y position of this line.
    pub y: String,

    /// Height of this line.
    pub height: String,
}

impl LineInfo {
    /// Create line information.
    #[must_use]
    pub fn new(number: u32, y: impl Into<String>, height: impl Into<String>) -> Self {
        Self {
            number,
            y: y.into(),
            height: height.into(),
        }
    }
}

/// Font metrics for exact text reproduction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FontMetrics {
    /// Font family name.
    pub family: String,

    /// Font style (normal, italic).
    #[serde(default = "default_font_style")]
    pub style: String,

    /// Font weight (100-900).
    #[serde(default = "default_font_weight")]
    pub weight: u16,

    /// Units per em.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub units_per_em: Option<u16>,

    /// Ascender height in font units.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ascender: Option<i32>,

    /// Descender depth in font units (typically negative).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descender: Option<i32>,
}

fn default_font_style() -> String {
    "normal".to_string()
}

fn default_font_weight() -> u16 {
    400
}

impl FontMetrics {
    /// Create font metrics for a font family.
    #[must_use]
    pub fn new(family: impl Into<String>) -> Self {
        Self {
            family: family.into(),
            style: default_font_style(),
            weight: default_font_weight(),
            units_per_em: None,
            ascender: None,
            descender: None,
        }
    }

    /// Set font style.
    #[must_use]
    pub fn with_style(mut self, style: impl Into<String>) -> Self {
        self.style = style.into();
        self
    }

    /// Set font weight.
    #[must_use]
    pub fn with_weight(mut self, weight: u16) -> Self {
        self.weight = weight;
        self
    }

    /// Set detailed font metrics.
    #[must_use]
    pub fn with_metrics(mut self, units_per_em: u16, ascender: i32, descender: i32) -> Self {
        self.units_per_em = Some(units_per_em);
        self.ascender = Some(ascender);
        self.descender = Some(descender);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precise_layout_new() {
        let layout = PreciseLayout::new_letter("sha256:abc123");
        assert_eq!(layout.presentation_type, "precise");
        assert_eq!(layout.target_format, "letter");
        assert_eq!(layout.page_size.width, "8.5in");
        assert_eq!(layout.page_size.height, "11in");
        assert_eq!(layout.content_hash, "sha256:abc123");
    }

    #[test]
    fn test_staleness_detection() {
        let layout = PreciseLayout::new_letter("sha256:abc123");
        assert!(!layout.is_stale("sha256:abc123"));
        assert!(layout.is_stale("sha256:xyz789"));
    }

    #[test]
    fn test_page_element_continuation() {
        let elem = PrecisePageElement::new("block-1", "1in", "2in", "6in", "3in")
            .continues();
        assert!(elem.continues);
        assert!(!elem.continuation);

        let next = PrecisePageElement::new("block-1", "1in", "1in", "6in", "1in")
            .continuation();
        assert!(!next.continues);
        assert!(next.continuation);
    }

    #[test]
    fn test_line_level_precision() {
        let lines = vec![
            LineInfo::new(1, "3in", "0.2in"),
            LineInfo::new(2, "3.25in", "0.2in"),
            LineInfo::new(3, "3.5in", "0.2in"),
        ];
        let elem = PrecisePageElement::new("block-5", "1in", "3in", "6.5in", "1.5in")
            .with_lines(lines);
        assert_eq!(elem.lines.len(), 3);
        assert_eq!(elem.lines[0].number, 1);
    }

    #[test]
    fn test_serialization() {
        let mut layout = PreciseLayout::new_letter("sha256:abc123");
        layout.add_page(
            PrecisePage::new(1)
                .with_element(PrecisePageElement::new("block-1", "1in", "1in", "6.5in", "0.5in"))
        );

        let json = serde_json::to_string_pretty(&layout).unwrap();
        assert!(json.contains("\"presentationType\": \"precise\""));
        assert!(json.contains("\"targetFormat\": \"letter\""));
        assert!(json.contains("\"blockId\": \"block-1\""));
    }

    #[test]
    fn test_page_template() {
        let template = PageTemplate::new()
            .with_margins(Margins::all("1.5in"))
            .with_footer(PageRegion::page_number_footer("10.5in"));

        assert_eq!(template.margins.top, "1.5in");
        assert!(template.footer.is_some());
        assert!(template.header.is_none());
    }

    #[test]
    fn test_font_metrics() {
        let metrics = FontMetrics::new("Times New Roman")
            .with_weight(700)
            .with_metrics(2048, 1825, -443);

        assert_eq!(metrics.family, "Times New Roman");
        assert_eq!(metrics.weight, 700);
        assert_eq!(metrics.units_per_em, Some(2048));
    }
}
