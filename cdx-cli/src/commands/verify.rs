//! Verify command implementation.

use anyhow::{Context, Result};
use cdx_core::Document;
use colored::Colorize;
use std::path::PathBuf;

use crate::output::OutputConfig;

pub fn run(file: PathBuf, key_paths: Vec<PathBuf>, config: &OutputConfig) -> Result<()> {
    config.verbose(&format!("Verifying: {}", file.display()));

    // Open the document
    let doc = Document::open(&file)
        .with_context(|| format!("Failed to open document: {}", file.display()))?;

    // Verify document integrity
    let report = doc.verify().context("Verification failed")?;

    let all_valid = report.is_valid();
    let mut verification_results = Vec::new();

    // Check document integrity
    verification_results.push(serde_json::json!({
        "check": "integrity",
        "valid": report.is_valid(),
        "document_id_valid": report.id_valid,
        "content_valid": report.content_valid,
        "errors": report.errors
    }));

    // Signature verification would require reading the signatures file
    // and verifying with the provided public keys
    // For now, we just verify document integrity

    if !key_paths.is_empty() {
        config.verbose(&format!(
            "Public keys provided: {} (signature verification not yet implemented)",
            key_paths.len()
        ));
        verification_results.push(serde_json::json!({
            "check": "signatures",
            "note": "Signature verification not yet fully implemented",
            "keys_provided": key_paths.len()
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

    // Note about signature verification
    if !key_paths.is_empty() {
        config.section("Signatures");
        config.info(&format!(
            "{} public key(s) provided - signature verification not yet implemented",
            key_paths.len()
        ));
    }

    println!();

    if all_valid {
        config.success("Document integrity verified");
        Ok(())
    } else {
        anyhow::bail!("Verification failed")
    }
}
