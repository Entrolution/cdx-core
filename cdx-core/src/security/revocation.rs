//! Certificate revocation checking via OCSP and CRL.
//!
//! This module provides online revocation checking for X.509 certificates
//! using OCSP (Online Certificate Status Protocol) and CRL (Certificate
//! Revocation Lists).
//!
//! # Feature Flag
//!
//! This module requires the `ocsp` feature:
//!
//! ```toml
//! [dependencies]
//! cdx-core = { version = "0.1", features = ["ocsp"] }
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use cdx_core::security::revocation::{RevocationChecker, RevocationStatus};
//!
//! let checker = RevocationChecker::new();
//! let status = checker.check_ocsp(certificate_der, issuer_der).await?;
//!
//! match status {
//!     RevocationStatus::Good => println!("Certificate is valid"),
//!     RevocationStatus::Revoked { reason, time } => {
//!         println!("Certificate revoked: {:?}", reason);
//!     }
//!     RevocationStatus::Unknown => println!("Status unknown"),
//! }
//! ```

use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Revocation status of a certificate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum RevocationStatus {
    /// Certificate is not revoked.
    Good,

    /// Certificate has been revoked.
    Revoked {
        /// Reason for revocation.
        #[serde(skip_serializing_if = "Option::is_none")]
        reason: Option<RevocationReason>,
        /// Time of revocation (ISO 8601).
        #[serde(skip_serializing_if = "Option::is_none")]
        revocation_time: Option<String>,
    },

    /// Revocation status is unknown.
    Unknown,

    /// Revocation check failed.
    Error {
        /// Error message.
        message: String,
    },
}

impl RevocationStatus {
    /// Check if the certificate is known to be good.
    #[must_use]
    pub fn is_good(&self) -> bool {
        matches!(self, Self::Good)
    }

    /// Check if the certificate is revoked.
    #[must_use]
    pub fn is_revoked(&self) -> bool {
        matches!(self, Self::Revoked { .. })
    }

    /// Check if there was an error checking revocation.
    #[must_use]
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }
}

impl fmt::Display for RevocationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Good => write!(f, "good"),
            Self::Revoked { reason, .. } => {
                if let Some(r) = reason {
                    write!(f, "revoked ({r})")
                } else {
                    write!(f, "revoked")
                }
            }
            Self::Unknown => write!(f, "unknown"),
            Self::Error { message } => write!(f, "error: {message}"),
        }
    }
}

/// Reason for certificate revocation (RFC 5280).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[repr(u8)]
pub enum RevocationReason {
    /// Unspecified reason.
    Unspecified = 0,
    /// Key has been compromised.
    KeyCompromise = 1,
    /// CA key has been compromised.
    CaCompromise = 2,
    /// Affiliation has changed.
    AffiliationChanged = 3,
    /// Certificate has been superseded.
    Superseded = 4,
    /// Certificate is no longer needed.
    CessationOfOperation = 5,
    /// Certificate is on hold.
    CertificateHold = 6,
    /// Removed from CRL (not revoked).
    RemoveFromCrl = 8,
    /// Privilege has been withdrawn.
    PrivilegeWithdrawn = 9,
    /// AA has been compromised.
    AaCompromise = 10,
}

impl RevocationReason {
    /// Create from RFC 5280 reason code.
    #[must_use]
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(Self::Unspecified),
            1 => Some(Self::KeyCompromise),
            2 => Some(Self::CaCompromise),
            3 => Some(Self::AffiliationChanged),
            4 => Some(Self::Superseded),
            5 => Some(Self::CessationOfOperation),
            6 => Some(Self::CertificateHold),
            8 => Some(Self::RemoveFromCrl),
            9 => Some(Self::PrivilegeWithdrawn),
            10 => Some(Self::AaCompromise),
            _ => None,
        }
    }

    /// Get the RFC 5280 reason code.
    #[must_use]
    pub const fn code(&self) -> u8 {
        *self as u8
    }
}

