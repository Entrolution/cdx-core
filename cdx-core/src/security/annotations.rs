//! Core annotations for minimal annotation support without extensions.
//!
//! This module provides lightweight annotation support for frozen/published documents
//! when the collaboration extension is not available. Core annotations are stored in
//! `security/annotations.json` and provide basic comment functionality without the
//! full feature set of the collaboration extension.
//!
//! # When to Use
//!
//! - For simple read-only annotations on frozen documents
//! - When the collaboration extension is not available or overkill
//! - For implementations that don't need threaded discussions or suggestions
//!
//! # Example
//!
//! ```rust
//! use cdx_core::security::{Annotation, AnnotationType, AnnotationsFile};
//! use cdx_core::anchor::ContentAnchor;
//!
//! let annotation = Annotation::new(
//!     "anno-1",
//!     AnnotationType::Comment,
//!     ContentAnchor::block("para-1"),
//!     "Reviewer",
//!     "This section needs clarification.",
//! );
//!
//! let annotations = AnnotationsFile::new(vec![annotation]);
//! ```

use serde::{Deserialize, Serialize};

use crate::anchor::ContentAnchor;

/// Core annotations file for `security/annotations.json`.
///
/// Provides minimal annotation support for implementations that don't use
/// the collaboration extension.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnnotationsFile {
    /// Format version (e.g., "0.1").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Array of annotations.
    pub annotations: Vec<Annotation>,
}

impl AnnotationsFile {
    /// Create a new annotations file.
    #[must_use]
    pub fn new(annotations: Vec<Annotation>) -> Self {
        Self {
            version: Some(crate::SPEC_VERSION.to_string()),
            annotations,
        }
    }

    /// Create an empty annotations file.
    #[must_use]
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    /// Add an annotation.
    pub fn add(&mut self, annotation: Annotation) {
        self.annotations.push(annotation);
    }

    /// Get an annotation by ID.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Annotation> {
        self.annotations.iter().find(|a| a.id == id)
    }

    /// Get all annotations for a specific block.
    #[must_use]
    pub fn for_block(&self, block_id: &str) -> Vec<&Annotation> {
        self.annotations
            .iter()
            .filter(|a| a.anchor.block_id == block_id)
            .collect()
    }

    /// Check if there are any annotations.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.annotations.is_empty()
    }

    /// Get the number of annotations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.annotations.len()
    }
}

impl Default for AnnotationsFile {
    fn default() -> Self {
        Self::empty()
    }
}

/// A core annotation.
///
/// Core annotations are simpler than collaboration comments - they use a plain
/// string author field instead of the full `Collaborator` object, and don't
/// support threading, suggestions, or reactions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Annotation {
    /// Unique annotation identifier.
    pub id: String,

    /// Annotation type.
    #[serde(rename = "type")]
    pub annotation_type: AnnotationType,

    /// Anchor to document content.
    pub anchor: ContentAnchor,

    /// Author name or identifier.
    pub author: String,

    /// ISO 8601 creation timestamp.
    pub created: String,

    /// Annotation content (text).
    pub content: String,
}

impl Annotation {
    /// Create a new annotation.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        annotation_type: AnnotationType,
        anchor: ContentAnchor,
        author: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            annotation_type,
            anchor,
            author: author.into(),
            created: chrono_now(),
            content: content.into(),
        }
    }

    /// Create a comment annotation.
    #[must_use]
    pub fn comment(
        id: impl Into<String>,
        anchor: ContentAnchor,
        author: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self::new(id, AnnotationType::Comment, anchor, author, content)
    }

    /// Create a highlight annotation.
    #[must_use]
    pub fn highlight(
        id: impl Into<String>,
        anchor: ContentAnchor,
        author: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self::new(id, AnnotationType::Highlight, anchor, author, content)
    }

    /// Create a note annotation.
    #[must_use]
    pub fn note(
        id: impl Into<String>,
        anchor: ContentAnchor,
        author: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self::new(id, AnnotationType::Note, anchor, author, content)
    }

    /// Create a reaction annotation.
    #[must_use]
    pub fn reaction(
        id: impl Into<String>,
        anchor: ContentAnchor,
        author: impl Into<String>,
        emoji: impl Into<String>,
    ) -> Self {
        Self::new(id, AnnotationType::Reaction, anchor, author, emoji)
    }

    /// Set the creation timestamp.
    #[must_use]
    pub fn with_created(mut self, created: impl Into<String>) -> Self {
        self.created = created.into();
        self
    }
}

/// Annotation type for core annotations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum AnnotationType {
    /// General comment on content.
    Comment,
    /// Highlighted text with optional note.
    Highlight,
    /// A note attached to content.
    Note,
    /// Emoji reaction.
    Reaction,
}

impl AnnotationType {
    /// Get the type as a string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Comment => "comment",
            Self::Highlight => "highlight",
            Self::Note => "note",
            Self::Reaction => "reaction",
        }
    }
}

