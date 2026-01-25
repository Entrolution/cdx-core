//! Block-level proofs for selective disclosure.

use serde::{Deserialize, Serialize};

use crate::{DocumentId, HashAlgorithm, Hasher};

/// A Merkle proof for a specific block.
///
/// This proof allows verification that a block is part of a document
/// without revealing the entire document content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockProof {
    /// Index of the block in the document.
    pub index: usize,

    /// Path from leaf to root: (`sibling_hash`, `is_right_sibling`).
    pub path: Vec<(DocumentId, bool)>,

    /// Expected root hash.
    pub root_hash: DocumentId,

    /// Hash algorithm used.
    pub algorithm: HashAlgorithm,
}

impl BlockProof {
    /// Verify that a block hash is part of the document.
    ///
    /// # Arguments
    ///
    /// * `block_hash` - The hash of the block to verify
    ///
    /// # Returns
    ///
    /// `true` if the proof is valid, `false` otherwise.
    #[must_use]
    pub fn verify(&self, block_hash: &DocumentId) -> bool {
        let mut current = block_hash.clone();

        for (sibling, is_right) in &self.path {
            let combined = if *is_right {
                // Sibling is on right, we are on left
                format!("{}{}", current.hex_digest(), sibling.hex_digest())
            } else {
                // Sibling is on left, we are on right
                format!("{}{}", sibling.hex_digest(), current.hex_digest())
            };

            current = Hasher::hash(self.algorithm, combined.as_bytes());
        }

        current == self.root_hash
    }

    /// Create a verification result with details.
    #[must_use]
    pub fn verify_detailed(&self, block_hash: &DocumentId) -> ProofVerification {
        let valid = self.verify(block_hash);
        ProofVerification {
            valid,
            index: self.index,
            root_hash: self.root_hash.clone(),
            error: if valid {
                None
            } else {
                Some("Computed root hash does not match expected".to_string())
            },
        }
    }
}

/// Result of proof verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofVerification {
    /// Whether the proof is valid.
    pub valid: bool,

    /// Block index that was verified.
    pub index: usize,

    /// Expected root hash.
    pub root_hash: DocumentId,

    /// Error message if verification failed.
    pub error: Option<String>,
}

impl ProofVerification {
    /// Check if verification passed.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.valid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provenance::MerkleTree;

    #[test]
    fn test_proof_verification() {
        let items = vec!["block0", "block1", "block2", "block3"];
        let tree = MerkleTree::from_items(&items, HashAlgorithm::Sha256).unwrap();

        // Generate proof for block 1
        let proof = tree.prove(1).unwrap();

        // Compute hash of block 1
        let block_hash = Hasher::hash(HashAlgorithm::Sha256, b"block1");

        // Verify
        assert!(proof.verify(&block_hash));
    }

    #[test]
    fn test_proof_fails_wrong_block() {
        let items = vec!["block0", "block1", "block2", "block3"];
        let tree = MerkleTree::from_items(&items, HashAlgorithm::Sha256).unwrap();

        let proof = tree.prove(1).unwrap();

        // Try to verify with wrong block hash
        let wrong_hash = Hasher::hash(HashAlgorithm::Sha256, b"wrong_block");
        assert!(!proof.verify(&wrong_hash));
    }

    #[test]
    fn test_proof_verification_detailed() {
        let items = vec!["a", "b", "c", "d"];
        let tree = MerkleTree::from_items(&items, HashAlgorithm::Sha256).unwrap();

        let proof = tree.prove(2).unwrap();
        let block_hash = Hasher::hash(HashAlgorithm::Sha256, b"c");

        let result = proof.verify_detailed(&block_hash);
        assert!(result.is_valid());
        assert_eq!(result.index, 2);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_proof_serialization() {
        let items = vec!["x", "y"];
        let tree = MerkleTree::from_items(&items, HashAlgorithm::Sha256).unwrap();

        let proof = tree.prove(0).unwrap();

        let json = serde_json::to_string_pretty(&proof).unwrap();
        assert!(json.contains("\"index\": 0"));

        let deserialized: BlockProof = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.index, proof.index);
    }

    #[test]
    fn test_all_blocks_verifiable() {
        let items: Vec<&str> = vec!["0", "1", "2", "3", "4", "5", "6", "7"];
        let tree = MerkleTree::from_items(&items, HashAlgorithm::Sha256).unwrap();

        for (i, item) in items.iter().enumerate() {
            let proof = tree.prove(i).unwrap();
            let block_hash = Hasher::hash(HashAlgorithm::Sha256, item.as_bytes());
            assert!(proof.verify(&block_hash), "Proof failed for block {i}");
        }
    }
}
