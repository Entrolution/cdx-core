//! Legal extension for Codex documents.
//!
//! This extension provides specialized content types for legal documents
//! including citations, tables of authorities, and court captions.
//!
//! # Features
//!
//! - **`TableOfAuthorities`**: Auto-generated citation index
//! - **`Caption`**: Court caption block
//! - **`SignatureBlock`**: Legal signature block format
//! - **`LegalCitation`**: Legal citation marks (Bluebook, ALWD, etc.)
//!
//! # Example
//!
//! ```json
//! {
//!   "type": "legal:cite",
//!   "citation": "Brown v. Board of Education",
//!   "cite": "347 U.S. 483",
//!   "year": 1954,
//!   "category": "case"
//! }
//! ```

use serde::{Deserialize, Serialize};

// ============================================================================
// Table of Authorities
// ============================================================================

/// A table of authorities listing all legal citations in the document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableOfAuthorities {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Title for the table (defaults to "TABLE OF AUTHORITIES").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Citation categories to include.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<CitationCategory>,

    /// Whether to generate automatically from document citations.
    #[serde(default = "default_true")]
    pub auto_generate: bool,

    /// Citation format to use.
    #[serde(default)]
    pub format: LegalCitationFormat,
}

fn default_true() -> bool {
    true
}

impl TableOfAuthorities {
    /// Create a new table of authorities.
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: None,
            title: None,
            categories: Vec::new(),
            auto_generate: true,
            format: LegalCitationFormat::default(),
        }
    }

    /// Set the title.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Add a category.
    #[must_use]
    pub fn with_category(mut self, category: CitationCategory) -> Self {
        self.categories.push(category);
        self
    }

    /// Set the citation format.
    #[must_use]
    pub fn with_format(mut self, format: LegalCitationFormat) -> Self {
        self.format = format;
        self
    }
}

impl Default for TableOfAuthorities {
    fn default() -> Self {
        Self::new()
    }
}

/// A category in the table of authorities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CitationCategory {
    /// Category type.
    pub category_type: LegalCitationType,

    /// Custom heading (overrides default).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heading: Option<String>,

    /// Entries in this category (populated during generation).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entries: Vec<TableOfAuthoritiesEntry>,
}

/// An entry in the table of authorities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableOfAuthoritiesEntry {
    /// Full citation text.
    pub citation: String,

    /// Page numbers where cited.
    pub pages: Vec<String>,

    /// Whether this is a primary authority.
    #[serde(default)]
    pub primary: bool,
}

// ============================================================================
// Court Caption
// ============================================================================

/// A court caption block for legal documents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Caption {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Court name.
    pub court: String,

    /// Case number/docket number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub case_number: Option<String>,

    /// Docket identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docket: Option<String>,

    /// Plaintiffs/appellants.
    pub plaintiffs: Vec<Party>,

    /// Defendants/appellees.
    pub defendants: Vec<Party>,

    /// Case title override (if different from generated).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Document type (Brief, Motion, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document_type: Option<String>,

    /// Judge name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub judge: Option<String>,

    /// Caption style.
    #[serde(default)]
    pub style: CaptionStyle,
}

impl Caption {
    /// Create a new caption.
    #[must_use]
    pub fn new(court: impl Into<String>) -> Self {
        Self {
            id: None,
            court: court.into(),
            case_number: None,
            docket: None,
            plaintiffs: Vec::new(),
            defendants: Vec::new(),
            title: None,
            document_type: None,
            judge: None,
            style: CaptionStyle::default(),
        }
    }

    /// Set the case number.
    #[must_use]
    pub fn with_case_number(mut self, case_number: impl Into<String>) -> Self {
        self.case_number = Some(case_number.into());
        self
    }

    /// Add a plaintiff.
    #[must_use]
    pub fn with_plaintiff(mut self, party: Party) -> Self {
        self.plaintiffs.push(party);
        self
    }

    /// Add a defendant.
    #[must_use]
    pub fn with_defendant(mut self, party: Party) -> Self {
        self.defendants.push(party);
        self
    }

    /// Set the document type.
    #[must_use]
    pub fn with_document_type(mut self, doc_type: impl Into<String>) -> Self {
        self.document_type = Some(doc_type.into());
        self
    }
}

/// A party in a legal case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Party {
    /// Party name.
    pub name: String,

    /// Party role (e.g., "Appellant", "Defendant-Appellee").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// Whether this is the primary party (for "et al." shortening).
    #[serde(default)]
    pub primary: bool,
}

