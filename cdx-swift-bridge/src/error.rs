//! Error types for the Swift bridge.

use thiserror::Error;

/// Error type exposed to Swift via UniFFI.
#[derive(Debug, Error, uniffi::Error)]
pub enum CdxError {
    #[error("I/O error: {0}")]
    IoError(String),

    #[error("Invalid archive format")]
    InvalidArchive,

    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    #[error("Invalid content: {0}")]
    InvalidContent(String),

    #[error("Unsupported version: {0}")]
    UnsupportedVersion(String),

    #[error("Hash mismatch: {0}")]
    HashMismatch(String),

    #[error("Signature error: {0}")]
    SignatureError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid state transition: {0}")]
    InvalidState(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Signing failed: {0}")]
    SigningFailed(String),

    #[error("Cannot modify immutable document: {0}")]
    ImmutableDocument(String),

    #[error("Extension not available: {0}")]
    ExtensionNotAvailable(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Document ID mismatch: {0}")]
    DocumentIdMismatch(String),

    #[error("Path traversal detected: {0}")]
    PathTraversal(String),

    #[error("Unsupported hash algorithm: {0}")]
    UnsupportedHashAlgorithm(String),

    #[error("Invalid certificate: {0}")]
    InvalidCertificate(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

impl From<cdx_core::Error> for CdxError {
    fn from(err: cdx_core::Error) -> Self {
        match err {
            cdx_core::Error::Io(e) => CdxError::IoError(e.to_string()),
            cdx_core::Error::InvalidArchive(e) => {
                CdxError::IoError(format!("invalid archive: {e}"))
            }
            cdx_core::Error::Json(e) => CdxError::SerializationError(e.to_string()),
            cdx_core::Error::MissingFile { path } => CdxError::NotFound(path),
            cdx_core::Error::InvalidManifest { reason } => CdxError::InvalidManifest(reason),
            cdx_core::Error::UnsupportedVersion { version } => {
                CdxError::UnsupportedVersion(version)
            }
            cdx_core::Error::HashMismatch {
                path,
                expected,
                actual,
            } => CdxError::HashMismatch(format!("{path}: expected {expected}, got {actual}")),
            cdx_core::Error::DocumentIdMismatch { expected, actual } => {
                CdxError::DocumentIdMismatch(format!("expected {expected}, got {actual}"))
            }
            cdx_core::Error::InvalidStateTransition { from, to } => {
                CdxError::InvalidState(format!("cannot transition from {from:?} to {to:?}"))
            }
            cdx_core::Error::StateRequirementNotMet { state, requirement } => {
                CdxError::InvalidState(format!("{state:?} requires {requirement}"))
            }
            cdx_core::Error::PathTraversal { path } => CdxError::PathTraversal(path),
            cdx_core::Error::UnsupportedHashAlgorithm { algorithm } => {
                CdxError::UnsupportedHashAlgorithm(algorithm)
            }
            cdx_core::Error::InvalidHashFormat { value } => {
                CdxError::InvalidContent(format!("invalid hash format: {value}"))
            }
            cdx_core::Error::FileNotFound { path } => {
                CdxError::NotFound(path.display().to_string())
            }
            cdx_core::Error::InvalidCertificate { reason } => CdxError::InvalidCertificate(reason),
            cdx_core::Error::Network { message } => CdxError::NetworkError(message),
            cdx_core::Error::NotImplemented { feature } => CdxError::NotImplemented(feature),
            cdx_core::Error::ImmutableDocument { action, state } => {
                CdxError::ImmutableDocument(format!("cannot {action} in {state:?} state"))
            }
            cdx_core::Error::ExtensionNotAvailable { extension } => {
                CdxError::ExtensionNotAvailable(extension)
            }
            cdx_core::Error::ValidationFailed { reason } => CdxError::ValidationFailed(reason),
            cdx_core::Error::SignatureError { reason } => CdxError::SignatureError(reason),
            cdx_core::Error::EncryptionError { reason } => CdxError::EncryptionError(reason),
        }
    }
}

impl From<std::io::Error> for CdxError {
    fn from(err: std::io::Error) -> Self {
        CdxError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for CdxError {
    fn from(err: serde_json::Error) -> Self {
        CdxError::SerializationError(err.to_string())
    }
}
