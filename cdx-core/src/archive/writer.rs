//! Archive writer for CDX documents.

use std::fs::File;
use std::io::{BufWriter, Cursor, Seek, Write};
use std::path::Path;

use zip::write::FileOptions;
use zip::ZipWriter;

use crate::{Manifest, Result};

use super::{validate_path, PHANTOMS_PATH, ZIP_COMMENT};

/// Compression method for files in the archive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompressionMethod {
    /// Store without compression (for pre-compressed content like images).
    Stored,
    /// Deflate compression (widely compatible, required support).
    #[default]
    Deflate,
    /// Zstandard compression (better ratio, optional support).
    #[cfg(feature = "zstd")]
    Zstd,
}

impl CompressionMethod {
    fn to_zip_method(self) -> zip::CompressionMethod {
        match self {
            Self::Stored => zip::CompressionMethod::Stored,
            Self::Deflate => zip::CompressionMethod::Deflated,
            #[cfg(feature = "zstd")]
            Self::Zstd => zip::CompressionMethod::Zstd,
        }
    }
}

/// Writer for creating CDX document archives.
///
/// `CdxWriter` creates properly formatted `.cdx` files, ensuring the manifest
/// is written first and all required structure is maintained.
///
/// # Example
///
/// ```rust,ignore
/// use cdx_core::archive::{CdxWriter, CompressionMethod};
///
/// let mut writer = CdxWriter::create("output.cdx")?;
///
/// writer.write_manifest(&manifest)?;
/// writer.write_file("content/document.json", &content, CompressionMethod::Deflate)?;
/// writer.write_file("metadata/dublin-core.json", &metadata, CompressionMethod::Deflate)?;
///
/// writer.finish()?;
/// ```
pub struct CdxWriter<W: Write + Seek> {
    zip: ZipWriter<W>,
    manifest_written: bool,
    files_written: Vec<String>,
}

impl CdxWriter<BufWriter<File>> {
    /// Create a new CDX document at the given file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created.
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::create(path.as_ref()).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                crate::Error::FileNotFound {
                    path: path.as_ref().to_path_buf(),
                }
            } else {
                crate::Error::Io(e)
            }
        })?;
        let writer = BufWriter::new(file);
        Self::new(writer)
    }
}

impl CdxWriter<Cursor<Vec<u8>>> {
    /// Create a new CDX document in memory.
    ///
    /// # Panics
    ///
    /// This function will not panic in practice, as initializing
    /// a `ZipWriter` on an in-memory buffer cannot fail.
    #[must_use]
    pub fn in_memory() -> Self {
        let cursor = Cursor::new(Vec::new());
        // This cannot fail for an in-memory buffer
        Self::new(cursor).expect("in-memory writer should not fail")
    }
}

impl<W: Write + Seek> CdxWriter<W> {
    /// Create a new writer wrapping any `Write + Seek` destination.
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails.
    pub fn new(writer: W) -> Result<Self> {
        let mut zip = ZipWriter::new(writer);
        zip.set_comment(ZIP_COMMENT);

        Ok(Self {
            zip,
            manifest_written: false,
            files_written: Vec::new(),
        })
    }

    /// Write the manifest to the archive.
    ///
    /// This must be called before writing any other files, as the manifest
    /// must be the first file in the archive per the CDX specification.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Writing fails
    /// - The manifest has already been written
    pub fn write_manifest(&mut self, manifest: &Manifest) -> Result<()> {
        if self.manifest_written {
            return Err(crate::Error::InvalidManifest {
                reason: "manifest already written".to_string(),
            });
        }

        if !self.files_written.is_empty() {
            return Err(crate::Error::InvalidManifest {
                reason: "manifest must be the first file in the archive".to_string(),
            });
        }

        let json = serde_json::to_vec_pretty(manifest)?;
        self.write_file_internal(super::MANIFEST_PATH, &json, CompressionMethod::Deflate)?;
        self.manifest_written = true;

        Ok(())
    }

    /// Write a file to the archive.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The manifest has not been written yet
    /// - The path contains traversal patterns (security check)
    /// - Writing fails
    /// - A file with the same path already exists
    pub fn write_file(
        &mut self,
        path: &str,
        data: &[u8],
        compression: CompressionMethod,
    ) -> Result<()> {
        if !self.manifest_written {
            return Err(crate::Error::InvalidManifest {
                reason: "manifest must be written before other files".to_string(),
            });
        }

        validate_path(path)?;

        if self.files_written.contains(&path.to_string()) {
            return Err(crate::Error::InvalidManifest {
                reason: format!("file already exists: {path}"),
            });
        }

        self.write_file_internal(path, data, compression)
    }

    /// Internal file writing without manifest check (for manifest itself).
    fn write_file_internal(
        &mut self,
        path: &str,
        data: &[u8],
        compression: CompressionMethod,
    ) -> Result<()> {
        let options = FileOptions::<()>::default()
            .compression_method(compression.to_zip_method())
            .unix_permissions(0o644);

        self.zip.start_file(path, options)?;
        self.zip.write_all(data)?;
        self.files_written.push(path.to_string());

        Ok(())
    }

    /// Write a file with automatic hash computation.
    ///
    /// Returns the computed hash for inclusion in the manifest.
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub fn write_file_hashed(
        &mut self,
        path: &str,
        data: &[u8],
        compression: CompressionMethod,
        algorithm: crate::HashAlgorithm,
    ) -> Result<crate::DocumentId> {
        let hash = crate::Hasher::hash(algorithm, data);
        self.write_file(path, data, compression)?;
        Ok(hash)
    }

