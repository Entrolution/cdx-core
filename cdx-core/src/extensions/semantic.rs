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

use serde::{Deserialize, Serialize};

// ============================================================================
// Bibliography
// ============================================================================

/// A bibliography containing all references cited in a document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bibliography {
    /// Citation style used for formatting.
    #[serde(default)]
    pub style: CitationStyle,

    /// Bibliography entries.
    pub entries: Vec<BibliographyEntry>,
}

impl Bibliography {
    /// Create a new empty bibliography.
    #[must_use]
    pub fn new(style: CitationStyle) -> Self {
        Self {
            style,
            entries: Vec::new(),
        }
    }

    /// Add an entry to the bibliography.
    pub fn add_entry(&mut self, entry: BibliographyEntry) {
        self.entries.push(entry);
    }

    /// Find an entry by its ID.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&BibliographyEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Check if the bibliography contains an entry with the given ID.
    #[must_use]
    pub fn contains(&self, id: &str) -> bool {
        self.entries.iter().any(|e| e.id == id)
    }

    /// Get the number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the bibliography is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for Bibliography {
    fn default() -> Self {
        Self::new(CitationStyle::default())
    }
}

/// Citation style for formatting references.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CitationStyle {
    /// APA (American Psychological Association) style.
    #[default]
    Apa,
    /// MLA (Modern Language Association) style.
    Mla,
    /// Chicago Manual of Style.
    Chicago,
    /// IEEE style.
    Ieee,
    /// Harvard style.
    Harvard,
    /// Vancouver style.
    Vancouver,
    /// ACM style.
    Acm,
    /// Custom style (implementation-defined).
    Custom,
}

impl std::fmt::Display for CitationStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Apa => write!(f, "APA"),
            Self::Mla => write!(f, "MLA"),
            Self::Chicago => write!(f, "Chicago"),
            Self::Ieee => write!(f, "IEEE"),
            Self::Harvard => write!(f, "Harvard"),
            Self::Vancouver => write!(f, "Vancouver"),
            Self::Acm => write!(f, "ACM"),
            Self::Custom => write!(f, "Custom"),
        }
    }
}

/// A bibliography entry representing a single reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BibliographyEntry {
    /// Unique identifier for the entry (used in citations).
    pub id: String,

    /// Type of the entry.
    pub entry_type: EntryType,

    /// Title of the work.
    pub title: String,

    /// Authors of the work.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<Author>,

    /// Publication date.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issued: Option<PartialDate>,

    /// Container title (e.g., journal name, book title for chapters).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container_title: Option<String>,

    /// Volume number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,

    /// Issue number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue: Option<String>,

    /// Page range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,

    /// Digital Object Identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,

    /// URL to the work.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// ISBN for books.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub isbn: Option<String>,

    /// ISSN for journals.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issn: Option<String>,

    /// Publisher name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,

    /// Publication location.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publisher_place: Option<String>,

    /// Edition number or description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,

    /// Editors (for edited volumes).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub editors: Vec<Author>,

    /// Abstract text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abstract_text: Option<String>,

    /// Keywords or tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,

    /// Language of the work.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Access date for online resources.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accessed: Option<PartialDate>,

    /// Additional notes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl BibliographyEntry {
    /// Create a new bibliography entry.
    #[must_use]
    pub fn new(id: impl Into<String>, entry_type: EntryType, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            entry_type,
            title: title.into(),
            authors: Vec::new(),
            issued: None,
            container_title: None,
            volume: None,
            issue: None,
            page: None,
            doi: None,
            url: None,
            isbn: None,
            issn: None,
            publisher: None,
            publisher_place: None,
            edition: None,
            editors: Vec::new(),
            abstract_text: None,
            keywords: Vec::new(),
            language: None,
            accessed: None,
            note: None,
        }
    }

    /// Add an author.
    #[must_use]
    pub fn with_author(mut self, author: Author) -> Self {
        self.authors.push(author);
        self
    }

    /// Add multiple authors.
    #[must_use]
    pub fn with_authors(mut self, authors: Vec<Author>) -> Self {
        self.authors = authors;
        self
    }

    /// Set the publication date.
    #[must_use]
    pub fn with_issued(mut self, date: PartialDate) -> Self {
        self.issued = Some(date);
        self
    }

    /// Set the container title.
    #[must_use]
    pub fn with_container(mut self, container: impl Into<String>) -> Self {
        self.container_title = Some(container.into());
        self
    }

    /// Set volume and issue.
    #[must_use]
    pub fn with_volume_issue(mut self, volume: impl Into<String>, issue: Option<String>) -> Self {
        self.volume = Some(volume.into());
        self.issue = issue;
        self
    }

    /// Set page range.
    #[must_use]
    pub fn with_pages(mut self, pages: impl Into<String>) -> Self {
        self.page = Some(pages.into());
        self
    }

    /// Set DOI.
    #[must_use]
    pub fn with_doi(mut self, doi: impl Into<String>) -> Self {
        self.doi = Some(doi.into());
        self
    }

    /// Set URL.
    #[must_use]
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Set publisher information.
    #[must_use]
    pub fn with_publisher(mut self, publisher: impl Into<String>, place: Option<String>) -> Self {
        self.publisher = Some(publisher.into());
        self.publisher_place = place;
        self
    }
}

