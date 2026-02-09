//! Citations and footnotes for academic documents.

use serde::{Deserialize, Serialize};

use crate::content::Block;

/// An inline citation reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Citation {
    /// Reference to bibliography entry ID.
    #[serde(rename = "ref")]
    pub reference: String,

    /// Page or location within the reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locator: Option<String>,

    /// Locator type (page, chapter, section, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locator_type: Option<LocatorType>,

    /// Text before the citation (e.g., "see").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,

    /// Text after the citation (e.g., "for details").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,

    /// Suppress author name in citation.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub suppress_author: bool,
}

impl Citation {
    /// Create a new citation.
    #[must_use]
    pub fn new(reference: impl Into<String>) -> Self {
        Self {
            reference: reference.into(),
            locator: None,
            locator_type: None,
            prefix: None,
            suffix: None,
            suppress_author: false,
        }
    }

    /// Set page locator.
    #[must_use]
    pub fn with_page(mut self, page: impl Into<String>) -> Self {
        self.locator = Some(page.into());
        self.locator_type = Some(LocatorType::Page);
        self
    }

    /// Set locator with type.
    #[must_use]
    pub fn with_locator(mut self, locator: impl Into<String>, locator_type: LocatorType) -> Self {
        self.locator = Some(locator.into());
        self.locator_type = Some(locator_type);
        self
    }

    /// Set prefix text.
    #[must_use]
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Set suffix text.
    #[must_use]
    pub fn with_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }

    /// Suppress author name.
    #[must_use]
    pub const fn suppress_author(mut self) -> Self {
        self.suppress_author = true;
        self
    }
}

/// Type of locator within a reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LocatorType {
    /// Page number.
    Page,
    /// Chapter number.
    Chapter,
    /// Section number.
    Section,
    /// Paragraph number.
    Paragraph,
    /// Verse number.
    Verse,
    /// Line number.
    Line,
    /// Figure number.
    Figure,
    /// Table number.
    Table,
    /// Equation number.
    Equation,
    /// Timestamp (for media).
    Timestamp,
}

/// A footnote with content blocks.
///
/// Per the spec, footnotes support either `content` (plain text) or `children`
/// (rich content with blocks), but not both on the same footnote.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Footnote {
    /// Sequential footnote number.
    pub number: u32,

    /// Optional unique identifier for cross-referencing with footnote marks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Simple text content (for footnotes without complex formatting).
    /// Mutually exclusive with `children`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Rich content blocks (paragraph blocks with formatting).
    /// Mutually exclusive with `content`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<Block>,
}

impl Footnote {
    /// Create a new footnote with the given number.
    #[must_use]
    pub fn new(number: u32) -> Self {
        Self {
            number,
            id: None,
            content: None,
            children: Vec::new(),
        }
    }

    /// Set the unique identifier.
    #[must_use]
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the text content (simple footnotes without formatting).
    ///
    /// Note: This is mutually exclusive with `with_children`. If both are
    /// set, implementations should prefer `children`.
    #[must_use]
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Set the rich content blocks (footnotes with formatting).
    ///
    /// Note: This is mutually exclusive with `with_content`. If both are
    /// set, implementations should prefer `children`.
    #[must_use]
    pub fn with_children(mut self, children: Vec<Block>) -> Self {
        self.children = children;
        self
    }

    /// Check if this footnote has rich content (children).
    #[must_use]
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Check if this footnote has simple content.
    #[must_use]
    pub fn has_content(&self) -> bool {
        self.content.is_some()
    }
}
