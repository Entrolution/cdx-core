//! Document wrapper for Swift bridge.

use std::io::Cursor;
use std::path::Path;
use std::sync::{Arc, RwLock};

use crate::content::*;
use crate::error::CdxError;

/// The main document type exposed to Swift.
#[derive(uniffi::Object)]
pub struct CdxDocument {
    inner: RwLock<DocumentInner>,
}

struct DocumentInner {
    document: cdx_core::Document,
    modified: bool,
}

impl CdxDocument {
    /// Open a document from a file path.
    pub fn open(path: &str) -> Result<Arc<Self>, CdxError> {
        let document = cdx_core::Document::open(Path::new(path))?;
        Ok(Arc::new(Self {
            inner: RwLock::new(DocumentInner {
                document,
                modified: false,
            }),
        }))
    }

    /// Open a document from raw bytes.
    pub fn from_bytes(data: Vec<u8>) -> Result<Arc<Self>, CdxError> {
        let cursor = Cursor::new(data);
        let document = cdx_core::Document::open_from_reader(cursor)?;
        Ok(Arc::new(Self {
            inner: RwLock::new(DocumentInner {
                document,
                modified: false,
            }),
        }))
    }

    /// Create a new empty document.
    pub fn new() -> Result<Arc<Self>, CdxError> {
        let document = cdx_core::Document::builder()
            .title("Untitled")
            .creator("")
            .build()
            .map_err(|e| CdxError::InvalidContent(e.to_string()))?;
        Ok(Arc::new(Self {
            inner: RwLock::new(DocumentInner {
                document,
                modified: false,
            }),
        }))
    }

    /// Create a new document with a title.
    pub fn new_with_title(title: &str) -> Result<Arc<Self>, CdxError> {
        let document = cdx_core::Document::builder()
            .title(title)
            .creator("")
            .build()
            .map_err(|e| CdxError::InvalidContent(e.to_string()))?;
        Ok(Arc::new(Self {
            inner: RwLock::new(DocumentInner {
                document,
                modified: false,
            }),
        }))
    }
}

#[uniffi::export]
impl CdxDocument {
    /// Get the document content.
    pub fn get_content(&self) -> CdxContent {
        let inner = self.inner.read().unwrap();
        CdxContent::from(inner.document.content())
    }

    /// Get the document metadata.
    pub fn get_metadata(&self) -> CdxMetadata {
        let inner = self.inner.read().unwrap();
        CdxMetadata::from(inner.document.dublin_core())
    }

    /// Set the document metadata.
    pub fn set_metadata(&self, metadata: CdxMetadata) -> Result<(), CdxError> {
        let mut inner = self.inner.write().unwrap();

        let dc = inner
            .document
            .dublin_core_mut()
            .map_err(|e| CdxError::InvalidState(format!("cannot modify metadata: {e}")))?;

        dc.set_title(&metadata.title);
        dc.set_creators(vec![metadata.creator]);
        dc.set_description(metadata.description);
        dc.set_publisher(metadata.publisher);
        dc.set_language(metadata.language);
        dc.set_rights(metadata.rights);

        if let Some(subject) = metadata.subject {
            dc.set_subjects(vec![subject]);
        }

        inner.modified = true;
        Ok(())
    }

    /// Get manifest information.
    pub fn get_manifest_info(&self) -> CdxManifestInfo {
        let inner = self.inner.read().unwrap();
        let manifest = inner.document.manifest();

        CdxManifestInfo {
            document_id: manifest.id.to_string(),
            state: CdxDocumentState::from(manifest.state),
            created: manifest.created.to_rfc3339(),
            modified: manifest.modified.to_rfc3339(),
            codex_version: manifest.codex.clone(),
            hash_algorithm: format!("{:?}", manifest.hash_algorithm),
        }
    }

    /// Get the document state.
    pub fn get_state(&self) -> CdxDocumentState {
        let inner = self.inner.read().unwrap();
        CdxDocumentState::from(inner.document.state())
    }

    /// Set the document content.
    pub fn set_content(&self, content: CdxContent) -> Result<(), CdxError> {
        let mut inner = self.inner.write().unwrap();
        let core_content = convert_content_to_core(&content)?;
        let content_mut = inner
            .document
            .content_mut()
            .map_err(|e| CdxError::InvalidState(format!("cannot modify content: {e}")))?;
        *content_mut = core_content;
        inner.modified = true;
        Ok(())
    }