impl Party {
    /// Create a new party.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            role: None,
            primary: false,
        }
    }

    /// Set the role.
    #[must_use]
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }

    /// Mark as primary party.
    #[must_use]
    pub fn primary(mut self) -> Self {
        self.primary = true;
        self
    }
}

/// Caption style format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CaptionStyle {
    /// Standard federal style.
    #[default]
    Federal,
    /// California style.
    California,
    /// New York style.
    NewYork,
    /// Texas style.
    Texas,
}

// ============================================================================
// Legal Signature Block
// ============================================================================

/// A legal signature block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegalSignatureBlock {
    /// Optional unique identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Signatory information.
    pub signatory: Signatory,

    /// Firm/organization information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firm: Option<FirmInfo>,

    /// Date signed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,

    /// Certificate of service included.
    #[serde(default)]
    pub certificate_of_service: bool,
}

impl LegalSignatureBlock {
    /// Create a new legal signature block.
    #[must_use]
    pub fn new(signatory: Signatory) -> Self {
        Self {
            id: None,
            signatory,
            firm: None,
            date: None,
            certificate_of_service: false,
        }
    }

    /// Set the firm.
    #[must_use]
    pub fn with_firm(mut self, firm: FirmInfo) -> Self {
        self.firm = Some(firm);
        self
    }

    /// Set the date.
    #[must_use]
    pub fn with_date(mut self, date: impl Into<String>) -> Self {
        self.date = Some(date.into());
        self
    }

    /// Include certificate of service.
    #[must_use]
    pub fn with_certificate_of_service(mut self) -> Self {
        self.certificate_of_service = true;
        self
    }
}

/// Information about the signatory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Signatory {
    /// Name.
    pub name: String,

    /// Bar number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bar_number: Option<String>,

    /// State(s) admitted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub states_admitted: Vec<String>,

    /// Title/position.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Email address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Phone number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
}

impl Signatory {
    /// Create a new signatory.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            bar_number: None,
            states_admitted: Vec::new(),
            title: None,
            email: None,
            phone: None,
        }
    }

    /// Set bar number.
    #[must_use]
    pub fn with_bar_number(mut self, bar_number: impl Into<String>) -> Self {
        self.bar_number = Some(bar_number.into());
        self
    }

    /// Add state admitted.
    #[must_use]
    pub fn admitted_in(mut self, state: impl Into<String>) -> Self {
        self.states_admitted.push(state.into());
        self
    }

    /// Set title.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set email.
    #[must_use]
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }
}

/// Law firm information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirmInfo {
    /// Firm name.
    pub name: String,

    /// Address lines.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub address: Vec<String>,

    /// Phone number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,

    /// Fax number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fax: Option<String>,
}

impl FirmInfo {
    /// Create new firm info.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            address: Vec::new(),
            phone: None,
            fax: None,
        }
    }

    /// Add address line.
    #[must_use]
    pub fn with_address_line(mut self, line: impl Into<String>) -> Self {
        self.address.push(line.into());
        self
    }

    /// Set phone.
    #[must_use]
    pub fn with_phone(mut self, phone: impl Into<String>) -> Self {
        self.phone = Some(phone.into());
        self
    }
}

// ============================================================================
// Legal Citations
// ============================================================================

/// A legal citation (for inline marks).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegalCitation {
    /// Full citation text.
    pub citation: String,

    /// Short form for subsequent references.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_form: Option<String>,

    /// Reporter citation (e.g., "347 U.S. 483").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cite: Option<String>,

    /// Year of decision.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub year: Option<u16>,

    /// Citation category.
    pub category: LegalCitationType,

    /// Pinpoint reference (page, paragraph, section).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pinpoint: Option<Pinpoint>,

    /// Parenthetical explanation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parenthetical: Option<String>,

    /// Signal (e.g., "See", "Cf.", "But see").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signal: Option<CitationSignal>,

    /// Whether this is the first reference.
    #[serde(default = "default_true")]
    pub first_reference: bool,
}

impl LegalCitation {
    /// Create a new case citation.
    #[must_use]
    pub fn case(citation: impl Into<String>, cite: impl Into<String>, year: u16) -> Self {
        Self {
            citation: citation.into(),
            short_form: None,
            cite: Some(cite.into()),
            year: Some(year),
            category: LegalCitationType::Case,
            pinpoint: None,
            parenthetical: None,
            signal: None,
            first_reference: true,
        }
    }

