//! Revision history for tracking document evolution.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::DocumentId;

use super::{Collaborator, CrdtFormat};

/// Revision history for tracking document evolution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevisionHistory {
    /// List of revisions in chronological order.
    pub revisions: Vec<Revision>,
}

impl RevisionHistory {
    /// Create new empty revision history.
    #[must_use]
    pub fn new() -> Self {
        Self {
            revisions: Vec::new(),
        }
    }

    /// Add a revision.
    pub fn add(&mut self, revision: Revision) {
        self.revisions.push(revision);
    }

    /// Get the latest revision.
    #[must_use]
    pub fn latest(&self) -> Option<&Revision> {
        self.revisions.last()
    }

    /// Get a revision by version number.
    #[must_use]
    pub fn get_version(&self, version: u32) -> Option<&Revision> {
        self.revisions.iter().find(|r| r.version == version)
    }

    /// Get the next version number.
    #[must_use]
    pub fn next_version(&self) -> u32 {
        self.revisions
            .iter()
            .map(|r| r.version)
            .max()
            .map_or(1, |v| v + 1)
    }

    /// Get the number of revisions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.revisions.len()
    }

    /// Check if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.revisions.is_empty()
    }
}

impl Default for RevisionHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// A single revision in the document history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Revision {
    /// Version number (1-indexed).
    pub version: u32,

    /// Document ID (hash) of this revision.
    pub document_id: DocumentId,

    /// When this revision was created.
    pub created: DateTime<Utc>,

    /// Author of this revision.
    pub author: Collaborator,

    /// Optional note describing the revision.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Tags or labels for this revision.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

impl Revision {
    /// Create a new revision.
    #[must_use]
    pub fn new(version: u32, document_id: DocumentId, author: Collaborator) -> Self {
        Self {
            version,
            document_id,
            created: Utc::now(),
            author,
            note: None,
            tags: Vec::new(),
        }
    }

    /// Set the note.
    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Add a tag.
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// Materialization event for CRDT documents.
///
/// When a document with CRDT state is exported or exchanged between
/// different CRDT implementations, it must be materialized to static content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterializationEvent {
    /// Timestamp of the materialization.
    pub timestamp: DateTime<Utc>,

    /// Tool that performed the materialization.
    pub agent: String,

    /// Original CRDT format (if applicable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_crdt_format: Option<CrdtFormat>,

    /// Target CRDT format (if applicable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_crdt_format: Option<CrdtFormat>,

    /// Reason for materialization.
    pub reason: MaterializationReason,
}

impl MaterializationEvent {
    /// Create a new materialization event.
    #[must_use]
    pub fn new(agent: impl Into<String>, reason: MaterializationReason) -> Self {
        Self {
            timestamp: Utc::now(),
            agent: agent.into(),
            from_crdt_format: None,
            to_crdt_format: None,
            reason,
        }
    }

    /// Set the source CRDT format.
    #[must_use]
    pub fn with_source_format(mut self, format: CrdtFormat) -> Self {
        self.from_crdt_format = Some(format);
        self
    }

    /// Set the target CRDT format.
    #[must_use]
    pub fn with_target_format(mut self, format: CrdtFormat) -> Self {
        self.to_crdt_format = Some(format);
        self
    }

    /// Create an event for cross-tool exchange.
    #[must_use]
    pub fn cross_tool_exchange(agent: impl Into<String>, from: CrdtFormat, to: CrdtFormat) -> Self {
        Self::new(agent, MaterializationReason::CrossToolExchange)
            .with_source_format(from)
            .with_target_format(to)
    }

    /// Create an event for export to static format.
    #[must_use]
    pub fn export(agent: impl Into<String>, from: CrdtFormat) -> Self {
        Self::new(agent, MaterializationReason::Export).with_source_format(from)
    }
}

/// Reason for materializing CRDT state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MaterializationReason {
    /// Exchanging between tools using different CRDT formats.
    CrossToolExchange,
    /// Exporting to a non-CRDT format.
    Export,
    /// Archiving for long-term storage.
    Archive,
    /// Manual user request.
    UserRequest,
}

impl std::fmt::Display for MaterializationReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CrossToolExchange => write!(f, "cross-tool-exchange"),
            Self::Export => write!(f, "export"),
            Self::Archive => write!(f, "archive"),
            Self::UserRequest => write!(f, "user-request"),
        }
    }
}
