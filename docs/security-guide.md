# Security Best Practices Guide

This guide covers security features in cdx-core: digital signatures, encryption, certificate validation, revocation checking, access control, and post-quantum readiness.

## Algorithm Selection

cdx-core supports multiple signature algorithms via feature flags:

| Algorithm | Feature Flag | Key Size | Recommended For |
|-----------|-------------|----------|-----------------|
| **ES256** (P-256 ECDSA) | `signatures` (default) | 256-bit | General use, broadest compatibility |
| **ES384** (P-384 ECDSA) | `signatures-es384` | 384-bit | Higher security margin |
| **EdDSA** (Ed25519) | `eddsa` | 256-bit | Performance-sensitive, modern systems |
| **PS256** (RSA-PSS) | `signatures-rsa` | 2048-4096-bit | Legacy interop, enterprise PKI |
| **ML-DSA-65** (FIPS 204) | `ml-dsa` | 4032-byte SK | Post-quantum preparedness |

**Recommendations:**
- Use **ES256** for maximum interoperability (required by spec, always available).
- Use **EdDSA** when performance matters and peers support it.
- Use **ML-DSA-65** only for forward-looking deployments; key/signature sizes are significantly larger.
- Avoid **PS256** unless integrating with systems that require RSA.

## Signing Documents

### Key Generation

Every signer implementation provides a `generate()` method that returns the signer and the public key:

```rust
use cdx_core::security::{EcdsaSigner, SignerInfo};

let info = SignerInfo::new("Alice Author")
    .with_email("alice@example.com")
    .with_organization("Example Corp");

let (signer, public_key_pem) = EcdsaSigner::generate(info)?;

// Store public_key_pem for verification
// Keep the signer in memory or serialize the private key securely
```

### Importing Existing Keys

Load a PEM-encoded private key:

```rust
let signer = EcdsaSigner::from_pem(pem_string, signer_info)?;
```

For ML-DSA-65, keys are raw bytes rather than PEM:

```rust
use cdx_core::security::MlDsaSigner;

let signer = MlDsaSigner::from_bytes(&secret_key_bytes, signer_info)?;
```

### Signing

All signers implement the `Signer` trait:

```rust
use cdx_core::security::Signer;

// document_id must not be pending
let signature = signer.sign(&document_id)?;
```

Documents with a `pending` ID cannot be signed. Transition to `review` state first to compute the document ID.

### Verification

```rust
use cdx_core::security::{EcdsaVerifier, Verifier};

let verifier = EcdsaVerifier::from_pem(&public_key_pem)?;
let result = verifier.verify(&document_id, &signature)?;

if result.is_valid() {
    // Signature is cryptographically valid
}
```

## Signature Scope and Layout Attestation

`SignatureScope` binds a signature to both the document content and optionally its presentation layouts:

```rust
use cdx_core::security::SignatureScope;

let scope = SignatureScope::new(document_id)
    .with_layout("presentation/print.css", layout_hash);

// Serialize with JCS (RFC 8785) for deterministic signing
let canonical_bytes = scope.to_jcs()?;
```

This ensures that if a presentation layer is tampered with, signatures attesting to that layout become invalid.

## Managing Signatures

`SignatureFile` stores all signatures for a document:

```rust
use cdx_core::security::SignatureFile;

let mut sig_file = SignatureFile::new(document_id);
sig_file.add_signature(signature);

// Serialize/deserialize
let json = sig_file.to_json()?;
let restored = SignatureFile::from_json(&json)?;

// Look up by ID
if let Some(sig) = sig_file.find_signature("sig-abc123") {
    // ...
}
```

## Certificate Validation

### Building a Certificate Chain

```rust
use cdx_core::security::{CertificateChain, CertificateInfo};

let chain = CertificateChain {
    certificates: vec![leaf_cert, intermediate_cert, root_cert],
};

// Structural validation (offline, no network)
let validation = chain.validate_structure();
if !validation.valid {
    for error in &validation.errors {
        eprintln!("Chain error: {error}");
    }
}
```

### Trust Anchor Validation

```rust
let trusted_roots: Vec<CertificateInfo> = load_trusted_roots();
let validation = chain.validate_trust(&trusted_roots);
```

Trust matching is by SHA-256 fingerprint. The chain must end at (or contain) a certificate whose fingerprint matches a trusted root.

### Key Usage Enforcement

Check that a certificate's key usage permits document signing:

```rust
use cdx_core::security::KeyUsage;

let leaf = chain.leaf().unwrap();
if !leaf.key_usage.contains(&KeyUsage::DigitalSignature) {
    // Certificate not authorized for signing
}
```

For extended key usage, check the `DOCUMENT_SIGNING` OID:

```rust
use cdx_core::security::eku;

if !leaf.extended_key_usage.contains(&eku::DOCUMENT_SIGNING.to_string()) {
    // Not a document signing certificate
}
```

## Revocation Checking

Requires the `ocsp` feature flag. Revocation checking is async and requires network access.

```rust
use cdx_core::security::{RevocationChecker, RevocationConfig, RevocationStatus};
use std::time::Duration;

let config = RevocationConfig {
    timeout: Duration::from_secs(10),
    prefer_ocsp: true,
    strict_mode: false,    // false = treat network errors as "unknown" not "revoked"
    max_crl_age: 86400,    // Cache CRLs for 24 hours
    ..Default::default()
};

let checker = RevocationChecker::new(config);

// Check using OCSP (preferred) with CRL fallback
let result = checker.check(&cert_der, Some(&issuer_der)).await?;

match result.status {
    RevocationStatus::Good => { /* Certificate is valid */ }
    RevocationStatus::Revoked { reason, .. } => { /* Reject */ }
    RevocationStatus::Unknown => { /* Policy decision: accept or reject */ }
    RevocationStatus::Error { message } => { /* Network or parsing error */ }
}
```

