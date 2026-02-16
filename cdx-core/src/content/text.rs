//! Text nodes and formatting marks.

use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};

use crate::content::block::MathFormat;

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
///
/// # Serialization
///
/// Simple marks (Bold, Italic, etc.) serialize as plain strings: `"bold"`.
/// Complex marks (Link, Math, etc.) serialize as objects with a `"type"` field.
/// Extension marks serialize with their namespaced type as `"type"`.
#[derive(Debug, Clone, PartialEq, Eq)]
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
        id: Option<String>,
    },

    /// Inline mathematical expression.
    Math {
        /// Math format (latex or mathml).
        format: MathFormat,

        /// The mathematical expression source.
        source: String,
    },

    /// Extension mark for custom/unknown mark types.
    ///
    /// Extension marks use namespaced types like "semantic:citation" or
    /// "legal:cite". This enables extensions to add custom inline marks
    /// without modifying the core Mark enum.
    Extension(ExtensionMark),
}

/// An extension mark for unsupported or unknown mark types.
///
/// When parsing a document with extension marks (e.g., "semantic:citation"),
/// this struct preserves the raw data so it can be:
/// - Passed through unchanged when saving
/// - Processed by extension-aware applications
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionMark {
    /// The extension namespace (e.g., "semantic", "legal", "presentation").
    pub namespace: String,

    /// The mark type within the namespace (e.g., "citation", "entity", "index").
    pub mark_type: String,

    /// Extension-specific attributes.
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub attributes: serde_json::Value,
}

