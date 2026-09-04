use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;

use super::*;
use crate::coord::model::recovery_manifest::bootstrap_build::RecoveryBootstrapBuildObservationV1;

fn digest(byte: char) -> String {
    format!("sha256:{}", byte.to_string().repeat(64))
}

fn builder() -> RecoveryBootstrapBuilderContractV1 {
    RecoveryBootstrapBuilderContractV1::from_rootfs(digest('a'), 4096, digest('b')).unwrap()
}

fn member(
    role: ToolchainRoleV1,
    path: &str,
    kind: ToolchainArtifactKindV1,
    mode: u32,
    byte: char,
) -> ToolchainMemberV1 {
    ToolchainMemberV1::observed(role, path.to_owned(), kind, mode, 100, digest(byte))
}

fn members() -> Vec<ToolchainMemberV1> {
    vec![
        member(
            ToolchainRoleV1::Git,
            "/toolchain/bin/git",
            ToolchainArtifactKindV1::RegularFile,
            0o555,
            '1',
        ),
        member(
            ToolchainRoleV1::Cargo,
            "/toolchain/bin/cargo",
            ToolchainArtifactKindV1::RegularFile,
            0o555,
            '2',
        ),
        member(
            ToolchainRoleV1::Rustc,
            "/toolchain/bin/rustc",
            ToolchainArtifactKindV1::RegularFile,
            0o555,
            '3',
        ),
        member(
            ToolchainRoleV1::Linker,
            "/toolchain/bin/cc",
            ToolchainArtifactKindV1::RegularFile,
            0o555,
            '4',
        ),
        member(
            ToolchainRoleV1::Sysroot,
            "/toolchain/rust/sysroot",
            ToolchainArtifactKindV1::DirectoryTree,
            0o555,
            '5',
        ),
        member(
            ToolchainRoleV1::RuntimeLoader,
            "/toolchain/lib/ld-linux-x86-64.so.2",
            ToolchainArtifactKindV1::RegularFile,
            0o555,
            '6',
        ),
        member(
            ToolchainRoleV1::RuntimeLibrary,
            "/toolchain/lib/libc.so.6",
            ToolchainArtifactKindV1::RegularFile,
            0o444,
            '7',
        ),
    ]
}

fn contracts() -> (
    RecoveryBootstrapBuilderContractV1,
    RecoveryBootstrapToolchainContractV1,
    RecoveryBootstrapCommandContractV1,
) {
    let builder = builder();
    let toolchain =
        RecoveryBootstrapToolchainContractV1::from_members(&builder, members()).unwrap();
    let command = RecoveryBootstrapCommandContractV1::exact(&builder, &toolchain).unwrap();
    (builder, toolchain, command)
}

fn assert_invalid(result: Result<(), CoordError>) {
    assert_eq!(
        result.unwrap_err().code(),
        "INVALID_RECOVERY_MANIFEST_PRODUCTION"
    );
}

fn assert_unknown_refused<T: DeserializeOwned + Serialize>(value: &T) {
    let mut unknown = serde_json::to_value(value).unwrap();
    unknown
        .as_object_mut()
        .unwrap()
        .insert("admitted".to_owned(), json!(true));
    assert!(serde_json::from_value::<T>(unknown).is_err());
}

#[test]
fn contracts_are_closed_component_only_deterministic_and_bounded() {
    let (builder, toolchain, command) = contracts();
    let first = contract_digests(&builder, &toolchain, &command).unwrap();
    assert_eq!(
        first,
        contract_digests(&builder, &toolchain, &command).unwrap()
    );
    for encoded in [
        serde_json::to_value(&builder).unwrap(),
        serde_json::to_value(&toolchain).unwrap(),
        serde_json::to_value(&command).unwrap(),
    ] {
        assert_eq!(encoded["authority"], "COMPONENT_ONLY");
    }
    assert_unknown_refused(&builder);
    assert_unknown_refused(&toolchain);
    assert_unknown_refused(&command);
    let encoded = String::from_utf8(bullet_wire::canonical_json(&builder).unwrap()).unwrap();
    let duplicate = encoded.replacen("{", "{\"kind\":\"duplicate\",", 1);
    assert!(
        bullet_wire::decode_canonical::<RecoveryBootstrapBuilderContractV1>(duplicate.as_bytes())
            .is_err()
    );
    let unsafe_number = encoded.replace(
        "\"rootfs_archive_byte_length\":4096",
        "\"rootfs_archive_byte_length\":9007199254740992",
    );
    assert!(
        bullet_wire::decode_canonical::<RecoveryBootstrapBuilderContractV1>(
            unsafe_number.as_bytes()
        )
        .is_err()
    );
}

