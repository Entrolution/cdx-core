//! Publish command implementation.

use anyhow::{Context, Result};
use cdx_core::Document;
use std::path::{Path, PathBuf};

use crate::output::OutputConfig;

pub fn run(file: &Path, output: Option<PathBuf>, config: &OutputConfig) -> Result<()> {
    config.verbose(&format!("Publishing document: {}", file.display()));

    // Open the document
    let mut doc = Document::open(file)
        .with_context(|| format!("Failed to open document: {}", file.display()))?;

    let current_state = doc.state();
    config.verbose(&format!("Current state: {current_state}"));

    // Publish the document
    doc.publish()
        .with_context(|| "Failed to publish document")?;

    let doc_id = doc.id().to_string();

    // Determine output path
    let output_path = output.unwrap_or_else(|| file.to_path_buf());

    // Save the document
    doc.save(&output_path).with_context(|| {
        format!(
            "Failed to save published document to: {}",
            output_path.display()
        )
    })?;

    if config.json {
        let result = serde_json::json!({
            "status": "success",
            "file": output_path.display().to_string(),
            "previous_state": current_state.to_string(),
            "new_state": "published",
            "document_id": doc_id
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        config.success("Document published successfully");
        config.field("Output", &output_path.display().to_string());
        config.field("Document ID", &doc_id);
        config.field("State", "published");
    }

    Ok(())
}
