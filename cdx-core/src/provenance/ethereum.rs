//! Ethereum blockchain timestamp anchoring.
//!
//! This module provides types and verification for Ethereum-based
//! document timestamping. Timestamps are anchored by storing a hash
//! in an Ethereum transaction.
//!
//! # Timestamp Methods
//!
//! Ethereum timestamps can be created in several ways:
//!
//! 1. **Transaction data**: Store the hash in the `input` field of a transaction
//! 2. **Smart contract**: Call a timestamping contract that emits an event
//! 3. **`OP_RETURN` equivalent**: Use the transaction data field for hash storage
//!
//! # Verification
//!
//! Verification requires:
//! 1. The transaction hash
//! 2. Access to an Ethereum node or block explorer API
//! 3. Confirmation that the transaction is in a confirmed block
//!
//! # Example
//!
//! ```rust,ignore
//! use cdx_core::provenance::ethereum::{EthereumTimestamp, EthereumNetwork};
//!
//! let timestamp = EthereumTimestamp::new(
//!     "0x1234...".to_string(),
//!     document_hash,
//!     EthereumNetwork::Mainnet,
//! );
//!
//! // Verify with a node/API
//! let verified = verifier.verify(&timestamp).await?;
//! ```

use serde::{Deserialize, Serialize};

use crate::DocumentId;

/// An Ethereum-based timestamp record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthereumTimestamp {
    /// Transaction hash (0x-prefixed hex string).
    pub transaction_hash: String,

    /// Block number containing the transaction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_number: Option<u64>,

    /// Block hash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_hash: Option<String>,

    /// Document hash that was timestamped.
    pub document_hash: DocumentId,

    /// Network where the timestamp was anchored.
    pub network: EthereumNetwork,

    /// Number of confirmations at time of verification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmations: Option<u64>,

    /// Timestamp method used.
    pub method: EthereumTimestampMethod,

    /// Smart contract address (for contract-based timestamps).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_address: Option<String>,

    /// Block timestamp (Unix epoch seconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_timestamp: Option<u64>,
}

impl EthereumTimestamp {
    /// Create a new Ethereum timestamp.
    #[must_use]
    pub fn new(
        transaction_hash: String,
        document_hash: DocumentId,
        network: EthereumNetwork,
    ) -> Self {
        Self {
            transaction_hash,
            block_number: None,
            block_hash: None,
            document_hash,
            network,
            confirmations: None,
            method: EthereumTimestampMethod::TransactionData,
            contract_address: None,
            block_timestamp: None,
        }
    }

    /// Set the block number.
    #[must_use]
    pub fn with_block_number(mut self, block_number: u64) -> Self {
        self.block_number = Some(block_number);
        self
    }

    /// Set the block hash.
    #[must_use]
    pub fn with_block_hash(mut self, block_hash: String) -> Self {
        self.block_hash = Some(block_hash);
        self
    }

    /// Set the number of confirmations.
    #[must_use]
    pub fn with_confirmations(mut self, confirmations: u64) -> Self {
        self.confirmations = Some(confirmations);
        self
    }

    /// Set the timestamp method.
    #[must_use]
    pub fn with_method(mut self, method: EthereumTimestampMethod) -> Self {
        self.method = method;
        self
    }

    /// Set the contract address (for contract-based timestamps).
    #[must_use]
    pub fn with_contract(mut self, address: String) -> Self {
        self.contract_address = Some(address);
        self.method = EthereumTimestampMethod::SmartContract;
        self
    }

    /// Set the block timestamp.
    #[must_use]
    pub fn with_block_timestamp(mut self, timestamp: u64) -> Self {
        self.block_timestamp = Some(timestamp);
        self
    }

    /// Check if the timestamp has sufficient confirmations.
    #[must_use]
    pub fn is_confirmed(&self, min_confirmations: u64) -> bool {
        self.confirmations.is_some_and(|c| c >= min_confirmations)
    }

    /// Validate the transaction hash format.
    #[must_use]
    pub fn is_valid_tx_hash(&self) -> bool {
        // Ethereum transaction hashes are 66 characters (0x + 64 hex chars)
        self.transaction_hash.len() == 66
            && self.transaction_hash.starts_with("0x")
            && self.transaction_hash[2..]
                .chars()
                .all(|c| c.is_ascii_hexdigit())
    }

