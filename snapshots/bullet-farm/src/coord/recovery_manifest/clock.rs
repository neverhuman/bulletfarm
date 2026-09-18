use std::{
    fs::{self, File},
    io::Read,
    os::unix::{ffi::OsStrExt, fs::MetadataExt, io::AsRawFd},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(test)]
use std::cell::Cell;

use serde::{Deserialize, Serialize};

use super::{changed, invalid, trust::ClockObservation};
use crate::coord::{CoordError, model::validate_linux_boot_id, sealed};

const NSFS_MAGIC: u64 = 0x6e73_6673;
const MAX_EXPECTATION_BYTES: u64 = 512;
const EXPECTATION_KIND: &str = "bullet.coord.recovery-clock-expectation.v1";
pub(super) const EXPECTATION_PATH: &str = "/run/bullet/recovery-clock-v1.json";

#[cfg(test)]
const TEST_BOOT_ID: &str = "00000000-0000-4000-8000-000000000001";
#[cfg(test)]
const TEST_NAMESPACE: (u64, u64) = (1, 1);

pub(super) fn observe() -> Result<ClockObservation, CoordError> {
    #[cfg(test)]
    if let Some(value) = TEST_CLOCK.with(Cell::get) {
        return require_expected(value.observation(), &ExpectedClockV1::test_value());
    }
    let expected_before = read_expected_clock()?;
    let boot_before = read_boot_id()?;
    let (namespace, retained_namespace) = open_time_namespace()?;
    let unix_value = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| invalid("system time precedes the Unix epoch"))?
        .as_millis();
    let unix_ms = u64::try_from(unix_value)
        .map_err(|_| invalid("system time exceeds the recovery time domain"))?;
    let boottime = rustix::time::clock_gettime(rustix::time::ClockId::Boottime);
    let seconds = u64::try_from(boottime.tv_sec)
        .map_err(|_| invalid("Linux boot time is outside the recovery time domain"))?;
    let nanoseconds = u64::try_from(boottime.tv_nsec)
        .map_err(|_| invalid("Linux boot time has a negative nanosecond component"))?;
    let boottime_ms = seconds
        .checked_mul(1_000)
        .and_then(|value| value.checked_add(nanoseconds / 1_000_000))
        .ok_or_else(|| invalid("Linux boot time exceeds the recovery time domain"))?;
    let retained_after = validate_time_namespace_descriptor(&retained_namespace)?;
    let boot_after = read_boot_id()?;
    let (namespace_after, _) = open_time_namespace()?;
    if boot_before != boot_after || namespace != retained_after || namespace != namespace_after {
        return Err(changed(
            "Linux boot or time namespace changed across the clock sample",
        ));
    }
    let expected_after = read_expected_clock()?;
    if expected_before != expected_after {
        return Err(policy_disabled(
            "root-custodied recovery clock expectation changed across the clock sample",
        ));
    }
    drop(retained_namespace);
    require_expected(
        ClockObservation {
            unix_ms,
            boottime_ms,
            boot_id: boot_before,
            time_namespace_device: namespace.0,
            time_namespace_inode: namespace.1,
        },
        &expected_before,
    )
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "policy-disabled publisher awaits its no-argument CLI"
    )
)]
pub(super) fn publish_expected() -> Result<(), CoordError> {
    let _ = super::trust::installed_policy()?;
    require_root(rustix::process::geteuid().as_raw())?;
    let expected = observe_expectation_subject()?;
    sealed::write_root_runtime(Path::new(EXPECTATION_PATH), &expected)?;
    if observe_expectation_subject()? != expected || read_expected_clock()? != expected {
        return Err(changed(
            "Linux boot, time namespace, or published clock changed across publication",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ExpectedClockV1 {
    kind: String,
    schema_version: u32,
    boot_id: String,
    time_namespace_device: u64,
    time_namespace_inode: u64,
}

impl ExpectedClockV1 {
    fn validate(&self) -> Result<(), CoordError> {
        if self.kind != EXPECTATION_KIND
            || self.schema_version != 1
            || self.time_namespace_device == 0
            || self.time_namespace_inode == 0
            || validate_linux_boot_id(&self.boot_id).is_err()
        {
            return Err(policy_disabled(
                "root-custodied recovery clock expectation is invalid",
            ));
        }
        Ok(())
    }

    #[cfg(test)]
    fn test_value() -> Self {
        Self {
            kind: EXPECTATION_KIND.to_owned(),
            schema_version: 1,
            boot_id: TEST_BOOT_ID.to_owned(),
            time_namespace_device: TEST_NAMESPACE.0,
            time_namespace_inode: TEST_NAMESPACE.1,
        }
    }
}

fn observe_expectation_subject() -> Result<ExpectedClockV1, CoordError> {
    let boot_before = read_boot_id()?;
    let (namespace_before, retained) = open_time_namespace()?;
    let retained_after = validate_time_namespace_descriptor(&retained)?;
    let boot_after = read_boot_id()?;
    let (namespace_after, _) = open_time_namespace()?;
    expected_from_samples(
        boot_before,
        namespace_before,
        retained_after,
        boot_after,
        namespace_after,
    )
}

fn expected_from_samples(
    boot_before: String,
    namespace_before: (u64, u64),
    retained_after: (u64, u64),
    boot_after: String,
    namespace_after: (u64, u64),
) -> Result<ExpectedClockV1, CoordError> {
    if boot_before != boot_after
        || namespace_before != retained_after
        || namespace_before != namespace_after
    {
        return Err(changed(
            "Linux boot or time namespace changed across expectation observation",
        ));
    }
    let expected = ExpectedClockV1 {
        kind: EXPECTATION_KIND.to_owned(),
        schema_version: 1,
        boot_id: boot_before,
        time_namespace_device: namespace_before.0,
        time_namespace_inode: namespace_before.1,
    };
    expected.validate()?;
    Ok(expected)
}

fn require_root(effective_uid: u32) -> Result<(), CoordError> {
    if effective_uid != 0 {
        return Err(policy_disabled(
            "only the root recovery supervisor may publish the clock expectation",
        ));
    }
    Ok(())
}

fn read_expected_clock() -> Result<ExpectedClockV1, CoordError> {
    let expected: ExpectedClockV1 =
        sealed::read_root_runtime(Path::new(EXPECTATION_PATH), MAX_EXPECTATION_BYTES).map_err(
            |error| {
                policy_disabled(format!(
                    "root-custodied recovery clock expectation is unavailable or untrusted ({})",
                    error.code()
                ))
            },
        )?;
    expected.validate()?;
    Ok(expected)
}

fn require_expected(
    observation: ClockObservation,
    expected: &ExpectedClockV1,
) -> Result<ClockObservation, CoordError> {
    if observation.boot_id != expected.boot_id {
        return Err(CoordError::new(
            "RECOVERY_AUTHORIZATION_BOOT_CHANGED",
            "observed Linux boot differs from the root-custodied recovery expectation",
        ));
    }
    if (
        observation.time_namespace_device,
        observation.time_namespace_inode,
    ) != (
        expected.time_namespace_device,
        expected.time_namespace_inode,
    ) {
        return Err(CoordError::new(
            "RECOVERY_TIME_NAMESPACE_CHANGED",
            "observed Linux time namespace differs from the root-custodied recovery expectation",
        ));
    }
    Ok(observation)
}

fn read_boot_id() -> Result<String, CoordError> {
    let mut file = File::open("/proc/sys/kernel/random/boot_id").map_err(CoordError::io)?;
    let mut bytes = Vec::new();
    (&mut file)
        .take(38)
        .read_to_end(&mut bytes)
        .map_err(CoordError::io)?;
    if bytes.last() != Some(&b'\n') || bytes.len() != 37 {
        return Err(invalid("Linux boot ID has an unexpected wire shape"));
    }
    bytes.pop();
    let value = String::from_utf8(bytes).map_err(|_| invalid("Linux boot ID is not UTF-8"))?;
    validate_linux_boot_id(&value)?;
    Ok(value)
}

pub(super) fn open_time_namespace() -> Result<((u64, u64), File), CoordError> {
    let file = File::open("/proc/self/ns/time").map_err(CoordError::io)?;
    let identity = validate_time_namespace_descriptor(&file)?;
    let link = fs::read_link("/proc/self/ns/time").map_err(CoordError::io)?;
    if !time_namespace_link_matches(link.as_os_str().as_bytes(), identity.1) {
        return Err(invalid(
            "Linux namespace pathname is not the exact retained time namespace type",
        ));
    }
    Ok((identity, file))
}

pub(super) fn validate_time_namespace_descriptor(file: &File) -> Result<(u64, u64), CoordError> {
    let metadata = file.metadata().map_err(CoordError::io)?;
    let identity = (metadata.dev(), metadata.ino());
    let filesystem = rustix::fs::fstatfs(file)
        .map_err(|error| invalid(format!("cannot identify Linux time namespace: {error}")))?;
    if identity.0 == 0 || identity.1 == 0 || filesystem.f_type as u64 != NSFS_MAGIC {
        return Err(invalid(
            "Linux time namespace descriptor identity or namespace type is invalid",
        ));
    }
    let link =
        fs::read_link(format!("/proc/self/fd/{}", file.as_raw_fd())).map_err(CoordError::io)?;
    if !time_namespace_link_matches(link.as_os_str().as_bytes(), identity.1) {
        return Err(invalid(
            "retained Linux namespace descriptor is not the exact time namespace type",
        ));
    }
    Ok(identity)
}

#[cfg(test)]
pub(super) fn time_namespace_link_matches(link: &[u8], inode: u64) -> bool {
    link == format!("time:[{inode}]").as_bytes()
}

#[cfg(not(test))]
fn time_namespace_link_matches(link: &[u8], inode: u64) -> bool {
    link == format!("time:[{inode}]").as_bytes()
}

fn policy_disabled(reason: impl Into<String>) -> CoordError {
    CoordError::new("RECOVERY_POLICY_DISABLED", reason)
}

#[cfg(test)]
#[derive(Clone, Copy)]
struct TestClock {
    unix_ms: u64,
    boottime_ms: u64,
    namespace: (u64, u64),
}

#[cfg(test)]
impl TestClock {
    fn observation(self) -> ClockObservation {
        ClockObservation {
            unix_ms: self.unix_ms,
            boottime_ms: self.boottime_ms,
            boot_id: TEST_BOOT_ID.to_owned(),
            time_namespace_device: self.namespace.0,
            time_namespace_inode: self.namespace.1,
        }
    }
}

#[cfg(test)]
thread_local! {
    static TEST_CLOCK: Cell<Option<TestClock>> = const { Cell::new(None) };
}

#[cfg(test)]
pub(in crate::coord) struct TestClockGuard(Option<TestClock>);

#[cfg(test)]
impl Drop for TestClockGuard {
    fn drop(&mut self) {
        TEST_CLOCK.with(|value| value.set(self.0.take()));
    }
}

#[cfg(test)]
pub(in crate::coord) fn install_test_clock(now_unix_ms: u64) -> TestClockGuard {
    install_test_clock_pair(now_unix_ms, now_unix_ms)
}

#[cfg(test)]
pub(in crate::coord) fn install_test_clock_pair(unix_ms: u64, boottime_ms: u64) -> TestClockGuard {
    let clock = TestClock {
        unix_ms,
        boottime_ms,
        namespace: TEST_NAMESPACE,
    };
    TestClockGuard(TEST_CLOCK.with(|value| value.replace(Some(clock))))
}

#[cfg(test)]
pub(in crate::coord) fn set_test_clock(unix_ms: u64, boottime_ms: u64, namespace: (u64, u64)) {
    TEST_CLOCK.with(|value| {
        value.set(Some(TestClock {
            unix_ms,
            boottime_ms,
            namespace,
        }));
    });
}

#[cfg(test)]
mod publisher_tests {
    use super::*;

    #[test]
    fn publisher_is_policy_disabled_before_runtime_io() {
        let error = publish_expected().unwrap_err();
        assert_eq!(error.code(), "RECOVERY_POLICY_DISABLED");
    }

    #[test]
    fn expectation_requires_root_and_stable_safe_kernel_facts() {
        assert_eq!(
            require_root(1).unwrap_err().code(),
            "RECOVERY_POLICY_DISABLED"
        );
        let boot = TEST_BOOT_ID.to_owned();
        let exact = expected_from_samples(
            boot.clone(),
            TEST_NAMESPACE,
            TEST_NAMESPACE,
            boot.clone(),
            TEST_NAMESPACE,
        )
        .unwrap();
        exact.validate().unwrap();

        for result in [
            expected_from_samples(
                boot.clone(),
                TEST_NAMESPACE,
                (2, 1),
                boot.clone(),
                TEST_NAMESPACE,
            ),
            expected_from_samples(
                boot.clone(),
                TEST_NAMESPACE,
                TEST_NAMESPACE,
                "00000000-0000-4000-8000-000000000002".to_owned(),
                TEST_NAMESPACE,
            ),
            expected_from_samples(
                boot,
                TEST_NAMESPACE,
                TEST_NAMESPACE,
                TEST_BOOT_ID.to_owned(),
                (1, 2),
            ),
        ] {
            assert_eq!(result.unwrap_err().code(), "RECOVERY_INSPECTION_CHANGED");
        }
    }

    #[test]
    fn expectation_shape_refuses_unsafe_values() {
        let mut value = ExpectedClockV1::test_value();
        value.time_namespace_device = 0;
        assert_eq!(
            value.validate().unwrap_err().code(),
            "RECOVERY_POLICY_DISABLED"
        );
        let mut value = ExpectedClockV1::test_value();
        value.boot_id = "not-a-boot-id".to_owned();
        assert_eq!(
            value.validate().unwrap_err().code(),
            "RECOVERY_POLICY_DISABLED"
        );
    }
}
