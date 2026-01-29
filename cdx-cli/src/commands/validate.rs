//! Validate command implementation.

use anyhow::{Context, Result};
use cdx_core::Document;
use colored::Colorize;
use std::path::PathBuf;

use crate::output::OutputConfig;

pub fn run(file: PathBuf, config: &OutputConfig) -> Result<()> {
    config.verbose(&format!("Validating: {}", file.display()));

    // Open the document
    let doc = Document::open(&file)
        .with_context(|| format!("Failed to open document: {}", file.display()))?;

    // Get verification report
    let report = doc.verify().context("Verification failed")?;

    // Get manifest for additional checks
    let manifest = doc.manifest();
    let state = doc.state();
    let has_precise_layout = manifest.has_precise_layout();
    let precise_layouts = manifest.precise_layouts();

    if config.json {
        let result = serde_json::json!({
            "file": file.display().to_string(),
            "valid": report.is_valid(),
            "document_id": doc.id().to_string(),
            "document_id_verified": report.id_valid,
            "content_verified": report.content_valid,
            "state": state.to_string(),
            "has_precise_layout": has_precise_layout,
            "precise_layout_count": precise_layouts.len(),
            "presentation_valid": !state.requires_precise_layout() || has_precise_layout,
            "errors": report.errors
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    config.field("File", &file.display().to_string());
    config.field("Document ID", &doc.id().to_string());
    config.field("State", &state.to_string());

    // Report results
    let mut has_errors = false;

    // Check document ID
    if report.id_valid {
        config.info(&format!("{} Document ID verified", "✓".green()));
    } else {
        config.info(&format!("{} Document ID verification failed", "✗".red()));
        has_errors = true;
    }

    // Check content hashes
    if report.content_valid {
        config.info(&format!("{} Content verified", "✓".green()));
    } else {
        config.info(&format!("{} Content verification failed", "✗".red()));
        has_errors = true;
    }

    // Check precise layout requirements
    if state.requires_precise_layout() {
        if has_precise_layout {
            config.info(&format!(
                "{} Precise layout present ({} format{})",
                "✓".green(),
                precise_layouts.len(),
                if precise_layouts.len() == 1 { "" } else { "s" }
            ));
        } else {
            config.info(&format!(
                "{} Missing precise layout (required for {} state)",
                "✗".red(),
                state
            ));
            has_errors = true;
        }
    } else if has_precise_layout {
        config.info(&format!(
            "{} Precise layout present (optional for {} state)",
            "ℹ".blue(),
            state
        ));
    }

    // Print any errors
    if !report.errors.is_empty() {
        config.section("Errors");
        for error in &report.errors {
            config.info(&format!("{} {}", "•".red(), error));
        }
    }

    if has_errors {
        anyhow::bail!("Document validation failed");
    }

    config.success("Document is valid");
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

    fn create_test_document(path: &PathBuf, title: &str) {
        let doc = Document::builder()
            .title(title)
            .creator("Test")
            .add_paragraph("Test content")
            .build()
            .unwrap();
        doc.save(path).unwrap();
    }

    #[test]
    fn test_validate_valid_document() {
        let temp = TempDir::new().unwrap();
        let doc_path = temp.path().join("test.cdx");

        create_test_document(&doc_path, "Valid Document");

        let result = run(doc_path, &test_config());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_nonexistent_file() {
        let result = run(PathBuf::from("/nonexistent/file.cdx"), &test_config());
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_draft_document() {
        let temp = TempDir::new().unwrap();
        let doc_path = temp.path().join("test.cdx");

        let doc = Document::builder()
            .title("Draft Doc")
            .creator("Test")
            .state(cdx_core::DocumentState::Draft)
            .add_paragraph("Content")
            .build()
            .unwrap();
        doc.save(&doc_path).unwrap();

        let result = run(doc_path, &test_config());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_with_multiple_blocks() {
        let temp = TempDir::new().unwrap();
        let doc_path = temp.path().join("test.cdx");

        let doc = Document::builder()
            .title("Multi Block Doc")
            .creator("Test")
            .state(cdx_core::DocumentState::Draft)
            .add_heading(1, "Introduction")
            .add_paragraph("First paragraph")
            .add_paragraph("Second paragraph")
            .build()
            .unwrap();
        doc.save(&doc_path).unwrap();

        let result = run(doc_path, &test_config());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_document_integrity() {
        let temp = TempDir::new().unwrap();
        let doc_path = temp.path().join("test.cdx");

        create_test_document(&doc_path, "Integrity Test");

        // Validate and verify the document passes integrity checks
        let result = run(doc_path.clone(), &test_config());
        assert!(result.is_ok());

        // Also verify by opening and checking the report directly
        let doc = Document::open(&doc_path).unwrap();
        let report = doc.verify().unwrap();
        assert!(report.is_valid());
        assert!(report.id_valid);
        assert!(report.content_valid);
    }

    #[test]
    fn test_validate_with_verbose_config() {
        let temp = TempDir::new().unwrap();
        let doc_path = temp.path().join("test.cdx");

        create_test_document(&doc_path, "Verbose Test");

        let config = OutputConfig {
            verbose: true,
            quiet: false,
            json: false,
        };

        let result = run(doc_path, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_preserves_state() {
        let temp = TempDir::new().unwrap();
        let doc_path = temp.path().join("test.cdx");

        let doc = Document::builder()
            .title("State Test")
            .creator("Test")
            .state(cdx_core::DocumentState::Draft)
            .add_paragraph("Content")
            .build()
            .unwrap();
        doc.save(&doc_path).unwrap();

        // Validate to ensure state doesn't cause validation failure
        let result = run(doc_path.clone(), &test_config());
        assert!(result.is_ok());

        // Verify state was preserved
        let opened = Document::open(&doc_path).unwrap();
        assert_eq!(opened.state(), cdx_core::DocumentState::Draft);
    }
}
