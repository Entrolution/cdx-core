# Codex Specification Conformance Matrix

This document maps requirements from the [Codex File Format Specification](../codex-file-format-spec/) to test coverage in cdx-core.

## Legend

- **Status**: `PASS` = test exists and passes, `TODO` = test needed, `N/A` = not applicable
- **Spec Section**: References to spec files in format `XX-name.md §Y.Z`

---

## 1. Container Format (01-container-format.md)

| Spec Section | Requirement | Test File | Test Name | Status |
|--------------|-------------|-----------|-----------|--------|
| §3.3 | `manifest.json` required at root | archive/mod.rs | existing validation | PASS |
| §3.3 | `content/document.json` required | archive/mod.rs | existing validation | PASS |
| §3.3 | `metadata/dublin-core.json` required | archive/mod.rs | existing validation | PASS |
| §4.2 | `manifest.json` must be first file in ZIP | tests/integration.rs | test_manifest_must_be_first_file | PASS |
| §5.2 | Archives up to 100MB supported | N/A | Implementation limit | N/A |

## 2. Manifest (02-manifest.md)

| Spec Section | Requirement | Test File | Test Name | Status |
|--------------|-------------|-----------|-----------|--------|
| §3.2 | `codex` version required | manifest.rs | test_manifest_creation | PASS |
| §3.2 | `id` required (format: `algorithm:hexdigest` or `pending`) | tests/conformance.rs | test_manifest_id_valid_hash_pattern | PASS |
| §3.2 | `state` required (draft/review/frozen/published) | manifest.rs | test_manifest_validation | PASS |
| §3.2 | `created` timestamp required (ISO 8601) | tests/conformance.rs | test_manifest_timestamps_iso8601 | PASS |
| §3.2 | `modified` timestamp required (ISO 8601) | tests/conformance.rs | test_manifest_timestamps_iso8601 | PASS |
| §3.2 | `content` reference required | manifest.rs | test_manifest_creation | PASS |
| §3.2 | `metadata.dublinCore` required | manifest.rs | test_manifest_creation | PASS |
| §4.2 | Draft ID can be `pending` | tests/conformance.rs | test_manifest_id_pending_for_draft | PASS |
| §4.10 | Extension `id` field required | tests/conformance.rs | test_extension_id_format | PASS |
| §4.10 | Extension `version` field required | tests/conformance.rs | test_extension_version_present | PASS |
| §4.10 | Extension `required` field determines rejection | tests/conformance.rs | test_required_extension_unsupported_detection | PASS |
| §5.3 | Frozen/published requires signatures | tests/conformance.rs | test_frozen_requires_signatures_in_manifest | PASS |
| §5.3 | Frozen/published requires lineage (if forked) | manifest.rs | test_frozen_requires_lineage | PASS |

## 3. Content Blocks (03-content-blocks.md)

| Spec Section | Requirement | Test File | Test Name | Status |
|--------------|-------------|-----------|-----------|--------|
| §2 | Block `type` field required | content/block.rs | existing validation | PASS |
| §2 | Block `id` optional but unique if present | content/block.rs | existing validation | PASS |
| §3+ | All block types serialize/deserialize correctly | tests/conformance.rs | test_block_type_round_trips | PASS |

## 4. Document Hashing (06-document-hashing.md)

