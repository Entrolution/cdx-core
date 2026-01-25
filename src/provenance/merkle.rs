//! Merkle tree implementation for content integrity.

use serde::{Deserialize, Serialize};

use crate::{DocumentId, HashAlgorithm, Hasher, Result};

/// A node in a Merkle tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MerkleNode {
    /// Hash of this node.
    pub hash: DocumentId,

    /// Left child hash (None for leaf nodes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left: Option<Box<MerkleNode>>,

    /// Right child hash (None for leaf nodes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right: Option<Box<MerkleNode>>,
}

impl MerkleNode {
    /// Create a leaf node from data.
    #[must_use]
    pub fn leaf(hash: DocumentId) -> Self {
        Self {
            hash,
            left: None,
            right: None,
        }
    }

    /// Create a branch node from two children.
    #[must_use]
    pub fn branch(left: MerkleNode, right: MerkleNode, algorithm: HashAlgorithm) -> Self {
        // Combine child hashes to compute parent hash
        let combined = format!("{}{}", left.hash.hex_digest(), right.hash.hex_digest());
        let hash = Hasher::hash(algorithm, combined.as_bytes());

        Self {
            hash,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        }
    }

    /// Check if this is a leaf node.
    #[must_use]
    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }
}

/// A Merkle tree for content blocks.
///
/// Merkle trees enable efficient verification of content integrity
/// and support selective disclosure proofs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MerkleTree {
    /// Root node of the tree.
    root: MerkleNode,

    /// Hash algorithm used.
    algorithm: HashAlgorithm,

    /// Number of leaf nodes.
    leaf_count: usize,
}

impl MerkleTree {
    /// Build a Merkle tree from a list of data items.
    ///
    /// # Errors
    ///
    /// Returns an error if the items list is empty.
    ///
    /// # Panics
    ///
    /// This function will not panic if the items list is non-empty.
    pub fn from_items<T: AsRef<[u8]>>(items: &[T], algorithm: HashAlgorithm) -> Result<Self> {
        if items.is_empty() {
            return Err(crate::Error::InvalidManifest {
                reason: "Cannot build Merkle tree from empty items".to_string(),
            });
        }

        let leaf_count = items.len();

        // Create leaf nodes
        let mut nodes: Vec<MerkleNode> = items
            .iter()
            .map(|item| MerkleNode::leaf(Hasher::hash(algorithm, item.as_ref())))
            .collect();

        // Build tree bottom-up
        while nodes.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in nodes.chunks(2) {
                if chunk.len() == 2 {
                    let branch = MerkleNode::branch(chunk[0].clone(), chunk[1].clone(), algorithm);
                    next_level.push(branch);
                } else {
                    // Odd node: duplicate it to pair with itself
                    let branch = MerkleNode::branch(chunk[0].clone(), chunk[0].clone(), algorithm);
                    next_level.push(branch);
                }
            }

            nodes = next_level;
        }

        Ok(Self {
            root: nodes.into_iter().next().expect("nodes should not be empty"),
            algorithm,
            leaf_count,
        })
    }

    /// Build a Merkle tree from pre-computed hashes.
    ///
    /// # Errors
    ///
    /// Returns an error if the hashes list is empty.
    ///
    /// # Panics
    ///
    /// This function will not panic if the hashes list is non-empty.
    pub fn from_hashes(hashes: &[DocumentId], algorithm: HashAlgorithm) -> Result<Self> {
        if hashes.is_empty() {
            return Err(crate::Error::InvalidManifest {
                reason: "Cannot build Merkle tree from empty hashes".to_string(),
            });
        }

        let leaf_count = hashes.len();

        // Create leaf nodes
        let mut nodes: Vec<MerkleNode> =
            hashes.iter().map(|h| MerkleNode::leaf(h.clone())).collect();

        // Build tree bottom-up
        while nodes.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in nodes.chunks(2) {
                if chunk.len() == 2 {
                    let branch = MerkleNode::branch(chunk[0].clone(), chunk[1].clone(), algorithm);
                    next_level.push(branch);
                } else {
                    // Odd node: duplicate it
                    let branch = MerkleNode::branch(chunk[0].clone(), chunk[0].clone(), algorithm);
                    next_level.push(branch);
                }
            }

            nodes = next_level;
        }

        Ok(Self {
            root: nodes.into_iter().next().expect("nodes should not be empty"),
            algorithm,
            leaf_count,
        })
    }

    /// Get the root hash of the tree.
    #[must_use]
    pub fn root_hash(&self) -> &DocumentId {
        &self.root.hash
    }

    /// Get the root node.
    #[must_use]
    pub fn root(&self) -> &MerkleNode {
        &self.root
    }

    /// Get the hash algorithm used.
    #[must_use]
    pub fn algorithm(&self) -> HashAlgorithm {
        self.algorithm
    }

    /// Get the number of leaf nodes.
    #[must_use]
    pub fn leaf_count(&self) -> usize {
        self.leaf_count
    }

    /// Generate a proof for a leaf at the given index.
    ///
    /// # Errors
    ///
    /// Returns an error if the index is out of bounds.
    pub fn prove(&self, index: usize) -> Result<super::BlockProof> {
        if index >= self.leaf_count {
            return Err(crate::Error::InvalidManifest {
                reason: format!(
                    "Index {} out of bounds for tree with {} leaves",
                    index, self.leaf_count
                ),
            });
        }

        let mut path = Vec::new();

        // Collect sibling hashes along the path to root
        collect_proof_path(&self.root, index, 0, self.leaf_count, &mut path);

        Ok(super::BlockProof {
            index,
            path,
            root_hash: self.root.hash.clone(),
            algorithm: self.algorithm,
        })
    }
}

