//! Status command implementation.

use anyhow::{Context, Result};
use cdx_core::Document;
use colored::Colorize;
use std::path::Path;

use crate::output::OutputConfig;

/// Display comprehensive document status.
pub fn run(file: &Path, config: &OutputConfig) -> Result<()> {
    config.verbose(&format!("Checking status of: {}", file.display()));

    let doc = Document::open(file)
        .with_context(|| format!("Failed to open document: {}", file.display()))?;

    if config.json {
        display_json_status(&doc, file)
    } else {
        display_text_status(&doc, file, config);
        Ok(())
    }
}

fn display_json_status(doc: &Document, file: &Path) -> Result<()> {
    let manifest = doc.manifest();
    let dc = doc.dublin_core();
    let has_lineage = manifest.lineage.is_some();
    let has_signatures = doc.has_signatures();
    let has_precise_layout = manifest.has_precise_layout();
    let integrity_ok = doc
        .verify()
        .ok()
        .as_ref()
        .is_some_and(cdx_core::VerificationReport::is_valid);
    let merkle_root = doc.merkle_root().ok();
    let can_freeze = has_signatures && has_lineage && has_precise_layout;

    let status = serde_json::json!({
        "file": file.display().to_string(),
        "document": {
            "id": doc.id().to_string(),
            "state": doc.state().to_string(),
            "title": dc.title(),
            "spec_version": manifest.cdx,
        },
        "integrity": {
            "valid": integrity_ok,
            "content_hash": manifest.content.hash.to_string(),
            "merkle_root": merkle_root.map(|r| r.to_string()),
        },
        "content": {
            "block_count": doc.content().len(),
            "has_presentation": !manifest.presentation.is_empty(),
            "presentation_count": manifest.presentation.len(),
        },
        "security": {
            "has_signatures": has_signatures,
            "signature_count": doc.signatures().len(),
            "is_encrypted": doc.is_encrypted(),
        },
        "lineage": {
            "has_lineage": has_lineage,
            "parent": manifest.lineage.as_ref().and_then(|l| l.parent.as_ref().map(std::string::ToString::to_string)),
            "version": manifest.lineage.as_ref().and_then(|l| l.version),
            "depth": manifest.lineage.as_ref().and_then(|l| l.depth),
        },
        "requirements": {
            "has_precise_layout": has_precise_layout,
            "can_freeze": can_freeze,
            "can_publish": doc.state() == cdx_core::DocumentState::Frozen,
        },
        "timestamps": {
            "created": manifest.created.to_rfc3339(),
            "modified": manifest.modified.to_rfc3339(),
        }
    });
    println!("{}", serde_json::to_string_pretty(&status)?);
    Ok(())
}

fn display_text_status(doc: &Document, file: &Path, config: &OutputConfig) {
    let manifest = doc.manifest();
    let dc = doc.dublin_core();
    let has_signatures = doc.has_signatures();
    let is_encrypted = doc.is_encrypted();
    let has_precise_layout = manifest.has_precise_layout();
    let verification = doc.verify().ok();
    let integrity_ok = verification
        .as_ref()
        .is_some_and(cdx_core::VerificationReport::is_valid);
    let merkle_root = doc.merkle_root().ok();
    let block_count = doc.content().len();

    println!("\n{}", "Document Status".blue().bold());
    println!("{}", "═".repeat(60).blue());

    config.field("File", &file.display().to_string());
    config.field("Title", dc.title());
    config.field("Document ID", &doc.id().to_string());

    let state_str = match doc.state() {
        cdx_core::DocumentState::Draft => "Draft".yellow().to_string(),
        cdx_core::DocumentState::Review => "Review".cyan().to_string(),
        cdx_core::DocumentState::Frozen => "Frozen".blue().to_string(),
        cdx_core::DocumentState::Published => "Published".green().to_string(),
    };
    println!("{}: {}", "State".bold(), state_str);

    config.section("Integrity");
    if integrity_ok {
        println!("  {} Content hash verified", "✓".green());
        println!("  {} Document ID verified", "✓".green());
    } else if let Some(report) = verification {
        if !report.content_valid {
            println!("  {} Content hash mismatch", "✗".red());
        }
        if !report.id_valid {
            println!("  {} Document ID mismatch", "✗".red());
        }
        for err in &report.errors {
            println!("    {}", err.dimmed());
        }
    } else {
        println!("  {} Could not verify integrity", "?".yellow());
    }

    if let Some(root) = merkle_root {
        config.field("Merkle Root", &root.to_string());
    }

    config.section("Content");
    config.field("Block Count", &block_count.to_string());
    config.field(
        "Presentations",
        &format!("{} defined", manifest.presentation.len()),
    );
    if has_precise_layout {
        println!("  {} Has precise layout", "✓".green());
    } else {
        println!("  {} No precise layout", "○".dimmed());
    }

    config.section("Security");
    if has_signatures {
        let sig_count = doc.signatures().len();
        println!(
            "  {} {} signature{}",
            "✓".green(),
            sig_count,
            if sig_count == 1 { "" } else { "s" }
        );
    } else {
        println!("  {} No signatures", "○".dimmed());
    }
    if is_encrypted {
        println!("  {} Encrypted", "✓".green());
    } else {
        println!("  {} Not encrypted", "○".dimmed());
    }

    config.section("Lineage");
    if let Some(ref lineage) = manifest.lineage {
        if let Some(ref parent) = lineage.parent {
            config.field("Parent", &parent.to_string());
        } else {
            println!("  {} Root document", "✓".green());
        }
        if let Some(version) = lineage.version {
            config.field("Version", &version.to_string());
        }
        if let Some(depth) = lineage.depth {
            config.field("Depth", &depth.to_string());
        }
        if !lineage.ancestors.is_empty() {
            config.field("Ancestor Count", &lineage.ancestors.len().to_string());
        }
    } else {
        println!("  {} No lineage set", "○".dimmed());
    }

    display_state_transitions(doc, config);

    config.section("Timestamps");
    config.field("Created", &manifest.created.to_rfc3339());
    config.field("Modified", &manifest.modified.to_rfc3339());

    println!();
}

fn display_state_transitions(doc: &Document, config: &OutputConfig) {
    let manifest = doc.manifest();
    let has_signatures = doc.has_signatures();
    let has_lineage = manifest.lineage.is_some();
    let has_precise_layout = manifest.has_precise_layout();
    let can_freeze = has_signatures && has_lineage && has_precise_layout;

    config.section("State Transitions");
    match doc.state() {
        cdx_core::DocumentState::Draft => {
            println!("  {} Can submit for review", "→".cyan());
        }
        cdx_core::DocumentState::Review => {
            if can_freeze {
                println!("  {} Ready to freeze", "→".cyan());
            } else {
                println!("  {} Cannot freeze yet:", "!".yellow());
                if !has_signatures {
                    println!("    - Missing signatures");
                }
                if !has_lineage {
                    println!("    - Missing lineage");
                }
                if !has_precise_layout {
                    println!("    - Missing precise layout");
                }
            }
            if !has_signatures {
                println!("  {} Can revert to draft", "←".dimmed());
            }
        }
        cdx_core::DocumentState::Frozen => {
            println!("  {} Ready to publish", "→".green());
        }
        cdx_core::DocumentState::Published => {
            println!("  {} Final state (can fork)", "■".green());
        }
    }
}
