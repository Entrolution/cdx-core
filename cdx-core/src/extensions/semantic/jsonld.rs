//! JSON-LD metadata for semantic web integration.

use serde::{Deserialize, Serialize};

use super::BibliographyEntry;

/// JSON-LD metadata for semantic web integration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonLdMetadata {
    /// JSON-LD context URIs.
    #[serde(rename = "@context")]
    pub context: Vec<String>,

    /// JSON-LD graph containing structured data.
    #[serde(rename = "@graph", default, skip_serializing_if = "Vec::is_empty")]
    pub graph: Vec<serde_json::Value>,
}

impl JsonLdMetadata {
    /// Create new JSON-LD metadata with schema.org context.
    #[must_use]
    pub fn new() -> Self {
        Self {
            context: vec!["https://schema.org".to_string()],
            graph: Vec::new(),
        }
    }

    /// Add a context URI.
    #[must_use]
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context.push(context.into());
        self
    }

    /// Add a graph node.
    pub fn add_node(&mut self, node: serde_json::Value) {
        self.graph.push(node);
    }

    /// Create a JSON-LD representation of a creative work.
    #[must_use]
    pub fn creative_work(name: impl Into<String>, author: impl Into<String>) -> serde_json::Value {
        serde_json::json!({
            "@type": "CreativeWork",
            "name": name.into(),
            "author": {
                "@type": "Person",
                "name": author.into()
            }
        })
    }

    /// Create a JSON-LD representation of a scholarly article.
    #[must_use]
    pub fn scholarly_article(entry: &BibliographyEntry) -> serde_json::Value {
        let mut article = serde_json::json!({
            "@type": "ScholarlyArticle",
            "name": entry.title
        });

        if !entry.authors.is_empty() {
            let authors: Vec<_> = entry
                .authors
                .iter()
                .map(|a| {
                    serde_json::json!({
                        "@type": "Person",
                        "name": a.display_name()
                    })
                })
                .collect();
            article["author"] = serde_json::json!(authors);
        }

        if let Some(date) = &entry.issued {
            article["datePublished"] = serde_json::json!(date.to_string());
        }

        if let Some(doi) = &entry.doi {
            article["identifier"] = serde_json::json!({
                "@type": "PropertyValue",
                "propertyID": "DOI",
                "value": doi
            });
        }

        if let Some(journal) = &entry.container_title {
            article["isPartOf"] = serde_json::json!({
                "@type": "Periodical",
                "name": journal
            });
        }

        article
    }
}

impl Default for JsonLdMetadata {
    fn default() -> Self {
        Self::new()
    }
}