#[test]
fn builder_refuses_platform_custody_isolation_bounds_and_identity_substitution() {
    let mut value = builder();
    value.kind.push_str(".future");
    assert_invalid(value.validate());
    let mut value = builder();
    value.rootfs_archive_byte_length = 0;
    assert_invalid(value.validate());
    let mut value = builder();
    value.rootfs_archive_byte_length = MAX_ROOTFS_BYTES + 1;
    assert_invalid(value.validate());
    let mut value = builder();
    value.rootfs_read_only = false;
    assert_invalid(value.validate());
    let mut value = builder();
    value.rootfs_owner_uid = 1;
    assert_invalid(value.validate());
    let mut value = builder();
    value.builder_uid = 0;
    assert_invalid(value.validate());
    let mut value = builder();
    value.inherited_environment = true;
    assert_invalid(value.validate());
    let mut value = builder();
    value.credential_input_count = 1;
    assert_invalid(value.validate());
    let mut value = builder();
    value.rootfs_tree_sha256 = digest('f');
    assert_invalid(value.validate());
}

#[test]
fn toolchain_requires_complete_sorted_root_owned_read_only_inventory() {
    let builder = builder();
    let valid = RecoveryBootstrapToolchainContractV1::from_members(&builder, members()).unwrap();
    valid.validate().unwrap();

    let mut hostile = valid.clone();
    hostile.members.remove(3);
    hostile.member_count -= 1;
    assert_invalid(hostile.validate());
    let mut hostile = valid.clone();
    hostile.members.swap(0, 1);
    assert_invalid(hostile.validate());
    let mut hostile = valid.clone();
    hostile.members.push(hostile.members[6].clone());
    hostile.member_count += 1;
    assert_invalid(hostile.validate());
    let mut hostile = valid.clone();
    hostile.members[6].absolute_path = hostile.members[5].absolute_path.clone();
    assert_invalid(hostile.validate());
    let mut hostile = valid.clone();
    hostile.members[0].absolute_path = "/toolchain/bin/../git".to_owned();
    assert_invalid(hostile.validate());
    let mut hostile = valid.clone();
    hostile.members[1].mode = 0o755;
    assert_invalid(hostile.validate());
    let mut hostile = valid.clone();
    hostile.members[2].owner_uid = 1000;
    assert_invalid(hostile.validate());
    let mut hostile = valid.clone();
    hostile.members[3].link_count = 2;
    assert_invalid(hostile.validate());
    let mut hostile = valid.clone();
    hostile.members[4].artifact_kind = ToolchainArtifactKindV1::RegularFile;
    assert_invalid(hostile.validate());
    let mut hostile = valid.clone();
    hostile.members[5].sha256 = format!("sha256:{}", "A".repeat(64));
    assert_invalid(hostile.validate());
    let mut hostile = valid;
    hostile.members[6].absolute_path = "/usr/lib/libc.so.6".to_owned();
    assert_invalid(hostile.validate());
}

#[test]
fn command_refuses_added_arguments_environment_loader_and_nonisolated_runs() {
    let (_, _, valid) = contracts();
    let mut hostile = valid.clone();
    hostile.argv.push("--features=unsafe".to_owned());
    assert_invalid(hostile.validate());
    let mut hostile = valid.clone();
    hostile.profile = "debug".to_owned();
    assert_invalid(hostile.validate());
    let mut hostile = valid.clone();
    hostile.network_mode = "ONLINE".to_owned();
    assert_invalid(hostile.validate());
    let mut hostile = valid.clone();
    hostile.target_triple = "aarch64-unknown-linux-gnu".to_owned();
    assert_invalid(hostile.validate());
    for name in ["AWS_SECRET_ACCESS_KEY", "LD_PRELOAD", "RUSTC_WRAPPER"] {
        let mut hostile = valid.clone();
        hostile.runs[0].environment.push(EnvironmentVariableV1 {
            name: name.to_owned(),
            value: "injected".to_owned(),
        });
        assert_invalid(hostile.validate());
    }
    let mut hostile = valid.clone();
    hostile.runs[1].build_root = hostile.runs[0].build_root.clone();
    assert_invalid(hostile.validate());
    let mut hostile = valid;
    hostile.runs[0].environment[0].value = "/ambient/cargo".to_owned();
    assert_invalid(hostile.validate());
}

#[test]
fn build_observation_derives_digests_from_one_exact_contract_set() {
    let (builder, toolchain, command) = contracts();
    let value = RecoveryBootstrapBuildObservationV1::from_contracts(
        digest('8'),
        (digest('9'), 400),
        (digest('a'), digest('b'), 500),
        (&builder, &toolchain, &command),
        [(700, digest('c')), (700, digest('c'))],
    )
    .unwrap();
    value.validate().unwrap();
    let mut substituted_builder = builder.clone();
    substituted_builder.rootfs_archive_sha256 = digest('d');
    assert!(contract_digests(&substituted_builder, &toolchain, &command).is_err());
    let other_builder =
        RecoveryBootstrapBuilderContractV1::from_rootfs(digest('e'), 4096, digest('f')).unwrap();
    let other_toolchain =
        RecoveryBootstrapToolchainContractV1::from_members(&other_builder, members()).unwrap();
    assert!(RecoveryBootstrapCommandContractV1::exact(&builder, &other_toolchain).is_err());
}
