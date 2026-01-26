//! Certificate chain validation (offline).
//!
//! This module provides certificate parsing and chain validation functionality
//! for verifying X.509 certificate chains used in document signatures.
//!
//! Note: Online revocation checks (OCSP, CRL) are deferred to a separate
//! feature-gated module and require network access.

use serde::{Deserialize, Serialize};

/// Result of certificate chain validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertificateValidation {
    /// Whether the certificate chain is valid (structurally correct and trusted).
    pub valid: bool,

    /// Whether any certificate in the chain has expired.
    pub expired: bool,

    /// Whether any certificate is not yet valid (notBefore is in the future).
    pub not_yet_valid: bool,

    /// The trust path from leaf to root (subject names).
    pub trust_path: Vec<String>,

    /// Validation errors encountered.
    pub errors: Vec<String>,

    /// Warnings (non-fatal issues).
    pub warnings: Vec<String>,
}

impl CertificateValidation {
    /// Create a successful validation result.
    #[must_use]
    pub fn success(trust_path: Vec<String>) -> Self {
        Self {
            valid: true,
            expired: false,
            not_yet_valid: false,
            trust_path,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create a failed validation result.
    #[must_use]
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            valid: false,
            expired: false,
            not_yet_valid: false,
            trust_path: Vec::new(),
            errors: vec![error.into()],
            warnings: Vec::new(),
        }
    }

    /// Check if validation passed without errors.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.valid && self.errors.is_empty()
    }

    /// Check if validation passed but with warnings.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Add an error to the validation result.
    pub fn add_error(&mut self, error: impl Into<String>) {
        self.errors.push(error.into());
        self.valid = false;
    }

    /// Add a warning to the validation result.
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }
}

impl Default for CertificateValidation {
    fn default() -> Self {
        Self::failure("Not validated")
    }
}

/// Information extracted from a certificate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificateInfo {
    /// Subject distinguished name.
    pub subject: String,

    /// Issuer distinguished name.
    pub issuer: String,

    /// Serial number (hex encoded).
    pub serial_number: String,

    /// Not valid before (ISO 8601).
    pub not_before: String,

    /// Not valid after (ISO 8601).
    pub not_after: String,

    /// Whether this is a CA certificate.
    pub is_ca: bool,

    /// Key usage flags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub key_usage: Vec<KeyUsage>,

    /// Extended key usage OIDs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extended_key_usage: Vec<String>,

    /// Subject alternative names.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subject_alt_names: Vec<String>,

    /// SHA-256 fingerprint of the certificate (hex encoded).
    pub fingerprint_sha256: String,
}

impl CertificateInfo {
    /// Create certificate info with minimal required fields.
    #[must_use]
    pub fn new(
        subject: impl Into<String>,
        issuer: impl Into<String>,
        serial_number: impl Into<String>,
    ) -> Self {
        Self {
            subject: subject.into(),
            issuer: issuer.into(),
            serial_number: serial_number.into(),
            not_before: String::new(),
            not_after: String::new(),
            is_ca: false,
            key_usage: Vec::new(),
            extended_key_usage: Vec::new(),
            subject_alt_names: Vec::new(),
            fingerprint_sha256: String::new(),
        }
    }

    /// Check if the certificate is self-signed.
    #[must_use]
    pub fn is_self_signed(&self) -> bool {
        self.subject == self.issuer
    }

    /// Set the validity period.
    #[must_use]
    pub fn with_validity(
        mut self,
        not_before: impl Into<String>,
        not_after: impl Into<String>,
    ) -> Self {
        self.not_before = not_before.into();
        self.not_after = not_after.into();
        self
    }

    /// Set the CA flag.
    #[must_use]
    pub fn with_ca(mut self, is_ca: bool) -> Self {
        self.is_ca = is_ca;
        self
    }

    /// Set the fingerprint.
    #[must_use]
    pub fn with_fingerprint(mut self, fingerprint: impl Into<String>) -> Self {
        self.fingerprint_sha256 = fingerprint.into();
        self
    }

    /// Add a key usage.
    #[must_use]
    pub fn with_key_usage(mut self, usage: KeyUsage) -> Self {
        self.key_usage.push(usage);
        self
    }

    /// Add an extended key usage OID.
    #[must_use]
    pub fn with_extended_key_usage(mut self, oid: impl Into<String>) -> Self {
        self.extended_key_usage.push(oid.into());
        self
    }
}

/// Key usage flags for X.509 certificates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum KeyUsage {
    /// Digital signature.
    DigitalSignature,
    /// Non-repudiation (content commitment).
    NonRepudiation,
    /// Key encipherment.
    KeyEncipherment,
    /// Data encipherment.
    DataEncipherment,
    /// Key agreement.
    KeyAgreement,
    /// Key certificate signing.
    KeyCertSign,
    /// CRL signing.
    CrlSign,
    /// Encipher only (with key agreement).
    EncipherOnly,
    /// Decipher only (with key agreement).
    DecipherOnly,
}

