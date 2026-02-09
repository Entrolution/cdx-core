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
| §4.2 | `manifest.json` must be first file in ZIP | tests/integration.rs | test_manifest_must_be_first_file | TODO |
| §5.2 | Archives up to 100MB supported | N/A | Implementation limit | N/A |

## 2. Manifest (02-manifest.md)

| Spec Section | Requirement | Test File | Test Name | Status |
|--------------|-------------|-----------|-----------|--------|
| §3.2 | `codex` version required | manifest.rs | test_manifest_creation | PASS |
| §3.2 | `id` required (format: `algorithm:hexdigest` or `pending`) | tests/integration.rs | test_manifest_id_valid_hash_pattern | TODO |
| §3.2 | `state` required (draft/review/frozen/published) | manifest.rs | test_manifest_validation | PASS |
| §3.2 | `created` timestamp required (ISO 8601) | tests/integration.rs | test_manifest_timestamps_iso8601 | TODO |
| §3.2 | `modified` timestamp required (ISO 8601) | tests/integration.rs | test_manifest_timestamps_iso8601 | TODO |
| §3.2 | `content` reference required | manifest.rs | test_manifest_creation | PASS |
| §3.2 | `metadata.dublinCore` required | manifest.rs | test_manifest_creation | PASS |
| §4.2 | Draft ID can be `pending` | tests/integration.rs | test_manifest_id_pending_allowed_for_draft | TODO |
| §4.10 | Extension `id` field required | extensions/mod.rs | test_extension_id_format_valid | TODO |
| §4.10 | Extension `version` field required | extensions/mod.rs | test_extension_version_present | TODO |
| §4.10 | Extension `required` field determines rejection | extensions/mod.rs | test_required_extension_unsupported_error | TODO |
| §5.3 | Frozen/published requires signatures | manifest.rs | test_frozen_requires_signature | TODO |
| §5.3 | Frozen/published requires lineage (if forked) | manifest.rs | test_frozen_requires_lineage | PASS |

## 3. Content Blocks (03-content-blocks.md)

| Spec Section | Requirement | Test File | Test Name | Status |
|--------------|-------------|-----------|-----------|--------|
| §2 | Block `type` field required | content/block.rs | existing validation | PASS |
| §2 | Block `id` optional but unique if present | content/block.rs | existing validation | PASS |
| §3+ | All block types serialize/deserialize correctly | tests/integration.rs | test_complex_content_round_trip | PASS |

## 4. Document Hashing (06-document-hashing.md)

| Spec Section | Requirement | Test File | Test Name | Status |
|--------------|-------------|-----------|-----------|--------|
| §3.1 | Hash format: `algorithm:hexdigest` | hash.rs | existing validation | PASS |
| §3.2 | SHA-256 required (default) | hash.rs | existing validation | PASS |
| §4.1 | Hash INCLUDES content blocks | tests/integration.rs | test_hash_changes_with_content | PASS |
| §4.1 | Hash INCLUDES title metadata | tests/integration.rs | test_hash_changes_with_title | TODO |
| §4.1 | Hash INCLUDES creator metadata | tests/integration.rs | test_hash_changes_with_creator | TODO |
| §4.1 | Hash INCLUDES subject metadata | tests/integration.rs | test_hash_changes_with_subject | TODO |
| §4.1 | Hash INCLUDES description metadata | tests/integration.rs | test_hash_changes_with_description | TODO |
| §4.1 | Hash INCLUDES language metadata | tests/integration.rs | test_hash_changes_with_language | TODO |
| §4.1 | Hash EXCLUDES presentation layers | tests/integration.rs | test_hash_unchanged_by_presentation | TODO |
| §4.1 | Hash EXCLUDES security/signatures | tests/integration.rs | test_hash_unchanged_by_signatures | TODO |
| §4.1 | Hash EXCLUDES phantom data | tests/integration.rs | test_hash_unchanged_by_phantoms | TODO |
| §4.1 | Hash EXCLUDES form data | tests/integration.rs | test_hash_unchanged_by_forms | TODO |
| §4.1 | Hash EXCLUDES collaboration data | tests/integration.rs | test_hash_unchanged_by_comments | TODO |
| §4.3 | JCS canonicalization (RFC 8785) | document.rs | test_compute_id | PASS |
| §7.1 | Draft documents may have `pending` ID | tests/integration.rs | test_draft_pending_id | PASS |

## 5. State Machine (07-state-machine.md)

