//! # cdx-core
//!
//! Core library for reading, writing, and validating Codex Document Format (`.cdx`) files.
//!
//! ## Overview
//!
//! Codex is an open document format designed for semantic content, verifiable integrity,
//! and machine readability. This library provides the foundational capabilities for working
//! with Codex documents in Rust.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use cdx_core::{Document, DocumentState};
//!
//! // Open an existing document
//! let doc = Document::open("example.cdx")?;
//! println!("State: {:?}", doc.state());
//!
//! // Verify document integrity
//! doc.verify()?;
//! ```
//!
//! ## Features
//!
//! - `zstd` (default): Zstandard compression support
//! - `wasm`: WASM compilation support

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod anchor;
pub mod archive;
pub mod asset;
pub mod content;
mod document;
mod error;
pub mod extensions;
mod hash;
mod manifest;
pub mod metadata;
pub mod presentation;
pub mod provenance;
#[cfg(feature = "signatures")]
#[cfg_attr(docsrs, doc(cfg(feature = "signatures")))]
pub mod security;
mod state;
pub mod validation;

pub use document::{Document, DocumentBuilder, VerificationReport};
pub use error::{Error, Result};
pub use hash::{DocumentId, HashAlgorithm, Hasher};
pub use manifest::{
    AssetManifest, ContentRef, Extension, FileRef, Lineage, Manifest, Metadata, PresentationRef,
    SecurityRef,
};
pub use state::DocumentState;

/// Codex specification version implemented by this library.
pub const SPEC_VERSION: &str = "0.1";

/// Default file extension for Codex documents.
pub const FILE_EXTENSION: &str = "cdx";

/// MIME type for Codex documents (JSON format).
pub const MIME_TYPE: &str = "application/vnd.codex+json";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(SPEC_VERSION, "0.1");
        assert_eq!(FILE_EXTENSION, "cdx");
        assert_eq!(MIME_TYPE, "application/vnd.codex+json");
    }
}
