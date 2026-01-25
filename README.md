# cdx-core

Core Rust library for reading, writing, and validating [Codex Document Format](https://github.com/gvonness-apolitical/codex-file-format-spec) (`.cdx`) files.

## Overview

Codex is an open document format designed for:

- **Semantic content** - Structured content blocks, not just rendered output
- **Verifiable integrity** - Content-addressable hashing and digital signatures
- **Machine readability** - JSON-based format with well-defined schemas
- **Modern security** - Built-in support for signatures and encryption

This library provides the foundational capabilities for working with Codex documents in Rust, with support for WASM compilation.

## Features

- Parse and validate `.cdx` archives
- Read and write manifests, content, and metadata
- Compute and verify content-addressable document IDs
- Support for multiple hash algorithms (SHA-256, SHA-3, BLAKE3)
- Zstandard compression support (optional)
- WASM-compatible (with `wasm` feature)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
cdx-core = "0.1"
```

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `zstd` | Yes | Zstandard compression support |
| `wasm` | No | WASM compilation support |

## Usage

```rust
use cdx_core::{Document, DocumentState};

// Open an existing document
let doc = Document::open("example.cdx")?;
println!("Title: {}", doc.metadata().title());
println!("State: {:?}", doc.state());

// Verify document integrity
doc.verify()?;

// Create a new document
let mut builder = Document::builder()
    .title("My Document")
    .creator("Jane Doe");

builder.add_paragraph("Hello, world!");
let doc = builder.build()?;

// Save as draft
doc.save("output.cdx")?;
```

## Specification Compliance

This library implements the [Codex Document Format Specification v0.1](https://github.com/gvonness-apolitical/codex-file-format-spec).

### Core Modules

| Spec Section | Status |
|--------------|--------|
| Container Format | In Progress |
| Manifest | In Progress |
| Content Blocks | Planned |
| Presentation Layers | Planned |
| Asset Embedding | Planned |
| Document Hashing | In Progress |
| State Machine | Planned |
| Metadata | Planned |

## Development

### Prerequisites

- Rust 1.75 or later
- Cargo

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Linting

```bash
cargo clippy --all-features
cargo fmt --check
```

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) before submitting PRs.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
