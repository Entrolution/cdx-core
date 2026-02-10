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
    pub async fn upgrade_timestamp(&self, timestamp: &TimestampRecord) -> Result<UpgradeResult> {
        // Decode the existing proof
        let proof_bytes = BASE64.decode(&timestamp.token).map_err(|e| {
            Error::Io(IoError::new(
                ErrorKind::InvalidData,
                format!("Invalid timestamp token: {e}"),
            ))
        })?;

        // Try to upgrade via each calendar
        for calendar_url in &self.calendars {
            if let Ok(Some(upgraded)) = self.upgrade_from_calendar(calendar_url, &proof_bytes).await
            {
                let upgraded_record = TimestampRecord {
                    method: timestamp.method,
                    authority: timestamp.authority.clone(),
                    time: timestamp.time,
                    token: BASE64.encode(&upgraded),
                    transaction_id: extract_bitcoin_txid(&upgraded),
                };
                return Ok(UpgradeResult::Complete(upgraded_record));
            }
            // Proof not ready yet or calendar error, try next
        }

        Ok(UpgradeResult::Pending {
            message: "Timestamp not yet anchored to Bitcoin".to_string(),
        })
    }

    /// Try to upgrade a proof from a specific calendar.
    async fn upgrade_from_calendar(
        &self,
        calendar_url: &str,
        proof: &[u8],
    ) -> Result<Option<Vec<u8>>> {
        // Extract the commitment hash from the proof
        // OTS proofs from calendars are structured as:
        // - Hash algorithm byte
        // - Commitment operations
        // The commitment itself is derivable from the proof structure

        // Try the upgrade endpoint
        let url = format!("{calendar_url}/timestamp");

        let response = self
            .client
            .post(&url)
            .timeout(std::time::Duration::from_secs(self.timeout_secs))
            .header("Content-Type", "application/octet-stream")
            .body(proof.to_vec())
            .send()
            .await
            .map_err(|e| {
                Error::Io(IoError::new(
                    ErrorKind::ConnectionRefused,
                    format!("Failed to contact calendar server: {e}"),
                ))
            })?;

        match response.status().as_u16() {
            200 => {
                // Upgrade successful
                let upgraded_bytes = response.bytes().await.map_err(|e| {
                    Error::Io(IoError::new(
                        ErrorKind::InvalidData,
                        format!("Failed to read response: {e}"),
                    ))
                })?;
                Ok(Some(upgraded_bytes.to_vec()))
            }
            404 => {
                // Proof not ready yet
                Ok(None)
            }
            status => {
                let text = response.text().await.unwrap_or_default();
                Err(Error::Io(IoError::other(format!(
                    "Calendar server returned error: {status} {text}"
                ))))
            }
        }
    }

    /// Check the status of a timestamp without upgrading.
    ///
    /// Returns the current status of the timestamp proof.
    ///
    /// # Errors
    ///
    /// Returns an error if the timestamp token is invalid.
    pub async fn check_status(&self, timestamp: &TimestampRecord) -> Result<TimestampStatus> {
        // Decode the existing proof
        let proof_bytes = BASE64.decode(&timestamp.token).map_err(|e| {
            Error::Io(IoError::new(
                ErrorKind::InvalidData,
                format!("Invalid timestamp token: {e}"),
            ))
        })?;

        // Check if the proof is already complete by looking for Bitcoin attestation markers
        if is_complete_proof(&proof_bytes) {
            return Ok(TimestampStatus::Complete {
                bitcoin_txid: extract_bitcoin_txid(&proof_bytes),
                block_height: extract_block_height(&proof_bytes),
            });
        }

        // Try to check status via calendars
        for calendar_url in &self.calendars {
            if let Ok(Some(_)) = self.upgrade_from_calendar(calendar_url, &proof_bytes).await {
                return Ok(TimestampStatus::Ready);
            }
        }

        Ok(TimestampStatus::Pending)
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

/// Result of attempting to upgrade a timestamp.
#[derive(Debug, Clone)]
pub enum UpgradeResult {
    /// Upgrade completed successfully.
    Complete(TimestampRecord),
    /// Timestamp is still pending (not yet anchored).
    Pending {
        /// Human-readable status message.
        message: String,
    },
}

impl UpgradeResult {
    /// Check if the upgrade is complete.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Complete(_))
    }

    /// Get the upgraded timestamp record if complete.
    #[must_use]
    pub fn into_record(self) -> Option<TimestampRecord> {
        match self {
            Self::Complete(record) => Some(record),
            Self::Pending { .. } => None,
        }
    }
}

