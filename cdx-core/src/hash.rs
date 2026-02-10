//! Document hashing and content-addressable identity.
//!
//! Codex documents use content-addressable hashing as a core identity mechanism.
//! The document's hash serves as its canonical identifier.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha384, Sha512};
use sha3::{Sha3_256, Sha3_512};
use std::fmt;
use std::io::Read;
use std::str::FromStr;

use crate::{Error, Result};

/// Hash algorithm identifier.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, strum::Display,
)]
#[serde(rename_all = "lowercase")]
pub enum HashAlgorithm {
    /// SHA-256 (default, required).
    #[default]
    #[serde(rename = "sha256")]
    #[strum(serialize = "sha256")]
    Sha256,

    /// SHA-384 (optional).
    #[serde(rename = "sha384")]
    #[strum(serialize = "sha384")]
    Sha384,

    /// SHA-512 (optional).
    #[serde(rename = "sha512")]
    #[strum(serialize = "sha512")]
    Sha512,

    /// SHA3-256 (optional).
    #[serde(rename = "sha3-256")]
    #[strum(serialize = "sha3-256")]
    Sha3_256,

    /// SHA3-512 (optional).
    #[serde(rename = "sha3-512")]
    #[strum(serialize = "sha3-512")]
    Sha3_512,

    /// BLAKE3 (optional).
    #[serde(rename = "blake3")]
    #[strum(serialize = "blake3")]
    Blake3,
}

impl HashAlgorithm {
    /// Get the algorithm identifier string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Sha256 => "sha256",
            Self::Sha384 => "sha384",
            Self::Sha512 => "sha512",
            Self::Sha3_256 => "sha3-256",
            Self::Sha3_512 => "sha3-512",
            Self::Blake3 => "blake3",
        }
    }

    /// Get the output size in bytes.
    #[must_use]
    pub const fn output_size(&self) -> usize {
        match self {
            Self::Sha256 | Self::Sha3_256 | Self::Blake3 => 32,
            Self::Sha384 => 48,
            Self::Sha512 | Self::Sha3_512 => 64,
        }
    }
}

impl FromStr for HashAlgorithm {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "sha256" => Ok(Self::Sha256),
            "sha384" => Ok(Self::Sha384),
            "sha512" => Ok(Self::Sha512),
            "sha3-256" => Ok(Self::Sha3_256),
            "sha3-512" => Ok(Self::Sha3_512),
            "blake3" => Ok(Self::Blake3),
            _ => Err(Error::UnsupportedHashAlgorithm {
                algorithm: s.to_string(),
            }),
        }
    }
}

/// A content-addressable document identifier.
///
/// Format: `algorithm:hexdigest`
///
/// # Examples
///
/// ```
/// use cdx_core::{DocumentId, HashAlgorithm};
///
/// let id = DocumentId::new(HashAlgorithm::Sha256, vec![0xab; 32]);
/// assert!(id.to_string().starts_with("sha256:"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocumentId {
    algorithm: HashAlgorithm,
    digest: Vec<u8>,
}

impl DocumentId {
    /// Create a new document ID from algorithm and digest bytes.
    #[must_use]
    pub fn new(algorithm: HashAlgorithm, digest: Vec<u8>) -> Self {
        Self { algorithm, digest }
    }

    /// Get the hash algorithm.
    #[must_use]
    pub const fn algorithm(&self) -> HashAlgorithm {
        self.algorithm
    }

    /// Get the digest bytes.
    #[must_use]
    pub fn digest(&self) -> &[u8] {
        &self.digest
    }

    /// Get the hex-encoded digest.
    #[must_use]
    pub fn hex_digest(&self) -> String {
        hex_encode(&self.digest)
    }

    /// Check if this is a pending (placeholder) ID.
    #[must_use]
    pub fn is_pending(&self) -> bool {
        self.digest.is_empty()
    }

    /// Create a pending (placeholder) document ID.
    #[must_use]
    pub fn pending() -> Self {
        Self {
            algorithm: HashAlgorithm::default(),
            digest: Vec::new(),
        }
    }
}

impl fmt::Display for DocumentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_pending() {
            write!(f, "pending")
        } else {
            write!(f, "{}:{}", self.algorithm, self.hex_digest())
        }
    }
}

