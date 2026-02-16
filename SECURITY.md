# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 0.5.x   | Yes       |
| < 0.5   | No        |

Only the latest minor release receives security updates. Earlier versions are not supported.

## Reporting a Vulnerability

If you discover a security vulnerability, please report it responsibly:

1. **Do not** open a public GitHub issue.
2. Email **security@entrolution.com** with details of the vulnerability.
3. Include steps to reproduce, if possible.

We aim to acknowledge reports within 48 hours and provide a fix or mitigation within 7 days for critical issues.

## Security Practices

- All cryptographic operations use well-audited Rust crates (`sha2`, `p256`, `aes-gcm`, `ed25519-dalek`).
- Archive extraction is bounded to prevent decompression bombs (256 MiB limit).
- Path traversal attacks are rejected at the archive reader/writer level.
- Document integrity is verified via SHA-256 content hashes and JCS-canonicalized document IDs.
- Spec conformance is validated by 1,000+ tests covering all 78 testable requirements.
