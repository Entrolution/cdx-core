//! Inspect command implementation.

use anyhow::{Context, Result};
use cdx_core::Document;
use colored::Colorize;
use std::path::PathBuf;

use crate::output::OutputConfig;

pub fn run(
    file: PathBuf,
    show_blocks: bool,
    show_signatures: bool,
    show_provenance: bool,
    config: &OutputConfig,
) -> Result<()> {
    config.verbose(&format!("Inspecting: {}", file.display()));

    // Open the document
    let doc = Document::open(&file)
        .with_context(|| format!("Failed to open document: {}", file.display()))?;

    let manifest = doc.manifest();
    let content = doc.content();
    let dublin_core = doc.dublin_core();

    if config.json {
        let mut result = serde_json::json!({
            "file": file.display().to_string(),
            "document_id": doc.id().to_string(),
            "spec_version": manifest.codex,
            "state": doc.state().to_string(),
            "metadata": {
                "title": dublin_core.title(),
                "creators": dublin_core.creators(),
                "description": dublin_core.description(),
            },
            "content": {
                "block_count": content.len(),
            }
        });

        // Add signature info if requested
        if show_signatures {
            if let Some(security) = &manifest.security {
                result["security"] = serde_json::json!({
                    "has_signatures": security.signatures.is_some(),
                    "is_encrypted": security.encryption.is_some(),
                });
            }
        }

        // Add lineage if requested and present
        if show_provenance {
            if let Some(lineage) = &manifest.lineage {
                result["lineage"] = serde_json::json!({
                    "parent": lineage.parent.as_ref().map(|p| p.to_string()),
                    "version": lineage.version,
                    "branch": lineage.branch,
                    "note": lineage.note,
                });
            }
        }

        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    // Header
    println!("\n{}", "Codex Document".blue().bold());
    println!("{}", "═".repeat(60).blue());

    // Basic info
    config.field("File", &file.display().to_string());
    config.field("Document ID", &doc.id().to_string());
    config.field("Spec Version", &manifest.codex);
    config.field("State", &doc.state().to_string());

    // Metadata
    config.section("Metadata");
    config.field("Title", dublin_core.title());
    let creators = dublin_core.creators();
    if !creators.is_empty() {
        config.field("Creator(s)", &creators.join(", "));
    }
    if let Some(description) = dublin_core.description() {
        config.field("Description", description);
    }
    if let Some(language) = dublin_core.language() {
        config.field("Language", language);
    }

    // Content summary
    config.section("Content");
    config.field("Block Count", &content.len().to_string());

    if show_blocks {
        println!("\n{}", "Blocks:".dimmed());
        for (i, block) in content.blocks.iter().enumerate() {
            println!(
                "  {}. {}",
                i + 1,
                format_block_type(block.block_type()).cyan(),
            );
        }
    }

    // Security info
    if show_signatures {
        if let Some(security) = &manifest.security {
            config.section("Security");
            if security.signatures.is_some() {
                config.field("Signatures", "Yes");
            }
            if security.encryption.is_some() {
                config.field("Encrypted", "Yes");
            }
        }
    }

    // Provenance/lineage
    if show_provenance {
        if let Some(lineage) = &manifest.lineage {
            config.section("Provenance");
            if let Some(parent) = &lineage.parent {
                config.field("Parent", &parent.to_string());
            }
            if let Some(version) = lineage.version {
                config.field("Version", &version.to_string());
            }
            if let Some(branch) = &lineage.branch {
                config.field("Branch", branch);
            }
            if let Some(note) = &lineage.note {
                config.field("Note", note);
            }
        }
    }

    println!();
    Ok(())
}

fn format_block_type(block_type: &str) -> String {
    match block_type {
        "paragraph" => "Paragraph",
        "heading" => "Heading",
        "list" => "List",
        "listItem" => "List Item",
        "blockquote" => "Blockquote",
        "codeBlock" => "Code Block",
        "horizontalRule" => "Horizontal Rule",
        "image" => "Image",
        "table" => "Table",
        "tableRow" => "Table Row",
        "tableCell" => "Table Cell",
        "math" => "Math",
        "break" => "Break",
        other => other,
    }
    .to_string()
}
