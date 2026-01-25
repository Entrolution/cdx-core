//! Benchmarks for cdx-core document operations.
//!
//! Run with: cargo bench

#![feature(test)]
extern crate test;

use test::Bencher;
use cdx_core::{Document, HashAlgorithm, Hasher};

/// Benchmark creating a simple document.
#[bench]
fn bench_create_simple_document(b: &mut Bencher) {
    b.iter(|| {
        Document::builder()
            .title("Benchmark Document")
            .creator("Benchmark")
            .add_heading(1, "Title")
            .add_paragraph("Content paragraph.")
            .build()
            .unwrap()
    });
}

/// Benchmark creating a document with many blocks.
#[bench]
fn bench_create_large_document(b: &mut Bencher) {
    b.iter(|| {
        let mut builder = Document::builder()
            .title("Large Document")
            .creator("Benchmark");

        for i in 0..100 {
            builder = builder
                .add_heading(2, format!("Section {i}"))
                .add_paragraph(format!("This is paragraph {i} with some content."));
        }

        builder.build().unwrap()
    });
}

/// Benchmark computing document ID.
#[bench]
fn bench_compute_document_id(b: &mut Bencher) {
    let doc = Document::builder()
        .title("ID Benchmark")
        .creator("Benchmark")
        .add_heading(1, "Title")
        .add_paragraph("Content for ID computation.")
        .build()
        .unwrap();

    b.iter(|| {
        doc.compute_id().unwrap()
    });
}

/// Benchmark SHA-256 hashing of small data.
#[bench]
fn bench_hash_sha256_small(b: &mut Bencher) {
    let data = b"Small piece of data for hashing";
    b.iter(|| {
        Hasher::hash(HashAlgorithm::Sha256, data)
    });
}

/// Benchmark SHA-256 hashing of 1MB data.
#[bench]
fn bench_hash_sha256_1mb(b: &mut Bencher) {
    let data = vec![0u8; 1024 * 1024];
    b.iter(|| {
        Hasher::hash(HashAlgorithm::Sha256, &data)
    });
}

/// Benchmark BLAKE3 hashing of 1MB data.
#[bench]
fn bench_hash_blake3_1mb(b: &mut Bencher) {
    let data = vec![0u8; 1024 * 1024];
    b.iter(|| {
        Hasher::hash(HashAlgorithm::Blake3, &data)
    });
}

/// Benchmark document verification.
#[bench]
fn bench_verify_document(b: &mut Bencher) {
    let doc = Document::builder()
        .title("Verify Benchmark")
        .creator("Benchmark")
        .add_heading(1, "Title")
        .add_paragraph("Content to verify.")
        .build()
        .unwrap();

    b.iter(|| {
        doc.verify().unwrap()
    });
}

/// Benchmark save and reload cycle.
#[bench]
fn bench_save_reload_cycle(b: &mut Bencher) {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("bench.cdx");

    let doc = Document::builder()
        .title("Cycle Benchmark")
        .creator("Benchmark")
        .add_heading(1, "Title")
        .add_paragraph("Content for cycle test.")
        .build()
        .unwrap();

    b.iter(|| {
        doc.save(&file_path).unwrap();
        Document::open(&file_path).unwrap()
    });
}

#[cfg(feature = "signatures")]
mod signature_benches {
    use super::*;
    use cdx_core::security::{EcdsaSigner, EcdsaVerifier, SignerInfo, Signer, Verifier};

    /// Benchmark key generation.
    #[bench]
    fn bench_generate_keypair(b: &mut Bencher) {
        b.iter(|| {
            let signer_info = SignerInfo::new("Benchmark");
            EcdsaSigner::generate(signer_info).unwrap()
        });
    }

    /// Benchmark signing.
    #[bench]
    fn bench_sign_document(b: &mut Bencher) {
        let doc = Document::builder()
            .title("Sign Benchmark")
            .creator("Benchmark")
            .add_paragraph("Content to sign.")
            .build()
            .unwrap();

        let doc_id = doc.compute_id().unwrap();
        let signer_info = SignerInfo::new("Benchmark");
        let (signer, _) = EcdsaSigner::generate(signer_info).unwrap();

        b.iter(|| {
            signer.sign(&doc_id).unwrap()
        });
    }

    /// Benchmark verification.
    #[bench]
    fn bench_verify_signature(b: &mut Bencher) {
        let doc = Document::builder()
            .title("Verify Benchmark")
            .creator("Benchmark")
            .add_paragraph("Content to verify.")
            .build()
            .unwrap();

        let doc_id = doc.compute_id().unwrap();
        let signer_info = SignerInfo::new("Benchmark");
        let (signer, public_key) = EcdsaSigner::generate(signer_info).unwrap();
        let signature = signer.sign(&doc_id).unwrap();
        let verifier = EcdsaVerifier::from_pem(&public_key).unwrap();

        b.iter(|| {
            verifier.verify(&doc_id, &signature).unwrap()
        });
    }
}
