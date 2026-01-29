//! Pack command implementation.

use anyhow::{Context, Result};
use cdx_core::content::Content;
use cdx_core::metadata::DublinCore;
use cdx_core::DocumentId;
use cdx_core::{Document, DocumentState};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

use crate::output::OutputConfig;

pub fn run(input: PathBuf, output: PathBuf, from_json: bool, config: &OutputConfig) -> Result<()> {
    if from_json {
        pack_from_json(input, output, config)
    } else {
        pack_from_directory(input, output, config)
    }
}

/// Output the result of a pack operation in either JSON or human-readable format.
fn output_pack_result(
    config: &OutputConfig,
    output: &Path,
    doc_id: &DocumentId,
    block_count: usize,
) -> Result<()> {
    if config.json {
        let result = serde_json::json!({
            "status": "success",
            "file": output.display().to_string(),
            "document_id": doc_id.to_string(),
            "blocks": block_count,
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        config.success(&format!("Packed: {}", output.display()));
        config.field("Document ID", &doc_id.to_string());
        config.field("Blocks", &block_count.to_string());
    }
    Ok(())
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

    output_pack_result(config, &output, doc.id(), block_count)
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

    output_pack_result(config, &output, doc.id(), block_count)
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

#[cfg(test)]
mod tests {
    use super::*;
    use cdx_core::Document;
    use tempfile::TempDir;

    fn test_config() -> OutputConfig {
        OutputConfig {
            verbose: false,
            quiet: true,
            json: false,
        }
    }

    fn create_pack_directory(temp: &TempDir) -> PathBuf {
        let input_dir = temp.path().join("input");
        fs::create_dir_all(input_dir.join("content")).unwrap();
        fs::create_dir_all(input_dir.join("metadata")).unwrap();

        // Write content JSON - using correct cdx-core format:
        // - Block type is tagged with "type" field
        // - Paragraph uses "children" with Text nodes
        // - Text nodes use "value" field for text content
        let content_json = r#"{
            "version": "0.1",
            "blocks": [
                {
                    "type": "paragraph",
                    "children": [
                        {"value": "Hello, world!"}
                    ]
                }
            ]
        }"#;
        fs::write(input_dir.join("content/document.json"), content_json).unwrap();

        // Write Dublin Core metadata
        // - title is a String
        // - creator is StringOrArray (can be string or array of strings)
        // - version is required
        let dc_json = r#"{
            "version": "1.1",
            "terms": {
                "title": "Pack Test",
                "creator": "Test Author"
            }
        }"#;
        fs::write(input_dir.join("metadata/dublin-core.json"), dc_json).unwrap();

        input_dir
    }

    fn create_combined_json(temp: &TempDir) -> PathBuf {
        let json_path = temp.path().join("combined.json");
        let json = r#"{
            "content": {
                "version": "0.1",
                "blocks": [
                    {
                        "type": "paragraph",
                        "children": [
                            {"value": "From JSON"}
                        ]
                    }
                ]
            },
            "dublin_core": {
                "version": "1.1",
                "terms": {
                    "title": "JSON Pack Test",
                    "creator": "JSON Author"
                }
            }
        }"#;
        fs::write(&json_path, json).unwrap();
        json_path
    }

    #[test]
    fn test_pack_from_directory() {
        let temp = TempDir::new().unwrap();
        let input_dir = create_pack_directory(&temp);
        let output = temp.path().join("output.cdx");

        let result = run(input_dir, output.clone(), false, &test_config());
        assert!(result.is_ok());
        assert!(output.exists());

        // Verify the document can be opened
        let doc = Document::open(&output).unwrap();
        assert_eq!(doc.dublin_core().title(), "Pack Test");
    }

    #[test]
    fn test_pack_from_json() {
        let temp = TempDir::new().unwrap();
        let json_path = create_combined_json(&temp);
        let output = temp.path().join("output.cdx");

        let result = run(json_path, output.clone(), true, &test_config());
        assert!(result.is_ok());
        assert!(output.exists());

        // Verify the document can be opened
        let doc = Document::open(&output).unwrap();
        assert_eq!(doc.dublin_core().title(), "JSON Pack Test");
    }

    #[test]
    fn test_pack_creates_draft_document() {
        let temp = TempDir::new().unwrap();
        let input_dir = create_pack_directory(&temp);
        let output = temp.path().join("output.cdx");

        run(input_dir, output.clone(), false, &test_config()).unwrap();

        let doc = Document::open(&output).unwrap();
        assert_eq!(doc.state(), DocumentState::Draft);
    }

    #[test]
    fn test_pack_nonexistent_directory() {
        let temp = TempDir::new().unwrap();
        let output = temp.path().join("output.cdx");

        let result = run(
            temp.path().join("nonexistent"),
            output,
            false,
            &test_config(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_pack_nonexistent_json() {
        let temp = TempDir::new().unwrap();
        let output = temp.path().join("output.cdx");

        let result = run(
            temp.path().join("nonexistent.json"),
            output,
            true,
            &test_config(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_pack_invalid_json() {
        let temp = TempDir::new().unwrap();
        let json_path = temp.path().join("invalid.json");
        fs::write(&json_path, "not valid json").unwrap();
        let output = temp.path().join("output.cdx");

        let result = run(json_path, output, true, &test_config());
        assert!(result.is_err());
    }

    #[test]
    fn test_pack_json_missing_content() {
        let temp = TempDir::new().unwrap();
        let json_path = temp.path().join("missing_content.json");
        let json = r#"{
            "dublin_core": {
                "terms": {"title": ["Test"], "creator": ["Test"]}
            }
        }"#;
        fs::write(&json_path, json).unwrap();
        let output = temp.path().join("output.cdx");

        let result = run(json_path, output, true, &test_config());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("content"));
    }

    #[test]
    fn test_pack_json_missing_dublin_core() {
        let temp = TempDir::new().unwrap();
        let json_path = temp.path().join("missing_dc.json");
        let json = r#"{
            "content": {
                "version": "1.0",
                "blocks": []
            }
        }"#;
        fs::write(&json_path, json).unwrap();
        let output = temp.path().join("output.cdx");

        let result = run(json_path, output, true, &test_config());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("dublin_core"));
    }

    #[test]
    fn test_normalize_content_simple_marks() {
        let mut value = serde_json::json!({
            "type": "text",
            "text": "hello",
            "marks": ["bold", "italic"]
        });

        normalize_content(&mut value);

        let marks = value.get("marks").unwrap().as_array().unwrap();
        assert_eq!(marks[0], serde_json::json!({"type": "bold"}));
        assert_eq!(marks[1], serde_json::json!({"type": "italic"}));
    }

    #[test]
    fn test_normalize_content_object_marks_unchanged() {
        let mut value = serde_json::json!({
            "type": "text",
            "text": "link",
            "marks": [{"type": "link", "attrs": {"href": "http://example.com"}}]
        });

        let expected = value.clone();
        normalize_content(&mut value);

        assert_eq!(value, expected);
    }

    #[test]
    fn test_normalize_content_nested() {
        let mut value = serde_json::json!({
            "type": "paragraph",
            "content": [
                {
                    "type": "text",
                    "text": "bold text",
                    "marks": ["bold"]
                }
            ]
        });

        normalize_content(&mut value);

        let content = value.get("content").unwrap().as_array().unwrap();
        let text_node = &content[0];
        let marks = text_node.get("marks").unwrap().as_array().unwrap();
        assert_eq!(marks[0], serde_json::json!({"type": "bold"}));
    }
}
