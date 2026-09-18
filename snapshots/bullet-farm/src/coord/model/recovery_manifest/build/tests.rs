use serde_json::json;

use super::*;

fn digest(byte: char) -> String {
    format!("sha256:{}", byte.to_string().repeat(64))
}

fn packages() -> Vec<CargoPackageArchiveV1> {
    vec![
        ("alpha".to_owned(), "1.0.0".to_owned(), digest('a'), 17),
        (
            "zeta-core".to_owned(),
            "2.1.0-rc.1".to_owned(),
            digest('b'),
            23,
        ),
    ]
}

fn valid_cache() -> CargoOfflineCacheManifestV1 {
    CargoOfflineCacheManifestV1::from_observations(
        digest('c'),
        "index.crates.io-1949cf8c6b5b557f".to_owned(),
        (digest('d'), 101),
        digest('e'),
        packages(),
    )
    .unwrap()
}

fn valid_build() -> RecoveryBootstrapBuildObservationV1 {
    RecoveryBootstrapBuildObservationV1::from_observations(
        digest('1'),
        (digest('2'), 211),
        (digest('3'), digest('4'), 311),
        (digest('5'), digest('6'), digest('7')),
        [(41, digest('8')), (41, digest('8'))],
    )
    .unwrap()
}

fn assert_invalid(result: Result<(), CoordError>) {
    assert_eq!(
        result.unwrap_err().code(),
        "INVALID_RECOVERY_MANIFEST_PRODUCTION"
    );
}

#[test]
fn cache_manifest_is_closed_component_only_and_deterministic() {
    let value = valid_cache();
    value.validate().unwrap();
    assert_eq!(
        value.sealed_sha256().unwrap(),
        value.sealed_sha256().unwrap()
    );

    let encoded = serde_json::to_value(&value).unwrap();
    assert_eq!(encoded["authority"], "COMPONENT_ONLY");
    let mut unknown_field = encoded.clone();
    unknown_field
        .as_object_mut()
        .unwrap()
        .insert("trusted".to_owned(), json!(true));
    assert!(serde_json::from_value::<CargoOfflineCacheManifestV1>(unknown_field).is_err());

    let mut unknown_authority = encoded.clone();
    unknown_authority["authority"] = json!("RECOVERY_AUTHORITY");
    assert!(serde_json::from_value::<CargoOfflineCacheManifestV1>(unknown_authority).is_err());

    let mut wrong_tuple_arity = encoded;
    wrong_tuple_arity["package_archives"][0] = json!(["alpha", "1.0.0", digest('a')]);
    assert!(serde_json::from_value::<CargoOfflineCacheManifestV1>(wrong_tuple_arity).is_err());
}

#[test]
fn cache_manifest_refuses_hostile_identity_and_digest_substitution() {
    let mut value = valid_cache();
    value.kind.push_str(".future");
    assert_invalid(value.validate());

    let mut value = valid_cache();
    value.schema_version = 2;
    assert_invalid(value.validate());

    let mut value = valid_cache();
    value.registry_source = "registry+https://mirror.invalid/index".to_owned();
    assert_invalid(value.validate());

    for cache_id in ["../cache", ".hidden", "cache/child", "cache..other"] {
        let mut value = valid_cache();
        value.registry_cache_id = cache_id.to_owned();
        assert_invalid(value.validate());
    }

    let mut value = valid_cache();
    value.cargo_lock_sha256 = format!("sha256:{}", "A".repeat(64));
    assert_invalid(value.validate());

    let mut value = valid_cache();
    value.package_archives[0].0 = "../alpha".to_owned();
    assert_invalid(value.validate());

    let mut value = valid_cache();
    value.package_archives[0].1 = "1.0.0\nsubstitution".to_owned();
    assert_invalid(value.validate());

    let mut value = valid_cache();
    value.package_archives[0].2 = format!("sha256:{}", "B".repeat(64));
    assert_invalid(value.validate());

    let mut value = valid_cache();
    value.package_archives.swap(0, 1);
    assert_invalid(value.validate());

    let mut value = valid_cache();
    value.package_archives[1].0 = value.package_archives[0].0.clone();
    value.package_archives[1].1 = value.package_archives[0].1.clone();
    assert_invalid(value.validate());
}

