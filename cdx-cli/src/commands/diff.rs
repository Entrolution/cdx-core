//! Diff command implementation for comparing documents.

use anyhow::{Context, Result};
use cdx_core::Document;
use colored::Colorize;
use std::path::Path;

use crate::output::OutputConfig;

/// Compare two Codex documents.
pub fn run(file1: &Path, file2: &Path, config: &OutputConfig) -> Result<()> {
    config.verbose(&format!(
        "Comparing: {} vs {}",
        file1.display(),
        file2.display()
    ));

    let doc1 = Document::open(file1)
        .with_context(|| format!("Failed to open document: {}", file1.display()))?;
    let doc2 = Document::open(file2)
        .with_context(|| format!("Failed to open document: {}", file2.display()))?;

    let differences = collect_differences(&doc1, &doc2);
    let is_related = check_lineage_relation(&doc1, &doc2);

    if config.json {
        display_json_diff(file1, file2, &differences, is_related)
    } else {
        display_text_diff(file1, file2, &differences, is_related);
        Ok(())
    }
}

#[allow(clippy::too_many_lines)] // sequential field-by-field comparison — splitting would scatter related diff logic
fn collect_differences(doc1: &Document, doc2: &Document) -> Vec<DiffItem> {
    let mut differences = Vec::new();
    let manifest_a = doc1.manifest();
    let manifest_b = doc2.manifest();
    let metadata_a = doc1.dublin_core();
    let metadata_b = doc2.dublin_core();

    if doc1.id() != doc2.id() {
        differences.push(DiffItem {
            field: "Document ID".to_string(),
            value1: doc1.id().to_string(),
            value2: doc2.id().to_string(),
        });
    }

    if doc1.state() != doc2.state() {
        differences.push(DiffItem {
            field: "State".to_string(),
            value1: doc1.state().to_string(),
            value2: doc2.state().to_string(),
        });
    }

    if metadata_a.title() != metadata_b.title() {
        differences.push(DiffItem {
            field: "Title".to_string(),
            value1: metadata_a.title().to_string(),
            value2: metadata_b.title().to_string(),
        });
    }

    let creators_a = metadata_a.creators();
    let creators_b = metadata_b.creators();
    if creators_a != creators_b {
        differences.push(DiffItem {
            field: "Creators".to_string(),
            value1: creators_a.join(", "),
            value2: creators_b.join(", "),
        });
    }

    let content1 = doc1.content();
    let content2 = doc2.content();
    if content1.len() != content2.len() {
        differences.push(DiffItem {
            field: "Block Count".to_string(),
            value1: content1.len().to_string(),
            value2: content2.len().to_string(),
        });
    }

    if manifest_a.content.hash != manifest_b.content.hash {
        differences.push(DiffItem {
            field: "Content Hash".to_string(),
            value1: manifest_a.content.hash.to_string(),
            value2: manifest_b.content.hash.to_string(),
        });
    }

    let merkle1 = doc1.merkle_root().ok();
    let merkle2 = doc2.merkle_root().ok();
    if merkle1 != merkle2 {
        differences.push(DiffItem {
            field: "Merkle Root".to_string(),
            value1: merkle1.map_or_else(|| "(none)".to_string(), |r| r.to_string()),
            value2: merkle2.map_or_else(|| "(none)".to_string(), |r| r.to_string()),
        });
    }

    let sig_count1 = doc1.signatures().len();
    let sig_count2 = doc2.signatures().len();
    if sig_count1 != sig_count2 {
        differences.push(DiffItem {
            field: "Signature Count".to_string(),
            value1: sig_count1.to_string(),
            value2: sig_count2.to_string(),
        });
    }

    let has_lineage1 = manifest_a.lineage.is_some();
    let has_lineage2 = manifest_b.lineage.is_some();
    if has_lineage1 != has_lineage2 {
        differences.push(DiffItem {
            field: "Has Lineage".to_string(),
            value1: has_lineage1.to_string(),
            value2: has_lineage2.to_string(),
        });
    } else if let (Some(l1), Some(l2)) = (&manifest_a.lineage, &manifest_b.lineage) {
        if l1.parent != l2.parent {
            differences.push(DiffItem {
                field: "Lineage Parent".to_string(),
                value1: l1
                    .parent
                    .as_ref()
                    .map_or_else(|| "(root)".to_string(), ToString::to_string),
                value2: l2
                    .parent
                    .as_ref()
                    .map_or_else(|| "(root)".to_string(), ToString::to_string),
            });
        }
        if l1.version != l2.version {
            differences.push(DiffItem {
                field: "Lineage Version".to_string(),
                value1: l1
                    .version
                    .map_or_else(|| "(none)".to_string(), |v| v.to_string()),
                value2: l2
                    .version
                    .map_or_else(|| "(none)".to_string(), |v| v.to_string()),
            });
        }
    }

    differences
}

fn display_json_diff(
    file1: &Path,
    file2: &Path,
    differences: &[DiffItem],
    is_related: bool,
) -> Result<()> {
    let result = serde_json::json!({
        "file1": file1.display().to_string(),
        "file2": file2.display().to_string(),
        "identical": differences.is_empty(),
        "difference_count": differences.len(),
        "related_by_lineage": is_related,
        "differences": differences.iter().map(|d| serde_json::json!({
            "field": d.field,
            "file1_value": d.value1,
            "file2_value": d.value2,
        })).collect::<Vec<_>>(),
    });
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

fn display_text_diff(file1: &Path, file2: &Path, differences: &[DiffItem], is_related: bool) {
    println!("\n{}", "Document Comparison".blue().bold());
    println!("{}", "═".repeat(60).blue());

    println!("{}: {}", "File 1".bold(), file1.display());
    println!("{}: {}", "File 2".bold(), file2.display());

    if is_related {
        println!("\n{} Documents are related by lineage", "ℹ".cyan());
    }

    if differences.is_empty() {
        println!(
            "\n{} {}",
            "✓".green().bold(),
            "Documents are identical".green()
        );
    } else {
        println!(
            "\n{} {} difference{}:",
            "△".yellow(),
            differences.len(),
            if differences.len() == 1 { "" } else { "s" }
        );

        for diff in differences {
            println!("\n  {}:", diff.field.cyan());
            println!("    {} {}", "File 1:".dimmed(), diff.value1);
            println!("    {} {}", "File 2:".dimmed(), diff.value2);
        }
    }

    println!();
}

/// A single difference between documents.
struct DiffItem {
    field: String,
    value1: String,
    value2: String,
}

/// Check if two documents are related by lineage.
fn check_lineage_relation(doc1: &Document, doc2: &Document) -> bool {
    let id1 = doc1.id();
    let id2 = doc2.id();

    // Check if doc2 is a descendant of doc1
    if let Some(ref lineage) = doc2.manifest().lineage {
        if lineage.parent.as_ref() == Some(id1) {
            return true;
        }
        if lineage.ancestors.contains(id1) {
            return true;
        }
    }

    // Check if doc1 is a descendant of doc2
    if let Some(ref lineage) = doc1.manifest().lineage {
        if lineage.parent.as_ref() == Some(id2) {
            return true;
        }
        if lineage.ancestors.contains(id2) {
            return true;
        }
    }

    false
}
