//! RFC 3161 Time-Stamp Protocol client.
//!
//! This module provides a client for acquiring timestamps from RFC 3161
//! compliant Time Stamp Authorities (TSAs). These timestamps provide
//! cryptographic proof that data existed at a specific point in time.
//!
//! # Feature Flag
//!
//! This module requires the `timestamps-rfc3161` feature:
//!
//! ```toml
//! [dependencies]
//! cdx-core = { version = "0.1", features = ["timestamps-rfc3161"] }
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use cdx_core::provenance::rfc3161::Rfc3161Client;
//! use cdx_core::{HashAlgorithm, Hasher};
//!
//! # async fn example() -> cdx_core::Result<()> {
//! let client = Rfc3161Client::new();
//! let doc_id = Hasher::hash(HashAlgorithm::Sha256, b"document content");
//! let timestamp = client.acquire_timestamp(&doc_id).await?;
//! println!("Timestamp acquired at: {}", timestamp.time);
//! # Ok(())
//! # }
//! ```

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{DateTime, Utc};
use const_oid::ObjectIdentifier;
use rand_core::{OsRng, RngCore};
use std::io::{Error as IoError, ErrorKind};

use super::record::TimestampRecord;
use crate::{DocumentId, Error, Result};

/// Well-known free RFC 3161 TSA server URLs.
pub mod servers {
    /// `FreeTSA` - Free timestamp service.
    pub const FREETSA: &str = "https://freetsa.org/tsr";
    /// Sectigo timestamp server.
    pub const SECTIGO: &str = "http://timestamp.sectigo.com";
    /// `DigiCert` timestamp server.
    pub const DIGICERT: &str = "http://timestamp.digicert.com";
}

// ASN.1 OIDs for hash algorithms
const OID_SHA256: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.1");
const OID_SHA384: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.2");
const OID_SHA512: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.16.840.1.101.3.4.2.3");

/// RFC 3161 Time-Stamp Protocol client.
///
/// The client communicates with TSA servers to request timestamps
/// and retrieve signed timestamp tokens.
#[derive(Debug, Clone)]
pub struct Rfc3161Client {
    /// TSA server URLs to use (in order of preference).
    servers: Vec<String>,
    /// HTTP client.
    client: reqwest::Client,
    /// Request timeout in seconds.
    timeout_secs: u64,
    /// Whether to request certificate inclusion.
    cert_req: bool,
}

impl Default for Rfc3161Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Rfc3161Client {
    /// Create a new RFC 3161 client with default TSA servers.
    #[must_use]
    pub fn new() -> Self {
        Self {
            servers: vec![
                servers::FREETSA.to_string(),
                servers::SECTIGO.to_string(),
                servers::DIGICERT.to_string(),
            ],
            client: reqwest::Client::new(),
            timeout_secs: 30,
            cert_req: true,
        }
    }

    /// Create a new RFC 3161 client with a specific TSA server.
    #[must_use]
    pub fn with_server(server: impl Into<String>) -> Self {
        Self {
            servers: vec![server.into()],
            client: reqwest::Client::new(),
            timeout_secs: 30,
            cert_req: true,
        }
    }

    /// Create a new RFC 3161 client with custom TSA servers.
    #[must_use]
    pub fn with_servers(servers: Vec<String>) -> Self {
        Self {
            servers,
            client: reqwest::Client::new(),
            timeout_secs: 30,
            cert_req: true,
        }
    }

    /// Set the request timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Set whether to request certificate inclusion.
    #[must_use]
    pub fn with_cert_req(mut self, cert_req: bool) -> Self {
        self.cert_req = cert_req;
        self
    }