| Spec Section | Requirement | Test File | Test Name | Status |
|--------------|-------------|-----------|-----------|--------|
| §3.1 | Hash format: `algorithm:hexdigest` | hash.rs | existing validation | PASS |
| §3.2 | SHA-256 required (default) | hash.rs | existing validation | PASS |
| §4.1 | Hash INCLUDES content blocks | tests/conformance.rs | test_hash_changes_with_content | PASS |
| §4.1 | Hash INCLUDES title metadata | tests/conformance.rs | test_hash_changes_with_title | PASS |
| §4.1 | Hash INCLUDES creator metadata | tests/conformance.rs | test_hash_changes_with_creator | PASS |
| §4.1 | Hash INCLUDES subject metadata | tests/conformance.rs | test_hash_changes_with_subject | PASS |
| §4.1 | Hash INCLUDES description metadata | tests/conformance.rs | test_hash_changes_with_description | PASS |
| §4.1 | Hash INCLUDES language metadata | tests/conformance.rs | test_hash_changes_with_language | PASS |
| §4.1 | Hash EXCLUDES presentation layers | tests/conformance.rs | test_hash_unchanged_by_presentation | PASS |
| §4.1 | Hash EXCLUDES security/signatures | tests/conformance.rs | test_hash_unchanged_by_signatures | PASS |
| §4.1 | Hash EXCLUDES phantom data | tests/conformance.rs | test_hash_unchanged_by_phantoms | PASS |
| §4.1 | Hash EXCLUDES form data | tests/conformance.rs | test_hash_unchanged_by_forms | PASS |
| §4.1 | Hash EXCLUDES collaboration data | tests/conformance.rs | test_hash_unchanged_by_comments | PASS |
| §4.3 | JCS canonicalization (RFC 8785) | document.rs | test_compute_id | PASS |
| §4.3 | Hash determinism | tests/conformance.rs | test_hash_determinism | PASS |
| §7.1 | Draft documents may have `pending` ID | tests/conformance.rs | test_draft_pending_id | PASS |

## 5. State Machine (07-state-machine.md)

| Spec Section | Requirement | Test File | Test Name | Status |
|--------------|-------------|-----------|-----------|--------|
| §3.2 | Draft: fully editable | state.rs | test_editability | PASS |
| §3.3 | Review: document ID computed | tests/conformance.rs | test_review_state_requires_computed_id | PASS |
| §3.4 | Frozen: requires signature | tests/conformance.rs | test_frozen_requires_signature | PASS |
| §3.4 | Frozen: content immutable | state.rs | test_immutability | PASS |
| §3.5 | Published: requires signature | tests/conformance.rs | test_published_requires_signature | PASS |
| §4.1 | Valid transitions: draft→review | state.rs | test_valid_transitions | PASS |
| §4.1 | Valid transitions: review→frozen | state.rs | test_valid_transitions | PASS |
| §4.1 | Valid transitions: review→draft (if unsigned) | tests/integration.rs | test_revert_to_draft | PASS |
| §4.1 | Valid transitions: frozen→published | state.rs | test_valid_transitions | PASS |
| §6.2 | Frozen requires at least one valid signature | manifest.rs | test_frozen_requires_precise_layout | PASS |

## 6. Asset Embedding (05-asset-embedding.md)

| Spec Section | Requirement | Test File | Test Name | Status |
|--------------|-------------|-----------|-----------|--------|
| §3.2 | Asset `id` required | asset/index.rs | existing validation | PASS |
| §3.2 | Asset `path` required | asset/index.rs | existing validation | PASS |
| §3.2 | Asset `hash` required | asset/index.rs | existing validation | PASS |
| §8.1 | Asset hash must match file content | tests/integration.rs | test_asset_index_hash_matches_file | PASS |
| §8.1 | Missing asset file = error | tests/integration.rs | test_asset_missing_file_error | PASS |
| §8.1 | Hash mismatch = error | tests/integration.rs | test_asset_hash_mismatch_error | PASS |
| §4.1 | Asset hashes included in document ID | tests/integration.rs | test_asset_hashes_included_in_document_id | PASS |

## 7. Provenance and Lineage (09-provenance-and-lineage.md)

