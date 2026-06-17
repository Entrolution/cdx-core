//! Comments and annotations for CDX documents.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
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

/// Status of a suggestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum SuggestionStatus {
    /// Suggestion is pending review.
    #[default]
    Pending,
    /// Suggestion has been accepted.
    Accepted,
    /// Suggestion has been rejected.
    Rejected,
}

/// Priority level for comments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
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