/// Get current timestamp in ISO 8601 / RFC 3339 format.
fn chrono_now() -> String {
    chrono::Utc::now().to_rfc3339()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annotations_file_new() {
        let file = AnnotationsFile::new(vec![]);
        assert!(file.is_empty());
        assert_eq!(file.len(), 0);
        assert_eq!(file.version, Some("0.1".to_string()));
    }

    #[test]
    fn test_annotations_file_add() {
        let mut file = AnnotationsFile::empty();
        let anno = Annotation::comment("a1", ContentAnchor::block("block-1"), "Author", "Comment");
        file.add(anno);

        assert_eq!(file.len(), 1);
        assert!(!file.is_empty());
    }

    #[test]
    fn test_annotations_file_get() {
        let anno1 = Annotation::comment("a1", ContentAnchor::block("b1"), "Auth", "Text 1");
        let anno2 = Annotation::note("a2", ContentAnchor::block("b2"), "Auth", "Text 2");
        let file = AnnotationsFile::new(vec![anno1, anno2]);

        assert!(file.get("a1").is_some());
        assert!(file.get("a2").is_some());
        assert!(file.get("nonexistent").is_none());
    }

    #[test]
    fn test_annotations_file_for_block() {
        let anno1 = Annotation::comment("a1", ContentAnchor::block("b1"), "Auth", "Text 1");
        let anno2 = Annotation::note("a2", ContentAnchor::block("b1"), "Auth", "Text 2");
        let anno3 = Annotation::highlight("a3", ContentAnchor::block("b2"), "Auth", "Text 3");
        let file = AnnotationsFile::new(vec![anno1, anno2, anno3]);

        let b1_annos = file.for_block("b1");
        assert_eq!(b1_annos.len(), 2);

        let b2_annos = file.for_block("b2");
        assert_eq!(b2_annos.len(), 1);
    }

    #[test]
    fn test_annotation_new() {
        let anchor = ContentAnchor::range("block-1", 10, 25);
        let anno = Annotation::new(
            "anno-1",
            AnnotationType::Comment,
            anchor,
            "Alice",
            "A comment",
        );

        assert_eq!(anno.id, "anno-1");
        assert_eq!(anno.annotation_type, AnnotationType::Comment);
        assert_eq!(anno.author, "Alice");
        assert_eq!(anno.content, "A comment");
    }

    #[test]
    fn test_annotation_convenience_constructors() {
        let anchor = ContentAnchor::block("b1");

        let comment = Annotation::comment("c1", anchor.clone(), "Auth", "Comment text");
        assert_eq!(comment.annotation_type, AnnotationType::Comment);

        let highlight = Annotation::highlight("h1", anchor.clone(), "Auth", "Highlight note");
        assert_eq!(highlight.annotation_type, AnnotationType::Highlight);

        let note = Annotation::note("n1", anchor.clone(), "Auth", "Note text");
        assert_eq!(note.annotation_type, AnnotationType::Note);

        let reaction = Annotation::reaction("r1", anchor, "Auth", "thumbsup");
        assert_eq!(reaction.annotation_type, AnnotationType::Reaction);
        assert_eq!(reaction.content, "thumbsup");
    }

    #[test]
    fn test_annotation_serialization() {
        let anchor = ContentAnchor::range("para-1", 0, 10);
        let anno = Annotation::comment("a1", anchor, "Reviewer", "Needs work")
            .with_created("2025-01-15T10:00:00Z");

        let json = serde_json::to_string(&anno).unwrap();
        assert!(json.contains("\"id\":\"a1\""));
        assert!(json.contains("\"type\":\"comment\""));
        assert!(json.contains("\"author\":\"Reviewer\""));
        assert!(json.contains("\"content\":\"Needs work\""));
        assert!(json.contains("\"blockId\":\"para-1\""));
    }

    #[test]
    fn test_annotation_deserialization() {
        let json = r#"{
            "id": "anno-1",
            "type": "highlight",
            "anchor": {"blockId": "block-1", "start": 5, "end": 15},
            "author": "Bob",
            "created": "2025-01-15T12:00:00Z",
            "content": "Important section"
        }"#;

        let anno: Annotation = serde_json::from_str(json).unwrap();
        assert_eq!(anno.id, "anno-1");
        assert_eq!(anno.annotation_type, AnnotationType::Highlight);
        assert_eq!(anno.anchor.block_id, "block-1");
        assert_eq!(anno.anchor.start, Some(5));
        assert_eq!(anno.anchor.end, Some(15));
        assert_eq!(anno.author, "Bob");
        assert_eq!(anno.content, "Important section");
    }

    #[test]
    fn test_annotations_file_serialization() {
        let anno = Annotation::comment("a1", ContentAnchor::block("b1"), "Auth", "Comment")
            .with_created("2025-01-15T10:00:00Z");
        let file = AnnotationsFile::new(vec![anno]);

        let json = serde_json::to_string_pretty(&file).unwrap();
        assert!(json.contains("\"version\": \"0.1\""));
        assert!(json.contains("\"annotations\""));
        assert!(json.contains("\"type\": \"comment\""));
    }

    #[test]
    fn test_annotations_file_deserialization() {
        let json = r#"{
            "version": "0.1",
            "annotations": [
                {
                    "id": "a1",
                    "type": "note",
                    "anchor": {"blockId": "intro"},
                    "author": "Editor",
                    "created": "2025-01-15T10:00:00Z",
                    "content": "Consider rephrasing."
                }
            ]
        }"#;

        let file: AnnotationsFile = serde_json::from_str(json).unwrap();
        assert_eq!(file.version, Some("0.1".to_string()));
        assert_eq!(file.len(), 1);

        let anno = &file.annotations[0];
        assert_eq!(anno.id, "a1");
        assert_eq!(anno.annotation_type, AnnotationType::Note);
    }

    #[test]
    fn test_annotation_type_display() {
        assert_eq!(AnnotationType::Comment.to_string(), "comment");
        assert_eq!(AnnotationType::Highlight.to_string(), "highlight");
        assert_eq!(AnnotationType::Note.to_string(), "note");
        assert_eq!(AnnotationType::Reaction.to_string(), "reaction");
    }

    #[test]
    fn test_annotation_type_as_str() {
        assert_eq!(AnnotationType::Comment.as_str(), "comment");
        assert_eq!(AnnotationType::Highlight.as_str(), "highlight");
        assert_eq!(AnnotationType::Note.as_str(), "note");
        assert_eq!(AnnotationType::Reaction.as_str(), "reaction");
    }
}
