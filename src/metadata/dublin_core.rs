//! Dublin Core metadata.

use serde::{Deserialize, Serialize};

/// Dublin Core metadata file structure.
///
/// This represents the `metadata/dublin-core.json` file in a Codex document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DublinCore {
    /// Dublin Core version (e.g., "1.1").
    pub version: String,

    /// Dublin Core terms.
    pub terms: DublinCoreTerms,
}

impl DublinCore {
    /// Create a new Dublin Core metadata structure with required fields.
    #[must_use]
    pub fn new(title: impl Into<String>, creator: impl Into<String>) -> Self {
        Self {
            version: "1.1".to_string(),
            terms: DublinCoreTerms {
                title: title.into(),
                creator: StringOrArray::Single(creator.into()),
                subject: None,
                description: None,
                publisher: None,
                contributor: None,
                date: None,
                dc_type: None,
                format: None,
                identifier: None,
                source: None,
                language: None,
                relation: None,
                coverage: None,
                rights: None,
            },
        }
    }

    /// Get the document title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.terms.title
    }

    /// Get the creator(s) as a slice.
    #[must_use]
    pub fn creators(&self) -> Vec<&str> {
        self.terms.creator.as_slice()
    }

    /// Get the description if present.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.terms.description.as_deref()
    }

    /// Get the language if present.
    #[must_use]
    pub fn language(&self) -> Option<&str> {
        self.terms.language.as_deref()
    }
}

/// Dublin Core terms.
///
/// These are the standard 15 Dublin Core elements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DublinCoreTerms {
    /// Document title (required).
    pub title: String,

    /// Author(s) (required).
    pub creator: StringOrArray,

    /// Topics or keywords.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<StringOrArray>,

    /// Summary or abstract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Publishing entity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,

    /// Other contributors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contributor: Option<StringOrArray>,

    /// Publication date (ISO 8601).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,

    /// Nature or genre of content.
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub dc_type: Option<String>,

    /// MIME type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,

    /// Unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,

    /// Source reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Language code (BCP 47).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Related resource.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relation: Option<String>,

    /// Scope (temporal/spatial).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage: Option<String>,

    /// Rights statement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rights: Option<String>,
}

/// A string or array of strings.
///
/// Dublin Core terms can be either a single string or an array of strings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrArray {
    /// Single string value.
    Single(String),
    /// Multiple string values.
    Multiple(Vec<String>),
}

impl StringOrArray {
    /// Get values as a slice of string references.
    #[must_use]
    pub fn as_slice(&self) -> Vec<&str> {
        match self {
            Self::Single(s) => vec![s.as_str()],
            Self::Multiple(v) => v.iter().map(String::as_str).collect(),
        }
    }

    /// Get the first value.
    #[must_use]
    pub fn first(&self) -> &str {
        match self {
            Self::Single(s) => s,
            Self::Multiple(v) => v.first().map_or("", String::as_str),
        }
    }

    /// Check if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Single(s) => s.is_empty(),
            Self::Multiple(v) => v.is_empty(),
        }
    }
}

impl From<String> for StringOrArray {
    fn from(s: String) -> Self {
        Self::Single(s)
    }
}

impl From<&str> for StringOrArray {
    fn from(s: &str) -> Self {
        Self::Single(s.to_string())
    }
}

impl From<Vec<String>> for StringOrArray {
    fn from(v: Vec<String>) -> Self {
        Self::Multiple(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dublin_core_new() {
        let dc = DublinCore::new("Test Document", "Author Name");
        assert_eq!(dc.title(), "Test Document");
        assert_eq!(dc.creators(), vec!["Author Name"]);
        assert_eq!(dc.version, "1.1");
    }

    #[test]
    fn test_string_or_array() {
        let single = StringOrArray::Single("one".to_string());
        assert_eq!(single.as_slice(), vec!["one"]);
        assert_eq!(single.first(), "one");

        let multiple = StringOrArray::Multiple(vec!["one".to_string(), "two".to_string()]);
        assert_eq!(multiple.as_slice(), vec!["one", "two"]);
        assert_eq!(multiple.first(), "one");
    }

    #[test]
    fn test_serialization() {
        let dc = DublinCore::new("Test", "Author");
        let json = serde_json::to_string_pretty(&dc).unwrap();
        assert!(json.contains("\"title\": \"Test\""));
        assert!(json.contains("\"creator\": \"Author\""));
    }

    #[test]
    fn test_deserialization_single_creator() {
        let json = r#"{
            "version": "1.1",
            "terms": {
                "title": "My Document",
                "creator": "John Doe"
            }
        }"#;
        let dc: DublinCore = serde_json::from_str(json).unwrap();
        assert_eq!(dc.title(), "My Document");
        assert_eq!(dc.creators(), vec!["John Doe"]);
    }

    #[test]
    fn test_deserialization_multiple_creators() {
        let json = r#"{
            "version": "1.1",
            "terms": {
                "title": "Collaboration",
                "creator": ["Alice", "Bob", "Charlie"],
                "subject": ["Science", "Research"]
            }
        }"#;
        let dc: DublinCore = serde_json::from_str(json).unwrap();
        assert_eq!(dc.creators(), vec!["Alice", "Bob", "Charlie"]);
        assert_eq!(
            dc.terms.subject.as_ref().unwrap().as_slice(),
            vec!["Science", "Research"]
        );
    }

    #[test]
    fn test_full_dublin_core() {
        let json = r#"{
            "version": "1.1",
            "terms": {
                "title": "Annual Report 2025",
                "creator": ["Jane Doe", "John Smith"],
                "subject": ["Finance", "Annual Report"],
                "description": "Comprehensive annual financial report",
                "publisher": "Acme Corporation",
                "contributor": "Finance Team",
                "date": "2025-01-15",
                "type": "Text",
                "format": "application/vnd.codex+json",
                "identifier": "sha256:3a7bd3e2",
                "language": "en",
                "coverage": "2024 fiscal year",
                "rights": "Copyright 2025 Acme Corporation"
            }
        }"#;
        let dc: DublinCore = serde_json::from_str(json).unwrap();
        assert_eq!(dc.title(), "Annual Report 2025");
        assert_eq!(dc.description(), Some("Comprehensive annual financial report"));
        assert_eq!(dc.language(), Some("en"));
    }
}
