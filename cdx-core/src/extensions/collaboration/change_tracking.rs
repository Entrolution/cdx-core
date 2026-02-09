//! Change tracking for Codex documents.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::content::Block;
use crate::DocumentId;

use super::{Collaborator, TextRange};

/// Change tracking for a document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeTracking {
    /// Base version this tracking is relative to.
    pub base_version: DocumentId,

    /// Tracked changes.
    pub changes: Vec<TrackedChange>,

    /// Whether change tracking is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

impl ChangeTracking {
    /// Create new change tracking.
    #[must_use]
    pub fn new(base_version: DocumentId) -> Self {
        Self {
            base_version,
            changes: Vec::new(),
            enabled: true,
        }
    }

    /// Add a tracked change.
    pub fn add_change(&mut self, change: TrackedChange) {
        self.changes.push(change);
    }

    /// Get all pending changes.
    #[must_use]
    pub fn pending_changes(&self) -> Vec<&TrackedChange> {
        self.changes
            .iter()
            .filter(|c| c.status == ChangeStatus::Pending)
            .collect()
    }

    /// Get changes by author.
    #[must_use]
    pub fn changes_by_author(&self, author_name: &str) -> Vec<&TrackedChange> {
        self.changes
            .iter()
            .filter(|c| c.author.name == author_name)
            .collect()
    }

    /// Accept a change by ID.
    ///
    /// Returns `true` if the change was found and accepted.
    pub fn accept_change(&mut self, change_id: &str) -> bool {
        if let Some(change) = self.changes.iter_mut().find(|c| c.id == change_id) {
            change.status = ChangeStatus::Accepted;
            true
        } else {
            false
        }
    }

    /// Reject a change by ID.
    ///
    /// Returns `true` if the change was found and rejected.
    pub fn reject_change(&mut self, change_id: &str) -> bool {
        if let Some(change) = self.changes.iter_mut().find(|c| c.id == change_id) {
            change.status = ChangeStatus::Rejected;
            true
        } else {
            false
        }
    }

    /// Accept all pending changes.
    pub fn accept_all(&mut self) {
        for change in &mut self.changes {
            if change.status == ChangeStatus::Pending {
                change.status = ChangeStatus::Accepted;
            }
        }
    }

    /// Reject all pending changes.
    pub fn reject_all(&mut self) {
        for change in &mut self.changes {
            if change.status == ChangeStatus::Pending {
                change.status = ChangeStatus::Rejected;
            }
        }
    }

    /// Get the number of changes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.changes.len()
    }

    /// Check if there are no changes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
}

/// A tracked change in the document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackedChange {
    /// Unique identifier.
    pub id: String,

    /// Type of change.
    pub change_type: ChangeType,

    /// Reference to the affected block.
    pub block_ref: String,

    /// Content before the change (for modify/delete).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<Box<Block>>,

    /// Content after the change (for insert/modify).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<Box<Block>>,

    /// Text range affected (for inline changes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<TextRange>,

    /// Original text (for inline changes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_text: Option<String>,

    /// New text (for inline changes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_text: Option<String>,

    /// Author of the change.
    pub author: Collaborator,

    /// When the change was made.
    pub timestamp: DateTime<Utc>,

    /// Current status of the change.
    #[serde(default)]
    pub status: ChangeStatus,

    /// Optional note or reason for the change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl TrackedChange {
    /// Create a new tracked change.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        change_type: ChangeType,
        block_ref: impl Into<String>,
        author: Collaborator,
    ) -> Self {
        Self {
            id: id.into(),
            change_type,
            block_ref: block_ref.into(),
            before: None,
            after: None,
            range: None,
            original_text: None,
            new_text: None,
            author,
            timestamp: Utc::now(),
            status: ChangeStatus::Pending,
            note: None,
        }
    }

    /// Create an insertion change.
    #[must_use]
    pub fn insert(
        id: impl Into<String>,
        block_ref: impl Into<String>,
        author: Collaborator,
        content: Block,
    ) -> Self {
        Self::new(id, ChangeType::Insert, block_ref, author).with_after(content)
    }

    /// Create a deletion change.
    #[must_use]
    pub fn delete(
        id: impl Into<String>,
        block_ref: impl Into<String>,
        author: Collaborator,
        content: Block,
    ) -> Self {
        Self::new(id, ChangeType::Delete, block_ref, author).with_before(content)
    }

    /// Create a modification change.
    #[must_use]
    pub fn modify(
        id: impl Into<String>,
        block_ref: impl Into<String>,
        author: Collaborator,
        before: Block,
        after: Block,
    ) -> Self {
        Self::new(id, ChangeType::Modify, block_ref, author)
            .with_before(before)
            .with_after(after)
    }

    /// Create an inline text change.
    #[must_use]
    pub fn inline_text(
        id: impl Into<String>,
        block_ref: impl Into<String>,
        author: Collaborator,
        range: TextRange,
        original: impl Into<String>,
        replacement: impl Into<String>,
    ) -> Self {
        Self::new(id, ChangeType::Modify, block_ref, author)
            .with_range(range)
            .with_text_change(original, replacement)
    }

    /// Set the before state.
    #[must_use]
    pub fn with_before(mut self, block: Block) -> Self {
        self.before = Some(Box::new(block));
        self
    }

    /// Set the after state.
    #[must_use]
    pub fn with_after(mut self, block: Block) -> Self {
        self.after = Some(Box::new(block));
        self
    }

    /// Set the text range.
    #[must_use]
    pub fn with_range(mut self, range: TextRange) -> Self {
        self.range = Some(range);
        self
    }

    /// Set inline text change.
    #[must_use]
    pub fn with_text_change(mut self, original: impl Into<String>, new: impl Into<String>) -> Self {
        self.original_text = Some(original.into());
        self.new_text = Some(new.into());
        self
    }

    /// Set a note.
    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// Accept this change.
    pub fn accept(&mut self) {
        self.status = ChangeStatus::Accepted;
    }

    /// Reject this change.
    pub fn reject(&mut self) {
        self.status = ChangeStatus::Rejected;
    }

    /// Check if this change is pending.
    #[must_use]
    pub fn is_pending(&self) -> bool {
        self.status == ChangeStatus::Pending
    }
}

/// Type of tracked change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeType {
    /// Content was inserted.
    Insert,
    /// Content was deleted.
    Delete,
    /// Content was modified.
    Modify,
    /// Content was moved.
    Move,
    /// Formatting was changed.
    Format,
}

impl std::fmt::Display for ChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Insert => write!(f, "insert"),
            Self::Delete => write!(f, "delete"),
            Self::Modify => write!(f, "modify"),
            Self::Move => write!(f, "move"),
            Self::Format => write!(f, "format"),
        }
    }
}

/// Status of a tracked change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeStatus {
    /// Change is pending review.
    #[default]
    Pending,
    /// Change has been accepted.
    Accepted,
    /// Change has been rejected.
    Rejected,
}

impl std::fmt::Display for ChangeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Accepted => write!(f, "accepted"),
            Self::Rejected => write!(f, "rejected"),
        }
    }
}
