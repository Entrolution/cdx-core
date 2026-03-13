//! Archive reader for Codex documents.

use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek};
use std::path::Path;

use zip::ZipArchive;

use crate::{Error, HashAlgorithm, Hasher, Manifest, Result};

use super::{validate_path, CONTENT_PATH, DUBLIN_CORE_PATH, MANIFEST_PATH, PHANTOMS_PATH};

/// Reader for Codex document archives.
///
/// `CdxReader` opens and validates `.cdx` files, providing access to their contents.
/// The reader validates the archive structure on creation and provides lazy access
/// to individual files.
///
/// # Example
///
/// ```rust,ignore
/// use cdx_core::archive::CdxReader;
///
/// let mut reader = CdxReader::open("document.cdx")?;
///
/// // Access the manifest
/// let manifest = reader.manifest();
/// println!("Document state: {:?}", manifest.state);
///
/// // Read a file from the archive
/// let content = reader.read_file("content/document.json")?;
/// ```
pub struct CdxReader<R: Read + Seek> {
    archive: ZipArchive<R>,
    manifest: Manifest,
}

impl CdxReader<BufReader<File>> {
    /// Open a Codex document from a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be opened
    /// - The file is not a valid ZIP archive
    /// - Required files are missing
    /// - The manifest is invalid
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref()).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                Error::FileNotFound {
                    path: path.as_ref().to_path_buf(),
                }
            } else {
                Error::Io(e)
            }
        })?;
        let reader = BufReader::new(file);
        Self::new(reader)
    }
}

impl CdxReader<Cursor<Vec<u8>>> {
    /// Open a Codex document from bytes in memory.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The data is not a valid ZIP archive
    /// - Required files are missing
    /// - The manifest is invalid
    pub fn from_bytes(data: Vec<u8>) -> Result<Self> {
        let cursor = Cursor::new(data);
        Self::new(cursor)
    }
}

impl<R: Read + Seek> CdxReader<R> {
    /// Create a new reader from any `Read + Seek` source.
    ///
    /// This enables reading from files, memory buffers, network streams, etc.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The source is not a valid ZIP archive
    /// - Required files are missing
    /// - The manifest is invalid
    pub fn new(reader: R) -> Result<Self> {
        let mut archive = ZipArchive::new(reader)?;

        // Validate structure
        Self::validate_structure(&archive)?;

        // Read and parse manifest
        let manifest = Self::read_manifest(&mut archive)?;

        // Validate manifest
        manifest.validate()?;

        Ok(Self { archive, manifest })
    }

    /// Validate the archive structure.
    fn validate_structure(archive: &ZipArchive<R>) -> Result<()> {
        // Check for required files
        let required_files = [MANIFEST_PATH, CONTENT_PATH, DUBLIN_CORE_PATH];

        for path in required_files {
            if archive.index_for_name(path).is_none() {
                return Err(Error::MissingFile {
                    path: path.to_string(),
                });
            }
        }

        // Manifest must be the first file in the archive (per spec)
        if let Some(first_file) = archive.file_names().next() {
            if first_file != MANIFEST_PATH {
                return Err(Error::InvalidArchiveStructure {
                    reason: format!(
                        "manifest.json must be the first file in the archive (found '{first_file}')"
                    ),
                });
            }
        }

        Ok(())
    }

    /// Strip a UTF-8 BOM (byte order mark) prefix if present.
    fn strip_utf8_bom(data: &[u8]) -> &[u8] {
        data.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(data)
    }

    /// Read a file and parse it as JSON, stripping any UTF-8 BOM prefix.
    fn read_json_file<T: serde::de::DeserializeOwned>(
        archive: &mut ZipArchive<R>,
        path: &str,
    ) -> Result<T> {
        let data = Self::read_file_internal(archive, path)?;
        let json_data = Self::strip_utf8_bom(&data);
        Ok(serde_json::from_slice(json_data)?)
    }

    /// Read and parse the manifest.
    fn read_manifest(archive: &mut ZipArchive<R>) -> Result<Manifest> {
        Self::read_json_file(archive, MANIFEST_PATH)
    }

