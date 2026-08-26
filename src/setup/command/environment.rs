use std::{ffi::OsString, path::PathBuf, process::Command};

use super::Toolchain;
use crate::{
    coord::CoordError,
    setup::transaction::{AdmittedRoot, Staging},
};

const HOME_COMPONENT: &str = "home";

#[derive(Debug)]
pub(in crate::setup) struct SetupEnvironment {
    staging: Option<Staging>,
    home: PathBuf,
    cargo_home: PathBuf,
    npm_cache: PathBuf,
    temporary: PathBuf,
    trusted_path: OsString,
}

impl SetupEnvironment {
    pub(in crate::setup) fn create(
        family_root: &AdmittedRoot,
        toolchain: &Toolchain,
    ) -> Result<Self, CoordError> {
        let staging = family_root.create_staging(HOME_COMPONENT)?;
        let home = staging.path().join("home");
        let cargo_home = staging.path().join("cargo");
        let npm_cache = staging.path().join("npm-cache");
        let temporary = staging.path().join("tmp");
        for name in ["home", "cargo", "npm-cache", "tmp"] {
            staging.create_private_dir(name)?;
        }
        Ok(Self {
            staging: Some(staging),
            home,
            cargo_home,
            npm_cache,
            temporary,
            trusted_path: toolchain.trusted_path().to_owned(),
        })
    }

    pub(super) fn verify(&self) -> Result<(), CoordError> {
        self.staging
            .as_ref()
            .ok_or_else(|| {
                CoordError::new(
                    "SETUP_STAGING_CLOSED",
                    "setup environment is already closed",
                )
            })?
            .ensure_path_identity()
    }

    pub(in crate::setup) fn finish(mut self) -> Result<(), CoordError> {
        self.staging
            .take()
            .expect("setup environment staging exists")
            .finish()
    }

    pub(super) fn apply(&self, command: &mut Command) {
        command
            .env_clear()
            .env("HOME", &self.home)
            .env("CARGO_HOME", &self.cargo_home)
            .env("npm_config_cache", &self.npm_cache)
            .env("TMPDIR", &self.temporary)
            .env("PATH", &self.trusted_path)
            .env("LC_ALL", "C")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_NO_REPLACE_OBJECTS", "1");
    }

    #[cfg(test)]
    pub(super) fn home_path(&self) -> &std::path::Path {
        &self.home
    }
}
