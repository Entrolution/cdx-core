//! Benchmarks for cdx-core document operations.
//!
//! Run with: cargo bench

use cdx_core::{Document, HashAlgorithm, Hasher};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

/// Benchmark creating a simple document.
fn bench_create_simple_document(c: &mut Criterion) {
    c.bench_function("create_simple_document", |b| {
        b.iter(|| {
            Document::builder()
                .title("Benchmark Document")
                .creator("Benchmark")
                .add_heading(1, "Title")
                .add_paragraph("Content paragraph.")
                .build()
                .unwrap()
        })
    });
}

/// Benchmark creating a document with many blocks.
fn bench_create_large_document(c: &mut Criterion) {
    c.bench_function("create_large_document", |b| {
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
        })
    });
}

/// Benchmark computing document ID.
fn bench_compute_document_id(c: &mut Criterion) {
    let doc = Document::builder()
        .title("ID Benchmark")
        .creator("Benchmark")
        .add_heading(1, "Title")
        .add_paragraph("Content for ID computation.")
        .build()
        .unwrap();

    c.bench_function("compute_document_id", |b| {
        b.iter(|| black_box(&doc).compute_id().unwrap())
    });
}

/// Benchmark SHA-256 hashing of small data.
fn bench_hash_sha256_small(c: &mut Criterion) {
    let data = b"Small piece of data for hashing";
    c.bench_function("hash_sha256_small", |b| {
        b.iter(|| Hasher::hash(HashAlgorithm::Sha256, black_box(data)))
    });
}

/// Benchmark SHA-256 hashing of 1MB data.
fn bench_hash_sha256_1mb(c: &mut Criterion) {
    let data = vec![0u8; 1024 * 1024];
    c.bench_function("hash_sha256_1mb", |b| {
        b.iter(|| Hasher::hash(HashAlgorithm::Sha256, black_box(&data)))
    });
}

/// Benchmark BLAKE3 hashing of 1MB data.
fn bench_hash_blake3_1mb(c: &mut Criterion) {
    let data = vec![0u8; 1024 * 1024];
    c.bench_function("hash_blake3_1mb", |b| {
        b.iter(|| Hasher::hash(HashAlgorithm::Blake3, black_box(&data)))
    });
}

/// Benchmark document verification.
fn bench_verify_document(c: &mut Criterion) {
    let doc = Document::builder()
        .title("Verify Benchmark")
        .creator("Benchmark")
        .add_heading(1, "Title")
        .add_paragraph("Content to verify.")
        .build()
        .unwrap();

    c.bench_function("verify_document", |b| {
        b.iter(|| black_box(&doc).verify().unwrap())
    });
}

/// Benchmark save and reload cycle.
fn bench_save_reload_cycle(c: &mut Criterion) {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("bench.cdx");

    let doc = Document::builder()
        .title("Cycle Benchmark")
        .creator("Benchmark")
        .add_heading(1, "Title")
        .add_paragraph("Content for cycle test.")
        .build()
        .unwrap();

    c.bench_function("save_reload_cycle", |b| {
        b.iter(|| {
            doc.save(black_box(&file_path)).unwrap();
            Document::open(black_box(&file_path)).unwrap()
        })
    });
}

criterion_group!(
    benches,
    bench_create_simple_document,
    bench_create_large_document,
    bench_compute_document_id,
    bench_hash_sha256_small,
    bench_hash_sha256_1mb,
    bench_hash_blake3_1mb,
    bench_verify_document,
    bench_save_reload_cycle,
);

#[cfg(feature = "signatures")]
mod signature_benches {
    use super::*;
    use cdx_core::security::{EcdsaSigner, EcdsaVerifier, Signer, SignerInfo, Verifier};

    /// Benchmark key generation.
    pub fn bench_generate_keypair(c: &mut Criterion) {
        c.bench_function("generate_keypair", |b| {
            b.iter(|| {
                let signer_info = SignerInfo::new("Benchmark");
                EcdsaSigner::generate(signer_info).unwrap()
            })
        });
    }

    /// Benchmark signing.
    pub fn bench_sign_document(c: &mut Criterion) {
        let doc = Document::builder()
            .title("Sign Benchmark")
            .creator("Benchmark")
            .add_paragraph("Content to sign.")
            .build()
            .unwrap();

        let doc_id = doc.compute_id().unwrap();
        let signer_info = SignerInfo::new("Benchmark");
        let (signer, _) = EcdsaSigner::generate(signer_info).unwrap();

        c.bench_function("sign_document", |b| {
            b.iter(|| signer.sign(black_box(&doc_id)).unwrap())
        });
    }

    /// Benchmark verification.
    pub fn bench_verify_signature(c: &mut Criterion) {
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

        c.bench_function("verify_signature", |b| {
            b.iter(|| {
                verifier
                    .verify(black_box(&doc_id), black_box(&signature))
                    .unwrap()
            })
        });
    }

    criterion_group!(
        signature_benches,
        bench_generate_keypair,
        bench_sign_document,
        bench_verify_signature,
    );
}

#[cfg(feature = "signatures")]
criterion_main!(benches, signature_benches::signature_benches);

#[cfg(not(feature = "signatures"))]
criterion_main!(benches);
