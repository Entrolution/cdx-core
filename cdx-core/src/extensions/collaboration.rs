//! Collaboration extension for comments, annotations, and change tracking.
//!
//! This extension provides collaborative editing features for Codex documents.
//!
//! # Features
//!
//! - **Comments**: Inline comments with threading and replies
//! - **Highlights**: Text highlighting with colors
//! - **Suggestions**: Proposed text changes (accept/reject workflow)
//! - **Reactions**: Emoji reactions to content
//! - **Change Tracking**: Track insertions, deletions, and modifications
//!
//! # Example
//!
//! ```json
//! {
//!   "type": "collaboration:comment",
//!   "id": "comment-1",
//!   "blockRef": "block-42",
//!   "range": {"start": 10, "end": 25},
//!   "author": {"name": "Alice", "email": "alice@example.com"},
//!   "content": "Consider rephrasing this section.",
//!   "created": "2024-01-15T10:30:00Z"
//! }
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::content::Block;
use crate::DocumentId;

// ============================================================================
// Comments & Annotations
// ============================================================================

/// A comment or annotation on document content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    /// Unique identifier.
    pub id: String,

    /// Type of comment.
    pub comment_type: CommentType,

    /// Reference to the block being commented on.
    pub block_ref: String,

    /// Text range within the block (if applicable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<TextRange>,

    /// Author of the comment.
    pub author: Collaborator,

    /// When the comment was created.
    pub created: DateTime<Utc>,

    /// Comment content.
    pub content: String,

    /// Whether the comment has been resolved.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub resolved: bool,

    /// Who resolved the comment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_by: Option<Collaborator>,

    /// When the comment was resolved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<DateTime<Utc>>,

    /// Replies to this comment.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub replies: Vec<Comment>,

    /// Parent comment ID (for nested replies).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,

    /// Priority level.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,

    /// Tags or labels.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

impl Comment {
    /// Create a new comment.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        block_ref: impl Into<String>,
        author: Collaborator,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            comment_type: CommentType::Comment,
            block_ref: block_ref.into(),
            range: None,
            author,
            created: Utc::now(),
            content: content.into(),
            resolved: false,
            resolved_by: None,
            resolved_at: None,
            replies: Vec::new(),
            parent_id: None,
            priority: None,
            tags: Vec::new(),
        }
    }

    /// Create a new highlight.
    #[must_use]
    pub fn highlight(
        id: impl Into<String>,
        block_ref: impl Into<String>,
        range: TextRange,
        author: Collaborator,
        color: HighlightColor,
    ) -> Self {
        Self {
            id: id.into(),
            comment_type: CommentType::Highlight { color },
            block_ref: block_ref.into(),
            range: Some(range),
            author,
            created: Utc::now(),
            content: String::new(),
            resolved: false,
            resolved_by: None,
            resolved_at: None,
            replies: Vec::new(),
            parent_id: None,
            priority: None,
            tags: Vec::new(),
        }
    }

    /// Create a new suggestion.
    #[must_use]
    pub fn suggestion(
        id: impl Into<String>,
        block_ref: impl Into<String>,
        range: TextRange,
        author: Collaborator,
        original: impl Into<String>,
        suggested: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            comment_type: CommentType::Suggestion {
                original: original.into(),
                suggested: suggested.into(),
                status: SuggestionStatus::Pending,
            },
            block_ref: block_ref.into(),
            range: Some(range),
            author,
            created: Utc::now(),
            content: String::new(),
            resolved: false,
            resolved_by: None,
            resolved_at: None,
            replies: Vec::new(),
            parent_id: None,
            priority: None,
            tags: Vec::new(),
        }
    }

    /// Create a reaction.
    #[must_use]
    pub fn reaction(
        id: impl Into<String>,
        block_ref: impl Into<String>,
        author: Collaborator,
        emoji: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            comment_type: CommentType::Reaction {
                emoji: emoji.into(),
            },
            block_ref: block_ref.into(),
            range: None,
            author,
            created: Utc::now(),
            content: String::new(),
            resolved: false,
            resolved_by: None,
            resolved_at: None,
            replies: Vec::new(),
            parent_id: None,
            priority: None,
            tags: Vec::new(),
        }
    }

    /// Set the text range.
    #[must_use]
    pub fn with_range(mut self, range: TextRange) -> Self {
        self.range = Some(range);
        self
    }

    /// Set priority.
    #[must_use]
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Add a tag.
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add a reply.
    pub fn add_reply(&mut self, mut reply: Comment) {
        reply.parent_id = Some(self.id.clone());
        self.replies.push(reply);
    }

    /// Resolve the comment.
    pub fn resolve(&mut self, by: Collaborator) {
        self.resolved = true;
        self.resolved_by = Some(by);
        self.resolved_at = Some(Utc::now());
    }

    /// Unresolve the comment.
    pub fn unresolve(&mut self) {
        self.resolved = false;
        self.resolved_by = None;
        self.resolved_at = None;
    }

    /// Check if this is a suggestion.
    #[must_use]
    pub fn is_suggestion(&self) -> bool {
        matches!(self.comment_type, CommentType::Suggestion { .. })
    }

    /// Get the suggestion status if this is a suggestion.
    #[must_use]
    pub fn suggestion_status(&self) -> Option<SuggestionStatus> {
        match &self.comment_type {
            CommentType::Suggestion { status, .. } => Some(*status),
            _ => None,
        }
    }

    /// Accept a suggestion.
    ///
    /// Returns `true` if the suggestion was accepted, `false` if this is not a suggestion.
    pub fn accept_suggestion(&mut self) -> bool {
        if let CommentType::Suggestion { status, .. } = &mut self.comment_type {
            *status = SuggestionStatus::Accepted;
            true
        } else {
            false
        }
    }

    /// Reject a suggestion.
    ///
    /// Returns `true` if the suggestion was rejected, `false` if this is not a suggestion.
    pub fn reject_suggestion(&mut self) -> bool {
        if let CommentType::Suggestion { status, .. } = &mut self.comment_type {
            *status = SuggestionStatus::Rejected;
            true
        } else {
            false
        }
    }
}

