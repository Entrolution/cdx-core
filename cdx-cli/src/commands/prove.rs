//! Proof generation and verification command implementations.

use anyhow::{Context, Result};
use cdx_core::provenance::BlockProof;
use cdx_core::Document;
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::output::OutputConfig;

/// Generate a Merkle proof for a specific block.
pub fn run_prove(
    file: &Path,
    block_id: Option<String>,
    block_index: Option<usize>,
    output: Option<PathBuf>,
    config: &OutputConfig,
) -> Result<()> {
    config.verbose(&format!("Generating proof for: {}", file.display()));

    let doc = Document::open(file)
        .with_context(|| format!("Failed to open document: {}", file.display()))?;

    // Determine which block to prove
    let proof = if let Some(id) = block_id {
        config.verbose(&format!("Proving block by ID: {id}"));
        doc.prove_block_by_id(&id)
            .with_context(|| format!("Failed to generate proof for block '{id}'"))?
    } else if let Some(idx) = block_index {
        config.verbose(&format!("Proving block by index: {idx}"));
        doc.prove_block(idx)
            .with_context(|| format!("Failed to generate proof for block at index {idx}"))?
    } else {
        anyhow::bail!("Either --block-id or --block-index must be specified");
    };

    // Serialize the proof
    let proof_json = serde_json::to_string_pretty(&proof)?;

    if let Some(output_path) = output {
        fs::write(&output_path, &proof_json)
            .with_context(|| format!("Failed to write proof to: {}", output_path.display()))?;
        config.success(&format!("Proof written to: {}", output_path.display()));
    } else if config.json {
        println!("{proof_json}");
    } else {
        // Pretty print proof details
        println!("\n{}", "Block Proof".blue().bold());
        println!("{}", "═".repeat(60).blue());
        config.field("Block Index", &proof.index.to_string());
        config.field("Algorithm", &format!("{:?}", proof.algorithm));
        config.field("Root Hash", &proof.root_hash.to_string());
        config.field("Proof Path Length", &proof.path.len().to_string());
        println!("\n{}", "Proof JSON:".dimmed());
        println!("{proof_json}");
    }

    Ok(())
}

/// Verify a Merkle proof against a document.
pub fn run_verify_proof(file: &Path, proof_file: &Path, config: &OutputConfig) -> Result<()> {
    config.verbose(&format!(
        "Verifying proof {} against {}",
        proof_file.display(),
        file.display()
    ));

    let doc = Document::open(file)
        .with_context(|| format!("Failed to open document: {}", file.display()))?;

    let proof_json = fs::read_to_string(proof_file)
        .with_context(|| format!("Failed to read proof file: {}", proof_file.display()))?;

    let proof: BlockProof =
        serde_json::from_str(&proof_json).with_context(|| "Failed to parse proof JSON")?;

    // Get the block index to compute its hash
    let block_index = doc.block_index()?;
    let block_entry = block_index
        .get_block(proof.index)
        .ok_or_else(|| anyhow::anyhow!("Block index {} not found in document", proof.index))?;

    // Verify the proof
    let is_valid = doc.verify_proof(&proof, &block_entry.hash);

    if config.json {
        let result = serde_json::json!({
            "valid": is_valid,
            "block_index": proof.index,
            "block_id": block_entry.id,
            "root_hash": proof.root_hash.to_string(),
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else if is_valid {
        config.success(&format!(
            "Proof is valid for block {} (index {})",
            block_entry.id, proof.index
        ));
        config.field("Root Hash", &proof.root_hash.to_string());
    } else {
        println!("{} {}", "✗".red().bold(), "Proof verification failed".red());

        // Try to provide more details
        let doc_merkle_root = doc.merkle_root()?;
        if proof.root_hash != doc_merkle_root {
            config.warning(&format!(
                "Root hash mismatch: proof has {}, document has {}",
                proof.root_hash, doc_merkle_root
            ));
        }

        std::process::exit(1);
    }

    Ok(())
}

/// Show document lineage (ancestor chain).
pub fn run_show_lineage(file: &Path, config: &OutputConfig) -> Result<()> {
    config.verbose(&format!("Showing lineage for: {}", file.display()));

    let doc = Document::open(file)
        .with_context(|| format!("Failed to open document: {}", file.display()))?;

    let manifest = doc.manifest();

    if config.json {
        let lineage_json = if let Some(ref lineage) = manifest.lineage {
            serde_json::json!({
                "document_id": doc.id().to_string(),
                "parent": lineage.parent.as_ref().map(|p| p.to_string()),
                "ancestors": lineage.ancestors.iter().map(|a| a.to_string()).collect::<Vec<_>>(),
                "version": lineage.version,
                "depth": lineage.depth,
                "branch": lineage.branch,
                "merged_from": lineage.merged_from.iter().map(|m| m.to_string()).collect::<Vec<_>>(),
                "note": lineage.note,
            })
        } else {
            serde_json::json!({
                "document_id": doc.id().to_string(),
                "lineage": null
            })
        };
        println!("{}", serde_json::to_string_pretty(&lineage_json)?);
        return Ok(());
    }

    println!("\n{}", "Document Lineage".blue().bold());
    println!("{}", "═".repeat(60).blue());

    config.field("Document ID", &doc.id().to_string());
    config.field("State", &doc.state().to_string());

    if let Some(ref lineage) = manifest.lineage {
        println!();

        if let Some(ref parent) = lineage.parent {
            config.field("Parent", &parent.to_string());
        } else {
            config.field("Parent", "(root document)");
        }

        if let Some(version) = lineage.version {
            config.field("Version", &version.to_string());
        }

        if let Some(depth) = lineage.depth {
            config.field("Depth", &depth.to_string());
        }

        if let Some(ref branch) = lineage.branch {
            config.field("Branch", branch);
        }

        if let Some(ref note) = lineage.note {
            config.field("Note", note);
        }

        // Show ancestor chain
        if !lineage.ancestors.is_empty() {
            println!("\n{}", "Ancestor Chain:".dimmed());
            for (i, ancestor) in lineage.ancestors.iter().enumerate() {
                println!("  {}. {}", i + 1, ancestor);
            }
        }

        // Show merged documents
        if !lineage.merged_from.is_empty() {
            println!("\n{}", "Merged From:".dimmed());
            for merged in &lineage.merged_from {
                println!("  - {merged}");
            }
        }
    } else {
        println!("\n{}", "No lineage information".dimmed());
    }

    println!();
    Ok(())
}
