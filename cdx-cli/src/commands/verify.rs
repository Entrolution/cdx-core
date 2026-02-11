//! Verify command implementation.

use anyhow::{Context, Result};
use cdx_core::security::{
    EcdsaVerifier, EddsaVerifier, Signature, SignatureAlgorithm, SignatureVerification, Verifier,
};
use cdx_core::{Document, DocumentId};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::output::OutputConfig;

/// Loaded public key with its PEM content
struct LoadedKey {
    path: PathBuf,
    pem: String,
}

/// Try to verify a signature with a given public key
fn try_verify_signature(
    doc_id: &DocumentId,
    signature: &Signature,
    key_pem: &str,
) -> Option<SignatureVerification> {
    match signature.algorithm {
        SignatureAlgorithm::ES256 | SignatureAlgorithm::ES384 => {
            // Try ECDSA verifier
            if let Ok(verifier) = EcdsaVerifier::from_pem(key_pem) {
                if let Ok(result) = verifier.verify(doc_id, signature) {
                    return Some(result);
                }
            }
            None
        }
        SignatureAlgorithm::EdDSA => {
            // Try EdDSA verifier
            if let Ok(verifier) = EddsaVerifier::from_pem(key_pem) {
                if let Ok(result) = verifier.verify(doc_id, signature) {
                    return Some(result);
                }
            }
            None
        }
        _ => {
            // Unsupported algorithm
            None
        }
    }
}

/// Verify a single signature against all provided keys
fn verify_signature_with_keys(
    doc_id: &DocumentId,
    signature: &Signature,
    keys: &[LoadedKey],
) -> SignatureVerificationResult {
    // Try each key until we find one that works
    for key in keys {
        if let Some(result) = try_verify_signature(doc_id, signature, &key.pem) {
            if result.is_valid() {
                return SignatureVerificationResult {
                    signature_id: signature.id.clone(),
                    algorithm: signature.algorithm,
                    signer_name: signature.signer.name.clone(),
                    valid: true,
                    matched_key: Some(key.path.display().to_string()),
                    error: None,
                };
            }
        }
    }

    // No key verified the signature
    SignatureVerificationResult {
        signature_id: signature.id.clone(),
        algorithm: signature.algorithm,
        signer_name: signature.signer.name.clone(),
        valid: false,
        matched_key: None,
        error: Some("No matching public key found or signature invalid".to_string()),
    }
}

struct SignatureVerificationResult {
    signature_id: String,
    algorithm: SignatureAlgorithm,
    signer_name: String,
    valid: bool,
    matched_key: Option<String>,
    error: Option<String>,
}

pub fn run(file: &Path, key_paths: &[PathBuf], config: &OutputConfig) -> Result<()> {
    config.verbose(&format!("Verifying: {}", file.display()));

    // Open the document
    let doc = Document::open(file)
        .with_context(|| format!("Failed to open document: {}", file.display()))?;

    // Verify document integrity
    let report = doc.verify().context("Verification failed")?;

    let mut all_valid = report.is_valid();
    let mut verification_results = Vec::new();

    // Check document integrity
    verification_results.push(serde_json::json!({
        "check": "integrity",
        "valid": report.is_valid(),
        "document_id_valid": report.id_valid,
        "content_valid": report.content_valid,
        "errors": report.errors
    }));

    // Load public keys
    let mut loaded_keys = Vec::new();
    for key_path in key_paths {
        match fs::read_to_string(key_path) {
            Ok(pem) => {
                config.verbose(&format!("Loaded key: {}", key_path.display()));
                loaded_keys.push(LoadedKey {
                    path: key_path.clone(),
                    pem,
                });
            }
            Err(e) => {
                config.warning(&format!("Failed to load key {}: {}", key_path.display(), e));
            }
        }
    }

    // Verify signatures
    let signatures = doc.signatures();
    let mut signature_results = Vec::new();

    if !signatures.is_empty() {
        let doc_id = doc.compute_id().context("Failed to compute document ID")?;

        for signature in signatures {
            if loaded_keys.is_empty() {
                // No keys provided, mark as unverified
                signature_results.push(SignatureVerificationResult {
                    signature_id: signature.id.clone(),
                    algorithm: signature.algorithm,
                    signer_name: signature.signer.name.clone(),
                    valid: false,
                    matched_key: None,
                    error: Some("No public keys provided for verification".to_string()),
                });
            } else {
                let result = verify_signature_with_keys(&doc_id, signature, &loaded_keys);
                if !result.valid {
                    all_valid = false;
                }
                signature_results.push(result);
            }
        }

        // Build JSON for signature results
        let sig_json: Vec<_> = signature_results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "signature_id": r.signature_id,
                    "algorithm": r.algorithm.as_str(),
                    "signer": r.signer_name,
                    "valid": r.valid,
                    "matched_key": r.matched_key,
                    "error": r.error
                })
            })
            .collect();

        verification_results.push(serde_json::json!({
            "check": "signatures",
            "signature_count": signatures.len(),
            "keys_provided": loaded_keys.len(),
            "results": sig_json
        }));
    }

    if config.json {
        let result = serde_json::json!({
            "file": file.display().to_string(),
            "document_id": doc.id().to_string(),
            "all_valid": all_valid,
            "checks": verification_results
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
        return if all_valid {
            Ok(())
        } else {
            anyhow::bail!("Verification failed")
        };
    }

    // Human-readable output
    config.field("File", &file.display().to_string());
    config.field("Document ID", &doc.id().to_string());

    config.section("Integrity");
    if report.id_valid {
        println!("{} Document ID verified", "✓".green());
    } else {
        println!("{} Document ID verification failed", "✗".red());
    }

    if report.content_valid {
        println!("{} Content verified", "✓".green());
    } else {
        println!("{} Content verification failed", "✗".red());
    }

    if !report.errors.is_empty() {
        for error in &report.errors {
            println!("  {} {}", "•".red(), error);
        }
    }

    // Signature verification output
    if !signatures.is_empty() {
        config.section("Signatures");
        println!(
            "  {} signature(s) found, {} key(s) provided",
            signatures.len(),
            loaded_keys.len()
        );

        for result in &signature_results {
            let status = if result.valid {
                format!("{} Valid", "✓".green())
            } else {
                format!("{} Invalid", "✗".red())
            };

            println!(
                "  {} [{}] {} ({})",
                status, result.signature_id, result.signer_name, result.algorithm
            );

            if let Some(ref key) = result.matched_key {
                println!("      Matched key: {}", key);
            }
            if let Some(ref error) = result.error {
                println!("      {}", error.red());
            }
        }
    } else if !key_paths.is_empty() {
        config.section("Signatures");
        println!("  No signatures found in document");
    }

    println!();

    if all_valid {
        if signatures.is_empty() {
            config.success("Document integrity verified (no signatures present)");
        } else {
            config.success("Document verified successfully");
        }
        Ok(())
    } else {
        anyhow::bail!("Verification failed")
    }
}