/// Type of comment or annotation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CommentType {
    /// Standard comment.
    Comment,

    /// Text highlight with color.
    Highlight {
        /// Highlight color.
        color: HighlightColor,
    },

    /// Suggested text change.
    Suggestion {
        /// Original text being replaced.
        original: String,
        /// Suggested replacement text.
        suggested: String,
        /// Current status of the suggestion.
        status: SuggestionStatus,
    },

    /// Emoji reaction.
    Reaction {
        /// Emoji character or shortcode.
        emoji: String,
    },
}

/// Highlight color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HighlightColor {
    /// Yellow highlight.
    #[default]
    Yellow,
    /// Green highlight.
    Green,
    /// Blue highlight.
    Blue,
    /// Pink highlight.
    Pink,
    /// Orange highlight.
    Orange,
    /// Purple highlight.
    Purple,
    /// Red highlight.
    Red,
}

impl std::fmt::Display for HighlightColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Yellow => write!(f, "yellow"),
            Self::Green => write!(f, "green"),
            Self::Blue => write!(f, "blue"),
            Self::Pink => write!(f, "pink"),
            Self::Orange => write!(f, "orange"),
            Self::Purple => write!(f, "purple"),
            Self::Red => write!(f, "red"),
        }
    }
}

/// Status of a suggestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SuggestionStatus {
    /// Suggestion is pending review.
    #[default]
    Pending,
    /// Suggestion has been accepted.
    Accepted,
    /// Suggestion has been rejected.
    Rejected,
}

impl std::fmt::Display for SuggestionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Accepted => write!(f, "accepted"),
            Self::Rejected => write!(f, "rejected"),
        }
    }
}

/// Priority level for comments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    /// Low priority.
    Low,
    /// Normal priority.
    Normal,
    /// High priority.
    High,
    /// Critical priority.
    Critical,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Normal => write!(f, "normal"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// A text range within a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRange {
    /// Start offset (inclusive).
    pub start: usize,
    /// End offset (exclusive).
    pub end: usize,
}

impl TextRange {
    /// Create a new text range.
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Get the length of the range.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Check if the range is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Check if this range contains a position.
    #[must_use]
    pub const fn contains(&self, pos: usize) -> bool {
        pos >= self.start && pos < self.end
    }

    /// Check if this range overlaps with another.
    #[must_use]
    pub const fn overlaps(&self, other: &Self) -> bool {
        self.start < other.end && other.start < self.end
    }

    /// Check if this range fully contains another.
    #[must_use]
    pub const fn contains_range(&self, other: &Self) -> bool {
        self.start <= other.start && other.end <= self.end
    }
}

/// Collaborator information for comments and changes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Collaborator {
    /// Display name.
    pub name: String,

    /// Email address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Avatar URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,

    /// User ID in an external system.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    /// Color for real-time cursor coloring (e.g., "#FF5733" or "blue").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

impl Collaborator {
    /// Create a new author.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            email: None,
            avatar: None,
            user_id: None,
            color: None,
        }
    }

    /// Set email.
    #[must_use]
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Set avatar URL.
    #[must_use]
    pub fn with_avatar(mut self, avatar: impl Into<String>) -> Self {
        self.avatar = Some(avatar.into());
        self
    }

    /// Set user ID.
    #[must_use]
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set color for real-time cursor coloring.
    #[must_use]
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }
}

// ============================================================================
// Change Tracking
// ============================================================================

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

// ============================================================================
// Comment Thread
// ============================================================================