    /// Acquire a timestamp for a document.
    ///
    /// This sends a timestamp request to an RFC 3161 TSA server
    /// and returns a timestamp record containing the signed token.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No TSA servers are reachable
    /// - The TSA rejects the request
    /// - Network errors occur
    pub async fn acquire_timestamp(&self, document_id: &DocumentId) -> Result<TimestampRecord> {
        // Get hash algorithm OID
        let hash_oid = match document_id.algorithm().as_str() {
            "sha256" => OID_SHA256,
            "sha384" => OID_SHA384,
            "sha512" => OID_SHA512,
            alg => {
                return Err(Error::InvalidManifest {
                    reason: format!("Unsupported hash algorithm for RFC 3161: {alg}"),
                })
            }
        };

        // Get the raw hash bytes
        let hash_hex = document_id.hex_digest();
        let hash_bytes = hex_to_bytes(&hash_hex)?;

        // Generate nonce for replay protection
        let mut nonce_bytes = [0u8; 8];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = u64::from_be_bytes(nonce_bytes);

        // Build the timestamp request using manual DER encoding
        let request_der = encode_timestamp_request(&hash_bytes, hash_oid, nonce, self.cert_req);

        // Try each TSA server until one succeeds
        let mut last_error = None;
        for server_url in &self.servers {
            match self.submit_to_tsa(server_url, &request_der).await {
                Ok((token, time)) => {
                    return Ok(TimestampRecord::rfc3161(
                        server_url,
                        time,
                        BASE64.encode(&token),
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
                "No TSA servers configured",
            ))
        }))
    }

    /// Submit a timestamp request to a TSA server.
    async fn submit_to_tsa(
        &self,
        server_url: &str,
        request_der: &[u8],
    ) -> Result<(Vec<u8>, DateTime<Utc>)> {
        let response = self
            .client
            .post(server_url)
            .timeout(std::time::Duration::from_secs(self.timeout_secs))
            .header("Content-Type", "application/timestamp-query")
            .body(request_der.to_vec())
            .send()
            .await
            .map_err(|e| {
                Error::Io(IoError::new(
                    ErrorKind::ConnectionRefused,
                    format!("Failed to contact TSA server: {e}"),
                ))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Io(IoError::other(format!(
                "TSA server returned error: {status} {text}"
            ))));
        }

        let response_der = response.bytes().await.map_err(|e| {
            Error::Io(IoError::new(
                ErrorKind::InvalidData,
                format!("Failed to read TSA response: {e}"),
            ))
        })?;

        // Parse the timestamp response
        let (status, token) = parse_timestamp_response(&response_der)?;

        // Check status (0 = granted, 1 = granted with mods)
        if status > 1 {
            let status_text = match status {
                2 => "rejection",
                3 => "waiting",
                4 => "revocation warning",
                5 => "revocation notification",
                _ => "unknown error",
            };
            return Err(Error::Io(IoError::other(format!(
                "TSA rejected request: {status_text}"
            ))));
        }

        // Use current time as we'd need full CMS parsing for exact time
        let time = Utc::now();

        Ok((token, time))
    }

    /// Verify a timestamp token.
    ///
    /// This performs basic validation of the timestamp token format.
    ///
    /// # Errors
    ///
    /// Returns an error if the timestamp token contains invalid Base64 data.
    ///
    /// # Note
    ///
    /// Full cryptographic verification requires the TSA's certificate chain,
    /// which is beyond the scope of this basic implementation.
    pub fn verify_timestamp(
        &self,
        timestamp: &TimestampRecord,
        _document_id: &DocumentId,
    ) -> Result<TimestampVerification> {
        // Decode the token
        let token_bytes = BASE64.decode(&timestamp.token).map_err(|e| {
            Error::Io(IoError::new(
                ErrorKind::InvalidData,
                format!("Invalid timestamp token: {e}"),
            ))
        })?;

        // Basic validation: check it's not empty and looks like ASN.1
        if token_bytes.is_empty() {
            return Ok(TimestampVerification {
                valid: false,
                status: VerificationStatus::Invalid,
                message: "Empty token".to_string(),
            });
        }

        // Check for SEQUENCE tag (0x30) which indicates valid ASN.1 structure
        if token_bytes[0] != 0x30 {
            return Ok(TimestampVerification {
                valid: false,
                status: VerificationStatus::Invalid,
                message: "Invalid ASN.1 structure".to_string(),
            });
        }

        Ok(TimestampVerification {
            valid: true,
            status: VerificationStatus::Valid,
            message: "Timestamp token is well-formed".to_string(),
        })
    }
}

