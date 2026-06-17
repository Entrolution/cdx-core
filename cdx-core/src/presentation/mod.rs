//! Presentation layer types.
//!
//! Presentation layers define how semantic content is rendered visually.
//! CDX supports three presentation types:
//!
//! - [`Paginated`] - Fixed pages for print/PDF
//! - [`Continuous`] - Vertical scroll for screens
//! - [`Responsive`] - Adapts to viewport size
//!
//! # Philosophy
//!
//! Content is authoritative; presentation is derived. The same content
//! can have multiple presentation layers for different contexts.
//!
//! # Print Features
//!
//! For professional print workflows, see the [`print`] module which provides:
//!
//! - [`MasterPage`] - Reusable page templates with headers, footers, and backgrounds
//! - [`PrintSpecification`] - Bleed, crop marks, spot colors, and color space settings
//! - [`PdfXCompliance`] - PDF/X conformance metadata for prepress workflows

mod continuous;
mod layout;
mod notes;
mod paginated;
mod precise;
mod print;
mod responsive;
mod style;
mod toc;
mod typography;

pub use continuous::{Continuous, Section};
pub use layout::{ColumnLayout, GridArea, GridLayout};
pub use notes::{EndnotesConfig, FootnotePosition, FootnoteSeparator, FootnotesConfig};
pub use paginated::{FlowElement, Margins, PageElement, PageSize, Paginated, Position};
pub use precise::{
    FontMetrics, LineInfo, PageRegion, PageTemplate, PreciseLayout, PrecisePage,
    PrecisePageElement, PrecisePageSize,
};
pub use print::{
    AlternateColor, BleedBox, ColorSpace, CropMarkStyle, MasterElementType, MasterPage,
    MasterPageElement, MasterPageRegion, OutputIntent, PageBox, PdfXCompliance, PdfXLevel,
    PlaceholderDefinition, PlaceholderType, PrintSpecification, RegionAlignment, SpotColor,
    SpotColorType,
};
pub use responsive::{Breakpoint, Responsive, ResponsiveDefaults, ResponsiveStyle};
pub use style::{
    Color, CssValue, FontWeight, Scale, Style, StyleMap, TextAlign, Transform, TransformOrigin,
    WritingMode,
};
pub use toc::{TocConfig, TocLeaders};
pub use typography::{
    BaselineGrid, HyphenationConfig, LineNumbering, LineNumberingRestart, LineNumberingSide,
    TypographyConfig,
};

/// Presentation type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum PresentationType {
    /// Fixed pages for print/PDF (reactive).
    Paginated,
    /// Vertical scroll for screens (reactive).
    Continuous,
    /// Adapts to viewport size (reactive).
    Responsive,
    /// Exact coordinates for pixel-perfect reproduction.
    /// Required for FROZEN and PUBLISHED documents.
    Precise,
}

impl PresentationType {
    /// Get the type identifier string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Paginated => "paginated",
            Self::Continuous => "continuous",
            Self::Responsive => "responsive",
            Self::Precise => "precise",
        }
    }

    /// Check if this is a reactive (hint-based) presentation type.
    #[must_use]
    pub const fn is_reactive(&self) -> bool {
        matches!(self, Self::Paginated | Self::Continuous | Self::Responsive)
    }

    /// Check if this is a precise (coordinate-based) presentation type.
    #[must_use]
    pub const fn is_precise(&self) -> bool {
        matches!(self, Self::Precise)
    }
}