/// Type of bibliography entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EntryType {
    /// Journal article.
    Article,
    /// Book.
    Book,
    /// Chapter in a book.
    Chapter,
    /// Conference paper.
    Conference,
    /// Thesis or dissertation.
    Thesis,
    /// Technical report.
    Report,
    /// Website or webpage.
    Webpage,
    /// Patent.
    Patent,
    /// Dataset.
    Dataset,
    /// Software.
    Software,
    /// Legal case.
    LegalCase,
    /// Legislation or statute.
    Legislation,
    /// Personal communication.
    Personal,
    /// Manuscript.
    Manuscript,
    /// Other type.
    Other,
}

impl std::fmt::Display for EntryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Article => write!(f, "article"),
            Self::Book => write!(f, "book"),
            Self::Chapter => write!(f, "chapter"),
            Self::Conference => write!(f, "conference"),
            Self::Thesis => write!(f, "thesis"),
            Self::Report => write!(f, "report"),
            Self::Webpage => write!(f, "webpage"),
            Self::Patent => write!(f, "patent"),
            Self::Dataset => write!(f, "dataset"),
            Self::Software => write!(f, "software"),
            Self::LegalCase => write!(f, "legal-case"),
            Self::Legislation => write!(f, "legislation"),
            Self::Personal => write!(f, "personal"),
            Self::Manuscript => write!(f, "manuscript"),
            Self::Other => write!(f, "other"),
        }
    }
}

/// An author or contributor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    /// Given name (first name).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub given: Option<String>,

    /// Family name (last name).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,

    /// Full literal name (for non-standard names or organizations).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub literal: Option<String>,

    /// ORCID identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orcid: Option<String>,

    /// Affiliation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub affiliation: Option<String>,
}

impl Author {
    /// Create an author from given and family names.
    #[must_use]
    pub fn new(given: impl Into<String>, family: impl Into<String>) -> Self {
        Self {
            given: Some(given.into()),
            family: Some(family.into()),
            literal: None,
            orcid: None,
            affiliation: None,
        }
    }

    /// Create an author from a literal name (e.g., organization).
    #[must_use]
    pub fn literal(name: impl Into<String>) -> Self {
        Self {
            given: None,
            family: None,
            literal: Some(name.into()),
            orcid: None,
            affiliation: None,
        }
    }

    /// Set ORCID.
    #[must_use]
    pub fn with_orcid(mut self, orcid: impl Into<String>) -> Self {
        self.orcid = Some(orcid.into());
        self
    }

    /// Set affiliation.
    #[must_use]
    pub fn with_affiliation(mut self, affiliation: impl Into<String>) -> Self {
        self.affiliation = Some(affiliation.into());
        self
    }

    /// Get the display name.
    #[must_use]
    pub fn display_name(&self) -> String {
        if let Some(literal) = &self.literal {
            return literal.clone();
        }
        match (&self.family, &self.given) {
            (Some(family), Some(given)) => format!("{family}, {given}"),
            (Some(family), None) => family.clone(),
            (None, Some(given)) => given.clone(),
            (None, None) => String::new(),
        }
    }
}

/// A partial date (year, year-month, or full date).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartialDate {
    /// Year.
    pub year: i32,

    /// Month (1-12).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub month: Option<u8>,

    /// Day (1-31).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub day: Option<u8>,

    /// Season (for quarterly publications).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub season: Option<String>,
}

impl PartialDate {
    /// Create a year-only date.
    #[must_use]
    pub const fn year(year: i32) -> Self {
        Self {
            year,
            month: None,
            day: None,
            season: None,
        }
    }

    /// Create a year-month date.
    #[must_use]
    pub const fn year_month(year: i32, month: u8) -> Self {
        Self {
            year,
            month: Some(month),
            day: None,
            season: None,
        }
    }

    /// Create a full date.
    #[must_use]
    pub const fn full(year: i32, month: u8, day: u8) -> Self {
        Self {
            year,
            month: Some(month),
            day: Some(day),
            season: None,
        }
    }

    /// Create a seasonal date.
    #[must_use]
    pub fn seasonal(year: i32, season: impl Into<String>) -> Self {
        Self {
            year,
            month: None,
            day: None,
            season: Some(season.into()),
        }
    }
}

impl std::fmt::Display for PartialDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(season) = &self.season {
            return write!(f, "{} {}", season, self.year);
        }
        match (self.month, self.day) {
            (Some(month), Some(day)) => write!(f, "{}-{:02}-{:02}", self.year, month, day),
            (Some(month), None) => write!(f, "{}-{:02}", self.year, month),
            _ => write!(f, "{}", self.year),
        }
    }
}

// ============================================================================
// Citations
// ============================================================================

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

// ============================================================================
// Glossary
// ============================================================================

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

// ============================================================================
// Entity Linking
// ============================================================================

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Person => write!(f, "person"),
            Self::Organization => write!(f, "organization"),
            Self::Place => write!(f, "place"),
            Self::Event => write!(f, "event"),
            Self::Product => write!(f, "product"),
            Self::CreativeWork => write!(f, "creative-work"),
            Self::Concept => write!(f, "concept"),
            Self::Scientific => write!(f, "scientific"),
            Self::TimePeriod => write!(f, "time-period"),
            Self::Other => write!(f, "other"),
        }
    }
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

// ============================================================================
// JSON-LD Metadata
// ============================================================================

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
}
