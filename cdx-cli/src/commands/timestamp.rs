//! Timestamp command implementations.
//!
//! Commands for managing timestamp records in Codex documents.

use anyhow::{Context, Result};
use cdx_core::provenance::{TimestampMethod, TimestampRecord};
use cdx_core::Document;
use chrono::{DateTime, Utc};
use colored::Colorize;
use std::path::PathBuf;

use crate::output::OutputConfig;

/// Show timestamps in a document.
pub fn run_show_timestamps(file: PathBuf, config: &OutputConfig) -> Result<()> {
    config.verbose(&format!("Showing timestamps for: {}", file.display()));

    let doc = Document::open(&file)
        .with_context(|| format!("Failed to open document: {}", file.display()))?;

    let record = doc
        .provenance_record()
        .with_context(|| "Failed to generate provenance record")?;

    if config.json {
        let output = serde_json::json!({
            "document_id": doc.id().to_string(),
            "timestamp_count": record.timestamps.len(),
            "timestamps": record.timestamps.iter().map(|ts| serde_json::json!({
                "method": format!("{:?}", ts.method).to_lowercase(),
                "authority": ts.authority,
                "time": ts.time.to_rfc3339(),
                "token_preview": truncate_token(&ts.token, 32),
                "transaction_id": ts.transaction_id,
            })).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    println!("\n{}", "Document Timestamps".blue().bold());
    println!("{}", "═".repeat(60).blue());

    config.field("Document ID", &doc.id().to_string());

    if record.timestamps.is_empty() {
        println!("\n{}", "No timestamps recorded".dimmed());
    } else {
        println!(
            "\n{} {} timestamp{}:",
            "Found".green(),
            record.timestamps.len(),
            if record.timestamps.len() == 1 {
                ""
            } else {
                "s"
            }
        );

        for (i, ts) in record.timestamps.iter().enumerate() {
            println!("\n{}. {}", i + 1, format!("{}", ts.method).cyan());
            config.field("  Authority", &ts.authority);
            config.field("  Time", &ts.time.to_rfc3339());
            config.field("  Token", &truncate_token(&ts.token, 48));
            if let Some(ref tx_id) = ts.transaction_id {
                config.field("  Transaction", tx_id);
            }
        }
    }

    println!();
    Ok(())
}

/// Verify timestamps in a document.
pub fn run_verify_timestamps(file: PathBuf, config: &OutputConfig) -> Result<()> {
    config.verbose(&format!("Verifying timestamps for: {}", file.display()));

    let doc = Document::open(&file)
        .with_context(|| format!("Failed to open document: {}", file.display()))?;

    let record = doc
        .provenance_record()
        .with_context(|| "Failed to generate provenance record")?;

    if record.timestamps.is_empty() {
        if config.json {
            let output = serde_json::json!({
                "document_id": doc.id().to_string(),
                "verified": false,
                "error": "No timestamps found",
                "results": [],
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            config.warning("No timestamps found in document");
        }
        return Ok(());
    }

    let mut all_valid = true;
    let mut results = Vec::new();

    for (i, ts) in record.timestamps.iter().enumerate() {
        // Basic validation - check token is not empty and matches document
        let token_valid = !ts.token.is_empty();
        let matches = ts.matches_document(&record.document_id);
        let valid = token_valid && matches;

        if !valid {
            all_valid = false;
        }

        results.push(TimestampVerification {
            index: i,
            method: ts.method,
            authority: ts.authority.clone(),
            valid,
            note: if !token_valid {
                Some("Empty token".to_string())
            } else if !matches {
                Some("Token does not match document".to_string())
            } else {
                None
            },
        });
    }

    if config.json {
        let output = serde_json::json!({
            "document_id": doc.id().to_string(),
            "verified": all_valid,
            "timestamp_count": record.timestamps.len(),
            "results": results.iter().map(|r| serde_json::json!({
                "index": r.index,
                "method": format!("{:?}", r.method).to_lowercase(),
                "authority": r.authority,
                "valid": r.valid,
                "note": r.note,
            })).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    println!("\n{}", "Timestamp Verification".blue().bold());
    println!("{}", "═".repeat(60).blue());

    for result in &results {
        let status = if result.valid {
            "✓".green()
        } else {
            "✗".red()
        };
        println!(
            "\n{} {}. {} ({})",
            status,
            result.index + 1,
            result.authority,
            result.method
        );
        if let Some(ref note) = result.note {
            println!("    {}", note.yellow());
        }
    }

    println!();
    if all_valid {
        config.success(&format!(
            "All {} timestamps verified",
            record.timestamps.len()
        ));
    } else {
        println!(
            "{} {}",
            "✗".red().bold(),
            "Some timestamps failed verification".red()
        );
        std::process::exit(1);
    }

    Ok(())
}

/// Add a timestamp record to a document.
///
/// Note: This adds a pre-existing timestamp record. To acquire a new timestamp
/// from a TSA, use `cdx timestamp-acquire` (requires network features).
#[allow(clippy::too_many_arguments)]
pub fn run_add_timestamp(
    file: PathBuf,
    method: String,
    authority: String,
    token: String,
    time: Option<String>,
    transaction_id: Option<String>,
    _output: Option<PathBuf>,
    config: &OutputConfig,
) -> Result<()> {
    config.verbose(&format!("Adding timestamp to: {}", file.display()));

    let doc = Document::open(&file)
        .with_context(|| format!("Failed to open document: {}", file.display()))?;

    // Check document state
    if doc.state().is_immutable() {
        anyhow::bail!("Cannot add timestamp: document is in {} state", doc.state());
    }

    // Parse method
    let ts_method = match method.to_lowercase().as_str() {
        "rfc3161" => TimestampMethod::Rfc3161,
        "bitcoin" => TimestampMethod::Bitcoin,
        "ethereum" => TimestampMethod::Ethereum,
        "opentimestamps" | "ots" => TimestampMethod::OpenTimestamps,
        _ => anyhow::bail!(
            "Unknown timestamp method: {}. Valid options: rfc3161, bitcoin, ethereum, opentimestamps",
            method
        ),
    };

    // Parse time
    let ts_time = if let Some(time_str) = time {
        DateTime::parse_from_rfc3339(&time_str)
            .with_context(|| format!("Invalid timestamp format: {time_str}"))?
            .with_timezone(&Utc)
    } else {
        Utc::now()
    };

    // Create timestamp record
    let timestamp = TimestampRecord {
        method: ts_method,
        authority,
        time: ts_time,
        token,
        transaction_id,
    };

    // For now, we can't actually add the timestamp to the document
    // because Document doesn't expose a method to add timestamps to the provenance record.
    // This would need to be added to the Document API.

    // Let's report what would be added
    if config.json {
        let output_json = serde_json::json!({
            "status": "dry_run",
            "message": "Adding timestamps to documents requires provenance record persistence (not yet implemented)",
            "timestamp": {
                "method": format!("{:?}", timestamp.method).to_lowercase(),
                "authority": timestamp.authority,
                "time": timestamp.time.to_rfc3339(),
                "token_preview": truncate_token(&timestamp.token, 32),
                "transaction_id": timestamp.transaction_id,
            }
        });
        println!("{}", serde_json::to_string_pretty(&output_json)?);
    } else {
        config.warning(
            "Adding timestamps requires provenance record persistence (planned for future release)",
        );
        println!("\n{}", "Timestamp to add:".dimmed());
        config.field("  Method", &format!("{}", timestamp.method));
        config.field("  Authority", &timestamp.authority);
        config.field("  Time", &timestamp.time.to_rfc3339());
        config.field("  Token", &truncate_token(&timestamp.token, 48));
        if let Some(ref tx_id) = timestamp.transaction_id {
            config.field("  Transaction", tx_id);
        }
    }

    // Note: Full implementation would:
    // 1. Load or create provenance record
    // 2. Add timestamp to record
    // 3. Save provenance record to document
    // 4. Save document

    Ok(())
}

/// Timestamp verification result.
struct TimestampVerification {
    index: usize,
    method: TimestampMethod,
    authority: String,
    valid: bool,
    note: Option<String>,
}

/// Truncate a token for display.
fn truncate_token(token: &str, max_len: usize) -> String {
    if token.len() <= max_len {
        token.to_string()
    } else {
        format!("{}...", &token[..max_len])
    }
}
