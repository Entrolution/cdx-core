//! Print-specific presentation features.
//!
//! This module provides types for professional print workflows including:
//!
//! - Master pages/templates for reusable page layouts
//! - Print specifications (bleed, crop marks, spot colors)
//! - PDF/X compliance metadata
//!
//! # Master Pages
//!
//! Master pages define reusable templates that can be applied to document pages:
//!
//! ```
//! use cdx_core::presentation::{MasterPage, MasterPageElement, PageSize, Margins};
//!
//! let master = MasterPage::new("default")
//!     .with_page_size(PageSize::a4())
//!     .with_margins(Margins::all("1in"))
//!     .with_header("Company Name")
//!     .with_footer("{pageNumber} of {totalPages}");
//! ```
//!
//! # Print Specifications
//!
//! Print specifications define bleeding area, crop marks, and color settings:
//!
//! ```
//! use cdx_core::presentation::{PrintSpecification, BleedBox, CropMarkStyle};
//!
//! let print_spec = PrintSpecification::default()
//!     .with_bleed(BleedBox::all("0.125in"))
//!     .with_crop_marks(CropMarkStyle::All);
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::paginated::{Margins, Orientation, PageSize, Position};
use super::style::Transform;

// =============================================================================
// Master Pages
// =============================================================================

/// A master page template that can be applied to document pages.
///
/// Master pages define reusable layouts including headers, footers,
/// background elements, and placeholders for dynamic content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MasterPage {
    /// Unique identifier for this master page.
    pub name: String,

    /// Human-readable display name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Page size (overrides document default if specified).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_size: Option<PageSize>,

    /// Page orientation (overrides document default if specified).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orientation: Option<Orientation>,

    /// Page margins.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub margins: Option<Margins>,

    /// Header region definition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub header: Option<MasterPageRegion>,

    /// Footer region definition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub footer: Option<MasterPageRegion>,

    /// Background elements (rendered behind content).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub background_elements: Vec<MasterPageElement>,

    /// Foreground elements (rendered above content).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub foreground_elements: Vec<MasterPageElement>,

    /// Placeholders for dynamic content insertion.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub placeholders: HashMap<String, PlaceholderDefinition>,

    /// Parent master page to inherit from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub based_on: Option<String>,
}

impl MasterPage {
    /// Create a new master page with the given name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            display_name: None,
            page_size: None,
            orientation: None,
            margins: None,
            header: None,
            footer: None,
            background_elements: Vec::new(),
            foreground_elements: Vec::new(),
            placeholders: HashMap::new(),
            based_on: None,
        }
    }

    /// Set the display name.
    #[must_use]
    pub fn with_display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Set the page size.
    #[must_use]
    pub fn with_page_size(mut self, size: PageSize) -> Self {
        self.page_size = Some(size);
        self
    }

    /// Set the page orientation.
    #[must_use]
    pub fn with_orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = Some(orientation);
        self
    }

    /// Set the margins.
    #[must_use]
    pub fn with_margins(mut self, margins: Margins) -> Self {
        self.margins = Some(margins);
        self
    }

    /// Add a simple text header.
    #[must_use]
    pub fn with_header(mut self, content: impl Into<String>) -> Self {
        self.header = Some(MasterPageRegion::text(content));
        self
    }

    /// Add a header region.
    #[must_use]
    pub fn with_header_region(mut self, region: MasterPageRegion) -> Self {
        self.header = Some(region);
        self
    }

    /// Add a simple text footer.
    #[must_use]
    pub fn with_footer(mut self, content: impl Into<String>) -> Self {
        self.footer = Some(MasterPageRegion::text(content));
        self
    }

    /// Add a footer region.
    #[must_use]
    pub fn with_footer_region(mut self, region: MasterPageRegion) -> Self {
        self.footer = Some(region);
        self
    }

    /// Add a background element.
    #[must_use]
    pub fn with_background_element(mut self, element: MasterPageElement) -> Self {
        self.background_elements.push(element);
        self
    }

    /// Add a foreground element.
    #[must_use]
    pub fn with_foreground_element(mut self, element: MasterPageElement) -> Self {
        self.foreground_elements.push(element);
        self
    }

    /// Set the parent master page.
    #[must_use]
    pub fn based_on(mut self, parent: impl Into<String>) -> Self {
        self.based_on = Some(parent.into());
        self
    }

    /// Create a standard "default" master page.
    #[must_use]
    pub fn default_master() -> Self {
        Self::new("default").with_display_name("Default")
    }

    /// Create a master page for odd (right-hand) pages.
    #[must_use]
    pub fn odd_page() -> Self {
        Self::new("odd").with_display_name("Odd Pages (Right)")
    }

    /// Create a master page for even (left-hand) pages.
    #[must_use]
    pub fn even_page() -> Self {
        Self::new("even").with_display_name("Even Pages (Left)")
    }

    /// Create a master page for title/cover pages.
    #[must_use]
    pub fn title_page() -> Self {
        Self::new("title").with_display_name("Title Page")
    }
}

