//! Error types for cdx-core.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias using [`enum@Error`].
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

    /// Invalid certificate.
    #[error("invalid certificate: {reason}")]
    InvalidCertificate {
        /// Description of the certificate issue.
        reason: String,
    },

    /// Network operation failed.
    #[error("network error: {message}")]
    Network {
        /// Description of the network error.
        message: String,
    },

    /// Feature not implemented.
    #[error("not implemented: {feature}")]
    NotImplemented {
        /// Description of the unimplemented feature.
        feature: String,
    },

    /// Cannot modify document in immutable state.
    #[error("cannot {action} in {state:?} state")]
    ImmutableDocument {
        /// The action that was attempted.
        action: String,
        /// Current document state.
        state: crate::DocumentState,
    },

    /// Extension not found or not loaded.
    #[error("extension not available: {extension}")]
    ExtensionNotAvailable {
        /// Name of the missing extension.
        extension: String,
    },

    /// Content validation failed.
    #[error("content validation failed: {reason}")]
    ValidationFailed {
        /// Description of the validation failure.
        reason: String,
    },

    /// Signature operation failed.
    #[error("signature error: {reason}")]
    SignatureError {
        /// Description of the signature issue.
        reason: String,
    },

    /// Encryption operation failed.
    #[error("encryption error: {reason}")]
    EncryptionError {
        /// Description of the encryption issue.
        reason: String,
    },

    /// File exceeds the maximum allowed size (decompression bomb protection).
    #[error("file too large: {path} is {size} bytes (limit: {limit} bytes)")]
    FileTooLarge {
        /// Path of the oversized file.
        path: String,
        /// Actual or declared size in bytes.
        size: u64,
        /// Maximum allowed size in bytes.
        limit: u64,
    },

    /// Archive structure is invalid.
    #[error("invalid archive structure: {reason}")]
    InvalidArchiveStructure {
        /// Description of the structural issue.
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn display_invalid_archive() {
        let err = Error::InvalidArchive(zip::result::ZipError::FileNotFound);
        assert!(err.to_string().contains("invalid ZIP archive"));
    }

    #[test]
    fn display_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let err = Error::Json(json_err);
        assert!(err.to_string().starts_with("JSON error:"));
    }

    #[test]
    fn display_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let err = Error::Io(io_err);
        assert!(err.to_string().starts_with("I/O error:"));
    }

    #[test]
    fn display_missing_file() {
        let err = Error::MissingFile {
            path: "manifest.json".to_string(),
        };
        assert_eq!(err.to_string(), "missing required file: manifest.json");
    }

    #[test]
    fn display_invalid_manifest() {
        let err = Error::InvalidManifest {
            reason: "bad version".to_string(),
        };
        assert_eq!(err.to_string(), "invalid manifest: bad version");
    }

    #[test]
    fn display_unsupported_version() {
        let err = Error::UnsupportedVersion {
            version: "99.0".to_string(),
        };
        assert_eq!(err.to_string(), "unsupported Codex version: 99.0");
    }

    #[test]
    fn display_hash_mismatch() {
        let err = Error::HashMismatch {
            path: "content.json".to_string(),
            expected: "abc".to_string(),
            actual: "def".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "hash mismatch for content.json: expected abc, got def"
        );
    }

    #[test]
    fn display_document_id_mismatch() {
        let err = Error::DocumentIdMismatch {
            expected: "id1".to_string(),
            actual: "id2".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "document ID mismatch: expected id1, got id2"
        );
    }

    #[test]
    fn display_invalid_state_transition() {
        let err = Error::InvalidStateTransition {
            from: crate::DocumentState::Draft,
            to: crate::DocumentState::Frozen,
        };
        assert!(err.to_string().contains("invalid state transition"));
        assert!(err.to_string().contains("Draft"));
        assert!(err.to_string().contains("Frozen"));
    }

    #[test]
    fn display_state_requirement_not_met() {
        let err = Error::StateRequirementNotMet {
            state: crate::DocumentState::Frozen,
            requirement: "at least one signature".to_string(),
        };
        assert!(err.to_string().contains("Frozen"));
        assert!(err.to_string().contains("at least one signature"));
    }

    #[test]
    fn display_path_traversal() {
        let err = Error::PathTraversal {
            path: "../etc/passwd".to_string(),
        };
        assert_eq!(err.to_string(), "path traversal detected: ../etc/passwd");
    }

    #[test]
    fn display_unsupported_hash_algorithm() {
        let err = Error::UnsupportedHashAlgorithm {
            algorithm: "md5".to_string(),
        };
        assert_eq!(err.to_string(), "unsupported hash algorithm: md5");
    }

    #[test]
    fn display_invalid_hash_format() {
        let err = Error::InvalidHashFormat {
            value: "not-a-hash".to_string(),
        };
        assert_eq!(err.to_string(), "invalid hash format: not-a-hash");
    }

    #[test]
    fn display_file_not_found() {
        let err = Error::FileNotFound {
            path: PathBuf::from("/tmp/missing.cdx"),
        };
        assert!(err.to_string().contains("file not found"));
        assert!(err.to_string().contains("missing.cdx"));
    }

    #[test]
    fn display_invalid_certificate() {
        let err = Error::InvalidCertificate {
            reason: "expired".to_string(),
        };
        assert_eq!(err.to_string(), "invalid certificate: expired");
    }

    #[test]
    fn display_network() {
        let err = Error::Network {
            message: "timeout".to_string(),
        };
        assert_eq!(err.to_string(), "network error: timeout");
    }

    #[test]
    fn display_not_implemented() {
        let err = Error::NotImplemented {
            feature: "blockchain anchoring".to_string(),
        };
        assert_eq!(err.to_string(), "not implemented: blockchain anchoring");
    }

    #[test]
    fn display_immutable_document() {
        let err = Error::ImmutableDocument {
            action: "modify content".to_string(),
            state: crate::DocumentState::Frozen,
        };
        assert!(err.to_string().contains("cannot modify content"));
        assert!(err.to_string().contains("Frozen"));
    }

    #[test]
    fn display_extension_not_available() {
        let err = Error::ExtensionNotAvailable {
            extension: "forms".to_string(),
        };
        assert_eq!(err.to_string(), "extension not available: forms");
    }

    #[test]
    fn display_validation_failed() {
        let err = Error::ValidationFailed {
            reason: "empty content".to_string(),
        };
        assert_eq!(err.to_string(), "content validation failed: empty content");
    }

    #[test]
    fn display_signature_error() {
        let err = Error::SignatureError {
            reason: "invalid key".to_string(),
        };
        assert_eq!(err.to_string(), "signature error: invalid key");
    }

    #[test]
    fn display_encryption_error() {
        let err = Error::EncryptionError {
            reason: "wrong password".to_string(),
        };
        assert_eq!(err.to_string(), "encryption error: wrong password");
    }

    #[test]
    fn display_file_too_large() {
        let err = Error::FileTooLarge {
            path: "assets/huge.bin".to_string(),
            size: 512 * 1024 * 1024,
            limit: 256 * 1024 * 1024,
        };
        let msg = err.to_string();
        assert!(msg.contains("file too large"));
        assert!(msg.contains("assets/huge.bin"));
    }

    #[test]
    fn display_invalid_archive_structure() {
        let err = Error::InvalidArchiveStructure {
            reason: "manifest not first".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "invalid archive structure: manifest not first"
        );
    }
}
