//! Create command implementation.

use anyhow::{Context, Result};
use cdx_core::{Document, DocumentState};
use std::fs;
use std::path::PathBuf;

use crate::output::OutputConfig;

pub fn run(
    title: String,
    authors: Vec<String>,
    state: String,
    input: Option<PathBuf>,
    output: PathBuf,
    config: &OutputConfig,
) -> Result<()> {
    config.verbose(&format!("Creating document: {}", title));

    // Parse state
    let doc_state = match state.to_lowercase().as_str() {
        "draft" => DocumentState::Draft,
        "review" => DocumentState::Review,
        "frozen" => DocumentState::Frozen,
        "published" => DocumentState::Published,
        _ => {
            anyhow::bail!(
                "Invalid state '{}'. Valid states: draft, review, frozen, published",
                state
            );
        }
    };

    // Read content from input file if provided
    let content_text = if let Some(input_path) = input {
        config.verbose(&format!("Reading content from: {}", input_path.display()));
        fs::read_to_string(&input_path)
            .with_context(|| format!("Failed to read input file: {}", input_path.display()))?
    } else {
        title.clone()
    };

    // Convert content to blocks (simple paragraph parsing)
    let paragraphs: Vec<&str> = content_text
        .split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .collect();

    config.verbose(&format!("Content has {} paragraphs", paragraphs.len()));

    // Build the document
    let creator = if authors.is_empty() {
        "Unknown".to_string()
    } else {
        authors.join(", ")
    };

    let mut builder = Document::builder()
        .title(&title)
        .creator(&creator)
        .state(doc_state);

    for paragraph in paragraphs {
        let text = paragraph.trim().replace('\n', " ");
        builder = builder.add_paragraph(&text);
    }

    let doc = builder.build().context("Failed to build document")?;

    config.verbose(&format!("Document ID: {}", doc.id()));

    // Write to file
    doc.save(&output)
        .with_context(|| format!("Failed to write document to: {}", output.display()))?;

    if config.json {
        let result = serde_json::json!({
            "status": "success",
            "file": output.display().to_string(),
            "document_id": doc.id().to_string(),
            "title": title,
            "state": state,
            "blocks": doc.content().len()
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        config.success(&format!("Created: {}", output.display()));
        config.field("Document ID", &doc.id().to_string());
        config.field("State", &doc_state.to_string());
    }

    Ok(())
}