/// A header or footer region in a master page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MasterPageRegion {
    /// Content template with placeholders like `{pageNumber}`, `{totalPages}`.
    pub content: String,

    /// Height of the region.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,

    /// Style name to apply.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,

    /// Alignment of content within the region.
    #[serde(default)]
    pub alignment: RegionAlignment,
}

impl MasterPageRegion {
    /// Create a region with text content.
    #[must_use]
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            height: None,
            style: None,
            alignment: RegionAlignment::default(),
        }
    }

    /// Create a page number footer.
    #[must_use]
    pub fn page_number() -> Self {
        Self::text("{pageNumber}").with_alignment(RegionAlignment::Center)
    }

    /// Create a "page X of Y" footer.
    #[must_use]
    pub fn page_number_of_total() -> Self {
        Self::text("{pageNumber} of {totalPages}").with_alignment(RegionAlignment::Center)
    }

    /// Set the height.
    #[must_use]
    pub fn with_height(mut self, height: impl Into<String>) -> Self {
        self.height = Some(height.into());
        self
    }

    /// Set the style.
    #[must_use]
    pub fn with_style(mut self, style: impl Into<String>) -> Self {
        self.style = Some(style.into());
        self
    }

    /// Set the alignment.
    #[must_use]
    pub fn with_alignment(mut self, alignment: RegionAlignment) -> Self {
        self.alignment = alignment;
        self
    }
}

/// Alignment of content within a region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RegionAlignment {
    /// Left-aligned.
    Left,
    /// Center-aligned.
    #[default]
    Center,
    /// Right-aligned.
    Right,
}

/// An element placed on a master page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MasterPageElement {
    /// Unique identifier for this element.
    pub id: String,

    /// Element type.
    #[serde(rename = "type")]
    pub element_type: MasterElementType,

    /// Position on the page.
    pub position: Position,

    /// Element-specific content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Style name to apply.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,

    /// 2D transform.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transform: Option<Transform>,

    /// Opacity (0.0 to 1.0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
}

impl MasterPageElement {
    /// Create a text element.
    #[must_use]
    pub fn text(id: impl Into<String>, content: impl Into<String>, position: Position) -> Self {
        Self {
            id: id.into(),
            element_type: MasterElementType::Text,
            position,
            content: Some(content.into()),
            style: None,
            transform: None,
            opacity: None,
        }
    }

    /// Create an image element.
    #[must_use]
    pub fn image(id: impl Into<String>, src: impl Into<String>, position: Position) -> Self {
        Self {
            id: id.into(),
            element_type: MasterElementType::Image,
            position,
            content: Some(src.into()),
            style: None,
            transform: None,
            opacity: None,
        }
    }

    /// Create a shape element.
    #[must_use]
    pub fn shape(id: impl Into<String>, shape_type: impl Into<String>, position: Position) -> Self {
        Self {
            id: id.into(),
            element_type: MasterElementType::Shape,
            position,
            content: Some(shape_type.into()),
            style: None,
            transform: None,
            opacity: None,
        }
    }

    /// Set the style.
    #[must_use]
    pub fn with_style(mut self, style: impl Into<String>) -> Self {
        self.style = Some(style.into());
        self
    }

    /// Set the opacity.
    #[must_use]
    pub fn with_opacity(mut self, opacity: f64) -> Self {
        self.opacity = Some(opacity);
        self
    }
}

/// Type of master page element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MasterElementType {
    /// Text content.
    Text,
    /// Image reference.
    Image,
    /// Shape (rectangle, line, etc.).
    Shape,
    /// Logo placeholder.
    Logo,
    /// Page number field.
    PageNumber,
}

/// Definition of a placeholder in a master page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaceholderDefinition {
    /// Placeholder type.
    #[serde(rename = "type")]
    pub placeholder_type: PlaceholderType,

    /// Position on the page.
    pub position: Position,

    /// Default content if not overridden.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_content: Option<String>,

    /// Style name to apply.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
}

