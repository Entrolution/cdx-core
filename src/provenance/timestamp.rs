//! RFC 3161 timestamp token support.
//!
//! This module provides types for timestamp anchoring, allowing
//! documents to be cryptographically anchored to a specific point in time.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::DocumentId;

/// A timestamp request to be sent to a Time Stamp Authority (TSA).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimestampRequest {
    /// Version of the request format.
    pub version: u32,

    /// Hash of the data to be timestamped.
    pub message_imprint: MessageImprint,

    /// Optional nonce for replay protection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,

    /// Whether to include the TSA certificate in the response.
    #[serde(default)]
    pub cert_req: bool,
}

impl TimestampRequest {
    /// Create a new timestamp request for a document.
    #[must_use]
    pub fn for_document(document_id: &DocumentId) -> Self {
        Self {
            version: 1,
            message_imprint: MessageImprint {
                hash_algorithm: document_id.algorithm().as_str().to_string(),
                hashed_message: document_id.hex_digest().clone(),
            },
            nonce: None,
            cert_req: true,
        }
    }

    /// Set a nonce for replay protection.
    #[must_use]
    pub fn with_nonce(mut self, nonce: impl Into<String>) -> Self {
        self.nonce = Some(nonce.into());
        self
    }
}

/// Hash of the message being timestamped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageImprint {
    /// Hash algorithm OID or name.
    pub hash_algorithm: String,

    /// Hex-encoded hash value.
    pub hashed_message: String,
}

/// Response from a Time Stamp Authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimestampResponse {
    /// Status of the request.
    pub status: TimestampStatus,

    /// The timestamp token (if successful).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<TimestampToken>,
}

/// Status of a timestamp request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimestampStatus {
    /// Status code (0 = granted).
    pub status: u32,

    /// Status string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_string: Option<String>,

    /// Failure info if request failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fail_info: Option<String>,
}

impl TimestampStatus {
    /// Check if the request was granted.
    #[must_use]
    pub fn is_granted(&self) -> bool {
        self.status == 0
    }
}

/// A timestamp token from a TSA.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimestampToken {
    /// Version of the token format.
    pub version: u32,

    /// Policy OID under which the token was issued.
    pub policy: String,

    /// The message imprint that was timestamped.
    pub message_imprint: MessageImprint,

    /// Serial number of this token.
    pub serial_number: String,

    /// Time when the timestamp was generated.
    pub gen_time: DateTime<Utc>,

    /// Accuracy of the timestamp (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accuracy: Option<TimestampAccuracy>,

    /// Ordering flag.
    #[serde(default)]
    pub ordering: bool,

    /// Nonce (if provided in request).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,

    /// TSA name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tsa: Option<String>,

    /// Base64-encoded signature over the token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,

    /// Base64-encoded DER token (for verification).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_token: Option<String>,
}

impl TimestampToken {
    /// Verify that this token matches a document.
    ///
    /// Note: This only checks the message imprint, not the cryptographic signature.
    /// Full verification requires the TSA's certificate chain.
    #[must_use]
    pub fn matches_document(&self, document_id: &DocumentId) -> bool {
        self.message_imprint.hashed_message == document_id.hex_digest()
    }

    /// Get the timestamp time.
    #[must_use]
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.gen_time
    }
}

/// Accuracy of a timestamp.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimestampAccuracy {
    /// Seconds component.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds: Option<u32>,

    /// Milliseconds component.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub millis: Option<u32>,

    /// Microseconds component.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub micros: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HashAlgorithm, Hasher};

    #[test]
    fn test_timestamp_request_creation() {
        let doc_id = Hasher::hash(HashAlgorithm::Sha256, b"test document");
        let request = TimestampRequest::for_document(&doc_id);

        assert_eq!(request.version, 1);
        assert!(request.cert_req);
        assert_eq!(request.message_imprint.hashed_message, doc_id.hex_digest());
    }

    #[test]
    fn test_timestamp_request_with_nonce() {
        let doc_id = Hasher::hash(HashAlgorithm::Sha256, b"test");
        let request = TimestampRequest::for_document(&doc_id).with_nonce("abc123");

        assert_eq!(request.nonce, Some("abc123".to_string()));
    }

    #[test]
    fn test_timestamp_status_granted() {
        let status = TimestampStatus {
            status: 0,
            status_string: Some("Granted".to_string()),
            fail_info: None,
        };

        assert!(status.is_granted());
    }

    #[test]
    fn test_timestamp_status_rejected() {
        let status = TimestampStatus {
            status: 2,
            status_string: Some("Rejection".to_string()),
            fail_info: Some("Bad algorithm".to_string()),
        };

        assert!(!status.is_granted());
    }

    #[test]
    fn test_timestamp_token_matches() {
        let doc_id = Hasher::hash(HashAlgorithm::Sha256, b"document");

        let token = TimestampToken {
            version: 1,
            policy: "1.2.3.4".to_string(),
            message_imprint: MessageImprint {
                hash_algorithm: "SHA-256".to_string(),
                hashed_message: doc_id.hex_digest(),
            },
            serial_number: "12345".to_string(),
            gen_time: Utc::now(),
            accuracy: None,
            ordering: false,
            nonce: None,
            tsa: Some("Example TSA".to_string()),
            signature: None,
            raw_token: None,
        };

        assert!(token.matches_document(&doc_id));
    }

    #[test]
    fn test_timestamp_serialization() {
        let doc_id = Hasher::hash(HashAlgorithm::Sha256, b"test");
        let request = TimestampRequest::for_document(&doc_id);

        let json = serde_json::to_string_pretty(&request).unwrap();
        assert!(json.contains("\"version\": 1"));
        assert!(json.contains("\"certReq\": true"));

        let deserialized: TimestampRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(
            deserialized.message_imprint.hashed_message,
            request.message_imprint.hashed_message
        );
    }
}
