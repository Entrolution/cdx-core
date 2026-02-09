use chrono::Utc;

use crate::manifest::Lineage;
use crate::{DocumentId, DocumentState, Result};

use super::Document;
use super::MutableResource;

impl Document {
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
            return Err(crate::Error::ValidationFailed {
                reason: "cannot revert to draft: document has signatures".to_string(),
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
        self.require_mutable("modify lineage")?;

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
}
