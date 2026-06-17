//! Metadata types for CDX documents.
//!
//! This module provides types for Dublin Core metadata and extended metadata
//! as defined in the CDX specification.
//!
//! # Dublin Core
//!
//! Dublin Core is the required metadata standard for CDX documents:
//!
//! ```rust
//! use cdx_core::metadata::DublinCore;
//!
//! let dc = DublinCore::new("Annual Report 2025", "Jane Doe");
//! assert_eq!(dc.title(), "Annual Report 2025");
//! ```

mod dublin_core;

pub use dublin_core::{DublinCore, DublinCoreTerms, StringOrArray};