/// Type of placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlaceholderType {
    /// Text placeholder.
    Text,
    /// Image placeholder.
    Image,
    /// Content flow area.
    Content,
    /// Page number.
    PageNumber,
    /// Total pages.
    TotalPages,
    /// Current date.
    Date,
    /// Document title.
    Title,
    /// Author name.
    Author,
}

// =============================================================================
// Print Specifications
// =============================================================================

/// Print specifications for professional output.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintSpecification {
    /// Bleed area beyond the trim edge.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bleed: Option<BleedBox>,

    /// Crop mark style.
    #[serde(default)]
    pub crop_marks: CropMarkStyle,

    /// Registration mark settings.
    #[serde(default)]
    pub registration_marks: bool,

    /// Color bars for press calibration.
    #[serde(default)]
    pub color_bars: bool,

    /// Page information (file name, date, etc.).
    #[serde(default)]
    pub page_information: bool,

    /// Trim box dimensions (final page size after cutting).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trim_box: Option<PageBox>,

    /// Media box dimensions (physical media size).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_box: Option<PageBox>,

    /// Art box dimensions (intended content area).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub art_box: Option<PageBox>,

    /// Spot colors used in the document.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub spot_colors: Vec<SpotColor>,

    /// Default color space for the document.
    #[serde(default)]
    pub color_space: ColorSpace,
}

impl Default for PrintSpecification {
    fn default() -> Self {
        Self {
            bleed: None,
            crop_marks: CropMarkStyle::None,
            registration_marks: false,
            color_bars: false,
            page_information: false,
            trim_box: None,
            media_box: None,
            art_box: None,
            spot_colors: Vec::new(),
            color_space: ColorSpace::default(),
        }
    }
}

impl PrintSpecification {
    /// Set the bleed area.
    #[must_use]
    pub fn with_bleed(mut self, bleed: BleedBox) -> Self {
        self.bleed = Some(bleed);
        self
    }

    /// Set the crop mark style.
    #[must_use]
    pub fn with_crop_marks(mut self, style: CropMarkStyle) -> Self {
        self.crop_marks = style;
        self
    }

    /// Enable registration marks.
    #[must_use]
    pub fn with_registration_marks(mut self) -> Self {
        self.registration_marks = true;
        self
    }

    /// Enable color bars.
    #[must_use]
    pub fn with_color_bars(mut self) -> Self {
        self.color_bars = true;
        self
    }

    /// Enable page information.
    #[must_use]
    pub fn with_page_information(mut self) -> Self {
        self.page_information = true;
        self
    }

    /// Set the color space.
    #[must_use]
    pub fn with_color_space(mut self, color_space: ColorSpace) -> Self {
        self.color_space = color_space;
        self
    }

    /// Add a spot color.
    #[must_use]
    pub fn with_spot_color(mut self, spot_color: SpotColor) -> Self {
        self.spot_colors.push(spot_color);
        self
    }

    /// Create a standard commercial print specification.
    #[must_use]
    pub fn commercial_print() -> Self {
        Self::default()
            .with_bleed(BleedBox::all("0.125in"))
            .with_crop_marks(CropMarkStyle::All)
            .with_registration_marks()
            .with_color_bars()
            .with_color_space(ColorSpace::Cmyk)
    }
}

/// Bleed area beyond the trim edge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BleedBox {
    /// Top bleed.
    pub top: String,
    /// Right bleed.
    pub right: String,
    /// Bottom bleed.
    pub bottom: String,
    /// Left bleed.
    pub left: String,
}

impl BleedBox {
    /// Create bleed with all sides equal.
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

    /// Standard commercial print bleed (0.125 inch / 3mm).
    #[must_use]
    pub fn standard() -> Self {
        Self::all("0.125in")
    }

    /// Create bleed with individual values.
    #[must_use]
    pub fn new(
        top: impl Into<String>,
        right: impl Into<String>,
        bottom: impl Into<String>,
        left: impl Into<String>,
    ) -> Self {
        Self {
            top: top.into(),
            right: right.into(),
            bottom: bottom.into(),
            left: left.into(),
        }
    }
}

/// Page box dimensions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageBox {
    /// Box width.
    pub width: String,
    /// Box height.
    pub height: String,
}

impl PageBox {
    /// Create a page box.
    #[must_use]
    pub fn new(width: impl Into<String>, height: impl Into<String>) -> Self {
        Self {
            width: width.into(),
            height: height.into(),
        }
    }
}