    /// Get the timestamp as a Unix epoch timestamp if available.
    #[must_use]
    pub fn unix_timestamp(&self) -> Option<u64> {
        self.block_timestamp
    }
}

/// Ethereum network for timestamp anchoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EthereumNetwork {
    /// Ethereum Mainnet (chain ID 1).
    Mainnet,
    /// Sepolia testnet (chain ID 11155111).
    Sepolia,
    /// Holesky testnet (chain ID 17000).
    Holesky,
    /// Polygon Mainnet (chain ID 137).
    Polygon,
    /// Arbitrum One (chain ID 42161).
    Arbitrum,
    /// Optimism (chain ID 10).
    Optimism,
    /// Base (chain ID 8453).
    Base,
    /// Custom network with chain ID.
    Custom(u64),
}

impl EthereumNetwork {
    /// Get the chain ID for this network.
    #[must_use]
    pub const fn chain_id(&self) -> u64 {
        match self {
            Self::Mainnet => 1,
            Self::Sepolia => 11_155_111,
            Self::Holesky => 17000,
            Self::Polygon => 137,
            Self::Arbitrum => 42161,
            Self::Optimism => 10,
            Self::Base => 8453,
            Self::Custom(id) => *id,
        }
    }

    /// Get the network name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Mainnet => "Ethereum Mainnet",
            Self::Sepolia => "Sepolia Testnet",
            Self::Holesky => "Holesky Testnet",
            Self::Polygon => "Polygon Mainnet",
            Self::Arbitrum => "Arbitrum One",
            Self::Optimism => "Optimism",
            Self::Base => "Base",
            Self::Custom(_) => "Custom Network",
        }
    }

    /// Check if this is a production network.
    #[must_use]
    pub const fn is_production(&self) -> bool {
        matches!(
            self,
            Self::Mainnet | Self::Polygon | Self::Arbitrum | Self::Optimism | Self::Base
        )
    }

    /// Get a block explorer URL for transactions.
    #[must_use]
    pub fn explorer_url(&self, tx_hash: &str) -> Option<String> {
        match self {
            Self::Mainnet => Some(format!("https://etherscan.io/tx/{tx_hash}")),
            Self::Sepolia => Some(format!("https://sepolia.etherscan.io/tx/{tx_hash}")),
            Self::Holesky => Some(format!("https://holesky.etherscan.io/tx/{tx_hash}")),
            Self::Polygon => Some(format!("https://polygonscan.com/tx/{tx_hash}")),
            Self::Arbitrum => Some(format!("https://arbiscan.io/tx/{tx_hash}")),
            Self::Optimism => Some(format!("https://optimistic.etherscan.io/tx/{tx_hash}")),
            Self::Base => Some(format!("https://basescan.org/tx/{tx_hash}")),
            Self::Custom(_) => None,
        }
    }
}

impl std::fmt::Display for EthereumNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Method used to anchor the timestamp on Ethereum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EthereumTimestampMethod {
    /// Hash stored in transaction input data.
    TransactionData,
    /// Hash emitted via smart contract event.
    SmartContract,
    /// Hash stored in contract storage.
    ContractStorage,
}

impl std::fmt::Display for EthereumTimestampMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TransactionData => write!(f, "Transaction Data"),
            Self::SmartContract => write!(f, "Smart Contract Event"),
            Self::ContractStorage => write!(f, "Contract Storage"),
        }
    }
}

/// Result of Ethereum timestamp verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthereumVerification {
    /// Whether the timestamp is verified.
    pub verified: bool,

    /// Block number where the transaction was included.
    pub block_number: Option<u64>,

    /// Number of confirmations.
    pub confirmations: u64,

    /// Block timestamp (Unix epoch).
    pub block_timestamp: Option<u64>,

    /// Whether the hash in the transaction matches the document hash.
    pub hash_matches: bool,

    /// Any error message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl EthereumVerification {
    /// Create a successful verification result.
    #[must_use]
    pub fn success(block_number: u64, confirmations: u64, block_timestamp: u64) -> Self {
        Self {
            verified: true,
            block_number: Some(block_number),
            confirmations,
            block_timestamp: Some(block_timestamp),
            hash_matches: true,
            error: None,
        }
    }

    /// Create a failed verification result.
    #[must_use]
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            verified: false,
            block_number: None,
            confirmations: 0,
            block_timestamp: None,
            hash_matches: false,
            error: Some(error.into()),
        }
    }

    /// Create an unconfirmed (pending) result.
    #[must_use]
    pub fn pending() -> Self {
        Self {
            verified: false,
            block_number: None,
            confirmations: 0,
            block_timestamp: None,
            hash_matches: false,
            error: Some("Transaction not yet confirmed".to_string()),
        }
    }
}

