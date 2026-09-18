use std::{collections::BTreeMap, path::Path};

use super::{family_selection, mismatch, observe, verify_expected};
use crate::coord::{
    CoordError,
    model::{
        ForensicRecordRefV1, RecoveryGitExpectationV1, RecoveryGitLeafStatusV1,
        RecoveryGitLeafTransitionV1,
    },
};

pub(crate) fn derive_recovery_commit(
    family_root: &Path,
    repo: &str,
    commit_oid: &str,
    parent_receipts: &BTreeMap<String, ForensicRecordRefV1>,
) -> Result<RecoveryGitExpectationV1, CoordError> {
    crate::coord::validate_commit_oid(commit_oid)?;
    let before = family_selection(family_root, repo)?;
    let first = observe(&before.checkout, commit_oid)?;
    super::after_first_read();
    let middle = family_selection(family_root, repo)?;
    let second = observe(&middle.checkout, commit_oid)?;
    let after = family_selection(family_root, repo)?;
    if before != middle || middle != after || first != second {
        return Err(mismatch(
            "repository identity or Git objects changed across exact derivation read-back",
        ));
    }
    let parent_oid = first
        .parent_oid
        .strip_prefix("sha1:")
        .ok_or_else(|| mismatch("derived parent OID is not tagged"))?;
    let parent_receipt_observation = parent_receipts
        .get(parent_oid)
        .cloned()
        .ok_or_else(|| mismatch("derived parent commit lacks one exact trusted receipt"))?;
    let expected = RecoveryGitExpectationV1 {
        object_format: first.object_format,
        commit_oid: first.commit_oid,
        raw_commit_bytes: first.raw_commit_bytes,
        raw_commit_sha256: first.raw_commit_sha256,
        parent_oid: first.parent_oid,
        parent_tree_oid: first.parent_tree_oid,
        parent_receipt_observation,
        result_tree_oid: first.result_tree_oid,
        raw_tree_sha256: first.raw_tree_sha256,
        leaf_transitions: first
            .leaves
            .into_iter()
            .map(|leaf| RecoveryGitLeafTransitionV1 {
                status: RecoveryGitLeafStatusV1::Modified,
                path: leaf.path,
                old_mode: leaf.old_mode,
                new_mode: leaf.new_mode,
                old_blob_oid: leaf.old_blob_oid,
                new_blob_oid: leaf.new_blob_oid,
                old_bytes: leaf.old_bytes,
                new_bytes: leaf.new_bytes,
                old_sha256: leaf.old_sha256,
                new_sha256: leaf.new_sha256,
            })
            .collect(),
    };
    verify_expected(&second, &expected)?;
    Ok(expected)
}
