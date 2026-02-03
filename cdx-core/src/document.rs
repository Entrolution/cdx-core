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

use chrono::Utc;

#[cfg(feature = "encryption")]
use crate::archive::ENCRYPTION_PATH;
#[cfg(feature = "signatures")]
use crate::archive::SIGNATURES_PATH;
use crate::archive::{
    CdxReader, CdxWriter, CompressionMethod, ACADEMIC_NUMBERING_PATH, BIBLIOGRAPHY_PATH,
    COMMENTS_PATH, CONTENT_PATH, DUBLIN_CORE_PATH, FORMS_DATA_PATH, JSONLD_PATH, PHANTOMS_PATH,
};
use crate::content::{Block, Content, Text};
use crate::extensions::academic::NumberingConfig;
use crate::extensions::{Bibliography, CommentThread, FormData, JsonLdMetadata, PhantomClusters};
use crate::manifest::Lineage;
#[cfg(any(feature = "signatures", feature = "encryption"))]
use crate::manifest::SecurityRef;
use crate::metadata::DublinCore;
#[cfg(feature = "encryption")]
use crate::security::EncryptionMetadata;
#[cfg(feature = "signatures")]
use crate::security::{Signature, SignatureFile};
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
    #[cfg(feature = "signatures")]
    signature_file: Option<SignatureFile>,
    #[cfg(feature = "encryption")]
    encryption_metadata: Option<EncryptionMetadata>,
    /// Academic extension numbering configuration.
    academic_numbering: Option<NumberingConfig>,
    /// Collaboration extension comments.
    comments: Option<CommentThread>,
    /// Phantom extension clusters.
    phantom_clusters: Option<PhantomClusters>,
    /// Forms extension data.
    form_data: Option<FormData>,
    /// Semantic extension bibliography.
    bibliography: Option<Bibliography>,
    /// JSON-LD metadata for semantic web integration.
    jsonld_metadata: Option<JsonLdMetadata>,
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

        // Read signatures if present (only when signatures feature is enabled)
        #[cfg(feature = "signatures")]
        let signature_file = if let Some(ref security) = manifest.security {
            if let Some(ref sig_path) = security.signatures {
                if reader.file_exists(sig_path)? {
                    let sig_data = reader.read_file(sig_path)?;
                    let sig_file: SignatureFile = serde_json::from_slice(&sig_data)?;
                    Some(sig_file)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Read encryption metadata if present (only when encryption feature is enabled)
        #[cfg(feature = "encryption")]
        let encryption_metadata = if let Some(ref security) = manifest.security {
            if let Some(ref enc_path) = security.encryption {
                if reader.file_exists(enc_path)? {
                    let enc_data = reader.read_file(enc_path)?;
                    let enc_meta: EncryptionMetadata = serde_json::from_slice(&enc_data)?;
                    Some(enc_meta)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Read academic numbering configuration if present
        let academic_numbering = if reader.file_exists(ACADEMIC_NUMBERING_PATH)? {
            let numbering_data = reader.read_file(ACADEMIC_NUMBERING_PATH)?;
            Some(serde_json::from_slice(&numbering_data)?)
        } else {
            None
        };

        // Read collaboration comments if present
        let comments = if reader.file_exists(COMMENTS_PATH)? {
            let comments_data = reader.read_file(COMMENTS_PATH)?;
            Some(serde_json::from_slice(&comments_data)?)
        } else {
            None
        };

        // Read phantom clusters if present
        let phantom_clusters = if reader.file_exists(PHANTOMS_PATH)? {
            let phantoms_data = reader.read_file(PHANTOMS_PATH)?;
            Some(serde_json::from_slice(&phantoms_data)?)
        } else {
            None
        };

        // Read form data if present
        let form_data = if reader.file_exists(FORMS_DATA_PATH)? {
            let forms_data = reader.read_file(FORMS_DATA_PATH)?;
            Some(serde_json::from_slice(&forms_data)?)
        } else {
            None
        };

        // Read bibliography if present
        let bibliography = if reader.file_exists(BIBLIOGRAPHY_PATH)? {
            let bib_data = reader.read_file(BIBLIOGRAPHY_PATH)?;
            Some(serde_json::from_slice(&bib_data)?)
        } else {
            None
        };

        // Read JSON-LD metadata if present
        let jsonld_metadata = if reader.file_exists(JSONLD_PATH)? {
            let jsonld_data = reader.read_file(JSONLD_PATH)?;
            Some(serde_json::from_slice(&jsonld_data)?)
        } else {
            None
        };

        Ok(Self {
            manifest,
            content,
            dublin_core,
            #[cfg(feature = "signatures")]
            signature_file,
            #[cfg(feature = "encryption")]
            encryption_metadata,
            academic_numbering,
            comments,
            phantom_clusters,
            form_data,
            bibliography,
            jsonld_metadata,
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

        // Update security reference if we have signatures or encryption
        #[cfg(any(feature = "signatures", feature = "encryption"))]
        {
            #[cfg(feature = "signatures")]
            let has_signatures = self
                .signature_file
                .as_ref()
                .is_some_and(|sf| !sf.is_empty());
            #[cfg(not(feature = "signatures"))]
            let has_signatures = false;

            #[cfg(feature = "encryption")]
            let has_encryption = self.encryption_metadata.is_some();
            #[cfg(not(feature = "encryption"))]
            let has_encryption = false;

            if has_signatures || has_encryption {
                #[cfg(feature = "signatures")]
                let signatures_ref = if has_signatures {
                    Some(SIGNATURES_PATH.to_string())
                } else {
                    None
                };
                #[cfg(not(feature = "signatures"))]
                let signatures_ref = None;

                #[cfg(feature = "encryption")]
                let encryption_ref = if has_encryption {
                    Some(ENCRYPTION_PATH.to_string())
                } else {
                    None
                };
                #[cfg(not(feature = "encryption"))]
                let encryption_ref = None;

                manifest.security = Some(SecurityRef {
                    signatures: signatures_ref,
                    encryption: encryption_ref,
                });
            }
        }

        // Write files
        cdx_writer.write_manifest(&manifest)?;
        cdx_writer.write_file(CONTENT_PATH, &content_json, CompressionMethod::Deflate)?;
        cdx_writer.write_file(DUBLIN_CORE_PATH, &dc_json, CompressionMethod::Deflate)?;

        // Write signatures if present
        #[cfg(feature = "signatures")]
        if let Some(ref sig_file) = self.signature_file {
            if !sig_file.is_empty() {
                let sig_json = sig_file.to_json()?;
                cdx_writer.write_file(
                    SIGNATURES_PATH,
                    sig_json.as_bytes(),
                    CompressionMethod::Deflate,
                )?;
            }
        }

        // Write encryption metadata if present
        #[cfg(feature = "encryption")]
        if let Some(ref enc_meta) = self.encryption_metadata {
            let enc_json = serde_json::to_vec_pretty(enc_meta)?;
            cdx_writer.write_file(ENCRYPTION_PATH, &enc_json, CompressionMethod::Deflate)?;
        }

        // Write academic numbering configuration if present
        if let Some(ref numbering) = self.academic_numbering {
            let numbering_json = serde_json::to_vec_pretty(numbering)?;
            cdx_writer.write_file(
                ACADEMIC_NUMBERING_PATH,
                &numbering_json,
                CompressionMethod::Deflate,
            )?;
        }

        // Write collaboration comments if present
        if let Some(ref comments) = self.comments {
            let comments_json = serde_json::to_vec_pretty(comments)?;
            cdx_writer.write_file(COMMENTS_PATH, &comments_json, CompressionMethod::Deflate)?;
        }

        // Write phantom clusters if present
        if let Some(ref phantoms) = self.phantom_clusters {
            let phantoms_json = serde_json::to_vec_pretty(phantoms)?;
            cdx_writer.write_file(PHANTOMS_PATH, &phantoms_json, CompressionMethod::Deflate)?;
        }

        // Write form data if present
        if let Some(ref form_data) = self.form_data {
            let forms_json = serde_json::to_vec_pretty(form_data)?;
            cdx_writer.write_file(FORMS_DATA_PATH, &forms_json, CompressionMethod::Deflate)?;
        }

        // Write bibliography if present
        if let Some(ref bibliography) = self.bibliography {
            let bib_json = serde_json::to_vec_pretty(bibliography)?;
            cdx_writer.write_file(BIBLIOGRAPHY_PATH, &bib_json, CompressionMethod::Deflate)?;
        }

        // Write JSON-LD metadata if present
        if let Some(ref jsonld) = self.jsonld_metadata {
            let jsonld_json = serde_json::to_vec_pretty(jsonld)?;
            cdx_writer.write_file(JSONLD_PATH, &jsonld_json, CompressionMethod::Deflate)?;
        }

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

    /// Get a mutable reference to the content.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is in an immutable state.
    pub fn content_mut(&mut self) -> Result<&mut Content> {
        if self.manifest.state.is_immutable() {
            return Err(crate::Error::InvalidManifest {
                reason: format!("Cannot modify content in {} state", self.manifest.state),
            });
        }
        self.manifest.modified = Utc::now();
        Ok(&mut self.content)
    }

    /// Get a reference to the Dublin Core metadata.
    #[must_use]
    pub fn dublin_core(&self) -> &DublinCore {
        &self.dublin_core
    }

    /// Get a mutable reference to the Dublin Core metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is in an immutable state.
    pub fn dublin_core_mut(&mut self) -> Result<&mut DublinCore> {
        if self.manifest.state.is_immutable() {
            return Err(crate::Error::InvalidManifest {
                reason: format!(
                    "Cannot modify Dublin Core metadata in {} state",
                    self.manifest.state
                ),
            });
        }
        self.manifest.modified = Utc::now();
        Ok(&mut self.dublin_core)
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

    /// Get a reference to the signature file, if present.
    #[cfg(feature = "signatures")]
    #[must_use]
    pub fn signature_file(&self) -> Option<&SignatureFile> {
        self.signature_file.as_ref()
    }

    /// Get the signatures from the document.
    #[cfg(feature = "signatures")]
    #[must_use]
    pub fn signatures(&self) -> &[Signature] {
        self.signature_file
            .as_ref()
            .map_or(&[], |sf| sf.signatures.as_slice())
    }

    /// Add a signature to the document.
    ///
    /// This adds the signature to the document's signature file. If no signature file
    /// exists, one will be created. The document ID in the signature file will be
    /// updated to match the current computed document ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the document ID cannot be computed.
    #[cfg(feature = "signatures")]
    pub fn add_signature(&mut self, signature: Signature) -> Result<()> {
        let doc_id = self.compute_id()?;

        if let Some(sig_file) = self.signature_file.as_mut() {
            // Update document ID if it changed
            sig_file.document_id = doc_id;
            sig_file.add_signature(signature);
        } else {
            let mut sig_file = SignatureFile::new(doc_id);
            sig_file.add_signature(signature);
            self.signature_file = Some(sig_file);
        }

        // Update manifest to reference the security section
        self.manifest.security = Some(SecurityRef {
            signatures: Some(SIGNATURES_PATH.to_string()),
            encryption: self
                .manifest
                .security
                .as_ref()
                .and_then(|s| s.encryption.clone()),
        });

        Ok(())
    }

    /// Check if the document has any signatures.
    #[cfg(feature = "signatures")]
    #[must_use]
    pub fn has_signatures(&self) -> bool {
        self.signature_file
            .as_ref()
            .is_some_and(|sf| !sf.is_empty())
    }

    /// Check if the document has any signatures.
    ///
    /// Always returns false when the signatures feature is disabled.
    #[cfg(not(feature = "signatures"))]
    #[must_use]
    pub fn has_signatures(&self) -> bool {
        false
    }

    /// Get a reference to the encryption metadata, if present.
    #[cfg(feature = "encryption")]
    #[must_use]
    pub fn encryption_metadata(&self) -> Option<&EncryptionMetadata> {
        self.encryption_metadata.as_ref()
    }

    /// Check if the document has encryption metadata.
    #[cfg(feature = "encryption")]
    #[must_use]
    pub fn is_encrypted(&self) -> bool {
        self.encryption_metadata.is_some()
    }

    /// Check if the document has encryption metadata.
    ///
    /// Always returns false when the encryption feature is disabled.
    #[cfg(not(feature = "encryption"))]
    #[must_use]
    pub fn is_encrypted(&self) -> bool {
        false
    }

    /// Set encryption metadata for this document.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is in an immutable state.
    #[cfg(feature = "encryption")]
    pub fn set_encryption(&mut self, metadata: EncryptionMetadata) -> Result<()> {
        if self.manifest.state.is_immutable() {
            return Err(crate::Error::InvalidManifest {
                reason: format!("Cannot set encryption in {} state", self.manifest.state),
            });
        }

        self.encryption_metadata = Some(metadata);
        Ok(())
    }

    /// Remove encryption metadata from this document.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is in an immutable state.
    #[cfg(feature = "encryption")]
    pub fn clear_encryption(&mut self) -> Result<()> {
        if self.manifest.state.is_immutable() {
            return Err(crate::Error::InvalidManifest {
                reason: format!("Cannot remove encryption in {} state", self.manifest.state),
            });
        }

        self.encryption_metadata = None;
        Ok(())
    }

    // ===== Academic Extension Methods =====

    /// Get the academic numbering configuration, if present.
    #[must_use]
    pub fn academic_numbering(&self) -> Option<&NumberingConfig> {
        self.academic_numbering.as_ref()
    }

    /// Check if the document has academic numbering configuration.
    #[must_use]
    pub fn has_academic_numbering(&self) -> bool {
        self.academic_numbering.is_some()
    }

    /// Set the academic numbering configuration.
    ///
    /// This configures how equations, theorems, algorithms, figures, and tables
    /// are numbered within the document.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is in an immutable state.
    pub fn set_academic_numbering(&mut self, config: NumberingConfig) -> Result<()> {
        if self.manifest.state.is_immutable() {
            return Err(crate::Error::InvalidManifest {
                reason: format!(
                    "Cannot set academic numbering in {} state",
                    self.manifest.state
                ),
            });
        }

        self.academic_numbering = Some(config);
        Ok(())
    }

    /// Remove the academic numbering configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is in an immutable state.
    pub fn clear_academic_numbering(&mut self) -> Result<()> {
        if self.manifest.state.is_immutable() {
            return Err(crate::Error::InvalidManifest {
                reason: format!(
                    "Cannot remove academic numbering in {} state",
                    self.manifest.state
                ),
            });
        }

        self.academic_numbering = None;
        Ok(())
    }

    // ===== Collaboration Extension Methods =====

    /// Get the collaboration comments, if present.
    #[must_use]
    pub fn comments(&self) -> Option<&CommentThread> {
        self.comments.as_ref()
    }

    /// Get a mutable reference to the comments.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is in an immutable state.
    pub fn comments_mut(&mut self) -> Result<Option<&mut CommentThread>> {
        if self.manifest.state.is_immutable() {
            return Err(crate::Error::InvalidManifest {
                reason: format!("Cannot modify comments in {} state", self.manifest.state),
            });
        }
        Ok(self.comments.as_mut())
    }

    /// Check if the document has collaboration comments.
    #[must_use]
    pub fn has_comments(&self) -> bool {
        self.comments.is_some()
    }

    /// Set the collaboration comments.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is in an immutable state.
    pub fn set_comments(&mut self, comments: CommentThread) -> Result<()> {
        if self.manifest.state.is_immutable() {
            return Err(crate::Error::InvalidManifest {
                reason: format!("Cannot set comments in {} state", self.manifest.state),
            });
        }

        self.comments = Some(comments);
        Ok(())
    }

    /// Remove the collaboration comments.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is in an immutable state.
    pub fn clear_comments(&mut self) -> Result<()> {
        if self.manifest.state.is_immutable() {
            return Err(crate::Error::InvalidManifest {
                reason: format!("Cannot remove comments in {} state", self.manifest.state),
            });
        }

        self.comments = None;
        Ok(())
    }

    // ===== Phantom Extension Methods =====

    /// Get the phantom clusters, if present.
    #[must_use]
    pub fn phantom_clusters(&self) -> Option<&PhantomClusters> {
        self.phantom_clusters.as_ref()
    }

    /// Get a mutable reference to phantom clusters.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is in an immutable state.
    pub fn phantom_clusters_mut(&mut self) -> Result<Option<&mut PhantomClusters>> {
        if self.manifest.state.is_immutable() {
            return Err(crate::Error::InvalidManifest {
                reason: format!(
                    "Cannot modify phantom clusters in {} state",
                    self.manifest.state
                ),
            });
        }
        Ok(self.phantom_clusters.as_mut())
    }

    /// Check if the document has phantom clusters.
    #[must_use]
    pub fn has_phantom_clusters(&self) -> bool {
        self.phantom_clusters.is_some()
    }

    /// Set the phantom clusters.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is in an immutable state.
    pub fn set_phantom_clusters(&mut self, clusters: PhantomClusters) -> Result<()> {
        if self.manifest.state.is_immutable() {
            return Err(crate::Error::InvalidManifest {
                reason: format!(
                    "Cannot set phantom clusters in {} state",
                    self.manifest.state
                ),
            });
        }

        self.phantom_clusters = Some(clusters);
        Ok(())
    }

    /// Remove the phantom clusters.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is in an immutable state.
    pub fn clear_phantom_clusters(&mut self) -> Result<()> {
        if self.manifest.state.is_immutable() {
            return Err(crate::Error::InvalidManifest {
                reason: format!(
                    "Cannot remove phantom clusters in {} state",
                    self.manifest.state
                ),
            });
        }

        self.phantom_clusters = None;
        Ok(())
    }

    // ===== Forms Extension Methods =====

    /// Get the form data, if present.
    #[must_use]
    pub fn form_data(&self) -> Option<&FormData> {
        self.form_data.as_ref()
    }

    /// Get a mutable reference to the form data.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is in an immutable state.
    pub fn form_data_mut(&mut self) -> Result<Option<&mut FormData>> {
        if self.manifest.state.is_immutable() {
            return Err(crate::Error::InvalidManifest {
                reason: format!("Cannot modify form data in {} state", self.manifest.state),
            });
        }
        Ok(self.form_data.as_mut())
    }

    /// Check if the document has form data.
    #[must_use]
    pub fn has_form_data(&self) -> bool {
        self.form_data.is_some()
    }

    /// Set the form data.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is in an immutable state.
    pub fn set_form_data(&mut self, form_data: FormData) -> Result<()> {
        if self.manifest.state.is_immutable() {
            return Err(crate::Error::InvalidManifest {
                reason: format!("Cannot set form data in {} state", self.manifest.state),
            });
        }

        self.form_data = Some(form_data);
        Ok(())
    }

    /// Remove the form data.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is in an immutable state.
    pub fn clear_form_data(&mut self) -> Result<()> {
        if self.manifest.state.is_immutable() {
            return Err(crate::Error::InvalidManifest {
                reason: format!("Cannot remove form data in {} state", self.manifest.state),
            });
        }

        self.form_data = None;
        Ok(())
    }

    // ===== Semantic Extension Methods =====

    /// Get the bibliography, if present.
    #[must_use]
    pub fn bibliography(&self) -> Option<&Bibliography> {
        self.bibliography.as_ref()
    }

    /// Get a mutable reference to the bibliography.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is in an immutable state.
    pub fn bibliography_mut(&mut self) -> Result<Option<&mut Bibliography>> {
        if self.manifest.state.is_immutable() {
            return Err(crate::Error::InvalidManifest {
                reason: format!(
                    "Cannot modify bibliography in {} state",
                    self.manifest.state
                ),
            });
        }
        Ok(self.bibliography.as_mut())
    }

    /// Check if the document has a bibliography.
    #[must_use]
    pub fn has_bibliography(&self) -> bool {
        self.bibliography.is_some()
    }

    /// Set the bibliography.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is in an immutable state.
    pub fn set_bibliography(&mut self, bibliography: Bibliography) -> Result<()> {
        if self.manifest.state.is_immutable() {
            return Err(crate::Error::InvalidManifest {
                reason: format!("Cannot set bibliography in {} state", self.manifest.state),
            });
        }

        self.bibliography = Some(bibliography);
        Ok(())
    }

    /// Remove the bibliography.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is in an immutable state.
    pub fn clear_bibliography(&mut self) -> Result<()> {
        if self.manifest.state.is_immutable() {
            return Err(crate::Error::InvalidManifest {
                reason: format!(
                    "Cannot remove bibliography in {} state",
                    self.manifest.state
                ),
            });
        }

        self.bibliography = None;
        Ok(())
    }

    /// Get the JSON-LD metadata, if present.
    #[must_use]
    pub fn jsonld_metadata(&self) -> Option<&JsonLdMetadata> {
        self.jsonld_metadata.as_ref()
    }

    /// Get a mutable reference to the JSON-LD metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is in an immutable state.
    pub fn jsonld_metadata_mut(&mut self) -> Result<Option<&mut JsonLdMetadata>> {
        if self.manifest.state.is_immutable() {
            return Err(crate::Error::InvalidManifest {
                reason: format!(
                    "Cannot modify JSON-LD metadata in {} state",
                    self.manifest.state
                ),
            });
        }
        Ok(self.jsonld_metadata.as_mut())
    }

    /// Check if the document has JSON-LD metadata.
    #[must_use]
    pub fn has_jsonld_metadata(&self) -> bool {
        self.jsonld_metadata.is_some()
    }

    /// Set the JSON-LD metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is in an immutable state.
    pub fn set_jsonld_metadata(&mut self, metadata: JsonLdMetadata) -> Result<()> {
        if self.manifest.state.is_immutable() {
            return Err(crate::Error::InvalidManifest {
                reason: format!(
                    "Cannot set JSON-LD metadata in {} state",
                    self.manifest.state
                ),
            });
        }

        self.jsonld_metadata = Some(metadata);
        Ok(())
    }

    /// Remove the JSON-LD metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is in an immutable state.
    pub fn clear_jsonld_metadata(&mut self) -> Result<()> {
        if self.manifest.state.is_immutable() {
            return Err(crate::Error::InvalidManifest {
                reason: format!(
                    "Cannot remove JSON-LD metadata in {} state",
                    self.manifest.state
                ),
            });
        }

        self.jsonld_metadata = None;
        Ok(())
    }

    /// Compute the document ID from content.
    ///
    /// The document ID is computed by hashing the canonicalized semantic content layer.
    /// This covers only the content blocks and their structure, not presentation/layout
    /// information. Presentation layers have their own hashes in the manifest.
    ///
    /// # Errors
    ///
    /// Returns an error if canonicalization fails.
    pub fn compute_id(&self) -> Result<DocumentId> {
        // Serialize content to canonical JSON
        let content_json = serde_json::to_vec(&self.content)?;
        let canonical =
            json_canon::to_string(&serde_json::from_slice::<serde_json::Value>(&content_json)?)?;

        Ok(Hasher::hash(
            self.manifest.hash_algorithm,
            canonical.as_bytes(),
        ))
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

    /// Validate extension declarations.
    ///
    /// This checks that all extension namespaces used in the document's content
    /// (blocks and marks) are declared in the manifest's extensions list.
    ///
    /// # Returns
    ///
    /// An `ExtensionValidationReport` containing:
    /// - List of used extension namespaces
    /// - List of declared extension namespaces
    /// - List of undeclared (used but not declared) namespaces
    /// - Warnings for any issues found
    #[must_use]
    pub fn validate_extensions(&self) -> ExtensionValidationReport {
        // Collect declared namespaces
        let declared_namespaces: Vec<String> = self
            .manifest
            .extensions
            .iter()
            .map(|e| e.namespace().to_string())
            .collect();

        // Collect used namespaces from content
        let mut used = std::collections::HashSet::new();
        Self::collect_extension_namespaces(&self.content.blocks, &mut used);

        let mut used_namespaces: Vec<String> = used.iter().cloned().collect();
        used_namespaces.sort();

        // Find undeclared namespaces
        let mut undeclared = Vec::new();
        let mut warnings = Vec::new();
        for namespace in &used_namespaces {
            if !self.manifest.has_extension(namespace) {
                undeclared.push(namespace.clone());
                warnings.push(format!(
                    "Extension namespace '{namespace}' is used but not declared in manifest"
                ));
            }
        }

        ExtensionValidationReport {
            used_namespaces,
            declared_namespaces,
            undeclared,
            unsupported_required: Vec::new(),
            warnings,
        }
    }

    /// Recursively collect extension namespaces from blocks.
    fn collect_extension_namespaces(
        blocks: &[Block],
        namespaces: &mut std::collections::HashSet<String>,
    ) {
        for block in blocks {
            // Check if this is an extension block
            if let Some(ext) = block.as_extension() {
                namespaces.insert(ext.namespace.clone());
            }

            // Recursively check children and collect marks from text nodes
            match block {
                Block::Paragraph { children, .. }
                | Block::Heading { children, .. }
                | Block::CodeBlock { children, .. }
                | Block::DefinitionTerm { children, .. } => {
                    Self::collect_marks_namespaces(children, namespaces);
                }
                Block::List { children, .. }
                | Block::ListItem { children, .. }
                | Block::Blockquote { children, .. }
                | Block::Table { children, .. }
                | Block::TableRow { children, .. }
                | Block::DefinitionItem { children, .. }
                | Block::DefinitionDescription { children, .. } => {
                    Self::collect_extension_namespaces(children, namespaces);
                }
                Block::DefinitionList(dl) => {
                    Self::collect_extension_namespaces(&dl.children, namespaces);
                }
                Block::TableCell(cell) => {
                    Self::collect_marks_namespaces(&cell.children, namespaces);
                }
                Block::Figure(fig) => {
                    Self::collect_extension_namespaces(&fig.children, namespaces);
                }
                Block::FigCaption(fc) => {
                    Self::collect_marks_namespaces(&fc.children, namespaces);
                }
                Block::Admonition(adm) => {
                    Self::collect_extension_namespaces(&adm.children, namespaces);
                }
                Block::Extension(ext) => {
                    // Already handled above, but also check children
                    Self::collect_extension_namespaces(&ext.children, namespaces);
                }
                // Leaf blocks without children
                Block::HorizontalRule { .. }
                | Block::Image(_)
                | Block::Math(_)
                | Block::Break { .. }
                | Block::Measurement(_)
                | Block::Signature(_)
                | Block::Svg(_)
                | Block::Barcode(_) => {}
            }
        }
    }

    /// Collect extension namespaces from text marks.
    fn collect_marks_namespaces(
        texts: &[Text],
        namespaces: &mut std::collections::HashSet<String>,
    ) {
        for text in texts {
            for mark in &text.marks {
                if let Some(ext) = mark.as_extension() {
                    namespaces.insert(ext.namespace.clone());
                }
            }
        }
    }

    // ===== Provenance & Proof Methods =====

    /// Generate a block index for this document.
    ///
    /// The block index contains hashes for all content blocks and the Merkle root.
    /// This is used for generating proofs and verifying content integrity.
    ///
    /// # Errors
    ///
    /// Returns an error if the content has no blocks.
    pub fn block_index(&self) -> Result<crate::provenance::BlockIndex> {
        crate::provenance::BlockIndex::from_content(&self.content, self.manifest.hash_algorithm)
    }

    /// Generate a Merkle proof for a specific block by index.
    ///
    /// The proof can be used to verify that a block is part of this document
    /// without revealing the entire document content.
    ///
    /// # Arguments
    ///
    /// * `block_index` - The zero-based index of the block to prove
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The content has no blocks
    /// - The block index is out of bounds
    pub fn prove_block(&self, block_index: usize) -> Result<crate::provenance::BlockProof> {
        let index = self.block_index()?;
        let hashes: Vec<_> = index.hashes().into_iter().cloned().collect();
        let tree =
            crate::provenance::MerkleTree::from_hashes(&hashes, self.manifest.hash_algorithm)?;
        tree.prove(block_index)
    }

    /// Generate a Merkle proof for a block by its ID.
    ///
    /// # Arguments
    ///
    /// * `block_id` - The ID of the block to prove
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The content has no blocks
    /// - No block with the given ID exists
    pub fn prove_block_by_id(&self, block_id: &str) -> Result<crate::provenance::BlockProof> {
        let index = self.block_index()?;
        let entry = index
            .find_block(block_id)
            .ok_or_else(|| crate::Error::InvalidManifest {
                reason: format!("Block with ID '{block_id}' not found"),
            })?;
        self.prove_block(entry.index)
    }

    /// Verify a block proof against this document.
    ///
    /// # Arguments
    ///
    /// * `proof` - The proof to verify
    /// * `block_hash` - The hash of the block being verified
    ///
    /// # Returns
    ///
    /// `true` if the proof is valid and the block is part of this document.
    #[must_use]
    pub fn verify_proof(
        &self,
        proof: &crate::provenance::BlockProof,
        block_hash: &crate::DocumentId,
    ) -> bool {
        // First verify the proof is internally consistent
        if !proof.verify(block_hash) {
            return false;
        }

        // Then verify the root matches this document's Merkle root
        if let Ok(index) = self.block_index() {
            proof.root_hash == *index.merkle_root()
        } else {
            false
        }
    }

    /// Get the Merkle root hash for this document's content.
    ///
    /// # Errors
    ///
    /// Returns an error if the content has no blocks.
    pub fn merkle_root(&self) -> Result<crate::DocumentId> {
        let index = self.block_index()?;
        Ok(index.merkle_root().clone())
    }

    /// Create a provenance record for this document.
    ///
    /// # Errors
    ///
    /// Returns an error if computing the document ID or Merkle root fails.
    pub fn provenance_record(&self) -> Result<crate::provenance::ProvenanceRecord> {
        let doc_id = if self.manifest.id.is_pending() {
            self.compute_id()?
        } else {
            self.manifest.id.clone()
        };

        let index = self.block_index()?;
        let merkle = crate::provenance::MerkleInfo::new(
            index.merkle_root().clone(),
            index.block_count(),
            self.manifest.hash_algorithm,
        );

        let mut record = crate::provenance::ProvenanceRecord::new(doc_id, merkle);

        // Add lineage if present
        if let Some(ref lineage) = self.manifest.lineage {
            record = record.with_lineage(lineage.clone());
        }

        Ok(record)
    }

    // ===== State Transition Methods =====

    /// Submit the document for review.
    ///
    /// Transitions from `draft` to `review` state. This computes the document ID
    /// and stores it in the manifest.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The document is not in draft state
    /// - Computing the document ID fails
    pub fn submit_for_review(&mut self) -> Result<()> {
        if self.manifest.state != DocumentState::Draft {
            return Err(crate::Error::InvalidStateTransition {
                from: self.manifest.state,
                to: DocumentState::Review,
            });
        }

        // Compute and store the document ID
        let doc_id = self.compute_id()?;
        self.manifest.id = doc_id;
        self.manifest.state = DocumentState::Review;
        self.manifest.modified = Utc::now();

        Ok(())
    }

    /// Freeze the document.
    ///
    /// Transitions from `review` to `frozen` state. This requires:
    /// - At least one signature
    /// - Lineage information (parent reference or explicit root)
    /// - At least one precise layout (for visual reproduction)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The document is not in review state
    /// - No signatures are present
    /// - No lineage is set
    /// - No precise layout is present
    pub fn freeze(&mut self) -> Result<()> {
        if self.manifest.state != DocumentState::Review {
            return Err(crate::Error::InvalidStateTransition {
                from: self.manifest.state,
                to: DocumentState::Frozen,
            });
        }

        // Verify requirements
        if !self.has_signatures() {
            return Err(crate::Error::StateRequirementNotMet {
                state: DocumentState::Frozen,
                requirement: "at least one signature".to_string(),
            });
        }

        if self.manifest.lineage.is_none() {
            return Err(crate::Error::StateRequirementNotMet {
                state: DocumentState::Frozen,
                requirement: "lineage information".to_string(),
            });
        }

        if !self.manifest.has_precise_layout() {
            return Err(crate::Error::StateRequirementNotMet {
                state: DocumentState::Frozen,
                requirement: "at least one precise layout".to_string(),
            });
        }

        // Ensure document ID is computed
        if self.manifest.id.is_pending() {
            let doc_id = self.compute_id()?;
            self.manifest.id = doc_id;
        }

        self.manifest.state = DocumentState::Frozen;
        self.manifest.modified = Utc::now();

        Ok(())
    }

    /// Publish the document.
    ///
    /// Transitions from `frozen` to `published` state.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is not in frozen state.
    pub fn publish(&mut self) -> Result<()> {
        if self.manifest.state != DocumentState::Frozen {
            return Err(crate::Error::InvalidStateTransition {
                from: self.manifest.state,
                to: DocumentState::Published,
            });
        }

        self.manifest.state = DocumentState::Published;
        self.manifest.modified = Utc::now();

        Ok(())
    }

    /// Revert the document to draft state.
    ///
    /// Transitions from `review` back to `draft` state. This is only allowed
    /// if the document has no signatures (to prevent removing signed content).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The document is not in review state
    /// - The document has signatures
    pub fn revert_to_draft(&mut self) -> Result<()> {
        if self.manifest.state != DocumentState::Review {
            return Err(crate::Error::InvalidStateTransition {
                from: self.manifest.state,
                to: DocumentState::Draft,
            });
        }

        if self.has_signatures() {
            return Err(crate::Error::InvalidManifest {
                reason: "Cannot revert to draft: document has signatures".to_string(),
            });
        }

        self.manifest.state = DocumentState::Draft;
        self.manifest.id = DocumentId::pending();
        self.manifest.modified = Utc::now();

        Ok(())
    }

    /// Fork the document to create a new draft with lineage.
    ///
    /// Creates a new document in draft state that references this document
    /// as its parent in the lineage chain. The forked document:
    /// - Has a new (pending) document ID
    /// - Is in draft state
    /// - Has lineage pointing to this document with ancestor chain
    /// - Has incremented version number and depth
    /// - Has no signatures
    ///
    /// # Errors
    ///
    /// Returns an error if computing the document ID fails.
    pub fn fork(&self) -> Result<Document> {
        // Compute the current document's ID for lineage
        let parent_id = if self.manifest.id.is_pending() {
            self.compute_id()?
        } else {
            self.manifest.id.clone()
        };

        // Create lineage using from_parent to properly track ancestors
        let lineage = Lineage::from_parent(parent_id, self.manifest.lineage.as_ref());

        // Clone the document
        let mut forked = self.clone();

        // Reset to draft state
        forked.manifest.id = DocumentId::pending();
        forked.manifest.state = DocumentState::Draft;
        forked.manifest.created = Utc::now();
        forked.manifest.modified = Utc::now();
        forked.manifest.lineage = Some(lineage);
        forked.manifest.security = None;
        #[cfg(feature = "signatures")]
        {
            forked.signature_file = None;
        }
        #[cfg(feature = "encryption")]
        {
            forked.encryption_metadata = None;
        }

        Ok(forked)
    }

    /// Set lineage information for this document.
    ///
    /// This is used to establish lineage before freezing a document.
    /// For the first version of a document, call with `None` as parent
    /// to create a root lineage entry.
    ///
    /// # Errors
    ///
    /// Returns an error if the document is in an immutable state.
    pub fn set_lineage(
        &mut self,
        parent: Option<DocumentId>,
        version: u32,
        note: Option<String>,
    ) -> Result<()> {
        if self.manifest.state.is_immutable() {
            return Err(crate::Error::InvalidManifest {
                reason: format!("Cannot modify lineage in {} state", self.manifest.state),
            });
        }

        let lineage = if let Some(parent_id) = parent {
            Lineage::from_parent(parent_id, None).with_note(note.unwrap_or_default())
        } else {
            let mut l = Lineage::root();
            l.version = Some(version);
            if let Some(n) = note {
                l = l.with_note(n);
            }
            l
        };

        self.manifest.lineage = Some(lineage);
        self.manifest.modified = Utc::now();

        Ok(())
    }

    /// Get a mutable reference to the manifest for advanced modifications.
    ///
    /// Use with caution - this bypasses state machine validation.
    #[must_use]
    pub fn manifest_mut(&mut self) -> &mut Manifest {
        &mut self.manifest
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

/// Report from extension validation.
///
/// This report identifies which extension namespaces are used in the document
/// content but not declared in the manifest's extensions list.
#[derive(Debug, Clone, Default)]
pub struct ExtensionValidationReport {
    /// Extension namespaces used in content (from blocks and marks).
    pub used_namespaces: Vec<String>,
    /// Extension namespaces that are declared in the manifest.
    pub declared_namespaces: Vec<String>,
    /// Extension namespaces used but not declared.
    pub undeclared: Vec<String>,
    /// Extension namespaces declared as required but not supported by this reader.
    /// (Currently empty since we support all built-in extensions)
    pub unsupported_required: Vec<String>,
    /// Warning messages.
    pub warnings: Vec<String>,
}

impl ExtensionValidationReport {
    /// Check if extension validation passed without warnings.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.undeclared.is_empty() && self.unsupported_required.is_empty()
    }

    /// Check if there are any warnings.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
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
    content_override: Option<Content>,
    dublin_core_override: Option<DublinCore>,
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
            content_override: None,
            dublin_core_override: None,
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

    /// Set pre-built content, overriding any blocks added via `add_block()`.
    #[must_use]
    pub fn with_content(mut self, content: Content) -> Self {
        self.content_override = Some(content);
        self
    }

    /// Set pre-built Dublin Core metadata, overriding title/creator/description/language.
    #[must_use]
    pub fn with_dublin_core(mut self, dublin_core: DublinCore) -> Self {
        self.dublin_core_override = Some(dublin_core);
        self
    }

    /// Build the document.
    ///
    /// # Errors
    ///
    /// Returns an error if the document cannot be constructed.
    pub fn build(self) -> Result<Document> {
        use crate::manifest::{ContentRef, Metadata};

        let content = self
            .content_override
            .unwrap_or_else(|| Content::new(self.blocks));
        let dublin_core = self.dublin_core_override.unwrap_or_else(|| {
            let mut dc = DublinCore::new(&self.title, &self.creator);
            dc.terms.description = self.description;
            dc.terms.language = self.language;
            dc
        });

        let content_ref = ContentRef {
            path: CONTENT_PATH.to_string(),
            hash: DocumentId::pending(),
            compression: Some("deflate".to_string()),
            merkle_root: None,
            block_count: None,
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
            #[cfg(feature = "signatures")]
            signature_file: None,
            #[cfg(feature = "encryption")]
            encryption_metadata: None,
            academic_numbering: None,
            comments: None,
            phantom_clusters: None,
            form_data: None,
            bibliography: None,
            jsonld_metadata: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::Mark;

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

    // Extension validation tests

    #[test]
    fn test_extension_validation_no_extensions() {
        let doc = Document::builder()
            .title("Simple Doc")
            .creator("Author")
            .add_paragraph("Just plain text")
            .build()
            .unwrap();

        let report = doc.validate_extensions();
        assert!(report.is_valid());
        assert!(report.used_namespaces.is_empty());
        assert!(report.undeclared.is_empty());
    }

    #[test]
    fn test_extension_validation_with_extension_block() {
        use crate::extensions::ExtensionBlock;

        let ext_block = Block::Extension(
            ExtensionBlock::new("forms", "textInput")
                .with_id("name-field")
                .with_attributes(serde_json::json!({"label": "Name"})),
        );

        let content = Content::new(vec![
            Block::paragraph(vec![Text::plain("Fill out this form:")]),
            ext_block,
        ]);

        let doc = Document::builder()
            .title("Form Doc")
            .creator("Author")
            .with_content(content)
            .build()
            .unwrap();

        let report = doc.validate_extensions();
        assert!(!report.is_valid()); // undeclared extension
        assert_eq!(report.used_namespaces, vec!["forms"]);
        assert_eq!(report.undeclared, vec!["forms"]);
        assert!(report.warnings[0].contains("forms"));
    }

    #[test]
    fn test_extension_validation_with_declared_extension() {
        use crate::extensions::ExtensionBlock;
        use crate::manifest::Extension;

        let ext_block = Block::Extension(
            ExtensionBlock::new("semantic", "citation")
                .with_attributes(serde_json::json!({"ref": "smith2023"})),
        );

        let content = Content::new(vec![
            Block::paragraph(vec![Text::plain("According to research")]),
            ext_block,
        ]);

        let mut doc = Document::builder()
            .title("Academic Doc")
            .creator("Author")
            .with_content(content)
            .build()
            .unwrap();

        // Declare the extension
        doc.manifest_mut()
            .extensions
            .push(Extension::required("codex.semantic", "0.1"));

        let report = doc.validate_extensions();
        assert!(report.is_valid());
        assert_eq!(report.used_namespaces, vec!["semantic"]);
        assert!(report.undeclared.is_empty());
    }

    #[test]
    fn test_extension_validation_with_extension_marks() {
        use crate::content::ExtensionMark;

        let citation_mark = Mark::Extension(ExtensionMark::citation("smith2023"));
        let text_with_citation = Text::with_marks("important finding", vec![citation_mark]);

        let content = Content::new(vec![Block::paragraph(vec![text_with_citation])]);

        let doc = Document::builder()
            .title("Cited Doc")
            .creator("Author")
            .with_content(content)
            .build()
            .unwrap();

        let report = doc.validate_extensions();
        assert!(!report.is_valid()); // undeclared
        assert_eq!(report.used_namespaces, vec!["semantic"]);
        assert_eq!(report.undeclared, vec!["semantic"]);
    }

    #[test]
    fn test_extension_validation_mixed() {
        use crate::content::ExtensionMark;
        use crate::extensions::ExtensionBlock;
        use crate::manifest::Extension;

        // Create content with multiple extensions
        let citation_mark = Mark::Extension(ExtensionMark::citation("smith2023"));
        let entity_mark =
            Mark::Extension(ExtensionMark::entity("https://wikidata.org/Q937", "person"));

        let form_block = Block::Extension(
            ExtensionBlock::new("forms", "textInput")
                .with_id("email")
                .with_attributes(serde_json::json!({"label": "Email"})),
        );

        let content = Content::new(vec![
            Block::paragraph(vec![
                Text::with_marks("Einstein", vec![entity_mark]),
                Text::plain(" published his theory "),
                Text::with_marks("(ref)", vec![citation_mark]),
            ]),
            form_block,
        ]);

        let mut doc = Document::builder()
            .title("Mixed Extensions")
            .creator("Author")
            .with_content(content)
            .build()
            .unwrap();

        // Only declare semantic, not forms
        doc.manifest_mut()
            .extensions
            .push(Extension::required("codex.semantic", "0.1"));

        let report = doc.validate_extensions();
        assert!(!report.is_valid()); // forms not declared
        assert!(report.used_namespaces.contains(&"semantic".to_string()));
        assert!(report.used_namespaces.contains(&"forms".to_string()));
        assert_eq!(report.undeclared, vec!["forms"]);
        assert!(report.warnings.len() == 1);
    }

    #[test]
    fn test_extension_validation_report_methods() {
        let mut report = ExtensionValidationReport::default();
        assert!(report.is_valid());
        assert!(!report.has_warnings());

        report.undeclared.push("test".to_string());
        report.warnings.push("Test warning".to_string());
        assert!(!report.is_valid());
        assert!(report.has_warnings());
    }

    // ===== Extension File I/O Tests =====

    #[test]
    fn test_comments_round_trip() {
        use crate::extensions::{Collaborator, Comment, CommentThread};

        let mut doc = Document::builder()
            .title("Comments Test")
            .creator("Author")
            .add_paragraph("Content to comment on")
            .build()
            .unwrap();

        // Create a comment thread
        let mut thread = CommentThread::new();
        let author = Collaborator::new("Alice");
        let comment = Comment::new("c1", "block-1", author, "This is a test comment");
        thread.add(comment);

        // Set comments
        doc.set_comments(thread).unwrap();
        assert!(doc.has_comments());

        // Round-trip through bytes
        let bytes = doc.to_bytes().unwrap();
        let loaded = Document::from_bytes(bytes).unwrap();

        assert!(loaded.has_comments());
        let loaded_thread = loaded.comments().unwrap();
        assert_eq!(loaded_thread.comments.len(), 1);
        assert_eq!(loaded_thread.comments[0].id, "c1");
        assert_eq!(loaded_thread.comments[0].content, "This is a test comment");
    }

    #[test]
    fn test_phantom_clusters_round_trip() {
        use crate::anchor::ContentAnchor;
        use crate::extensions::{
            Phantom, PhantomCluster, PhantomClusters, PhantomContent, PhantomPosition, PhantomScope,
        };

        let mut doc = Document::builder()
            .title("Phantoms Test")
            .creator("Author")
            .add_paragraph("Content with phantoms")
            .build()
            .unwrap();

        // Create phantom clusters
        let mut clusters = PhantomClusters::new();
        let position = PhantomPosition::new(100.0, 200.0);
        let content = PhantomContent::paragraph("Alternative text");
        let phantom = Phantom::new("phantom-1", position, content);
        let cluster =
            PhantomCluster::new("cluster-1", ContentAnchor::block("block-1"), "Test cluster")
                .with_phantom(phantom)
                .with_scope(PhantomScope::Shared);
        clusters.add_cluster(cluster);

        // Set phantom clusters
        doc.set_phantom_clusters(clusters).unwrap();
        assert!(doc.has_phantom_clusters());

        // Round-trip through bytes
        let bytes = doc.to_bytes().unwrap();
        let loaded = Document::from_bytes(bytes).unwrap();

        assert!(loaded.has_phantom_clusters());
        let loaded_clusters = loaded.phantom_clusters().unwrap();
        assert_eq!(loaded_clusters.len(), 1);
        assert_eq!(loaded_clusters.clusters[0].id, "cluster-1");
    }

    #[test]
    fn test_form_data_round_trip() {
        use crate::extensions::FormData;

        let mut doc = Document::builder()
            .title("Form Data Test")
            .creator("Author")
            .add_paragraph("Form content")
            .build()
            .unwrap();

        // Create form data
        let mut form_data = FormData::new();
        form_data.set("name", serde_json::json!("John Doe"));
        form_data.set("email", serde_json::json!("john@example.com"));
        form_data.set("age", serde_json::json!(30));

        // Set form data
        doc.set_form_data(form_data).unwrap();
        assert!(doc.has_form_data());

        // Round-trip through bytes
        let bytes = doc.to_bytes().unwrap();
        let loaded = Document::from_bytes(bytes).unwrap();

        assert!(loaded.has_form_data());
        let loaded_form = loaded.form_data().unwrap();
        assert_eq!(
            loaded_form.get("name"),
            Some(&serde_json::json!("John Doe"))
        );
        assert_eq!(
            loaded_form.get("email"),
            Some(&serde_json::json!("john@example.com"))
        );
        assert_eq!(loaded_form.get("age"), Some(&serde_json::json!(30)));
    }

    #[test]
    fn test_bibliography_round_trip() {
        use crate::extensions::{Bibliography, BibliographyEntry, CitationStyle, EntryType};

        let mut doc = Document::builder()
            .title("Bibliography Test")
            .creator("Author")
            .add_paragraph("Content with citations")
            .build()
            .unwrap();

        // Create bibliography
        let mut bibliography = Bibliography::new(CitationStyle::Apa);
        let entry = BibliographyEntry::new("smith2023", EntryType::Article, "Test Article");
        bibliography.add_entry(entry);

        // Set bibliography
        doc.set_bibliography(bibliography).unwrap();
        assert!(doc.has_bibliography());

        // Round-trip through bytes
        let bytes = doc.to_bytes().unwrap();
        let loaded = Document::from_bytes(bytes).unwrap();

        assert!(loaded.has_bibliography());
        let loaded_bib = loaded.bibliography().unwrap();
        assert_eq!(loaded_bib.len(), 1);
        assert_eq!(loaded_bib.style, CitationStyle::Apa);
        assert!(loaded_bib.contains("smith2023"));
    }

    #[test]
    fn test_jsonld_round_trip() {
        use crate::extensions::JsonLdMetadata;
        use serde_json::json;

        let mut doc = Document::builder()
            .title("JSON-LD Test")
            .creator("Author")
            .add_paragraph("Content with structured data")
            .build()
            .unwrap();

        // Create JSON-LD metadata
        let mut jsonld = JsonLdMetadata::new();
        jsonld.add_node(json!({
            "@type": "Article",
            "name": "Test Article",
            "author": {
                "@type": "Person",
                "name": "Test Author"
            }
        }));

        // Set JSON-LD metadata
        doc.set_jsonld_metadata(jsonld).unwrap();
        assert!(doc.has_jsonld_metadata());

        // Round-trip through bytes
        let bytes = doc.to_bytes().unwrap();
        let loaded = Document::from_bytes(bytes).unwrap();

        assert!(loaded.has_jsonld_metadata());
        let loaded_jsonld = loaded.jsonld_metadata().unwrap();
        assert_eq!(loaded_jsonld.graph.len(), 1);
        assert!(loaded_jsonld
            .context
            .contains(&"https://schema.org".to_string()));
    }

    #[test]
    fn test_clear_extension_data() {
        use crate::extensions::{Bibliography, CitationStyle, CommentThread, FormData};

        let mut doc = Document::builder()
            .title("Clear Test")
            .creator("Author")
            .add_paragraph("Content")
            .build()
            .unwrap();

        // Set all extension data
        doc.set_comments(CommentThread::new()).unwrap();
        doc.set_form_data(FormData::new()).unwrap();
        doc.set_bibliography(Bibliography::new(CitationStyle::Chicago))
            .unwrap();

        assert!(doc.has_comments());
        assert!(doc.has_form_data());
        assert!(doc.has_bibliography());

        // Clear each
        doc.clear_comments().unwrap();
        doc.clear_form_data().unwrap();
        doc.clear_bibliography().unwrap();

        assert!(!doc.has_comments());
        assert!(!doc.has_form_data());
        assert!(!doc.has_bibliography());
    }

    #[test]
    fn test_extension_data_mutable_access() {
        use crate::extensions::{Collaborator, Comment, CommentThread};

        let mut doc = Document::builder()
            .title("Mutable Test")
            .creator("Author")
            .add_paragraph("Content")
            .build()
            .unwrap();

        // Set initial comments
        let mut thread = CommentThread::new();
        let author = Collaborator::new("Alice");
        thread.add(Comment::new("c1", "block-1", author.clone(), "First"));
        doc.set_comments(thread).unwrap();

        // Modify through mutable reference
        if let Some(comments) = doc.comments_mut().unwrap() {
            comments.add(Comment::new("c2", "block-2", author, "Second"));
        }

        assert_eq!(doc.comments().unwrap().comments.len(), 2);
    }
}
