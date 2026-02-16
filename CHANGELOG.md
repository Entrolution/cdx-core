# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### Unified Anchor System
- `ContentAnchor` for block-level, point, and range anchors
- `ContentAnchorUri` for URI format parsing/formatting (`#blockId/start-end`)
- `Mark::Anchor { id }` variant for named anchor marks in text
- Full bidirectional conversion between anchor types

#### Phantom Extension
- `PhantomClusters` for off-page annotation clusters
- `PhantomCluster` with anchor, scope, author, and metadata
- `Phantom` with position, size, content, and connections
- `PhantomScope` for visibility control (Shared, Private, Role-based)
- `PhantomConnection` with style options (Line, Arrow, Dashed)
- Connection validation with cycle detection
- Archive integration: `read_phantoms()` and `write_phantoms()`

#### Scoped Signatures
- `SignatureScope` for layout attestation
- JCS (RFC 8785) serialization for deterministic scope hashing
- `Signature.scope` field for scoped signature support
- `with_layout()` builder for adding layout hashes

#### Declarative Validation Rules
- `ValidationRule::ContainsUppercase` - requires uppercase letter
- `ValidationRule::ContainsLowercase` - requires lowercase letter
- `ValidationRule::ContainsDigit` - requires digit
- `ValidationRule::ContainsSpecial` - requires special character
- `ValidationRule::MatchesField` - cross-field validation

#### Collaboration
- `Collaborator.color` field for real-time cursor coloring
- `with_color()` builder method

#### Spec Compliance: Core Struct Fields (PR #66)
- `PhantomsRef` struct and `phantoms` field on `Manifest`
- `KeyManagementAlgorithm` enum and `key_management` field on `EncryptionMetadata`
- `ephemeral_public_key` field on `Recipient`
- `TrustedTimestamp` struct and `timestamp` field on `Signature`

#### Spec Compliance: Content & Extension Fields (PR #67)
- `CodeToken` struct, `highlighting` and `tokens` fields on `CodeBlock`
- `FigureNumbering` enum, `Subfigure` struct, `numbering` and `subfigures` fields on `FigureBlock`
- `uses` and `restate` fields on `Theorem`
- `StructuralInduction`, `Counting`, `Probabilistic` variants on `ProofMethod`
- `start_line` field on `Algorithm`
- `docket` field on `Caption`

#### Spec Compliance: Key Wrapping (PR #71)
- `EcdhEsKeyWrapper` and `EcdhEsKeyUnwrapper` for ECDH-ES+A256KW key agreement (RFC 7518 / RFC 3394)
- `WrappedKeyData` struct for wrapped content encryption keys
- New `key-wrapping` feature flag (depends on `encryption`, adds `aes-kw` and `hkdf`)

#### Spec Compliance: Extended Key Wrapping + BOM (PR #73)
- `RsaOaepKeyWrapper` and `RsaOaepKeyUnwrapper` for RSA-OAEP-256 key wrapping
- `Pbes2KeyWrapper` and `Pbes2KeyUnwrapper` for PBES2-HS256+A256KW password-based key wrapping
- New `key-wrapping-rsa` and `key-wrapping-pbes2` feature flags
- UTF-8 BOM stripping for all JSON files in archive reader

#### Spec Compliance: Form Conditional Validation (PR #75)
- `ConditionalValidation`, `Condition`, `ConditionOperator`, `ConditionalAction` types
- `conditional_validation` field on all 7 form field types
- Supports `equals`, `notEquals`, `isEmpty`, `isNotEmpty` operators

#### Spec Compliance: Advanced Presentation (PR #76)
- `TypographyConfig` with `LineNumbering`, `BaselineGrid`, `HyphenationConfig`
- `ColumnLayout` and `GridLayout` with `GridArea` for multi-column and CSS Grid layouts
- `TocConfig` with `TocLeaders` for table of contents configuration
- `FootnotesConfig`, `FootnotePosition`, `FootnoteSeparator` for footnote placement and styling
- `EndnotesConfig` for endnote section configuration

### Changed

#### Spec Compliance: Validation Fixes (PR #70)
- Relax lineage validation: root (non-forked) documents can now reach Frozen/Published without lineage
- Enforce manifest-first ordering in archive reader (error instead of silent acceptance)
- Add decompression bomb protection: 256 MiB file size limit with declared-size pre-check and bounded reads
- Add `is_url_safe_path()` utility for spec SHOULD-level asset path validation
- Add `FileTooLarge` and `InvalidArchiveStructure` error variants

#### Dependencies
- Bump `zip` from 7.2 to 8.0 (resolves yanked 7.4.0; no code changes required)
- Bump `assert_cmd` from 2.0 to 2.1.2
- Update `keccak` from 0.1.5 (yanked) to 0.1.6

#### Code Quality
- Enable `clippy::pedantic` in `cdx-cli` (already zero warnings; prevents regressions)
- Remove unused `PropertySchema` variants (`Integer`, `Number`, `Boolean`) and their match arms
- Replace `strum` derive macros for ~10 enum Display implementations (PR #55)
- Extract shared crypto helpers into `crypto_common` module (PR #57)
- Fix all pedantic lint warnings across workspace (PR #58)
- Tighten `cargo-deny` configuration: `yanked = "deny"`, remove unused license allowances
- Replace `clippy::too_many_arguments` suppressions with parameter structs in CLI (PR #64)
- Bump `uniffi` from 0.28 to 0.31 in `cdx-swift-bridge` (PR #65)

#### Breaking Changes
- **Paginated presentation**: `blockRef` renamed to `blockId`, `blockRefs` to `blockIds`
- **Forms**: Removed `ValidationRule::Custom` (executable expressions prohibited per DD-010/DD-019)
- **EquationGroup** (PR #74): `Equation` renamed to `EquationLine`, `latex` field renamed to `value`, `equations` field renamed to `lines`, added `tag` field, added `Alignat` environment variant
- **Legal SignatureBlock** (PR #74): `Signatory` and `FirmInfo` flattened into `LegalSigner`, added `role` field on `LegalSignatureBlock`

#### CI
- Increase cargo-tarpaulin timeout to 180s and make coverage non-blocking (`continue-on-error`)

#### Documentation
- Clarified lineage requirements: parent only required for forked documents
- Clarified hash scope: document ID covers semantic content only, not layout
- Fix MSRV in CONTRIBUTING.md (1.85 → 1.88)
- Add security audit documentation (`cargo audit`, `cargo deny`) to CONTRIBUTING.md

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