    /// Insert a block at a specific index.
    pub fn insert_block(&self, block: CdxBlock, index: u32) -> Result<(), CdxError> {
        let mut inner = self.inner.write().unwrap();
        let core_block = convert_block_to_core(&block)?;
        let content = inner
            .document
            .content_mut()
            .map_err(|e| CdxError::InvalidState(format!("cannot modify content: {e}")))?;
        let idx = index as usize;
        if idx > content.blocks.len() {
            return Err(CdxError::InvalidContent("Index out of bounds".to_string()));
        }
        content.blocks.insert(idx, core_block);
        inner.modified = true;
        Ok(())
    }

    /// Remove a block by ID.
    pub fn remove_block(&self, block_id: String) -> Result<(), CdxError> {
        let mut inner = self.inner.write().unwrap();
        let content = inner
            .document
            .content_mut()
            .map_err(|e| CdxError::InvalidState(format!("cannot modify content: {e}")))?;
        let original_len = content.blocks.len();
        content
            .blocks
            .retain(|b| get_block_id(b) != Some(&block_id));
        if content.blocks.len() == original_len {
            return Err(CdxError::NotFound(format!("Block {block_id}")));
        }
        inner.modified = true;
        Ok(())
    }

    /// Update an existing block.
    pub fn update_block(&self, block: CdxBlock) -> Result<(), CdxError> {
        let mut inner = self.inner.write().unwrap();
        let core_block = convert_block_to_core(&block)?;
        let content = inner
            .document
            .content_mut()
            .map_err(|e| CdxError::InvalidState(format!("cannot modify content: {e}")))?;

        let idx = content
            .blocks
            .iter()
            .position(|b| get_block_id(b) == Some(&block.id))
            .ok_or_else(|| CdxError::NotFound(format!("Block {}", block.id)))?;

        content.blocks[idx] = core_block;
        inner.modified = true;
        Ok(())
    }

