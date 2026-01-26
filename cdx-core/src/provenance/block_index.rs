//! Block index for content integrity verification.
//!
//! The block index (`content/block-index.json`) stores individual block hashes
//! for Merkle proof generation and selective disclosure.

use serde::{Deserialize, Serialize};

use crate::content::Content;
use crate::{DocumentId, HashAlgorithm, Hasher};

/// Index of content blocks with their hashes.
///
/// This structure is stored at `content/block-index.json` and enables:
/// - Merkle proof generation for individual blocks
/// - Efficient verification of specific content sections
/// - Selective disclosure without revealing full content
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockIndex {
    /// Version of the block index format.
    pub version: String,

    /// Hash algorithm used for all hashes.
    pub algorithm: HashAlgorithm,

    /// Merkle root hash of all blocks.
    pub root: DocumentId,

    /// Individual block entries with their hashes.
    pub blocks: Vec<BlockHashEntry>,
}

impl BlockIndex {
    /// Current version of the block index format.
    pub const VERSION: &'static str = "0.1";

    /// Create a block index from document content.
    ///
    /// This computes hashes for each block and builds a Merkle tree
    /// to derive the root hash.
    ///
    /// # Errors
    ///
    /// Returns an error if the content has no blocks.
    pub fn from_content(content: &Content, algorithm: HashAlgorithm) -> crate::Result<Self> {
        let blocks = &content.blocks;
        if blocks.is_empty() {
            return Err(crate::Error::InvalidManifest {
                reason: "Cannot create block index from empty content".to_string(),
            });
        }

        // Compute hash for each block
        let mut entries = Vec::with_capacity(blocks.len());
        let mut hashes = Vec::with_capacity(blocks.len());

        for (index, block) in blocks.iter().enumerate() {
            // Serialize block to canonical JSON for hashing
            let block_json = serde_json::to_vec(block)?;
            let canonical =
                json_canon::to_string(&serde_json::from_slice::<serde_json::Value>(&block_json)?)?;
            let hash = Hasher::hash(algorithm, canonical.as_bytes());

            entries.push(BlockHashEntry {
                id: block
                    .id()
                    .map_or_else(|| format!("block-{index}"), String::from),
                hash: hash.clone(),
                index,
            });
            hashes.push(hash);
        }

        // Build Merkle tree to get root
        let tree = super::MerkleTree::from_hashes(&hashes, algorithm)?;

        Ok(Self {
            version: Self::VERSION.to_string(),
            algorithm,
            root: tree.root_hash().clone(),
            blocks: entries,
        })
    }

    /// Get the Merkle root hash.
    #[must_use]
    pub fn merkle_root(&self) -> &DocumentId {
        &self.root
    }

    /// Get the number of blocks.
    #[must_use]
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    /// Find a block entry by ID.
    #[must_use]
    pub fn find_block(&self, id: &str) -> Option<&BlockHashEntry> {
        self.blocks.iter().find(|b| b.id == id)
    }

    /// Find a block entry by index.
    #[must_use]
    pub fn get_block(&self, index: usize) -> Option<&BlockHashEntry> {
        self.blocks.get(index)
    }

    /// Get all block hashes in order.
    #[must_use]
    pub fn hashes(&self) -> Vec<&DocumentId> {
        self.blocks.iter().map(|b| &b.hash).collect()
    }

    /// Serialize to JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_json(&self) -> crate::Result<String> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }

    /// Deserialize from JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails.
    pub fn from_json(json: &str) -> crate::Result<Self> {
        serde_json::from_str(json).map_err(Into::into)
    }
}

/// Entry for a single block in the index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockHashEntry {
    /// Block identifier.
    pub id: String,

    /// Hash of the canonicalized block JSON.
    pub hash: DocumentId,

    /// Position in the block list (0-indexed).
    pub index: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{Block, Text};

    fn create_test_content() -> Content {
        Content::new(vec![
            Block::heading(1, vec![Text::plain("Title")]),
            Block::paragraph(vec![Text::plain("First paragraph.")]),
            Block::paragraph(vec![Text::plain("Second paragraph.")]),
        ])
    }

    #[test]
    fn test_block_index_creation() {
        let content = create_test_content();
        let index = BlockIndex::from_content(&content, HashAlgorithm::Sha256).unwrap();

        assert_eq!(index.version, "0.1");
        assert_eq!(index.algorithm, HashAlgorithm::Sha256);
        assert_eq!(index.blocks.len(), 3);
        assert!(!index.root.is_pending());
    }

    #[test]
    fn test_block_index_deterministic() {
        let content = create_test_content();
        let index1 = BlockIndex::from_content(&content, HashAlgorithm::Sha256).unwrap();
        let index2 = BlockIndex::from_content(&content, HashAlgorithm::Sha256).unwrap();

        assert_eq!(index1.root, index2.root);
        assert_eq!(index1.blocks, index2.blocks);
    }

    #[test]
    fn test_block_index_find_block() {
        let content = create_test_content();
        let index = BlockIndex::from_content(&content, HashAlgorithm::Sha256).unwrap();

        // Find by index
        let entry = index.get_block(1).unwrap();
        assert_eq!(entry.index, 1);

        // Verify hash is not pending
        assert!(!entry.hash.is_pending());
    }

    #[test]
    fn test_block_index_empty_content_fails() {
        let content = Content::new(vec![]);
        let result = BlockIndex::from_content(&content, HashAlgorithm::Sha256);
        assert!(result.is_err());
    }

    #[test]
    fn test_block_index_serialization() {
        let content = create_test_content();
        let index = BlockIndex::from_content(&content, HashAlgorithm::Sha256).unwrap();

        let json = index.to_json().unwrap();
        assert!(json.contains("\"version\": \"0.1\""));
        assert!(json.contains("\"algorithm\": \"sha256\""));

        let deserialized = BlockIndex::from_json(&json).unwrap();
        assert_eq!(deserialized.root, index.root);
        assert_eq!(deserialized.blocks.len(), index.blocks.len());
    }

    #[test]
    fn test_block_index_content_changes_root() {
        let content1 = Content::new(vec![Block::paragraph(vec![Text::plain("Hello")])]);
        let content2 = Content::new(vec![Block::paragraph(vec![Text::plain("World")])]);

        let index1 = BlockIndex::from_content(&content1, HashAlgorithm::Sha256).unwrap();
        let index2 = BlockIndex::from_content(&content2, HashAlgorithm::Sha256).unwrap();

        assert_ne!(index1.root, index2.root);
    }
}
