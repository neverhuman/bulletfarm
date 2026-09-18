use std::path::{Component, Path, PathBuf};

use crate::coord::CoordError;

use super::{
    generation::manifest::{
        ArtifactBinding, ByteRange, CreateBodyInput, GenerationManifest, RecoveryArtifacts,
        RelativeArtifactPath, Sha256Digest,
    },
    model::{
        FrozenClaimSubject, RecoveryAuthorizationSignatureV1, RecoveryAuthorizationV1,
        RecoveryBootstrapProvenanceV1, RecoveryFileIdentityV1, RecoveryInspectionArtifactsV1,
        RecoveryInspectionSubjectV1, RecoveryInspectionV1, RecoverySourceInspectionV1,
    },
};

pub(crate) mod authoring;
pub(crate) mod bootstrap_build;
#[cfg(target_os = "linux")]
mod clock;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
mod provenance;
mod trust;

#[cfg(all(test, target_os = "linux"))]
pub(in crate::coord) use clock::{
    TestClockGuard, install_test_clock, install_test_clock_pair, set_test_clock,
};
#[cfg(all(test, target_os = "linux"))]
pub(crate) use provenance::test_preflight_document;
#[cfg(test)]
pub(in crate::coord) use trust::{TestAuthority, test_authority, test_authority_with_decision};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RecoveryInspectionCommand {
    pub(crate) interrupted_capture: PathBuf,
    pub(crate) tainted_generation: PathBuf,
    pub(crate) frozen_live_source: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RecoveryProvenanceCommand {
    pub(crate) bootstrap_commit_oid: String,
    pub(crate) cargo_bin: PathBuf,
    pub(crate) rustc_bin: PathBuf,
    pub(crate) source_archive_output: PathBuf,
    pub(crate) output: PathBuf,
}

pub(crate) fn produce_provenance(
    family_root: &Path,
    command: &RecoveryProvenanceCommand,
) -> Result<RecoveryBootstrapProvenanceV1, CoordError> {
    #[cfg(target_os = "linux")]
    {
        provenance::produce(family_root, command)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (family_root, command);
        Err(unsupported())
    }
}

pub(crate) fn inspect(
    family_root: &Path,
    command: &RecoveryInspectionCommand,
) -> Result<RecoveryInspectionV1, CoordError> {
    #[cfg(target_os = "linux")]
    {
        linux::inspect(family_root, command)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (family_root, command);
        Err(unsupported())
    }
}

pub(crate) fn manifest(
    family_root: &Path,
    command: &RecoveryInspectionCommand,
    sealed: &RecoveryInspectionV1,
    authorization: &RecoveryAuthorizationV1,
    signature: &RecoveryAuthorizationSignatureV1,
    provenance: &RecoveryBootstrapProvenanceV1,
) -> Result<GenerationManifest, CoordError> {
    #[cfg(target_os = "linux")]
    {
        sealed.validate()?;
        let observed = linux::inspect(family_root, command)?;
        if &observed != sealed {
            return Err(changed(
                "sealed inspection differs from complete source rederivation",
            ));
        }
        let authorized = authorize(sealed, authorization, signature, provenance)?;
        authorized.require_active()?;
        Ok(authorized.manifest)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (
            family_root,
            command,
            sealed,
            authorization,
            signature,
            provenance,
        );
        Err(unsupported())
    }
}

#[cfg(target_os = "linux")]
pub(crate) struct AuthorizedManifest {
    pub(crate) manifest: GenerationManifest,
    verified: trust::VerifiedAuthorization,
}

#[cfg(target_os = "linux")]
impl AuthorizedManifest {
    pub(crate) fn require_active(&self) -> Result<(), CoordError> {
        self.verified.require_active(clock::observe()?)
    }

    pub(crate) fn require_read_only_replay(&self) -> Result<(), CoordError> {
        self.verified.require_read_only_replay(clock::observe()?)
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn authorize(
    sealed: &RecoveryInspectionV1,
    authorization: &RecoveryAuthorizationV1,
    signature: &RecoveryAuthorizationSignatureV1,
    provenance: &RecoveryBootstrapProvenanceV1,
) -> Result<AuthorizedManifest, CoordError> {
    sealed.validate()?;
    let verified =
        trust::verify_observed(sealed, authorization, signature, provenance, clock::observe)?;
    let manifest = manifest_from_verified(sealed, &verified)?;
    Ok(AuthorizedManifest { manifest, verified })
}

#[cfg(target_os = "linux")]
fn manifest_from_verified(
    sealed: &RecoveryInspectionV1,
    verified: &trust::VerifiedAuthorization,
) -> Result<GenerationManifest, CoordError> {
    let subject = &sealed.subject;
    let body = super::generation::manifest::create_body(CreateBodyInput {
        recovery_operator: verified.recovery_operator.clone(),
        recovery_policy_sha256: Sha256Digest::parse(verified.policy_sha256.clone())?,
        operator_decision_sha256: Sha256Digest::parse(verified.operator_decision_sha256.clone())?,
        replay_contract_version: verified.replay_contract_version,
        replay_contract_sha256: Sha256Digest::parse(verified.replay_contract_sha256.clone())?,
        bootstrap_commit_oid: verified.bootstrap_commit_oid.clone(),
        bootstrap_paths: verified.bootstrap_paths.clone(),
        legacy_source_device: subject.artifacts.frozen_live_source.identity.device,
        legacy_source_inode: subject.artifacts.frozen_live_source.identity.inode,
        parent_generation: subject.parent_generation.clone(),
        incident_at_unix_ms: subject.incident_at_unix_ms,
        recovered_at_unix_ms: verified.decision_at_unix_ms,
        trusted_record_count: subject.trusted_record_count,
        trusted_projection_inventory: subject.trusted_projection_inventory.clone(),
        discarded_range: subject.discarded_range,
        ambiguous_tail_range: subject.ambiguous_tail_range,
        ambiguous_tail_sha256: subject.ambiguous_tail_sha256.clone(),
        artifacts: subject.artifacts.manifest_artifacts(),
        trusted_state_blake3: subject.trusted_state_blake3.clone(),
        frozen_claims: subject.frozen_claims.clone(),
        post_prefix_inventory_blake3: subject.post_prefix_inventory_blake3.clone(),
    })?;
    GenerationManifest::from_body(body)
}

pub(crate) fn require_normalized_absolute(
    path: &Path,
    label: &'static str,
) -> Result<(), CoordError> {
    if !is_normalized_absolute(path) {
        return Err(invalid(format!(
            "{label} path must be normalized absolute lexical bytes"
        )));
    }
    Ok(())
}

pub(crate) fn is_normalized_absolute(path: &Path) -> bool {
    let Some(raw) = path.as_os_str().to_str() else {
        return false;
    };
    let bytes = raw.as_bytes();
    path.is_absolute()
        && bytes.len() > 1
        && !bytes.ends_with(b"/")
        && !bytes.windows(2).any(|pair| pair == b"//")
        && !bytes.windows(3).any(|part| part == b"/./")
        && !bytes.windows(4).any(|part| part == b"/../")
        && !bytes.ends_with(b"/.")
        && !bytes.ends_with(b"/..")
        && !path.components().enumerate().any(|(index, component)| {
            if index == 0 {
                component != Component::RootDir
            } else {
                !matches!(component, Component::Normal(_))
            }
        })
}

#[cfg(not(target_os = "linux"))]
fn unsupported() -> CoordError {
    CoordError::new(
        "COORD_RECOVERY_PLATFORM_UNSUPPORTED",
        "recovery inspection and manifest production require exact Linux descriptor proof",
    )
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_RECOVERY_MANIFEST_PRODUCTION", reason)
}

fn changed(reason: impl Into<String>) -> CoordError {
    CoordError::new("RECOVERY_INSPECTION_CHANGED", reason)
}

#[cfg(all(test, target_os = "linux"))]
mod provenance_archive_tests {
    use std::collections::BTreeMap;

    use sha2::{Digest, Sha256};
    use tar::{EntryType, Header};

    use super::provenance::{
        TreeBlob,
        archive::{
            MAX_ARCHIVE_BYTES, MAX_ENTRIES, increment_entry_count, inspect_logical_archive,
            pax_records, validate_archive_length, verify_blob_batch, verify_raw_archive,
        },
        parse_tree,
    };

    const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn pax(key: &str, value: &[u8]) -> Vec<u8> {
        let mut length = key.len() + value.len() + 4;
        loop {
            let next = key.len() + value.len() + length.to_string().len() + 3;
            if next == length {
                return [format!("{length} {key}=").as_bytes(), value, b"\n"].concat();
            }
            length = next;
        }
    }

    fn member(
        kind: EntryType,
        path: &str,
        mode: u32,
        declared: u64,
        content: &[u8],
        link: Option<&str>,
    ) -> Vec<u8> {
        let mut header = Header::new_ustar();
        header.set_entry_type(kind);
        header.set_path(path).unwrap();
        header.set_mode(mode);
        header.set_size(declared);
        if let Some(link) = link {
            header.set_link_name(link).unwrap();
        }
        header.set_cksum();
        let mut bytes = header.as_bytes().to_vec();
        bytes.extend_from_slice(content);
        bytes.resize(bytes.len().next_multiple_of(512), 0);
        bytes
    }

    fn exact_member(kind: EntryType, path: &str, mode: u32, content: &[u8]) -> Vec<u8> {
        member(kind, path, mode, content.len() as u64, content, None)
    }

    fn tar(members: impl IntoIterator<Item = Vec<u8>>) -> Vec<u8> {
        let mut bytes = members.into_iter().flatten().collect::<Vec<_>>();
        bytes.extend_from_slice(&[0; 1_024]);
        bytes
    }

    fn global(oid: &str) -> Vec<u8> {
        exact_member(
            EntryType::XGlobalHeader,
            "pax_global_header",
            0o644,
            &pax("comment", oid.as_bytes()),
        )
    }

    fn tree(entries: &[(&str, u32, &str)]) -> BTreeMap<String, TreeBlob> {
        entries
            .iter()
            .map(|(path, mode, oid)| {
                (
                    (*path).to_owned(),
                    TreeBlob {
                        mode: *mode,
                        oid: (*oid).to_owned(),
                    },
                )
            })
            .collect()
    }

    fn base_members() -> Vec<Vec<u8>> {
        vec![
            exact_member(EntryType::Regular, "Cargo.lock", 0o664, b"lock"),
            exact_member(
                EntryType::Regular,
                "rust-toolchain.toml",
                0o664,
                b"[toolchain]\nchannel='1'\n",
            ),
        ]
    }

    fn base_tree() -> BTreeMap<String, TreeBlob> {
        tree(&[("Cargo.lock", 0o664, A), ("rust-toolchain.toml", 0o664, B)])
    }

    fn tree_with(path: &str) -> BTreeMap<String, TreeBlob> {
        let mut value = base_tree();
        value.insert(
            path.to_owned(),
            TreeBlob {
                mode: 0o664,
                oid: A.to_owned(),
            },
        );
        value
    }

    fn with_first(
        kind: EntryType,
        mode: u32,
        size: u64,
        content: &[u8],
        link: Option<&str>,
    ) -> Vec<Vec<u8>> {
        let mut members = base_members();
        members[0] = member(kind, "Cargo.lock", mode, size, content, link);
        members
    }

    #[test]
    fn raw_marker_extensions_padding_and_trailer_are_closed() {
        let valid = tar([global(A)]);
        verify_raw_archive(&valid, A).unwrap();
        assert!(verify_raw_archive(&valid, B).is_err());
        let file = exact_member(EntryType::Regular, "stub", 0o664, b"x");
        let local = exact_member(EntryType::XHeader, "pax", 0o644, &pax("path", b"long"));
        let gnu = exact_member(EntryType::GNULongName, "LongLink", 0o644, b"gnu\0");
        for extension in [local.clone(), gnu] {
            verify_raw_archive(&tar([global(A), extension, file.clone()]), A).unwrap();
        }
        assert!(verify_raw_archive(&tar([global(A), local.clone()]), A).is_err());
        assert!(
            verify_raw_archive(&tar([global(A), local.clone(), local, file.clone()]), A).is_err()
        );
        assert!(
            verify_raw_archive(
                &tar([
                    global(A),
                    exact_member(EntryType::GNULongName, "LongLink", 0o644, b"bad"),
                    file,
                ]),
                A,
            )
            .is_err()
        );
        let marker_length = pax("comment", A.as_bytes()).len();
        let mut padding = valid.clone();
        padding[512 + marker_length] = 1;
        assert!(verify_raw_archive(&padding, A).is_err());
        let mut trailing = valid.clone();
        *trailing.last_mut().unwrap() = 1;
        assert!(verify_raw_archive(&trailing, A).is_err());
        assert!(verify_raw_archive(&valid[..valid.len() - 512], A).is_err());
    }

    #[test]
    fn pax_metadata_archive_and_total_entry_bounds_are_closed() {
        assert_eq!(pax_records(&pax("path", b"file")).unwrap().len(), 1);
        let mut hidden = pax("path", b"file");
        hidden.push(b'\n');
        hidden.extend(pax("comment", b"hidden"));
        assert!(pax_records(&hidden).is_err());
        let duplicate = [pax("path", b"a"), pax("path", b"b")].concat();
        assert!(pax_records(&duplicate).is_err());
        assert!(pax_records(b"5 a=b\n").is_err());
        assert!(pax_records(b"06 a=b\n").is_err());
        assert!(validate_archive_length(0).is_err());
        assert!(validate_archive_length(513).is_err());
        validate_archive_length(MAX_ARCHIVE_BYTES).unwrap();
        assert!(validate_archive_length(MAX_ARCHIVE_BYTES + 512).is_err());
        let mut count = MAX_ENTRIES - 1;
        increment_entry_count(&mut count).unwrap();
        assert_eq!(count, MAX_ENTRIES);
        assert!(increment_entry_count(&mut count).is_err());
        let oversized = vec![b'x'; 4 * 1024 + 1];
        assert!(
            verify_raw_archive(
                &tar([
                    global(A),
                    exact_member(EntryType::XHeader, "pax", 0o644, &oversized),
                ]),
                A,
            )
            .is_err()
        );
        let mut many = Vec::with_capacity(MAX_ENTRIES + 1);
        many.push(global(A));
        many.extend(
            (0..MAX_ENTRIES).map(|index| {
                exact_member(EntryType::Directory, &format!("d{index:05}/"), 0o775, b"")
            }),
        );
        assert!(verify_raw_archive(&tar(many), A).is_err());
    }

    #[test]
    fn logical_inventory_refuses_duplicates_modes_types_links_sets_and_sizes() {
        let valid = tar(base_members());
        inspect_logical_archive(&valid, &base_tree()).unwrap();
        let mut duplicate = base_members();
        duplicate.push(exact_member(EntryType::Regular, "Cargo.lock", 0o664, b"x"));
        for members in [
            duplicate,
            with_first(EntryType::Regular, 0o600, 4, b"lock", None),
            with_first(
                EntryType::Regular,
                0o664,
                4,
                b"lock",
                Some("rust-toolchain.toml"),
            ),
            with_first(
                EntryType::Symlink,
                0o777,
                0,
                b"",
                Some("rust-toolchain.toml"),
            ),
            with_first(EntryType::Regular, 0o664, 0, b"", None),
            with_first(EntryType::Regular, 0o664, 64 * 1024 * 1024 + 1, b"", None),
        ] {
            assert!(inspect_logical_archive(&tar(members), &base_tree()).is_err());
        }
        let extra_tree = tree_with("export-ignored");
        assert!(inspect_logical_archive(&valid, &extra_tree).is_err());
        let nested_tree = tree_with("dir/file");
        let mut missing_directory = base_members();
        missing_directory.push(exact_member(EntryType::Regular, "dir/file", 0o664, b"x"));
        assert!(inspect_logical_archive(&tar(missing_directory), &nested_tree).is_err());
        let mut wrong_directory = base_members();
        wrong_directory.push(exact_member(EntryType::Directory, "dir/", 0o755, b""));
        wrong_directory.push(exact_member(EntryType::Regular, "dir/file", 0o664, b"x"));
        assert!(inspect_logical_archive(&tar(wrong_directory), &nested_tree).is_err());
    }

    #[test]
    fn tree_and_blob_batch_parsers_require_exact_complete_output() {
        let record = format!("100644 blob {A}\tCargo.lock\0");
        let parsed = parse_tree(record.as_bytes(), 40).unwrap();
        assert_eq!(parsed["Cargo.lock"].mode, 0o664);
        for malformed in [
            record.trim_end_matches('\0').as_bytes().to_vec(),
            format!("120000 blob {A}\tCargo.lock\0").into_bytes(),
            format!("100644 commit {A}\tCargo.lock\0").into_bytes(),
            [record.as_bytes(), record.as_bytes()].concat(),
        ] {
            assert!(parse_tree(&malformed, 40).is_err());
        }
        let digest = format!("sha256:{:x}", Sha256::digest(b"x"));
        let sources = vec![("Cargo.lock".to_owned(), 1, digest)];
        let tree = tree(&[("Cargo.lock", 0o664, A)]);
        let good = format!("{A} blob 1\nx\n").into_bytes();
        verify_blob_batch(&good, &tree, &sources).unwrap();
        for malformed in [
            format!("{A} blob 1\ny\n").into_bytes(),
            format!("{A} blob 01\nx\n").into_bytes(),
            [good.as_slice(), b"tail"].concat(),
            format!("{B} blob 1\nx\n").into_bytes(),
        ] {
            assert!(verify_blob_batch(&malformed, &tree, &sources).is_err());
        }
    }
}
