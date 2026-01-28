//! Pack command implementation.

use anyhow::{Context, Result};
use cdx_core::content::Content;
use cdx_core::metadata::DublinCore;
use cdx_core::{Document, DocumentState};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

use crate::output::OutputConfig;

pub fn run(
    input: PathBuf,
    output: PathBuf,
    from_json: bool,
    config: &OutputConfig,
) -> Result<()> {
    if from_json {
        pack_from_json(input, output, config)
    } else {
        pack_from_directory(input, output, config)
    }
}

fn pack_from_json(input: PathBuf, output: PathBuf, config: &OutputConfig) -> Result<()> {
    config.verbose(&format!("Packing from JSON: {}", input.display()));

    let json_str = fs::read_to_string(&input)
        .with_context(|| format!("Failed to read input file: {}", input.display()))?;
    let combined: Value = serde_json::from_str(&json_str)
        .with_context(|| format!("Failed to parse JSON from: {}", input.display()))?;

    // Normalize and parse the content section
    let mut content_val = combined
        .get("content")
        .cloned()
        .context("Missing 'content' section in JSON")?;
    normalize_content(&mut content_val);

    let content: Content =
        serde_json::from_value(content_val).context("Failed to parse content section")?;

    // Parse the Dublin Core section
    let dublin_core: DublinCore = serde_json::from_value(
        combined
            .get("dublin_core")
            .cloned()
            .context("Missing 'dublin_core' section in JSON")?,
    )
    .context("Failed to parse dublin_core section")?;

    let block_count = content.len();

    let doc = Document::builder()
        .state(DocumentState::Draft)
        .with_content(content)
        .with_dublin_core(dublin_core)
        .build()
        .context("Failed to build document")?;

    doc.save(&output)
        .with_context(|| format!("Failed to write document to: {}", output.display()))?;

    if config.json {
        let result = serde_json::json!({
            "status": "success",
            "file": output.display().to_string(),
            "document_id": doc.id().to_string(),
            "blocks": block_count,
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        config.success(&format!("Packed: {}", output.display()));
        config.field("Document ID", &doc.id().to_string());
        config.field("Blocks", &block_count.to_string());
    }

    Ok(())
}

fn pack_from_directory(input: PathBuf, output: PathBuf, config: &OutputConfig) -> Result<()> {
    config.verbose(&format!("Packing from directory: {}", input.display()));

    let content_path = input.join("content/document.json");
    let dc_path = input.join("metadata/dublin-core.json");

    // Read and normalize content JSON
    let content_str = fs::read_to_string(&content_path)
        .with_context(|| format!("Failed to open: {}", content_path.display()))?;
    let mut content_val: Value = serde_json::from_str(&content_str)
        .with_context(|| format!("Failed to parse: {}", content_path.display()))?;
    normalize_content(&mut content_val);

    let content: Content = serde_json::from_value(content_val)
        .with_context(|| format!("Failed to deserialize: {}", content_path.display()))?;

    let dublin_core: DublinCore = serde_json::from_reader(
        fs::File::open(&dc_path)
            .with_context(|| format!("Failed to open: {}", dc_path.display()))?,
    )
    .with_context(|| format!("Failed to parse: {}", dc_path.display()))?;

    let block_count = content.len();

    let doc = Document::builder()
        .state(DocumentState::Draft)
        .with_content(content)
        .with_dublin_core(dublin_core)
        .build()
        .context("Failed to build document")?;

    doc.save(&output)
        .with_context(|| format!("Failed to write document to: {}", output.display()))?;

    if config.json {
        let result = serde_json::json!({
            "status": "success",
            "file": output.display().to_string(),
            "document_id": doc.id().to_string(),
            "blocks": block_count,
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        config.success(&format!("Packed: {}", output.display()));
        config.field("Document ID", &doc.id().to_string());
        config.field("Blocks", &block_count.to_string());
    }

    Ok(())
}

/// Normalize Pandoc writer JSON to match cdx-core's expected format.
///
/// The Pandoc writer uses string marks (e.g., `"bold"`) while cdx-core
/// expects internally tagged objects (e.g., `{"type": "bold"}`). This
/// function walks the JSON tree and converts marks in-place.
fn normalize_content(value: &mut Value) {
    match value {
        Value::Object(map) => {
            // Normalize marks array: "bold" → {"type": "bold"}
            if let Some(marks) = map.get_mut("marks") {
                normalize_marks(marks);
            }
            // Recurse into all object values
            for val in map.values_mut() {
                normalize_content(val);
            }
        }
        Value::Array(arr) => {
            for item in arr {
                normalize_content(item);
            }
        }
        _ => {}
    }
}

/// Convert string marks to tagged objects.
fn normalize_marks(marks: &mut Value) {
    if let Value::Array(arr) = marks {
        for mark in arr.iter_mut() {
            if let Value::String(s) = mark {
                *mark = serde_json::json!({"type": s.clone()});
            }
            // Object marks (e.g., link) are already in the right format
        }
    }
}
