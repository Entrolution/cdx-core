use crate::Result;

use super::Document;

impl Document {
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
            .ok_or_else(|| crate::Error::ValidationFailed {
                reason: format!("block with ID '{block_id}' not found"),
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
}