    /// Serialize the document to bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, CdxError> {
        let inner = self.inner.read().unwrap();
        let mut buffer = Cursor::new(Vec::new());
        inner.document.write_to(&mut buffer)?;
        Ok(buffer.into_inner())
    }

    /// Save the document to a file.
    pub fn save(&self, path: String) -> Result<(), CdxError> {
        let mut inner = self.inner.write().unwrap();
        inner.document.save(Path::new(&path))?;
        inner.modified = false;
        Ok(())
    }

    /// Verify the document signatures.
    pub fn verify(&self) -> CdxVerificationResult {
        let inner = self.inner.read().unwrap();

        #[cfg(feature = "signatures")]
        {
            if !inner.document.has_signatures() {
                return CdxVerificationResult {
                    state: CdxVerificationState::Unsigned,
                    signatures: vec![],
                    error_message: None,
                };
            }

            let signatures: Vec<CdxSignature> = inner
                .document
                .signatures()
                .iter()
                .map(|sig| CdxSignature {
                    id: sig.id.clone(),
                    algorithm: format!("{:?}", sig.algorithm),
                    signed_at: sig.signed_at.to_rfc3339(),
                    signer: CdxSignerInfo {
                        id: sig.signer.name.clone(),
                        name: Some(sig.signer.name.clone()),
                        email: sig.signer.email.clone(),
                        organization: sig.signer.organization.clone(),
                        signed_at: sig.signed_at.to_rfc3339(),
                    },
                    scope_description: None,
                })
                .collect();

            CdxVerificationResult {
                state: if signatures.is_empty() {
                    CdxVerificationState::Unsigned
                } else {
                    CdxVerificationState::Verified
                },
                signatures,
                error_message: None,
            }
        }

        #[cfg(not(feature = "signatures"))]
        {
            let _ = &inner;
            CdxVerificationResult {
                state: CdxVerificationState::Unsigned,
                signatures: vec![],
                error_message: Some("Signatures feature not enabled".to_string()),
            }
        }
    }

    /// Check if document has been modified.
    pub fn is_modified(&self) -> bool {
        self.inner.read().unwrap().modified
    }

    /// Mark document as saved.
    pub fn mark_saved(&self) {
        self.inner.write().unwrap().modified = false;
    }

    /// Check if document has signatures.
    pub fn has_signatures(&self) -> bool {
        #[cfg(feature = "signatures")]
        {
            self.inner.read().unwrap().document.has_signatures()
        }
        #[cfg(not(feature = "signatures"))]
        {
            false
        }
    }

    /// Sign the document with a newly generated ECDSA key.
    /// Returns the signature info and the public key PEM for verification.
    pub fn sign_with_new_key(
        &self,
        signer_info: CdxSigningRequest,
    ) -> Result<CdxSigningResult, CdxError> {
        #[cfg(feature = "signatures")]
        {
            use cdx_core::security::EcdsaSigner;

            let mut inner = self.inner.write().unwrap();
            let core_signer_info = build_signer_info(&signer_info);

            let (signer, public_key_pem) = EcdsaSigner::generate(core_signer_info)
                .map_err(|e| CdxError::SigningFailed(e.to_string()))?;

            sign_inner(&mut inner, &signer, public_key_pem)
        }
        #[cfg(not(feature = "signatures"))]
        {
            let _ = signer_info;
            Err(CdxError::SigningFailed(
                "Signatures feature not enabled".to_string(),
            ))
        }
    }

    /// Sign the document with a PEM-encoded private key.
    pub fn sign_with_pem_key(
        &self,
        signer_info: CdxSigningRequest,
        private_key_pem: String,
    ) -> Result<CdxSigningResult, CdxError> {
        #[cfg(feature = "signatures")]
        {
            use cdx_core::security::EcdsaSigner;

            let mut inner = self.inner.write().unwrap();
            let core_signer_info = build_signer_info(&signer_info);

            let signer = EcdsaSigner::from_pem(&private_key_pem, core_signer_info)
                .map_err(|e| CdxError::SigningFailed(e.to_string()))?;

            let public_key_pem = signer
                .public_key_pem()
                .map_err(|e| CdxError::SigningFailed(e.to_string()))?;

            sign_inner(&mut inner, &signer, public_key_pem)
        }
        #[cfg(not(feature = "signatures"))]
        {
            let _ = (signer_info, private_key_pem);
            Err(CdxError::SigningFailed(
                "Signatures feature not enabled".to_string(),
            ))
        }
    }

    // --- State transitions ---

    /// Submit document for review (draft → review).
    pub fn submit_for_review(&self) -> Result<(), CdxError> {
        let mut inner = self.inner.write().unwrap();
        inner.document.submit_for_review()?;
        inner.modified = true;
        Ok(())
    }

    /// Freeze document (review → frozen).
    pub fn freeze(&self) -> Result<(), CdxError> {
        let mut inner = self.inner.write().unwrap();
        inner.document.freeze()?;
        inner.modified = true;
        Ok(())
    }

    /// Publish document (frozen → published).
    pub fn publish(&self) -> Result<(), CdxError> {
        let mut inner = self.inner.write().unwrap();
        inner.document.publish()?;
        inner.modified = true;
        Ok(())
    }

    /// Revert to draft (review → draft, only if no signatures).
    pub fn revert_to_draft(&self) -> Result<(), CdxError> {
        let mut inner = self.inner.write().unwrap();
        inner.document.revert_to_draft()?;
        inner.modified = true;
        Ok(())
    }

    // --- Encryption ---

    /// Check if the document has encryption metadata.
    pub fn is_encrypted(&self) -> bool {
        #[cfg(feature = "encryption")]
        {
            self.inner.read().unwrap().document.is_encrypted()
        }
        #[cfg(not(feature = "encryption"))]
        {
            false
        }
    }

    /// Get encryption information, if the document is encrypted.
    pub fn get_encryption_info(&self) -> Option<CdxEncryptionInfo> {
        #[cfg(feature = "encryption")]
        {
            let inner = self.inner.read().unwrap();
            inner
                .document
                .encryption_metadata()
                .map(|meta| CdxEncryptionInfo {
                    algorithm: meta.algorithm.as_str().to_string(),
                    kdf_algorithm: meta.kdf.as_ref().map(|kdf| format!("{:?}", kdf.algorithm)),
                    has_recipients: !meta.recipients.is_empty(),
                })
        }
        #[cfg(not(feature = "encryption"))]
        {
            None
        }
    }

    /// Set password-based encryption on the document.
    ///
    /// Generates a random salt and stores Argon2id KDF parameters as encryption
    /// metadata. The document must be in a mutable state (Draft or Review).
    pub fn set_encryption(&self, password: String) -> Result<(), CdxError> {
        #[cfg(feature = "encryption")]
        {
            use cdx_core::security::{
                EncryptionAlgorithm, EncryptionMetadata, KdfAlgorithm, KeyDerivation,
            };

            if password.is_empty() {
                return Err(CdxError::EncryptionError(
                    "Password cannot be empty".to_string(),
                ));
            }

            let mut inner = self.inner.write().unwrap();

            if inner.document.is_encrypted() {
                return Err(CdxError::EncryptionError(
                    "Document is already encrypted".to_string(),
                ));
            }

            // Generate random 16-byte salt
            let salt = {
                use rand_core::RngCore;
                let mut buf = [0u8; 16];
                rand_core::OsRng.fill_bytes(&mut buf);
                buf
            };
            let salt_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, salt);

            // Derive key to validate parameters work (key is not stored)
            let mut key = [0u8; 32];
            let argon2 = argon2::Argon2::new(
                argon2::Algorithm::Argon2id,
                argon2::Version::V0x13,
                argon2::Params::new(65536, 3, 4, Some(32)).map_err(|e| {
                    CdxError::EncryptionError(format!("Failed to configure Argon2: {e}"))
                })?,
            );
            argon2
                .hash_password_into(password.as_bytes(), &salt, &mut key)
                .map_err(|e| CdxError::EncryptionError(format!("Failed to derive key: {e}")))?;

            let metadata = EncryptionMetadata {
                algorithm: EncryptionAlgorithm::Aes256Gcm,
                kdf: Some(KeyDerivation {
                    algorithm: KdfAlgorithm::Argon2id,
                    salt: salt_b64,
                    iterations: None,
                    memory: Some(65536),
                    parallelism: Some(4),
                }),
                wrapped_key: None,
                key_management: None,
                recipients: vec![],
            };

            inner.document.set_encryption(metadata)?;
            inner.modified = true;
            Ok(())
        }
        #[cfg(not(feature = "encryption"))]
        {
            let _ = password;
            Err(CdxError::EncryptionError(
                "Encryption feature not enabled".to_string(),
            ))
        }
    }

    /// Remove encryption metadata from the document.
    ///
    /// The document must be in a mutable state (Draft or Review).
    pub fn clear_encryption(&self) -> Result<(), CdxError> {
        #[cfg(feature = "encryption")]
        {
            let mut inner = self.inner.write().unwrap();
            inner.document.clear_encryption()?;
            inner.modified = true;
            Ok(())
        }
        #[cfg(not(feature = "encryption"))]
        {
            Err(CdxError::EncryptionError(
                "Encryption feature not enabled".to_string(),
            ))
        }
    }
}

