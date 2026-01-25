//! Provenance and integrity verification features.
//!
//! This module provides:
//!
//! - **Merkle Trees**: Content-addressable tree structures for efficient verification
//! - **Block Proofs**: Selective disclosure proofs for individual content blocks
//! - **Lineage Verification**: Chain verification for document version history
//! - **Timestamp Anchoring**: RFC 3161 timestamp token support
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

mod merkle;
mod proof;
mod timestamp;

pub use merkle::{MerkleNode, MerkleTree};
pub use proof::{BlockProof, ProofVerification};
pub use timestamp::{TimestampRequest, TimestampResponse, TimestampToken};
