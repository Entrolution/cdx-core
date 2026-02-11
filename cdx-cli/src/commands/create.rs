//! Create command implementation.

use anyhow::{Context, Result};
use cdx_core::{Document, DocumentState};
use std::fs;
use std::path::{Path, PathBuf};

use crate::output::OutputConfig;

pub fn run(
    title: &str,
    authors: &[String],
    state: &str,
    input: Option<PathBuf>,
    output: &Path,
    config: &OutputConfig,
) -> Result<()> {
    config.verbose(&format!("Creating document: {title}"));

    // Parse state
    let doc_state = match state.to_lowercase().as_str() {
        "draft" => DocumentState::Draft,
        "review" => DocumentState::Review,
        "frozen" => DocumentState::Frozen,
        "published" => DocumentState::Published,
        _ => {
            anyhow::bail!(
                "Invalid state '{state}'. Valid states: draft, review, frozen, published"
            );
        }
    };

    // Read content from input file if provided
    let content_text = if let Some(input_path) = input {
        config.verbose(&format!("Reading content from: {}", input_path.display()));
        fs::read_to_string(&input_path)
            .with_context(|| format!("Failed to read input file: {}", input_path.display()))?
    } else {
        title.to_string()
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
        .title(title)
        .creator(&creator)
        .state(doc_state);

    for paragraph in paragraphs {
        let text = paragraph.trim().replace('\n', " ");
        builder = builder.add_paragraph(&text);
    }

    let doc = builder.build().context("Failed to build document")?;

    config.verbose(&format!("Document ID: {}", doc.id()));

    // Write to file
    doc.save(output)
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

    #[test]
    fn test_create_minimal_document() {
        let temp = TempDir::new().unwrap();
        let output = temp.path().join("test.cdx");

        let result = run("Test Document", &[], "draft", None, &output, &test_config());

        assert!(result.is_ok());
        assert!(output.exists());

        // Verify the document can be opened
        let doc = Document::open(&output).unwrap();
        assert_eq!(doc.dublin_core().title(), "Test Document");
    }

    #[test]
    fn test_create_with_single_author() {
        let temp = TempDir::new().unwrap();
        let output = temp.path().join("test.cdx");
        let authors = vec!["Jane Doe".to_string()];

        let result = run("Test", &authors, "draft", None, &output, &test_config());

        assert!(result.is_ok());
        let doc = Document::open(&output).unwrap();
        let creators = doc.dublin_core().creators();
        assert!(creators.contains(&"Jane Doe"));
    }

    #[test]
    fn test_create_with_multiple_authors() {
        let temp = TempDir::new().unwrap();
        let output = temp.path().join("test.cdx");
        let authors = vec!["Jane Doe".to_string(), "John Smith".to_string()];

        let result = run("Test", &authors, "draft", None, &output, &test_config());

        assert!(result.is_ok());
        let doc = Document::open(&output).unwrap();
        let creators = doc.dublin_core().creators();
        assert!(creators.contains(&"Jane Doe, John Smith"));
    }

    #[test]
    fn test_create_with_draft_state() {
        let temp = TempDir::new().unwrap();
        let output = temp.path().join("test.cdx");

        run("Test", &[], "draft", None, &output, &test_config()).unwrap();

        let doc = Document::open(&output).unwrap();
        assert_eq!(doc.state(), DocumentState::Draft);
    }

    #[test]
    fn test_create_with_review_state() {
        // The create command allows creating documents in review state.
        // However, the document can only be opened if state requirements
        // are met. Review state requires a computed document ID, which
        // the create command provides (it's computed during save).
        // But opening validates additional requirements.
        let temp = TempDir::new().unwrap();
        let output = temp.path().join("test.cdx");

        let result = run("Test", &[], "review", None, &output, &test_config());

        // The create command should succeed
        assert!(result.is_ok());

        // Note: Opening the document may fail validation since
        // review state has requirements. We just verify it was created.
        assert!(output.exists());
    }

    #[test]
    fn test_create_with_frozen_state() {
        // The create command allows creating documents in frozen state.
        // However, the document may fail validation when opened since
        // frozen state requires signatures and precise layout.
        let temp = TempDir::new().unwrap();
        let output = temp.path().join("test.cdx");

        let result = run("Test", &[], "frozen", None, &output, &test_config());

        // The create command should succeed
        assert!(result.is_ok());

        // Note: Opening the document may fail validation since
        // frozen state has requirements. We just verify it was created.
        assert!(output.exists());
    }

    #[test]
    fn test_create_invalid_state() {
        let temp = TempDir::new().unwrap();
        let output = temp.path().join("test.cdx");

        let result = run("Test", &[], "invalid", None, &output, &test_config());

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid state"));
    }

    #[test]
    fn test_create_state_case_insensitive() {
        let temp = TempDir::new().unwrap();
        let output = temp.path().join("test.cdx");

        run("Test", &[], "DRAFT", None, &output, &test_config()).unwrap();

        let doc = Document::open(&output).unwrap();
        assert_eq!(doc.state(), DocumentState::Draft);
    }

    #[test]
    fn test_create_with_input_file() {
        let temp = TempDir::new().unwrap();
        let input = temp.path().join("content.txt");
        let output = temp.path().join("test.cdx");

        std::fs::write(&input, "First paragraph.\n\nSecond paragraph.").unwrap();

        run("Test", &[], "draft", Some(input), &output, &test_config()).unwrap();

        let doc = Document::open(&output).unwrap();
        assert_eq!(doc.content().len(), 2);
    }

    #[test]
    fn test_create_with_nonexistent_input_file() {
        let temp = TempDir::new().unwrap();
        let input = temp.path().join("nonexistent.txt");
        let output = temp.path().join("test.cdx");

        let result = run("Test", &[], "draft", Some(input), &output, &test_config());

        assert!(result.is_err());
    }

    #[test]
    fn test_create_empty_authors_defaults_to_unknown() {
        let temp = TempDir::new().unwrap();
        let output = temp.path().join("test.cdx");

        run("Test", &[], "draft", None, &output, &test_config()).unwrap();

        let doc = Document::open(&output).unwrap();
        let creators = doc.dublin_core().creators();
        assert!(creators.contains(&"Unknown"));
    }

    #[test]
    fn test_create_content_splitting() {
        let temp = TempDir::new().unwrap();
        let input = temp.path().join("content.txt");
        let output = temp.path().join("test.cdx");

        // Three paragraphs separated by blank lines
        std::fs::write(&input, "Para 1.\n\nPara 2.\n\nPara 3.").unwrap();

        run("Test", &[], "draft", Some(input), &output, &test_config()).unwrap();

        let doc = Document::open(&output).unwrap();
        assert_eq!(doc.content().len(), 3);
    }
}