/// A collection of comments organized by thread.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentThread {
    /// All comments in the thread.
    pub comments: Vec<Comment>,
}

impl CommentThread {
    /// Create a new empty thread.
    #[must_use]
    pub fn new() -> Self {
        Self {
            comments: Vec::new(),
        }
    }

    /// Add a comment to the thread.
    pub fn add(&mut self, comment: Comment) {
        self.comments.push(comment);
    }

    /// Get a comment by ID.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Comment> {
        Self::find_comment(id, &self.comments)
    }

    /// Get a mutable comment by ID.
    #[must_use]
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Comment> {
        self.find_comment_mut(id)
    }

    /// Get comments for a specific block.
    #[must_use]
    pub fn for_block(&self, block_ref: &str) -> Vec<&Comment> {
        self.comments
            .iter()
            .filter(|c| c.block_ref == block_ref)
            .collect()
    }

    /// Get all unresolved comments.
    #[must_use]
    pub fn unresolved(&self) -> Vec<&Comment> {
        self.comments.iter().filter(|c| !c.resolved).collect()
    }

    /// Get all resolved comments.
    #[must_use]
    pub fn resolved(&self) -> Vec<&Comment> {
        self.comments.iter().filter(|c| c.resolved).collect()
    }

    /// Get the number of comments.
    #[must_use]
    pub fn len(&self) -> usize {
        self.comments.len()
    }

    /// Check if the thread is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.comments.is_empty()
    }

    /// Find a comment recursively.
    fn find_comment<'a>(id: &str, comments: &'a [Comment]) -> Option<&'a Comment> {
        for comment in comments {
            if comment.id == id {
                return Some(comment);
            }
            if let Some(found) = Self::find_comment(id, &comment.replies) {
                return Some(found);
            }
        }
        None
    }

    /// Find a mutable comment.
    fn find_comment_mut(&mut self, id: &str) -> Option<&mut Comment> {
        // Note: Can't recurse into replies with mutable reference easily
        // This is a limitation of the current implementation
        self.comments.iter_mut().find(|comment| comment.id == id)
    }
}

