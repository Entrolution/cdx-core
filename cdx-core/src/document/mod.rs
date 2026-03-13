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

mod builder;
mod extensions;
mod io;
mod provenance;
mod security;
mod state;
mod verification;

#[cfg(test)]
mod tests;

pub use builder::DocumentBuilder;
pub use verification::{ExtensionValidationReport, VerificationReport};

use chrono::Utc;

use crate::content::Content;
use crate::metadata::DublinCore;
use crate::{DocumentId, DocumentState, HashAlgorithm, Manifest, Result};

#[cfg(feature = "signatures")]
use crate::security::SignatureFile;

#[cfg(feature = "encryption")]
use crate::security::EncryptionMetadata;

use crate::extensions::academic::NumberingConfig;
use crate::extensions::{Bibliography, CommentThread, FormData, JsonLdMetadata, PhantomClusters};

/// Trait for resources that can be in mutable or immutable states.
///
/// This trait provides a common pattern for checking if a resource
/// can be modified and returning an appropriate error if not.
trait MutableResource {
    /// Get the current document state.
    fn state(&self) -> DocumentState;

    /// Check if the resource can be modified, returning an error if not.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ImmutableDocument`] if the resource is in an immutable state.
    fn require_mutable(&self, action: &str) -> Result<()> {
        if self.state().is_immutable() {
            return Err(crate::Error::ImmutableDocument {
                action: action.to_string(),
                state: self.state(),
            });
        }
        Ok(())
    }
}

/// Generates the five standard accessor methods for an optional extension field:
/// - `$field(&self) -> Option<&$type>` — immutable access
/// - `$field_mut(&mut self) -> Result<Option<&mut $type>>` — mutable access
/// - `has_$field(&self) -> bool` — presence check
/// - `set_$field(&mut self, value: $type) -> Result<()>` — set value
/// - `clear_$field(&mut self) -> Result<()>` — remove value
macro_rules! define_extension_accessors {
    ($field:ident, $field_mut:ident, $has:ident, $set:ident, $clear:ident, $type:ty, $label:expr) => {
        #[doc = concat!("Get the ", $label, ", if present.")]
        #[must_use]
        pub fn $field(&self) -> Option<&$type> {
            self.$field.as_ref()
        }

        #[doc = concat!("Get a mutable reference to the ", $label, ".\n\n# Errors\n\nReturns an error if the document is in an immutable state.")]
        pub fn $field_mut(&mut self) -> Result<Option<&mut $type>> {
            self.require_mutable(concat!("modify ", $label))?;
            self.manifest.modified = chrono::Utc::now();
            Ok(self.$field.as_mut())
        }

        #[doc = concat!("Check if the document has ", $label, ".")]
        #[must_use]
        pub fn $has(&self) -> bool {
            self.$field.is_some()
        }

        #[doc = concat!("Set the ", $label, ".\n\n# Errors\n\nReturns an error if the document is in an immutable state.")]
        pub fn $set(&mut self, value: $type) -> Result<()> {
            self.require_mutable(concat!("set ", $label))?;
            self.$field = Some(value);
            self.manifest.modified = chrono::Utc::now();
            Ok(())
        }

        #[doc = concat!("Remove the ", $label, ".\n\n# Errors\n\nReturns an error if the document is in an immutable state.")]
        pub fn $clear(&mut self) -> Result<()> {
            self.require_mutable(concat!("remove ", $label))?;
            self.$field = None;
            self.manifest.modified = chrono::Utc::now();
            Ok(())
        }
    };
}

// Make macro available to submodules
pub(crate) use define_extension_accessors;

impl MutableResource for Document {
    fn state(&self) -> DocumentState {
        self.manifest.state
    }
}

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
    /// Create a new document builder.
    #[must_use]
    pub fn builder() -> DocumentBuilder {
        DocumentBuilder::new()
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
        self.require_mutable("modify content")?;
        self.manifest.modified = Utc::now();
        // Reset document ID so freeze() recomputes it from the modified content
        if !self.manifest.id.is_pending() {
            self.manifest.id = DocumentId::pending();
        }
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
        self.require_mutable("modify Dublin Core metadata")?;
        self.manifest.modified = Utc::now();
        // Reset document ID so freeze() recomputes it from the modified metadata
        if !self.manifest.id.is_pending() {
            self.manifest.id = DocumentId::pending();
        }
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

    /// Get a mutable reference to the manifest for advanced modifications.
    ///
    /// Use with caution - this bypasses state machine validation.
    #[must_use]
    pub fn manifest_mut(&mut self) -> &mut Manifest {
        &mut self.manifest
    }
}
