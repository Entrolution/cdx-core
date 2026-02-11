//! Freeze command implementation.

use anyhow::{Context, Result};
use cdx_core::Document;
use std::path::{Path, PathBuf};

use crate::output::OutputConfig;

pub fn run(file: &Path, output: Option<PathBuf>, config: &OutputConfig) -> Result<()> {
    config.verbose(&format!("Freezing document: {}", file.display()));

    // Open the document
    let mut doc = Document::open(file)
        .with_context(|| format!("Failed to open document: {}", file.display()))?;

    let current_state = doc.state();
    config.verbose(&format!("Current state: {}", current_state));

    // Check requirements before attempting freeze
    if !doc.has_signatures() {
        anyhow::bail!(
            "Cannot freeze: document has no signatures. Use 'cdx sign' to add a signature first."
        );
    }

    if doc.manifest().lineage.is_none() {
        anyhow::bail!("Cannot freeze: document has no lineage. Set lineage before freezing.");
    }

    if !doc.manifest().has_precise_layout() {
        anyhow::bail!(
            "Cannot freeze: document has no precise layout. Add a precise layout before freezing."
        );
    }

    // Freeze the document
    doc.freeze().with_context(|| "Failed to freeze document")?;

    let doc_id = doc.id().to_string();

    // Determine output path
    let output_path = output.unwrap_or_else(|| file.to_path_buf());

    // Save the document
    doc.save(&output_path).with_context(|| {
        format!(
            "Failed to save frozen document to: {}",
            output_path.display()
        )
    })?;

    if config.json {
        let result = serde_json::json!({
            "status": "success",
            "file": output_path.display().to_string(),
            "previous_state": current_state.to_string(),
            "new_state": "frozen",
            "document_id": doc_id
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        config.success("Document frozen successfully");
        config.field("Output", &output_path.display().to_string());
        config.field("Document ID", &doc_id);
        config.field("State", "frozen");
    }

    Ok(())
}
