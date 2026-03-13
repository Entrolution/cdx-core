//! Provenance record for complete document history tracking.
//!
//! The provenance record (`provenance/record.json`) stores comprehensive
//! provenance information including lineage, timestamps, and derivation history.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::manifest::Lineage;
use crate::{DocumentId, HashAlgorithm};

/// Complete provenance record for a document.
///
/// This structure is stored at `provenance/record.json` and provides:
/// - Document identity and creation information
/// - Full lineage chain
/// - Merkle tree information for content integrity
/// - Timestamp records for temporal anchoring
/// - Derivation records for tracking content sources
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvenanceRecord {
    /// Version of the provenance record format.
    pub version: String,

    /// Document identifier.
    pub document_id: DocumentId,

    /// When the document was created.
    pub created: DateTime<Utc>,

    /// Information about the document creator.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub creator: Option<CreatorInfo>,

    /// Lineage information.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lineage: Option<Lineage>,

    /// Merkle tree information.
    pub merkle: MerkleInfo,

    /// Timestamp records for temporal anchoring.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub timestamps: Vec<TimestampRecord>,

    /// Records of content derived from other sources.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub derived_from: Vec<DerivationRecord>,
}

impl ProvenanceRecord {
    /// Current version of the provenance record format.
    pub const VERSION: &'static str = "0.1";

    /// Create a new provenance record.
    #[must_use]
    pub fn new(document_id: DocumentId, merkle: MerkleInfo) -> Self {
        Self {
            version: Self::VERSION.to_string(),
            document_id,
            created: Utc::now(),
            creator: None,
            lineage: None,
            merkle,
            timestamps: Vec::new(),
            derived_from: Vec::new(),
        }
    }

    /// Set the creator information.
    #[must_use]
    pub fn with_creator(mut self, creator: CreatorInfo) -> Self {
        self.creator = Some(creator);
        self
    }

    /// Set the lineage information.
    #[must_use]
    pub fn with_lineage(mut self, lineage: Lineage) -> Self {
        self.lineage = Some(lineage);
        self
    }

    /// Add a timestamp record.
    #[must_use]
    pub fn with_timestamp(mut self, timestamp: TimestampRecord) -> Self {
        self.timestamps.push(timestamp);
        self
    }

    /// Add a derivation record.
    #[must_use]
    pub fn with_derivation(mut self, derivation: DerivationRecord) -> Self {
        self.derived_from.push(derivation);
        self
    }

    /// Serialize to JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_json(&self) -> crate::Result<String> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }

    /// Deserialize from JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails.
    pub fn from_json(json: &str) -> crate::Result<Self> {
        serde_json::from_str(json).map_err(Into::into)
    }
}

/// Information about the document creator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatorInfo {
    /// Creator's name.
    pub name: String,

    /// Creator's email address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Creator's organization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,

    /// URI identifying the creator.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
}

impl CreatorInfo {
    /// Create new creator info with just a name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            email: None,
            organization: None,
            uri: None,
        }
    }

    /// Set the email address.
    #[must_use]
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Set the organization.
    #[must_use]
    pub fn with_organization(mut self, org: impl Into<String>) -> Self {
        self.organization = Some(org.into());
        self
    }

    /// Set the URI.
    #[must_use]
    pub fn with_uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = Some(uri.into());
        self
    }
}

/// Information about the Merkle tree structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MerkleInfo {
    /// Merkle root hash.
    pub root: DocumentId,

    /// Number of content blocks.
    pub block_count: usize,

    /// Hash algorithm used.
    pub algorithm: HashAlgorithm,
}

impl MerkleInfo {
    /// Create new Merkle info.
    #[must_use]
    pub fn new(root: DocumentId, block_count: usize, algorithm: HashAlgorithm) -> Self {
        Self {
            root,
            block_count,
            algorithm,
        }
    }
}

/// Record of a timestamp anchoring.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimestampRecord {
    /// Timestamp method used.
    pub method: TimestampMethod,

    /// Name or URL of the timestamp authority.
    pub authority: String,

    /// Time recorded by the authority.
    pub time: DateTime<Utc>,

    /// Base64-encoded timestamp token or proof.
    pub token: String,

    /// Transaction ID or reference (for blockchain anchors).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<String>,
}

impl TimestampRecord {
    /// Create a new RFC 3161 timestamp record.
    #[must_use]
    pub fn rfc3161(
        authority: impl Into<String>,
        time: DateTime<Utc>,
        token: impl Into<String>,
    ) -> Self {
        Self {
            method: TimestampMethod::Rfc3161,
            authority: authority.into(),
            time,
            token: token.into(),
            transaction_id: None,
        }
    }

    /// Create a new Bitcoin timestamp record.
    #[must_use]
    pub fn bitcoin(
        time: DateTime<Utc>,
        token: impl Into<String>,
        tx_id: impl Into<String>,
    ) -> Self {
        Self {
            method: TimestampMethod::Bitcoin,
            authority: "Bitcoin Mainnet".to_string(),
            time,
            token: token.into(),
            transaction_id: Some(tx_id.into()),
        }
    }

    /// Create a new Ethereum timestamp record.
    #[must_use]
    pub fn ethereum(
        time: DateTime<Utc>,
        token: impl Into<String>,
        tx_id: impl Into<String>,
    ) -> Self {
        Self {
            method: TimestampMethod::Ethereum,
            authority: "Ethereum Mainnet".to_string(),
            time,
            token: token.into(),
            transaction_id: Some(tx_id.into()),
        }
    }

