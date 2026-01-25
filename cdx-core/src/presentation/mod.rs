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
mod responsive;
mod style;

pub use continuous::{Continuous, Section};
pub use paginated::{FlowElement, Margins, PageElement, PageSize, Paginated, Position};
pub use responsive::{Breakpoint, Responsive, ResponsiveDefaults, ResponsiveStyle};
pub use style::{Color, CssValue, FontWeight, Style, StyleMap, TextAlign};

/// Presentation type identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PresentationType {
    /// Fixed pages for print/PDF.
    Paginated,
    /// Vertical scroll for screens.
    Continuous,
    /// Adapts to viewport size.
    Responsive,
}

impl PresentationType {
    /// Get the type identifier string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Paginated => "paginated",
            Self::Continuous => "continuous",
            Self::Responsive => "responsive",
        }
    }
}

impl std::fmt::Display for PresentationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