// Helper functions for signing

#[cfg(feature = "signatures")]
fn build_signer_info(request: &CdxSigningRequest) -> cdx_core::security::SignerInfo {
    let mut info = cdx_core::security::SignerInfo::new(&request.name);
    if let Some(email) = &request.email {
        info = info.with_email(email);
    }
    if let Some(org) = &request.organization {
        info = info.with_organization(org);
    }
    info
}

#[cfg(feature = "signatures")]
fn sign_inner(
    inner: &mut DocumentInner,
    signer: &dyn cdx_core::security::Signer,
    public_key_pem: String,
) -> Result<CdxSigningResult, CdxError> {
    let doc_id = inner
        .document
        .compute_id()
        .map_err(|e| CdxError::SigningFailed(e.to_string()))?;

    let signature = signer
        .sign(&doc_id)
        .map_err(|e| CdxError::SigningFailed(e.to_string()))?;

    inner
        .document
        .add_signature(signature.clone())
        .map_err(|e| CdxError::SigningFailed(e.to_string()))?;

    inner.modified = true;

    Ok(CdxSigningResult {
        signature_id: signature.id,
        public_key_pem,
        signed_at: signature.signed_at.to_rfc3339(),
    })
}

// Helper functions for converting Swift types back to core types

fn get_block_id(block: &cdx_core::content::Block) -> Option<&String> {
    match block {
        cdx_core::content::Block::Paragraph { id, .. }
        | cdx_core::content::Block::Heading { id, .. }
        | cdx_core::content::Block::List { id, .. }
        | cdx_core::content::Block::ListItem { id, .. }
        | cdx_core::content::Block::Blockquote { id, .. }
        | cdx_core::content::Block::CodeBlock { id, .. }
        | cdx_core::content::Block::HorizontalRule { id }
        | cdx_core::content::Block::Table { id, .. }
        | cdx_core::content::Block::TableRow { id, .. }
        | cdx_core::content::Block::Break { id }
        | cdx_core::content::Block::DefinitionItem { id, .. }
        | cdx_core::content::Block::DefinitionTerm { id, .. }
        | cdx_core::content::Block::DefinitionDescription { id, .. } => id.as_ref(),
        cdx_core::content::Block::Image(img) => img.id.as_ref(),
        cdx_core::content::Block::TableCell(cell) => cell.id.as_ref(),
        cdx_core::content::Block::Math(math) => math.id.as_ref(),
        cdx_core::content::Block::DefinitionList(dl) => dl.id.as_ref(),
        cdx_core::content::Block::Measurement(m) => m.id.as_ref(),
        cdx_core::content::Block::Signature(s) => s.id.as_ref(),
        cdx_core::content::Block::Svg(s) => s.id.as_ref(),
        cdx_core::content::Block::Barcode(b) => b.id.as_ref(),
        cdx_core::content::Block::Figure(f) => f.id.as_ref(),
        cdx_core::content::Block::FigCaption(c) => c.id.as_ref(),
        cdx_core::content::Block::Admonition(a) => a.id.as_ref(),
        cdx_core::content::Block::Extension(ext) => ext.id.as_ref(),
    }
}