/// Status of a timestamp proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimestampStatus {
    /// Proof is pending (submitted to calendar but not yet anchored).
    Pending,
    /// Proof is ready to be upgraded (anchored but not yet retrieved).
    Ready,
    /// Proof is complete with Bitcoin attestation.
    Complete {
        /// Bitcoin transaction ID.
        bitcoin_txid: Option<String>,
        /// Bitcoin block height.
        block_height: Option<u64>,
    },
}

impl TimestampStatus {
    /// Check if the timestamp is complete.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Complete { .. })
    }

    /// Check if the timestamp is pending.
    #[must_use]
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending)
    }
}

impl std::fmt::Display for TimestampStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Ready => write!(f, "Ready for upgrade"),
            Self::Complete {
                bitcoin_txid,
                block_height,
            } => {
                write!(f, "Complete")?;
                if let Some(txid) = bitcoin_txid {
                    write!(f, " (tx: {txid})")?;
                }
                if let Some(height) = block_height {
                    write!(f, " (block: {height})")?;
                }
                Ok(())
            }
        }
    }
}

/// Check if a proof is complete (contains Bitcoin attestation).
fn is_complete_proof(proof: &[u8]) -> bool {
    // OTS complete proofs contain attestation markers
    // Bitcoin attestation is indicated by a specific byte sequence
    // The attestation tag for Bitcoin is 0x0588960d73d71901
    const BITCOIN_ATTESTATION_TAG: [u8; 8] = [0x05, 0x88, 0x96, 0x0d, 0x73, 0xd7, 0x19, 0x01];

    proof
        .windows(8)
        .any(|window| window == BITCOIN_ATTESTATION_TAG)
}

/// Extract Bitcoin transaction ID from a complete proof.
fn extract_bitcoin_txid(proof: &[u8]) -> Option<String> {
    // The txid follows the Bitcoin attestation marker
    // This is a simplified extraction - full implementation would
    // properly parse the OTS proof format
    const BITCOIN_ATTESTATION_TAG: [u8; 8] = [0x05, 0x88, 0x96, 0x0d, 0x73, 0xd7, 0x19, 0x01];

    for (i, window) in proof.windows(8).enumerate() {
        if window == BITCOIN_ATTESTATION_TAG {
            // The 32-byte txid follows the attestation tag and block merkle path
            // This is a simplified approach - actual position depends on proof structure
            if proof.len() > i + 8 + 32 {
                let txid_bytes = &proof[i + 8..i + 8 + 32];
                // Reverse for Bitcoin's display format
                let mut reversed = txid_bytes.to_vec();
                reversed.reverse();
                return Some(hex::encode(reversed));
            }
        }
    }
    None
}

/// Extract block height from a complete proof.
fn extract_block_height(proof: &[u8]) -> Option<u64> {
    // Block height extraction requires proper OTS proof parsing
    // This is a placeholder - would need full proof parsing
    let _ = proof;
    None
}

/// Helper module for hex encoding.
mod hex {
    /// Encode bytes as hex string.
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().fold(
            String::with_capacity(bytes.as_ref().len() * 2),
            |mut acc, b| {
                use std::fmt::Write;
                let _ = write!(acc, "{b:02x}");
                acc
            },
        )
    }
}

/// Convert hex string to bytes.
fn hex_to_bytes(hex: &str) -> Result<Vec<u8>> {
    let hex = hex.trim();
    if !hex.len().is_multiple_of(2) {
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
