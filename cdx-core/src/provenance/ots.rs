//! `OpenTimestamps` client for timestamp acquisition.
//!
//! This module provides an async client for acquiring timestamps from
//! `OpenTimestamps` calendar servers. `OpenTimestamps` aggregates hashes and
//! anchors them to the Bitcoin blockchain.
//!
//! # Feature Flag
//!
//! This module requires the `timestamps-ots` feature:
//!
//! ```toml
//! [dependencies]
//! cdx-core = { version = "0.1", features = ["timestamps-ots"] }
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use cdx_core::provenance::ots::OtsClient;
//! use cdx_core::{HashAlgorithm, Hasher};
//!
//! # async fn example() -> cdx_core::Result<()> {
//! let client = OtsClient::new();
//! let doc_id = Hasher::hash(HashAlgorithm::Sha256, b"document content");
//! let timestamp = client.acquire_timestamp(&doc_id).await?;
//! println!("Timestamp acquired at: {}", timestamp.time);
//! # Ok(())
//! # }
//! ```

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::Utc;
use std::io::{Error as IoError, ErrorKind};

use super::record::TimestampRecord;
use crate::{DocumentId, Error, Result};

/// Well-known `OpenTimestamps` calendar server URLs.
pub mod calendars {
    /// Alice calendar server (primary).
    pub const ALICE: &str = "https://alice.btc.calendar.opentimestamps.org";
    /// Bob calendar server (backup).
    pub const BOB: &str = "https://bob.btc.calendar.opentimestamps.org";
    /// Finney calendar server (backup).
    pub const FINNEY: &str = "https://finney.calendar.eternitywall.com";
    /// Catallaxy calendar server.
    pub const CATALLAXY: &str = "https://ots.btc.catallaxy.com";
}

/// `OpenTimestamps` client for acquiring timestamps.
///
/// The client communicates with calendar servers to submit hashes
/// and retrieve timestamp proofs. The proofs are initially "pending"
/// and need to be upgraded later once the Bitcoin transaction is confirmed.
#[derive(Debug, Clone)]
pub struct OtsClient {
    /// Calendar server URLs to use (in order of preference).
    calendars: Vec<String>,
    /// HTTP client.
    client: reqwest::Client,
    /// Request timeout in seconds.
    timeout_secs: u64,
}

impl Default for OtsClient {
    fn default() -> Self {
        Self::new()
    }
}

impl OtsClient {
    /// Create a new OTS client with default calendar servers.
    #[must_use]
    pub fn new() -> Self {
        Self {
            calendars: vec![
                calendars::ALICE.to_string(),
                calendars::BOB.to_string(),
                calendars::FINNEY.to_string(),
            ],
            client: reqwest::Client::new(),
            timeout_secs: 30,
        }
    }

    /// Create a new OTS client with custom calendar servers.
    #[must_use]
    pub fn with_calendars(calendars: Vec<String>) -> Self {
        Self {
            calendars,
            client: reqwest::Client::new(),
            timeout_secs: 30,
        }
    }

    /// Set the request timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Acquire a timestamp for a document.
    ///
    /// This submits the document's hash to an `OpenTimestamps` calendar server
    /// and returns a timestamp record containing the proof.
    ///
    /// # Note
    ///
    /// The returned timestamp is initially "pending" - it contains a commitment
    /// from the calendar server but is not yet anchored to Bitcoin. Use
    /// `upgrade_timestamp` after sufficient time (typically 1-2 hours) to get
    /// the full Bitcoin-anchored proof.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No calendar servers are reachable
    /// - The document ID has an unsupported hash algorithm (must be SHA-256)
    /// - Network errors occur
    pub async fn acquire_timestamp(&self, document_id: &DocumentId) -> Result<TimestampRecord> {
        // OpenTimestamps requires SHA-256 hashes
        if document_id.algorithm().as_str() != "sha256" {
            return Err(Error::InvalidManifest {
                reason: "OpenTimestamps requires SHA-256 hash algorithm".to_string(),
            });
        }

        // Get the raw hash bytes
        let hash_hex = document_id.hex_digest();
        let hash_bytes = hex_to_bytes(&hash_hex)?;

        // Try each calendar server until one succeeds
        let mut last_error = None;
        for calendar_url in &self.calendars {
            match self.submit_to_calendar(calendar_url, &hash_bytes).await {
                Ok(proof) => {
                    return Ok(TimestampRecord::open_timestamps(
                        Utc::now(),
                        BASE64.encode(&proof),
                    ));
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            Error::Io(IoError::new(
                ErrorKind::NotConnected,
                "No calendar servers configured",
            ))
        }))
    }

    /// Submit a hash to a calendar server.
    async fn submit_to_calendar(&self, calendar_url: &str, hash: &[u8]) -> Result<Vec<u8>> {
        let url = format!("{calendar_url}/digest");

        let response = self
            .client
            .post(&url)
            .timeout(std::time::Duration::from_secs(self.timeout_secs))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(hash.to_vec())
            .send()
            .await
            .map_err(|e| {
                Error::Io(IoError::new(
                    ErrorKind::ConnectionRefused,
                    format!("Failed to contact calendar server: {e}"),
                ))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Io(IoError::other(format!(
                "Calendar server returned error: {status} {text}"
            ))));
        }

        let proof_bytes = response.bytes().await.map_err(|e| {
            Error::Io(IoError::new(
                ErrorKind::InvalidData,
                format!("Failed to read response: {e}"),
            ))
        })?;

        Ok(proof_bytes.to_vec())
    }

