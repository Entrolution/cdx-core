//! Collaboration session management for Codex documents.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::DocumentId;

use super::Collaborator;

/// A collaborative editing session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollaborationSession {
    /// Session ID.
    pub id: String,

    /// Document being collaborated on.
    pub document_id: DocumentId,

    /// Current participants.
    pub participants: Vec<Participant>,

    /// When the session started.
    pub started: DateTime<Utc>,

    /// Session status.
    pub status: SessionStatus,
}

impl CollaborationSession {
    /// Create a new collaboration session.
    #[must_use]
    pub fn new(id: impl Into<String>, document_id: DocumentId) -> Self {
        Self {
            id: id.into(),
            document_id,
            participants: Vec::new(),
            started: Utc::now(),
            status: SessionStatus::Active,
        }
    }

    /// Add a participant.
    pub fn add_participant(&mut self, participant: Participant) {
        self.participants.push(participant);
    }

    /// Remove a participant by user ID.
    pub fn remove_participant(&mut self, user_id: &str) {
        self.participants
            .retain(|p| p.author.user_id.as_deref() != Some(user_id));
    }

    /// Get the number of participants.
    #[must_use]
    pub fn participant_count(&self) -> usize {
        self.participants.len()
    }

    /// End the session.
    pub fn end(&mut self) {
        self.status = SessionStatus::Ended;
    }
}

/// A participant in a collaboration session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Participant {
    /// Author information.
    pub author: Collaborator,

    /// When the participant joined.
    pub joined: DateTime<Utc>,

    /// Participant's cursor position.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<CursorPosition>,

    /// Assigned color for this participant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    /// Current selection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection: Option<Selection>,
}

impl Participant {
    /// Create a new participant.
    #[must_use]
    pub fn new(author: Collaborator) -> Self {
        Self {
            author,
            joined: Utc::now(),
            cursor: None,
            color: None,
            selection: None,
        }
    }

    /// Set cursor position.
    #[must_use]
    pub fn with_cursor(mut self, cursor: CursorPosition) -> Self {
        self.cursor = Some(cursor);
        self
    }

    /// Set assigned color.
    #[must_use]
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }
}

/// Cursor position in the document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorPosition {
    /// Block containing the cursor.
    pub block_ref: String,

    /// Offset within the block.
    pub offset: usize,
}

impl CursorPosition {
    /// Create a new cursor position.
    #[must_use]
    pub fn new(block_ref: impl Into<String>, offset: usize) -> Self {
        Self {
            block_ref: block_ref.into(),
            offset,
        }
    }
}

/// A text selection in the document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Selection {
    /// Start position.
    pub start: CursorPosition,

    /// End position.
    pub end: CursorPosition,
}

impl Selection {
    /// Create a new selection.
    #[must_use]
    pub fn new(start: CursorPosition, end: CursorPosition) -> Self {
        Self { start, end }
    }

    /// Create a selection within a single block.
    #[must_use]
    pub fn within_block(
        block_ref: impl Into<String>,
        start_offset: usize,
        end_offset: usize,
    ) -> Self {
        let block_ref = block_ref.into();
        Self {
            start: CursorPosition::new(block_ref.clone(), start_offset),
            end: CursorPosition::new(block_ref, end_offset),
        }
    }
}

/// Status of a collaboration session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    /// Session is active.
    #[default]
    Active,
    /// Session is paused.
    Paused,
    /// Session has ended.
    Ended,
}
