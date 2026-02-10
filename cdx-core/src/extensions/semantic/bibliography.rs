//! Bibliography management for academic documents.

use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
pub enum CitationStyle {
    /// APA (American Psychological Association) style.
    #[default]
    #[strum(serialize = "APA")]
    Apa,
    /// MLA (Modern Language Association) style.
    #[strum(serialize = "MLA")]
    Mla,
    /// Chicago Manual of Style.
    Chicago,
    /// IEEE style.
    #[strum(serialize = "IEEE")]
    Ieee,
    /// Harvard style.
    Harvard,
    /// Vancouver style.
    Vancouver,
    /// ACM style.
    #[strum(serialize = "ACM")]
    Acm,
    /// Custom style (implementation-defined).
    Custom,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "lowercase")]
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
    #[strum(serialize = "legal-case")]
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
