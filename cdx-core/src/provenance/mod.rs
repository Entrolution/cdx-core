//! Provenance and integrity verification features.
//!
//! This module provides:
//!
//! - **Merkle Trees**: Content-addressable tree structures for efficient verification
//! - **Block Index**: Persistent block hash index for Merkle proof generation
//! - **Block Proofs**: Selective disclosure proofs for individual content blocks
//! - **Lineage Verification**: Chain verification for document version history
//! - **Timestamp Anchoring**: RFC 3161 timestamp token support
//! - **Provenance Records**: Complete provenance tracking for documents
//!
//! # Block Index Example
//!
//! ```rust,ignore
//! use cdx_core::provenance::BlockIndex;
//! use cdx_core::HashAlgorithm;
//!
//! // Create block index from document content
//! let index = BlockIndex::from_content(&content, HashAlgorithm::Sha256)?;
//! let merkle_root = index.merkle_root();
//! ```
//!
//! # Merkle Tree Example
//!
//! ```rust,ignore
//! use cdx_core::provenance::{MerkleTree, MerkleProof};
//!
//! // Build a tree from content blocks
//! let tree = MerkleTree::from_blocks(&blocks, HashAlgorithm::Sha256)?;
//! let root = tree.root();
//!
//! // Generate a proof for a specific block
//! let proof = tree.prove(2)?;  // Proof for block at index 2
//!
//! // Verify the proof
//! assert!(proof.verify(&block_hash, &root));
//! ```

mod block_index;
mod merkle;
mod proof;
mod record;
mod timestamp;

pub use block_index::{BlockHashEntry, BlockIndex};
pub use merkle::{MerkleNode, MerkleTree};
pub use proof::{BlockProof, ProofVerification};
pub use record::{
    CreatorInfo, DerivationRecord, DerivationType, MerkleInfo, ProvenanceRecord, TimestampMethod,
    TimestampRecord,
};
pub use timestamp::{TimestampRequest, TimestampResponse, TimestampToken};