    /// Write phantom clusters to the archive.
    ///
    /// Phantom clusters are stored at `phantoms/clusters.json` and are
    /// not included in the content hash since they exist outside the
    /// core content boundary.
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub fn write_phantoms(&mut self, phantoms: &crate::extensions::PhantomClusters) -> Result<()> {
        let json = serde_json::to_vec_pretty(phantoms)?;
        self.write_file(PHANTOMS_PATH, &json, CompressionMethod::Deflate)
    }

    /// Start a directory in the archive.
    ///
    /// This is optional, as ZIP archives create directories implicitly,
    /// but can be useful for clarity.
    ///
    /// # Errors
    ///
    /// Returns an error if adding the directory fails.
    pub fn add_directory(&mut self, path: &str) -> Result<()> {
        validate_path(path)?;

        let dir_path = if path.ends_with('/') {
            path.to_string()
        } else {
            format!("{path}/")
        };

        let options =
            FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored);

        self.zip.add_directory(&dir_path, options)?;

        Ok(())
    }

    /// Check if the manifest has been written.
    #[must_use]
    pub fn manifest_written(&self) -> bool {
        self.manifest_written
    }

    /// Get the list of files that have been written.
    #[must_use]
    pub fn files_written(&self) -> &[String] {
        &self.files_written
    }

    /// Finish writing and close the archive.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The manifest was not written
    /// - Finalizing the archive fails
    pub fn finish(self) -> Result<W> {
        if !self.manifest_written {
            return Err(crate::Error::InvalidManifest {
                reason: "manifest must be written before finishing".to_string(),
            });
        }

        let writer = self.zip.finish()?;
        Ok(writer)
    }

    /// Abort writing and return the underlying writer without finalizing.
    ///
    /// The resulting archive will be invalid.
    ///
    /// # Panics
    ///
    /// Panics if the ZIP finalization fails, which should not happen
    /// for valid writer implementations.
    #[must_use]
    pub fn abort(self) -> W {
        self.zip.finish().unwrap_or_else(|_| {
            // If finish fails, we've already aborted, which is fine
            panic!("abort should not fail")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archive::{CONTENT_PATH, DUBLIN_CORE_PATH};
    use crate::{ContentRef, DocumentId, Metadata};

    fn create_test_manifest() -> Manifest {
        let content = ContentRef {
            path: CONTENT_PATH.to_string(),
            hash: DocumentId::pending(),
            compression: None,
            merkle_root: None,
            block_count: None,
        };
        let metadata = Metadata {
            dublin_core: DUBLIN_CORE_PATH.to_string(),
            custom: None,
        };
        Manifest::new(content, metadata)
    }

    #[test]
    fn test_writer_in_memory() {
        let mut writer = CdxWriter::in_memory();
        let manifest = create_test_manifest();

        writer.write_manifest(&manifest).unwrap();
        writer
            .write_file(
                CONTENT_PATH,
                br#"{"version":"0.1","blocks":[]}"#,
                CompressionMethod::Deflate,
            )
            .unwrap();
        writer
            .write_file(
                DUBLIN_CORE_PATH,
                br#"{"title":"Test"}"#,
                CompressionMethod::Deflate,
            )
            .unwrap();

        let result = writer.finish().unwrap();
        assert!(!result.into_inner().is_empty());
    }

    #[test]
    fn test_writer_manifest_first() {
        let mut writer = CdxWriter::in_memory();

        // Try to write a file before manifest
        let result = writer.write_file(CONTENT_PATH, b"test", CompressionMethod::Deflate);
        assert!(result.is_err());
    }

    #[test]
    fn test_writer_manifest_once() {
        let mut writer = CdxWriter::in_memory();
        let manifest = create_test_manifest();

        writer.write_manifest(&manifest).unwrap();

        // Try to write manifest again
        let result = writer.write_manifest(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_writer_path_traversal_rejected() {
        let mut writer = CdxWriter::in_memory();
        let manifest = create_test_manifest();
        writer.write_manifest(&manifest).unwrap();

        let result = writer.write_file("../secret", b"data", CompressionMethod::Deflate);
        assert!(result.is_err());
    }

    #[test]
    fn test_writer_duplicate_file_rejected() {
        let mut writer = CdxWriter::in_memory();
        let manifest = create_test_manifest();
        writer.write_manifest(&manifest).unwrap();

        writer
            .write_file(CONTENT_PATH, b"first", CompressionMethod::Deflate)
            .unwrap();

        let result = writer.write_file(CONTENT_PATH, b"second", CompressionMethod::Deflate);
        assert!(result.is_err());
    }

    #[test]
    fn test_writer_finish_requires_manifest() {
        let writer = CdxWriter::in_memory();
        let result = writer.finish();
        assert!(result.is_err());
    }

    #[test]
    fn test_writer_compression_stored() {
        let mut writer = CdxWriter::in_memory();
        let manifest = create_test_manifest();
        writer.write_manifest(&manifest).unwrap();

        writer
            .write_file(CONTENT_PATH, b"test data", CompressionMethod::Stored)
            .unwrap();

        assert!(writer.files_written().contains(&CONTENT_PATH.to_string()));
    }

    #[test]
    fn test_writer_hashed() {
        let mut writer = CdxWriter::in_memory();
        let manifest = create_test_manifest();
        writer.write_manifest(&manifest).unwrap();

        let data = b"test content";
        let hash = writer
            .write_file_hashed(
                CONTENT_PATH,
                data,
                CompressionMethod::Deflate,
                crate::HashAlgorithm::Sha256,
            )
            .unwrap();

        assert!(!hash.is_pending());
        assert_eq!(hash.algorithm(), crate::HashAlgorithm::Sha256);
    }
}
