//! CLI integration tests for the cdx command.
//!
//! These tests exercise the CLI end-to-end using the `assert_cmd` crate.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Get a Command for the cdx binary.
#[allow(deprecated)] // cargo_bin deprecated in 2.1.2 for custom build-dir; no replacement available yet
fn cdx() -> Command {
    Command::cargo_bin("cdx").unwrap()
}

// =============================================================================
// Create Command Tests
// =============================================================================

#[test]
fn test_create_minimal_document() {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("test.cdx");

    cdx()
        .arg("create")
        .args(["-t", "Test Document"])
        .arg(&output)
        .assert()
        .success();

    assert!(output.exists(), "Output file should be created");
}

#[test]
fn test_create_with_author() {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("test.cdx");

    cdx()
        .arg("create")
        .args(["-t", "Test Document"])
        .args(["-a", "Jane Doe"])
        .arg(&output)
        .assert()
        .success();

    assert!(output.exists());
}

#[test]
fn test_create_with_multiple_authors() {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("test.cdx");

    cdx()
        .arg("create")
        .args(["-t", "Test Document"])
        .args(["-a", "Jane Doe"])
        .args(["-a", "John Smith"])
        .arg(&output)
        .assert()
        .success();

    assert!(output.exists());
}

#[test]
fn test_create_with_state() {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("test.cdx");

    cdx()
        .arg("create")
        .args(["-t", "Test Document"])
        .args(["--state", "review"])
        .arg(&output)
        .assert()
        .success();

    assert!(output.exists());
}

#[test]
fn test_create_invalid_state() {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("test.cdx");

    cdx()
        .arg("create")
        .args(["-t", "Test Document"])
        .args(["--state", "invalid"])
        .arg(&output)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid state"));
}

#[test]
fn test_create_with_input_file() {
    let temp = TempDir::new().unwrap();
    let input = temp.path().join("content.txt");
    let output = temp.path().join("test.cdx");

    fs::write(
        &input,
        "This is the first paragraph.\n\nThis is the second paragraph.",
    )
    .unwrap();

    cdx()
        .arg("create")
        .args(["-t", "Test Document"])
        .args(["-i"])
        .arg(&input)
        .arg(&output)
        .assert()
        .success();

    assert!(output.exists());
}

#[test]
fn test_create_json_output() {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("test.cdx");

    cdx()
        .arg("--json")
        .arg("create")
        .args(["-t", "Test Document"])
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"status\": \"success\""))
        .stdout(predicate::str::contains("\"document_id\""));
}

// =============================================================================
// Validate Command Tests
// =============================================================================

#[test]
fn test_validate_valid_document() {
    let temp = TempDir::new().unwrap();
    let doc_path = temp.path().join("test.cdx");

    // First create a document
    cdx()
        .arg("create")
        .args(["-t", "Test Document"])
        .arg(&doc_path)
        .assert()
        .success();

    // Then validate it
    cdx()
        .arg("validate")
        .arg(&doc_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"));
}

#[test]
fn test_validate_json_output() {
    let temp = TempDir::new().unwrap();
    let doc_path = temp.path().join("test.cdx");

    // Create a document
    cdx()
        .arg("create")
        .args(["-t", "Test Document"])
        .arg(&doc_path)
        .assert()
        .success();

    // Validate with JSON output
    cdx()
        .arg("--json")
        .arg("validate")
        .arg(&doc_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"valid\": true"));
}

#[test]
fn test_validate_nonexistent_file() {
    cdx()
        .arg("validate")
        .arg("/nonexistent/path/to/file.cdx")
        .assert()
        .failure();
}

// =============================================================================
// Inspect Command Tests
// =============================================================================

#[test]
fn test_inspect_document() {
    let temp = TempDir::new().unwrap();
    let doc_path = temp.path().join("test.cdx");

    // Create a document
    cdx()
        .arg("create")
        .args(["-t", "My Test Document"])
        .args(["-a", "Test Author"])
        .arg(&doc_path)
        .assert()
        .success();

    // Inspect it
    cdx()
        .arg("inspect")
        .arg(&doc_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("My Test Document"))
        .stdout(predicate::str::contains("Document ID"));
}

#[test]
fn test_inspect_with_blocks() {
    let temp = TempDir::new().unwrap();
    let doc_path = temp.path().join("test.cdx");

    cdx()
        .arg("create")
        .args(["-t", "Test Document"])
        .arg(&doc_path)
        .assert()
        .success();

    cdx()
        .arg("inspect")
        .arg("--blocks")
        .arg(&doc_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Blocks"));
}

#[test]
fn test_inspect_json_output() {
    let temp = TempDir::new().unwrap();
    let doc_path = temp.path().join("test.cdx");

    cdx()
        .arg("create")
        .args(["-t", "Test Document"])
        .arg(&doc_path)
        .assert()
        .success();

    cdx()
        .arg("--json")
        .arg("inspect")
        .arg(&doc_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"document_id\""))
        .stdout(predicate::str::contains("\"spec_version\""));
}

// =============================================================================
// Status Command Tests
// =============================================================================