### Revocation Reasons

When a certificate is revoked, `RevocationReason` indicates why:

| Reason | Code | Meaning |
|--------|------|---------|
| `KeyCompromise` | 1 | Private key was compromised |
| `CaCompromise` | 2 | CA's key was compromised |
| `AffiliationChanged` | 3 | Subject's affiliation changed |
| `Superseded` | 4 | Replaced by a new certificate |
| `CessationOfOperation` | 5 | No longer in use |
| `CertificateHold` | 6 | Temporarily suspended |

## Encryption

Requires the `encryption` feature flag. Two AEAD algorithms are supported:

| Algorithm | Feature Flag | Key Size | Nonce | Tag |
|-----------|-------------|----------|-------|-----|
| AES-256-GCM | `encryption` | 32 bytes | 12 bytes | 16 bytes |
| ChaCha20-Poly1305 | `encryption-chacha` | 32 bytes | 12 bytes | 16 bytes |

### Encrypting Content

```rust
use cdx_core::security::Aes256GcmEncryptor;

// Generate a random key
let key = Aes256GcmEncryptor::generate_key();
let encryptor = Aes256GcmEncryptor::new(&key)?;

let encrypted = encryptor.encrypt(plaintext)?;
// encrypted.ciphertext, encrypted.nonce, encrypted.tag

let decrypted = encryptor.decrypt(&encrypted.ciphertext, &encrypted.nonce)?;
assert_eq!(decrypted, plaintext);
```

### Password-Based Key Derivation

For password-protected documents, derive the encryption key using a KDF:

```rust
use cdx_core::security::{KdfAlgorithm, KeyDerivation};

// Argon2id is recommended for password-based derivation
let kdf = KeyDerivation {
    algorithm: KdfAlgorithm::Argon2id,
    salt: base64_encoded_salt,
    iterations: None,
    memory: Some(65536),       // 64 MiB
    parallelism: Some(4),
};
```

- Use **Argon2id** for password-based encryption (memory-hard, resists GPU attacks).
- Use **PBKDF2** only when Argon2id is unavailable in the target environment.

## Access Control

Access control policies define per-principal permissions on a document:

```rust
use cdx_core::security::{AccessControl, Principal, Permissions, PermissionGrant, Operation};

let acl = AccessControl {
    default: Permissions::read_only(),
    grants: vec![
        PermissionGrant::full_access_for_user("admin@example.com"),
        PermissionGrant::reviewer_for_user("reviewer@example.com"),
        PermissionGrant::editor_for_user("editor@example.com"),
    ],
};

let principal = Principal::User("reviewer@example.com".into());
assert!(acl.can(&principal, Operation::Annotate));
assert!(!acl.can(&principal, Operation::Edit));
```

### Permission Presets

| Preset | view | print | copy | annotate | edit | sign | decrypt |
|--------|------|-------|------|----------|------|------|---------|
| `none()` | - | - | - | - | - | - | - |
| `read_only()` | x | - | - | - | - | - | - |
| `view_and_print()` | x | x | - | - | - | - | - |
| `reviewer()` | x | x | x | x | - | x | x |
| `editor()` | x | x | x | x | x | - | x |
| `all()` | x | x | x | x | x | x | x |

### Principal Types

- `Principal::User("email")` - Individual user
- `Principal::Group("group-id")` - Group membership
- `Principal::Role("admin")` - Role-based
- `Principal::Everyone` - Wildcard, matches all principals

## WebAuthn / FIDO2

Requires the `webauthn` feature flag. Enables browser-based signing with hardware security keys.

```rust
use cdx_core::security::WebAuthnVerifier;

let verifier = WebAuthnVerifier::new("https://example.com", &public_key_bytes)?
    .with_credential_id(credential_id);

let result = verifier.verify(&document_id, &signature)?;
```

The verification process:
1. Decodes base64 fields from the `WebAuthnSignature` struct
2. Validates the challenge matches the document ID
3. Checks origin matches the expected origin
4. Verifies user presence flag in authenticator data
5. Verifies the ECDSA (P-256) signature over `authenticatorData || SHA-256(clientDataJSON)`

## Key Management Best Practices

1. **Never store private keys in the document archive.** Keys should be managed externally (HSM, OS keychain, vault).

2. **Use separate keys for signing and encryption.** A signing key compromise doesn't automatically compromise encrypted content.

3. **Include `SignerInfo` metadata.** Always provide `name` and ideally `email` and `organization` so verifiers can identify the signer.

4. **Attach certificates when available.** Set `signer_info.certificate` to the PEM-encoded X.509 certificate for chain validation.

5. **Rotate keys regularly.** Use the `key_id` field to track which key version was used.

6. **Verify before trusting.** Always verify signatures against a known public key or trusted certificate chain. A signature alone does not prove identity.

7. **Check revocation for high-assurance workflows.** Enable the `ocsp` feature and check certificate revocation before accepting signatures on frozen/published documents.

## Feature Flag Reference

| Feature | What It Enables |
|---------|----------------|
| `signatures` | ES256 signing/verification (default, always on) |
| `signatures-es384` | ES384 signing/verification |
| `eddsa` | EdDSA (Ed25519) signing/verification |
| `signatures-rsa` | PS256 (RSA-PSS) signing/verification |
| `ml-dsa` | ML-DSA-65 post-quantum signing/verification |
| `encryption` | AES-256-GCM encryption/decryption |
| `encryption-chacha` | ChaCha20-Poly1305 encryption/decryption |
| `ocsp` | Online revocation checking (OCSP + CRL) |
| `webauthn` | WebAuthn/FIDO2 signature verification |
