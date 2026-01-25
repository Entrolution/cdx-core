//! Validate command implementation.

use anyhow::{Context, Result};
use cdx_core::Document;
use colored::Colorize;
use std::path::PathBuf;

use crate::output::OutputConfig;

pub fn run(file: PathBuf, config: &OutputConfig) -> Result<()> {
    config.verbose(&format!("Validating: {}", file.display()));

    // Open the document
    let doc = Document::open(&file)
        .with_context(|| format!("Failed to open document: {}", file.display()))?;

    // Get verification report
    let report = doc.verify().context("Verification failed")?;

    if config.json {
        let result = serde_json::json!({
            "file": file.display().to_string(),
            "valid": report.is_valid(),
            "document_id": doc.id().to_string(),
            "document_id_verified": report.id_valid,
            "content_verified": report.content_valid,
            "errors": report.errors
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    config.field("File", &file.display().to_string());
    config.field("Document ID", &doc.id().to_string());

    // Report results
    let mut has_errors = false;

    // Check document ID
    if report.id_valid {
        config.info(&format!("{} Document ID verified", "✓".green()));
    } else {
        config.info(&format!("{} Document ID verification failed", "✗".red()));
        has_errors = true;
    }

    // Check content hashes
    if report.content_valid {
        config.info(&format!("{} Content verified", "✓".green()));
    } else {
        config.info(&format!("{} Content verification failed", "✗".red()));
        has_errors = true;
    }

    // Check state requirements
    let state = doc.state();
    config.field("State", &state.to_string());

    // Print any errors
    if !report.errors.is_empty() {
        config.section("Errors");
        for error in &report.errors {
            config.info(&format!("{} {}", "•".red(), error));
        }
    }

    if has_errors {
        anyhow::bail!("Document validation failed");
    }

    config.success("Document is valid");
    Ok(())
}
