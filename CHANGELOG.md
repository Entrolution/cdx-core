# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.7.1] - 2026-03-13

### Fixed

#### Correctness
- Clear stale `manifest.security` ref when writing a document with no signatures or encryption
- Map ZIP `FileNotFound` errors to `MissingFile` and other ZIP errors to `InvalidArchive` (was mapping all to `MissingFile`)
- Improve freeze error message to mention `set_lineage` for root documents

#### Document Mutation Consistency
- Update `modified` timestamp in `define_extension_accessors!` macro (`set_*`, `clear_*`, `*_mut` methods)
- Update `modified` timestamp in `set_encryption` and `clear_encryption`

#### Security
- Add `zeroize` crate for key material cleanup on drop (`Aes256GcmEncryptor`, `ChaCha20Poly1305Encryptor`, `Pbes2KeyWrapper`, `Pbes2KeyUnwrapper`, `MlDsaSigner` seed)
- Enforce PBKDF2 iteration bounds (10,000 - 10,000,000) in `Pbes2KeyWrapper::new` and `Pbes2KeyUnwrapper::unwrap`
- Fix `permissions_for` to check specific User/Group/Role grants before `Everyone` wildcard
- Correct error variants: 7 security modules switched from `invalid_manifest()` to `SignatureError`
- Propagate OCSP/CRL errors in revocation checker instead of silently falling through

#### Validation
- Validate subfigure blocks and IDs in `validate_figure`
- Clamp heading level to 1-6 on deserialization (was accepting any u8)
- Add `PartialDate` validation: month 1-12, day 1-31 (on deserialization and via `try_year_month`/`try_full`)

#### CLI
- Add warning that content-level encrypt/decrypt is not yet implemented
- Return non-zero exit code from `add-timestamp` (was `Ok(())` for unimplemented feature)
- Replace `std::process::exit(1)` with `anyhow::bail!` in `prove` and `timestamp` commands
- Return non-zero exit code from disabled-feature JSON paths in `decrypt`, `timestamp`
- Fix `truncate_token` to use char-boundary-safe truncation (was byte-indexing)

#### API
- Implement recursive `get_mut` for `CommentThread` (now finds nested replies, was top-level only)
- Fix OTS `verify_timestamp` to return `valid: false` for unverified proofs (was `true`)
- Fix Ethereum `verify_offline` to set `hash_matches: false` (offline cannot verify on-chain data)
- Update `matches_document` doc to clarify it only checks token presence

### Added
- `PartialDate::try_year_month` and `PartialDate::try_full` fallible constructors
- `Pbes2KeyWrapper::MIN_ITERATIONS` and `MAX_ITERATIONS` constants
- `merge_styles` regression test covering all 35 `Style` fields
- Tests for stale security ref, lineage error, mutation timestamps, subfigure validation, heading clamping, PBKDF2 bounds, permissions specificity, recursive thread `get_mut`

### Changed
- **Breaking:** `Pbes2KeyWrapper::new` now returns `Result` (validates iteration bounds)

## [0.7.0] - 2026-02-16

### Changed

- **Breaking:** `ExtensionMark::glossary()` now emits `"ref"` instead of `"termId"` to match the spec's `glossaryMark` schema
- **Breaking:** `GlossaryRef.term_id` serializes as `"ref"` instead of `"termId"`
- Deserialization accepts both old `"termId"` and new `"ref"` for backward compatibility

### Added

- `ExtensionMark::get_glossary_ref()` helper supporting both `"ref"` and legacy `"termId"` keys
- `ExtensionMark::normalize_glossary_attrs()` to migrate `"termId"` → `"ref"` in-place
- Backward-compatibility tests for glossary `"termId"` deserialization

## [0.6.0] - 2026-02-16

### Changed

- **Breaking:** `Citation.reference` (String) renamed to `Citation.refs` (Vec\<String\>) to support multi-citation clusters (e.g., `[smith2023; jones2024]`)
- **Breaking:** `ExtensionMark::citation()` and `citation_with_page()` now emit `"refs"` (array) instead of `"ref"` (string)
- Deserialization accepts both old `"ref"` (string) and new `"refs"` (array) for backward compatibility

### Added