#[test]
fn test_status_command() {
    let temp = TempDir::new().unwrap();
    let doc_path = temp.path().join("test.cdx");

    cdx()
        .arg("create")
        .args(["-t", "Test Document"])
        .arg(&doc_path)
        .assert()
        .success();

    cdx()
        .arg("status")
        .arg(&doc_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("State"));
}

// =============================================================================
// State Transition Tests
// =============================================================================

#[test]
fn test_submit_review_transition() {
    let temp = TempDir::new().unwrap();
    let doc_path = temp.path().join("test.cdx");

    // Create a draft document
    cdx()
        .arg("create")
        .args(["-t", "Test Document"])
        .args(["--state", "draft"])
        .arg(&doc_path)
        .assert()
        .success();

    // Submit for review
    cdx().arg("submit-review").arg(&doc_path).assert().success();

    // Verify state changed
    cdx()
        .arg("--json")
        .arg("inspect")
        .arg(&doc_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"state\": \"review\""));
}

#[test]
fn test_freeze_requires_signatures() {
    // Freezing a document requires signatures first.
    // This test verifies the proper error message is shown.
    let temp = TempDir::new().unwrap();
    let doc_path = temp.path().join("test.cdx");

    // Create a draft document
    cdx()
        .arg("create")
        .args(["-t", "Test Document"])
        .arg(&doc_path)
        .assert()
        .success();

    // Submit for review
    cdx().arg("submit-review").arg(&doc_path).assert().success();

    // Try to freeze without signatures - should fail with helpful message
    cdx()
        .arg("freeze")
        .arg(&doc_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("signature"));
}

#[test]
fn test_revert_transition() {
    let temp = TempDir::new().unwrap();
    let doc_path = temp.path().join("test.cdx");

    // Create a draft document
    cdx()
        .arg("create")
        .args(["-t", "Test Document"])
        .arg(&doc_path)
        .assert()
        .success();

    // Submit for review
    cdx().arg("submit-review").arg(&doc_path).assert().success();

    // Revert to draft
    cdx().arg("revert").arg(&doc_path).assert().success();

    // Verify state changed back to draft
    cdx()
        .arg("--json")
        .arg("inspect")
        .arg(&doc_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"state\": \"draft\""));
}

// =============================================================================
// Extract Command Tests
// =============================================================================

#[test]
fn test_extract_content() {
    let temp = TempDir::new().unwrap();
    let doc_path = temp.path().join("test.cdx");

    cdx()
        .arg("create")
        .args(["-t", "Test Document"])
        .arg(&doc_path)
        .assert()
        .success();

    cdx()
        .arg("extract")
        .arg("--content")
        .arg("-o")
        .arg(temp.path())
        .arg(&doc_path)
        .assert()
        .success();
}

#[test]
fn test_extract_text() {
    let temp = TempDir::new().unwrap();
    let doc_path = temp.path().join("test.cdx");

    cdx()
        .arg("create")
        .args(["-t", "Test Document"])
        .arg(&doc_path)
        .assert()
        .success();

    cdx()
        .arg("extract")
        .arg("--text")
        .arg("-o")
        .arg(temp.path())
        .arg(&doc_path)
        .assert()
        .success();
}

// =============================================================================
// Diff Command Tests
// =============================================================================

#[test]
fn test_diff_identical_documents() {
    let temp = TempDir::new().unwrap();
    let doc1 = temp.path().join("doc1.cdx");
    let doc2 = temp.path().join("doc2.cdx");

    // Create two identical documents (same title)
    cdx()
        .arg("create")
        .args(["-t", "Test Document"])
        .arg(&doc1)
        .assert()
        .success();

    cdx()
        .arg("create")
        .args(["-t", "Test Document"])
        .arg(&doc2)
        .assert()
        .success();

    // Diff them
    cdx().arg("diff").arg(&doc1).arg(&doc2).assert().success();
}

#[test]
fn test_diff_different_documents() {
    let temp = TempDir::new().unwrap();
    let doc1 = temp.path().join("doc1.cdx");
    let doc2 = temp.path().join("doc2.cdx");

    cdx()
        .arg("create")
        .args(["-t", "Document One"])
        .arg(&doc1)
        .assert()
        .success();

    cdx()
        .arg("create")
        .args(["-t", "Document Two"])
        .arg(&doc2)
        .assert()
        .success();

    cdx().arg("diff").arg(&doc1).arg(&doc2).assert().success();
}

// =============================================================================
// Fork Command Tests
// =============================================================================

#[test]
fn test_fork_document() {
    let temp = TempDir::new().unwrap();
    let original = temp.path().join("original.cdx");
    let forked = temp.path().join("forked.cdx");

    cdx()
        .arg("create")
        .args(["-t", "Original Document"])
        .arg(&original)
        .assert()
        .success();

    cdx()
        .arg("fork")
        .arg(&original)
        .args(["-o"])
        .arg(&forked)
        .args(["-n", "Forked for testing"])
        .assert()
        .success();

    assert!(forked.exists());
}