    /// Maximum allowed file size for decompression (256 MiB).
    ///
    /// This limit protects against decompression bombs (zip bombs) where a small
    /// compressed file expands to a very large size.
    const MAX_FILE_SIZE: u64 = 256 * 1024 * 1024;

    /// Internal file reading without path validation (for known-safe paths).
    fn read_file_internal(archive: &mut ZipArchive<R>, path: &str) -> Result<Vec<u8>> {
        let file = archive.by_name(path).map_err(|e| match e {
            zip::result::ZipError::FileNotFound => Error::MissingFile {
                path: path.to_string(),
            },
            other => Error::InvalidArchive(other),
        })?;

        // Check declared size before allocating (catches honest oversized files)
        if file.size() > Self::MAX_FILE_SIZE {
            return Err(Error::FileTooLarge {
                path: path.to_string(),
                size: file.size(),
                limit: Self::MAX_FILE_SIZE,
            });
        }

        // Use try_from with fallback to 0 for platforms with smaller usize
        let capacity = usize::try_from(file.size()).unwrap_or(0);
        let mut data = Vec::with_capacity(capacity);
        // Bounded read to catch spoofed/mismatched declared sizes
        let bytes_read = file.take(Self::MAX_FILE_SIZE + 1).read_to_end(&mut data)?;
        if bytes_read as u64 > Self::MAX_FILE_SIZE {
            return Err(Error::FileTooLarge {
                path: path.to_string(),
                size: bytes_read as u64,
                limit: Self::MAX_FILE_SIZE,
            });
        }
        Ok(data)
    }

    /// Get a reference to the document manifest.
    #[must_use]
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Read a file from the archive.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path contains traversal patterns (security check)
    /// - The file does not exist in the archive
    /// - Reading the file fails
    pub fn read_file(&mut self, path: &str) -> Result<Vec<u8>> {
        validate_path(path)?;
        Self::read_file_internal(&mut self.archive, path)
    }

    /// Read a file and verify its hash against the expected hash.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path contains traversal patterns
    /// - The file does not exist
    /// - The hash does not match the expected value
    pub fn read_file_verified(
        &mut self,
        path: &str,
        expected_hash: &crate::DocumentId,
    ) -> Result<Vec<u8>> {
        let data = self.read_file(path)?;

        // Skip verification for pending hashes
        if expected_hash.is_pending() {
            return Ok(data);
        }

        let actual_hash = Hasher::hash(expected_hash.algorithm(), &data);

        if actual_hash != *expected_hash {
            return Err(Error::HashMismatch {
                path: path.to_string(),
                expected: expected_hash.to_string(),
                actual: actual_hash.to_string(),
            });
        }

        Ok(data)
    }

    /// Read the content file.
    ///
    /// This is a convenience method for reading `content/document.json`.
    ///
    /// # Errors
    ///
    /// Returns an error if reading the content file fails.
    pub fn read_content(&mut self) -> Result<Vec<u8>> {
        self.read_file_verified(CONTENT_PATH, &self.manifest.content.hash.clone())
    }

    /// Read the Dublin Core metadata file.
    ///
    /// This is a convenience method for reading `metadata/dublin-core.json`.
    ///
    /// # Errors
    ///
    /// Returns an error if reading the metadata file fails.
    pub fn read_dublin_core(&mut self) -> Result<Vec<u8>> {
        self.read_file(&self.manifest.metadata.dublin_core.clone())
    }

    /// Check if a file exists in the archive.
    ///
    /// # Errors
    ///
    /// Returns an error if the path contains traversal patterns.
    pub fn file_exists(&self, path: &str) -> Result<bool> {
        validate_path(path)?;
        Ok(self.archive.index_for_name(path).is_some())
    }

    /// Get the list of all file paths in the archive.
    #[must_use]
    pub fn file_names(&self) -> Vec<String> {
        self.archive.file_names().map(String::from).collect()
    }

    /// Get the number of files in the archive.
    #[must_use]
    pub fn file_count(&self) -> usize {
        self.archive.len()
    }

    /// Get the hash algorithm used by this document.
    #[must_use]
    pub fn hash_algorithm(&self) -> HashAlgorithm {
        self.manifest.hash_algorithm
    }