impl std::fmt::Display for KeyUsage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DigitalSignature => write!(f, "digitalSignature"),
            Self::NonRepudiation => write!(f, "nonRepudiation"),
            Self::KeyEncipherment => write!(f, "keyEncipherment"),
            Self::DataEncipherment => write!(f, "dataEncipherment"),
            Self::KeyAgreement => write!(f, "keyAgreement"),
            Self::KeyCertSign => write!(f, "keyCertSign"),
            Self::CrlSign => write!(f, "cRLSign"),
            Self::EncipherOnly => write!(f, "encipherOnly"),
            Self::DecipherOnly => write!(f, "decipherOnly"),
        }
    }
}

/// Common extended key usage OIDs.
pub mod eku {
    /// Server authentication (1.3.6.1.5.5.7.3.1)
    pub const SERVER_AUTH: &str = "1.3.6.1.5.5.7.3.1";
    /// Client authentication (1.3.6.1.5.5.7.3.2)
    pub const CLIENT_AUTH: &str = "1.3.6.1.5.5.7.3.2";
    /// Code signing (1.3.6.1.5.5.7.3.3)
    pub const CODE_SIGNING: &str = "1.3.6.1.5.5.7.3.3";
    /// Email protection (1.3.6.1.5.5.7.3.4)
    pub const EMAIL_PROTECTION: &str = "1.3.6.1.5.5.7.3.4";
    /// Time stamping (1.3.6.1.5.5.7.3.8)
    pub const TIME_STAMPING: &str = "1.3.6.1.5.5.7.3.8";
    /// Document signing (1.3.6.1.5.5.7.3.36)
    pub const DOCUMENT_SIGNING: &str = "1.3.6.1.5.5.7.3.36";
}

/// A certificate chain for validation.
#[derive(Debug, Clone)]
pub struct CertificateChain {
    /// Certificates in the chain, from leaf to root.
    pub certificates: Vec<CertificateInfo>,
}

impl CertificateChain {
    /// Create a new certificate chain.
    #[must_use]
    pub fn new(certificates: Vec<CertificateInfo>) -> Self {
        Self { certificates }
    }

