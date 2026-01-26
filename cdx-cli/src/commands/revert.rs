//! Revert command implementation.

use anyhow::{Context, Result};
use cdx_core::Document;
use std::path::PathBuf;

use crate::output::OutputConfig;

pub fn run(file: PathBuf, output: Option<PathBuf>, config: &OutputConfig) -> Result<()> {
    config.verbose(&format!("Reverting document to draft: {}", file.display()));

    // Open the document
    let mut doc = Document::open(&file)
        .with_context(|| format!("Failed to open document: {}", file.display()))?;

    let current_state = doc.state();
    config.verbose(&format!("Current state: {}", current_state));

    // Check for signatures
    if doc.has_signatures() {
        anyhow::bail!(
            "Cannot revert: document has signatures. \
            Signed documents cannot be reverted to draft state."
        );
    }

    // Revert to draft
    doc.revert_to_draft()
        .with_context(|| "Failed to revert document to draft")?;

    // Determine output path
    let output_path = output.unwrap_or_else(|| file.clone());

    // Save the document
    doc.save(&output_path).with_context(|| {
        format!(
            "Failed to save reverted document to: {}",
            output_path.display()
        )
    })?;

    if config.json {
        let result = serde_json::json!({
            "status": "success",
            "file": output_path.display().to_string(),
            "previous_state": current_state.to_string(),
            "new_state": "draft"
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        config.success("Document reverted to draft");
        config.field("Output", &output_path.display().to_string());
        config.field("State", "draft");
    }

    Ok(())
}