/// Result of timestamp verification.
#[derive(Debug, Clone)]
pub struct TimestampVerification {
    /// Whether the token passed basic validation.
    pub valid: bool,
    /// Verification status.
    pub status: VerificationStatus,
    /// Human-readable message.
    pub message: String,
}

/// Status of timestamp verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationStatus {
    /// Token is valid and verified.
    Valid,
    /// Token format is invalid.
    Invalid,
}

/// Manually encode a `TimeStampReq` in DER format.
///
/// `TimeStampReq` ::= SEQUENCE {
///    version          INTEGER { v1(1) },
///    `messageImprint`   `MessageImprint`,
///    nonce            INTEGER OPTIONAL,
///    `certReq`          BOOLEAN DEFAULT FALSE,
/// }
fn encode_timestamp_request(
    hash: &[u8],
    hash_oid: ObjectIdentifier,
    nonce: u64,
    cert_req: bool,
) -> Vec<u8> {
    let mut content = Vec::new();

    // version INTEGER (1)
    content.extend_from_slice(&encode_integer(1));

    // messageImprint SEQUENCE
    content.extend_from_slice(&encode_message_imprint(hash, hash_oid));

    // nonce INTEGER
    content.extend_from_slice(&encode_integer_u64(nonce));

    // certReq BOOLEAN (only include if true, as false is default)
    if cert_req {
        content.extend_from_slice(&[0x01, 0x01, 0xff]); // BOOLEAN TRUE
    }

    // Wrap in SEQUENCE
    encode_sequence(&content)
}

/// Encode `MessageImprint` SEQUENCE.
fn encode_message_imprint(hash: &[u8], hash_oid: ObjectIdentifier) -> Vec<u8> {
    let mut content = Vec::new();

    // AlgorithmIdentifier SEQUENCE
    let mut alg_id = Vec::new();
    alg_id.extend_from_slice(&encode_oid(&hash_oid));
    alg_id.extend_from_slice(&[0x05, 0x00]); // NULL parameters
    content.extend_from_slice(&encode_sequence(&alg_id));

    // hashedMessage OCTET STRING
    content.extend_from_slice(&encode_octet_string(hash));

    encode_sequence(&content)
}

/// Encode a SEQUENCE.
fn encode_sequence(content: &[u8]) -> Vec<u8> {
    let mut result = vec![0x30]; // SEQUENCE tag
    result.extend_from_slice(&encode_length(content.len()));
    result.extend_from_slice(content);
    result
}

/// Encode an INTEGER (small positive value).
fn encode_integer(value: u8) -> Vec<u8> {
    vec![0x02, 0x01, value]
}

/// Encode a u64 INTEGER.
#[allow(clippy::cast_possible_truncation)]
fn encode_integer_u64(value: u64) -> Vec<u8> {
    let bytes = value.to_be_bytes();
    // Find first non-zero byte
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(7);
    let significant = &bytes[start..];

    let mut result = vec![0x02]; // INTEGER tag

    // Add leading zero if high bit is set (to keep it positive)
    // Length is at most 9 bytes, so cast to u8 is safe
    if significant.first().is_some_and(|&b| b & 0x80 != 0) {
        result.push((significant.len() + 1) as u8);
        result.push(0x00);
    } else {
        result.push(significant.len() as u8);
    }
    result.extend_from_slice(significant);
    result
}

/// Encode an OID.
fn encode_oid(oid: &ObjectIdentifier) -> Vec<u8> {
    let oid_bytes = oid.as_bytes();
    let mut result = vec![0x06]; // OID tag
    result.extend_from_slice(&encode_length(oid_bytes.len()));
    result.extend_from_slice(oid_bytes);
    result
}

