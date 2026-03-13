[![CI](https://github.com/Entrolution/cdx-core/actions/workflows/ci.yml/badge.svg)](https://github.com/Entrolution/cdx-core/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/Entrolution/cdx-core/graph/badge.svg)](https://codecov.io/gh/Entrolution/cdx-core)
[![Crates.io](https://img.shields.io/crates/v/cdx-core.svg)](https://crates.io/crates/cdx-core)
[![Documentation](https://docs.rs/cdx-core/badge.svg)](https://docs.rs/cdx-core)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)
[![MSRV](https://img.shields.io/badge/MSRV-1.88-blue.svg)](https://www.rust-lang.org)

# cdx-core

Core Rust library for reading, writing, and validating [Codex Document Format](https://github.com/Entrolution/codex-file-format-spec) (`.cdx`) files.

## Overview

Codex is an open document format designed for:

- **Semantic content** - Structured content blocks, not just rendered output
- **Verifiable integrity** - Content-addressable hashing and digital signatures
- **Machine readability** - JSON-based format with well-defined schemas
- **Modern security** - Built-in support for signatures and encryption

This library provides the foundational capabilities for working with Codex documents in Rust, with support for WASM compilation.

## Features

- **Archive I/O** - Read and write `.cdx` ZIP archives with proper structure validation
- **Content Model** - Full support for all 13 content block types (paragraphs, headings, lists, tables, code, math, images, etc.)
- **Document Builder** - Fluent API for creating documents programmatically
- **Metadata** - Dublin Core metadata support
- **Presentation Layers** - Paginated and continuous presentation types
- **Digital Signatures** - ECDSA P-256 (ES256), P-384 (ES384), Ed25519, RSA-PSS (PS256), and ML-DSA-65 signing and verification
- **Encryption** - AES-256-GCM and ChaCha20-Poly1305 content encryption
- **Asset Management** - Embed and manage images, fonts, and files
- **Document Verification** - Verify content hashes and document integrity
- **Provenance** - Merkle trees, block proofs, and timestamp support for content lineage
- **Multiple Hash Algorithms** - SHA-256 (default), SHA-3, and BLAKE3
- **Zstandard Compression** - Optional high-ratio compression support
- **WASM Compatible** - Compile to WebAssembly with the `wasm` feature

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
cdx-core = "0.7"
```

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `zstd` | Yes | Zstandard compression support |
| `signatures` | Yes | ECDSA P-256 digital signatures (ES256) |
| `signatures-es384` | No | P-384 ECDSA digital signatures (ES384) |
| `signatures-rsa` | No | RSA-PSS digital signatures (PS256) |
| `encryption` | No | AES-256-GCM content encryption |
| `encryption-chacha` | No | ChaCha20-Poly1305 content encryption |
| `key-wrapping` | No | ECDH-ES+A256KW key agreement |
| `key-wrapping-rsa` | No | RSA-OAEP-256 key wrapping |
| `key-wrapping-pbes2` | No | PBES2 password-based key wrapping |
| `eddsa` | No | Ed25519 digital signatures |
| `ml-dsa` | No | ML-DSA-65 post-quantum signatures |
| `webauthn` | No | WebAuthn/FIDO2 signature verification |
| `timestamps-rfc3161` | No | RFC 3161 timestamp acquisition |
| `timestamps-ots` | No | OpenTimestamps (Bitcoin anchoring) |
| `ocsp` | No | Certificate revocation checking (OCSP/CRL) |
| `wasm` | No | WASM compilation support |
| `full` | No | All features enabled |

## Quick Start

### Creating a Document

```rust
use cdx_core::{Document, Result};

fn main() -> Result<()> {
    let document = Document::builder()
        .title("My Document")
        .creator("Jane Doe")
        .add_heading(1, "Introduction")
        .add_paragraph("This is my first Codex document.")
        .add_heading(2, "Features")
        .add_paragraph("Codex provides semantic structure and verifiable integrity.")
        .build()?;

    // Compute the document ID
    let doc_id = document.compute_id()?;
    println!("Document ID: {doc_id}");

    // Save the document
    document.save("output.cdx")?;
    Ok(())
}
```

### Opening and Verifying

```rust
use cdx_core::{Document, Result};

fn main() -> Result<()> {
    let document = Document::open("example.cdx")?;

    println!("Title: {:?}", document.dublin_core().terms.title);
    println!("State: {:?}", document.state());

    // Verify document integrity
    let report = document.verify()?;
    if report.is_valid() {
        println!("Document integrity verified!");
    }
    Ok(())
}
```

### Digital Signatures

```rust
use cdx_core::{Document, Result};
use cdx_core::security::{EcdsaSigner, SignerInfo, Signer};

fn main() -> Result<()> {
    let document = Document::builder()
        .title("Signed Document")
        .creator("Signing Example")
        .add_paragraph("This document will be signed.")
        .build()?;

    let doc_id = document.compute_id()?;

    // Create a signer
    let signer_info = SignerInfo::new("Alice")
        .with_email("alice@example.com");
    let (signer, public_key) = EcdsaSigner::generate(signer_info)?;

    // Sign the document
    let signature = signer.sign(&doc_id)?;
    println!("Signed with algorithm: {}", signature.algorithm);
    Ok(())
}
```

## Examples

Run the included examples:

```bash
# Create a document from scratch
cargo run --example create_document

# Extract content from a document
cargo run --example extract_content

# Sign a document (requires signatures feature)
cargo run --example sign_document --features signatures

# Open and verify a document
cargo run --example open_and_verify path/to/document.cdx
```

## Specification Compliance

This library implements the [Codex Document Format Specification v0.1](https://github.com/Entrolution/codex-file-format-spec).

### Module Status

| Spec Section | Status |
|--------------|--------|
| Container Format | Complete |
| Manifest | Complete |
| Content Model | Complete |
| Dublin Core Metadata | Complete |
| Presentation Layers | Complete |
| Asset Management | Complete |
| Digital Signatures | Complete (ECDSA, EdDSA, ML-DSA) |
| Document Hashing | Complete |
| State Machine | Complete |
| Encryption | Complete (AES-256-GCM) |
| Merkle Proofs | Complete |
| Provenance & Lineage | Complete |
| RFC 3161 Timestamps | Complete |
| OpenTimestamps (Bitcoin) | Complete |
| Ethereum Timestamps | Types only (network client optional) |
| Certificate Revocation | Complete (OCSP/CRL) |
| JSON Schema Validation | Complete |

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed development setup, building, testing, and contribution guidelines.

### Quick Start

```bash
cargo build --all-features    # Build
cargo test --all-features     # Test
cargo clippy --all-features   # Lint
```

## API Documentation

See the [API documentation on docs.rs](https://docs.rs/cdx-core) for detailed information about all types and functions.

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting PRs.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