impl FromStr for DocumentId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s == "pending" {
            return Ok(Self::pending());
        }

        let (alg_str, hex_str) = s.split_once(':').ok_or_else(|| Error::InvalidHashFormat {
            value: s.to_string(),
        })?;

        let algorithm: HashAlgorithm = alg_str.parse()?;
        let digest = hex_decode(hex_str).map_err(|()| Error::InvalidHashFormat {
            value: s.to_string(),
        })?;

        // Validate digest length
        if digest.len() != algorithm.output_size() {
            return Err(Error::InvalidHashFormat {
                value: s.to_string(),
            });
        }

        Ok(Self { algorithm, digest })
    }
}

impl Serialize for DocumentId {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for DocumentId {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

/// Streaming hasher for computing document and file hashes.
pub struct Hasher {
    algorithm: HashAlgorithm,
    state: HasherState,
}

enum HasherState {
    Sha256(Sha256),
    Sha384(Sha384),
    Sha512(Sha512),
    Sha3_256(Sha3_256),
    Sha3_512(Sha3_512),
    Blake3(Box<blake3::Hasher>),
}

impl Hasher {
    /// Create a new hasher with the specified algorithm.
    #[must_use]
    pub fn new(algorithm: HashAlgorithm) -> Self {
        let state = match algorithm {
            HashAlgorithm::Sha256 => HasherState::Sha256(Sha256::new()),
            HashAlgorithm::Sha384 => HasherState::Sha384(Sha384::new()),
            HashAlgorithm::Sha512 => HasherState::Sha512(Sha512::new()),
            HashAlgorithm::Sha3_256 => HasherState::Sha3_256(Sha3_256::new()),
            HashAlgorithm::Sha3_512 => HasherState::Sha3_512(Sha3_512::new()),
            HashAlgorithm::Blake3 => HasherState::Blake3(Box::new(blake3::Hasher::new())),
        };
        Self { algorithm, state }
    }

    /// Create a new hasher with the default algorithm (SHA-256).
    #[must_use]
    pub fn default_algorithm() -> Self {
        Self::new(HashAlgorithm::default())
    }

    /// Update the hasher with data.
    pub fn update(&mut self, data: &[u8]) {
        match &mut self.state {
            HasherState::Sha256(h) => h.update(data),
            HasherState::Sha384(h) => h.update(data),
            HasherState::Sha512(h) => h.update(data),
            HasherState::Sha3_256(h) => h.update(data),
            HasherState::Sha3_512(h) => h.update(data),
            HasherState::Blake3(h) => {
                h.update(data);
            }
        }
    }

    /// Finalize the hash and return the document ID.
    #[must_use]
    pub fn finalize(self) -> DocumentId {
        let digest = match self.state {
            HasherState::Sha256(h) => h.finalize().to_vec(),
            HasherState::Sha384(h) => h.finalize().to_vec(),
            HasherState::Sha512(h) => h.finalize().to_vec(),
            HasherState::Sha3_256(h) => h.finalize().to_vec(),
            HasherState::Sha3_512(h) => h.finalize().to_vec(),
            HasherState::Blake3(h) => h.finalize().as_bytes().to_vec(),
        };
        DocumentId::new(self.algorithm, digest)
    }

    /// Hash data in one shot.
    #[must_use]
    pub fn hash(algorithm: HashAlgorithm, data: &[u8]) -> DocumentId {
        let mut hasher = Self::new(algorithm);
        hasher.update(data);
        hasher.finalize()
    }

