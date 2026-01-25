//! High-level Document API.
//!
//! This module provides the main [`Document`] type and [`DocumentBuilder`]
//! for working with Codex documents.
//!
//! # Opening Documents
//!
//! ```rust,ignore
//! use cdx_core::Document;
//!
//! let doc = Document::open("example.cdx")?;
//! println!("Title: {}", doc.title());
//! ```
//!
//! # Creating Documents
//!
//! ```rust,ignore
//! use cdx_core::{Document, content::{Block, Text}};
//!
//! let doc = Document::builder()
//!     .title("My Document")
//!     .creator("Jane Doe")
//!     .add_heading(1, "Introduction")
//!     .add_paragraph("This is the first paragraph.")
//!     .build()?;
//!
//! doc.save("output.cdx")?;
//! ```

use std::fs::File;
use std::io::{Cursor, Read, Seek, Write};
use std::path::Path;

use crate::archive::{CdxReader, CdxWriter, CompressionMethod, CONTENT_PATH, DUBLIN_CORE_PATH};
use crate::content::{Block, Content, Text};
use crate::metadata::DublinCore;
use crate::{DocumentId, DocumentState, HashAlgorithm, Hasher, Manifest, Result};

/// A Codex document.
///
/// `Document` provides a high-level interface for working with Codex documents,
/// abstracting away the underlying archive structure.
#[derive(Debug, Clone)]
pub struct Document {
    manifest: Manifest,
    content: Content,
    dublin_core: DublinCore,
}

