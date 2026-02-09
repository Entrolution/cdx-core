//! Entity linking to external knowledge bases.

use serde::{Deserialize, Serialize};

/// A link to an external entity in a knowledge base.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityLink {
    /// URI of the entity.
    pub uri: String,

    /// Type of entity.
    pub entity_type: EntityType,

    /// Display label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    /// Confidence score (0.0 to 1.0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,

    /// Source knowledge base.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<KnowledgeBase>,
}

impl EntityLink {
    /// Create a new entity link.
    #[must_use]
    pub fn new(uri: impl Into<String>, entity_type: EntityType) -> Self {
        Self {
            uri: uri.into(),
            entity_type,
            label: None,
            confidence: None,
            source: None,
        }
    }

    /// Set the display label.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set confidence score.
    #[must_use]
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = Some(confidence.clamp(0.0, 1.0));
        self
    }

    /// Set source knowledge base.
    #[must_use]
    pub fn with_source(mut self, source: KnowledgeBase) -> Self {
        self.source = Some(source);
        self
    }

    /// Create a Wikipedia entity link.
    #[must_use]
    pub fn wikipedia(title: impl Into<String>, entity_type: EntityType) -> Self {
        let title = title.into();
        let uri = format!("https://en.wikipedia.org/wiki/{}", title.replace(' ', "_"));
        Self::new(uri, entity_type)
            .with_label(title)
            .with_source(KnowledgeBase::Wikipedia)
    }

    /// Create a Wikidata entity link.
    #[must_use]
    pub fn wikidata(qid: impl Into<String>, entity_type: EntityType) -> Self {
        let qid = qid.into();
        let uri = format!("https://www.wikidata.org/wiki/{qid}");
        Self::new(uri, entity_type).with_source(KnowledgeBase::Wikidata)
    }
}

/// Type of entity being linked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "kebab-case")]
pub enum EntityType {
    /// A person.
    Person,
    /// An organization or company.
    Organization,
    /// A geographic location.
    Place,
    /// A historical or scheduled event.
    Event,
    /// A product.
    Product,
    /// A creative work (book, film, etc.).
    CreativeWork,
    /// A concept or idea.
    Concept,
    /// A scientific term or phenomenon.
    Scientific,
    /// A time period or era.
    TimePeriod,
    /// Other entity type.
    Other,
}

/// Known knowledge bases for entity linking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KnowledgeBase {
    /// Wikipedia.
    Wikipedia,
    /// Wikidata.
    Wikidata,
    /// `DBpedia`.
    Dbpedia,
    /// Schema.org.
    Schema,
    /// Library of Congress.
    Loc,
    /// `GeoNames`.
    Geonames,
    /// Other knowledge base.
    Other,
}
