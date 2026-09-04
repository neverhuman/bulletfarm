#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "COMPONENT_ONLY contracts await separate observer and verifier custody"
    )
)]

use serde::{Deserialize, Serialize};

use crate::coord::{CoordError, generation::manifest::Sha256Digest};

use super::{invalid, validate_prefixed};

const BUILDER_KIND: &str = "bullet.coord.recovery-bootstrap-builder-contract.v1";
const BUILDER_DOMAIN: &str = "bullet-family.coord.recovery-bootstrap-builder-contract.v1";
const COMMAND_KIND: &str = "bullet.coord.recovery-bootstrap-command-contract.v1";
const COMMAND_DOMAIN: &str = "bullet-family.coord.recovery-bootstrap-command-contract.v1";
const TARGET: &str = "x86_64-unknown-linux-gnu";
const MAX_ROOTFS_BYTES: u64 = 2 * 1024 * 1024 * 1024;

#[path = "bootstrap_contract/toolchain.rs"]
mod toolchain;
pub(in crate::coord) use toolchain::{
    RecoveryBootstrapToolchainContractV1, ToolchainArtifactKindV1, ToolchainMemberV1,
    ToolchainRoleV1,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum ComponentEvidenceClassV1 {
    ComponentOnly,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum BuilderOperatingSystemV1 {
    Linux,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum BuilderArchitectureV1 {
    X86_64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum BuilderNetworkV1 {
    None,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::coord) struct RecoveryBootstrapBuilderContractV1 {
    kind: String,
    schema_version: u32,
    authority: ComponentEvidenceClassV1,
    contract_id: String,
    operating_system: BuilderOperatingSystemV1,
    architecture: BuilderArchitectureV1,
    rootfs_archive_sha256: String,
    rootfs_archive_byte_length: u64,
    rootfs_tree_sha256: String,
    rootfs_owner_uid: u32,
    rootfs_owner_gid: u32,
    rootfs_read_only: bool,
    builder_uid: u32,
    builder_gid: u32,
    network: BuilderNetworkV1,
    inherited_environment: bool,
    credential_input_count: u32,
}

impl RecoveryBootstrapBuilderContractV1 {
    pub(in crate::coord) fn from_rootfs(
        archive_sha256: String,
        archive_byte_length: u64,
        tree_sha256: String,
    ) -> Result<Self, CoordError> {
        let mut value = Self {
            kind: BUILDER_KIND.to_owned(),
            schema_version: 1,
            authority: ComponentEvidenceClassV1::ComponentOnly,
            contract_id: String::new(),
            operating_system: BuilderOperatingSystemV1::Linux,
            architecture: BuilderArchitectureV1::X86_64,
            rootfs_archive_sha256: archive_sha256,
            rootfs_archive_byte_length: archive_byte_length,
            rootfs_tree_sha256: tree_sha256,
            rootfs_owner_uid: 0,
            rootfs_owner_gid: 0,
            rootfs_read_only: true,
            builder_uid: 65_532,
            builder_gid: 65_532,
            network: BuilderNetworkV1::None,
            inherited_environment: false,
            credential_input_count: 0,
        };
        value.contract_id = value.expected_id()?;
        value.validate()?;
        Ok(value)
    }

    pub(in crate::coord) fn validate(&self) -> Result<(), CoordError> {
        if self.kind != BUILDER_KIND
            || self.schema_version != 1
            || self.operating_system != BuilderOperatingSystemV1::Linux
            || self.architecture != BuilderArchitectureV1::X86_64
        {
            return Err(invalid(
                "bootstrap builder platform is not closed Linux/x86_64",
            ));
        }
        validate_sha256(
            &self.rootfs_archive_sha256,
            "builder rootfs archive SHA-256",
        )?;
        validate_sha256(&self.rootfs_tree_sha256, "builder rootfs tree SHA-256")?;
        if !(1..=MAX_ROOTFS_BYTES).contains(&self.rootfs_archive_byte_length)
            || self.rootfs_owner_uid != 0
            || self.rootfs_owner_gid != 0
            || !self.rootfs_read_only
            || self.builder_uid == 0
            || self.builder_gid == 0
            || self.builder_uid != self.builder_gid
            || self.network != BuilderNetworkV1::None
            || self.inherited_environment
            || self.credential_input_count != 0
        {
            return Err(invalid(
                "bootstrap builder custody or isolation contract is unsafe",
            ));
        }
        validate_identity(
            &self.contract_id,
            "rbc_",
            self.expected_id()?,
            "builder contract",
        )?;
        canonical_bound(self)
    }

    fn expected_id(&self) -> Result<String, CoordError> {
        let mut identity = self.clone();
        identity.contract_id.clear();
        domain_id(BUILDER_DOMAIN, "rbc_", &identity)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum EnvironmentInheritanceV1 {
    Empty,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct EnvironmentVariableV1 {
    name: String,
    value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct BuildRunContractV1 {
    run_ordinal: u32,
    build_root: String,
    working_directory: String,
    environment: Vec<EnvironmentVariableV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::coord) struct RecoveryBootstrapCommandContractV1 {
    kind: String,
    schema_version: u32,
    authority: ComponentEvidenceClassV1,
    contract_id: String,
    builder_contract_id: String,
    toolchain_contract_id: String,
    target_triple: String,
    profile: String,
    network_mode: String,
    environment_inheritance: EnvironmentInheritanceV1,
    argv: Vec<String>,
    runs: [BuildRunContractV1; 2],
}

impl RecoveryBootstrapCommandContractV1 {
    pub(in crate::coord) fn exact(
        builder: &RecoveryBootstrapBuilderContractV1,
        toolchain: &RecoveryBootstrapToolchainContractV1,
    ) -> Result<Self, CoordError> {
        builder.validate()?;
        toolchain.validate()?;
        if toolchain.builder_contract_id != builder.contract_id {
            return Err(invalid(
                "toolchain contract does not bind the selected builder",
            ));
        }
        let mut value = Self {
            kind: COMMAND_KIND.to_owned(),
            schema_version: 1,
            authority: ComponentEvidenceClassV1::ComponentOnly,
            contract_id: String::new(),
            builder_contract_id: builder.contract_id.clone(),
            toolchain_contract_id: toolchain.contract_id.clone(),
            target_triple: TARGET.to_owned(),
            profile: "release".to_owned(),
            network_mode: "OFFLINE".to_owned(),
            environment_inheritance: EnvironmentInheritanceV1::Empty,
            argv: expected_argv(),
            runs: [expected_run(1), expected_run(2)],
        };
        value.contract_id = value.expected_id()?;
        value.validate()?;
        Ok(value)
    }

    pub(in crate::coord) fn validate(&self) -> Result<(), CoordError> {
        if self.kind != COMMAND_KIND
            || self.schema_version != 1
            || self.target_triple != TARGET
            || self.profile != "release"
            || self.network_mode != "OFFLINE"
            || self.environment_inheritance != EnvironmentInheritanceV1::Empty
            || self.argv != expected_argv()
            || self.runs != [expected_run(1), expected_run(2)]
        {
            return Err(invalid(
                "bootstrap command must be the exact isolated two-run locked offline release command",
            ));
        }
        validate_prefixed(
            &self.builder_contract_id,
            "rbc_",
            64,
            "command builder reference",
        )?;
        validate_prefixed(
            &self.toolchain_contract_id,
            "rtc_",
            64,
            "command toolchain reference",
        )?;
        validate_identity(
            &self.contract_id,
            "rcc_",
            self.expected_id()?,
            "command contract",
        )?;
        canonical_bound(self)
    }

    fn expected_id(&self) -> Result<String, CoordError> {
        domain_id(
            COMMAND_DOMAIN,
            "rcc_",
            &(
                &self.kind,
                self.schema_version,
                self.authority,
                &self.builder_contract_id,
                &self.toolchain_contract_id,
                &self.target_triple,
                &self.profile,
                &self.network_mode,
                self.environment_inheritance,
                &self.argv,
                &self.runs,
            ),
        )
    }
}

pub(super) fn contract_digests(
    builder: &RecoveryBootstrapBuilderContractV1,
    toolchain: &RecoveryBootstrapToolchainContractV1,
    command: &RecoveryBootstrapCommandContractV1,
) -> Result<(String, String, String), CoordError> {
    builder.validate()?;
    toolchain.validate()?;
    command.validate()?;
    if toolchain.builder_contract_id != builder.contract_id
        || command.builder_contract_id != builder.contract_id
        || command.toolchain_contract_id != toolchain.contract_id
    {
        return Err(invalid(
            "bootstrap build contracts do not form one exact bound set",
        ));
    }
    Ok((
        sealed(builder)?.as_str().to_owned(),
        sealed(toolchain)?.as_str().to_owned(),
        sealed(command)?.as_str().to_owned(),
    ))
}

pub(in crate::coord) type BootstrapContractRefsV1<'a> = (
    &'a RecoveryBootstrapBuilderContractV1,
    &'a RecoveryBootstrapToolchainContractV1,
    &'a RecoveryBootstrapCommandContractV1,
);

fn expected_argv() -> Vec<String> {
    [
        "/toolchain/bin/cargo",
        "build",
        "--locked",
        "--offline",
        "--release",
        "--target",
        TARGET,
        "--package",
        "bullet-family",
        "--bin",
        "bullet-family",
    ]
    .map(str::to_owned)
    .to_vec()
}

fn expected_run(ordinal: u32) -> BuildRunContractV1 {
    let root = format!("/build/run-{ordinal}");
    let source = format!("{root}/source");
    let environment = [("CARGO_HOME", format!("{root}/cargo-home")), ("CARGO_NET_OFFLINE", "true".to_owned()),
        ("CARGO_TARGET_DIR", format!("{root}/target")), ("HOME", format!("{root}/empty-home")),
        ("LANG", "C".to_owned()), ("LC_ALL", "C".to_owned()), ("PATH", "/toolchain/bin".to_owned()),
        ("RUSTFLAGS", format!("-C linker=/toolchain/bin/cc --remap-path-prefix={source}=/usr/src/bullet --remap-path-prefix={root}/target=/usr/src/bullet-target")),
        ("SOURCE_DATE_EPOCH", "946684800".to_owned())]
        .into_iter().map(|(name, value)| EnvironmentVariableV1 { name: name.to_owned(), value }).collect();
    BuildRunContractV1 {
        run_ordinal: ordinal,
        build_root: root,
        working_directory: source,
        environment,
    }
}

fn domain_id(domain: &str, prefix: &str, value: &impl Serialize) -> Result<String, CoordError> {
    Ok(format!(
        "{prefix}{}",
        bullet_wire::hash_canonical(domain, value)
            .map_err(wire)?
            .to_hex()
    ))
}

fn validate_identity(
    actual: &str,
    prefix: &str,
    expected: String,
    label: &str,
) -> Result<(), CoordError> {
    validate_prefixed(actual, prefix, 64, label)?;
    if actual != expected {
        return Err(invalid(format!("{label} does not bind its exact subject")));
    }
    Ok(())
}

fn validate_sha256(value: &str, label: &str) -> Result<(), CoordError> {
    validate_prefixed(value, "sha256:", 64, label)
}

fn canonical_bound(value: &impl Serialize) -> Result<(), CoordError> {
    let bytes = bullet_wire::canonical_json(value).map_err(wire)?;
    if bytes.len() > bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES {
        return Err(invalid(
            "recovery bootstrap contract exceeds the canonical one-MiB bound",
        ));
    }
    Ok(())
}

fn sealed(value: &impl Serialize) -> Result<Sha256Digest, CoordError> {
    let mut bytes = bullet_wire::canonical_json(value).map_err(wire)?;
    bytes.push(b'\n');
    Ok(Sha256Digest::for_bytes(&bytes))
}

fn wire(error: impl std::fmt::Display) -> CoordError {
    invalid(format!("cannot bind recovery bootstrap contract: {error}"))
}

#[cfg(test)]
#[path = "bootstrap_contract/tests.rs"]
mod tests;
