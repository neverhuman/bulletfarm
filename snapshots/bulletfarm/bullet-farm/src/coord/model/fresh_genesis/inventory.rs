use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::coord::CoordError;

use super::{
    canonical_bound, decode_path_hex, domain_id, invalid, safe, validate_absolute_path_hex,
    validate_destination_name_hex, validate_mode, validate_relative_path_hex,
    validate_tagged_digest,
};

const INVENTORY_DOMAIN: &str = "bullet-family.coord.fresh-genesis-incident-inventory.v1";
pub(super) const MAX_INVENTORY_NODES: usize = 2_048;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum IncidentInventoryKindV1 {
    IncidentInventoryV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum IncidentInventoryNodeTypeV1 {
    Directory,
    RegularFile,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IncidentDirectoryIdentityV1 {
    pub(crate) absolute_path_hex: String,
    pub(crate) device: u64,
    pub(crate) inode: u64,
    pub(crate) owner_uid: u32,
    pub(crate) owner_gid: u32,
    pub(crate) mode: u32,
    pub(crate) link_count: u64,
    pub(crate) byte_length: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IncidentInventoryNodeV1 {
    pub(crate) relative_path_hex: String,
    pub(crate) node_type: IncidentInventoryNodeTypeV1,
    pub(crate) owner_uid: u32,
    pub(crate) owner_gid: u32,
    pub(crate) mode: u32,
    pub(crate) link_count: u64,
    pub(crate) byte_length: u64,
    pub(crate) content_sha256: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IncidentInventorySubjectV1 {
    pub(crate) source_directory: IncidentDirectoryIdentityV1,
    pub(crate) destination_name_hex: String,
    pub(crate) node_count: u64,
    pub(crate) directory_count: u64,
    pub(crate) regular_file_count: u64,
    pub(crate) regular_file_byte_length: u64,
    pub(crate) nodes: Vec<IncidentInventoryNodeV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IncidentInventoryV1 {
    kind: IncidentInventoryKindV1,
    schema_version: u32,
    pub(crate) inventory_id: String,
    pub(crate) subject: IncidentInventorySubjectV1,
}

impl IncidentInventoryV1 {
    pub(crate) const fn maximum_nodes() -> usize {
        MAX_INVENTORY_NODES
    }

    pub(crate) const fn maximum_path_bytes() -> usize {
        super::MAX_PATH_BYTES
    }

    pub(crate) fn validate_destination_name_hex(value: &str) -> Result<(), CoordError> {
        validate_destination_name_hex(value)
    }

    pub(crate) fn from_subject(subject: IncidentInventorySubjectV1) -> Result<Self, CoordError> {
        subject.validate()?;
        let mut value = Self {
            kind: IncidentInventoryKindV1::IncidentInventoryV1,
            schema_version: 1,
            inventory_id: String::new(),
            subject,
        };
        value.inventory_id = value.expected_id()?;
        value.validate()?;
        Ok(value)
    }

    pub(crate) fn validate(&self) -> Result<(), CoordError> {
        if self.kind != IncidentInventoryKindV1::IncidentInventoryV1 || self.schema_version != 1 {
            return Err(invalid(
                "incident inventory kind or schema version is unsupported",
            ));
        }
        self.subject.validate()?;
        if self.inventory_id != self.expected_id()? {
            return Err(invalid(
                "incident inventory ID differs from its exact source and nodes",
            ));
        }
        canonical_bound(self, "incident inventory")
    }

    fn expected_id(&self) -> Result<String, CoordError> {
        domain_id("fgi_", INVENTORY_DOMAIN, &self.subject)
    }
}

impl IncidentInventorySubjectV1 {
    fn validate(&self) -> Result<(), CoordError> {
        self.source_directory.validate()?;
        validate_destination_name_hex(&self.destination_name_hex)?;
        let source = decode_path_hex(
            &self.source_directory.absolute_path_hex,
            "incident source directory",
        )?;
        let destination = decode_path_hex(&self.destination_name_hex, "incident destination name")?;
        if source.rsplit(|byte| *byte == b'/').next() == Some(destination.as_slice()) {
            return Err(invalid(
                "incident destination must differ from its source name",
            ));
        }
        if self.nodes.is_empty() || self.nodes.len() > MAX_INVENTORY_NODES {
            return Err(invalid(
                "incident inventory node count is outside 1..=2,048",
            ));
        }
        if self
            .nodes
            .windows(2)
            .any(|pair| pair[0].relative_path_hex >= pair[1].relative_path_hex)
        {
            return Err(invalid(
                "incident inventory paths must be byte-sorted and unique",
            ));
        }

        let mut directories = 0_u64;
        let mut regular_files = 0_u64;
        let mut regular_file_bytes = 0_u64;
        for node in &self.nodes {
            node.validate()?;
            match node.node_type {
                IncidentInventoryNodeTypeV1::Directory => directories += 1,
                IncidentInventoryNodeTypeV1::RegularFile => {
                    regular_files += 1;
                    regular_file_bytes = regular_file_bytes
                        .checked_add(node.byte_length)
                        .filter(|value| *value <= super::MAX_SAFE_INTEGER)
                        .ok_or_else(|| {
                            invalid("incident regular-file byte count is not JSON-safe")
                        })?;
                }
            }
        }
        let node_types = self
            .nodes
            .iter()
            .map(|node| (node.relative_path_hex.as_str(), node.node_type))
            .collect::<BTreeMap<_, _>>();
        for node in &self.nodes {
            for (index, pair) in node
                .relative_path_hex
                .as_bytes()
                .chunks_exact(2)
                .enumerate()
            {
                if pair == b"2f"
                    && node_types.get(&node.relative_path_hex[..index * 2])
                        != Some(&IncidentInventoryNodeTypeV1::Directory)
                {
                    return Err(invalid(
                        "incident inventory must include every parent directory node",
                    ));
                }
            }
        }
        for (value, label) in [
            (self.node_count, "incident node count"),
            (self.directory_count, "incident directory count"),
            (self.regular_file_count, "incident regular-file count"),
            (
                self.regular_file_byte_length,
                "incident regular-file byte length",
            ),
        ] {
            safe(value, label)?;
        }
        if self.node_count != self.nodes.len() as u64
            || self.directory_count != directories
            || self.regular_file_count != regular_files
            || self.regular_file_byte_length != regular_file_bytes
            || self.node_count != self.directory_count + self.regular_file_count
        {
            return Err(invalid(
                "incident inventory counts differ from its complete node set",
            ));
        }
        Ok(())
    }
}

impl IncidentDirectoryIdentityV1 {
    fn validate(&self) -> Result<(), CoordError> {
        validate_absolute_path_hex(&self.absolute_path_hex, "incident source directory")?;
        safe(self.device, "incident source device")?;
        safe(self.inode, "incident source inode")?;
        safe(self.link_count, "incident source link count")?;
        safe(self.byte_length, "incident source byte length")?;
        validate_mode(self.mode, "incident source mode")?;
        if self.device == 0 || self.inode == 0 || self.link_count == 0 {
            return Err(invalid(
                "incident source directory identity must have positive device, inode, and links",
            ));
        }
        Ok(())
    }
}

impl IncidentInventoryNodeV1 {
    fn validate(&self) -> Result<(), CoordError> {
        validate_relative_path_hex(&self.relative_path_hex, "incident relative path")?;
        validate_mode(self.mode, "incident node mode")?;
        safe(self.link_count, "incident node link count")?;
        safe(self.byte_length, "incident node byte length")?;
        if self.link_count == 0 {
            return Err(invalid("incident node link count must be positive"));
        }
        match (&self.node_type, &self.content_sha256) {
            (IncidentInventoryNodeTypeV1::Directory, None) => Ok(()),
            (IncidentInventoryNodeTypeV1::RegularFile, Some(digest)) => {
                validate_tagged_digest(digest, "sha256:", 64, "regular-file SHA-256")
            }
            (IncidentInventoryNodeTypeV1::Directory, Some(_)) => Err(invalid(
                "incident directory must not claim a regular-file digest",
            )),
            (IncidentInventoryNodeTypeV1::RegularFile, None) => Err(invalid(
                "incident regular file must carry its exact SHA-256 digest",
            )),
        }
    }
}
