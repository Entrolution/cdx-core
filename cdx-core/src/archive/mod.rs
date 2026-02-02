//! Archive reading and writing for Codex documents.
//!
//! Codex documents are packaged as ZIP archives with the `.cdx` extension.
//! This module provides [`CdxReader`] and [`CdxWriter`] for working with these archives.
//!
//! # Reading Documents
//!
//! ```rust,ignore
//! use cdx_core::archive::CdxReader;
//!
//! let mut reader = CdxReader::open("document.cdx")?;
//! let manifest = reader.manifest();
//! let content = reader.read_file("content/document.json")?;
//! ```
//!
//! # Writing Documents
//!
//! ```rust,ignore
//! use cdx_core::archive::CdxWriter;
//!
//! let mut writer = CdxWriter::create("output.cdx")?;
//! writer.write_manifest(&manifest)?;
//! writer.write_file("content/document.json", &content)?;
//! writer.finish()?;
//! ```

mod reader;
mod writer;

pub use reader::CdxReader;
pub use writer::{CdxWriter, CompressionMethod};

/// Path to the manifest file within the archive.
pub const MANIFEST_PATH: &str = "manifest.json";

/// Path to the content file within the archive.
pub const CONTENT_PATH: &str = "content/document.json";

/// Path to the Dublin Core metadata file within the archive.
pub const DUBLIN_CORE_PATH: &str = "metadata/dublin-core.json";

/// Path to the signatures file within the archive.
pub const SIGNATURES_PATH: &str = "security/signatures.json";

/// Path to the encryption metadata file within the archive.
pub const ENCRYPTION_PATH: &str = "security/encryption.json";

/// Path to the phantom clusters file within the archive.
pub const PHANTOMS_PATH: &str = "phantoms/clusters.json";

/// Path to the academic numbering configuration file within the archive.
pub const ACADEMIC_NUMBERING_PATH: &str = "academic/numbering.json";

/// ZIP comment for Codex documents.
pub const ZIP_COMMENT: &str = "Codex Document Format v0.1";

/// Validate that a path is safe (no path traversal).
///
/// # Errors
///
/// Returns `PathTraversal` error if the path contains `..` segments or other unsafe patterns.
pub(crate) fn validate_path(path: &str) -> crate::Result<()> {
    // Check for path traversal attempts
    if path.contains("..") {
        return Err(crate::Error::PathTraversal {
            path: path.to_string(),
        });
    }

    // Check for absolute paths (should not start with /)
    if path.starts_with('/') {
        return Err(crate::Error::PathTraversal {
            path: path.to_string(),
        });
    }

    // Check for backslashes (Windows path separators)
    if path.contains('\\') {
        return Err(crate::Error::PathTraversal {
            path: path.to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_path_safe() {
        assert!(validate_path("manifest.json").is_ok());
        assert!(validate_path("content/document.json").is_ok());
        assert!(validate_path("assets/images/photo.png").is_ok());
    }

    #[test]
    fn test_validate_path_traversal() {
        assert!(validate_path("../secret").is_err());
        assert!(validate_path("foo/../bar").is_err());
        assert!(validate_path("foo/..").is_err());
    }

    #[test]
    fn test_validate_path_absolute() {
        assert!(validate_path("/etc/passwd").is_err());
        assert!(validate_path("/manifest.json").is_err());
    }

    #[test]
    fn test_validate_path_backslash() {
        assert!(validate_path("foo\\bar").is_err());
        assert!(validate_path("..\\secret").is_err());
    }
}