#[test]
fn cache_manifest_enforces_exact_counts_and_byte_bounds() {
    let mut value = valid_cache();
    value.archive_byte_length = 0;
    assert_invalid(value.validate());

    let mut value = valid_cache();
    value.archive_byte_length = MAX_CACHE_ARCHIVE_BYTES + 1;
    assert_invalid(value.validate());

    let mut value = valid_cache();
    value.package_archives.clear();
    value.package_count = 0;
    value.aggregate_byte_length = 0;
    assert_invalid(value.validate());

    let mut value = valid_cache();
    value.package_count += 1;
    assert_invalid(value.validate());

    let mut value = valid_cache();
    value.aggregate_byte_length += 1;
    assert_invalid(value.validate());

    let mut value = valid_cache();
    value.package_archives[0].3 = 0;
    assert_invalid(value.validate());

    let mut value = valid_cache();
    value.package_archives[0].3 = MAX_CRATE_ARCHIVE_BYTES + 1;
    assert_invalid(value.validate());

    let maximum_packages = (0..MAX_PACKAGES)
        .map(|index| (format!("p{index:04}"), "1.0.0".to_owned(), digest('a'), 1))
        .collect();
    let maximum = CargoOfflineCacheManifestV1::from_observations(
        digest('c'),
        "index.crates.io-1949cf8c6b5b557f".to_owned(),
        (digest('d'), MAX_CACHE_ARCHIVE_BYTES),
        digest('e'),
        maximum_packages,
    )
    .unwrap();
    maximum.validate().unwrap();

    let mut too_many = maximum;
    too_many
        .package_archives
        .push(("z-extra".to_owned(), "1.0.0".to_owned(), digest('f'), 1));
    too_many.package_count += 1;
    too_many.aggregate_byte_length += 1;
    assert_invalid(too_many.validate());

    let aggregate_maximum = (0..8)
        .map(|index| {
            (
                format!("q{index}"),
                "1.0.0".to_owned(),
                digest('a'),
                MAX_CRATE_ARCHIVE_BYTES,
            )
        })
        .collect();
    let maximum = CargoOfflineCacheManifestV1::from_observations(
        digest('c'),
        "index.crates.io-1949cf8c6b5b557f".to_owned(),
        (digest('d'), MAX_CACHE_ARCHIVE_BYTES),
        digest('e'),
        aggregate_maximum,
    )
    .unwrap();
    assert_eq!(maximum.aggregate_byte_length, MAX_CACHE_AGGREGATE_BYTES);
}

#[test]
fn build_observation_is_closed_component_only_and_subject_exact() {
    let value = valid_build();
    value.validate().unwrap();
    assert!(value.observation_id.starts_with("rbo_"));

    let encoded = serde_json::to_value(&value).unwrap();
    assert_eq!(encoded["authority"], "COMPONENT_ONLY");
    assert_eq!(encoded["comparison"], "EXACT_BYTES_EQUAL");
    let mut unknown_field = encoded.clone();
    unknown_field
        .as_object_mut()
        .unwrap()
        .insert("admitted".to_owned(), json!(true));
    assert!(serde_json::from_value::<RecoveryBootstrapBuildObservationV1>(unknown_field).is_err());

    let mut unknown_authority = encoded;
    unknown_authority["authority"] = json!("TRUSTED");
    assert!(
        serde_json::from_value::<RecoveryBootstrapBuildObservationV1>(unknown_authority).is_err()
    );

    let mut substituted = value.clone();
    substituted.command_contract_sha256 = digest('9');
    assert_invalid(substituted.validate());

    let independently_bound = RecoveryBootstrapBuildObservationV1::from_observations(
        digest('1'),
        (digest('2'), 211),
        (digest('3'), digest('4'), 311),
        (digest('5'), digest('6'), digest('9')),
        [(41, digest('8')), (41, digest('8'))],
    )
    .unwrap();
    assert_ne!(value.observation_id, independently_bound.observation_id);
}

#[test]
fn build_observation_refuses_hostile_modes_bounds_and_digests() {
    let mut value = valid_build();
    value.kind.push_str(".future");
    assert_invalid(value.validate());

    let mut value = valid_build();
    value.observation_id = format!("rbo_{}", "A".repeat(64));
    assert_invalid(value.validate());

    let mut value = valid_build();
    value.source_archive_sha256 = format!("sha256:{}", "A".repeat(64));
    assert_invalid(value.validate());

    let mut value = valid_build();
    value.source_archive_byte_length = 0;
    assert_invalid(value.validate());

    let mut value = valid_build();
    value.source_archive_byte_length = MAX_SOURCE_ARCHIVE_BYTES + 1;
    assert_invalid(value.validate());

    let mut value = valid_build();
    value.cargo_cache_archive_byte_length = MAX_CACHE_ARCHIVE_BYTES + 1;
    assert_invalid(value.validate());

    for (target, profile, network, runs) in [
        ("aarch64-unknown-linux-gnu", BUILD_PROFILE, NETWORK_MODE, 2),
        (TARGET_TRIPLE, "debug", NETWORK_MODE, 2),
        (TARGET_TRIPLE, BUILD_PROFILE, "ONLINE", 2),
        (TARGET_TRIPLE, BUILD_PROFILE, NETWORK_MODE, 1),
    ] {
        let mut value = valid_build();
        value.target_triple = target.to_owned();
        value.build_profile = profile.to_owned();
        value.network_mode = network.to_owned();
        value.run_count = runs;
        assert_invalid(value.validate());
    }

    let mut value = valid_build();
    value.executable_byte_lengths[0] = 0;
    assert_invalid(value.validate());

    let mut value = valid_build();
    value.executable_byte_lengths = [MAX_EXECUTABLE_BYTES + 1; 2];
    assert_invalid(value.validate());

    let mut value = valid_build();
    value.executable_byte_lengths[1] += 1;
    assert_invalid(value.validate());

    let mut value = valid_build();
    value.executable_sha256[1] = digest('9');
    assert_invalid(value.validate());
}