impl ExtensionMark {
    /// Create a new extension mark.
    #[must_use]
    pub fn new(namespace: impl Into<String>, mark_type: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            mark_type: mark_type.into(),
            attributes: serde_json::Value::Null,
        }
    }

    /// Parse an extension type string like "semantic:citation" into (namespace, `mark_type`).
    ///
    /// Returns `None` if the type doesn't contain a colon.
    #[must_use]
    pub fn parse_type(type_str: &str) -> Option<(&str, &str)> {
        type_str.split_once(':')
    }

    /// Get the full type string (e.g., "semantic:citation").
    #[must_use]
    pub fn full_type(&self) -> String {
        format!("{}:{}", self.namespace, self.mark_type)
    }

    /// Check if this extension is from a specific namespace.
    #[must_use]
    pub fn is_namespace(&self, namespace: &str) -> bool {
        self.namespace == namespace
    }

    /// Check if this is a specific extension type.
    #[must_use]
    pub fn is_type(&self, namespace: &str, mark_type: &str) -> bool {
        self.namespace == namespace && self.mark_type == mark_type
    }

    /// Set the attributes.
    #[must_use]
    pub fn with_attributes(mut self, attributes: serde_json::Value) -> Self {
        self.attributes = attributes;
        self
    }

    /// Get an attribute value by key.
    #[must_use]
    pub fn get_attribute(&self, key: &str) -> Option<&serde_json::Value> {
        self.attributes.get(key)
    }

    /// Get a string attribute.
    #[must_use]
    pub fn get_string_attribute(&self, key: &str) -> Option<&str> {
        self.attributes.get(key).and_then(serde_json::Value::as_str)
    }

    // ===== Convenience constructors for common extension marks =====

    /// Create a citation mark (semantic extension).
    #[must_use]
    pub fn citation(reference: impl Into<String>) -> Self {
        Self::new("semantic", "citation").with_attributes(serde_json::json!({
            "ref": reference.into()
        }))
    }

    /// Create a citation mark with page locator.
    #[must_use]
    pub fn citation_with_page(reference: impl Into<String>, page: impl Into<String>) -> Self {
        Self::new("semantic", "citation").with_attributes(serde_json::json!({
            "ref": reference.into(),
            "locator": page.into(),
            "locatorType": "page"
        }))
    }

    /// Create an entity link mark (semantic extension).
    #[must_use]
    pub fn entity(uri: impl Into<String>, entity_type: impl Into<String>) -> Self {
        Self::new("semantic", "entity").with_attributes(serde_json::json!({
            "uri": uri.into(),
            "entityType": entity_type.into()
        }))
    }

    /// Create a glossary reference mark (semantic extension).
    #[must_use]
    pub fn glossary(term_id: impl Into<String>) -> Self {
        Self::new("semantic", "glossary").with_attributes(serde_json::json!({
            "termId": term_id.into()
        }))
    }

    /// Create an index mark (presentation extension).
    #[must_use]
    pub fn index(term: impl Into<String>) -> Self {
        Self::new("presentation", "index").with_attributes(serde_json::json!({
            "term": term.into()
        }))
    }

    /// Create an index mark with subterm.
    #[must_use]
    pub fn index_with_subterm(term: impl Into<String>, subterm: impl Into<String>) -> Self {
        Self::new("presentation", "index").with_attributes(serde_json::json!({
            "term": term.into(),
            "subterm": subterm.into()
        }))
    }

    // ===== Academic extension marks =====

    /// Create an equation reference mark (academic extension).
    ///
    /// References an equation by its ID (e.g., "#eq-pythagoras").
    #[must_use]
    pub fn equation_ref(target: impl Into<String>) -> Self {
        Self::new("academic", "equation-ref").with_attributes(serde_json::json!({
            "target": target.into()
        }))
    }

    /// Create an equation reference mark with custom format.
    ///
    /// The format string can use `{number}` as a placeholder for the equation number.
    #[must_use]
    pub fn equation_ref_formatted(target: impl Into<String>, format: impl Into<String>) -> Self {
        Self::new("academic", "equation-ref").with_attributes(serde_json::json!({
            "target": target.into(),
            "format": format.into()
        }))
    }

    /// Create an algorithm reference mark (academic extension).
    ///
    /// References an algorithm by its ID (e.g., "#alg-quicksort").
    #[must_use]
    pub fn algorithm_ref(target: impl Into<String>) -> Self {
        Self::new("academic", "algorithm-ref").with_attributes(serde_json::json!({
            "target": target.into()
        }))
    }

    /// Create an algorithm reference mark with line reference.
    ///
    /// References a specific line within an algorithm.
    #[must_use]
    pub fn algorithm_ref_line(target: impl Into<String>, line: impl Into<String>) -> Self {
        Self::new("academic", "algorithm-ref").with_attributes(serde_json::json!({
            "target": target.into(),
            "line": line.into()
        }))
    }

    /// Create an algorithm reference mark with custom format.
    ///
    /// The format string can use `{number}` and `{line}` as placeholders.
    #[must_use]
    pub fn algorithm_ref_formatted(target: impl Into<String>, format: impl Into<String>) -> Self {
        Self::new("academic", "algorithm-ref").with_attributes(serde_json::json!({
            "target": target.into(),
            "format": format.into()
        }))
    }

    /// Create an algorithm reference mark with line and custom format.
    #[must_use]
    pub fn algorithm_ref_line_formatted(
        target: impl Into<String>,
        line: impl Into<String>,
        format: impl Into<String>,
    ) -> Self {
        Self::new("academic", "algorithm-ref").with_attributes(serde_json::json!({
            "target": target.into(),
            "line": line.into(),
            "format": format.into()
        }))
    }

    /// Create a theorem reference mark (academic extension).
    ///
    /// References a theorem by its ID (e.g., "#thm-pythagoras").
    #[must_use]
    pub fn theorem_ref(target: impl Into<String>) -> Self {
        Self::new("academic", "theorem-ref").with_attributes(serde_json::json!({
            "target": target.into()
        }))
    }

    /// Create a theorem reference mark with custom format.
    ///
    /// The format string can use `{number}` and `{variant}` as placeholders.
    #[must_use]
    pub fn theorem_ref_formatted(target: impl Into<String>, format: impl Into<String>) -> Self {
        Self::new("academic", "theorem-ref").with_attributes(serde_json::json!({
            "target": target.into(),
            "format": format.into()
        }))
    }

    // ===== Collaboration extension marks =====

    /// Create a highlight mark (collaboration extension).
    ///
    /// Applies a colored highlight to text for collaborative annotation.
    /// Default color is yellow if not specified.
    #[must_use]
    pub fn highlight(color: impl Into<String>) -> Self {
        Self::new("collaboration", "highlight").with_attributes(serde_json::json!({
            "color": color.into()
        }))
    }

    /// Create a highlight mark with default yellow color.
    #[must_use]
    pub fn highlight_yellow() -> Self {
        Self::highlight("yellow")
    }

    /// Create a highlight mark with a specific color.
    ///
    /// Convenience method that accepts the `HighlightColor` display string.
    #[must_use]
    pub fn highlight_colored(color: impl std::fmt::Display) -> Self {
        Self::highlight(color.to_string())
    }
}