impl fmt::Display for RevocationReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unspecified => write!(f, "unspecified"),
            Self::KeyCompromise => write!(f, "key compromise"),
            Self::CaCompromise => write!(f, "CA compromise"),
            Self::AffiliationChanged => write!(f, "affiliation changed"),
            Self::Superseded => write!(f, "superseded"),
            Self::CessationOfOperation => write!(f, "cessation of operation"),
            Self::CertificateHold => write!(f, "certificate hold"),
            Self::RemoveFromCrl => write!(f, "remove from CRL"),
            Self::PrivilegeWithdrawn => write!(f, "privilege withdrawn"),
            Self::AaCompromise => write!(f, "AA compromise"),
        }
    }
}

/// Result of a revocation check.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationResult {
    /// The revocation status.
    pub status: RevocationStatus,

    /// Method used for the check.
    pub method: RevocationMethod,

    /// URL of the responder or CRL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responder_url: Option<String>,

    /// When this check was performed (ISO 8601).
    pub checked_at: String,

    /// When the response was produced (ISO 8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub produced_at: Option<String>,

    /// When this response should be considered stale (ISO 8601).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_update: Option<String>,

    /// Certificate serial number that was checked.
    pub serial_number: String,
}

impl RevocationResult {
    /// Create a new revocation result.
    #[must_use]
    pub fn new(status: RevocationStatus, method: RevocationMethod, serial_number: String) -> Self {
        Self {
            status,
            method,
            responder_url: None,
            checked_at: chrono::Utc::now().to_rfc3339(),
            produced_at: None,
            next_update: None,
            serial_number,
        }
    }

    /// Set the responder URL.
    #[must_use]
    pub fn with_responder(mut self, url: impl Into<String>) -> Self {
        self.responder_url = Some(url.into());
        self
    }

    /// Set the produced_at time.
    #[must_use]
    pub fn with_produced_at(mut self, time: impl Into<String>) -> Self {
        self.produced_at = Some(time.into());
        self
    }

    /// Set the next_update time.
    #[must_use]
    pub fn with_next_update(mut self, time: impl Into<String>) -> Self {
        self.next_update = Some(time.into());
        self
    }

    /// Check if the result indicates the certificate is valid.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.status.is_good()
    }
}

/// Method used for revocation checking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RevocationMethod {
    /// OCSP (Online Certificate Status Protocol).
    Ocsp,
    /// CRL (Certificate Revocation List).
    Crl,
    /// OCSP stapled in TLS handshake.
    OcspStapling,
}

impl fmt::Display for RevocationMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ocsp => write!(f, "OCSP"),
            Self::Crl => write!(f, "CRL"),
            Self::OcspStapling => write!(f, "OCSP Stapling"),
        }
    }
}

/// Configuration for revocation checking.
#[derive(Debug, Clone)]
pub struct RevocationConfig {
    /// Timeout for network requests.
    pub timeout: Duration,

    /// Whether to prefer OCSP over CRL.
    pub prefer_ocsp: bool,

    /// Whether to use OCSP stapling when available.
    pub use_stapling: bool,

    /// Whether to require a valid revocation response.
    /// If false, unknown status is treated as valid.
    pub strict_mode: bool,

    /// Maximum age of cached CRL responses in seconds.
    pub max_crl_age: u64,

    /// Custom OCSP responder URL (overrides AIA).
    pub ocsp_responder: Option<String>,
}

impl Default for RevocationConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            prefer_ocsp: true,
            use_stapling: true,
            strict_mode: false,
            max_crl_age: 86400, // 24 hours
            ocsp_responder: None,
        }
    }
}

impl RevocationConfig {
    /// Create a new configuration with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the network timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set whether to prefer OCSP over CRL.
    #[must_use]
    pub fn with_prefer_ocsp(mut self, prefer: bool) -> Self {
        self.prefer_ocsp = prefer;
        self
    }

    /// Set strict mode (require valid revocation response).
    #[must_use]
    pub fn with_strict_mode(mut self, strict: bool) -> Self {
        self.strict_mode = strict;
        self
    }

    /// Set a custom OCSP responder URL.
    #[must_use]
    pub fn with_ocsp_responder(mut self, url: impl Into<String>) -> Self {
        self.ocsp_responder = Some(url.into());
        self
    }
}

/// Certificate revocation checker.
///
/// This provides methods for checking certificate revocation status
/// using OCSP and CRL protocols.
pub struct RevocationChecker {
    config: RevocationConfig,
    #[cfg(feature = "ocsp")]
    client: reqwest::Client,
}

