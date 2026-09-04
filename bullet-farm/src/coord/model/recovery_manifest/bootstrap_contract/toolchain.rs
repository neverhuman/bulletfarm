use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::coord::CoordError;

use super::{
    ComponentEvidenceClassV1, RecoveryBootstrapBuilderContractV1, canonical_bound, domain_id,
    invalid, validate_identity, validate_prefixed, validate_sha256,
};

const TOOLCHAIN_KIND: &str = "bullet.coord.recovery-bootstrap-toolchain-contract.v1";
const TOOLCHAIN_DOMAIN: &str = "bullet-family.coord.recovery-bootstrap-toolchain-contract.v1";
const MAX_MEMBER_BYTES: u64 = 512 * 1024 * 1024;
const MAX_MEMBERS: usize = 512;
const MAX_PATH_BYTES: usize = 512;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(in crate::coord) enum ToolchainRoleV1 {
    Git,
    Cargo,
    Rustc,
    Linker,
    Sysroot,
    RuntimeLoader,
    RuntimeLibrary,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(in crate::coord) enum ToolchainArtifactKindV1 {
    RegularFile,
    DirectoryTree,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::coord) struct ToolchainMemberV1 {
    pub(super) role: ToolchainRoleV1,
    pub(super) absolute_path: String,
    pub(super) artifact_kind: ToolchainArtifactKindV1,
    pub(super) owner_uid: u32,
    pub(super) owner_gid: u32,
    pub(super) mode: u32,
    pub(super) link_count: u64,
    pub(super) byte_length: u64,
    pub(super) sha256: String,
}

impl ToolchainMemberV1 {
    pub(in crate::coord) fn observed(
        role: ToolchainRoleV1,
        absolute_path: String,
        artifact_kind: ToolchainArtifactKindV1,
        mode: u32,
        byte_length: u64,
        sha256: String,
    ) -> Self {
        Self {
            role,
            absolute_path,
            artifact_kind,
            owner_uid: 0,
            owner_gid: 0,
            mode,
            link_count: 1,
            byte_length,
            sha256,
        }
    }

    fn validate(&self) -> Result<(), CoordError> {
        validate_path(&self.absolute_path, "toolchain member path")?;
        validate_sha256(&self.sha256, "toolchain member SHA-256")?;
        if self.owner_uid != 0
            || self.owner_gid != 0
            || self.link_count != 1
            || !(1..=MAX_MEMBER_BYTES).contains(&self.byte_length)
            || self.mode & 0o222 != 0
        {
            return Err(invalid(
                "toolchain member custody, size, link, or mode is unsafe",
            ));
        }
        let expected = match self.role {
            ToolchainRoleV1::Git => fixed("/toolchain/bin/git", 0o555),
            ToolchainRoleV1::Cargo => fixed("/toolchain/bin/cargo", 0o555),
            ToolchainRoleV1::Rustc => fixed("/toolchain/bin/rustc", 0o555),
            ToolchainRoleV1::Linker => fixed("/toolchain/bin/cc", 0o555),
            ToolchainRoleV1::Sysroot => (
                "/toolchain/rust/sysroot",
                ToolchainArtifactKindV1::DirectoryTree,
                0o555,
            ),
            ToolchainRoleV1::RuntimeLoader => fixed("/toolchain/lib/ld-linux-x86-64.so.2", 0o555),
            ToolchainRoleV1::RuntimeLibrary => {
                if !self.absolute_path.starts_with("/toolchain/lib/") {
                    return Err(invalid(
                        "runtime library is outside the closed toolchain library root",
                    ));
                }
                fixed(self.absolute_path.as_str(), 0o444)
            }
        };
        if self.absolute_path != expected.0
            || self.artifact_kind != expected.1
            || self.mode != expected.2
        {
            return Err(invalid(
                "toolchain member role, path, kind, or mode is inconsistent",
            ));
        }
        Ok(())
    }
}

fn fixed(path: &str, mode: u32) -> (&str, ToolchainArtifactKindV1, u32) {
    (path, ToolchainArtifactKindV1::RegularFile, mode)
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::coord) struct RecoveryBootstrapToolchainContractV1 {
    kind: String,
    schema_version: u32,
    authority: ComponentEvidenceClassV1,
    pub(super) contract_id: String,
    pub(super) builder_contract_id: String,
    inventory_complete: bool,
    pub(super) member_count: u32,
    pub(super) members: Vec<ToolchainMemberV1>,
}

impl RecoveryBootstrapToolchainContractV1 {
    pub(in crate::coord) fn from_members(
        builder: &RecoveryBootstrapBuilderContractV1,
        members: Vec<ToolchainMemberV1>,
    ) -> Result<Self, CoordError> {
        builder.validate()?;
        let mut value = Self {
            kind: TOOLCHAIN_KIND.to_owned(),
            schema_version: 1,
            authority: ComponentEvidenceClassV1::ComponentOnly,
            contract_id: String::new(),
            builder_contract_id: builder.contract_id.clone(),
            inventory_complete: true,
            member_count: u32::try_from(members.len())
                .map_err(|_| invalid("toolchain member count overflowed"))?,
            members,
        };
        value.contract_id = value.expected_id()?;
        value.validate()?;
        Ok(value)
    }

    pub(in crate::coord) fn validate(&self) -> Result<(), CoordError> {
        if self.kind != TOOLCHAIN_KIND
            || self.schema_version != 1
            || !self.inventory_complete
            || !(7..=MAX_MEMBERS).contains(&self.members.len())
            || self.member_count != self.members.len() as u32
        {
            return Err(invalid(
                "toolchain contract kind, completeness, or count is invalid",
            ));
        }
        validate_prefixed(
            &self.builder_contract_id,
            "rbc_",
            64,
            "builder contract reference",
        )?;
        let expected_roles = [
            ToolchainRoleV1::Git,
            ToolchainRoleV1::Cargo,
            ToolchainRoleV1::Rustc,
            ToolchainRoleV1::Linker,
            ToolchainRoleV1::Sysroot,
            ToolchainRoleV1::RuntimeLoader,
        ];
        let mut previous: Option<(ToolchainRoleV1, &str)> = None;
        let mut paths = BTreeSet::new();
        let mut counts = [0_u16; 7];
        for member in &self.members {
            member.validate()?;
            let key = (member.role, member.absolute_path.as_str());
            if previous.is_some_and(|item| item >= key)
                || !paths.insert(member.absolute_path.as_str())
            {
                return Err(invalid(
                    "toolchain members must be sorted with globally unique paths",
                ));
            }
            counts[member.role as usize] += 1;
            previous = Some(key);
        }
        if expected_roles
            .iter()
            .any(|role| counts[*role as usize] != 1)
            || counts[ToolchainRoleV1::RuntimeLibrary as usize] == 0
        {
            return Err(invalid(
                "toolchain inventory must contain every singleton role and runtime libraries",
            ));
        }
        validate_identity(
            &self.contract_id,
            "rtc_",
            self.expected_id()?,
            "toolchain contract",
        )?;
        canonical_bound(self)
    }

    fn expected_id(&self) -> Result<String, CoordError> {
        let mut identity = self.clone();
        identity.contract_id.clear();
        domain_id(TOOLCHAIN_DOMAIN, "rtc_", &identity)
    }
}

fn validate_path(value: &str, label: &str) -> Result<(), CoordError> {
    let valid = value.len() <= MAX_PATH_BYTES
        && value.starts_with('/')
        && value != "/"
        && value.is_ascii()
        && !value.as_bytes().iter().any(|byte| byte.is_ascii_control())
        && value
            .split('/')
            .skip(1)
            .all(|part| !part.is_empty() && part != "." && part != "..");
    valid
        .then_some(())
        .ok_or_else(|| invalid(format!("{label} is not a normalized bounded absolute path")))
}
