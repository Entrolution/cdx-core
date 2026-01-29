//! Text nodes and formatting marks.

use serde::{Deserialize, Serialize};

/// A text node containing content and optional formatting marks.
///
/// Text nodes are the leaf nodes in the content tree, containing
/// actual text content along with formatting information.
///
/// # Example
///
/// ```
/// use cdx_core::content::{Text, Mark};
///
/// // Plain text
/// let plain = Text::plain("Hello");
///
/// // Bold text
/// let bold = Text::with_marks("Important", vec![Mark::Bold]);
///
/// // Text with multiple marks
/// let bold_italic = Text::with_marks("Emphasis", vec![Mark::Bold, Mark::Italic]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Text {
    /// The text content.
    pub value: String,

    /// Formatting marks applied to this text.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub marks: Vec<Mark>,
}

impl Text {
    /// Create a plain text node without any marks.
    #[must_use]
    pub fn plain(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            marks: Vec::new(),
        }
    }

    /// Create a text node with formatting marks.
    #[must_use]
    pub fn with_marks(value: impl Into<String>, marks: Vec<Mark>) -> Self {
        Self {
            value: value.into(),
            marks,
        }
    }

    /// Create a bold text node.
    #[must_use]
    pub fn bold(value: impl Into<String>) -> Self {
        Self::with_marks(value, vec![Mark::Bold])
    }

    /// Create an italic text node.
    #[must_use]
    pub fn italic(value: impl Into<String>) -> Self {
        Self::with_marks(value, vec![Mark::Italic])
    }

    /// Create a code text node (inline code).
    #[must_use]
    pub fn code(value: impl Into<String>) -> Self {
        Self::with_marks(value, vec![Mark::Code])
    }

    /// Create a link text node.
    #[must_use]
    pub fn link(value: impl Into<String>, href: impl Into<String>) -> Self {
        Self::with_marks(
            value,
            vec![Mark::Link {
                href: href.into(),
                title: None,
            }],
        )
    }

    /// Create a footnote reference text node.
    #[must_use]
    pub fn footnote(value: impl Into<String>, number: u32) -> Self {
        Self::with_marks(value, vec![Mark::Footnote { number, id: None }])
    }

    /// Check if this text has any marks.
    #[must_use]
    pub fn has_marks(&self) -> bool {
        !self.marks.is_empty()
    }

    /// Check if this text has a specific mark type.
    #[must_use]
    pub fn has_mark(&self, mark_type: MarkType) -> bool {
        self.marks.iter().any(|m| m.mark_type() == mark_type)
    }
}

/// Formatting marks that can be applied to text.
///
/// Marks represent inline formatting such as bold, italic, links, etc.
/// Multiple marks can be applied to the same text node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Mark {
    /// Bold/strong text.
    Bold,

    /// Italic/emphasized text.
    Italic,

    /// Underlined text.
    Underline,

    /// Strikethrough text.
    Strikethrough,

    /// Inline code (monospace).
    Code,

    /// Superscript text.
    Superscript,

    /// Subscript text.
    Subscript,

    /// Hyperlink.
    Link {
        /// Link destination URL.
        href: String,

        /// Optional link title.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
    },

    /// Named anchor mark for creating anchor points in text.
    Anchor {
        /// Unique identifier for this anchor.
        id: String,
    },

    /// Footnote reference mark (semantic extension).
    ///
    /// Links text to a footnote block elsewhere in the document.
    Footnote {
        /// Sequential footnote number.
        number: u32,

        /// Optional unique identifier for cross-referencing.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        id: Option<String>,
    },
}

impl Mark {
    /// Get the type of this mark.
    #[must_use]
    pub fn mark_type(&self) -> MarkType {
        match self {
            Self::Bold => MarkType::Bold,
            Self::Italic => MarkType::Italic,
            Self::Underline => MarkType::Underline,
            Self::Strikethrough => MarkType::Strikethrough,
            Self::Code => MarkType::Code,
            Self::Superscript => MarkType::Superscript,
            Self::Subscript => MarkType::Subscript,
            Self::Link { .. } => MarkType::Link,
            Self::Anchor { .. } => MarkType::Anchor,
            Self::Footnote { .. } => MarkType::Footnote,
        }
    }
}