| Spec Section | Requirement | Test File | Test Name | Status |
|--------------|-------------|-----------|-----------|--------|
| §3.2 | Draft: fully editable | state.rs | test_editability | PASS |
| §3.3 | Review: document ID computed | tests/integration.rs | test_review_state_requires_computed_id | TODO |
| §3.4 | Frozen: requires signature | tests/integration.rs | test_frozen_requires_signature | TODO |
| §3.4 | Frozen: content immutable | state.rs | test_immutability | PASS |
| §3.5 | Published: requires signature | tests/integration.rs | test_published_requires_signature | TODO |
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
| §8.1 | Asset hash must match file content | tests/integration.rs | test_asset_index_hash_matches_file | TODO |
| §8.1 | Missing asset file = error | tests/integration.rs | test_asset_missing_file_error | TODO |
| §8.1 | Hash mismatch = error | tests/integration.rs | test_asset_hash_mismatch_error | TODO |
| §4.1 | Asset hashes included in document ID | tests/integration.rs | test_asset_hashes_included_in_document_id | TODO |

## 7. Provenance and Lineage (09-provenance-and-lineage.md)

| Spec Section | Requirement | Test File | Test Name | Status |
|--------------|-------------|-----------|-----------|--------|
| §3.1 | Parent hash format: `algorithm:hexdigest` | tests/integration.rs | test_lineage_parent_hash_format | TODO |
| §3.2 | Ancestors ordered nearest-first | tests/integration.rs | test_lineage_ancestors_ordered | TODO |
| §3.1 | Version >= 1 | tests/integration.rs | test_lineage_version_positive | TODO |
| §3.1 | Depth equals ancestors.len() + 1 for non-root | tests/integration.rs | test_lineage_depth_matches_ancestors | TODO |
| §4.1 | Merkle tree from block hashes | provenance/merkle.rs | test_merkle_tree_from_items | PASS |
| §4.4 | Merkle root in manifest | tests/integration.rs | test_merkle_root_matches_block_hashes | TODO |
| §4.5 | Block index hashes match computed | tests/integration.rs | test_block_index_hash_consistency | TODO |
| §5.1 | Proof path verifies block membership | provenance/proof.rs | test_proof_verification | PASS |
| §5.2 | Tampered block fails proof | provenance/proof.rs | test_proof_fails_wrong_block | PASS |
| §4.4 | Fork creates valid lineage | tests/integration.rs | test_fork_creates_lineage | PASS |

## 8. Metadata (08-metadata.md)

| Spec Section | Requirement | Test File | Test Name | Status |
|--------------|-------------|-----------|-----------|--------|
| §2.1 | Dublin Core `title` required | tests/integration.rs | test_dublin_core_title_required | TODO |
| §2.1 | Dublin Core `creator` required | tests/integration.rs | test_dublin_core_creator_required | TODO |
| §2 | Dublin Core serialization round-trip | tests/integration.rs | test_dublin_core_round_trip | PASS |

## 9. Security Extension

| Spec Section | Requirement | Test File | Test Name | Status |
|--------------|-------------|-----------|-----------|--------|
| §signatures | Signature `signer.name` required | tests/integration.rs | test_signature_requires_signer_name | TODO |
| §signatures | Signature `documentId` matches manifest | tests/integration.rs | test_signature_document_id_matches_manifest | TODO |
| §signatures | Signature persistence round-trip | tests/integration.rs | test_signature_persistence | PASS |
| §signatures | Multiple signatures supported | tests/integration.rs | test_multiple_signatures | PASS |

## 10. Extensions

| Spec Section | Requirement | Test File | Test Name | Status |
|--------------|-------------|-----------|-----------|--------|
| §extensions | Required extension unsupported = reject | extensions/mod.rs | test_required_extension_unsupported_error | TODO |
| §extensions | Optional extension unsupported = allow | extensions/mod.rs | test_optional_extension_unsupported_ok | TODO |
| §extensions | Extension ID format: `namespace.name` | extensions/mod.rs | test_extension_id_format_valid | TODO |
| §extensions | Extension version present | extensions/mod.rs | test_extension_version_present | TODO |

---

## Property-Based Tests

| Category | Property | Test File | Test Name | Status |
|----------|----------|-----------|-----------|--------|
| Hash boundary | Metadata subset inclusion consistent | tests/integration.rs | proptest_hash_boundary_metadata_inclusion | TODO |
| Hash determinism | Same content = same hash | tests/integration.rs | proptest_hash_determinism_random_content | TODO |
| Serialization | Content round-trip preserves structure | tests/integration.rs | proptest_content_serialization_roundtrip | TODO |
| Block structure | Valid blocks serialize correctly | tests/integration.rs | proptest_block_structure_constraints | TODO |

---

## Summary

| Category | Total | Passing | TODO |
|----------|-------|---------|------|
| Container Format | 5 | 3 | 1 |
| Manifest | 13 | 7 | 6 |
| Content Blocks | 3 | 3 | 0 |
| Document Hashing | 16 | 4 | 12 |
| State Machine | 10 | 7 | 3 |
| Asset Embedding | 7 | 3 | 4 |
| Provenance/Lineage | 12 | 5 | 7 |
| Metadata | 3 | 1 | 2 |
| Security | 4 | 2 | 2 |
| Extensions | 4 | 0 | 4 |
| Property-Based | 4 | 0 | 4 |
| **Total** | **81** | **35** | **45** |

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
