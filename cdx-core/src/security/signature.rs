//! Signature types and structures.

use std::collections::HashMap;

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
    ///
    /// For standard signatures, this contains the signature bytes.
    /// For WebAuthn signatures, this may be empty if `webauthn` is present.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub value: String,

    /// Optional certificate chain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificate_chain: Option<Vec<String>>,

    /// Optional signature scope for layout attestation.
    ///
    /// When present, the signature covers the scope object (serialized with JCS)
    /// instead of just the document ID. This allows signatures to attest to
    /// specific layout renditions in addition to the semantic content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<SignatureScope>,

    /// Optional RFC 3161 trusted timestamp token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<TrustedTimestamp>,

    /// Optional WebAuthn/FIDO2 signature data.
    ///
    /// When present, this contains the full WebAuthn assertion data
    /// instead of using the `value` field. The document ID is used
    /// as the challenge for WebAuthn verification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webauthn: Option<WebAuthnSignature>,
}

/// WebAuthn/FIDO2 signature data.
///
/// Contains the assertion response from a WebAuthn authenticator.
/// All fields are base64-encoded as per the Codex specification.
///
/// # Verification
///
/// To verify a WebAuthn signature:
/// 1. Decode all base64 fields
/// 2. Parse `client_data_json` and verify the challenge matches the document ID
/// 3. Verify the authenticator data flags and counter
/// 4. Verify the signature over authenticator data + SHA-256(client data JSON)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebAuthnSignature {
    /// Base64-encoded credential ID.
    ///
    /// Identifies the credential used for signing.
    pub credential_id: String,

    /// Base64-encoded authenticator data.
    ///
    /// Contains the RP ID hash, flags, and signature counter.
    pub authenticator_data: String,

    /// Base64-encoded client data JSON.
    ///
    /// Contains the challenge (document ID), origin, and type.
    pub client_data_json: String,

    /// Base64-encoded signature.
    ///
    /// The cryptographic signature over the authenticator data
    /// concatenated with the SHA-256 hash of the client data JSON.
    pub signature: String,
}

impl WebAuthnSignature {
    /// Create a new WebAuthn signature.
    #[must_use]
    pub fn new(
        credential_id: impl Into<String>,
        authenticator_data: impl Into<String>,
        client_data_json: impl Into<String>,
        signature: impl Into<String>,
    ) -> Self {
        Self {
            credential_id: credential_id.into(),
            authenticator_data: authenticator_data.into(),
            client_data_json: client_data_json.into(),
            signature: signature.into(),
        }
    }
}

/// RFC 3161 trusted timestamp token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrustedTimestamp {
    /// Base64-encoded RFC 3161 timestamp token.
    pub token: String,

    /// TSA (Time Stamping Authority) URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tsa: Option<String>,
}

/// Signature scope for scoped signatures.
///
/// Scoped signatures allow attesting to specific layout renditions
/// in addition to the semantic content. The scope is serialized using
/// JSON Canonicalization Scheme (JCS, RFC 8785) before signing.
///
/// # Example
///
/// ```rust
/// use cdx_core::security::SignatureScope;
/// use cdx_core::{HashAlgorithm, Hasher};
/// use std::collections::HashMap;
///
/// let doc_id = Hasher::hash(HashAlgorithm::Sha256, b"content");
/// let layout_hash = Hasher::hash(HashAlgorithm::Sha256, b"layout");
///
/// let mut layouts = HashMap::new();
/// layouts.insert("presentation/print.json".to_string(), layout_hash);
///
/// let scope = SignatureScope::new(doc_id).with_layouts(layouts);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureScope {
    /// Document ID that must match the top-level document ID.
    pub document_id: DocumentId,

    /// Layout paths and their content hashes for layout attestation.
    ///
    /// Keys are presentation file paths (e.g., "presentation/print.json"),
    /// values are the content hashes of those files.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layouts: Option<HashMap<String, DocumentId>>,
}

impl SignatureScope {
    /// Create a new signature scope for a document.
    #[must_use]
    pub fn new(document_id: DocumentId) -> Self {
        Self {
            document_id,
            layouts: None,
        }
    }

    /// Add layout attestations.
    #[must_use]
    pub fn with_layouts(mut self, layouts: HashMap<String, DocumentId>) -> Self {
        self.layouts = Some(layouts);
        self
    }

    /// Add a single layout attestation.
    #[must_use]
    pub fn with_layout(mut self, path: impl Into<String>, hash: DocumentId) -> Self {
        self.layouts
            .get_or_insert_with(HashMap::new)
            .insert(path.into(), hash);
        self
    }

