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
use crate::archive::{CdxReader, CdxWriter, CompressionMethod, CONTENT_PATH, DUBLIN_CORE_PATH};
use crate::content::{Block, Content, Text};
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

        Ok(Self {
            manifest,
            content,
            dublin_core,
            #[cfg(feature = "signatures")]
            signature_file,
            #[cfg(feature = "encryption")]
            encryption_metadata,
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
