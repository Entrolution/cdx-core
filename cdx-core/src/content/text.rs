//! Text nodes and formatting marks.

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

    /// Inline mathematical expression.
    Math {
        /// Math format (latex or mathml).
        format: MathFormat,

        /// The mathematical expression.
        value: String,
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

    #[test]
    fn test_math_mark() {
        use crate::content::block::MathFormat;

        let mark = Mark::Math {
            format: MathFormat::Latex,
            value: "E = mc^2".to_string(),
        };
        assert_eq!(mark.mark_type(), MarkType::Math);
    }

    #[test]
    fn test_math_mark_serialization() {
        use crate::content::block::MathFormat;

        let mark = Mark::Math {
            format: MathFormat::Latex,
            value: "\\frac{1}{2}".to_string(),
        };
        let json = serde_json::to_string(&mark).unwrap();
        assert!(json.contains("\"type\":\"math\""));
        assert!(json.contains("\"format\":\"latex\""));
        assert!(json.contains("\"value\":\"\\\\frac{1}{2}\""));
    }

    #[test]
    fn test_math_mark_deserialization() {
        use crate::content::block::MathFormat;

        let json = r#"{"type":"math","format":"mathml","value":"<math>...</math>"}"#;
        let mark: Mark = serde_json::from_str(json).unwrap();
        if let Mark::Math { format, value } = mark {
            assert_eq!(format, MathFormat::Mathml);
            assert_eq!(value, "<math>...</math>");
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
                value: "x^2".to_string(),
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
        assert!(json.contains("\"type\":\"extension\""));
        assert!(json.contains("\"namespace\":\"semantic\""));
        assert!(json.contains("\"markType\":\"citation\""));
        assert!(json.contains("\"ref\":\"smith2023\""));
    }

    #[test]
    fn test_extension_mark_deserialization() {
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