/// Crop mark style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CropMarkStyle {
    /// No crop marks.
    #[default]
    None,
    /// Trim marks at corners only.
    TrimMarks,
    /// Center marks on each edge.
    CenterMarks,
    /// Both trim and center marks.
    All,
}

/// Spot color definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotColor {
    /// Spot color name (e.g., "PANTONE 185 C").
    pub name: String,

    /// Color type.
    #[serde(rename = "type")]
    pub color_type: SpotColorType,

    /// Alternate color for display (typically CMYK or RGB).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alternate: Option<AlternateColor>,

    /// Tint percentage (0-100).
    #[serde(default = "default_tint")]
    pub tint: f64,
}

fn default_tint() -> f64 {
    100.0
}

impl SpotColor {
    /// Create a Pantone spot color.
    #[must_use]
    pub fn pantone(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            color_type: SpotColorType::Pantone,
            alternate: None,
            tint: 100.0,
        }
    }

    /// Create a custom spot color.
    #[must_use]
    pub fn custom(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            color_type: SpotColorType::Custom,
            alternate: None,
            tint: 100.0,
        }
    }

    /// Set an alternate CMYK color.
    #[must_use]
    pub fn with_cmyk_alternate(mut self, c: f64, m: f64, y: f64, k: f64) -> Self {
        self.alternate = Some(AlternateColor::Cmyk { c, m, y, k });
        self
    }

    /// Set an alternate RGB color.
    #[must_use]
    pub fn with_rgb_alternate(mut self, r: u8, g: u8, b: u8) -> Self {
        self.alternate = Some(AlternateColor::Rgb { r, g, b });
        self
    }

    /// Set the tint percentage.
    #[must_use]
    pub fn with_tint(mut self, tint: f64) -> Self {
        self.tint = tint;
        self
    }
}

/// Type of spot color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpotColorType {
    /// Pantone color.
    Pantone,
    /// Custom spot color.
    Custom,
    /// Metallic color.
    Metallic,
    /// Fluorescent color.
    Fluorescent,
}

/// Alternate color representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum AlternateColor {
    /// CMYK color (0-100 for each component).
    Cmyk {
        /// Cyan component (0-100).
        c: f64,
        /// Magenta component (0-100).
        m: f64,
        /// Yellow component (0-100).
        y: f64,
        /// Black (Key) component (0-100).
        k: f64,
    },
    /// RGB color (0-255 for each component).
    Rgb {
        /// Red component (0-255).
        r: u8,
        /// Green component (0-255).
        g: u8,
        /// Blue component (0-255).
        b: u8,
    },
    /// Lab color.
    Lab {
        /// Lightness component (0-100).
        l: f64,
        /// a* component (green-red axis).
        a: f64,
        /// b* component (blue-yellow axis).
        b: f64,
    },
}

/// Color space for the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorSpace {
    /// RGB color space (screen display).
    #[default]
    Rgb,
    /// CMYK color space (commercial print).
    Cmyk,
    /// Grayscale.
    Grayscale,
    /// Device-independent Lab color space.
    Lab,
}

// =============================================================================
// PDF/X Compliance
// =============================================================================

/// PDF/X compliance metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfXCompliance {
    /// PDF/X conformance level.
    pub level: PdfXLevel,

    /// Output intent specification.
    pub output_intent: OutputIntent,

    /// Whether all fonts are embedded.
    #[serde(default = "default_true")]
    pub fonts_embedded: bool,

    /// Whether transparency is flattened.
    #[serde(default)]
    pub transparency_flattened: bool,

    /// ICC color profile reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icc_profile: Option<String>,

    /// Additional compliance notes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

fn default_true() -> bool {
    true
}

impl PdfXCompliance {
    /// Create PDF/X-1a:2001 compliance specification.
    #[must_use]
    pub fn x1a_2001() -> Self {
        Self {
            level: PdfXLevel::X1a2001,
            output_intent: OutputIntent::swop(),
            fonts_embedded: true,
            transparency_flattened: true,
            icc_profile: None,
            notes: None,
        }
    }

    /// Create PDF/X-3:2002 compliance specification.
    #[must_use]
    pub fn x3_2002() -> Self {
        Self {
            level: PdfXLevel::X32002,
            output_intent: OutputIntent::default(),
            fonts_embedded: true,
            transparency_flattened: false,
            icc_profile: None,
            notes: None,
        }
    }