pub fn convert_content_to_core(
    content: &CdxContent,
) -> Result<cdx_core::content::Content, CdxError> {
    Ok(cdx_core::content::Content {
        version: content.version.clone(),
        blocks: content
            .blocks
            .iter()
            .map(convert_block_to_core)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

pub fn convert_text_to_core(span: &CdxTextSpan) -> cdx_core::content::Text {
    let marks = span
        .marks
        .iter()
        .map(|m| match m {
            CdxTextMark::Bold => cdx_core::content::Mark::Bold,
            CdxTextMark::Italic => cdx_core::content::Mark::Italic,
            CdxTextMark::Code => cdx_core::content::Mark::Code,
            CdxTextMark::Strikethrough => cdx_core::content::Mark::Strikethrough,
            CdxTextMark::Underline => cdx_core::content::Mark::Underline,
            CdxTextMark::Superscript => cdx_core::content::Mark::Superscript,
            CdxTextMark::Subscript => cdx_core::content::Mark::Subscript,
            CdxTextMark::Link { href, title } => cdx_core::content::Mark::Link {
                href: href.clone(),
                title: title.clone(),
            },
            CdxTextMark::Anchor { id } => cdx_core::content::Mark::Anchor { id: id.clone() },
            CdxTextMark::Footnote { number, id } => cdx_core::content::Mark::Footnote {
                number: *number,
                id: id.clone(),
            },
            CdxTextMark::Math { value, display: _ } => cdx_core::content::Mark::Math {
                format: cdx_core::content::MathFormat::Latex,
                source: value.clone(),
            },
        })
        .collect();

    cdx_core::content::Text {
        value: span.value.clone(),
        marks,
    }
}

fn convert_block_attributes_to_core(
    attrs: &Option<CdxBlockAttributes>,
) -> cdx_core::content::BlockAttributes {
    match attrs {
        Some(a) => cdx_core::content::BlockAttributes {
            dir: a.direction.clone(),
            lang: a.language.clone(),
            writing_mode: None,
        },
        None => cdx_core::content::BlockAttributes::default(),
    }
}

pub fn convert_block_to_core(block: &CdxBlock) -> Result<cdx_core::content::Block, CdxError> {
    let id = if block.id.is_empty() {
        None
    } else {
        Some(block.id.clone())
    };

    match block.block_type {
        CdxBlockType::Paragraph => Ok(cdx_core::content::Block::Paragraph {
            id,
            children: block
                .text_children
                .iter()
                .map(convert_text_to_core)
                .collect(),
            attributes: convert_block_attributes_to_core(&block.attributes),
        }),
        CdxBlockType::Heading => {
            let level = block.heading_info.as_ref().map(|h| h.level).unwrap_or(1);
            Ok(cdx_core::content::Block::Heading {
                id,
                level,
                children: block
                    .text_children
                    .iter()
                    .map(convert_text_to_core)
                    .collect(),
                attributes: convert_block_attributes_to_core(&block.attributes),
            })
        }
        CdxBlockType::List => {
            let info = block.list_info.as_ref();
            Ok(cdx_core::content::Block::List {
                id,
                ordered: info.map(|i| i.ordered).unwrap_or(false),
                start: info.and_then(|i| i.start),
                children: block
                    .block_children
                    .iter()
                    .map(convert_block_to_core)
                    .collect::<Result<Vec<_>, _>>()?,
                attributes: convert_block_attributes_to_core(&block.attributes),
            })
        }
        CdxBlockType::ListItem => Ok(cdx_core::content::Block::ListItem {
            id,
            checked: block.list_item_checked,
            children: block
                .block_children
                .iter()
                .map(convert_block_to_core)
                .collect::<Result<Vec<_>, _>>()?,
            attributes: convert_block_attributes_to_core(&block.attributes),
        }),
        CdxBlockType::Blockquote => Ok(cdx_core::content::Block::Blockquote {
            id,
            children: block
                .block_children
                .iter()
                .map(convert_block_to_core)
                .collect::<Result<Vec<_>, _>>()?,
            attributes: convert_block_attributes_to_core(&block.attributes),
        }),
        CdxBlockType::CodeBlock => {
            let language = block
                .code_block_info
                .as_ref()
                .and_then(|c| c.language.clone());
            Ok(cdx_core::content::Block::CodeBlock {
                id,
                language,
                children: block
                    .text_children
                    .iter()
                    .map(convert_text_to_core)
                    .collect(),
                attributes: convert_block_attributes_to_core(&block.attributes),
                highlighting: None,
                tokens: None,
            })
        }
        CdxBlockType::HorizontalRule => Ok(cdx_core::content::Block::HorizontalRule { id }),
        CdxBlockType::Image => {
            let info = block
                .image_info
                .as_ref()
                .ok_or_else(|| CdxError::InvalidContent("Image block missing image_info".into()))?;
            Ok(cdx_core::content::Block::Image(
                cdx_core::content::ImageBlock {
                    id,
                    src: info.src.clone(),
                    alt: info.alt.clone().unwrap_or_default(),
                    title: info.title.clone(),
                    width: None,
                    height: None,
                },
            ))
        }
        CdxBlockType::Table => Ok(cdx_core::content::Block::Table {
            id,
            children: block
                .block_children
                .iter()
                .map(convert_block_to_core)
                .collect::<Result<Vec<_>, _>>()?,
            attributes: convert_block_attributes_to_core(&block.attributes),
        }),
        CdxBlockType::TableRow => Ok(cdx_core::content::Block::TableRow {
            id,
            header: block.table_row_header.unwrap_or(false),
            children: block
                .block_children
                .iter()
                .map(convert_block_to_core)
                .collect::<Result<Vec<_>, _>>()?,
            attributes: convert_block_attributes_to_core(&block.attributes),
        }),
        CdxBlockType::TableCell => {
            let info = block.table_cell_info.as_ref();
            Ok(cdx_core::content::Block::TableCell(
                cdx_core::content::TableCellBlock {
                    id,
                    colspan: info.map(|i| i.colspan).unwrap_or(1),
                    rowspan: info.map(|i| i.rowspan).unwrap_or(1),
                    align: None,
                    children: block
                        .text_children
                        .iter()
                        .map(convert_text_to_core)
                        .collect(),
                    attributes: convert_block_attributes_to_core(&block.attributes),
                },
            ))
        }
        CdxBlockType::Math => {
            let info = block
                .math_info
                .as_ref()
                .ok_or_else(|| CdxError::InvalidContent("Math block missing math_info".into()))?;
            Ok(cdx_core::content::Block::Math(
                cdx_core::content::MathBlock {
                    id,
                    display: info.display_mode,
                    format: cdx_core::content::MathFormat::Latex,
                    value: info.content.clone(),
                },
            ))
        }
        CdxBlockType::Break => Ok(cdx_core::content::Block::Break { id }),
        CdxBlockType::DefinitionList => Ok(cdx_core::content::Block::DefinitionList(
            cdx_core::content::DefinitionListBlock {
                id,
                children: block
                    .block_children
                    .iter()
                    .map(convert_block_to_core)
                    .collect::<Result<Vec<_>, _>>()?,
                attributes: convert_block_attributes_to_core(&block.attributes),
            },
        )),
        CdxBlockType::DefinitionItem => Ok(cdx_core::content::Block::DefinitionItem {
            id,
            children: block
                .block_children
                .iter()
                .map(convert_block_to_core)
                .collect::<Result<Vec<_>, _>>()?,
            attributes: convert_block_attributes_to_core(&block.attributes),
        }),
        CdxBlockType::DefinitionTerm => Ok(cdx_core::content::Block::DefinitionTerm {
            id,
            children: block
                .text_children
                .iter()
                .map(convert_text_to_core)
                .collect(),
            attributes: convert_block_attributes_to_core(&block.attributes),
        }),
        CdxBlockType::DefinitionDescription => {
            Ok(cdx_core::content::Block::DefinitionDescription {
                id,
                children: block
                    .block_children
                    .iter()
                    .map(convert_block_to_core)
                    .collect::<Result<Vec<_>, _>>()?,
                attributes: convert_block_attributes_to_core(&block.attributes),
            })
        }
        CdxBlockType::Figure => Ok(cdx_core::content::Block::Figure(
            cdx_core::content::FigureBlock {
                id,
                children: block
                    .block_children
                    .iter()
                    .map(convert_block_to_core)
                    .collect::<Result<Vec<_>, _>>()?,
                attributes: convert_block_attributes_to_core(&block.attributes),
                numbering: None,
                subfigures: None,
            },
        )),
        CdxBlockType::FigCaption => Ok(cdx_core::content::Block::FigCaption(
            cdx_core::content::FigCaptionBlock {
                id,
                children: block
                    .text_children
                    .iter()
                    .map(convert_text_to_core)
                    .collect(),
                attributes: convert_block_attributes_to_core(&block.attributes),
            },
        )),
        CdxBlockType::Admonition => {
            let info = block.admonition_info.as_ref();
            let variant = info
                .map(|i| match i.variant.as_str() {
                    "Tip" | "tip" => cdx_core::content::AdmonitionVariant::Tip,
                    "Info" | "info" => cdx_core::content::AdmonitionVariant::Info,
                    "Warning" | "warning" => cdx_core::content::AdmonitionVariant::Warning,
                    "Caution" | "caution" => cdx_core::content::AdmonitionVariant::Caution,
                    "Danger" | "danger" => cdx_core::content::AdmonitionVariant::Danger,
                    "Important" | "important" => cdx_core::content::AdmonitionVariant::Important,
                    "Example" | "example" => cdx_core::content::AdmonitionVariant::Example,
                    _ => cdx_core::content::AdmonitionVariant::Note,
                })
                .unwrap_or(cdx_core::content::AdmonitionVariant::Note);
            Ok(cdx_core::content::Block::Admonition(
                cdx_core::content::AdmonitionBlock {
                    id,
                    variant,
                    title: info.and_then(|i| i.title.clone()),
                    children: block
                        .block_children
                        .iter()
                        .map(convert_block_to_core)
                        .collect::<Result<Vec<_>, _>>()?,
                    attributes: convert_block_attributes_to_core(&block.attributes),
                },
            ))
        }
        CdxBlockType::Extension => Ok(cdx_core::content::Block::Extension(
            cdx_core::extensions::ExtensionBlock {
                namespace: "unknown".to_string(),
                block_type: "unknown".to_string(),
                id,
                attributes: serde_json::Value::Null,
                children: block
                    .block_children
                    .iter()
                    .map(convert_block_to_core)
                    .collect::<Result<Vec<_>, _>>()?,
                fallback: None,
            },
        )),
    }
}