    /// Create an empty certificate chain.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            certificates: Vec::new(),
        }
    }

    /// Get the leaf (end-entity) certificate.
    #[must_use]
    pub fn leaf(&self) -> Option<&CertificateInfo> {
        self.certificates.first()
    }

    /// Get the root certificate.
    #[must_use]
    pub fn root(&self) -> Option<&CertificateInfo> {
        self.certificates.last()
    }

    /// Check if the chain is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.certificates.is_empty()
    }

    /// Get the number of certificates in the chain.
    #[must_use]
    pub fn len(&self) -> usize {
        self.certificates.len()
    }

    /// Add a certificate to the chain.
    pub fn push(&mut self, cert: CertificateInfo) {
        self.certificates.push(cert);
    }

    /// Validate the certificate chain structure (offline).
    ///
    /// This performs basic structural validation:
    /// - Chain is not empty
    /// - Each certificate is issued by the next one in the chain
    /// - Root certificate is self-signed
    /// - CA certificates have the CA flag set
    ///
    /// Note: This does NOT validate:
    /// - Cryptographic signatures (requires parsing actual X.509)
    /// - Expiration dates (requires current time)
    /// - Revocation status (requires network)
    #[must_use]
    pub fn validate_structure(&self) -> CertificateValidation {
        if self.certificates.is_empty() {
            return CertificateValidation::failure("Certificate chain is empty");
        }

        let mut result = CertificateValidation::success(Vec::new());

        // Build trust path
        for cert in &self.certificates {
            result.trust_path.push(cert.subject.clone());
        }

        // Validate chain linkage
        for i in 0..self.certificates.len() - 1 {
            let cert = &self.certificates[i];
            let issuer = &self.certificates[i + 1];

            // Check that cert's issuer matches issuer's subject
            if cert.issuer != issuer.subject {
                result.add_error(format!(
                    "Chain broken: '{}' issuer '{}' does not match next certificate subject '{}'",
                    cert.subject, cert.issuer, issuer.subject
                ));
            }

            // Check that intermediate/root certs have CA flag
            if !issuer.is_ca {
                result.add_warning(format!("Issuer '{}' is not marked as a CA", issuer.subject));
            }
        }

        // Check that root is self-signed
        if let Some(root) = self.root() {
            if !root.is_self_signed() {
                result.add_warning(format!(
                    "Root certificate '{}' is not self-signed (issuer: '{}')",
                    root.subject, root.issuer
                ));
            }
        }

        result
    }

    /// Validate that the chain is trusted by the given trust anchors.
    ///
    /// The chain's root must match one of the trusted roots by fingerprint.
    #[must_use]
    pub fn validate_trust(&self, trusted_roots: &[CertificateInfo]) -> CertificateValidation {
        // First do structural validation
        let mut result = self.validate_structure();
        if !result.valid {
            return result;
        }

        // Check if root is trusted
        if let Some(root) = self.root() {
            let is_trusted = trusted_roots.iter().any(|trusted| {
                trusted.fingerprint_sha256 == root.fingerprint_sha256
                    && !trusted.fingerprint_sha256.is_empty()
            });

            if !is_trusted {
                result.add_error(format!(
                    "Root certificate '{}' is not in the trusted roots",
                    root.subject
                ));
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_chain() -> CertificateChain {
        let leaf = CertificateInfo::new("CN=leaf.example.com", "CN=Intermediate CA", "1234")
            .with_fingerprint("aabbccdd")
            .with_key_usage(KeyUsage::DigitalSignature);

        let intermediate = CertificateInfo::new("CN=Intermediate CA", "CN=Root CA", "5678")
            .with_ca(true)
            .with_fingerprint("eeff0011")
            .with_key_usage(KeyUsage::KeyCertSign);

        let root = CertificateInfo::new("CN=Root CA", "CN=Root CA", "9999")
            .with_ca(true)
            .with_fingerprint("11223344")
            .with_key_usage(KeyUsage::KeyCertSign);

        CertificateChain::new(vec![leaf, intermediate, root])
    }

    #[test]
    fn test_certificate_validation_success() {
        let result = CertificateValidation::success(vec!["leaf".to_string(), "root".to_string()]);
        assert!(result.is_valid());
        assert!(!result.has_warnings());
        assert_eq!(result.trust_path.len(), 2);
    }

    #[test]
    fn test_certificate_validation_failure() {
        let result = CertificateValidation::failure("Invalid certificate");
        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_certificate_info_self_signed() {
        let self_signed = CertificateInfo::new("CN=Root CA", "CN=Root CA", "1234");
        assert!(self_signed.is_self_signed());

        let not_self_signed = CertificateInfo::new("CN=Leaf", "CN=Root CA", "5678");
        assert!(!not_self_signed.is_self_signed());
    }

    #[test]
    fn test_certificate_chain_structure() {
        let chain = create_test_chain();

        assert_eq!(chain.len(), 3);
        assert!(!chain.is_empty());
        assert_eq!(chain.leaf().unwrap().subject, "CN=leaf.example.com");
        assert_eq!(chain.root().unwrap().subject, "CN=Root CA");
    }

    #[test]
    fn test_validate_structure_valid() {
        let chain = create_test_chain();
        let result = chain.validate_structure();

        assert!(result.is_valid());
        assert_eq!(result.trust_path.len(), 3);
    }

    #[test]
    fn test_validate_structure_empty_chain() {
        let chain = CertificateChain::empty();
        let result = chain.validate_structure();

        assert!(!result.is_valid());
        assert!(result.errors[0].contains("empty"));
    }

    #[test]
    fn test_validate_structure_broken_chain() {
        let leaf = CertificateInfo::new("CN=leaf.example.com", "CN=Wrong Issuer", "1234");
        let root = CertificateInfo::new("CN=Root CA", "CN=Root CA", "9999").with_ca(true);

        let chain = CertificateChain::new(vec![leaf, root]);
        let result = chain.validate_structure();

        assert!(!result.is_valid());
        assert!(result.errors[0].contains("Chain broken"));
    }

    #[test]
    fn test_validate_trust_trusted_root() {
        let chain = create_test_chain();
        let trusted_root =
            CertificateInfo::new("CN=Root CA", "CN=Root CA", "9999").with_fingerprint("11223344");

        let result = chain.validate_trust(&[trusted_root]);
        assert!(result.is_valid());
    }

    #[test]
    fn test_validate_trust_untrusted_root() {
        let chain = create_test_chain();
        let other_root = CertificateInfo::new("CN=Other Root", "CN=Other Root", "0000")
            .with_fingerprint("99887766");

        let result = chain.validate_trust(&[other_root]);
        assert!(!result.is_valid());
        assert!(result.errors[0].contains("not in the trusted roots"));
    }

    #[test]
    fn test_key_usage_display() {
        assert_eq!(KeyUsage::DigitalSignature.to_string(), "digitalSignature");
        assert_eq!(KeyUsage::KeyCertSign.to_string(), "keyCertSign");
    }

    #[test]
    fn test_certificate_info_serialization() {
        let cert = CertificateInfo::new("CN=Test", "CN=Issuer", "1234")
            .with_validity("2024-01-01T00:00:00Z", "2025-01-01T00:00:00Z")
            .with_ca(true)
            .with_fingerprint("abcd1234")
            .with_key_usage(KeyUsage::DigitalSignature)
            .with_extended_key_usage(eku::DOCUMENT_SIGNING);

        let json = serde_json::to_string_pretty(&cert).unwrap();
        assert!(json.contains("\"subject\": \"CN=Test\""));
        assert!(json.contains("\"isCa\": true"));

        let deserialized: CertificateInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.subject, "CN=Test");
        assert!(deserialized.is_ca);
    }

    #[test]
    fn test_eku_constants() {
        assert_eq!(eku::SERVER_AUTH, "1.3.6.1.5.5.7.3.1");
        assert_eq!(eku::DOCUMENT_SIGNING, "1.3.6.1.5.5.7.3.36");
    }
}
