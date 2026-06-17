# cdx-core Architecture

This document describes the architecture of the cdx-core library.

## Overview

cdx-core is organized into layered modules that mirror the CDX Document Format specification. Each layer can be used independently or composed together through the high-level `Document` API.

## Module Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         Document API                             │
│                    (document.rs, lib.rs)                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐ │
│  │ Content  │  │ Metadata │  │Presentation│ │    Security      │ │
│  │          │  │          │  │           │  │                  │ │
│  │ - Block  │  │ - Dublin │  │ - Paginated│ │ - Signature     │ │
│  │ - Text   │  │   Core   │  │ - Continuous│ │ - Signer       │ │
│  │ - Valid. │  │          │  │ - Style    │  │ - Verifier     │ │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────────┘ │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    Asset Management                        │   │
│  │                                                            │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────┐  │   │
│  │  │  Image  │  │  Font   │  │  Embed  │  │ AssetIndex  │  │   │
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────────┘  │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│                         Core Types                               │
│                                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────────┐ │
│  │ Manifest │  │   Hash   │  │  State   │  │     Error        │ │
│  │          │  │          │  │          │  │                  │ │
│  │ - FileRef│  │- DocId   │  │- Draft   │  │ - Result<T>     │ │
│  │ - Lineage│  │- Hasher  │  │- Review  │  │ - Error enum    │ │
│  │          │  │- Algo    │  │- Frozen  │  │                  │ │
│  └──────────┘  └──────────┘  └──────────┘  └──────────────────┘ │
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│                        Archive I/O                               │
│                                                                  │
│  ┌─────────────────────────┐  ┌─────────────────────────────┐   │
│  │       CdxReader         │  │        CdxWriter            │   │
│  │                         │  │                             │   │
│  │ - Open from file/bytes  │  │ - Create new archives       │   │
│  │ - Read manifest         │  │ - Write with compression    │   │
│  │ - Extract files         │  │ - Compute hashes            │   │
│  │ - Verify hashes         │  │ - Path security             │   │
│  └─────────────────────────┘  └─────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │   ZIP Archive    │
                    │   (zip crate)    │
                    └──────────────────┘
```

## Module Descriptions

### Document API (`document.rs`)

The high-level entry point for working with CDX documents.

- **Document**: Main type representing a complete CDX document
- **DocumentBuilder**: Fluent builder for creating documents
- **VerificationReport**: Results of document integrity verification

### Archive (`archive/`)

Low-level ZIP archive handling with security checks.

- **CdxReader**: Opens and reads `.cdx` archives
- **CdxWriter**: Creates new `.cdx` archives with proper structure
- **Path validation**: Prevents path traversal attacks

### Content (`content/`)

Semantic content model with 20+ block types and 15+ mark types.

- **Block**: Enum of all block types (Paragraph, Heading, List, CodeBlock, Table, Figure, Math, etc.)
- **Text**: Text nodes with optional marks (bold, italic, link, code, highlight, etc.)
- **Content**: Root structure containing version and blocks array
- **Validation**: Structural validation (lists contain list items, etc.)
- **Custom serde**: Hand-written `Serialize`/`Deserialize` impls for `Block` and `Mark` that produce flat `{"type": "...", ...}` JSON matching the CDX spec wire format, replacing derived serde which produced nested `{"Paragraph": {...}}` Rust-style enums

### Metadata (`metadata/`)

Document metadata following standards.

- **DublinCore**: Dublin Core metadata terms
- **StringOrArray**: Flexible single/multiple value handling

### Presentation (`presentation/`)

Visual rendering instructions.

- **Paginated**: Fixed-page layout for print/PDF
- **Continuous**: Scrolling layout for screens
- **Style**: CSS-like styling properties

### Security (`security/`)

Cryptographic operations (feature-gated).

- **Signature**: Signature data structures
- **Signer/Verifier**: Traits for signing operations
- **EcdsaSigner**: ECDSA P-256 (ES256) implementation

### Asset (`asset/`)

Embedded resource management.

- **ImageAsset**: Image metadata and formats
- **FontAsset**: Font metadata and formats
- **EmbedAsset**: Generic file embedding
- **AssetIndex**: Collection management

### Core Types

Foundation types used throughout.

- **Manifest** (`manifest.rs`): Document manifest structure
- **DocumentId** (`hash.rs`): Content-addressable identifier
- **Hasher** (`hash.rs`): Hash computation utilities
- **DocumentState** (`state.rs`): Lifecycle state machine
- **Error** (`error.rs`): Error types and Result alias

## Data Flow

### Opening a Document

```
File/Bytes → CdxReader → Manifest + Content + Metadata → Document
                 │
                 ├── Validates ZIP structure
                 ├── Reads manifest.json
                 ├── Reads content/document.json
                 └── Reads metadata/dublin-core.json
```

### Creating a Document

```
DocumentBuilder → Document → CdxWriter → ZIP File
                      │
                      ├── Builds Content blocks
                      ├── Builds DublinCore metadata
                      ├── Creates Manifest
                      └── Computes document ID
```

### Verification Flow

```
Document.verify()
    │
    ├── Compute content hash
    │   └── Compare with manifest.content.hash
    │
    ├── Compute document ID
    │   └── Compare with manifest.id
    │
    └── Return VerificationReport
```

## Feature Flags

| Flag | Modules Affected | Dependencies Added |
|------|------------------|-------------------|
| `zstd` | archive | zip/zstd |
| `signatures` | security | p256, ecdsa, base64, rand_core |
| `wasm` | all | getrandom/js |

## Thread Safety

All types are `Send + Sync` where possible. The library uses no internal mutability that would require synchronization.

## Error Handling

All fallible operations return `Result<T, Error>`. Errors include context about what operation failed and why.

## Performance Considerations

- **Lazy loading**: Assets are not loaded until accessed
- **Streaming hashes**: Large files can be hashed without full memory load
- **Compression**: Zstd provides high compression ratios for text content
- **Zero-copy where possible**: References are preferred over cloning

## Implementation Status

All core features have been implemented:

```
┌─────────────────────────────────────────────────────────────────┐
│                         Security                                 │
│                                                                  │
│  ┌──────────────────┐  ┌──────────────────┐  ┌───────────────┐ │
│  │    Encryption    │  │  EdDSA Signer    │  │   ML-DSA-65   │ │
│  │   (AES-256-GCM)  │  │   (Ed25519)      │  │ (Post-Quantum)│ │
│  └──────────────────┘  └──────────────────┘  └───────────────┘ │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                        Provenance                                │
│                                                                  │
│  ┌──────────────────┐  ┌──────────────────┐  ┌───────────────┐ │
│  │   Merkle Tree    │  │  Block Proofs    │  │   Timestamp   │ │
│  │                  │  │                  │  │   Anchoring   │ │
│  └──────────────────┘  └──────────────────┘  └───────────────┘ │
│                                                                  │
│  ┌──────────────────┐  ┌──────────────────┐  ┌───────────────┐ │
│  │   RFC 3161 TSA   │  │ OpenTimestamps   │  │  OCSP/CRL     │ │
│  │                  │  │   (Bitcoin)      │  │  Revocation   │ │
│  └──────────────────┘  └──────────────────┘  └───────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```