// ============================================================================
// Collaboration Session
// ============================================================================

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_new() {
        let author = Collaborator::new("Alice");
        let comment = Comment::new("c1", "block-1", author, "Great point!");

        assert_eq!(comment.id, "c1");
        assert_eq!(comment.block_ref, "block-1");
        assert_eq!(comment.content, "Great point!");
        assert!(!comment.resolved);
    }

    #[test]
    fn test_comment_highlight() {
        let author = Collaborator::new("Bob");
        let range = TextRange::new(10, 20);
        let comment = Comment::highlight("h1", "block-2", range, author, HighlightColor::Yellow);

        assert!(matches!(
            comment.comment_type,
            CommentType::Highlight {
                color: HighlightColor::Yellow
            }
        ));
        assert_eq!(comment.range, Some(TextRange::new(10, 20)));
    }

    #[test]
    fn test_comment_suggestion() {
        let author = Collaborator::new("Carol");
        let range = TextRange::new(0, 5);
        let comment = Comment::suggestion("s1", "block-3", range, author, "Hello", "Hi");

        assert!(comment.is_suggestion());
        assert_eq!(comment.suggestion_status(), Some(SuggestionStatus::Pending));
    }

    #[test]
    fn test_suggestion_accept_reject() {
        let author = Collaborator::new("Dave");
        let range = TextRange::new(0, 5);
        let mut comment = Comment::suggestion("s1", "block-3", range, author, "old", "new");

        assert!(comment.accept_suggestion());
        assert_eq!(
            comment.suggestion_status(),
            Some(SuggestionStatus::Accepted)
        );

        let author2 = Collaborator::new("Eve");
        let range2 = TextRange::new(0, 5);
        let mut comment2 = Comment::suggestion("s2", "block-4", range2, author2, "old", "new");

        assert!(comment2.reject_suggestion());
        assert_eq!(
            comment2.suggestion_status(),
            Some(SuggestionStatus::Rejected)
        );
    }

    #[test]
    fn test_comment_resolve() {
        let author = Collaborator::new("Frank");
        let resolver = Collaborator::new("Grace");
        let mut comment = Comment::new("c1", "block-1", author, "Fix this");

        comment.resolve(resolver.clone());

        assert!(comment.resolved);
        assert_eq!(comment.resolved_by.as_ref().unwrap().name, "Grace");
        assert!(comment.resolved_at.is_some());

        comment.unresolve();
        assert!(!comment.resolved);
    }

    #[test]
    fn test_comment_reply() {
        let author1 = Collaborator::new("Alice");
        let author2 = Collaborator::new("Bob");
        let mut comment = Comment::new("c1", "block-1", author1, "Question?");
        let reply = Comment::new("c2", "block-1", author2, "Answer!");

        comment.add_reply(reply);

        assert_eq!(comment.replies.len(), 1);
        assert_eq!(comment.replies[0].parent_id, Some("c1".to_string()));
    }

    #[test]
    fn test_text_range() {
        let range = TextRange::new(10, 20);
        assert_eq!(range.len(), 10);
        assert!(!range.is_empty());
        assert!(range.contains(15));
        assert!(!range.contains(25));

        let other = TextRange::new(15, 25);
        assert!(range.overlaps(&other));

        let contained = TextRange::new(12, 18);
        assert!(range.contains_range(&contained));
    }

    #[test]
    fn test_author_builder() {
        let author = Collaborator::new("Alice")
            .with_email("alice@example.com")
            .with_user_id("user-123");

        assert_eq!(author.name, "Alice");
        assert_eq!(author.email, Some("alice@example.com".to_string()));
        assert_eq!(author.user_id, Some("user-123".to_string()));
    }

    #[test]
    fn test_change_tracking() {
        let base = "sha256:abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234"
            .parse()
            .unwrap();
        let mut tracking = ChangeTracking::new(base);
        let author = Collaborator::new("Alice");

        let change = TrackedChange::new("ch1", ChangeType::Insert, "block-1", author);
        tracking.add_change(change);

        assert_eq!(tracking.len(), 1);
        assert!(!tracking.is_empty());
        assert_eq!(tracking.pending_changes().len(), 1);
    }

    #[test]
    fn test_change_accept_reject() {
        let base = "sha256:abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234"
            .parse()
            .unwrap();
        let mut tracking = ChangeTracking::new(base);
        let author = Collaborator::new("Alice");

        tracking.add_change(TrackedChange::new(
            "ch1",
            ChangeType::Insert,
            "block-1",
            author.clone(),
        ));
        tracking.add_change(TrackedChange::new(
            "ch2",
            ChangeType::Delete,
            "block-2",
            author,
        ));

        assert!(tracking.accept_change("ch1"));
        assert!(tracking.reject_change("ch2"));

        assert_eq!(tracking.pending_changes().len(), 0);
        assert_eq!(tracking.changes[0].status, ChangeStatus::Accepted);
        assert_eq!(tracking.changes[1].status, ChangeStatus::Rejected);
    }

    #[test]
    fn test_tracked_change_inline() {
        let author = Collaborator::new("Bob");
        let range = TextRange::new(10, 20);
        let change =
            TrackedChange::inline_text("ch1", "block-1", author, range, "original", "replacement");

        assert_eq!(change.range, Some(TextRange::new(10, 20)));
        assert_eq!(change.original_text, Some("original".to_string()));
        assert_eq!(change.new_text, Some("replacement".to_string()));
    }

    #[test]
    fn test_comment_thread() {
        let mut thread = CommentThread::new();
        let author = Collaborator::new("Alice");

        thread.add(Comment::new("c1", "block-1", author.clone(), "First"));
        thread.add(Comment::new("c2", "block-2", author, "Second"));

        assert_eq!(thread.len(), 2);
        assert!(thread.get("c1").is_some());
        assert_eq!(thread.for_block("block-1").len(), 1);
    }

    #[test]
    fn test_collaboration_session() {
        let doc_id = "sha256:abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234"
            .parse()
            .unwrap();
        let mut session = CollaborationSession::new("session-1", doc_id);

        let participant = Participant::new(Collaborator::new("Alice").with_user_id("user-1"));
        session.add_participant(participant);

        assert_eq!(session.participant_count(), 1);
        assert_eq!(session.status, SessionStatus::Active);

        session.end();
        assert_eq!(session.status, SessionStatus::Ended);
    }

    #[test]
    fn test_cursor_and_selection() {
        let cursor = CursorPosition::new("block-1", 42);
        assert_eq!(cursor.block_ref, "block-1");
        assert_eq!(cursor.offset, 42);

        let selection = Selection::within_block("block-2", 10, 20);
        assert_eq!(selection.start.block_ref, "block-2");
        assert_eq!(selection.start.offset, 10);
        assert_eq!(selection.end.offset, 20);
    }

    #[test]
    fn test_comment_serialization() {
        let author = Collaborator::new("Alice").with_email("alice@example.com");
        let comment = Comment::new("c1", "block-1", author, "Test comment");

        let json = serde_json::to_string(&comment).unwrap();
        assert!(json.contains("\"id\":\"c1\""));
        assert!(json.contains("\"blockRef\":\"block-1\""));

        let deserialized: Comment = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "c1");
    }

    #[test]
    fn test_change_tracking_serialization() {
        let base = "sha256:abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234"
            .parse()
            .unwrap();
        let tracking = ChangeTracking::new(base);

        let json = serde_json::to_string(&tracking).unwrap();
        assert!(json.contains("\"baseVersion\""));
        assert!(json.contains("\"enabled\":true"));
    }
}
