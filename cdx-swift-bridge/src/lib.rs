//! cdx-swift-bridge: UniFFI bindings for cdx-core
//!
//! This crate provides Swift-friendly wrappers around the cdx-core library,
//! exposing the Codex document format to macOS and iOS applications.

mod content;
mod document;
mod error;

pub use content::*;
pub use document::*;
pub use error::*;

#[cfg(test)]
mod tests;

use std::sync::Arc;

uniffi::setup_scaffolding!();

// Top-level functions exposed to Swift

/// Open a document from a file path.
#[uniffi::export]
pub fn open_document(path: String) -> Result<Arc<CdxDocument>, CdxError> {
    CdxDocument::open(&path)
}

/// Open a document from raw bytes.
#[uniffi::export]
pub fn open_document_from_bytes(data: Vec<u8>) -> Result<Arc<CdxDocument>, CdxError> {
    CdxDocument::from_bytes(data)
}

/// Create a new empty document.
#[uniffi::export]
pub fn create_document() -> Result<Arc<CdxDocument>, CdxError> {
    CdxDocument::new()
}

/// Create a new document with a title.
#[uniffi::export]
pub fn create_document_with_title(title: String) -> Result<Arc<CdxDocument>, CdxError> {
    CdxDocument::new_with_title(&title)
}