/// Type identifier for marks (without associated data).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarkType {
    /// Bold mark type.
    Bold,
    /// Italic mark type.
    Italic,
    /// Underline mark type.
    Underline,
    /// Strikethrough mark type.
    Strikethrough,
    /// Code mark type.
    Code,
    /// Superscript mark type.
    Superscript,
    /// Subscript mark type.
    Subscript,
    /// Link mark type.
    Link,
    /// Anchor mark type.
    Anchor,
    /// Footnote mark type.
    Footnote,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_plain() {
        let text = Text::plain("Hello");
        assert_eq!(text.value, "Hello");
        assert!(text.marks.is_empty());
        assert!(!text.has_marks());
    }

    #[test]
    fn test_text_bold() {
        let text = Text::bold("Important");
        assert_eq!(text.marks, vec![Mark::Bold]);
        assert!(text.has_marks());
        assert!(text.has_mark(MarkType::Bold));
        assert!(!text.has_mark(MarkType::Italic));
    }

    #[test]
    fn test_text_link() {
        let text = Text::link("Click", "https://example.com");
        assert!(text.has_mark(MarkType::Link));
        if let Mark::Link { href, title } = &text.marks[0] {
            assert_eq!(href, "https://example.com");
            assert!(title.is_none());
        } else {
            panic!("Expected Link mark");
        }
    }

    #[test]
    fn test_text_serialization() {
        let text = Text::bold("Test");
        let json = serde_json::to_string(&text).unwrap();
        assert!(json.contains("\"value\":\"Test\""));
        assert!(json.contains("\"type\":\"bold\""));
    }

    #[test]
    fn test_text_deserialization() {
        let json = r#"{"value":"Test","marks":[{"type":"bold"},{"type":"italic"}]}"#;
        let text: Text = serde_json::from_str(json).unwrap();
        assert_eq!(text.value, "Test");
        assert_eq!(text.marks.len(), 2);
    }

    #[test]
    fn test_link_with_title() {
        let json = r#"{"type":"link","href":"https://example.com","title":"Example"}"#;
        let mark: Mark = serde_json::from_str(json).unwrap();
        if let Mark::Link { href, title } = mark {
            assert_eq!(href, "https://example.com");
            assert_eq!(title, Some("Example".to_string()));
        } else {
            panic!("Expected Link mark");
        }
    }

    #[test]
    fn test_text_footnote() {
        let text = Text::footnote("important claim", 1);
        assert!(text.has_mark(MarkType::Footnote));
        if let Mark::Footnote { number, id } = &text.marks[0] {
            assert_eq!(*number, 1);
            assert!(id.is_none());
        } else {
            panic!("Expected Footnote mark");
        }
    }

    #[test]
    fn test_footnote_mark_serialization() {
        let mark = Mark::Footnote {
            number: 1,
            id: Some("fn1".to_string()),
        };
        let json = serde_json::to_string(&mark).unwrap();
        assert!(json.contains("\"type\":\"footnote\""));
        assert!(json.contains("\"number\":1"));
        assert!(json.contains("\"id\":\"fn1\""));
    }

    #[test]
    fn test_footnote_mark_deserialization() {
        let json = r#"{"type":"footnote","number":2,"id":"fn-2"}"#;
        let mark: Mark = serde_json::from_str(json).unwrap();
        if let Mark::Footnote { number, id } = mark {
            assert_eq!(number, 2);
            assert_eq!(id, Some("fn-2".to_string()));
        } else {
            panic!("Expected Footnote mark");
        }
    }

    #[test]
    fn test_footnote_mark_without_id() {
        let json = r#"{"type":"footnote","number":3}"#;
        let mark: Mark = serde_json::from_str(json).unwrap();
        if let Mark::Footnote { number, id } = mark {
            assert_eq!(number, 3);
            assert!(id.is_none());
        } else {
            panic!("Expected Footnote mark");
        }
    }
}
