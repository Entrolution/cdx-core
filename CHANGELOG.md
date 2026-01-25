# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2025-01-25

### Added

#### Provenance Module
- `MerkleTree` for content-addressable tree structures
- `MerkleNode` for individual tree nodes (leaf and branch)
- `BlockProof` for selective disclosure proofs
- `ProofVerification` for detailed verification results
- Tree building from items or pre-computed hashes
- Proof generation for any leaf by index

#### Timestamp Anchoring
- `TimestampRequest` for RFC 3161 timestamp requests
- `TimestampResponse` for TSA responses
- `TimestampToken` for timestamp tokens
- `TimestampStatus` and `TimestampAccuracy` types
- Message imprint computation from document IDs
- Nonce support for replay protection

### Changed
- `provenance` module is now public and fully implemented

## [0.2.0] - 2025-01-25

### Added

#### EdDSA Signatures
- `EddsaSigner` for Ed25519 digital signatures
- `EddsaVerifier` for Ed25519 signature verification
- Full PEM key support for EdDSA keys
- New `eddsa` feature flag

#### Encryption
- `Aes256GcmEncryptor` for AES-256-GCM authenticated encryption
- `EncryptionMetadata` type for encryption configuration
- Key derivation function support (PBKDF2, Argon2id)
- Multi-recipient encryption metadata
- New `encryption` feature flag

### Changed
- `full` feature now includes `encryption` and `eddsa`

## [0.1.0] - 2025-01-25

Initial release implementing Codex Document Format Specification v0.1.

### Added

#### Core Infrastructure
- `Document` type with high-level API for working with Codex documents
- `DocumentBuilder` for fluent document creation
- `DocumentState` enum with state machine (Draft, Review, Frozen, Published)
- `DocumentId` for content-addressable document identification
- `Manifest` type with full serialization support

#### Archive I/O
- `CdxReader` for reading `.cdx` ZIP archives
- `CdxWriter` for creating `.cdx` archives with proper structure
- Path traversal security checks
- Support for Deflate and Zstandard compression
- Reading from files, bytes, or any `Read + Seek` source

#### Content Model
- All 13 block types: Paragraph, Heading, List, ListItem, Blockquote, CodeBlock, HorizontalRule, Image, Table, TableRow, TableCell, Math, Break
- Text nodes with marks (Bold, Italic, Underline, Strikethrough, Code, Superscript, Subscript, Link)
- Content validation with detailed error reporting
- Block attributes (dir, lang) support

#### Metadata
- Dublin Core metadata support with all standard terms
- Single value and array support for multi-valued fields
- Full serialization/deserialization

#### Presentation Layers
- `Paginated` presentation for print/PDF output
- `Continuous` presentation for screen/scroll layouts
- CSS-like `Style` type with common properties
- Page size, margins, and positioning support

#### Digital Signatures
- ECDSA P-256 (ES256) signing and verification
- `Signer` and `Verifier` traits for extensibility
- `EcdsaSigner` and `EcdsaVerifier` implementations
- `SignerInfo` with name, email, organization, certificate support
- Signature file structure per Codex security extension

#### Asset Management
- `ImageAsset` with format support (AVIF, WebP, PNG, JPEG, SVG)
- `FontAsset` with format support (WOFF2, WOFF, TTF, OTF)
- `EmbedAsset` for arbitrary file embedding
- `AssetIndex` for managing asset collections
- Asset hash verification

#### Hashing
- SHA-256 (default), SHA-3-256, and BLAKE3 support
- Streaming hash computation for large files
- `Hasher` utility for computing hashes

#### Verification
- Document ID verification
- Content hash verification
- `VerificationReport` with detailed results

### Features
- `zstd` (default) - Zstandard compression support
- `signatures` (default) - Digital signature support
- `wasm` - WebAssembly compilation support
- `full` - All features enabled

### Examples
- `create_document` - Create a document from scratch
- `open_and_verify` - Open and verify document integrity
- `sign_document` - Sign a document with ES256
- `extract_content` - Extract text content from blocks

[Unreleased]: https://github.com/gvonness-apolitical/cdx-core/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/gvonness-apolitical/cdx-core/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/gvonness-apolitical/cdx-core/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/gvonness-apolitical/cdx-core/releases/tag/v0.1.0