/// Infer the extension namespace from a mark type string.
///
/// Used during deserialization when an unknown type string is encountered
/// without explicit namespace information.
fn infer_mark_namespace(mark_type: &str) -> &'static str {
    match mark_type {
        "citation" | "entity" | "glossary" => "semantic",
        "theorem-ref" | "equation-ref" | "algorithm-ref" => "academic",
        "cite" => "legal",
        "highlight" => "collaboration",
        "index" => "presentation",
        _ => "",
    }
}

impl Serialize for Mark {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            // Simple marks serialize as plain strings
            Self::Bold => serializer.serialize_str("bold"),
            Self::Italic => serializer.serialize_str("italic"),
            Self::Underline => serializer.serialize_str("underline"),
            Self::Strikethrough => serializer.serialize_str("strikethrough"),
            Self::Code => serializer.serialize_str("code"),
            Self::Superscript => serializer.serialize_str("superscript"),
            Self::Subscript => serializer.serialize_str("subscript"),

            // Complex marks serialize as objects with "type" field
            Self::Link { href, title } => {
                let len = 2 + usize::from(title.is_some());
                let mut map = serializer.serialize_map(Some(len))?;
                map.serialize_entry("type", "link")?;
                map.serialize_entry("href", href)?;
                if let Some(t) = title {
                    map.serialize_entry("title", t)?;
                }
                map.end()
            }
            Self::Anchor { id } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "anchor")?;
                map.serialize_entry("id", id)?;
                map.end()
            }
            Self::Footnote { number, id } => {
                let len = 2 + usize::from(id.is_some());
                let mut map = serializer.serialize_map(Some(len))?;
                map.serialize_entry("type", "footnote")?;
                map.serialize_entry("number", number)?;
                if let Some(i) = id {
                    map.serialize_entry("id", i)?;
                }
                map.end()
            }
            Self::Math { format, source } => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("type", "math")?;
                map.serialize_entry("format", format)?;
                map.serialize_entry("source", source)?;
                map.end()
            }

            // Extension marks: type is "namespace:markType", attributes flattened
            Self::Extension(ext) => {
                let type_str = ext.full_type();
                let attr_count = ext.attributes.as_object().map_or(0, serde_json::Map::len);
                let mut map = serializer.serialize_map(Some(1 + attr_count))?;
                map.serialize_entry("type", &type_str)?;
                if let Some(obj) = ext.attributes.as_object() {
                    for (k, v) in obj {
                        map.serialize_entry(k, v)?;
                    }
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Mark {
    #[allow(clippy::too_many_lines)]
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct MarkVisitor;

        impl<'de> Visitor<'de> for MarkVisitor {
            type Value = Mark;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a string (simple mark) or an object (complex mark)")
            }

            // Simple marks can be plain strings
            fn visit_str<E: de::Error>(self, v: &str) -> Result<Mark, E> {
                match v {
                    "bold" => Ok(Mark::Bold),
                    "italic" => Ok(Mark::Italic),
                    "underline" => Ok(Mark::Underline),
                    "strikethrough" => Ok(Mark::Strikethrough),
                    "code" => Ok(Mark::Code),
                    "superscript" => Ok(Mark::Superscript),
                    "subscript" => Ok(Mark::Subscript),
                    other => {
                        // Unknown string: treat as extension mark
                        let (ns, mt) = if let Some((ns, mt)) = other.split_once(':') {
                            (ns.to_string(), mt.to_string())
                        } else {
                            (infer_mark_namespace(other).to_string(), other.to_string())
                        };
                        Ok(Mark::Extension(ExtensionMark::new(ns, mt)))
                    }
                }
            }

            // Complex marks are objects with a "type" field
            #[allow(clippy::too_many_lines)]
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Mark, A::Error> {
                let mut type_str: Option<String> = None;
                let mut fields = serde_json::Map::new();

                while let Some(key) = map.next_key::<String>()? {
                    if key == "type" {
                        type_str = Some(map.next_value()?);
                    } else {
                        let value: serde_json::Value = map.next_value()?;
                        fields.insert(key, value);
                    }
                }

                let type_str = type_str.ok_or_else(|| de::Error::missing_field("type"))?;

                match type_str.as_str() {
                    // Simple marks in object form
                    "bold" => Ok(Mark::Bold),
                    "italic" => Ok(Mark::Italic),
                    "underline" => Ok(Mark::Underline),
                    "strikethrough" => Ok(Mark::Strikethrough),
                    "code" => Ok(Mark::Code),
                    "superscript" => Ok(Mark::Superscript),
                    "subscript" => Ok(Mark::Subscript),

                    // Complex core marks
                    "link" => {
                        let href = fields
                            .get("href")
                            .and_then(serde_json::Value::as_str)
                            .ok_or_else(|| de::Error::missing_field("href"))?
                            .to_string();
                        let title = fields
                            .get("title")
                            .and_then(serde_json::Value::as_str)
                            .map(ToString::to_string);
                        Ok(Mark::Link { href, title })
                    }
                    "anchor" => {
                        let id = fields
                            .get("id")
                            .and_then(serde_json::Value::as_str)
                            .ok_or_else(|| de::Error::missing_field("id"))?
                            .to_string();
                        Ok(Mark::Anchor { id })
                    }
                    "footnote" => {
                        let number = fields
                            .get("number")
                            .and_then(serde_json::Value::as_u64)
                            .ok_or_else(|| de::Error::missing_field("number"))?;
                        let id = fields
                            .get("id")
                            .and_then(serde_json::Value::as_str)
                            .map(ToString::to_string);
                        Ok(Mark::Footnote {
                            number: u32::try_from(number)
                                .map_err(|_| de::Error::custom("footnote number too large"))?,
                            id,
                        })
                    }
                    "math" => {
                        let format_val = fields
                            .get("format")
                            .ok_or_else(|| de::Error::missing_field("format"))?;
                        let format: MathFormat = serde_json::from_value(format_val.clone())
                            .map_err(de::Error::custom)?;
                        // Accept both "source" and "value" (backward compat)
                        let source = fields
                            .get("source")
                            .or_else(|| fields.get("value"))
                            .and_then(serde_json::Value::as_str)
                            .ok_or_else(|| de::Error::missing_field("source"))?
                            .to_string();
                        Ok(Mark::Math { format, source })
                    }

                    // Old format backward compat: {"type": "extension", "namespace": "...", "markType": "..."}
                    "extension" => {
                        let namespace = fields
                            .get("namespace")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or("")
                            .to_string();
                        let mark_type = fields
                            .get("markType")
                            .and_then(serde_json::Value::as_str)
                            .unwrap_or("")
                            .to_string();
                        let attributes = fields
                            .get("attributes")
                            .cloned()
                            .unwrap_or(serde_json::Value::Null);
                        Ok(Mark::Extension(ExtensionMark {
                            namespace,
                            mark_type,
                            attributes,
                        }))
                    }

                    // Colon-delimited extension type or unknown type
                    other => {
                        let (namespace, mark_type) = if let Some((ns, mt)) = other.split_once(':') {
                            (ns.to_string(), mt.to_string())
                        } else {
                            (infer_mark_namespace(other).to_string(), other.to_string())
                        };
                        let attributes = if fields.is_empty() {
                            serde_json::Value::Null
                        } else {
                            serde_json::Value::Object(fields)
                        };
                        Ok(Mark::Extension(ExtensionMark {
                            namespace,
                            mark_type,
                            attributes,
                        }))
                    }
                }
            }
        }

        deserializer.deserialize_any(MarkVisitor)
    }
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
            Self::Math { .. } => MarkType::Math,
            Self::Extension(_) => MarkType::Extension,
        }
    }

    /// Check if this mark is an extension mark.
    #[must_use]
    pub fn is_extension(&self) -> bool {
        matches!(self, Self::Extension(_))
    }

    /// Get the extension mark if this is one.
    #[must_use]
    pub fn as_extension(&self) -> Option<&ExtensionMark> {
        match self {
            Self::Extension(ext) => Some(ext),
            _ => None,
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
    /// Math mark type.
    Math,
    /// Extension mark type.
    Extension,
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
        // Simple marks serialize as strings
        assert!(json.contains("\"bold\""));
    }

    #[test]
    fn test_text_deserialization() {
        // New format: simple marks as strings
        let json = r#"{"value":"Test","marks":["bold","italic"]}"#;
        let text: Text = serde_json::from_str(json).unwrap();
        assert_eq!(text.value, "Test");
        assert_eq!(text.marks.len(), 2);
        assert_eq!(text.marks[0], Mark::Bold);
        assert_eq!(text.marks[1], Mark::Italic);
    }

    #[test]
    fn test_text_deserialization_object_format() {
        // Old format: simple marks as objects (backward compat)
        let json = r#"{"value":"Test","marks":[{"type":"bold"},{"type":"italic"}]}"#;
        let text: Text = serde_json::from_str(json).unwrap();
        assert_eq!(text.value, "Test");
        assert_eq!(text.marks.len(), 2);
        assert_eq!(text.marks[0], Mark::Bold);
        assert_eq!(text.marks[1], Mark::Italic);
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

    #[test]
    fn test_math_mark() {
        use crate::content::block::MathFormat;

        let mark = Mark::Math {
            format: MathFormat::Latex,
            source: "E = mc^2".to_string(),
        };
        assert_eq!(mark.mark_type(), MarkType::Math);
    }

    #[test]
    fn test_math_mark_serialization() {
        use crate::content::block::MathFormat;

        let mark = Mark::Math {
            format: MathFormat::Latex,
            source: "\\frac{1}{2}".to_string(),
        };
        let json = serde_json::to_string(&mark).unwrap();
        assert!(json.contains("\"type\":\"math\""));
        assert!(json.contains("\"format\":\"latex\""));
        assert!(json.contains("\"source\":\"\\\\frac{1}{2}\""));
    }

    #[test]
    fn test_math_mark_deserialization() {
        use crate::content::block::MathFormat;

        let json = r#"{"type":"math","format":"mathml","source":"<math>...</math>"}"#;
        let mark: Mark = serde_json::from_str(json).unwrap();
        if let Mark::Math { format, source } = mark {
            assert_eq!(format, MathFormat::Mathml);
            assert_eq!(source, "<math>...</math>");
        } else {
            panic!("Expected Math mark");
        }
    }

    #[test]
    fn test_text_with_math_mark() {
        use crate::content::block::MathFormat;

        let text = Text::with_marks(
            "x²",
            vec![Mark::Math {
                format: MathFormat::Latex,
                source: "x^2".to_string(),
            }],
        );
        assert!(text.has_mark(MarkType::Math));
    }

    // Extension mark tests

    #[test]
    fn test_extension_mark_new() {
        let ext = ExtensionMark::new("semantic", "citation");
        assert_eq!(ext.namespace, "semantic");
        assert_eq!(ext.mark_type, "citation");
        assert_eq!(ext.full_type(), "semantic:citation");
    }

    #[test]
    fn test_extension_mark_parse_type() {
        assert_eq!(
            ExtensionMark::parse_type("semantic:citation"),
            Some(("semantic", "citation"))
        );
        assert_eq!(
            ExtensionMark::parse_type("legal:cite"),
            Some(("legal", "cite"))
        );
        assert_eq!(ExtensionMark::parse_type("bold"), None);
    }

    #[test]
    fn test_extension_mark_with_attributes() {
        let ext = ExtensionMark::new("semantic", "citation").with_attributes(serde_json::json!({
            "ref": "smith2023",
            "page": "42"
        }));

        assert_eq!(ext.get_string_attribute("ref"), Some("smith2023"));
        assert_eq!(ext.get_string_attribute("page"), Some("42"));
    }

    #[test]
    fn test_extension_mark_namespace_check() {
        let ext = ExtensionMark::new("semantic", "citation");
        assert!(ext.is_namespace("semantic"));
        assert!(!ext.is_namespace("legal"));
        assert!(ext.is_type("semantic", "citation"));
        assert!(!ext.is_type("semantic", "entity"));
    }

    #[test]
    fn test_mark_extension_variant() {
        let ext = ExtensionMark::new("semantic", "citation");
        let mark = Mark::Extension(ext.clone());

        assert!(mark.is_extension());
        assert_eq!(mark.mark_type(), MarkType::Extension);
        assert_eq!(
            mark.as_extension().unwrap().full_type(),
            "semantic:citation"
        );
    }

    #[test]
    fn test_extension_mark_serialization() {
        let ext = ExtensionMark::new("semantic", "citation").with_attributes(serde_json::json!({
            "ref": "smith2023"
        }));
        let mark = Mark::Extension(ext);

        let json = serde_json::to_string(&mark).unwrap();
        // New format: type is "namespace:markType", attributes flattened
        assert!(json.contains("\"type\":\"semantic:citation\""));
        assert!(json.contains("\"ref\":\"smith2023\""));
        // Should NOT contain old wrapper fields
        assert!(!json.contains("\"namespace\""));
        assert!(!json.contains("\"markType\""));
    }

    #[test]
    fn test_extension_mark_deserialization_new_format() {
        // New format: colon-delimited type with flattened attributes
        let json = r#"{
            "type": "legal:cite",
            "citation": "Brown v. Board of Education"
        }"#;
        let mark: Mark = serde_json::from_str(json).unwrap();

        if let Mark::Extension(ext) = mark {
            assert_eq!(ext.namespace, "legal");
            assert_eq!(ext.mark_type, "cite");
            assert_eq!(
                ext.get_string_attribute("citation"),
                Some("Brown v. Board of Education")
            );
        } else {
            panic!("Expected Extension mark");
        }
    }

    #[test]
    fn test_extension_mark_deserialization_old_format() {
        // Old format backward compat: "extension" wrapper with namespace/markType
        let json = r#"{
            "type": "extension",
            "namespace": "legal",
            "markType": "cite",
            "attributes": {
                "citation": "Brown v. Board of Education"
            }
        }"#;
        let mark: Mark = serde_json::from_str(json).unwrap();

        if let Mark::Extension(ext) = mark {
            assert_eq!(ext.namespace, "legal");
            assert_eq!(ext.mark_type, "cite");
            assert_eq!(
                ext.get_string_attribute("citation"),
                Some("Brown v. Board of Education")
            );
        } else {
            panic!("Expected Extension mark");
        }
    }

    #[test]
    fn test_text_with_extension_mark() {
        let mark = Mark::Extension(ExtensionMark::citation("smith2023"));
        let text = Text::with_marks("important claim", vec![mark]);

        assert!(text.has_mark(MarkType::Extension));
        if let Mark::Extension(ext) = &text.marks[0] {
            assert_eq!(ext.namespace, "semantic");
            assert_eq!(ext.mark_type, "citation");
        } else {
            panic!("Expected Extension mark");
        }
    }

    #[test]
    fn test_citation_convenience() {
        let ext = ExtensionMark::citation("smith2023");
        assert!(ext.is_type("semantic", "citation"));
        assert_eq!(ext.get_string_attribute("ref"), Some("smith2023"));
    }

    #[test]
    fn test_citation_with_page_convenience() {
        let ext = ExtensionMark::citation_with_page("smith2023", "42-45");
        assert!(ext.is_type("semantic", "citation"));
        assert_eq!(ext.get_string_attribute("ref"), Some("smith2023"));
        assert_eq!(ext.get_string_attribute("locator"), Some("42-45"));
        assert_eq!(ext.get_string_attribute("locatorType"), Some("page"));
    }

    #[test]
    fn test_entity_convenience() {
        let ext = ExtensionMark::entity("https://www.wikidata.org/wiki/Q937", "person");
        assert!(ext.is_type("semantic", "entity"));
        assert_eq!(
            ext.get_string_attribute("uri"),
            Some("https://www.wikidata.org/wiki/Q937")
        );
        assert_eq!(ext.get_string_attribute("entityType"), Some("person"));
    }

    #[test]
    fn test_glossary_convenience() {
        let ext = ExtensionMark::glossary("api-term");
        assert!(ext.is_type("semantic", "glossary"));
        assert_eq!(ext.get_string_attribute("termId"), Some("api-term"));
    }

    #[test]
    fn test_index_convenience() {
        let ext = ExtensionMark::index("algorithm");
        assert!(ext.is_type("presentation", "index"));
        assert_eq!(ext.get_string_attribute("term"), Some("algorithm"));
    }

    #[test]
    fn test_index_with_subterm_convenience() {
        let ext = ExtensionMark::index_with_subterm("algorithm", "sorting");
        assert!(ext.is_type("presentation", "index"));
        assert_eq!(ext.get_string_attribute("term"), Some("algorithm"));
        assert_eq!(ext.get_string_attribute("subterm"), Some("sorting"));
    }

    #[test]
    fn test_non_extension_mark_as_extension() {
        let mark = Mark::Bold;
        assert!(!mark.is_extension());
        assert!(mark.as_extension().is_none());
    }

    #[test]
    fn test_equation_ref_convenience() {
        let ext = ExtensionMark::equation_ref("#eq-pythagoras");
        assert!(ext.is_type("academic", "equation-ref"));
        assert_eq!(ext.get_string_attribute("target"), Some("#eq-pythagoras"));
        assert!(ext.get_string_attribute("format").is_none());
    }

    #[test]
    fn test_equation_ref_formatted_convenience() {
        let ext = ExtensionMark::equation_ref_formatted("#eq-1", "Equation ({number})");
        assert!(ext.is_type("academic", "equation-ref"));
        assert_eq!(ext.get_string_attribute("target"), Some("#eq-1"));
        assert_eq!(
            ext.get_string_attribute("format"),
            Some("Equation ({number})")
        );
    }

    #[test]
    fn test_algorithm_ref_convenience() {
        let ext = ExtensionMark::algorithm_ref("#alg-quicksort");
        assert!(ext.is_type("academic", "algorithm-ref"));
        assert_eq!(ext.get_string_attribute("target"), Some("#alg-quicksort"));
        assert!(ext.get_string_attribute("line").is_none());
    }

    #[test]
    fn test_algorithm_ref_line_convenience() {
        let ext = ExtensionMark::algorithm_ref_line("#alg-bisection", "loop");
        assert!(ext.is_type("academic", "algorithm-ref"));
        assert_eq!(ext.get_string_attribute("target"), Some("#alg-bisection"));
        assert_eq!(ext.get_string_attribute("line"), Some("loop"));
    }

    #[test]
    fn test_algorithm_ref_formatted_convenience() {
        let ext = ExtensionMark::algorithm_ref_formatted("#alg-1", "Algorithm {number}");
        assert!(ext.is_type("academic", "algorithm-ref"));
        assert_eq!(ext.get_string_attribute("target"), Some("#alg-1"));
        assert_eq!(
            ext.get_string_attribute("format"),
            Some("Algorithm {number}")
        );
    }

    #[test]
    fn test_algorithm_ref_line_formatted_convenience() {
        let ext = ExtensionMark::algorithm_ref_line_formatted("#alg-1", "pivot", "line {line}");
        assert!(ext.is_type("academic", "algorithm-ref"));
        assert_eq!(ext.get_string_attribute("target"), Some("#alg-1"));
        assert_eq!(ext.get_string_attribute("line"), Some("pivot"));
        assert_eq!(ext.get_string_attribute("format"), Some("line {line}"));
    }

    #[test]
    fn test_theorem_ref_convenience() {
        let ext = ExtensionMark::theorem_ref("#thm-pythagoras");
        assert!(ext.is_type("academic", "theorem-ref"));
        assert_eq!(ext.get_string_attribute("target"), Some("#thm-pythagoras"));
    }

    #[test]
    fn test_theorem_ref_formatted_convenience() {
        let ext = ExtensionMark::theorem_ref_formatted("#thm-1", "{variant} {number}");
        assert!(ext.is_type("academic", "theorem-ref"));
        assert_eq!(ext.get_string_attribute("target"), Some("#thm-1"));
        assert_eq!(
            ext.get_string_attribute("format"),
            Some("{variant} {number}")
        );
    }

    #[test]
    fn test_highlight_mark_convenience() {
        let ext = ExtensionMark::highlight("yellow");
        assert!(ext.is_type("collaboration", "highlight"));
        assert_eq!(ext.get_string_attribute("color"), Some("yellow"));
    }

    #[test]
    fn test_highlight_yellow_convenience() {
        let ext = ExtensionMark::highlight_yellow();
        assert!(ext.is_type("collaboration", "highlight"));
        assert_eq!(ext.get_string_attribute("color"), Some("yellow"));
    }

    #[test]
    fn test_highlight_colored_convenience() {
        // Test with a string that would come from HighlightColor::display()
        let ext = ExtensionMark::highlight_colored("green");
        assert!(ext.is_type("collaboration", "highlight"));
        assert_eq!(ext.get_string_attribute("color"), Some("green"));
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    /// Generate arbitrary text content.
    fn arb_text_value() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9 .,!?]{0,100}".prop_map(|s| s)
    }

    /// Generate arbitrary URL for links.
    fn arb_url() -> impl Strategy<Value = String> {
        "(https?://[a-z]{3,10}\\.[a-z]{2,4}(/[a-z0-9]{0,10})?)".prop_map(|s| s)
    }

    /// Generate arbitrary basic mark (no associated data).
    fn arb_simple_mark() -> impl Strategy<Value = Mark> {
        prop_oneof![
            Just(Mark::Bold),
            Just(Mark::Italic),
            Just(Mark::Underline),
            Just(Mark::Strikethrough),
            Just(Mark::Code),
            Just(Mark::Superscript),
            Just(Mark::Subscript),
        ]
    }

    /// Generate arbitrary link mark.
    fn arb_link_mark() -> impl Strategy<Value = Mark> {
        (arb_url(), prop::option::of("[a-zA-Z ]{0,20}"))
            .prop_map(|(href, title)| Mark::Link { href, title })
    }

    /// Generate arbitrary footnote mark.
    fn arb_footnote_mark() -> impl Strategy<Value = Mark> {
        (1u32..1000u32, prop::option::of("[a-z]{2,10}"))
            .prop_map(|(number, id)| Mark::Footnote { number, id })
    }

    /// Generate arbitrary mark.
    fn arb_mark() -> impl Strategy<Value = Mark> {
        prop_oneof![arb_simple_mark(), arb_link_mark(), arb_footnote_mark(),]
    }

    /// Generate arbitrary text node.
    fn arb_text() -> impl Strategy<Value = Text> {
        (arb_text_value(), prop::collection::vec(arb_mark(), 0..3))
            .prop_map(|(value, marks)| Text { value, marks })
    }

    proptest! {
        /// Plain text has no marks.
        #[test]
        fn plain_text_no_marks(value in arb_text_value()) {
            let text = Text::plain(&value);
            prop_assert_eq!(&text.value, &value);
            prop_assert!(text.marks.is_empty());
            prop_assert!(!text.has_marks());
        }

        /// Bold text has exactly one bold mark.
        #[test]
        fn bold_text_has_bold_mark(value in arb_text_value()) {
            let text = Text::bold(&value);
            prop_assert_eq!(&text.value, &value);
            prop_assert_eq!(text.marks.len(), 1);
            prop_assert!(text.has_mark(MarkType::Bold));
        }

        /// Italic text has exactly one italic mark.
        #[test]
        fn italic_text_has_italic_mark(value in arb_text_value()) {
            let text = Text::italic(&value);
            prop_assert_eq!(&text.value, &value);
            prop_assert_eq!(text.marks.len(), 1);
            prop_assert!(text.has_mark(MarkType::Italic));
        }

        /// Code text has exactly one code mark.
        #[test]
        fn code_text_has_code_mark(value in arb_text_value()) {
            let text = Text::code(&value);
            prop_assert_eq!(&text.value, &value);
            prop_assert_eq!(text.marks.len(), 1);
            prop_assert!(text.has_mark(MarkType::Code));
        }

        /// Link text has exactly one link mark with correct href.
        #[test]
        fn link_text_has_link_mark(value in arb_text_value(), href in arb_url()) {
            let text = Text::link(&value, &href);
            prop_assert_eq!(&text.value, &value);
            prop_assert_eq!(text.marks.len(), 1);
            prop_assert!(text.has_mark(MarkType::Link));
            if let Mark::Link { href: actual_href, .. } = &text.marks[0] {
                prop_assert_eq!(actual_href, &href);
            }
        }

        /// Text JSON roundtrip - serialize and deserialize should preserve data.
        #[test]
        fn text_json_roundtrip(text in arb_text()) {
            let json = serde_json::to_string(&text).unwrap();
            let parsed: Text = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(text, parsed);
        }

        /// Mark JSON roundtrip - serialize and deserialize should preserve data.
        #[test]
        fn mark_json_roundtrip(mark in arb_mark()) {
            let json = serde_json::to_string(&mark).unwrap();
            let parsed: Mark = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(mark, parsed);
        }

        /// Simple mark types are identified correctly.
        #[test]
        fn simple_mark_types(mark in arb_simple_mark()) {
            let expected = match mark {
                Mark::Bold => MarkType::Bold,
                Mark::Italic => MarkType::Italic,
                Mark::Underline => MarkType::Underline,
                Mark::Strikethrough => MarkType::Strikethrough,
                Mark::Code => MarkType::Code,
                Mark::Superscript => MarkType::Superscript,
                Mark::Subscript => MarkType::Subscript,
                Mark::Link { .. }
                | Mark::Anchor { .. }
                | Mark::Footnote { .. }
                | Mark::Math { .. }
                | Mark::Extension(_) => {
                    // arb_simple_mark() should never generate these variants
                    prop_assert!(false, "arb_simple_mark() generated non-simple mark: {mark:?}");
                    return Ok(());
                }
            };
            prop_assert_eq!(mark.mark_type(), expected);
        }

        /// Link marks return Link type.
        #[test]
        fn link_mark_type(mark in arb_link_mark()) {
            prop_assert_eq!(mark.mark_type(), MarkType::Link);
        }

        /// Footnote marks return Footnote type.
        #[test]
        fn footnote_mark_type(mark in arb_footnote_mark()) {
            prop_assert_eq!(mark.mark_type(), MarkType::Footnote);
        }

        /// has_marks is consistent with marks vector.
        #[test]
        fn has_marks_consistent(text in arb_text()) {
            prop_assert_eq!(text.has_marks(), !text.marks.is_empty());
        }
    }
}