    /// Create PDF/X-4 compliance specification.
    #[must_use]
    pub fn x4() -> Self {
        Self {
            level: PdfXLevel::X4,
            output_intent: OutputIntent::default(),
            fonts_embedded: true,
            transparency_flattened: false,
            icc_profile: None,
            notes: None,
        }
    }

    /// Set the ICC profile reference.
    #[must_use]
    pub fn with_icc_profile(mut self, profile: impl Into<String>) -> Self {
        self.icc_profile = Some(profile.into());
        self
    }

    /// Set the output intent.
    #[must_use]
    pub fn with_output_intent(mut self, intent: OutputIntent) -> Self {
        self.output_intent = intent;
        self
    }
}

/// PDF/X conformance level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
pub enum PdfXLevel {
    /// PDF/X-1a:2001 - CMYK/spot only, no transparency.
    #[serde(rename = "PDF/X-1a:2001")]
    #[strum(serialize = "PDF/X-1a:2001")]
    X1a2001,

    /// PDF/X-1a:2003 - Updated PDF/X-1a.
    #[serde(rename = "PDF/X-1a:2003")]
    #[strum(serialize = "PDF/X-1a:2003")]
    X1a2003,

    /// PDF/X-3:2002 - Allows RGB and device-independent color.
    #[serde(rename = "PDF/X-3:2002")]
    #[strum(serialize = "PDF/X-3:2002")]
    X32002,

    /// PDF/X-3:2003 - Updated PDF/X-3.
    #[serde(rename = "PDF/X-3:2003")]
    #[strum(serialize = "PDF/X-3:2003")]
    X32003,

    /// PDF/X-4 - Supports transparency, layers, and OpenType fonts.
    #[serde(rename = "PDF/X-4")]
    #[strum(serialize = "PDF/X-4")]
    X4,

    /// PDF/X-4p - PDF/X-4 with external ICC profile reference.
    #[serde(rename = "PDF/X-4p")]
    #[strum(serialize = "PDF/X-4p")]
    X4p,

    /// PDF/X-5g - For multi-file workflows with external graphics.
    #[serde(rename = "PDF/X-5g")]
    #[strum(serialize = "PDF/X-5g")]
    X5g,

    /// PDF/X-5pg - Combines X-4p and X-5g features.
    #[serde(rename = "PDF/X-5pg")]
    #[strum(serialize = "PDF/X-5pg")]
    X5pg,

    /// PDF/X-6 - Latest standard with expanded features.
    #[serde(rename = "PDF/X-6")]
    #[strum(serialize = "PDF/X-6")]
    X6,
}

/// Output intent specification for PDF/X.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputIntent {
    /// Output condition identifier (e.g., "FOGRA39").
    pub output_condition_identifier: String,

    /// Human-readable output condition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_condition: Option<String>,

    /// Registry name (e.g., `http://www.color.org`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registry_name: Option<String>,

    /// Additional information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub info: Option<String>,
}

impl Default for OutputIntent {
    fn default() -> Self {
        Self {
            output_condition_identifier: "sRGB".to_string(),
            output_condition: Some("sRGB IEC61966-2.1".to_string()),
            registry_name: Some("http://www.color.org".to_string()),
            info: None,
        }
    }
}

impl OutputIntent {
    /// Create a SWOP output intent (US web coated).
    #[must_use]
    pub fn swop() -> Self {
        Self {
            output_condition_identifier: "CGATS TR 001".to_string(),
            output_condition: Some("SWOP (Publication) Grade 1 Paper".to_string()),
            registry_name: Some("http://www.color.org".to_string()),
            info: None,
        }
    }

    /// Create a FOGRA39 output intent (European coated offset).
    #[must_use]
    pub fn fogra39() -> Self {
        Self {
            output_condition_identifier: "FOGRA39".to_string(),
            output_condition: Some("Coated FOGRA39 (ISO 12647-2:2004)".to_string()),
            registry_name: Some("http://www.color.org".to_string()),
            info: None,
        }
    }

    /// Create a `GRACoL` output intent (US commercial printing).
    #[must_use]
    pub fn gracol() -> Self {
        Self {
            output_condition_identifier: "CGATS TR 006".to_string(),
            output_condition: Some("GRACoL 2006 (Coated #1)".to_string()),
            registry_name: Some("http://www.color.org".to_string()),
            info: None,
        }
    }

    /// Create a custom output intent.
    #[must_use]
    pub fn custom(identifier: impl Into<String>) -> Self {
        Self {
            output_condition_identifier: identifier.into(),
            output_condition: None,
            registry_name: None,
            info: None,
        }
    }