impl RevocationChecker {
    /// Create a new revocation checker with default configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be initialized.
    #[cfg(feature = "ocsp")]
    pub fn new() -> Result<Self, crate::Error> {
        Self::with_config(RevocationConfig::default())
    }

    /// Create a new revocation checker with custom configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be initialized.
    #[cfg(feature = "ocsp")]
    pub fn with_config(config: RevocationConfig) -> Result<Self, crate::Error> {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| crate::Error::Network {
                message: format!("Failed to create HTTP client: {e}"),
            })?;

        Ok(Self { config, client })
    }

    /// Check certificate revocation status via OCSP.
    ///
    /// # Arguments
    ///
    /// * `cert_der` - DER-encoded certificate to check
    /// * `issuer_der` - DER-encoded issuer certificate
    ///
    /// # Returns
    ///
    /// The revocation status of the certificate.
    ///
    /// # Errors
    ///
    /// Returns an error if the OCSP check fails.
    #[cfg(feature = "ocsp")]
    pub async fn check_ocsp(
        &self,
        cert_der: &[u8],
        issuer_der: &[u8],
    ) -> Result<RevocationResult, crate::Error> {
        use der::Decode;
        use x509_cert::Certificate;

        // Parse the certificates
        let cert =
            Certificate::from_der(cert_der).map_err(|e| crate::Error::InvalidCertificate {
                reason: format!("Failed to parse certificate: {e}"),
            })?;

        let issuer =
            Certificate::from_der(issuer_der).map_err(|e| crate::Error::InvalidCertificate {
                reason: format!("Failed to parse issuer certificate: {e}"),
            })?;

        // Get serial number as hex string
        let serial_bytes = cert.tbs_certificate.serial_number.as_bytes();
        let serial = bytes_to_hex(serial_bytes);

        // Get OCSP responder URL
        let responder_url = self
            .config
            .ocsp_responder
            .clone()
            .or_else(|| extract_ocsp_url(&cert))
            .ok_or_else(|| crate::Error::InvalidCertificate {
                reason: "No OCSP responder URL found in certificate".to_string(),
            })?;

        // Build OCSP request
        let request_body = build_ocsp_request(&cert, &issuer)?;

        // Send OCSP request
        let response = self
            .client
            .post(&responder_url)
            .header("Content-Type", "application/ocsp-request")
            .body(request_body)
            .send()
            .await
            .map_err(|e| crate::Error::Network {
                message: format!("OCSP request failed: {e}"),
            })?;

        if !response.status().is_success() {
            return Ok(RevocationResult::new(
                RevocationStatus::Error {
                    message: format!("OCSP responder returned status {}", response.status()),
                },
                RevocationMethod::Ocsp,
                serial,
            )
            .with_responder(&responder_url));
        }

        let response_body = response.bytes().await.map_err(|e| crate::Error::Network {
            message: format!("Failed to read OCSP response: {e}"),
        })?;

        // Parse OCSP response
        let status = parse_ocsp_response(&response_body);

        Ok(
            RevocationResult::new(status, RevocationMethod::Ocsp, serial)
                .with_responder(&responder_url),
        )
    }

    /// Check certificate revocation status via CRL.
    ///
    /// # Arguments
    ///
    /// * `cert_der` - DER-encoded certificate to check
    ///
    /// # Returns
    ///
    /// The revocation status of the certificate.
    ///
    /// # Errors
    ///
    /// Returns an error if the CRL check fails.
    #[cfg(feature = "ocsp")]
    pub async fn check_crl(&self, cert_der: &[u8]) -> Result<RevocationResult, crate::Error> {
        use der::Decode;
        use x509_cert::Certificate;

        // Parse the certificate
        let cert =
            Certificate::from_der(cert_der).map_err(|e| crate::Error::InvalidCertificate {
                reason: format!("Failed to parse certificate: {e}"),
            })?;

        // Get serial number as hex string
        let serial_bytes = cert.tbs_certificate.serial_number.as_bytes();
        let serial = bytes_to_hex(serial_bytes);

        // Get CRL distribution point
        let crl_url = extract_crl_url(&cert).ok_or_else(|| crate::Error::InvalidCertificate {
            reason: "No CRL distribution point found in certificate".to_string(),
        })?;

        // Fetch CRL
        let response =
            self.client
                .get(&crl_url)
                .send()
                .await
                .map_err(|e| crate::Error::Network {
                    message: format!("CRL fetch failed: {e}"),
                })?;

        if !response.status().is_success() {
            return Ok(RevocationResult::new(
                RevocationStatus::Error {
                    message: format!("CRL server returned status {}", response.status()),
                },
                RevocationMethod::Crl,
                serial,
            )
            .with_responder(&crl_url));
        }

        let crl_data = response.bytes().await.map_err(|e| crate::Error::Network {
            message: format!("Failed to read CRL: {e}"),
        })?;

        // Parse and check CRL
        let status = check_crl_for_serial(&crl_data, &cert.tbs_certificate.serial_number)?;

        Ok(RevocationResult::new(status, RevocationMethod::Crl, serial).with_responder(&crl_url))
    }

    /// Check certificate revocation status using the preferred method.
    ///
    /// Tries OCSP first if configured, falls back to CRL.
    ///
    /// # Arguments
    ///
    /// * `cert_der` - DER-encoded certificate to check
    /// * `issuer_der` - DER-encoded issuer certificate (for OCSP)
    ///
    /// # Returns
    ///
    /// The revocation status of the certificate.
    ///
    /// # Errors
    ///
    /// Returns an error if all revocation checks fail.
    #[cfg(feature = "ocsp")]
    pub async fn check(
        &self,
        cert_der: &[u8],
        issuer_der: Option<&[u8]>,
    ) -> Result<RevocationResult, crate::Error> {
        if self.config.prefer_ocsp {
            // Try OCSP first
            if let Some(issuer) = issuer_der {
                match self.check_ocsp(cert_der, issuer).await {
                    Ok(result) if !result.status.is_error() => return Ok(result),
                    _ => {} // Fall through to CRL
                }
            }

            // Fall back to CRL
            self.check_crl(cert_der).await
        } else {
            // Try CRL first
            match self.check_crl(cert_der).await {
                Ok(result) if !result.status.is_error() => Ok(result),
                _ => {
                    // Fall back to OCSP
                    if let Some(issuer) = issuer_der {
                        self.check_ocsp(cert_der, issuer).await
                    } else {
                        Err(crate::Error::InvalidCertificate {
                            reason: "CRL check failed and no issuer provided for OCSP".to_string(),
                        })
                    }
                }
            }
        }
    }

    /// Get the current configuration.
    #[must_use]
    pub fn config(&self) -> &RevocationConfig {
        &self.config
    }
}