fn collect_proof_path(
    node: &MerkleNode,
    target_index: usize,
    current_start: usize,
    level_size: usize,
    path: &mut Vec<(DocumentId, bool)>,
) {
    if node.is_leaf() {
        return;
    }

    let mid = current_start + level_size / 2;
    let left = node
        .left
        .as_ref()
        .expect("branch node should have left child");
    let right = node
        .right
        .as_ref()
        .expect("branch node should have right child");

    if target_index < mid {
        // Target is in left subtree, recurse first then add sibling
        collect_proof_path(left, target_index, current_start, level_size / 2, path);
        // Add right sibling to path (sibling is on right)
        path.push((right.hash.clone(), true));
    } else {
        // Target is in right subtree, recurse first then add sibling
        collect_proof_path(right, target_index, mid, level_size / 2, path);
        // Add left sibling to path (sibling is on left)
        path.push((left.hash.clone(), false));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_tree_from_items() {
        let items = vec!["item1", "item2", "item3", "item4"];
        let tree = MerkleTree::from_items(&items, HashAlgorithm::Sha256).unwrap();

        assert_eq!(tree.leaf_count(), 4);
        assert!(!tree.root_hash().is_pending());
    }

    #[test]
    fn test_merkle_tree_odd_count() {
        let items = vec!["item1", "item2", "item3"];
        let tree = MerkleTree::from_items(&items, HashAlgorithm::Sha256).unwrap();

        assert_eq!(tree.leaf_count(), 3);
    }

    #[test]
    fn test_merkle_tree_single_item() {
        let items = vec!["single"];
        let tree = MerkleTree::from_items(&items, HashAlgorithm::Sha256).unwrap();

        assert_eq!(tree.leaf_count(), 1);
        // Root should be hash of the single item (doubled for branch)
    }

    #[test]
    fn test_merkle_tree_empty_fails() {
        let items: Vec<&str> = vec![];
        let result = MerkleTree::from_items(&items, HashAlgorithm::Sha256);
        assert!(result.is_err());
    }

    #[test]
    fn test_merkle_tree_deterministic() {
        let items = vec!["a", "b", "c", "d"];
        let tree1 = MerkleTree::from_items(&items, HashAlgorithm::Sha256).unwrap();
        let tree2 = MerkleTree::from_items(&items, HashAlgorithm::Sha256).unwrap();

        assert_eq!(tree1.root_hash(), tree2.root_hash());
    }

    #[test]
    fn test_merkle_tree_changes_with_content() {
        let items1 = vec!["a", "b", "c", "d"];
        let items2 = vec!["a", "b", "c", "e"];

        let tree1 = MerkleTree::from_items(&items1, HashAlgorithm::Sha256).unwrap();
        let tree2 = MerkleTree::from_items(&items2, HashAlgorithm::Sha256).unwrap();

        assert_ne!(tree1.root_hash(), tree2.root_hash());
    }

    #[test]
    fn test_generate_proof() {
        let items = vec!["item0", "item1", "item2", "item3"];
        let tree = MerkleTree::from_items(&items, HashAlgorithm::Sha256).unwrap();

        let proof = tree.prove(2).unwrap();
        assert_eq!(proof.index, 2);
        assert!(!proof.path.is_empty());
    }

    #[test]
    fn test_proof_out_of_bounds() {
        let items = vec!["a", "b"];
        let tree = MerkleTree::from_items(&items, HashAlgorithm::Sha256).unwrap();

        let result = tree.prove(5);
        assert!(result.is_err());
    }
}