    /// Read phantom clusters from the archive.
    ///
    /// Returns `None` if the phantom clusters file doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be parsed.
    pub fn read_phantoms(&mut self) -> Result<Option<crate::extensions::PhantomClusters>> {
        if self.archive.index_for_name(PHANTOMS_PATH).is_none() {
            return Ok(None);
        }

        let phantoms: crate::extensions::PhantomClusters =
            Self::read_json_file(&mut self.archive, PHANTOMS_PATH)?;
        Ok(Some(phantoms))
    }

    /// Verify all file hashes in the manifest.
    ///
    /// This checks:
    /// - Content file hash
    /// - Presentation file hashes (if any)
    ///
    /// # Errors
    ///
    /// Returns an error if any hash verification fails.
    pub fn verify_hashes(&mut self) -> Result<()> {
        // Verify content hash
        let content_data = self.read_file(CONTENT_PATH)?;
        if !self.manifest.content.hash.is_pending() {
            let actual = Hasher::hash(self.manifest.content.hash.algorithm(), &content_data);
            if actual != self.manifest.content.hash {
                return Err(Error::HashMismatch {
                    path: CONTENT_PATH.to_string(),
                    expected: self.manifest.content.hash.to_string(),
                    actual: actual.to_string(),
                });
            }
        }

        // Verify presentation hashes
        for pres in &self.manifest.presentation.clone() {
            if !pres.hash.is_pending() {
                let data = self.read_file(&pres.path)?;
                let actual = Hasher::hash(pres.hash.algorithm(), &data);
                if actual != pres.hash {
                    return Err(Error::HashMismatch {
                        path: pres.path.clone(),
                        expected: pres.hash.to_string(),
                        actual: actual.to_string(),
                    });
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archive::CdxWriter;
    use crate::{ContentRef, DocumentId, Metadata};
    use std::io::{Cursor, Write};

    fn create_test_archive() -> Vec<u8> {
        let buffer = Cursor::new(Vec::new());
        let mut writer = CdxWriter::new(buffer).unwrap();

        // Create a minimal manifest
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
        let manifest = Manifest::new(content, metadata);

        writer.write_manifest(&manifest).unwrap();
        writer
            .write_file(
                CONTENT_PATH,
                br#"{"version":"0.1","blocks":[]}"#,
                super::super::writer::CompressionMethod::Deflate,
            )
            .unwrap();
        writer
            .write_file(
                DUBLIN_CORE_PATH,
                br#"{"title":"Test"}"#,
                super::super::writer::CompressionMethod::Deflate,
            )
            .unwrap();

        writer.finish().unwrap().into_inner()
    }

    #[test]
    fn test_reader_from_bytes() {
        let data = create_test_archive();
        let reader = CdxReader::from_bytes(data).unwrap();
        assert_eq!(reader.manifest().codex, "0.1");
    }

    #[test]
    fn test_reader_file_list() {
        let data = create_test_archive();
        let reader = CdxReader::from_bytes(data).unwrap();
        let files = reader.file_names();
        assert!(files.contains(&MANIFEST_PATH.to_string()));
        assert!(files.contains(&CONTENT_PATH.to_string()));
        assert!(files.contains(&DUBLIN_CORE_PATH.to_string()));
    }

    #[test]
    fn test_reader_read_file() {
        let data = create_test_archive();
        let mut reader = CdxReader::from_bytes(data).unwrap();
        let content = reader.read_file(CONTENT_PATH).unwrap();
        assert!(!content.is_empty());
    }

    #[test]
    fn test_reader_file_exists() {
        let data = create_test_archive();
        let reader = CdxReader::from_bytes(data).unwrap();
        assert!(reader.file_exists(MANIFEST_PATH).unwrap());
        assert!(reader.file_exists(CONTENT_PATH).unwrap());
        assert!(!reader.file_exists("nonexistent.json").unwrap());
    }

    #[test]
    fn test_reader_path_traversal_rejected() {
        let data = create_test_archive();
        let mut reader = CdxReader::from_bytes(data).unwrap();
        assert!(reader.read_file("../secret").is_err());
        assert!(reader.file_exists("../secret").is_err());
    }

    #[test]
    fn test_reader_missing_file_error() {
        let data = create_test_archive();
        let mut reader = CdxReader::from_bytes(data).unwrap();
        let result = reader.read_file("nonexistent.json");
        assert!(matches!(result, Err(Error::MissingFile { .. })));
    }

    #[test]
    fn test_open_corrupted_zip() {
        // Random bytes that aren't a valid ZIP
        let corrupted = vec![0x50, 0x4B, 0x03, 0x04, 0xFF, 0xFF, 0xFF, 0xFF];
        let result = CdxReader::from_bytes(corrupted);
        assert!(result.is_err());
    }

    #[test]
    fn test_open_not_a_zip() {
        // Plain text, not a ZIP file
        let not_zip = b"This is not a ZIP file at all".to_vec();
        let result = CdxReader::from_bytes(not_zip);
        assert!(result.is_err());
    }

    #[test]
    fn test_open_empty_zip() {
        // Create an empty ZIP (no files)
        let buffer = Cursor::new(Vec::new());
        let writer = zip::ZipWriter::new(buffer);
        let empty_zip = writer.finish().unwrap().into_inner();

        let result = CdxReader::from_bytes(empty_zip);
        assert!(matches!(result, Err(Error::MissingFile { .. })));
    }

    #[test]
    fn test_open_missing_manifest() {
        // Create a ZIP without manifest.json
        let buffer = Cursor::new(Vec::new());
        let mut writer = zip::ZipWriter::new(buffer);
        writer
            .start_file::<&str, ()>(CONTENT_PATH, Default::default())
            .unwrap();
        writer.write_all(b"{}").unwrap();
        writer
            .start_file::<&str, ()>(DUBLIN_CORE_PATH, Default::default())
            .unwrap();
        writer.write_all(b"{}").unwrap();
        let data = writer.finish().unwrap().into_inner();

        let result = CdxReader::from_bytes(data);
        assert!(matches!(result, Err(Error::MissingFile { path }) if path == MANIFEST_PATH));
    }

    #[test]
    fn test_open_missing_content() {
        // Create a ZIP with manifest but no content file
        let buffer = Cursor::new(Vec::new());
        let mut writer = zip::ZipWriter::new(buffer);

        // Add manifest
        writer
            .start_file::<&str, ()>(MANIFEST_PATH, Default::default())
            .unwrap();
        writer.write_all(br#"{"codex":"0.1"}"#).unwrap();

        // Add Dublin Core but no content
        writer
            .start_file::<&str, ()>(DUBLIN_CORE_PATH, Default::default())
            .unwrap();
        writer.write_all(b"{}").unwrap();

        let data = writer.finish().unwrap().into_inner();

        let result = CdxReader::from_bytes(data);
        assert!(matches!(result, Err(Error::MissingFile { path }) if path == CONTENT_PATH));
    }

    #[test]
    fn test_open_invalid_manifest_json() {
        // Create a ZIP with invalid JSON in manifest
        let buffer = Cursor::new(Vec::new());
        let mut writer = zip::ZipWriter::new(buffer);

        writer
            .start_file::<&str, ()>(MANIFEST_PATH, Default::default())
            .unwrap();
        writer.write_all(b"{ invalid json }").unwrap();

        writer
            .start_file::<&str, ()>(CONTENT_PATH, Default::default())
            .unwrap();
        writer.write_all(b"{}").unwrap();

        writer
            .start_file::<&str, ()>(DUBLIN_CORE_PATH, Default::default())
            .unwrap();
        writer.write_all(b"{}").unwrap();

        let data = writer.finish().unwrap().into_inner();

        let result = CdxReader::from_bytes(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_read_file_hash_mismatch() {
        let buffer = Cursor::new(Vec::new());
        let mut writer = CdxWriter::new(buffer).unwrap();

        // Create manifest with a specific hash
        let expected_hash: DocumentId =
            "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                .parse()
                .unwrap();
        let content = ContentRef {
            path: CONTENT_PATH.to_string(),
            hash: expected_hash.clone(),
            compression: None,
            merkle_root: None,
            block_count: None,
        };
        let metadata = Metadata {
            dublin_core: DUBLIN_CORE_PATH.to_string(),
            custom: None,
        };
        let manifest = Manifest::new(content, metadata);

        writer.write_manifest(&manifest).unwrap();
        // Write content that doesn't match the hash
        writer
            .write_file(
                CONTENT_PATH,
                br#"{"version":"0.1","blocks":[]}"#,
                super::super::writer::CompressionMethod::Deflate,
            )
            .unwrap();
        writer
            .write_file(
                DUBLIN_CORE_PATH,
                br#"{"title":"Test"}"#,
                super::super::writer::CompressionMethod::Deflate,
            )
            .unwrap();

        let data = writer.finish().unwrap().into_inner();
        let mut reader = CdxReader::from_bytes(data).unwrap();

        let result = reader.read_file_verified(CONTENT_PATH, &expected_hash);
        assert!(matches!(result, Err(Error::HashMismatch { .. })));
    }

    #[test]
    fn test_verify_hashes_with_mismatch() {
        let buffer = Cursor::new(Vec::new());
        let mut writer = CdxWriter::new(buffer).unwrap();

        // Create manifest with a wrong hash
        let wrong_hash: DocumentId =
            "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
                .parse()
                .unwrap();
        let content = ContentRef {
            path: CONTENT_PATH.to_string(),
            hash: wrong_hash,
            compression: None,
            merkle_root: None,
            block_count: None,
        };
        let metadata = Metadata {
            dublin_core: DUBLIN_CORE_PATH.to_string(),
            custom: None,
        };
        let manifest = Manifest::new(content, metadata);

        writer.write_manifest(&manifest).unwrap();
        writer
            .write_file(
                CONTENT_PATH,
                br#"{"version":"0.1","blocks":[]}"#,
                super::super::writer::CompressionMethod::Deflate,
            )
            .unwrap();
        writer
            .write_file(
                DUBLIN_CORE_PATH,
                br#"{"title":"Test"}"#,
                super::super::writer::CompressionMethod::Deflate,
            )
            .unwrap();

        let data = writer.finish().unwrap().into_inner();
        let mut reader = CdxReader::from_bytes(data).unwrap();

        let result = reader.verify_hashes();
        assert!(matches!(result, Err(Error::HashMismatch { .. })));
    }

    #[test]
    fn test_read_file_verified_with_pending_hash() {
        let data = create_test_archive();
        let mut reader = CdxReader::from_bytes(data).unwrap();

        // Pending hashes should skip verification
        let pending = DocumentId::pending();
        let result = reader.read_file_verified(CONTENT_PATH, &pending);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unicode_filenames() {
        let buffer = Cursor::new(Vec::new());
        let mut writer = CdxWriter::new(buffer).unwrap();

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
        let manifest = Manifest::new(content, metadata);

        writer.write_manifest(&manifest).unwrap();
        writer
            .write_file(
                CONTENT_PATH,
                br#"{"version":"0.1","blocks":[]}"#,
                super::super::writer::CompressionMethod::Deflate,
            )
            .unwrap();
        writer
            .write_file(
                DUBLIN_CORE_PATH,
                br#"{"title":"Test"}"#,
                super::super::writer::CompressionMethod::Deflate,
            )
            .unwrap();

        // Add a file with Unicode characters
        writer
            .write_file(
                "assets/文档.txt",
                b"Unicode content",
                super::super::writer::CompressionMethod::Deflate,
            )
            .unwrap();
        writer
            .write_file(
                "assets/émoji_🎉.txt",
                b"Emoji content",
                super::super::writer::CompressionMethod::Deflate,
            )
            .unwrap();

        let data = writer.finish().unwrap().into_inner();
        let mut reader = CdxReader::from_bytes(data).unwrap();

        // Verify we can read the Unicode files
        let files = reader.file_names();
        assert!(files.contains(&"assets/文档.txt".to_string()));
        assert!(files.contains(&"assets/émoji_🎉.txt".to_string()));

        let content = reader.read_file("assets/文档.txt").unwrap();
        assert_eq!(content, b"Unicode content");

        let emoji_content = reader.read_file("assets/émoji_🎉.txt").unwrap();
        assert_eq!(emoji_content, b"Emoji content");
    }

    #[test]
    fn test_file_count() {
        let data = create_test_archive();
        let reader = CdxReader::from_bytes(data).unwrap();
        // manifest, content, dublin_core = 3 files
        assert_eq!(reader.file_count(), 3);
    }

    #[test]
    fn test_hash_algorithm() {
        let data = create_test_archive();
        let reader = CdxReader::from_bytes(data).unwrap();
        assert_eq!(reader.hash_algorithm(), HashAlgorithm::Sha256);
    }

    #[test]
    fn test_read_phantoms_none() {
        let data = create_test_archive();
        let mut reader = CdxReader::from_bytes(data).unwrap();
        // No phantoms file in the test archive
        let result = reader.read_phantoms().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_manifest_must_be_first_file() {
        // Create a ZIP where manifest is NOT the first file
        let buffer = Cursor::new(Vec::new());
        let mut writer = zip::ZipWriter::new(buffer);

        // Write content BEFORE manifest
        writer
            .start_file::<&str, ()>(CONTENT_PATH, Default::default())
            .unwrap();
        writer
            .write_all(br#"{"version":"0.1","blocks":[]}"#)
            .unwrap();

        // Now write manifest (not first)
        let manifest_json = r#"{
            "codex": "0.1",
            "id": "pending",
            "state": "draft",
            "created": "2024-01-01T00:00:00Z",
            "modified": "2024-01-01T00:00:00Z",
            "content": { "path": "content/document.json", "hash": "pending" },
            "metadata": { "dublinCore": "metadata/dublin-core.json" }
        }"#;
        writer
            .start_file::<&str, ()>(MANIFEST_PATH, Default::default())
            .unwrap();
        writer.write_all(manifest_json.as_bytes()).unwrap();

        writer
            .start_file::<&str, ()>(DUBLIN_CORE_PATH, Default::default())
            .unwrap();
        writer.write_all(br#"{"title":"Test"}"#).unwrap();

        let data = writer.finish().unwrap().into_inner();
        let result = CdxReader::from_bytes(data);

        let err = result.err().expect("should be an error");
        assert!(matches!(err, Error::InvalidArchiveStructure { .. }));
    }

    #[test]
    fn test_manifest_first_file_passes() {
        // Normal archive created by CdxWriter should have manifest first
        let data = create_test_archive();
        let result = CdxReader::from_bytes(data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_utf8_bom_stripped_from_manifest() {
        // Create a ZIP with BOM-prefixed manifest JSON
        let buffer = Cursor::new(Vec::new());
        let mut writer = zip::ZipWriter::new(buffer);

        // Manifest with UTF-8 BOM prefix
        let manifest_json = r#"{
            "codex": "0.1",
            "id": "pending",
            "state": "draft",
            "created": "2024-01-01T00:00:00Z",
            "modified": "2024-01-01T00:00:00Z",
            "hashAlgorithm": "sha256",
            "content": { "path": "content/document.json", "hash": "pending" },
            "metadata": { "dublinCore": "metadata/dublin-core.json" }
        }"#;
        let mut bom_manifest = vec![0xEF, 0xBB, 0xBF];
        bom_manifest.extend_from_slice(manifest_json.as_bytes());

        writer
            .start_file::<&str, ()>(MANIFEST_PATH, Default::default())
            .unwrap();
        writer.write_all(&bom_manifest).unwrap();

        writer
            .start_file::<&str, ()>(CONTENT_PATH, Default::default())
            .unwrap();
        writer
            .write_all(br#"{"version":"0.1","blocks":[]}"#)
            .unwrap();

        writer
            .start_file::<&str, ()>(DUBLIN_CORE_PATH, Default::default())
            .unwrap();
        writer.write_all(br#"{"title":"Test"}"#).unwrap();

        let data = writer.finish().unwrap().into_inner();
        let reader = CdxReader::from_bytes(data);
        assert!(
            reader.is_ok(),
            "BOM-prefixed manifest should parse correctly"
        );
        assert_eq!(reader.unwrap().manifest().codex, "0.1");
    }

    #[test]
    fn test_utf8_bom_not_required() {
        // Regular archive without BOM should still work fine
        let data = create_test_archive();
        let reader = CdxReader::from_bytes(data);
        assert!(reader.is_ok());
    }
}
