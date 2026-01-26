//! Fork command implementation.

use anyhow::{Context, Result};
use cdx_core::Document;
use std::path::PathBuf;

use crate::output::OutputConfig;

pub fn run(
    file: PathBuf,
    output: PathBuf,
    note: Option<String>,
    config: &OutputConfig,
) -> Result<()> {
    config.verbose(&format!("Forking document: {}", file.display()));

    // Open the document
    let doc = Document::open(&file)
        .with_context(|| format!("Failed to open document: {}", file.display()))?;

    let parent_id = if doc.id().is_pending() {
        doc.compute_id()
            .with_context(|| "Failed to compute document ID")?
            .to_string()
    } else {
        doc.id().to_string()
    };

    config.verbose(&format!("Parent document ID: {}", parent_id));

    // Fork the document
    let mut forked = doc.fork().with_context(|| "Failed to fork document")?;

    // Set the note if provided
    if let Some(ref note_text) = note {
        if let Some(ref mut lineage) = forked.manifest_mut().lineage {
            lineage.note = Some(note_text.clone());
        }
    }

    let new_version = forked
        .manifest()
        .lineage
        .as_ref()
        .and_then(|l| l.version)
        .unwrap_or(1);

    // Save the forked document
    forked
        .save(&output)
        .with_context(|| format!("Failed to save forked document to: {}", output.display()))?;

    if config.json {
        let result = serde_json::json!({
            "status": "success",
            "file": output.display().to_string(),
            "parent_id": parent_id,
            "version": new_version,
            "state": "draft",
            "note": note
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        config.success("Document forked successfully");
        config.field("Output", &output.display().to_string());
        config.field("Parent", &parent_id);
        config.field("Version", &new_version.to_string());
        config.field("State", "draft");
        if let Some(note_text) = note {
            config.field("Note", &note_text);
        }
    }

    Ok(())
}