    /// Hash data from a reader.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from the reader fails.
    pub fn hash_reader<R: Read>(algorithm: HashAlgorithm, reader: &mut R) -> Result<DocumentId> {
        let mut hasher = Self::new(algorithm);
        let mut buffer = [0u8; 8192];
        loop {
            let n = reader.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        Ok(hasher.finalize())
    }
}

/// Encode bytes as lowercase hexadecimal.
fn hex_encode(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        })
}

/// Decode hexadecimal string to bytes.
fn hex_decode(s: &str) -> std::result::Result<Vec<u8>, ()> {
    if s.len() % 2 != 0 {
        return Err(());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| ()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hash() {
        let id = Hasher::hash(HashAlgorithm::Sha256, b"hello world");
        assert_eq!(id.algorithm(), HashAlgorithm::Sha256);
        assert_eq!(
            id.hex_digest(),
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_document_id_parsing() {
        let id_str = "sha256:b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        let id: DocumentId = id_str.parse().unwrap();
        assert_eq!(id.algorithm(), HashAlgorithm::Sha256);
        assert_eq!(id.to_string(), id_str);
    }

    #[test]
    fn test_pending_id() {
        let id = DocumentId::pending();
        assert!(id.is_pending());
        assert_eq!(id.to_string(), "pending");

        let parsed: DocumentId = "pending".parse().unwrap();
        assert!(parsed.is_pending());
    }

    #[test]
    fn test_invalid_hash_format() {
        assert!("invalid".parse::<DocumentId>().is_err());
        assert!("sha256:xyz".parse::<DocumentId>().is_err());
        assert!("sha256:ab".parse::<DocumentId>().is_err()); // Too short
    }

    #[test]
    fn test_blake3_hash() {
        let id = Hasher::hash(HashAlgorithm::Blake3, b"hello world");
        assert_eq!(id.algorithm(), HashAlgorithm::Blake3);
        assert_eq!(id.digest().len(), 32);
    }

    #[test]
    fn test_streaming_hash() {
        let mut hasher = Hasher::new(HashAlgorithm::Sha256);
        hasher.update(b"hello ");
        hasher.update(b"world");
        let id = hasher.finalize();

        let direct = Hasher::hash(HashAlgorithm::Sha256, b"hello world");
        assert_eq!(id, direct);
    }

    #[test]
    fn test_serialization() {
        let id = Hasher::hash(HashAlgorithm::Sha256, b"test");
        let json = serde_json::to_string(&id).unwrap();
        let parsed: DocumentId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, parsed);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// Hashing is deterministic - same input always produces same output.
        #[test]
        fn hash_is_deterministic(data: Vec<u8>) {
            let h1 = Hasher::hash(HashAlgorithm::Sha256, &data);
            let h2 = Hasher::hash(HashAlgorithm::Sha256, &data);
            prop_assert_eq!(h1, h2);
        }

        /// DocumentId roundtrip through to_string and parse.
        #[test]
        fn document_id_roundtrip(data: Vec<u8>) {
            let original = Hasher::hash(HashAlgorithm::Sha256, &data);
            let serialized = original.to_string();
            let parsed: DocumentId = serialized.parse().unwrap();
            prop_assert_eq!(original, parsed);
        }

        /// Different inputs produce different hashes (with overwhelming probability).
        #[test]
        fn different_inputs_different_hashes(a: Vec<u8>, b: Vec<u8>) {
            prop_assume!(a != b);
            let h1 = Hasher::hash(HashAlgorithm::Sha256, &a);
            let h2 = Hasher::hash(HashAlgorithm::Sha256, &b);
            prop_assert_ne!(h1, h2);
        }

        /// Streaming hash equals one-shot hash.
        #[test]
        fn streaming_equals_oneshot(data: Vec<u8>) {
            let oneshot = Hasher::hash(HashAlgorithm::Sha256, &data);

            let mut streaming = Hasher::new(HashAlgorithm::Sha256);
            streaming.update(&data);
            let result = streaming.finalize();

            prop_assert_eq!(oneshot, result);
        }

        /// Hex encode/decode roundtrip.
        #[test]
        fn hex_roundtrip(data: Vec<u8>) {
            let encoded = hex_encode(&data);
            let decoded = hex_decode(&encoded).unwrap();
            prop_assert_eq!(data, decoded);
        }

        /// JSON serialization roundtrip.
        #[test]
        fn json_roundtrip(data: Vec<u8>) {
            let id = Hasher::hash(HashAlgorithm::Sha256, &data);
            let json = serde_json::to_string(&id).unwrap();
            let parsed: DocumentId = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(id, parsed);
        }

        /// BLAKE3 hashing is also deterministic.
        #[test]
        fn blake3_deterministic(data: Vec<u8>) {
            let h1 = Hasher::hash(HashAlgorithm::Blake3, &data);
            let h2 = Hasher::hash(HashAlgorithm::Blake3, &data);
            prop_assert_eq!(h1, h2);
        }
    }
}