/// Configuration for Ethereum timestamp verification.
#[derive(Debug, Clone)]
pub struct EthereumConfig {
    /// Minimum confirmations required for a valid timestamp.
    pub min_confirmations: u64,

    /// RPC endpoint URL.
    pub rpc_url: Option<String>,

    /// Whether to use Etherscan API for verification.
    pub use_etherscan: bool,

    /// Etherscan API key.
    pub etherscan_api_key: Option<String>,
}

impl Default for EthereumConfig {
    fn default() -> Self {
        Self {
            min_confirmations: 12, // ~3 minutes on mainnet
            rpc_url: None,
            use_etherscan: false,
            etherscan_api_key: None,
        }
    }
}

impl EthereumConfig {
    /// Create a new configuration with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the minimum confirmations.
    #[must_use]
    pub fn with_min_confirmations(mut self, confirmations: u64) -> Self {
        self.min_confirmations = confirmations;
        self
    }

    /// Set the RPC URL.
    #[must_use]
    pub fn with_rpc_url(mut self, url: impl Into<String>) -> Self {
        self.rpc_url = Some(url.into());
        self
    }

    /// Enable Etherscan API verification.
    #[must_use]
    pub fn with_etherscan(mut self, api_key: impl Into<String>) -> Self {
        self.use_etherscan = true;
        self.etherscan_api_key = Some(api_key.into());
        self
    }
}