    /// Check if this scope attests to any layouts.
    #[must_use]
    pub fn has_layouts(&self) -> bool {
        self.layouts.as_ref().is_some_and(|l| !l.is_empty())
    }

    /// Serialize the scope using JSON Canonicalization Scheme (JCS).
    ///
    /// This produces a deterministic JSON representation suitable for signing.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    pub fn to_jcs(&self) -> crate::Result<Vec<u8>> {
        use crate::error::invalid_manifest;

        // Use json-canon for JCS serialization
        let value = serde_json::to_value(self)?;
        let canonical = json_canon::to_string(&value)
            .map_err(|e| invalid_manifest(format!("JCS serialization failed: {e}")))?;
        Ok(canonical.into_bytes())
    }
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
            scope: None,
            timestamp: None,
            webauthn: None,
        }
    }

    /// Create a new WebAuthn signature.
    ///
    /// WebAuthn signatures use ES256 algorithm and store the full
    /// assertion data rather than a simple signature value.
    #[must_use]
    pub fn new_webauthn(
        id: impl Into<String>,
        signer: SignerInfo,
        webauthn: WebAuthnSignature,
    ) -> Self {
        Self {
            id: id.into(),
            algorithm: SignatureAlgorithm::ES256,
            signed_at: Utc::now(),
            signer,
            value: String::new(),
            certificate_chain: None,
            scope: None,
            timestamp: None,
            webauthn: Some(webauthn),
        }
    }

    /// Set the signature scope for layout attestation.
    #[must_use]
    pub fn with_scope(mut self, scope: SignatureScope) -> Self {
        self.scope = Some(scope);
        self
    }

    /// Set the trusted timestamp for this signature.
    #[must_use]
    pub fn with_timestamp(mut self, timestamp: TrustedTimestamp) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Check if this signature has a scope (scoped signature).
    #[must_use]
    pub fn is_scoped(&self) -> bool {
        self.scope.is_some()
    }

    /// Check if this is a WebAuthn signature.
    #[must_use]
    pub fn is_webauthn(&self) -> bool {
        self.webauthn.is_some()
    }

    /// Get the WebAuthn signature data, if present.
    #[must_use]
    pub fn webauthn_data(&self) -> Option<&WebAuthnSignature> {
        self.webauthn.as_ref()
    }
}

