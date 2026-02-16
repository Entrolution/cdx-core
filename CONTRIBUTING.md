# Contributing to cdx-core

Thank you for your interest in contributing to cdx-core! This document provides guidelines and information for contributors.

## Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). Please be respectful and constructive in all interactions.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/cdx-core.git`
3. Create a branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Run tests: `cargo test`
6. Run lints: `cargo clippy --all-features && cargo fmt --check`
7. Commit your changes
8. Push to your fork and submit a pull request

## Development Setup

### Prerequisites

- Rust 1.88 or later (install via [rustup](https://rustup.rs/))
- Cargo (included with Rust)

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Build with all features
cargo build --all-features
```

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_name

# Run tests with all features
cargo test --all-features
```

### Code Quality

Before submitting a PR, ensure:

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy --all-features -- -D warnings

# Check documentation
cargo doc --no-deps --all-features
```

### Security Audits

Run dependency audits before submitting PRs:

```bash
# Install audit tools (one-time)
cargo install cargo-audit cargo-deny

# Check for known vulnerabilities
cargo audit --ignore RUSTSEC-2023-0071

# Check licenses and advisories
cargo deny check
```

The `RUSTSEC-2023-0071` ignore is for an unmaintained dependency (`rsa` crate) that has no replacement yet.

## Pull Request Guidelines

### Before Submitting

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Code is formatted with `cargo fmt`
- [ ] No clippy warnings
- [ ] Documentation is updated if needed
- [ ] CHANGELOG.md is updated for user-facing changes

### PR Description

Please include:

- **What**: Brief description of the change
- **Why**: Motivation for the change
- **How**: High-level approach (if not obvious)
- **Testing**: How you tested the changes

### Commit Messages

Follow conventional commit format:

```
type(scope): short description

Longer description if needed.

Fixes #123
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

## Architecture Overview

```
src/
├── lib.rs          # Public API and module exports
├── error.rs        # Error types
├── manifest/       # Manifest parsing and validation
├── content/        # Content block types
├── hash/           # Hashing and document ID computation
├── archive/        # ZIP archive handling
└── state/          # Document state machine
```

## Adding New Features

1. **Discuss first**: Open an issue to discuss significant changes
2. **Spec compliance**: Ensure changes align with the Codex specification
3. **Backward compatibility**: Avoid breaking changes unless necessary
4. **Testing**: Add tests for new functionality
5. **Documentation**: Update rustdoc and README as needed

## Specification Reference

This library implements the [Codex Document Format Specification](https://github.com/Entrolution/codex-file-format-spec). When implementing new features:

- Reference the relevant spec section
- Note any deviations or extensions
- Consider edge cases mentioned in the spec

## Questions?

- Open an issue for bugs or feature requests
- Use discussions for general questions

Thank you for contributing!