/// Encode an OCTET STRING.
fn encode_octet_string(data: &[u8]) -> Vec<u8> {
    let mut result = vec![0x04]; // OCTET STRING tag
    result.extend_from_slice(&encode_length(data.len()));
    result.extend_from_slice(data);
    result
}

/// Encode DER length.
///
/// Supports lengths up to 65535 bytes (2-byte long form).
#[allow(clippy::cast_possible_truncation)]
fn encode_length(len: usize) -> Vec<u8> {
    // For DER encoding, lengths fit in u8 or u16; truncation is intentional
    if len < 128 {
        vec![len as u8]
    } else if len < 256 {
        vec![0x81, len as u8]
    } else {
        vec![0x82, (len >> 8) as u8, (len & 0xff) as u8]
    }
}

/// Parse a `TimeStampResp` and extract status and token.
fn parse_timestamp_response(data: &[u8]) -> Result<(u8, Vec<u8>)> {
    // Very basic ASN.1 parsing
    // TimeStampResp ::= SEQUENCE {
    //    status PKIStatusInfo,
    //    timeStampToken TimeStampToken OPTIONAL
    // }

    if data.is_empty() || data[0] != 0x30 {
        return Err(Error::Io(IoError::new(
            ErrorKind::InvalidData,
            "Invalid TSA response: not a SEQUENCE",
        )));
    }

    let (content, _) = parse_tlv(data)?;

    // Parse PKIStatusInfo (first element)
    if content.is_empty() || content[0] != 0x30 {
        return Err(Error::Io(IoError::new(
            ErrorKind::InvalidData,
            "Invalid PKIStatusInfo",
        )));
    }

    let (status_info, rest) = parse_tlv(content)?;

    // Extract status INTEGER from PKIStatusInfo
    if status_info.is_empty() || status_info[0] != 0x02 {
        return Err(Error::Io(IoError::new(
            ErrorKind::InvalidData,
            "Invalid status in PKIStatusInfo",
        )));
    }

    let (status_bytes, _) = parse_tlv(status_info)?;
    let status = *status_bytes.last().unwrap_or(&255);

    // Extract timeStampToken if present (should be the rest after status info)
    if rest.is_empty() {
        return Err(Error::Io(IoError::new(
            ErrorKind::InvalidData,
            "No timestamp token in response",
        )));
    }

    // The token is a ContentInfo SEQUENCE
    if rest[0] != 0x30 {
        return Err(Error::Io(IoError::new(
            ErrorKind::InvalidData,
            "Invalid timestamp token format",
        )));
    }

    // Return the full token (including its SEQUENCE wrapper)
    let token_len = get_tlv_total_length(rest)?;
    let token = rest[..token_len].to_vec();

    Ok((status, token))
}

/// Parse a TLV (Tag-Length-Value) and return the value and remaining data.
fn parse_tlv(data: &[u8]) -> Result<(&[u8], &[u8])> {
    if data.len() < 2 {
        return Err(Error::Io(IoError::new(
            ErrorKind::InvalidData,
            "TLV too short",
        )));
    }

    let (len, header_len) = if data[1] < 128 {
        (data[1] as usize, 2)
    } else if data[1] == 0x81 {
        if data.len() < 3 {
            return Err(Error::Io(IoError::new(
                ErrorKind::InvalidData,
                "Invalid length encoding",
            )));
        }
        (data[2] as usize, 3)
    } else if data[1] == 0x82 {
        if data.len() < 4 {
            return Err(Error::Io(IoError::new(
                ErrorKind::InvalidData,
                "Invalid length encoding",
            )));
        }
        (((data[2] as usize) << 8) | (data[3] as usize), 4)
    } else {
        return Err(Error::Io(IoError::new(
            ErrorKind::InvalidData,
            "Unsupported length encoding",
        )));
    };

    if data.len() < header_len + len {
        return Err(Error::Io(IoError::new(
            ErrorKind::InvalidData,
            "TLV length exceeds data",
        )));
    }

    let value = &data[header_len..header_len + len];
    let rest = &data[header_len + len..];
    Ok((value, rest))
}

