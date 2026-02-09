//! Comment thread management for Codex documents.

use serde::{Deserialize, Serialize};

use super::Comment;

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