    /// Create a new statute citation.
    #[must_use]
    pub fn statute(citation: impl Into<String>) -> Self {
        Self {
            citation: citation.into(),
            short_form: None,
            cite: None,
            year: None,
            category: LegalCitationType::Statute,
            pinpoint: None,
            parenthetical: None,
            signal: None,
            first_reference: true,
        }
    }

    /// Set short form.
    #[must_use]
    pub fn with_short_form(mut self, short_form: impl Into<String>) -> Self {
        self.short_form = Some(short_form.into());
        self
    }

    /// Set pinpoint reference.
    #[must_use]
    pub fn at(mut self, pinpoint: Pinpoint) -> Self {
        self.pinpoint = Some(pinpoint);
        self
    }

    /// Set parenthetical.
    #[must_use]
    pub fn with_parenthetical(mut self, text: impl Into<String>) -> Self {
        self.parenthetical = Some(text.into());
        self
    }

    /// Set signal.
    #[must_use]
    pub fn with_signal(mut self, signal: CitationSignal) -> Self {
        self.signal = Some(signal);
        self
    }

    /// Mark as subsequent reference (not first).
    #[must_use]
    pub fn subsequent(mut self) -> Self {
        self.first_reference = false;
        self
    }
}

/// Type of legal citation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
pub enum LegalCitationType {
    /// Court case.
    #[strum(serialize = "Cases")]
    Case,
    /// Statute or legislation.
    #[strum(serialize = "Statutes")]
    Statute,
    /// Regulation.
    #[strum(serialize = "Regulations")]
    Regulation,
    /// Constitutional provision.
    #[strum(serialize = "Constitutional Provisions")]
    Constitution,
    /// Secondary source (treatise, law review, etc.).
    #[strum(serialize = "Secondary Sources")]
    Secondary,
    /// Book or treatise.
    #[strum(serialize = "Books")]
    Book,
    /// Law review article.
    #[strum(serialize = "Law Review Articles")]
    LawReview,
    /// Legislative history.
    #[strum(serialize = "Legislative History")]
    Legislative,
    /// Other authority.
    #[strum(serialize = "Other Authorities")]
    Other,
}

/// Pinpoint reference within a citation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pinpoint {
    /// Type of pinpoint.
    pub pinpoint_type: PinpointType,

    /// Value (page number, section, etc.).
    pub value: String,
}

impl Pinpoint {
    /// Page pinpoint.
    #[must_use]
    pub fn page(page: impl Into<String>) -> Self {
        Self {
            pinpoint_type: PinpointType::Page,
            value: page.into(),
        }
    }

    /// Section pinpoint.
    #[must_use]
    pub fn section(section: impl Into<String>) -> Self {
        Self {
            pinpoint_type: PinpointType::Section,
            value: section.into(),
        }
    }

    /// Paragraph pinpoint.
    #[must_use]
    pub fn paragraph(para: impl Into<String>) -> Self {
        Self {
            pinpoint_type: PinpointType::Paragraph,
            value: para.into(),
        }
    }
}

/// Type of pinpoint reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PinpointType {
    /// Page number.
    Page,
    /// Section.
    Section,
    /// Paragraph.
    Paragraph,
    /// Footnote.
    Footnote,
    /// Clause.
    Clause,
    /// Article.
    Article,
}

/// Citation signal indicating how authority supports proposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
pub enum CitationSignal {
    /// No signal (direct support).
    #[strum(serialize = "")]
    None,
    /// E.g.,
    #[strum(serialize = "E.g.")]
    Eg,
    /// Accord
    Accord,
    /// See
    See,
    /// See also
    #[strum(serialize = "See also")]
    SeeAlso,
    /// Cf.
    #[strum(serialize = "Cf.")]
    Cf,
    /// Compare
    Compare,
    /// Contra
    Contra,
    /// But see
    #[strum(serialize = "But see")]
    ButSee,
    /// But cf.
    #[strum(serialize = "But cf.")]
    ButCf,
    /// See generally
    #[strum(serialize = "See generally")]
    SeeGenerally,
}