#[test]
fn test_fork_with_lineage() {
    let temp = TempDir::new().unwrap();
    let original = temp.path().join("original.cdx");
    let forked = temp.path().join("forked.cdx");

    cdx()
        .arg("create")
        .args(["-t", "Original Document"])
        .arg(&original)
        .assert()
        .success();

    cdx()
        .arg("fork")
        .arg(&original)
        .args(["-o"])
        .arg(&forked)
        .assert()
        .success();

    // Inspect forked document for lineage
    cdx()
        .arg("inspect")
        .arg("--provenance")
        .arg(&forked)
        .assert()
        .success()
        .stdout(predicate::str::contains("Provenance"));
}

// =============================================================================
// Metadata Command Tests
// =============================================================================

#[test]
fn test_get_metadata() {
    let temp = TempDir::new().unwrap();
    let doc_path = temp.path().join("test.cdx");

    cdx()
        .arg("create")
        .args(["-t", "Test Document"])
        .args(["-a", "Test Author"])
        .arg(&doc_path)
        .assert()
        .success();

    cdx()
        .arg("get-metadata")
        .arg(&doc_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Test Document"))
        .stdout(predicate::str::contains("Test Author"));
}

#[test]
fn test_set_metadata() {
    let temp = TempDir::new().unwrap();
    let doc_path = temp.path().join("test.cdx");

    cdx()
        .arg("create")
        .args(["-t", "Original Title"])
        .arg(&doc_path)
        .assert()
        .success();

    cdx()
        .arg("set-metadata")
        .arg(&doc_path)
        .args(["--title", "New Title"])
        .assert()
        .success();

    // Verify the change
    cdx()
        .arg("get-metadata")
        .arg(&doc_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("New Title"));
}

// =============================================================================
// Global Flags Tests
// =============================================================================

#[test]
fn test_verbose_flag() {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("test.cdx");

    cdx()
        .arg("--verbose")
        .arg("create")
        .args(["-t", "Test Document"])
        .arg(&output)
        .assert()
        .success();
}

#[test]
fn test_quiet_flag() {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("test.cdx");

    cdx()
        .arg("--quiet")
        .arg("create")
        .args(["-t", "Test Document"])
        .arg(&output)
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_color_never() {
    let temp = TempDir::new().unwrap();
    let output = temp.path().join("test.cdx");

    cdx()
        .arg("--color=never")
        .arg("create")
        .args(["-t", "Test Document"])
        .arg(&output)
        .assert()
        .success();
}

// =============================================================================
// Help Tests
// =============================================================================

#[test]
fn test_help() {
    cdx()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("CDX Document Format CLI"));
}

#[test]
fn test_version() {
    cdx().arg("--version").assert().success();
}

#[test]
fn test_create_help() {
    cdx()
        .arg("create")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Create a new CDX document"));
}

// =============================================================================
// Completions Command Tests
// =============================================================================

#[test]
fn test_completions_bash() {
    cdx()
        .arg("completions")
        .arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"));
}

#[test]
fn test_completions_zsh() {
    cdx()
        .arg("completions")
        .arg("zsh")
        .assert()
        .success()
        .stdout(predicate::str::contains("compdef"));
}

// =============================================================================
// End-to-End Workflow Tests
// =============================================================================

#[test]
fn test_full_workflow_create_validate_inspect() {
    let temp = TempDir::new().unwrap();
    let doc_path = temp.path().join("workflow.cdx");

    // Step 1: Create document
    cdx()
        .arg("create")
        .args(["-t", "Workflow Test"])
        .args(["-a", "Tester"])
        .arg(&doc_path)
        .assert()
        .success();

    // Step 2: Validate document
    cdx()
        .arg("validate")
        .arg(&doc_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"));

    // Step 3: Inspect document
    cdx()
        .arg("inspect")
        .arg(&doc_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Workflow Test"));

    // Step 4: Get metadata
    cdx()
        .arg("get-metadata")
        .arg(&doc_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Tester"));
}

#[test]
fn test_full_lifecycle_draft_to_review() {
    // Note: Full lifecycle to frozen/published requires signatures,
    // which would need key generation. This test covers draft -> review.
    let temp = TempDir::new().unwrap();
    let doc_path = temp.path().join("lifecycle.cdx");

    // Create as draft
    cdx()
        .arg("create")
        .args(["-t", "Lifecycle Test"])
        .args(["--state", "draft"])
        .arg(&doc_path)
        .assert()
        .success();

    // Verify draft state
    cdx()
        .arg("--json")
        .arg("inspect")
        .arg(&doc_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"state\": \"draft\""));

    // Submit for review
    cdx().arg("submit-review").arg(&doc_path).assert().success();

    // Verify review state
    cdx()
        .arg("--json")
        .arg("inspect")
        .arg(&doc_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"state\": \"review\""));

    // Revert back to draft
    cdx().arg("revert").arg(&doc_path).assert().success();

    // Verify back to draft
    cdx()
        .arg("--json")
        .arg("inspect")
        .arg(&doc_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"state\": \"draft\""));
}
