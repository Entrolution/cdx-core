//! Glossary and term definitions for documents.

use serde::{Deserialize, Serialize};

/// A glossary containing term definitions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Glossary {
    /// Glossary terms.
    pub terms: Vec<GlossaryTerm>,
}

impl Glossary {
    /// Create a new empty glossary.
    #[must_use]
    pub fn new() -> Self {
        Self { terms: Vec::new() }
    }

    /// Add a term.
    pub fn add_term(&mut self, term: GlossaryTerm) {
        self.terms.push(term);
    }

    /// Find a term by its ID.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&GlossaryTerm> {
        self.terms.iter().find(|t| t.id == id)
    }

    /// Find terms by text (case-insensitive).
    #[must_use]
    pub fn find_by_text(&self, text: &str) -> Option<&GlossaryTerm> {
        let lower = text.to_lowercase();
        self.terms.iter().find(|t| {
            t.term.to_lowercase() == lower || t.aliases.iter().any(|a| a.to_lowercase() == lower)
        })
    }

    /// Get the number of terms.
    #[must_use]
    pub fn len(&self) -> usize {
        self.terms.len()
    }

    /// Check if the glossary is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }
}

impl Default for Glossary {
    fn default() -> Self {
        Self::new()
    }
}

/// A glossary term definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlossaryTerm {
    /// Unique identifier.
    pub id: String,

    /// The term being defined.
    pub term: String,

    /// Definition text.
    pub definition: String,

    /// Alternative forms or spellings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,

    /// Related terms (by ID).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub see_also: Vec<String>,

    /// Category or subject area.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Pronunciation guide.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pronunciation: Option<String>,

    /// Etymology or origin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub etymology: Option<String>,

    /// Usage notes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<String>,
}

impl GlossaryTerm {
    /// Create a new glossary term.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        term: impl Into<String>,
        definition: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            term: term.into(),
            definition: definition.into(),
            aliases: Vec::new(),
            see_also: Vec::new(),
            category: None,
            pronunciation: None,
            etymology: None,
            usage: None,
        }
    }

    /// Add an alias.
    #[must_use]
    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    /// Add a "see also" reference.
    #[must_use]
    pub fn with_see_also(mut self, term_id: impl Into<String>) -> Self {
        self.see_also.push(term_id.into());
        self
    }

    /// Set category.
    #[must_use]
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Set pronunciation.
    #[must_use]
    pub fn with_pronunciation(mut self, pronunciation: impl Into<String>) -> Self {
        self.pronunciation = Some(pronunciation.into());
        self
    }
}

/// A reference to a glossary term in the document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlossaryRef {
    /// ID of the glossary term.
    pub term_id: String,

    /// Display text (if different from term).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
}

impl GlossaryRef {
    /// Create a new glossary reference.
    #[must_use]
    pub fn new(term_id: impl Into<String>) -> Self {
        Self {
            term_id: term_id.into(),
            display: None,
        }
    }

    /// Set custom display text.
    #[must_use]
    pub fn with_display(mut self, display: impl Into<String>) -> Self {
        self.display = Some(display.into());
        self
    }
}
