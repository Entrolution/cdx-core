//! CRDT integration for real-time collaboration.
//!
//! This module provides types for CRDT-based real-time collaboration,
//! enabling conflict-free merging of concurrent edits.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// CRDT format identifier.
///
/// Specifies which CRDT library/format is used for real-time collaboration.
/// Documents using CRDT-based collaboration MUST declare the format to enable
/// correct interpretation of CRDT state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CrdtFormat {
    /// [Yjs](https://yjs.dev/) CRDT library.
    Yjs,
    /// [Automerge](https://automerge.org/) JSON CRDT.
    Automerge,
    /// [Diamond Types](https://github.com/josephg/diamond-types) text CRDT.
    DiamondTypes,
}

impl std::fmt::Display for CrdtFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Yjs => write!(f, "yjs"),
            Self::Automerge => write!(f, "automerge"),
            Self::DiamondTypes => write!(f, "diamond-types"),
        }
    }
}

/// CRDT metadata for a content block.
///
/// Each content block can carry CRDT metadata for tracking distributed state.
/// This enables conflict-free merging of concurrent edits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrdtMetadata {
    /// Vector clock mapping site IDs to logical timestamps.
    pub clock: std::collections::HashMap<String, u64>,

    /// Site ID where this version originated.
    pub origin: String,

    /// Sequence number within the origin site.
    pub seq: u64,
}

impl CrdtMetadata {
    /// Create new CRDT metadata.
    #[must_use]
    pub fn new(origin: impl Into<String>, seq: u64) -> Self {
        let origin = origin.into();
        let mut clock = std::collections::HashMap::new();
        clock.insert(origin.clone(), seq);
        Self { clock, origin, seq }
    }

    /// Increment the sequence number for a site.
    pub fn increment(&mut self, site_id: &str) {
        let count = self.clock.entry(site_id.to_string()).or_insert(0);
        *count += 1;
        if site_id == self.origin {
            self.seq = *count;
        }
    }

    /// Merge with another CRDT metadata (takes maximum of each clock entry).
    pub fn merge(&mut self, other: &Self) {
        for (site, &count) in &other.clock {
            let entry = self.clock.entry(site.clone()).or_insert(0);
            *entry = (*entry).max(count);
        }
    }

    /// Check if this metadata happened before another (causally).
    #[must_use]
    pub fn happened_before(&self, other: &Self) -> bool {
        let mut dominated = false;
        for (site, &self_count) in &self.clock {
            let other_count = other.clock.get(site).copied().unwrap_or(0);
            if self_count > other_count {
                return false;
            }
            if self_count < other_count {
                dominated = true;
            }
        }
        // Check sites only in other
        for (site, &other_count) in &other.clock {
            if !self.clock.contains_key(site) && other_count > 0 {
                dominated = true;
            }
        }
        dominated
    }
}

/// CRDT metadata for text content within a block.
///
/// For rich text editing, each character can have a unique position ID
/// enabling character-level conflict resolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextCrdtMetadata {
    /// Character positions with unique IDs.
    pub positions: Vec<TextCrdtPosition>,
}

impl TextCrdtMetadata {
    /// Create new text CRDT metadata.
    #[must_use]
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
        }
    }

    /// Create metadata from text with a site ID prefix.
    #[must_use]
    pub fn from_text(text: &str, site_id: &str) -> Self {
        let positions = text
            .chars()
            .enumerate()
            .map(|(i, c)| TextCrdtPosition {
                id: format!("{site_id}:{}", i + 1),
                char: c,
            })
            .collect();
        Self { positions }
    }

    /// Get the text content.
    #[must_use]
    pub fn text(&self) -> String {
        self.positions.iter().map(|p| p.char).collect()
    }

    /// Get the number of characters.
    #[must_use]
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    /// Check if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
}

impl Default for TextCrdtMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// A single character position in text CRDT.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextCrdtPosition {
    /// Unique position identifier (e.g., "site1:42").
    pub id: String,

    /// The character at this position.
    pub char: char,
}

/// Synchronization state for CRDT-based collaboration.
///
/// Tracks the current sync state of a document for real-time collaboration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncState {
    /// CRDT format being used.
    pub crdt_format: CrdtFormat,

    /// Version of the CRDT library.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crdt_version: Option<String>,

    /// Logical clock or sequence number for sync state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sync_version: Option<u64>,

    /// Timestamp of last synchronization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_sync: Option<DateTime<Utc>>,

    /// Known collaboration peers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub peers: Vec<Peer>,
}

impl SyncState {
    /// Create new sync state with the specified CRDT format.
    #[must_use]
    pub fn new(crdt_format: CrdtFormat) -> Self {
        Self {
            crdt_format,
            crdt_version: None,
            sync_version: None,
            last_sync: None,
            peers: Vec::new(),
        }
    }

    /// Create sync state for Yjs.
    #[must_use]
    pub fn yjs() -> Self {
        Self::new(CrdtFormat::Yjs)
    }

    /// Create sync state for Automerge.
    #[must_use]
    pub fn automerge() -> Self {
        Self::new(CrdtFormat::Automerge)
    }

    /// Create sync state for Diamond Types.
    #[must_use]
    pub fn diamond_types() -> Self {
        Self::new(CrdtFormat::DiamondTypes)
    }

    /// Set the CRDT library version.
    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.crdt_version = Some(version.into());
        self
    }

    /// Set the sync version.
    #[must_use]
    pub fn with_sync_version(mut self, version: u64) -> Self {
        self.sync_version = Some(version);
        self
    }

    /// Update the last sync timestamp to now.
    pub fn mark_synced(&mut self) {
        self.last_sync = Some(Utc::now());
    }

    /// Add a peer.
    pub fn add_peer(&mut self, peer: Peer) {
        // Update existing peer or add new one
        if let Some(existing) = self.peers.iter_mut().find(|p| p.id == peer.id) {
            existing.last_seen = peer.last_seen;
        } else {
            self.peers.push(peer);
        }
    }

    /// Remove a peer by ID.
    pub fn remove_peer(&mut self, peer_id: &str) {
        self.peers.retain(|p| p.id != peer_id);
    }

    /// Get active peers (seen within the given duration).
    #[must_use]
    pub fn active_peers(&self, within: chrono::Duration) -> Vec<&Peer> {
        let cutoff = Utc::now() - within;
        self.peers.iter().filter(|p| p.last_seen > cutoff).collect()
    }
}

/// A collaboration peer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Peer {
    /// Unique peer identifier.
    pub id: String,

    /// When the peer was last seen.
    pub last_seen: DateTime<Utc>,

    /// Optional display name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl Peer {
    /// Create a new peer.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            last_seen: Utc::now(),
            name: None,
        }
    }

    /// Set the display name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Update the last seen timestamp.
    pub fn touch(&mut self) {
        self.last_seen = Utc::now();
    }
}