impl Document {
    /// Open a document from a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be opened
    /// - The archive is invalid
    /// - Required files are missing or malformed
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut reader = CdxReader::open(path)?;
        Self::from_reader(&mut reader)
    }

    /// Open a document from any `Read + Seek` source.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The source is not a valid Codex archive
    /// - Required files are missing or malformed
    pub fn open_from_reader<R: Read + Seek>(reader: R) -> Result<Self> {
        let mut cdx_reader = CdxReader::new(reader)?;
        Self::from_reader(&mut cdx_reader)
    }

    /// Open a document from bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is not a valid Codex document.
    pub fn from_bytes(data: Vec<u8>) -> Result<Self> {
        let mut reader = CdxReader::from_bytes(data)?;
        Self::from_reader(&mut reader)
    }

    /// Read document from a `CdxReader`.
    fn from_reader<R: Read + Seek>(reader: &mut CdxReader<R>) -> Result<Self> {
        let manifest = reader.manifest().clone();

        // Read and parse content
        let content_data = reader.read_content()?;
        let content: Content = serde_json::from_slice(&content_data)?;

        // Read and parse Dublin Core
        let dc_data = reader.read_dublin_core()?;
        let dublin_core: DublinCore = serde_json::from_slice(&dc_data)?;

        Ok(Self {
            manifest,
            content,
            dublin_core,
        })
    }

    /// Create a new document builder.
    #[must_use]
    pub fn builder() -> DocumentBuilder {
        DocumentBuilder::new()
    }

    /// Save the document to a file.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be created
    /// - Writing fails
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file = File::create(path)?;
        let writer = std::io::BufWriter::new(file);
        self.write_to(writer)
    }

    /// Write the document to any `Write + Seek` destination.
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub fn write_to<W: Write + Seek>(&self, writer: W) -> Result<()> {
        let mut cdx_writer = CdxWriter::new(writer)?;

        // Serialize content and dublin core
        let content_json = serde_json::to_vec_pretty(&self.content)?;
        let dc_json = serde_json::to_vec_pretty(&self.dublin_core)?;

        // Compute hashes
        let content_hash = Hasher::hash(self.manifest.hash_algorithm, &content_json);

        // Update manifest with computed hashes
        let mut manifest = self.manifest.clone();
        manifest.content.hash = content_hash;

        // Write files
        cdx_writer.write_manifest(&manifest)?;
        cdx_writer.write_file(CONTENT_PATH, &content_json, CompressionMethod::Deflate)?;
        cdx_writer.write_file(DUBLIN_CORE_PATH, &dc_json, CompressionMethod::Deflate)?;

        cdx_writer.finish()?;
        Ok(())
    }

    /// Write the document to bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let cursor = Cursor::new(Vec::new());
        let mut temp = cursor;
        self.write_to(&mut temp)?;
        Ok(temp.into_inner())
    }

    /// Get a reference to the manifest.
    #[must_use]
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Get a reference to the content.
    #[must_use]
    pub fn content(&self) -> &Content {
        &self.content
    }

    /// Get a reference to the Dublin Core metadata.
    #[must_use]
    pub fn dublin_core(&self) -> &DublinCore {
        &self.dublin_core
    }

    /// Get the document title.
    #[must_use]
    pub fn title(&self) -> &str {
        self.dublin_core.title()
    }

    /// Get the document creators.
    #[must_use]
    pub fn creators(&self) -> Vec<&str> {
        self.dublin_core.creators()
    }

    /// Get the document state.
    #[must_use]
    pub fn state(&self) -> DocumentState {
        self.manifest.state
    }

    /// Get the document ID.
    #[must_use]
    pub fn id(&self) -> &DocumentId {
        &self.manifest.id
    }

    /// Get the hash algorithm used.
    #[must_use]
    pub fn hash_algorithm(&self) -> HashAlgorithm {
        self.manifest.hash_algorithm
    }

    /// Compute the document ID from content.
    ///
    /// The document ID is computed by hashing the canonicalized content.
    ///
    /// # Errors
    ///
    /// Returns an error if canonicalization fails.
    pub fn compute_id(&self) -> Result<DocumentId> {
        // Serialize content to canonical JSON
        let content_json = serde_json::to_vec(&self.content)?;
        let canonical = json_canon::to_string(&serde_json::from_slice::<serde_json::Value>(
            &content_json,
        )?)?;

        Ok(Hasher::hash(self.manifest.hash_algorithm, canonical.as_bytes()))
    }

    /// Verify the document integrity.
    ///
    /// This checks:
    /// - Content hash matches manifest
    /// - Document ID is valid (if not pending)
    ///
    /// # Errors
    ///
    /// Returns an error if verification fails.
    pub fn verify(&self) -> Result<VerificationReport> {
        let mut report = VerificationReport {
            content_valid: true,
            id_valid: true,
            errors: Vec::new(),
        };

        // Verify content hash
        // Note: must use to_vec_pretty to match what write_to uses
        if !self.manifest.content.hash.is_pending() {
            let content_json = serde_json::to_vec_pretty(&self.content)?;
            let actual_hash = Hasher::hash(self.manifest.content.hash.algorithm(), &content_json);

            if actual_hash != self.manifest.content.hash {
                report.content_valid = false;
                report.errors.push(format!(
                    "Content hash mismatch: expected {}, got {}",
                    self.manifest.content.hash, actual_hash
                ));
            }
        }

        // Verify document ID
        if !self.manifest.id.is_pending() {
            let computed_id = self.compute_id()?;
            if computed_id != self.manifest.id {
                report.id_valid = false;
                report.errors.push(format!(
                    "Document ID mismatch: expected {}, got {}",
                    self.manifest.id, computed_id
                ));
            }
        }

        Ok(report)
    }
}

/// Report from document verification.
#[derive(Debug, Clone)]
pub struct VerificationReport {
    /// Whether content hash is valid.
    pub content_valid: bool,
    /// Whether document ID is valid.
    pub id_valid: bool,
    /// Error messages.
    pub errors: Vec<String>,
}

impl VerificationReport {
    /// Check if verification passed.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.content_valid && self.id_valid && self.errors.is_empty()
    }
}

/// Builder for creating Codex documents.
#[derive(Debug, Clone)]
pub struct DocumentBuilder {
    title: String,
    creator: String,
    description: Option<String>,
    language: Option<String>,
    blocks: Vec<Block>,
    state: DocumentState,
    hash_algorithm: HashAlgorithm,
}