| Spec Section | Requirement | Test File | Test Name | Status |
|--------------|-------------|-----------|-----------|--------|
| §3.1 | Parent hash format: `algorithm:hexdigest` | tests/conformance.rs | test_lineage_parent_hash_format | PASS |
| §3.2 | Ancestors ordered nearest-first | tests/conformance.rs | test_lineage_ancestors_ordered | PASS |
| §3.1 | Version >= 1 | tests/conformance.rs | test_lineage_version_positive | PASS |
| §3.1 | Depth reflects position in chain | tests/conformance.rs | test_lineage_depth_matches_ancestors | PASS |
| §4.1 | Merkle tree from block hashes | provenance/merkle.rs | test_merkle_tree_from_items | PASS |
| §4.4 | Merkle root matches block index | tests/conformance.rs | test_merkle_root_in_content_ref | PASS |
| §4.5 | Block index hashes match computed | tests/conformance.rs | test_block_index_hash_consistency | PASS |
| §5.1 | Proof path verifies block membership | provenance/proof.rs | test_proof_verification | PASS |
| §5.2 | Tampered block fails proof | provenance/proof.rs | test_proof_fails_wrong_block | PASS |
| §4.4 | Fork creates valid lineage | tests/conformance.rs | test_fork_creates_valid_lineage | PASS |

## 8. Metadata (08-metadata.md)

| Spec Section | Requirement | Test File | Test Name | Status |
|--------------|-------------|-----------|-----------|--------|
| §2.1 | Dublin Core `title` required | tests/conformance.rs | test_dublin_core_title_required | PASS |
| §2.1 | Dublin Core `creator` required | tests/conformance.rs | test_dublin_core_creator_required | PASS |
| §2 | Dublin Core serialization round-trip | tests/integration.rs | test_dublin_core_round_trip | PASS |

## 9. Security Extension

| Spec Section | Requirement | Test File | Test Name | Status |
|--------------|-------------|-----------|-----------|--------|
| §signatures | Signature `signer.name` required | tests/conformance.rs | test_signature_requires_signer_name | PASS |
| §signatures | Signature `documentId` matches manifest | tests/conformance.rs | test_signature_document_id_matches_manifest | PASS |
| §signatures | Signature persistence round-trip | tests/integration.rs | test_signature_persistence | PASS |
| §signatures | Multiple signatures supported | tests/integration.rs | test_multiple_signatures | PASS |

## 10. Extensions

| Spec Section | Requirement | Test File | Test Name | Status |
|--------------|-------------|-----------|-----------|--------|
| §extensions | Required extension detected in manifest | tests/conformance.rs | test_required_extension_unsupported_detection | PASS |
| §extensions | Optional extension unsupported = allow | tests/conformance.rs | test_optional_extension_unsupported_ok | PASS |
| §extensions | Undeclared extension produces warning | tests/conformance.rs | test_undeclared_extension_produces_warning | PASS |
| §extensions | Extension declaration serialization | tests/conformance.rs | test_extension_declaration_serialization | PASS |

---

## Property-Based Tests

| Category | Property | Test File | Test Name | Status |
|----------|----------|-----------|-----------|--------|
| Hash boundary | Metadata subset inclusion consistent | tests/integration.rs | proptest_hash_boundary_metadata_inclusion | PASS |
| Hash determinism | Same content = same hash | tests/integration.rs | proptest_hash_determinism_random_content | PASS |
| Serialization | Content round-trip preserves structure | tests/integration.rs | proptest_content_serialization_roundtrip | PASS |
| Block structure | Valid blocks serialize correctly | tests/integration.rs | proptest_block_structure_constraints | PASS |

---

## Summary

| Category | Total | Passing | TODO |
|----------|-------|---------|------|
| Container Format | 5 | 4 | 0 |
| Manifest | 13 | 13 | 0 |
| Content Blocks | 3 | 3 | 0 |
| Document Hashing | 16 | 16 | 0 |
| State Machine | 10 | 10 | 0 |
| Asset Embedding | 7 | 7 | 0 |
| Provenance/Lineage | 10 | 10 | 0 |
| Metadata | 3 | 3 | 0 |
| Security | 4 | 4 | 0 |
| Extensions | 4 | 4 | 0 |
| Property-Based | 4 | 4 | 0 |
| **Total** | **79** | **78** | **0** |

---

## Test Reference Format

Tests should include spec reference comments:

```rust
/// Per spec §06-document-hashing.md §4.1 - Hash includes title metadata
#[test]
fn test_hash_changes_with_title() {
    // ...
}
```