    /// Set the output condition description.
    #[must_use]
    pub fn with_condition(mut self, condition: impl Into<String>) -> Self {
        self.output_condition = Some(condition.into());
        self
    }

    /// Set the registry name.
    #[must_use]
    pub fn with_registry(mut self, registry: impl Into<String>) -> Self {
        self.registry_name = Some(registry.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_page_creation() {
        let master = MasterPage::new("default")
            .with_display_name("Default Page")
            .with_page_size(PageSize::a4())
            .with_margins(Margins::all("1in"))
            .with_header("Document Title")
            .with_footer("{pageNumber} of {totalPages}");

        assert_eq!(master.name, "default");
        assert_eq!(master.display_name, Some("Default Page".to_string()));
        assert!(master.header.is_some());
        assert!(master.footer.is_some());
    }

    #[test]
    fn test_master_page_serialization() {
        let master = MasterPage::new("test")
            .with_header("Header")
            .with_footer("Footer");

        let json = serde_json::to_string_pretty(&master).unwrap();
        assert!(json.contains("\"name\": \"test\""));

        let deserialized: MasterPage = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "test");
    }

    #[test]
    fn test_print_specification() {
        let spec = PrintSpecification::commercial_print();

        assert!(spec.bleed.is_some());
        assert_eq!(spec.crop_marks, CropMarkStyle::All);
        assert!(spec.registration_marks);
        assert!(spec.color_bars);
        assert_eq!(spec.color_space, ColorSpace::Cmyk);
    }

    #[test]
    fn test_bleed_box() {
        let bleed = BleedBox::standard();
        assert_eq!(bleed.top, "0.125in");
        assert_eq!(bleed.right, "0.125in");
        assert_eq!(bleed.bottom, "0.125in");
        assert_eq!(bleed.left, "0.125in");
    }

    #[test]
    fn test_spot_color() {
        let color = SpotColor::pantone("PANTONE 185 C")
            .with_cmyk_alternate(0.0, 91.0, 76.0, 0.0)
            .with_tint(100.0);

        assert_eq!(color.name, "PANTONE 185 C");
        assert_eq!(color.color_type, SpotColorType::Pantone);
        assert!(color.alternate.is_some());
    }

    #[test]
    fn test_pdfx_compliance() {
        let compliance = PdfXCompliance::x4()
            .with_icc_profile("sRGB IEC61966-2.1")
            .with_output_intent(OutputIntent::fogra39());

        assert_eq!(compliance.level, PdfXLevel::X4);
        assert!(compliance.fonts_embedded);
        assert!(!compliance.transparency_flattened);
        assert_eq!(
            compliance.output_intent.output_condition_identifier,
            "FOGRA39"
        );
    }

    #[test]
    fn test_pdfx_level_display() {
        assert_eq!(PdfXLevel::X1a2001.to_string(), "PDF/X-1a:2001");
        assert_eq!(PdfXLevel::X4.to_string(), "PDF/X-4");
    }

    #[test]
    fn test_output_intent_presets() {
        let swop = OutputIntent::swop();
        assert_eq!(swop.output_condition_identifier, "CGATS TR 001");

        let fogra = OutputIntent::fogra39();
        assert_eq!(fogra.output_condition_identifier, "FOGRA39");

        let gracol = OutputIntent::gracol();
        assert_eq!(gracol.output_condition_identifier, "CGATS TR 006");
    }

    #[test]
    fn test_master_page_presets() {
        let default = MasterPage::default_master();
        assert_eq!(default.name, "default");

        let odd = MasterPage::odd_page();
        assert_eq!(odd.name, "odd");

        let even = MasterPage::even_page();
        assert_eq!(even.name, "even");

        let title = MasterPage::title_page();
        assert_eq!(title.name, "title");
    }

    #[test]
    fn test_region_alignment() {
        let region = MasterPageRegion::page_number_of_total();
        assert_eq!(region.alignment, RegionAlignment::Center);
        assert_eq!(region.content, "{pageNumber} of {totalPages}");
    }

    #[test]
    fn test_print_spec_serialization() {
        let spec = PrintSpecification::commercial_print();
        let json = serde_json::to_string_pretty(&spec).unwrap();

        assert!(json.contains("\"cropMarks\": \"all\""));
        assert!(json.contains("\"colorSpace\": \"cmyk\""));

        let deserialized: PrintSpecification = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.crop_marks, CropMarkStyle::All);
    }
}