impl Default for DocumentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentBuilder {
    /// Create a new document builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            title: "Untitled".to_string(),
            creator: "Unknown".to_string(),
            description: None,
            language: None,
            blocks: Vec::new(),
            state: DocumentState::Draft,
            hash_algorithm: HashAlgorithm::default(),
        }
    }

    /// Set the document title.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the document creator.
    #[must_use]
    pub fn creator(mut self, creator: impl Into<String>) -> Self {
        self.creator = creator.into();
        self
    }

    /// Set the document description.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the document language.
    #[must_use]
    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Set the document state.
    #[must_use]
    pub fn state(mut self, state: DocumentState) -> Self {
        self.state = state;
        self
    }

    /// Set the hash algorithm.
    #[must_use]
    pub fn hash_algorithm(mut self, algorithm: HashAlgorithm) -> Self {
        self.hash_algorithm = algorithm;
        self
    }

    /// Add a content block.
    #[must_use]
    pub fn add_block(mut self, block: Block) -> Self {
        self.blocks.push(block);
        self
    }

    /// Add a heading block.
    #[must_use]
    pub fn add_heading(self, level: u8, text: impl Into<String>) -> Self {
        self.add_block(Block::heading(level, vec![Text::plain(text)]))
    }

    /// Add a paragraph block.
    #[must_use]
    pub fn add_paragraph(self, text: impl Into<String>) -> Self {
        self.add_block(Block::paragraph(vec![Text::plain(text)]))
    }

    /// Add a code block.
    #[must_use]
    pub fn add_code_block(self, code: impl Into<String>, language: Option<String>) -> Self {
        self.add_block(Block::code_block(code, language))
    }

    /// Build the document.
    ///
    /// # Errors
    ///
    /// Returns an error if the document cannot be constructed.
    pub fn build(self) -> Result<Document> {
        use crate::manifest::{ContentRef, Metadata};

        let content = Content::new(self.blocks);
        let dublin_core = {
            let mut dc = DublinCore::new(&self.title, &self.creator);
            dc.terms.description = self.description;
            dc.terms.language = self.language;
            dc
        };

        let content_ref = ContentRef {
            path: CONTENT_PATH.to_string(),
            hash: DocumentId::pending(),
            compression: Some("deflate".to_string()),
        };

        let metadata = Metadata {
            dublin_core: DUBLIN_CORE_PATH.to_string(),
            custom: None,
        };

        let mut manifest = Manifest::new(content_ref, metadata);
        manifest.state = self.state;
        manifest.hash_algorithm = self.hash_algorithm;

        Ok(Document {
            manifest,
            content,
            dublin_core,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let doc = Document::builder()
            .title("Test Document")
            .creator("Test Author")
            .add_heading(1, "Introduction")
            .add_paragraph("This is the first paragraph.")
            .build()
            .unwrap();

        assert_eq!(doc.title(), "Test Document");
        assert_eq!(doc.creators(), vec!["Test Author"]);
        assert_eq!(doc.content().len(), 2);
        assert_eq!(doc.state(), DocumentState::Draft);
    }

    #[test]
    fn test_builder_with_description() {
        let doc = Document::builder()
            .title("Report")
            .creator("Author")
            .description("A detailed report")
            .language("en")
            .build()
            .unwrap();

        assert_eq!(doc.dublin_core().description(), Some("A detailed report"));
        assert_eq!(doc.dublin_core().language(), Some("en"));
    }

    #[test]
    fn test_document_round_trip() {
        let original = Document::builder()
            .title("Round Trip Test")
            .creator("Tester")
            .add_heading(1, "Title")
            .add_paragraph("Content")
            .build()
            .unwrap();

        // Write to bytes
        let bytes = original.to_bytes().unwrap();

        // Read back
        let loaded = Document::from_bytes(bytes).unwrap();

        assert_eq!(loaded.title(), "Round Trip Test");
        assert_eq!(loaded.creators(), vec!["Tester"]);
        assert_eq!(loaded.content().len(), 2);
    }

    #[test]
    fn test_compute_id() {
        let doc = Document::builder()
            .title("ID Test")
            .creator("Author")
            .add_paragraph("Test content")
            .build()
            .unwrap();

        let id = doc.compute_id().unwrap();
        assert!(!id.is_pending());
        assert_eq!(id.algorithm(), HashAlgorithm::Sha256);
    }

    #[test]
    fn test_verification() {
        let doc = Document::builder()
            .title("Verify Test")
            .creator("Author")
            .add_paragraph("Content")
            .build()
            .unwrap();

        // Fresh documents with pending hashes should verify
        let report = doc.verify().unwrap();
        assert!(report.is_valid());
    }
}