- `Citation::multi()` constructor for multi-reference citations
- `Citation::first_ref()` and `Citation::refs()` accessors
- `ExtensionMark::multi_citation()` convenience constructor
- `ExtensionMark::get_string_array_attribute()` for array-typed attributes
- `ExtensionMark::get_citation_refs()` helper supporting both `"refs"` and legacy `"ref"` keys
- `ExtensionMark::normalize_citation_attrs()` to migrate `"ref"` → `"refs"` in-place
- `ExtensionBlock::get_string_array_attribute()` for parity with `ExtensionMark`
- Backward-compatibility conformance tests for singular `"ref"` deserialization
- Multi-reference citation roundtrip tests

## [0.5.0] - 2026-02-16

### Changed

#### Spec Serialization Compliance
- **Breaking:** `Mark::Math { value }` field renamed to `Mark::Math { source }` to match spec
- **Breaking:** Simple marks (Bold, Italic, etc.) now serialize as strings (`"bold"`) instead of objects (`{"type":"bold"}`)
- **Breaking:** Extension marks serialize with colon-delimited type (`"semantic:citation"`) instead of wrapper (`{"type":"extension","namespace":"semantic","markType":"citation"}`)
- **Breaking:** Extension blocks serialize with colon-delimited type (`"academic:theorem"`) instead of wrapper format
- **Breaking:** `Block::block_type()` returns `Cow<'_, str>` instead of `&'static str`; extension blocks return `"namespace:blockType"` instead of `"extension"`
- `FigCaption` block type serializes as `"figcaption"` (lowercase) instead of `"figCaption"`
- All old formats are accepted on deserialization for backward compatibility

#### CLI Restructuring
- Split `cdx-cli/src/main.rs` into `cli.rs` (argument definitions), `dispatcher.rs` (command dispatch), and `main.rs` (entry point)

### Added

#### Spec Conformance Testing
- Conformance test suite (`tests/conformance.rs`) covering all 78 testable spec requirements
- Conformance matrix (`docs/conformance-matrix.md`) mapping spec sections to tests — 78/79 PASS, 0 TODO
- Hash boundary tests verifying document ID includes/excludes correct fields
- Asset embedding tests: hash verification, missing file detection, hash mismatch errors
- State machine enforcement tests for review/frozen/published requirements
- Provenance/lineage validation tests
- Property-based tests using proptest for hash determinism, metadata inclusion, block round-trips
- Fuzz targets for Block, Mark, and Content deserialization (`fuzz/fuzz_targets/`)

#### Security Policy
- Added `SECURITY.md` with supported versions and vulnerability reporting process

## [0.4.0] - 2026-02-16

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
- Coordinated RustCrypto ecosystem upgrade:
  - `rand_core` 0.6 → 0.10 (stable); `der` 0.7 → 0.8 (stable)
  - `p256` 0.13 → 0.14.0-rc; `p384` 0.13 → 0.14.0-rc; `ecdsa` 0.16 → 0.17.0-rc
  - `rsa` 0.9 → 0.10.0-rc; `x509-cert` 0.2 → 0.3.0-rc
  - Migrate from `OsRng` / `fill_bytes` to `getrandom::fill` and `Generate` trait
  - Use `rsa::sha2::Sha256` for RSA operations (sha2 0.10 → 0.11 split)
  - Use `PublicKey::from_sec1_bytes` for P-256 key parsing (replaces `EncodedPoint` chain)
  - Use `tbs_certificate()` / `serial_number()` / `extensions()` accessors (x509-cert 0.3 made fields private)
- Replace `fips204` with RustCrypto `ml-dsa` 0.1.0-rc for ML-DSA-65 signatures (uses standard `signature::Signer`/`Verifier` traits, 32-byte seed key format)
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
- **ML-DSA** (PR #78): Switched from `fips204` to RustCrypto `ml-dsa` crate; `MlDsaSigner::from_bytes` now accepts a 32-byte seed (was 4032-byte expanded key); key/signature bytes are incompatible with prior `fips204`-based output

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

[Unreleased]: https://github.com/Entrolution/cdx-core/compare/v0.7.1...HEAD
[0.7.1]: https://github.com/Entrolution/cdx-core/compare/v0.7.0...v0.7.1
[0.7.0]: https://github.com/Entrolution/cdx-core/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/Entrolution/cdx-core/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/Entrolution/cdx-core/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/Entrolution/cdx-core/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/Entrolution/cdx-core/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/Entrolution/cdx-core/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Entrolution/cdx-core/releases/tag/v0.1.0
