//! Signature types and structures.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::DocumentId;

/// Signature file structure.
///
/// This represents the `security/signatures.json` file in a Codex document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureFile {
    /// Format version.
    pub version: String,

    /// Document ID that was signed.
    pub document_id: DocumentId,

    /// Array of signatures.
    pub signatures: Vec<Signature>,
}

impl SignatureFile {
    /// Create a new signature file.
    #[must_use]
    pub fn new(document_id: DocumentId) -> Self {
        Self {
            version: crate::SPEC_VERSION.to_string(),
            document_id,
            signatures: Vec::new(),
        }
    }

    /// Add a signature to the file.
    pub fn add_signature(&mut self, signature: Signature) {
        self.signatures.push(signature);
    }

    /// Check if the file has any signatures.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.signatures.is_empty()
    }

    /// Get the number of signatures.
    #[must_use]
    pub fn len(&self) -> usize {
        self.signatures.len()
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

    /// Find a signature by ID.
    #[must_use]
    pub fn find_signature(&self, id: &str) -> Option<&Signature> {
        self.signatures.iter().find(|s| s.id == id)
    }
}

/// A digital signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Signature {
    /// Unique signature identifier.
    pub id: String,

    /// Signature algorithm.
    pub algorithm: SignatureAlgorithm,

    /// Signing timestamp.
    pub signed_at: DateTime<Utc>,

    /// Signer information.
    pub signer: SignerInfo,

    /// Base64-encoded signature value.
    pub value: String,

    /// Optional certificate chain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificate_chain: Option<Vec<String>>,
}

impl Signature {
    /// Create a new signature.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        algorithm: SignatureAlgorithm,
        signer: SignerInfo,
        value: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            algorithm,
            signed_at: Utc::now(),
            signer,
            value: value.into(),
            certificate_chain: None,
        }
    }
}

/// Signature algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignatureAlgorithm {
    /// ECDSA with P-256 (required).
    ES256,
    /// ECDSA with P-384 (recommended).
    ES384,
    /// Edwards-curve Digital Signature Algorithm (recommended).
    EdDSA,
    /// RSA-PSS with SHA-256 (optional).
    PS256,
    /// ML-DSA-65 post-quantum signature (FIPS-204).
    #[serde(rename = "ML-DSA-65")]
    MlDsa65,
}

impl SignatureAlgorithm {
    /// Get the algorithm identifier string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ES256 => "ES256",
            Self::ES384 => "ES384",
            Self::EdDSA => "EdDSA",
            Self::PS256 => "PS256",
            Self::MlDsa65 => "ML-DSA-65",
        }
    }

    /// Check if this is a post-quantum algorithm.
    #[must_use]
    pub const fn is_post_quantum(&self) -> bool {
        matches!(self, Self::MlDsa65)
    }
}

impl std::fmt::Display for SignatureAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Information about the signer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignerInfo {
    /// Signer's display name.
    pub name: String,

    /// Signer's email address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Signer's organization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,

    /// X.509 certificate (PEM format).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificate: Option<String>,

    /// Key identifier (DID, URL, etc.).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_id: Option<String>,
}

impl SignerInfo {
    /// Create new signer info with just a name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            email: None,
            organization: None,
            certificate: None,
            key_id: None,
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
}

/// Result of signature verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureVerification {
    /// Signature ID.
    pub signature_id: String,

    /// Verification status.
    pub status: VerificationStatus,

    /// Error message if verification failed.
    pub error: Option<String>,
}

impl SignatureVerification {
    /// Create a successful verification result.
    #[must_use]
    pub fn valid(signature_id: impl Into<String>) -> Self {
        Self {
            signature_id: signature_id.into(),
            status: VerificationStatus::Valid,
            error: None,
        }
    }

    /// Create a failed verification result.
    #[must_use]
    pub fn invalid(signature_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            signature_id: signature_id.into(),
            status: VerificationStatus::Invalid,
            error: Some(error.into()),
        }
    }

    /// Check if the verification passed.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.status == VerificationStatus::Valid
    }
}

/// Signature verification status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationStatus {
    /// Signature verifies correctly.
    Valid,
    /// Signature does not verify.
    Invalid,
    /// Certificate has expired.
    Expired,
    /// Certificate has been revoked.
    Revoked,
    /// Certificate chain not trusted.
    Untrusted,
    /// Cannot determine validity.
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HashAlgorithm;

    #[test]
    fn test_signature_file_new() {
        let doc_id = crate::Hasher::hash(HashAlgorithm::Sha256, b"test");
        let file = SignatureFile::new(doc_id);
        assert_eq!(file.version, "0.1");
        assert!(file.is_empty());
    }

    #[test]
    fn test_signature_new() {
        let signer = SignerInfo::new("Test User").with_email("test@example.com");
        let sig = Signature::new("sig-1", SignatureAlgorithm::ES256, signer, "base64value");

        assert_eq!(sig.id, "sig-1");
        assert_eq!(sig.algorithm, SignatureAlgorithm::ES256);
        assert_eq!(sig.value, "base64value");
    }

    #[test]
    fn test_signer_info() {
        let info = SignerInfo::new("Alice")
            .with_email("alice@example.com")
            .with_organization("Acme Corp");

        assert_eq!(info.name, "Alice");
        assert_eq!(info.email, Some("alice@example.com".to_string()));
        assert_eq!(info.organization, Some("Acme Corp".to_string()));
    }

    #[test]
    fn test_serialization() {
        let doc_id = crate::Hasher::hash(HashAlgorithm::Sha256, b"test");
        let mut file = SignatureFile::new(doc_id);

        let signer = SignerInfo::new("Test User");
        let sig = Signature::new("sig-1", SignatureAlgorithm::ES256, signer, "base64value");
        file.add_signature(sig);

        let json = serde_json::to_string_pretty(&file).unwrap();
        assert!(json.contains("\"algorithm\": \"ES256\""));
        assert!(json.contains("\"documentId\":"));
    }

    #[test]
    fn test_verification_result() {
        let valid = SignatureVerification::valid("sig-1");
        assert!(valid.is_valid());

        let invalid = SignatureVerification::invalid("sig-2", "bad signature");
        assert!(!invalid.is_valid());
        assert_eq!(invalid.error, Some("bad signature".to_string()));
    }
}