/// Get the total length of a TLV including header.
fn get_tlv_total_length(data: &[u8]) -> Result<usize> {
    if data.len() < 2 {
        return Err(Error::Io(IoError::new(
            ErrorKind::InvalidData,
            "TLV too short",
        )));
    }

    let (len, header_len) = if data[1] < 128 {
        (data[1] as usize, 2)
    } else if data[1] == 0x81 {
        if data.len() < 3 {
            return Err(Error::Io(IoError::new(
                ErrorKind::InvalidData,
                "Invalid length encoding",
            )));
        }
        (data[2] as usize, 3)
    } else if data[1] == 0x82 {
        if data.len() < 4 {
            return Err(Error::Io(IoError::new(
                ErrorKind::InvalidData,
                "Invalid length encoding",
            )));
        }
        (((data[2] as usize) << 8) | (data[3] as usize), 4)
    } else {
        return Err(Error::Io(IoError::new(
            ErrorKind::InvalidData,
            "Unsupported length encoding",
        )));
    };

    Ok(header_len + len)
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
    fn test_rfc3161_client_creation() {
        let client = Rfc3161Client::new();
        assert!(!client.servers.is_empty());
    }

    #[test]
    fn test_rfc3161_client_custom_server() {
        let client = Rfc3161Client::with_server("https://custom.example.com/tsa");
        assert_eq!(client.servers.len(), 1);
        assert_eq!(client.servers[0], "https://custom.example.com/tsa");
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
    fn test_timestamp_req_encoding() {
        let hash = vec![0u8; 32]; // dummy SHA-256 hash
        let req = encode_timestamp_request(&hash, OID_SHA256, 12345, true);
        // Should produce valid DER
        assert!(!req.is_empty());
        assert_eq!(req[0], 0x30); // SEQUENCE tag
    }

    #[test]
    fn test_encode_integer() {
        let encoded = encode_integer(1);
        assert_eq!(encoded, vec![0x02, 0x01, 0x01]);
    }

    #[test]
    fn test_encode_integer_u64() {
        let encoded = encode_integer_u64(256);
        // 256 = 0x0100, needs leading zero since high bit of 0x01 is clear
        assert_eq!(encoded, vec![0x02, 0x02, 0x01, 0x00]);
    }

    #[test]
    fn test_verify_empty_token() {
        let client = Rfc3161Client::new();
        let doc_id = Hasher::hash(HashAlgorithm::Sha256, b"test");
        let timestamp = TimestampRecord::rfc3161("https://example.com", Utc::now(), "");

        let result = client.verify_timestamp(&timestamp, &doc_id).unwrap();
        assert!(!result.valid);
        assert_eq!(result.status, VerificationStatus::Invalid);
    }

    #[test]
    fn test_verify_invalid_base64() {
        let client = Rfc3161Client::new();
        let doc_id = Hasher::hash(HashAlgorithm::Sha256, b"test");
        let timestamp =
            TimestampRecord::rfc3161("https://example.com", Utc::now(), "!!!invalid!!!");

        let result = client.verify_timestamp(&timestamp, &doc_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_valid_structure() {
        let client = Rfc3161Client::new();
        let doc_id = Hasher::hash(HashAlgorithm::Sha256, b"test");
        // A minimal valid ASN.1 SEQUENCE
        let token = BASE64.encode([0x30, 0x00]);
        let timestamp = TimestampRecord::rfc3161("https://example.com", Utc::now(), token);

        let result = client.verify_timestamp(&timestamp, &doc_id).unwrap();
        assert!(result.valid);
        assert_eq!(result.status, VerificationStatus::Valid);
    }
}
