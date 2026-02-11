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

    let doc = Document::open(file)
        .with_context(|| format!("Failed to open document: {}", file.display()))?;

    let report = doc.verify().context("Verification failed")?;
    let mut all_valid = report.is_valid();

    let loaded_keys = load_keys(key_paths, config);
    let (signature_results, verification_results) =
        verify_signatures(&doc, &report, &loaded_keys, &mut all_valid)?;

    if config.json {
        display_json_verification(&doc, file, all_valid, &verification_results)?;
    } else {
        display_text_verification(
            &doc,
            file,
            &report,
            &signature_results,
            &loaded_keys,
            key_paths,
            all_valid,
            config,
        );
    }

    if all_valid {
        Ok(())
    } else {
        anyhow::bail!("Verification failed")
    }
}

fn load_keys(key_paths: &[PathBuf], config: &OutputConfig) -> Vec<LoadedKey> {
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
    loaded_keys
}

fn verify_signatures(
    doc: &Document,
    report: &cdx_core::VerificationReport,
    loaded_keys: &[LoadedKey],
    all_valid: &mut bool,
) -> Result<(Vec<SignatureVerificationResult>, Vec<serde_json::Value>)> {
    let mut verification_results = vec![serde_json::json!({
        "check": "integrity",
        "valid": report.is_valid(),
        "document_id_valid": report.id_valid,
        "content_valid": report.content_valid,
        "errors": report.errors
    })];

    let signatures = doc.signatures();
    let mut signature_results = Vec::new();

    if !signatures.is_empty() {
        let doc_id = doc.compute_id().context("Failed to compute document ID")?;

        for signature in signatures {
            if loaded_keys.is_empty() {
                signature_results.push(SignatureVerificationResult {
                    signature_id: signature.id.clone(),
                    algorithm: signature.algorithm,
                    signer_name: signature.signer.name.clone(),
                    valid: false,
                    matched_key: None,
                    error: Some("No public keys provided for verification".to_string()),
                });
            } else {
                let result = verify_signature_with_keys(&doc_id, signature, loaded_keys);
                if !result.valid {
                    *all_valid = false;
                }
                signature_results.push(result);
            }
        }

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

    Ok((signature_results, verification_results))
}

fn display_json_verification(
    doc: &Document,
    file: &Path,
    all_valid: bool,
    verification_results: &[serde_json::Value],
) -> Result<()> {
    let result = serde_json::json!({
        "file": file.display().to_string(),
        "document_id": doc.id().to_string(),
        "all_valid": all_valid,
        "checks": verification_results
    });
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn display_text_verification(
    doc: &Document,
    file: &Path,
    report: &cdx_core::VerificationReport,
    signature_results: &[SignatureVerificationResult],
    loaded_keys: &[LoadedKey],
    key_paths: &[PathBuf],
    all_valid: bool,
    config: &OutputConfig,
) {
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
    for error in &report.errors {
        println!("  {} {}", "•".red(), error);
    }

    let signatures = doc.signatures();
    if !signatures.is_empty() {
        config.section("Signatures");
        println!(
            "  {} signature(s) found, {} key(s) provided",
            signatures.len(),
            loaded_keys.len()
        );

        for result in signature_results {
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
                println!("      Matched key: {key}");
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
    }
}