    /// Create a new `OpenTimestamps` record.
    #[must_use]
    pub fn open_timestamps(time: DateTime<Utc>, token: impl Into<String>) -> Self {
        Self {
            method: TimestampMethod::OpenTimestamps,
            authority: "OpenTimestamps".to_string(),
            time,
            token: token.into(),
            transaction_id: None,
        }
    }

    /// Check whether this timestamp record has a non-empty token.
    ///
    /// This only validates that a token is present — it does **not** verify
    /// that the token corresponds to the given `document_id`. Full verification
    /// requires protocol-specific checks (RFC 3161 / OTS / blockchain).
    #[must_use]
    pub fn matches_document(&self, _document_id: &DocumentId) -> bool {
        // TODO: Implement protocol-specific document ID matching.
        // For now, only check that a token exists.
        !self.token.is_empty()
    }
}

/// Method used for timestamp anchoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
pub enum TimestampMethod {
    /// RFC 3161 Time Stamp Protocol.
    #[strum(serialize = "RFC 3161")]
    Rfc3161,
    /// Bitcoin blockchain anchoring.
    Bitcoin,
    /// Ethereum blockchain anchoring.
    Ethereum,
    /// `OpenTimestamps` protocol.
    OpenTimestamps,
}

/// Record of content derived from another source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DerivationRecord {
    /// Source document or resource identifier.
    pub source: String,

    /// Type of derivation.
    pub derivation_type: DerivationType,

    /// Human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// When the derivation occurred.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,

    /// License under which the source was used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

impl DerivationRecord {
    /// Create a new derivation record.
    #[must_use]
    pub fn new(source: impl Into<String>, derivation_type: DerivationType) -> Self {
        Self {
            source: source.into(),
            derivation_type,
            description: None,
            timestamp: None,
            license: None,
        }
    }

    /// Set a description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the timestamp.
    #[must_use]
    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Set the license.
    #[must_use]
    pub fn with_license(mut self, license: impl Into<String>) -> Self {
        self.license = Some(license.into());
        self
    }
}

/// Type of content derivation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "camelCase")]
pub enum DerivationType {
    /// Direct quotation from source.
    Quotation,
    /// Paraphrased or summarized content.
    Paraphrase,
    /// Content translated from another language.
    Translation,
    /// Content adapted or modified.
    Adaptation,
    /// Content based on or inspired by source.
    #[strum(serialize = "Based On")]
    BasedOn,
    /// Content imported from external source.
    Import,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_hash() -> DocumentId {
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            .parse()
            .unwrap()
    }

    #[test]
    fn test_provenance_record_creation() {
        let merkle = MerkleInfo::new(test_hash(), 10, HashAlgorithm::Sha256);
        let record = ProvenanceRecord::new(test_hash(), merkle);

        assert_eq!(record.version, "0.1");
        assert_eq!(record.merkle.block_count, 10);
        assert!(record.timestamps.is_empty());
    }

    #[test]
    fn test_provenance_record_with_creator() {
        let merkle = MerkleInfo::new(test_hash(), 5, HashAlgorithm::Sha256);
        let creator = CreatorInfo::new("Jane Doe")
            .with_email("jane@example.com")
            .with_organization("Acme Corp");

        let record = ProvenanceRecord::new(test_hash(), merkle).with_creator(creator);

        assert!(record.creator.is_some());
        assert_eq!(record.creator.as_ref().unwrap().name, "Jane Doe");
    }

    #[test]
    fn test_timestamp_record_rfc3161() {
        let timestamp =
            TimestampRecord::rfc3161("https://timestamp.example.com", Utc::now(), "base64token");

        assert_eq!(timestamp.method, TimestampMethod::Rfc3161);
        assert_eq!(timestamp.authority, "https://timestamp.example.com");
    }

    #[test]
    fn test_timestamp_record_bitcoin() {
        let timestamp = TimestampRecord::bitcoin(Utc::now(), "opreturn_data", "abc123def456");

        assert_eq!(timestamp.method, TimestampMethod::Bitcoin);
        assert!(timestamp.transaction_id.is_some());
    }

    #[test]
    fn test_derivation_record() {
        let derivation =
            DerivationRecord::new("https://example.com/source", DerivationType::Quotation)
                .with_description("Quote from chapter 3")
                .with_license("CC-BY-4.0");

        assert_eq!(derivation.derivation_type, DerivationType::Quotation);
        assert!(derivation.description.is_some());
    }

    #[test]
    fn test_provenance_record_serialization() {
        let merkle = MerkleInfo::new(test_hash(), 3, HashAlgorithm::Sha256);
        let record = ProvenanceRecord::new(test_hash(), merkle);

        let json = record.to_json().unwrap();
        assert!(json.contains("\"version\": \"0.1\""));
        assert!(json.contains("\"blockCount\": 3"));

        let deserialized = ProvenanceRecord::from_json(&json).unwrap();
        assert_eq!(deserialized.merkle.block_count, 3);
    }

    #[test]
    fn test_timestamp_method_display() {
        assert_eq!(TimestampMethod::Rfc3161.to_string(), "RFC 3161");
        assert_eq!(TimestampMethod::Bitcoin.to_string(), "Bitcoin");
    }

    #[test]
    fn test_derivation_type_display() {
        assert_eq!(DerivationType::Quotation.to_string(), "Quotation");
        assert_eq!(DerivationType::Translation.to_string(), "Translation");
    }
}
