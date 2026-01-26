//! Presentation layer types.
//!
//! Presentation layers define how semantic content is rendered visually.
//! Codex supports three presentation types:
//!
//! - [`Paginated`] - Fixed pages for print/PDF
//! - [`Continuous`] - Vertical scroll for screens
//! - [`Responsive`] - Adapts to viewport size
//!
//! # Philosophy
//!
//! Content is authoritative; presentation is derived. The same content
//! can have multiple presentation layers for different contexts.

mod continuous;
mod paginated;
mod precise;
mod responsive;
mod style;

pub use continuous::{Continuous, Section};
pub use paginated::{FlowElement, Margins, PageElement, PageSize, Paginated, Position};
pub use precise::{
    FontMetrics, LineInfo, PageRegion, PageTemplate, PreciseLayout, PrecisePage,
    PrecisePageElement, PrecisePageSize,
};
pub use responsive::{Breakpoint, Responsive, ResponsiveDefaults, ResponsiveStyle};
pub use style::{Color, CssValue, FontWeight, Style, StyleMap, TextAlign};

/// Presentation type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl std::fmt::Display for PresentationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