/// Legal citation format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
pub enum LegalCitationFormat {
    /// Bluebook format (US).
    #[default]
    Bluebook,
    /// ALWD Citation Manual.
    #[strum(serialize = "ALWD")]
    Alwd,
    /// `McGill` Guide (Canada).
    McGill,
    /// OSCOLA (UK).
    #[strum(serialize = "OSCOLA")]
    Oscola,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caption_new() {
        let caption = Caption::new("United States District Court, Southern District of New York");
        assert_eq!(
            caption.court,
            "United States District Court, Southern District of New York"
        );
        assert!(caption.plaintiffs.is_empty());
        assert!(caption.defendants.is_empty());
    }

    #[test]
    fn test_caption_builder() {
        let caption = Caption::new("Supreme Court of the United States")
            .with_case_number("No. 21-1234")
            .with_plaintiff(Party::new("John Doe").primary())
            .with_defendant(Party::new("Acme Corp"))
            .with_document_type("Brief for Petitioner");

        assert_eq!(caption.case_number, Some("No. 21-1234".to_string()));
        assert_eq!(caption.plaintiffs.len(), 1);
        assert!(caption.plaintiffs[0].primary);
        assert_eq!(caption.defendants.len(), 1);
    }

    #[test]
    fn test_legal_citation_case() {
        let cite = LegalCitation::case("Brown v. Board of Education", "347 U.S. 483", 1954);

        assert_eq!(cite.category, LegalCitationType::Case);
        assert_eq!(cite.year, Some(1954));
        assert_eq!(cite.cite, Some("347 U.S. 483".to_string()));
    }

    #[test]
    fn test_legal_citation_builder() {
        let cite = LegalCitation::case("Miranda v. Arizona", "384 U.S. 436", 1966)
            .with_short_form("Miranda")
            .at(Pinpoint::page("444"))
            .with_parenthetical("establishing Miranda warnings");

        assert_eq!(cite.short_form, Some("Miranda".to_string()));
        assert!(cite.pinpoint.is_some());
        assert!(cite.parenthetical.is_some());
    }

    #[test]
    fn test_signatory() {
        let sig = Signatory::new("Jane Doe")
            .with_bar_number("123456")
            .admitted_in("New York")
            .admitted_in("California")
            .with_email("jane.doe@lawfirm.com");

        assert_eq!(sig.bar_number, Some("123456".to_string()));
        assert_eq!(sig.states_admitted.len(), 2);
    }

    #[test]
    fn test_table_of_authorities() {
        let toa = TableOfAuthorities::new()
            .with_title("TABLE OF AUTHORITIES")
            .with_format(LegalCitationFormat::Bluebook);

        assert!(toa.auto_generate);
        assert_eq!(toa.format, LegalCitationFormat::Bluebook);
    }

    #[test]
    fn test_citation_serialization() {
        let cite = LegalCitation::statute("42 U.S.C. § 1983").at(Pinpoint::section("1983"));

        let json = serde_json::to_string(&cite).unwrap();
        assert!(json.contains("\"category\":\"statute\""));
        assert!(json.contains("\"citation\":\"42 U.S.C. § 1983\""));
    }

    #[test]
    fn test_citation_type_display() {
        assert_eq!(LegalCitationType::Case.to_string(), "Cases");
        assert_eq!(LegalCitationType::Statute.to_string(), "Statutes");
        assert_eq!(
            LegalCitationType::Constitution.to_string(),
            "Constitutional Provisions"
        );
    }

    #[test]
    fn test_signal_display() {
        assert_eq!(CitationSignal::See.to_string(), "See");
        assert_eq!(CitationSignal::ButSee.to_string(), "But see");
        assert_eq!(CitationSignal::None.to_string(), "");
    }

    #[test]
    fn test_caption_docket_roundtrip() {
        let caption = Caption::new("District Court").with_case_number("No. 24-1234");
        // Manually set docket
        let mut caption = caption;
        caption.docket = Some("DKT-2024-5678".to_string());

        let json = serde_json::to_string(&caption).unwrap();
        assert!(json.contains("\"docket\":\"DKT-2024-5678\""));

        let parsed: Caption = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.docket, Some("DKT-2024-5678".to_string()));
    }

    #[test]
    fn test_caption_without_docket_defaults_to_none() {
        let json = r#"{
            "court": "Supreme Court",
            "plaintiffs": [],
            "defendants": [],
            "style": "federal"
        }"#;
        let caption: Caption = serde_json::from_str(json).unwrap();
        assert!(caption.docket.is_none());
    }
}
