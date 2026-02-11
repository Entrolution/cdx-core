//! Inspect command implementation.

use anyhow::{Context, Result};
use cdx_core::Document;
use colored::Colorize;
use std::path::Path;

use crate::output::OutputConfig;

pub fn run(
    file: &Path,
    show_blocks: bool,
    show_signatures: bool,
    show_provenance: bool,
    config: &OutputConfig,
) -> Result<()> {
    config.verbose(&format!("Inspecting: {}", file.display()));

    // Open the document
    let doc = Document::open(file)
        .with_context(|| format!("Failed to open document: {}", file.display()))?;

    if config.json {
        display_json(&doc, file, show_signatures, show_provenance)
    } else {
        display_text(&doc, file, show_blocks, show_signatures, show_provenance, config);
        Ok(())
    }
}

fn display_json(
    doc: &Document,
    file: &Path,
    show_signatures: bool,
    show_provenance: bool,
) -> Result<()> {
    let manifest = doc.manifest();
    let content = doc.content();
    let dublin_core = doc.dublin_core();

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

    if show_signatures {
        if let Some(security) = &manifest.security {
            result["security"] = serde_json::json!({
                "has_signatures": security.signatures.is_some(),
                "is_encrypted": security.encryption.is_some(),
            });
        }
    }

    if show_provenance {
        if let Some(lineage) = &manifest.lineage {
            result["lineage"] = serde_json::json!({
                "parent": lineage.parent.as_ref().map(std::string::ToString::to_string),
                "version": lineage.version,
                "branch": lineage.branch,
                "note": lineage.note,
            });
        }
    }

    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

fn display_text(
    doc: &Document,
    file: &Path,
    show_blocks: bool,
    show_signatures: bool,
    show_provenance: bool,
    config: &OutputConfig,
) {
    let manifest = doc.manifest();
    let content = doc.content();
    let dublin_core = doc.dublin_core();

    println!("\n{}", "Codex Document".blue().bold());
    println!("{}", "═".repeat(60).blue());

    config.field("File", &file.display().to_string());
    config.field("Document ID", &doc.id().to_string());
    config.field("Spec Version", &manifest.codex);
    config.field("State", &doc.state().to_string());

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
        "definitionList" => "Definition List",
        "definitionItem" => "Definition Item",
        "definitionTerm" => "Definition Term",
        "definitionDescription" => "Definition Description",
        "measurement" => "Measurement",
        "signature" => "Signature",
        "svg" => "SVG",
        "barcode" => "Barcode",
        "figure" => "Figure",
        "figCaption" => "Figure Caption",
        other => other,
    }
    .to_string()
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

    fn create_test_document(path: &Path, title: &str) {
        let doc = Document::builder()
            .title(title)
            .creator("Test Author")
            .add_paragraph("Test content")
            .build()
            .unwrap();
        doc.save(path).unwrap();
    }

    #[test]
    fn test_inspect_basic() {
        let temp = TempDir::new().unwrap();
        let doc_path = temp.path().join("test.cdx");

        create_test_document(&doc_path, "Inspect Test");

        let result = run(&doc_path, false, false, false, &test_config());
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_with_blocks_flag() {
        let temp = TempDir::new().unwrap();
        let doc_path = temp.path().join("test.cdx");

        create_test_document(&doc_path, "Blocks Test");

        let result = run(&doc_path, true, false, false, &test_config());
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_with_signatures_flag() {
        let temp = TempDir::new().unwrap();
        let doc_path = temp.path().join("test.cdx");

        create_test_document(&doc_path, "Signatures Test");

        let result = run(&doc_path, false, true, false, &test_config());
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_with_provenance_flag() {
        let temp = TempDir::new().unwrap();
        let doc_path = temp.path().join("test.cdx");

        create_test_document(&doc_path, "Provenance Test");

        let result = run(&doc_path, false, false, true, &test_config());
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_all_flags() {
        let temp = TempDir::new().unwrap();
        let doc_path = temp.path().join("test.cdx");

        create_test_document(&doc_path, "All Flags Test");

        let result = run(&doc_path, true, true, true, &test_config());
        assert!(result.is_ok());
    }

    #[test]
    fn test_inspect_nonexistent_file() {
        let result = run(
            Path::new("/nonexistent/file.cdx"),
            false,
            false,
            false,
            &test_config(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_inspect_multiple_blocks() {
        let temp = TempDir::new().unwrap();
        let doc_path = temp.path().join("test.cdx");

        let doc = Document::builder()
            .title("Multi Block")
            .creator("Test")
            .add_heading(1, "Introduction")
            .add_paragraph("First paragraph")
            .add_paragraph("Second paragraph")
            .build()
            .unwrap();
        doc.save(&doc_path).unwrap();

        let result = run(&doc_path, true, false, false, &test_config());
        assert!(result.is_ok());

        // Verify block count
        let opened = Document::open(&doc_path).unwrap();
        assert_eq!(opened.content().len(), 3);
    }

    #[test]
    fn test_format_block_type_paragraph() {
        assert_eq!(format_block_type("paragraph"), "Paragraph");
    }

    #[test]
    fn test_format_block_type_heading() {
        assert_eq!(format_block_type("heading"), "Heading");
    }

    #[test]
    fn test_format_block_type_list() {
        assert_eq!(format_block_type("list"), "List");
    }

    #[test]
    fn test_format_block_type_code_block() {
        assert_eq!(format_block_type("codeBlock"), "Code Block");
    }

    #[test]
    fn test_format_block_type_unknown() {
        assert_eq!(format_block_type("unknownType"), "unknownType");
    }
}