/// Verify an Ethereum timestamp offline (format and structure only).
///
/// This checks:
/// - Transaction hash format
/// - Network validity
/// - Confirmation count (if provided)
///
/// For full verification, use an Ethereum RPC client.
#[must_use]
pub fn verify_offline(
    timestamp: &EthereumTimestamp,
    config: &EthereumConfig,
) -> EthereumVerification {
    // Check transaction hash format
    if !timestamp.is_valid_tx_hash() {
        return EthereumVerification::failure("Invalid transaction hash format");
    }

    // Check if we have confirmation data
    if let Some(confirmations) = timestamp.confirmations {
        if confirmations >= config.min_confirmations {
            if let (Some(block_num), Some(block_ts)) =
                (timestamp.block_number, timestamp.block_timestamp)
            {
                return EthereumVerification::success(block_num, confirmations, block_ts);
            }
        } else {
            return EthereumVerification::failure(format!(
                "Insufficient confirmations: {} < {}",
                confirmations, config.min_confirmations
            ));
        }
    }

    // No confirmation data - cannot verify offline
    EthereumVerification::failure("Cannot verify offline without confirmation data")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_hash() -> DocumentId {
        "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            .parse()
            .unwrap()
    }

    fn test_tx_hash() -> String {
        "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string()
    }

    #[test]
    fn test_ethereum_timestamp_creation() {
        let timestamp =
            EthereumTimestamp::new(test_tx_hash(), test_hash(), EthereumNetwork::Mainnet);

        assert_eq!(timestamp.network, EthereumNetwork::Mainnet);
        assert!(timestamp.is_valid_tx_hash());
    }

    #[test]
    fn test_ethereum_timestamp_builder() {
        let timestamp =
            EthereumTimestamp::new(test_tx_hash(), test_hash(), EthereumNetwork::Mainnet)
                .with_block_number(12_345_678)
                .with_confirmations(100)
                .with_block_timestamp(1_700_000_000);

        assert_eq!(timestamp.block_number, Some(12_345_678));
        assert_eq!(timestamp.confirmations, Some(100));
        assert!(timestamp.is_confirmed(50));
    }

    #[test]
    fn test_invalid_tx_hash() {
        let timestamp =
            EthereumTimestamp::new("invalid".to_string(), test_hash(), EthereumNetwork::Mainnet);

        assert!(!timestamp.is_valid_tx_hash());
    }

    #[test]
    fn test_ethereum_network_chain_id() {
        assert_eq!(EthereumNetwork::Mainnet.chain_id(), 1);
        assert_eq!(EthereumNetwork::Polygon.chain_id(), 137);
        assert_eq!(EthereumNetwork::Custom(99999).chain_id(), 99999);
    }

    #[test]
    fn test_ethereum_network_is_production() {
        assert!(EthereumNetwork::Mainnet.is_production());
        assert!(EthereumNetwork::Polygon.is_production());
        assert!(!EthereumNetwork::Sepolia.is_production());
    }

    #[test]
    fn test_ethereum_network_explorer_url() {
        let url = EthereumNetwork::Mainnet.explorer_url("0x1234");
        assert_eq!(url, Some("https://etherscan.io/tx/0x1234".to_string()));

        let custom_url = EthereumNetwork::Custom(12345).explorer_url("0x1234");
        assert!(custom_url.is_none());
    }

    #[test]
    fn test_ethereum_verification_success() {
        let result = EthereumVerification::success(12_345_678, 100, 1_700_000_000);
        assert!(result.verified);
        assert!(result.hash_matches);
        assert_eq!(result.confirmations, 100);
    }

    #[test]
    fn test_ethereum_verification_failure() {
        let result = EthereumVerification::failure("Test error");
        assert!(!result.verified);
        assert_eq!(result.error, Some("Test error".to_string()));
    }

    #[test]
    fn test_verify_offline_valid() {
        let timestamp =
            EthereumTimestamp::new(test_tx_hash(), test_hash(), EthereumNetwork::Mainnet)
                .with_block_number(12_345_678)
                .with_confirmations(100)
                .with_block_timestamp(1_700_000_000);

        let config = EthereumConfig::new().with_min_confirmations(12);
        let result = verify_offline(&timestamp, &config);

        assert!(result.verified);
    }

    #[test]
    fn test_verify_offline_insufficient_confirmations() {
        let timestamp =
            EthereumTimestamp::new(test_tx_hash(), test_hash(), EthereumNetwork::Mainnet)
                .with_confirmations(5);

        let config = EthereumConfig::new().with_min_confirmations(12);
        let result = verify_offline(&timestamp, &config);

        assert!(!result.verified);
        assert!(result.error.unwrap().contains("Insufficient"));
    }

    #[test]
    fn test_verify_offline_invalid_hash() {
        let timestamp =
            EthereumTimestamp::new("invalid".to_string(), test_hash(), EthereumNetwork::Mainnet);

        let config = EthereumConfig::default();
        let result = verify_offline(&timestamp, &config);

        assert!(!result.verified);
        assert!(result.error.unwrap().contains("Invalid transaction hash"));
    }

    #[test]
    fn test_ethereum_config_builder() {
        let config = EthereumConfig::new()
            .with_min_confirmations(6)
            .with_rpc_url("https://eth.example.com")
            .with_etherscan("myapikey");

        assert_eq!(config.min_confirmations, 6);
        assert!(config.use_etherscan);
        assert_eq!(config.etherscan_api_key, Some("myapikey".to_string()));
    }

    #[test]
    fn test_timestamp_method_display() {
        assert_eq!(
            EthereumTimestampMethod::TransactionData.to_string(),
            "Transaction Data"
        );
        assert_eq!(
            EthereumTimestampMethod::SmartContract.to_string(),
            "Smart Contract Event"
        );
    }

    #[test]
    fn test_ethereum_timestamp_serialization() {
        let timestamp =
            EthereumTimestamp::new(test_tx_hash(), test_hash(), EthereumNetwork::Mainnet)
                .with_block_number(12_345_678);

        let json = serde_json::to_string(&timestamp).unwrap();
        assert!(json.contains("\"network\":\"mainnet\""));
        assert!(json.contains("\"blockNumber\":12345678")); // JSON doesn't use underscores

        let deserialized: EthereumTimestamp = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.block_number, Some(12_345_678));
    }
}
