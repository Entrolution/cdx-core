//! Example: Sign a Codex document with ECDSA (ES256).
//!
//! This example demonstrates how to sign a document using
//! the security module's ECDSA signer.
//!
//! Note: This example requires the `signatures` feature.
//!
//! Usage: cargo run --example sign_document --features signatures

use cdx_core::{Document, Result};

#[cfg(feature = "signatures")]
use cdx_core::security::{EcdsaSigner, Signer, SignerInfo};

fn main() -> Result<()> {
    // Create a sample document
    let document = Document::builder()
        .title("Document to Sign")
        .creator("Signing Example")
        .add_heading(1, "Important Content")
        .add_paragraph("This document will be digitally signed.")
        .build()?;

    println!("Created document");
    println!("State: {:?}", document.state());

    // Compute the document ID (required for signing)
    let doc_id = document.compute_id()?;
    println!("Document ID: {doc_id}");

    #[cfg(feature = "signatures")]
    {
        // Create signer info
        let signer_info = SignerInfo::new("Example Signer")
            .with_email("signer@example.com")
            .with_organization("Example Corp");

        // Generate a new signing key (in production, you'd load an existing key)
        let (signer, public_key_pem) = EcdsaSigner::generate(signer_info)?;
        println!("\nGenerated new signing key");
        println!("Public key:\n{public_key_pem}");

        // Sign the document ID
        let signature = signer.sign(&doc_id)?;
        println!("\nDocument signed!");
        println!("Signature ID: {}", signature.id);
        println!("Algorithm: {}", signature.algorithm);
        println!("Signed at: {}", signature.signed_at);
        println!("Signer: {}", signature.signer.name);
    }

    #[cfg(not(feature = "signatures"))]
    {
        println!("\nNote: Enable the 'signatures' feature to see signing in action:");
        println!("  cargo run --example sign_document --features signatures");
    }

    Ok(())
}
