//! A complete pair of recorded inventories, without retirement or admission authority.

use serde::{Deserialize, Serialize};

use crate::coord::CoordError;

use super::{
    IncidentInventoryV1,
    fresh_genesis::{canonical_bound, domain_id, invalid, validate_absolute_path_hex},
};

const SUBJECT_DOMAIN: &str = "bullet-family.coord.fresh-preservation-subject.v1";
const OUTER_PARENT: &[u8] = b"/.bullet-family";
const HUB_PARENT: &[u8] = b"/bullet-farm/.bullet-family";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum PreservationKindV1 {
    FreshPreservationSubjectV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FreshPreservationSubjectV1 {
    kind: PreservationKindV1,
    schema_version: u32,
    pub(crate) preservation_id: String,
    pub(crate) family_root_path_hex: String,
    pub(crate) outer_inventory: IncidentInventoryV1,
    pub(crate) hub_inventory: IncidentInventoryV1,
}

#[derive(Serialize)]
struct PreservationIdentity<'a> {
    family_root_path_hex: &'a str,
    outer_inventory: &'a IncidentInventoryV1,
    hub_inventory: &'a IncidentInventoryV1,
}

impl FreshPreservationSubjectV1 {
    pub(crate) fn from_inventories(
        family_root_path_hex: String,
        outer_inventory: IncidentInventoryV1,
        hub_inventory: IncidentInventoryV1,
    ) -> Result<Self, CoordError> {
        let mut value = Self {
            kind: PreservationKindV1::FreshPreservationSubjectV1,
            schema_version: 1,
            preservation_id: String::new(),
            family_root_path_hex,
            outer_inventory,
            hub_inventory,
        };
        value.validate_inventories()?;
        value.preservation_id = value.expected_id()?;
        value.validate()?;
        Ok(value)
    }

    pub(crate) fn validate(&self) -> Result<(), CoordError> {
        if self.kind != PreservationKindV1::FreshPreservationSubjectV1 || self.schema_version != 1 {
            return Err(invalid(
                "preservation subject kind or schema is unsupported",
            ));
        }
        self.validate_inventories()?;
        if self.preservation_id != self.expected_id()? {
            return Err(invalid(
                "preservation ID differs from its complete inventory pair",
            ));
        }
        canonical_bound(self, "two-location preservation subject")
    }

    fn validate_inventories(&self) -> Result<(), CoordError> {
        validate_absolute_path_hex(&self.family_root_path_hex, "preservation family root")?;
        self.outer_inventory.validate()?;
        self.hub_inventory.validate()?;
        let outer = &self.outer_inventory.subject.source_directory;
        let hub = &self.hub_inventory.subject.source_directory;
        if (outer.device, outer.inode) == (hub.device, hub.inode) {
            return Err(invalid(
                "preservation locations share one original directory identity",
            ));
        }
        let mut locations = Vec::with_capacity(4);
        for (inventory, parent) in [
            (&self.outer_inventory, OUTER_PARENT),
            (&self.hub_inventory, HUB_PARENT),
        ] {
            let parent = append_hex(&self.family_root_path_hex, parent);
            let original = append_hex(&parent, b"/coord");
            if inventory.subject.source_directory.absolute_path_hex != original {
                return Err(invalid(
                    "preservation inventory is not its fixed family location",
                ));
            }
            let destination = format!("{parent}2f{}", inventory.subject.destination_name_hex);
            validate_absolute_path_hex(&destination, "preservation destination")?;
            locations.extend([original, destination]);
        }
        for (index, left) in locations.iter().enumerate() {
            if locations[index + 1..].iter().any(|right| {
                left == right
                    || left.starts_with(&format!("{right}2f"))
                    || right.starts_with(&format!("{left}2f"))
            }) {
                return Err(invalid(
                    "preservation original and destination locations overlap",
                ));
            }
        }
        Ok(())
    }

    fn expected_id(&self) -> Result<String, CoordError> {
        domain_id(
            "fgp_",
            SUBJECT_DOMAIN,
            &PreservationIdentity {
                family_root_path_hex: &self.family_root_path_hex,
                outer_inventory: &self.outer_inventory,
                hub_inventory: &self.hub_inventory,
            },
        )
    }
}

fn append_hex(prefix: &str, suffix: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut result = prefix.to_owned();
    for byte in suffix {
        write!(&mut result, "{byte:02x}").expect("formatting bytes into String cannot fail");
    }
    result
}

#[cfg(test)]
mod tests;