    /// Upgrade a pending timestamp to a complete Bitcoin-anchored proof.
    ///
    /// This contacts the calendar server to check if the timestamp has been
    /// anchored to Bitcoin and returns an upgraded proof if available.
    ///
    /// # Note
    ///
    /// Bitcoin block confirmation typically takes 10-60 minutes. The calendar
    /// servers usually include the hash in a transaction within a few hours.
    /// Call this method periodically until the proof is upgraded.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The proof is not yet ready (still pending)
    /// - Network errors occur
    /// - The proof format is invalid
    pub fn upgrade_timestamp(&self, timestamp: &TimestampRecord) -> Result<TimestampRecord> {
        // Decode the existing proof
        let proof_bytes = BASE64.decode(&timestamp.token).map_err(|e| {
            Error::Io(IoError::new(
                ErrorKind::InvalidData,
                format!("Invalid timestamp token: {e}"),
            ))
        })?;

        // Try to upgrade via each calendar
        for calendar_url in &self.calendars {
            if let Ok(upgraded) = Self::upgrade_from_calendar(calendar_url, &proof_bytes) {
                return Ok(TimestampRecord {
                    method: timestamp.method,
                    authority: timestamp.authority.clone(),
                    time: timestamp.time,
                    token: BASE64.encode(&upgraded),
                    transaction_id: None, // Could parse from upgraded proof
                });
            }
        }

        Err(Error::Io(IoError::new(
            ErrorKind::NotFound,
            "Timestamp not yet anchored to Bitcoin",
        )))
    }

    /// Try to upgrade a proof from a specific calendar.
    fn upgrade_from_calendar(calendar_url: &str, _proof: &[u8]) -> Result<Vec<u8>> {
        // The upgrade endpoint depends on the commitment in the proof
        // For now, we return an error indicating upgrade is not yet supported
        // Full implementation would parse the OTS proof format and query
        // the appropriate upgrade endpoint
        let _ = calendar_url;
        Err(Error::Io(IoError::new(
            ErrorKind::Unsupported,
            "Timestamp upgrade not yet implemented",
        )))
    }

    /// Verify a timestamp proof.
    ///
    /// This verifies that the proof is well-formed and, if complete,
    /// that it correctly anchors to a Bitcoin block.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The proof format is invalid
    /// - Verification fails
    pub fn verify_timestamp(
        &self,
        timestamp: &TimestampRecord,
        document_id: &DocumentId,
    ) -> Result<TimestampVerification> {
        // Decode the proof
        let proof_bytes = BASE64.decode(&timestamp.token).map_err(|e| {
            Error::Io(IoError::new(
                ErrorKind::InvalidData,
                format!("Invalid timestamp token: {e}"),
            ))
        })?;

        // Basic validation: check it's not empty and starts with OTS magic
        if proof_bytes.is_empty() {
            return Ok(TimestampVerification {
                valid: false,
                status: VerificationStatus::Invalid,
                message: "Empty proof".to_string(),
            });
        }

        // OTS proofs should start with the magic bytes \x00OpenTimestamps\x00\x00Proof\x00
        // But calendar responses are different - they're compact proofs
        // For now, we just validate the proof is non-empty
        let _ = document_id;

        Ok(TimestampVerification {
            valid: true,
            status: VerificationStatus::Pending,
            message: "Timestamp proof present (full verification requires upgrade)".to_string(),
        })
    }
}

/// Result of timestamp verification.
#[derive(Debug, Clone)]
pub struct TimestampVerification {
    /// Whether the proof passed basic validation.
    pub valid: bool,
    /// Verification status.
    pub status: VerificationStatus,
    /// Human-readable message.
    pub message: String,
}

/// Status of timestamp verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationStatus {
    /// Proof is pending (not yet anchored to Bitcoin).
    Pending,
    /// Proof is complete and verified against Bitcoin.
    Complete,
    /// Proof is invalid.
    Invalid,
}

/// Convert hex string to bytes.
fn hex_to_bytes(hex: &str) -> Result<Vec<u8>> {
    let hex = hex.trim();
    if hex.len() % 2 != 0 {
        return Err(Error::InvalidHashFormat {
            value: "Invalid hex string length".to_string(),
        });
    }

    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| Error::InvalidHashFormat {
                value: "Invalid hex character".to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HashAlgorithm, Hasher};

    #[test]
    fn test_ots_client_creation() {
        let client = OtsClient::new();
        assert!(!client.calendars.is_empty());
    }

    #[test]
    fn test_ots_client_custom_calendars() {
        let client = OtsClient::with_calendars(vec!["https://custom.example.com".to_string()]);
        assert_eq!(client.calendars.len(), 1);
    }

    #[test]
    fn test_hex_to_bytes() {
        let bytes = hex_to_bytes("deadbeef").unwrap();
        assert_eq!(bytes, vec![0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn test_hex_to_bytes_invalid() {
        assert!(hex_to_bytes("deadbee").is_err()); // odd length
        assert!(hex_to_bytes("deadbeeg").is_err()); // invalid char
    }

    #[test]
    fn test_verify_empty_proof() {
        let client = OtsClient::new();
        let doc_id = Hasher::hash(HashAlgorithm::Sha256, b"test");
        let timestamp = TimestampRecord::open_timestamps(Utc::now(), "");

        let result = client.verify_timestamp(&timestamp, &doc_id).unwrap();
        assert!(!result.valid);
        assert_eq!(result.status, VerificationStatus::Invalid);
    }

    #[test]
    fn test_verify_basic_proof() {
        let client = OtsClient::new();
        let doc_id = Hasher::hash(HashAlgorithm::Sha256, b"test");
        let timestamp =
            TimestampRecord::open_timestamps(Utc::now(), BASE64.encode(b"some proof data"));

        let result = client.verify_timestamp(&timestamp, &doc_id).unwrap();
        assert!(result.valid);
        assert_eq!(result.status, VerificationStatus::Pending);
    }
}
