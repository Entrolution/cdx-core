//! Citations and footnotes for academic documents.

use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};

use crate::content::Block;

/// An inline citation reference.
///
/// Supports both single and multi-reference citations (e.g., `[smith2023; jones2024]`).
///
/// # Backward Compatibility
///
/// Deserializes from both the old singular `"ref"` (string) format and the
/// new `"refs"` (array) format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Citation {
    /// References to bibliography entry IDs.
    pub refs: Vec<String>,

    /// Page or location within the reference.
    pub locator: Option<String>,

    /// Locator type (page, chapter, section, etc.).
    pub locator_type: Option<LocatorType>,

    /// Text before the citation (e.g., "see").
    pub prefix: Option<String>,

    /// Text after the citation (e.g., "for details").
    pub suffix: Option<String>,

    /// Suppress author name in citation.
    pub suppress_author: bool,
}

impl Citation {
    /// Create a new citation with a single reference.
    #[must_use]
    pub fn new(reference: impl Into<String>) -> Self {
        Self {
            refs: vec![reference.into()],
            locator: None,
            locator_type: None,
            prefix: None,
            suffix: None,
            suppress_author: false,
        }
    }

    /// Create a citation with multiple references (e.g., `[smith2023; jones2024]`).
    #[must_use]
    pub fn multi(refs: Vec<String>) -> Self {
        Self {
            refs,
            locator: None,
            locator_type: None,
            prefix: None,
            suffix: None,
            suppress_author: false,
        }
    }

    /// Get the first reference, if any.
    #[must_use]
    pub fn first_ref(&self) -> Option<&str> {
        self.refs.first().map(String::as_str)
    }

    /// Get all references.
    #[must_use]
    pub fn refs(&self) -> &[String] {
        &self.refs
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

impl Serialize for Citation {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut count = 1; // refs is always present
        if self.locator.is_some() {
            count += 1;
        }
        if self.locator_type.is_some() {
            count += 1;
        }
        if self.prefix.is_some() {
            count += 1;
        }
        if self.suffix.is_some() {
            count += 1;
        }
        if self.suppress_author {
            count += 1;
        }

        let mut map = serializer.serialize_map(Some(count))?;
        map.serialize_entry("refs", &self.refs)?;
        if let Some(ref locator) = self.locator {
            map.serialize_entry("locator", locator)?;
        }
        if let Some(ref locator_type) = self.locator_type {
            map.serialize_entry("locatorType", locator_type)?;
        }
        if let Some(ref prefix) = self.prefix {
            map.serialize_entry("prefix", prefix)?;
        }
        if let Some(ref suffix) = self.suffix {
            map.serialize_entry("suffix", suffix)?;
        }
        if self.suppress_author {
            map.serialize_entry("suppressAuthor", &true)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for Citation {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct CitationVisitor;

        impl<'de> Visitor<'de> for CitationVisitor {
            type Value = Citation;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a Citation object with 'refs' (array) or 'ref' (string)")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Citation, A::Error> {
                let mut refs: Option<Vec<String>> = None;
                let mut locator: Option<String> = None;
                let mut locator_type: Option<LocatorType> = None;
                let mut prefix: Option<String> = None;
                let mut suffix: Option<String> = None;
                let mut suppress_author = false;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "refs" => {
                            // Accept array of strings
                            refs = Some(map.next_value::<Vec<String>>()?);
                        }
                        "ref" => {
                            // Backward compat: singular string → wrap in vec
                            let single: String = map.next_value()?;
                            refs = Some(vec![single]);
                        }
                        "locator" => locator = Some(map.next_value()?),
                        "locatorType" | "locator_type" => locator_type = Some(map.next_value()?),
                        "prefix" => prefix = Some(map.next_value()?),
                        "suffix" => suffix = Some(map.next_value()?),
                        "suppressAuthor" | "suppress_author" => {
                            suppress_author = map.next_value()?;
                        }
                        _ => {
                            // Skip unknown fields for forward compatibility
                            let _: serde_json::Value = map.next_value()?;
                        }
                    }
                }

                let refs = refs.ok_or_else(|| de::Error::missing_field("refs"))?;

                Ok(Citation {
                    refs,
                    locator,
                    locator_type,
                    prefix,
                    suffix,
                    suppress_author,
                })
            }
        }

        deserializer.deserialize_map(CitationVisitor)
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
