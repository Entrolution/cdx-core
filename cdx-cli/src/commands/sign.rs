//! Sign command implementation.

use anyhow::{Context, Result};
use cdx_core::security::{EcdsaSigner, Signer, SignerInfo};
use cdx_core::Document;
use std::fs;
use std::path::PathBuf;

use crate::output::OutputConfig;

pub fn run(
    file: PathBuf,
    key_path: PathBuf,
    name: String,
    email: Option<String>,
    algorithm: String,
    output: Option<PathBuf>,
    config: &OutputConfig,
) -> Result<()> {
    config.verbose(&format!("Signing: {}", file.display()));

    // Open the document
    let mut doc = Document::open(&file)
        .with_context(|| format!("Failed to open document: {}", file.display()))?;

    // Compute the document ID for signing
    let doc_id = doc.compute_id().context("Failed to compute document ID")?;

    if doc_id.is_pending() {
        anyhow::bail!("Cannot sign a document with pending ID. Document must be finalized first.");
    }

    // Read the private key
    let key_pem = fs::read_to_string(&key_path)
        .with_context(|| format!("Failed to read private key: {}", key_path.display()))?;

    // Build signer info
    let mut signer_info = SignerInfo::new(&name);
    if let Some(email_addr) = email {
        signer_info = signer_info.with_email(email_addr);
    }

    // Create signer based on algorithm
    let signature = match algorithm.to_uppercase().as_str() {
        "ES256" | "ECDSA" => {
            config.verbose("Using ES256 (ECDSA P-256) signature algorithm");
            let signer = EcdsaSigner::from_pem(&key_pem, signer_info)
                .context("Failed to load ECDSA private key")?;
            signer.sign(&doc_id).context("Failed to sign document")?
        }
        "EDDSA" | "ED25519" => {
            config.verbose("Using EdDSA (Ed25519) signature algorithm");
            let signer = cdx_core::security::EddsaSigner::from_pem(&key_pem, signer_info)
                .context("Failed to load EdDSA private key")?;
            signer.sign(&doc_id).context("Failed to sign document")?
        }
        other => {
            anyhow::bail!("Unsupported algorithm '{}'. Supported: ES256, EdDSA", other);
        }
    };

    let signature_id = signature.id.clone();
    config.verbose(&format!("Signature ID: {}", signature_id));

    // Add signature to document
    doc.add_signature(signature)
        .context("Failed to add signature to document")?;

    // Determine output path
    let output_path = output.unwrap_or_else(|| file.clone());

    // Save the signed document
    doc.save(&output_path).with_context(|| {
        format!(
            "Failed to save signed document to: {}",
            output_path.display()
        )
    })?;

    if config.json {
        let result = serde_json::json!({
            "status": "success",
            "file": output_path.display().to_string(),
            "signature_id": signature_id,
            "algorithm": algorithm,
            "signer": name,
            "document_id": doc_id.to_string()
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        config.success("Document signed successfully");
        config.field("Output", &output_path.display().to_string());
        config.field("Signature ID", &signature_id);
        config.field("Algorithm", &algorithm);
        config.field("Signer", &name);
        config.field("Document ID", &doc_id.to_string());
    }

    Ok(())
}
