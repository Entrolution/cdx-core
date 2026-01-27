//! Metadata command implementations.

use anyhow::{Context, Result};
use cdx_core::Document;
use colored::Colorize;
use std::path::PathBuf;

use crate::output::OutputConfig;

/// Display document metadata.
pub fn run_get_metadata(file: PathBuf, config: &OutputConfig) -> Result<()> {
    config.verbose(&format!("Reading metadata from: {}", file.display()));

    let doc = Document::open(&file)
        .with_context(|| format!("Failed to open document: {}", file.display()))?;

    let dc = doc.dublin_core();
    let manifest = doc.manifest();

    if config.json {
        let metadata = serde_json::json!({
            "dublin_core": {
                "title": dc.title(),
                "creators": dc.creators(),
                "subjects": dc.subjects(),
                "description": dc.description(),
                "publisher": dc.publisher(),
                "contributors": dc.contributors(),
                "date": dc.date(),
                "type": dc.dc_type(),
                "format": dc.format(),
                "identifier": dc.identifier(),
                "source": dc.source(),
                "language": dc.language(),
                "relation": dc.relation(),
                "coverage": dc.coverage(),
                "rights": dc.rights(),
            },
            "manifest": {
                "id": doc.id().to_string(),
                "spec_version": manifest.codex,
                "state": doc.state().to_string(),
                "hash_algorithm": format!("{:?}", manifest.hash_algorithm),
                "created": manifest.created.to_rfc3339(),
                "modified": manifest.modified.to_rfc3339(),
            }
        });
        println!("{}", serde_json::to_string_pretty(&metadata)?);
        return Ok(());
    }

    println!("\n{}", "Document Metadata".blue().bold());
    println!("{}", "═".repeat(60).blue());

    // Dublin Core fields
    config.section("Dublin Core");
    config.field("Title", dc.title());

    let creators = dc.creators();
    if !creators.is_empty() {
        config.field("Creator(s)", &creators.join(", "));
    }

    let subjects = dc.subjects();
    if !subjects.is_empty() {
        config.field("Subject(s)", &subjects.join(", "));
    }

    if let Some(desc) = dc.description() {
        config.field("Description", desc);
    }

    if let Some(publisher) = dc.publisher() {
        config.field("Publisher", publisher);
    }

    let contributors = dc.contributors();
    if !contributors.is_empty() {
        config.field("Contributor(s)", &contributors.join(", "));
    }

    if let Some(date) = dc.date() {
        config.field("Date", date);
    }

    if let Some(dc_type) = dc.dc_type() {
        config.field("Type", dc_type);
    }

    if let Some(format) = dc.format() {
        config.field("Format", format);
    }

    if let Some(identifier) = dc.identifier() {
        config.field("Identifier", identifier);
    }

    if let Some(source) = dc.source() {
        config.field("Source", source);
    }

    if let Some(language) = dc.language() {
        config.field("Language", language);
    }

    if let Some(relation) = dc.relation() {
        config.field("Relation", relation);
    }

    if let Some(coverage) = dc.coverage() {
        config.field("Coverage", coverage);
    }

    if let Some(rights) = dc.rights() {
        config.field("Rights", rights);
    }

    // Manifest info
    config.section("Document Info");
    config.field("Document ID", &doc.id().to_string());
    config.field("Spec Version", &manifest.codex);
    config.field("State", &doc.state().to_string());
    config.field("Hash Algorithm", &format!("{:?}", manifest.hash_algorithm));
    config.field("Created", &manifest.created.to_rfc3339());
    config.field("Modified", &manifest.modified.to_rfc3339());

    println!();
    Ok(())
}

/// Set document metadata fields.
#[allow(clippy::too_many_arguments)]
pub fn run_set_metadata(
    file: PathBuf,
    title: Option<String>,
    creator: Vec<String>,
    subject: Vec<String>,
    description: Option<String>,
    publisher: Option<String>,
    language: Option<String>,
    rights: Option<String>,
    output: Option<PathBuf>,
    config: &OutputConfig,
) -> Result<()> {
    config.verbose(&format!("Updating metadata in: {}", file.display()));

    let mut doc = Document::open(&file)
        .with_context(|| format!("Failed to open document: {}", file.display()))?;

    // Check if anything to update
    let has_changes = title.is_some()
        || !creator.is_empty()
        || !subject.is_empty()
        || description.is_some()
        || publisher.is_some()
        || language.is_some()
        || rights.is_some();

    if !has_changes {
        config.info("No metadata changes specified. Use --help to see available options.");
        return run_get_metadata(file, config);
    }

    // Get mutable access to dublin core
    let dc = doc
        .dublin_core_mut()
        .with_context(|| "Cannot modify metadata in current document state")?;

    // Track what was changed
    let mut changes = Vec::new();

    if let Some(new_title) = title {
        dc.set_title(&new_title);
        changes.push(format!("title=\"{new_title}\""));
    }

    if !creator.is_empty() {
        dc.set_creators(creator.clone());
        changes.push(format!("creator={:?}", creator));
    }

    if !subject.is_empty() {
        dc.set_subjects(subject.clone());
        changes.push(format!("subject={:?}", subject));
    }

    if let Some(desc) = description {
        let display = if desc.is_empty() {
            "(cleared)".to_string()
        } else {
            desc.clone()
        };
        dc.set_description(if desc.is_empty() { None } else { Some(desc) });
        changes.push(format!("description=\"{display}\""));
    }

    if let Some(pub_val) = publisher {
        let display = if pub_val.is_empty() {
            "(cleared)".to_string()
        } else {
            pub_val.clone()
        };
        dc.set_publisher(if pub_val.is_empty() {
            None
        } else {
            Some(pub_val)
        });
        changes.push(format!("publisher=\"{display}\""));
    }

    if let Some(lang) = language {
        let display = if lang.is_empty() {
            "(cleared)".to_string()
        } else {
            lang.clone()
        };
        dc.set_language(if lang.is_empty() { None } else { Some(lang) });
        changes.push(format!("language=\"{display}\""));
    }

    if let Some(rights_val) = rights {
        let display = if rights_val.is_empty() {
            "(cleared)".to_string()
        } else {
            rights_val.clone()
        };
        dc.set_rights(if rights_val.is_empty() {
            None
        } else {
            Some(rights_val)
        });
        changes.push(format!("rights=\"{display}\""));
    }

    let output_path = output.unwrap_or(file);
    doc.save(&output_path)
        .with_context(|| format!("Failed to save document: {}", output_path.display()))?;

    config.success(&format!("Metadata updated: {}", changes.join(", ")));

    Ok(())
}