/// Signature algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display)]
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
    #[strum(serialize = "ML-DSA-65")]
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

    #[test]
    fn test_signature_scope_new() {
        let doc_id = crate::Hasher::hash(HashAlgorithm::Sha256, b"test");
        let scope = SignatureScope::new(doc_id.clone());

        assert_eq!(scope.document_id, doc_id);
        assert!(scope.layouts.is_none());
        assert!(!scope.has_layouts());
    }

    #[test]
    fn test_signature_scope_with_layouts() {
        let doc_id = crate::Hasher::hash(HashAlgorithm::Sha256, b"test");
        let layout_hash = crate::Hasher::hash(HashAlgorithm::Sha256, b"layout");

        let scope =
            SignatureScope::new(doc_id).with_layout("presentation/print.json", layout_hash.clone());

        assert!(scope.has_layouts());
        let layouts = scope.layouts.as_ref().unwrap();
        assert_eq!(layouts.get("presentation/print.json"), Some(&layout_hash));
    }

    #[test]
    fn test_signature_scope_jcs_serialization() {
        let doc_id = crate::Hasher::hash(HashAlgorithm::Sha256, b"test");
        let scope = SignatureScope::new(doc_id);

        let jcs = scope.to_jcs().unwrap();
        assert!(!jcs.is_empty());
        // JCS should produce valid JSON
        let json_str = String::from_utf8(jcs).unwrap();
        assert!(json_str.contains("documentId"));
    }

    #[test]
    fn test_scoped_signature() {
        let doc_id = crate::Hasher::hash(HashAlgorithm::Sha256, b"test");
        let scope = SignatureScope::new(doc_id);

        let signer = SignerInfo::new("Test User");
        let sig = Signature::new("sig-1", SignatureAlgorithm::ES256, signer, "base64value")
            .with_scope(scope);

        assert!(sig.is_scoped());
        assert!(sig.scope.is_some());
    }

    #[test]
    fn test_signature_scope_serialization() {
        let doc_id = crate::Hasher::hash(HashAlgorithm::Sha256, b"test");
        let layout_hash = crate::Hasher::hash(HashAlgorithm::Sha256, b"layout");

        let scope = SignatureScope::new(doc_id).with_layout("presentation/print.json", layout_hash);

        let json = serde_json::to_string(&scope).unwrap();
        assert!(json.contains("\"documentId\":"));
        assert!(json.contains("\"layouts\":"));
        assert!(json.contains("presentation/print.json"));

        // Roundtrip
        let parsed: SignatureScope = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, scope);
    }

    #[test]
    fn test_signature_algorithm_display() {
        assert_eq!(SignatureAlgorithm::ES256.to_string(), "ES256");
        assert_eq!(SignatureAlgorithm::ES384.to_string(), "ES384");
        assert_eq!(SignatureAlgorithm::EdDSA.to_string(), "EdDSA");
        assert_eq!(SignatureAlgorithm::PS256.to_string(), "PS256");
        assert_eq!(SignatureAlgorithm::MlDsa65.to_string(), "ML-DSA-65");
    }

    #[test]
    fn test_signature_algorithm_as_str() {
        assert_eq!(SignatureAlgorithm::ES256.as_str(), "ES256");
        assert_eq!(SignatureAlgorithm::ES384.as_str(), "ES384");
        assert_eq!(SignatureAlgorithm::EdDSA.as_str(), "EdDSA");
        assert_eq!(SignatureAlgorithm::PS256.as_str(), "PS256");
        assert_eq!(SignatureAlgorithm::MlDsa65.as_str(), "ML-DSA-65");
    }

    #[test]
    fn test_signature_algorithm_is_post_quantum() {
        assert!(!SignatureAlgorithm::ES256.is_post_quantum());
        assert!(!SignatureAlgorithm::ES384.is_post_quantum());
        assert!(!SignatureAlgorithm::EdDSA.is_post_quantum());
        assert!(!SignatureAlgorithm::PS256.is_post_quantum());
        assert!(SignatureAlgorithm::MlDsa65.is_post_quantum());
    }

    #[test]
    fn test_signature_algorithm_serialization() {
        // Test each algorithm variant serializes correctly
        let json = serde_json::to_string(&SignatureAlgorithm::ES256).unwrap();
        assert_eq!(json, "\"ES256\"");

        let json = serde_json::to_string(&SignatureAlgorithm::MlDsa65).unwrap();
        assert_eq!(json, "\"ML-DSA-65\"");

        // Test deserialization
        let algo: SignatureAlgorithm = serde_json::from_str("\"EdDSA\"").unwrap();
        assert_eq!(algo, SignatureAlgorithm::EdDSA);
    }

    #[test]
    fn test_signature_file_roundtrip() {
        let doc_id = crate::Hasher::hash(HashAlgorithm::Sha256, b"test document");
        let mut file = SignatureFile::new(doc_id.clone());

        let signer = SignerInfo::new("Test User").with_email("test@example.com");
        let sig = Signature::new("sig-1", SignatureAlgorithm::ES256, signer, "base64value");
        file.add_signature(sig);

        let json = file.to_json().unwrap();
        let parsed = SignatureFile::from_json(&json).unwrap();

        assert_eq!(parsed.document_id, doc_id);
        assert_eq!(parsed.signatures.len(), 1);
        assert_eq!(parsed.signatures[0].id, "sig-1");
    }

    #[test]
    fn test_signature_file_find_signature() {
        let doc_id = crate::Hasher::hash(HashAlgorithm::Sha256, b"test");
        let mut file = SignatureFile::new(doc_id);

        let signer1 = SignerInfo::new("User 1");
        let signer2 = SignerInfo::new("User 2");
        file.add_signature(Signature::new(
            "sig-1",
            SignatureAlgorithm::ES256,
            signer1,
            "val1",
        ));
        file.add_signature(Signature::new(
            "sig-2",
            SignatureAlgorithm::EdDSA,
            signer2,
            "val2",
        ));

        assert!(file.find_signature("sig-1").is_some());
        assert!(file.find_signature("sig-2").is_some());
        assert!(file.find_signature("sig-3").is_none());

        let sig1 = file.find_signature("sig-1").unwrap();
        assert_eq!(sig1.algorithm, SignatureAlgorithm::ES256);
    }

    #[test]
    fn test_signature_file_len() {
        let doc_id = crate::Hasher::hash(HashAlgorithm::Sha256, b"test");
        let mut file = SignatureFile::new(doc_id);

        assert_eq!(file.len(), 0);
        assert!(file.is_empty());

        let signer = SignerInfo::new("User");
        file.add_signature(Signature::new(
            "sig-1",
            SignatureAlgorithm::ES256,
            signer,
            "val",
        ));

        assert_eq!(file.len(), 1);
        assert!(!file.is_empty());
    }

    #[test]
    fn test_webauthn_signature() {
        let signer = SignerInfo::new("WebAuthn User");
        let webauthn = WebAuthnSignature::new(
            "credential-id-base64",
            "authenticator-data-base64",
            "client-data-json-base64",
            "signature-base64",
        );

        let sig = Signature::new_webauthn("sig-webauthn", signer, webauthn);

        assert!(sig.is_webauthn());
        assert!(!sig.is_scoped());
        assert_eq!(sig.algorithm, SignatureAlgorithm::ES256);
        assert!(sig.value.is_empty());

        let webauthn_data = sig.webauthn_data().unwrap();
        assert_eq!(webauthn_data.credential_id, "credential-id-base64");
    }

    #[test]
    fn test_webauthn_signature_serialization() {
        let signer = SignerInfo::new("WebAuthn User");
        let webauthn = WebAuthnSignature::new("cred-id", "auth-data", "client-data", "sig-value");

        let sig = Signature::new_webauthn("sig-1", signer, webauthn);
        let json = serde_json::to_string_pretty(&sig).unwrap();

        assert!(json.contains("\"webauthn\":"));
        assert!(json.contains("\"credentialId\":"));
        assert!(json.contains("\"authenticatorData\":"));
        assert!(json.contains("\"clientDataJson\":"));

        // Roundtrip
        let parsed: Signature = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_webauthn());
        assert_eq!(parsed.webauthn_data().unwrap().credential_id, "cred-id");
    }

    #[test]
    fn test_verification_status_variants() {
        let valid = SignatureVerification::valid("sig-1");
        assert_eq!(valid.status, VerificationStatus::Valid);

        let invalid = SignatureVerification::invalid("sig-2", "bad sig");
        assert_eq!(invalid.status, VerificationStatus::Invalid);

        // Test all status variants exist
        assert!(matches!(
            VerificationStatus::Valid,
            VerificationStatus::Valid
        ));
        assert!(matches!(
            VerificationStatus::Invalid,
            VerificationStatus::Invalid
        ));
        assert!(matches!(
            VerificationStatus::Expired,
            VerificationStatus::Expired
        ));
        assert!(matches!(
            VerificationStatus::Revoked,
            VerificationStatus::Revoked
        ));
        assert!(matches!(
            VerificationStatus::Untrusted,
            VerificationStatus::Untrusted
        ));
        assert!(matches!(
            VerificationStatus::Unknown,
            VerificationStatus::Unknown
        ));
    }

    #[test]
    fn test_trusted_timestamp_roundtrip() {
        let ts = TrustedTimestamp {
            token: "base64-timestamp-token".to_string(),
            tsa: Some("https://tsa.example.com".to_string()),
        };
        let json = serde_json::to_string(&ts).unwrap();
        assert!(json.contains("\"token\":"));
        assert!(json.contains("\"tsa\":"));

        let parsed: TrustedTimestamp = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ts);
    }

    #[test]
    fn test_trusted_timestamp_without_tsa() {
        let ts = TrustedTimestamp {
            token: "base64-timestamp-token".to_string(),
            tsa: None,
        };
        let json = serde_json::to_string(&ts).unwrap();
        assert!(!json.contains("tsa"));

        let parsed: TrustedTimestamp = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ts);
    }

    #[test]
    fn test_signature_with_timestamp_builder() {
        let signer = SignerInfo::new("Test User");
        let ts = TrustedTimestamp {
            token: "base64-timestamp-token".to_string(),
            tsa: Some("https://tsa.example.com".to_string()),
        };
        let sig = Signature::new("sig-1", SignatureAlgorithm::ES256, signer, "base64value")
            .with_timestamp(ts.clone());

        assert_eq!(sig.timestamp, Some(ts));
    }

    #[test]
    fn test_signature_backward_compat_no_timestamp() {
        // JSON without timestamp field should deserialize fine
        let json = r#"{
            "id": "sig-1",
            "algorithm": "ES256",
            "signedAt": "2024-01-01T00:00:00Z",
            "signer": { "name": "Test User" },
            "value": "base64value"
        }"#;
        let sig: Signature = serde_json::from_str(json).unwrap();
        assert!(sig.timestamp.is_none());
    }
}
