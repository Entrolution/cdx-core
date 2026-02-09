//! Semantic extension for citations, bibliographies, glossaries, and entity linking.
//!
//! This extension provides structured semantic content types for academic
//! and professional documents.
//!
//! # Features
//!
//! - **Bibliography**: Manage references with multiple citation styles
//! - **Citations**: Inline references to bibliography entries
//! - **Glossary**: Term definitions with cross-references
//! - **Entity Linking**: Connect mentions to external knowledge bases
//!
//! # Example
//!
//! ```json
//! {
//!   "type": "semantic:citation",
//!   "ref": "smith2023",
//!   "page": "42-45",
//!   "prefix": "see",
//!   "suffix": "for details"
//! }
//! ```

mod bibliography;
mod citation;
mod entity;
mod glossary;
mod jsonld;

// Re-export all public types for backwards compatibility
pub use bibliography::{
    Author, Bibliography, BibliographyEntry, CitationStyle, EntryType, PartialDate,
};
pub use citation::{Citation, Footnote, LocatorType};
pub use entity::{EntityLink, EntityType, KnowledgeBase};
pub use glossary::{Glossary, GlossaryRef, GlossaryTerm};
pub use jsonld::JsonLdMetadata;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Bibliography tests
    #[test]
    fn test_bibliography_new() {
        let bib = Bibliography::new(CitationStyle::Apa);
        assert!(bib.is_empty());
        assert_eq!(bib.style, CitationStyle::Apa);
    }

    #[test]
    fn test_bibliography_add_entry() {
        let mut bib = Bibliography::default();
        let entry = BibliographyEntry::new("smith2023", EntryType::Article, "A Great Paper");
        bib.add_entry(entry);

        assert_eq!(bib.len(), 1);
        assert!(bib.contains("smith2023"));
        assert!(!bib.contains("jones2024"));
    }

    #[test]
    fn test_bibliography_entry_builder() {
        let entry =
            BibliographyEntry::new("smith2023", EntryType::Article, "Deep Learning Advances")
                .with_author(Author::new("John", "Smith"))
                .with_author(Author::new("Jane", "Doe"))
                .with_issued(PartialDate::year(2023))
                .with_container("Nature")
                .with_volume_issue("100", Some("5".to_string()))
                .with_pages("123-145")
                .with_doi("10.1234/nature.2023.1234");

        assert_eq!(entry.id, "smith2023");
        assert_eq!(entry.authors.len(), 2);
        assert_eq!(entry.container_title, Some("Nature".to_string()));
        assert_eq!(entry.doi, Some("10.1234/nature.2023.1234".to_string()));
    }

    #[test]
    fn test_author_display_name() {
        let author1 = Author::new("John", "Smith");
        assert_eq!(author1.display_name(), "Smith, John");

        let author2 = Author::literal("World Health Organization");
        assert_eq!(author2.display_name(), "World Health Organization");
    }

    #[test]
    fn test_partial_date_display() {
        assert_eq!(PartialDate::year(2023).to_string(), "2023");
        assert_eq!(PartialDate::year_month(2023, 6).to_string(), "2023-06");
        assert_eq!(PartialDate::full(2023, 6, 15).to_string(), "2023-06-15");
        assert_eq!(
            PartialDate::seasonal(2023, "Spring").to_string(),
            "Spring 2023"
        );
    }

    // Citation tests
    #[test]
    fn test_citation_new() {
        let cite = Citation::new("smith2023");
        assert_eq!(cite.reference, "smith2023");
        assert!(!cite.suppress_author);
    }

    #[test]
    fn test_citation_with_page() {
        let cite = Citation::new("smith2023")
            .with_page("42")
            .with_prefix("see")
            .with_suffix("for details");

        assert_eq!(cite.locator, Some("42".to_string()));
        assert_eq!(cite.locator_type, Some(LocatorType::Page));
        assert_eq!(cite.prefix, Some("see".to_string()));
    }

    // Glossary tests
    #[test]
    fn test_glossary_new() {
        let glossary = Glossary::new();
        assert!(glossary.is_empty());
    }

    #[test]
    fn test_glossary_add_term() {
        let mut glossary = Glossary::default();
        let term = GlossaryTerm::new(
            "ai",
            "Artificial Intelligence",
            "The simulation of human intelligence by machines.",
        );
        glossary.add_term(term);

        assert_eq!(glossary.len(), 1);
        assert!(glossary.get("ai").is_some());
    }

    #[test]
    fn test_glossary_find_by_text() {
        let mut glossary = Glossary::new();
        glossary.add_term(
            GlossaryTerm::new("ml", "Machine Learning", "A subset of AI.")
                .with_alias("ML")
                .with_alias("statistical learning"),
        );

        assert!(glossary.find_by_text("Machine Learning").is_some());
        assert!(glossary.find_by_text("ML").is_some());
        assert!(glossary.find_by_text("ml").is_some());
        assert!(glossary.find_by_text("Deep Learning").is_none());
    }

    #[test]
    fn test_glossary_term_builder() {
        let term = GlossaryTerm::new("api", "API", "Application Programming Interface")
            .with_alias("Application Programming Interface")
            .with_see_also("rest")
            .with_category("Computing")
            .with_pronunciation("/ˌeɪpiˈaɪ/");

        assert_eq!(term.aliases.len(), 1);
        assert_eq!(term.see_also, vec!["rest"]);
        assert_eq!(term.category, Some("Computing".to_string()));
    }

    // Entity linking tests
    #[test]
    fn test_entity_link_new() {
        let link = EntityLink::new("https://example.org/entity/123", EntityType::Person);
        assert_eq!(link.entity_type, EntityType::Person);
    }

    #[test]
    fn test_entity_link_wikipedia() {
        let link = EntityLink::wikipedia("Albert Einstein", EntityType::Person);
        assert!(link.uri.contains("Albert_Einstein"));
        assert_eq!(link.source, Some(KnowledgeBase::Wikipedia));
    }

    #[test]
    fn test_entity_link_wikidata() {
        let link = EntityLink::wikidata("Q937", EntityType::Person);
        assert!(link.uri.contains("Q937"));
        assert_eq!(link.source, Some(KnowledgeBase::Wikidata));
    }

    #[test]
    fn test_entity_link_confidence() {
        let link =
            EntityLink::new("https://example.org", EntityType::Concept).with_confidence(0.95);
        assert_eq!(link.confidence, Some(0.95));

        // Test clamping
        let link2 =
            EntityLink::new("https://example.org", EntityType::Concept).with_confidence(1.5);
        assert_eq!(link2.confidence, Some(1.0));
    }

    // JSON-LD tests
    #[test]
    fn test_jsonld_new() {
        let jsonld = JsonLdMetadata::new();
        assert_eq!(jsonld.context, vec!["https://schema.org"]);
        assert!(jsonld.graph.is_empty());
    }

    #[test]
    fn test_jsonld_add_node() {
        let mut jsonld = JsonLdMetadata::new();
        jsonld.add_node(json!({
            "@type": "Person",
            "name": "John Smith"
        }));
        assert_eq!(jsonld.graph.len(), 1);
    }

    #[test]
    fn test_jsonld_scholarly_article() {
        let entry = BibliographyEntry::new("test", EntryType::Article, "Test Paper")
            .with_author(Author::new("John", "Doe"))
            .with_issued(PartialDate::year(2023))
            .with_doi("10.1234/test");

        let article = JsonLdMetadata::scholarly_article(&entry);
        assert_eq!(article["@type"], "ScholarlyArticle");
        assert_eq!(article["name"], "Test Paper");
    }

    // Serialization tests
    #[test]
    fn test_bibliography_serialization() {
        let mut bib = Bibliography::new(CitationStyle::Chicago);
        bib.add_entry(BibliographyEntry::new("test", EntryType::Book, "Test Book"));

        let json = serde_json::to_string(&bib).unwrap();
        assert!(json.contains("\"style\":\"chicago\""));
        assert!(json.contains("\"entryType\":\"book\""));
    }

    #[test]
    fn test_citation_serialization() {
        let cite = Citation::new("smith2023").with_page("42");
        let json = serde_json::to_string(&cite).unwrap();
        assert!(json.contains("\"ref\":\"smith2023\""));
        assert!(json.contains("\"locator\":\"42\""));
    }

    // Footnote tests
    #[test]
    fn test_footnote_new() {
        let fn1 = Footnote::new(1);
        assert_eq!(fn1.number, 1);
        assert_eq!(fn1.id, None);
        assert_eq!(fn1.content, None);
        assert!(fn1.children.is_empty());
    }

    #[test]
    fn test_footnote_builder() {
        let fn1 = Footnote::new(1)
            .with_id("fn-1")
            .with_content("This is the footnote text.");
        assert_eq!(fn1.number, 1);
        assert_eq!(fn1.id, Some("fn-1".to_string()));
        assert_eq!(fn1.content, Some("This is the footnote text.".to_string()));
    }

    #[test]
    fn test_footnote_serialization() {
        let fn1 = Footnote::new(1).with_content("A footnote.");
        let json_str = serde_json::to_string(&fn1).unwrap();
        assert!(json_str.contains("\"number\":1"));
        assert!(json_str.contains("\"content\":\"A footnote.\""));
        // id should be omitted when None
        assert!(!json_str.contains("\"id\""));
    }

    #[test]
    fn test_footnote_deserialization() {
        let json_str = r#"{"number":2,"id":"fn-2","content":"Some text."}"#;
        let fn2: Footnote = serde_json::from_str(json_str).unwrap();
        assert_eq!(fn2.number, 2);
        assert_eq!(fn2.id, Some("fn-2".to_string()));
        assert_eq!(fn2.content, Some("Some text.".to_string()));
    }

    #[test]
    fn test_footnote_roundtrip() {
        let original = Footnote::new(3)
            .with_id("fn-3")
            .with_content("Round-trip test.");
        let json_str = serde_json::to_string(&original).unwrap();
        let deserialized: Footnote = serde_json::from_str(&json_str).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_footnote_with_children() {
        use crate::content::{Block, Text};

        let para = Block::paragraph(vec![Text::plain("Rich footnote content.")]);
        let fn1 = Footnote::new(1).with_id("fn-1").with_children(vec![para]);

        assert!(fn1.has_children());
        assert!(!fn1.has_content());
        assert_eq!(fn1.children.len(), 1);
    }

    #[test]
    fn test_footnote_children_serialization() {
        use crate::content::{Block, Text};

        let para = Block::paragraph(vec![Text::plain("Footnote text.")]);
        let fn1 = Footnote::new(1).with_children(vec![para]);

        let json_str = serde_json::to_string(&fn1).unwrap();
        assert!(json_str.contains("\"number\":1"));
        assert!(json_str.contains("\"children\""));
        assert!(json_str.contains("\"type\":\"paragraph\""));
        // content should be omitted when None
        assert!(!json_str.contains("\"content\""));
    }

    #[test]
    fn test_footnote_children_deserialization() {
        let json_str = r#"{
            "number": 1,
            "id": "fn-1",
            "children": [
                {
                    "type": "paragraph",
                    "children": [{"value": "Rich content."}]
                }
            ]
        }"#;
        let fn1: Footnote = serde_json::from_str(json_str).unwrap();
        assert_eq!(fn1.number, 1);
        assert_eq!(fn1.id, Some("fn-1".to_string()));
        assert!(fn1.has_children());
        assert_eq!(fn1.children.len(), 1);
    }

    #[test]
    fn test_footnote_has_content() {
        let fn1 = Footnote::new(1).with_content("Text");
        assert!(fn1.has_content());
        assert!(!fn1.has_children());
    }
}
