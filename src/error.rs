//! Error types for cdx-core.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias using [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when working with Codex documents.
#[derive(Debug, Error)]
pub enum Error {
    /// The file is not a valid ZIP archive.
    #[error("invalid ZIP archive: {0}")]
    InvalidArchive(#[from] zip::result::ZipError),

    /// JSON parsing or serialization failed.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// I/O operation failed.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// A required file is missing from the archive.
    #[error("missing required file: {path}")]
    MissingFile {
        /// Path of the missing file.
        path: String,
    },

    /// The manifest is invalid.
    #[error("invalid manifest: {reason}")]
    InvalidManifest {
        /// Description of the validation failure.
        reason: String,
    },

    /// The document's Codex version is not supported.
    #[error("unsupported Codex version: {version}")]
    UnsupportedVersion {
        /// The unsupported version string.
        version: String,
    },

    /// Hash verification failed.
    #[error("hash mismatch for {path}: expected {expected}, got {actual}")]
    HashMismatch {
        /// Path of the file with mismatched hash.
        path: String,
        /// Expected hash value.
        expected: String,
        /// Actual computed hash value.
        actual: String,
    },

    /// Document ID verification failed.
    #[error("document ID mismatch: expected {expected}, got {actual}")]
    DocumentIdMismatch {
        /// Expected document ID.
        expected: String,
        /// Actual computed document ID.
        actual: String,
    },

    /// Invalid document state transition.
    #[error("invalid state transition from {from:?} to {to:?}")]
    InvalidStateTransition {
        /// Current state.
        from: crate::DocumentState,
        /// Attempted target state.
        to: crate::DocumentState,
    },

    /// State requirements not met.
    #[error("state {state:?} requires {requirement}")]
    StateRequirementNotMet {
        /// The document state with unmet requirements.
        state: crate::DocumentState,
        /// Description of the unmet requirement.
        requirement: String,
    },

    /// Path traversal attempt detected (security).
    #[error("path traversal detected: {path}")]
    PathTraversal {
        /// The suspicious path.
        path: String,
    },

    /// Hash algorithm is not supported.
    #[error("unsupported hash algorithm: {algorithm}")]
    UnsupportedHashAlgorithm {
        /// The unsupported algorithm identifier.
        algorithm: String,
    },

    /// Invalid hash format.
    #[error("invalid hash format: {value}")]
    InvalidHashFormat {
        /// The invalid hash string.
        value: String,
    },

    /// File not found.
    #[error("file not found: {}", path.display())]
    FileNotFound {
        /// Path to the missing file.
        path: PathBuf,
    },
}
