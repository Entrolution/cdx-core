#[cfg(feature = "encryption")]
use crate::security::EncryptionMetadata;

#[cfg(feature = "signatures")]
use crate::archive::SIGNATURES_PATH;
#[cfg(feature = "signatures")]
use crate::manifest::SecurityRef;
#[cfg(feature = "signatures")]
use crate::security::{Signature, SignatureFile};

use crate::Result;

use super::Document;
use super::MutableResource;

impl Document {
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
        self.require_mutable("set encryption")?;
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
        self.require_mutable("remove encryption")?;
        self.encryption_metadata = None;
        Ok(())
    }
}