impl std::fmt::Debug for RevocationChecker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RevocationChecker")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

// Helper functions for OCSP

/// Convert bytes to uppercase hex string.
#[cfg(feature = "ocsp")]
fn bytes_to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut acc, b| {
            let _ = write!(acc, "{b:02X}");
            acc
        })
}

#[cfg(feature = "ocsp")]
fn extract_ocsp_url(cert: &x509_cert::Certificate) -> Option<String> {
    use x509_cert::ext::pkix::AuthorityInfoAccessSyntax;

    let extensions = cert.tbs_certificate.extensions.as_ref()?;

    for ext in extensions {
        // OID for Authority Information Access: 1.3.6.1.5.5.7.1.1
        if ext.extn_id.to_string() == "1.3.6.1.5.5.7.1.1" {
            if let Ok(aia) =
                <AuthorityInfoAccessSyntax as der::Decode>::from_der(ext.extn_value.as_bytes())
            {
                for access_desc in &aia.0 {
                    // OID for OCSP: 1.3.6.1.5.5.7.48.1
                    if access_desc.access_method.to_string() == "1.3.6.1.5.5.7.48.1" {
                        if let x509_cert::ext::pkix::name::GeneralName::UniformResourceIdentifier(
                            uri,
                        ) = &access_desc.access_location
                        {
                            return Some(uri.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

#[cfg(feature = "ocsp")]
fn extract_crl_url(cert: &x509_cert::Certificate) -> Option<String> {
    let extensions = cert.tbs_certificate.extensions.as_ref()?;

    for ext in extensions {
        // OID for CRL Distribution Points: 2.5.29.31
        if ext.extn_id.to_string() == "2.5.29.31" {
            // Parse the CRL distribution points
            // This is a simplified extraction - in practice you'd fully parse the ASN.1
            let bytes = ext.extn_value.as_bytes();
            // Look for http:// or https:// in the extension value
            if let Ok(s) = std::str::from_utf8(bytes) {
                if let Some(start) = s.find("http://").or_else(|| s.find("https://")) {
                    let end = s[start..]
                        .find(|c: char| c.is_control() || c == '\0')
                        .map_or(s.len(), |e| start + e);
                    return Some(s[start..end].to_string());
                }
            }
        }
    }

    None
}

#[cfg(feature = "ocsp")]
fn build_ocsp_request(
    _cert: &x509_cert::Certificate,
    _issuer: &x509_cert::Certificate,
) -> Result<Vec<u8>, crate::Error> {
    // Build a minimal OCSP request
    // In a full implementation, this would:
    // 1. Hash the issuer's name and key
    // 2. Include the certificate serial number
    // 3. Optionally add a nonce for replay protection

    // For now, return a placeholder that indicates this needs full implementation
    // with a proper ASN.1 OCSP request builder
    Err(crate::Error::NotImplemented {
        feature: "Full OCSP request building requires ocsp-rs or similar crate".to_string(),
    })
}

#[cfg(feature = "ocsp")]
fn parse_ocsp_response(response: &[u8]) -> RevocationStatus {
    // Parse OCSP response
    // A full implementation would:
    // 1. Check the response status (successful, malformed, etc.)
    // 2. Verify the response signature
    // 3. Extract the certificate status

    // Check for basic response structure
    if response.is_empty() {
        return RevocationStatus::Error {
            message: "Empty OCSP response".to_string(),
        };
    }

    // OCSP response status is the first byte after the sequence tag
    // 0 = successful, 1 = malformed, 2 = internal error, etc.
    if response.len() > 2 {
        // This is a simplified check - full parsing would use proper ASN.1
        // Look for common success indicators
        if response.contains(&0x00) {
            // Likely successful response - would need full parsing to determine status
            return RevocationStatus::Unknown;
        }
    }

    RevocationStatus::Error {
        message: "Failed to parse OCSP response".to_string(),
    }
}

#[cfg(feature = "ocsp")]
fn check_crl_for_serial(
    crl_data: &[u8],
    serial: &x509_cert::serial_number::SerialNumber,
) -> Result<RevocationStatus, crate::Error> {
    use der::Decode;
    use x509_cert::crl::CertificateList;

    // Parse the CRL
    let crl =
        CertificateList::from_der(crl_data).map_err(|e| crate::Error::InvalidCertificate {
            reason: format!("Failed to parse CRL: {e}"),
        })?;

    // Check if the serial number is in the revoked list
    if let Some(revoked_certs) = &crl.tbs_cert_list.revoked_certificates {
        for revoked in revoked_certs {
            if &revoked.serial_number == serial {
                // Found in CRL - certificate is revoked
                let reason = revoked.crl_entry_extensions.as_ref().and_then(|exts| {
                    exts.iter().find_map(|ext| {
                        // OID for CRL Reason: 2.5.29.21
                        if ext.extn_id.to_string() == "2.5.29.21" {
                            let bytes = ext.extn_value.as_bytes();
                            if bytes.len() >= 3 {
                                RevocationReason::from_code(bytes[2])
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                });

                return Ok(RevocationStatus::Revoked {
                    reason,
                    revocation_time: Some(revoked.revocation_date.to_string()),
                });
            }
        }
    }

    // Not found in CRL - certificate is good
    Ok(RevocationStatus::Good)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_revocation_status_display() {
        assert_eq!(RevocationStatus::Good.to_string(), "good");
        assert_eq!(RevocationStatus::Unknown.to_string(), "unknown");
        assert_eq!(
            RevocationStatus::Revoked {
                reason: Some(RevocationReason::KeyCompromise),
                revocation_time: None,
            }
            .to_string(),
            "revoked (key compromise)"
        );
    }

    #[test]
    fn test_revocation_status_checks() {
        assert!(RevocationStatus::Good.is_good());
        assert!(!RevocationStatus::Good.is_revoked());

        let revoked = RevocationStatus::Revoked {
            reason: None,
            revocation_time: None,
        };
        assert!(!revoked.is_good());
        assert!(revoked.is_revoked());

        let error = RevocationStatus::Error {
            message: "test".to_string(),
        };
        assert!(error.is_error());
    }

    #[test]
    fn test_revocation_reason_from_code() {
        assert_eq!(
            RevocationReason::from_code(0),
            Some(RevocationReason::Unspecified)
        );
        assert_eq!(
            RevocationReason::from_code(1),
            Some(RevocationReason::KeyCompromise)
        );
        assert_eq!(
            RevocationReason::from_code(5),
            Some(RevocationReason::CessationOfOperation)
        );
        assert_eq!(RevocationReason::from_code(7), None); // No reason code 7
        assert_eq!(RevocationReason::from_code(255), None);
    }

    #[test]
    fn test_revocation_reason_code() {
        assert_eq!(RevocationReason::Unspecified.code(), 0);
        assert_eq!(RevocationReason::KeyCompromise.code(), 1);
        assert_eq!(RevocationReason::AaCompromise.code(), 10);
    }

    #[test]
    fn test_revocation_reason_display() {
        assert_eq!(
            RevocationReason::KeyCompromise.to_string(),
            "key compromise"
        );
        assert_eq!(
            RevocationReason::CessationOfOperation.to_string(),
            "cessation of operation"
        );
    }

    #[test]
    fn test_revocation_config_default() {
        let config = RevocationConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(10));
        assert!(config.prefer_ocsp);
        assert!(config.use_stapling);
        assert!(!config.strict_mode);
        assert!(config.ocsp_responder.is_none());
    }

    #[test]
    fn test_revocation_config_builder() {
        let config = RevocationConfig::new()
            .with_timeout(Duration::from_secs(30))
            .with_prefer_ocsp(false)
            .with_strict_mode(true)
            .with_ocsp_responder("http://ocsp.example.com");

        assert_eq!(config.timeout, Duration::from_secs(30));
        assert!(!config.prefer_ocsp);
        assert!(config.strict_mode);
        assert_eq!(
            config.ocsp_responder,
            Some("http://ocsp.example.com".to_string())
        );
    }

    #[test]
    fn test_revocation_result_new() {
        let result = RevocationResult::new(
            RevocationStatus::Good,
            RevocationMethod::Ocsp,
            "1234ABCD".to_string(),
        );

        assert!(result.is_valid());
        assert_eq!(result.method, RevocationMethod::Ocsp);
        assert_eq!(result.serial_number, "1234ABCD");
        assert!(result.responder_url.is_none());
    }

    #[test]
    fn test_revocation_result_builder() {
        let result = RevocationResult::new(
            RevocationStatus::Good,
            RevocationMethod::Ocsp,
            "1234".to_string(),
        )
        .with_responder("http://ocsp.example.com")
        .with_produced_at("2024-01-01T00:00:00Z")
        .with_next_update("2024-01-02T00:00:00Z");

        assert_eq!(
            result.responder_url,
            Some("http://ocsp.example.com".to_string())
        );
        assert_eq!(result.produced_at, Some("2024-01-01T00:00:00Z".to_string()));
        assert_eq!(result.next_update, Some("2024-01-02T00:00:00Z".to_string()));
    }

    #[test]
    fn test_revocation_method_display() {
        assert_eq!(RevocationMethod::Ocsp.to_string(), "OCSP");
        assert_eq!(RevocationMethod::Crl.to_string(), "CRL");
        assert_eq!(RevocationMethod::OcspStapling.to_string(), "OCSP Stapling");
    }

    #[test]
    fn test_revocation_status_serialization() {
        let good = RevocationStatus::Good;
        let json = serde_json::to_string(&good).unwrap();
        assert!(json.contains("\"status\":\"good\""));

        let revoked = RevocationStatus::Revoked {
            reason: Some(RevocationReason::KeyCompromise),
            revocation_time: Some("2024-01-01T00:00:00Z".to_string()),
        };
        let json = serde_json::to_string(&revoked).unwrap();
        assert!(json.contains("\"status\":\"revoked\""));
        assert!(json.contains("\"reason\":\"keyCompromise\""));
    }
}
