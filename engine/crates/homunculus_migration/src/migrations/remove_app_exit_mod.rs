//! Uninstalls the legacy `@hmcs/app-exit` MOD, whose Exit item is now built
//! into the engine's tray menu.

use crate::MigrationStartup;
use bevy::prelude::*;
use homunculus_utils::config::HomunculusConfig;
use homunculus_utils::process::CommandNoWindow;
use homunculus_utils::runtime::RuntimeResolver;
use serde_json::Value;
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

const PACKAGE_NAME: &str = "@hmcs/app-exit";

/// A failure to remove the package.
#[derive(Debug, Error)]
enum RemovalError {
    /// pnpm could not be started.
    #[error("failed to spawn pnpm: {0}")]
    Spawn(#[from] io::Error),
    /// pnpm ran and exited unsuccessfully.
    #[error("pnpm exited with code {code:?}: {output}")]
    Pnpm {
        /// The exit code, or `None` when pnpm was terminated by a signal.
        code: Option<i32>,
        /// What pnpm reported about the failure: its standard error, or its
        /// standard output when standard error is empty.
        output: String,
    },
}

/// Plugin that removes `@hmcs/app-exit` from the mods directory at startup
/// when the directory's `package.json` still declares it.
pub(crate) struct RemoveAppExitModPlugin;

impl Plugin for RemoveAppExitModPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(MigrationStartup, remove_app_exit_mod);
    }
}

impl RemovalError {
    /// Returns the error for a pnpm run that exited unsuccessfully with the
    /// given exit code and captured output.
    ///
    /// The reported output is standard error, or standard output when
    /// standard error is empty, with surrounding whitespace trimmed.
    fn pnpm_failure(code: Option<i32>, stdout: &[u8], stderr: &[u8]) -> Self {
        let stderr = String::from_utf8_lossy(stderr).trim().to_string();
        let output = if stderr.is_empty() {
            String::from_utf8_lossy(stdout).trim().to_string()
        } else {
            stderr
        };
        Self::Pnpm { code, output }
    }
}

fn remove_app_exit_mod(config: Res<HomunculusConfig>, runtime: Res<RuntimeResolver>) {
    if let Err(error) = remove(&config.mods_dir, &runtime) {
        error!("Failed to remove the legacy {PACKAGE_NAME} MOD: {error}");
    }
}

fn remove(mods_dir: &Path, runtime: &RuntimeResolver) -> Result<(), RemovalError> {
    let Some(mut command) = removal_command(mods_dir, runtime) else {
        return Ok(());
    };
    let output = command.output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(RemovalError::pnpm_failure(
            output.status.code(),
            &output.stdout,
            &output.stderr,
        ))
    }
}

fn removal_command(mods_dir: &Path, runtime: &RuntimeResolver) -> Option<Command> {
    if !is_declared(mods_dir) {
        return None;
    }
    let mut command = runtime.pnpm_command();
    command
        .arg("-C")
        .arg(mods_dir)
        .args(["remove", "--config.offline=true", PACKAGE_NAME])
        .no_window();
    Some(command)
}

fn is_declared(mods_dir: &Path) -> bool {
    let Ok(contents) = fs::read_to_string(mods_dir.join("package.json")) else {
        return false;
    };
    let Ok(manifest) = serde_json::from_str::<Value>(&contents) else {
        return false;
    };
    manifest
        .get("dependencies")
        .and_then(Value::as_object)
        .is_some_and(|dependencies| dependencies.contains_key(PACKAGE_NAME))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{fallback_runtime, scratch_dir};
    use std::ffi::OsStr;
    use std::path::PathBuf;

    fn mods_dir_with(name: &str, package_json: &str) -> PathBuf {
        let dir = scratch_dir(name);
        std::fs::write(dir.join("package.json"), package_json)
            .expect("package.json should be writable");
        dir
    }

    /// Asserts that a mods directory listing the package in `dependencies`
    /// needs the removal.
    ///
    /// Case: the user upgrades from a version that installed the Exit MOD.
    #[test]
    fn declared_dependency_needs_removal() {
        let dir = mods_dir_with(
            "declared",
            r#"{"dependencies":{"@hmcs/app-exit":"0.1.0","@hmcs/menu":"0.1.0"}}"#,
        );
        assert!(is_declared(&dir));
    }

    /// Asserts that a mods directory without the package in `dependencies`
    /// needs no removal.
    ///
    /// Case: the user already removed the Exit MOD, or never installed it.
    #[test]
    fn undeclared_dependency_needs_no_removal() {
        let dir = mods_dir_with("undeclared", r#"{"dependencies":{"@hmcs/menu":"0.1.0"}}"#);
        assert!(!is_declared(&dir));
    }

    /// Asserts that a leftover `node_modules/@hmcs/app-exit` directory alone
    /// does not trigger the removal.
    ///
    /// Case: an earlier removal left the package directory behind after the
    /// dependency was dropped from `package.json`.
    #[test]
    fn orphaned_node_modules_directory_needs_no_removal() {
        let dir = mods_dir_with("orphaned", r#"{"dependencies":{}}"#);
        let package_dir = dir.join("node_modules/@hmcs/app-exit");
        std::fs::create_dir_all(&package_dir).expect("package dir should be creatable");
        std::fs::write(
            package_dir.join("package.json"),
            r#"{"name":"@hmcs/app-exit"}"#,
        )
        .expect("package manifest should be writable");
        assert!(!is_declared(&dir));
    }

    /// Asserts that a missing mods directory needs no removal.
    ///
    /// Case: a fresh install starts before MOD discovery has created the mods
    /// directory.
    #[test]
    fn missing_mods_dir_needs_no_removal() {
        assert!(!is_declared(
            &scratch_dir("missing-dir").join("does-not-exist")
        ));
    }

    /// Asserts that a mods directory without `package.json` needs no removal.
    ///
    /// Case: the mods directory exists, but nothing has been installed into it
    /// yet.
    #[test]
    fn missing_package_json_needs_no_removal() {
        assert!(!is_declared(&scratch_dir("missing-package-json")));
    }

    /// Asserts that an unreadable `package.json` needs no removal.
    ///
    /// Case: the user's mods `package.json` was truncated by a crash.
    #[test]
    fn malformed_package_json_needs_no_removal() {
        assert!(!is_declared(&mods_dir_with("malformed", "{ not json")));
    }

    /// Asserts that a `package.json` whose `dependencies` is absent or not an
    /// object needs no removal.
    ///
    /// Case: the mods directory was initialized but no MOD was ever added.
    #[test]
    fn package_json_without_dependency_map_needs_no_removal() {
        assert!(!is_declared(&mods_dir_with("no-dependencies", "{}")));
        assert!(!is_declared(&mods_dir_with(
            "null-dependencies",
            r#"{"dependencies":null}"#
        )));
    }

    /// Asserts that the removal command removes only the package, offline,
    /// in the mods directory.
    ///
    /// Case: the migration runs on a machine without network access.
    #[test]
    fn removal_command_runs_offline_in_mods_dir() {
        let dir = mods_dir_with("command", r#"{"dependencies":{"@hmcs/app-exit":"0.1.0"}}"#);
        let command = removal_command(&dir, &fallback_runtime()).expect("the package is declared");
        let args: Vec<&OsStr> = command.get_args().collect();
        assert_eq!(
            args,
            [
                OsStr::new("-C"),
                dir.as_os_str(),
                OsStr::new("remove"),
                OsStr::new("--config.offline=true"),
                OsStr::new("@hmcs/app-exit"),
            ]
        );
    }

    /// Asserts that no removal command is built, so pnpm never starts, when
    /// the package is not declared.
    ///
    /// Case: the user starts an environment that never had the Exit MOD.
    #[test]
    fn removal_command_is_none_when_not_declared() {
        let dir = mods_dir_with("not-declared", r#"{"dependencies":{"@hmcs/menu":"0.1.0"}}"#);
        assert!(removal_command(&dir, &fallback_runtime()).is_none());
    }

    /// Asserts that a pnpm failure reports standard output when standard
    /// error is empty, with surrounding whitespace trimmed.
    ///
    /// Case: the bundled pnpm 10 cannot remove a package offline and prints
    /// `ERR_PNPM_NO_OFFLINE_TARBALL` on standard output only.
    #[test]
    fn pnpm_failure_reports_stdout_when_stderr_is_empty() {
        let error = RemovalError::pnpm_failure(
            Some(1),
            b" ERR_PNPM_NO_OFFLINE_TARBALL  A package is missing from the store\n",
            b"",
        );
        assert_eq!(
            error.to_string(),
            "pnpm exited with code Some(1): ERR_PNPM_NO_OFFLINE_TARBALL  A package is missing from the store"
        );
    }

    /// Asserts that a pnpm failure reports standard error rather than
    /// standard output when standard error is not empty.
    ///
    /// Case: pnpm prints progress on standard output and its error on
    /// standard error.
    #[test]
    fn pnpm_failure_prefers_stderr() {
        let error = RemovalError::pnpm_failure(
            Some(1),
            b"Progress: resolved 1\n",
            b"ERR_PNPM_NO_OFFLINE_META\n",
        );
        assert_eq!(
            error.to_string(),
            "pnpm exited with code Some(1): ERR_PNPM_NO_OFFLINE_META"
        );
    }
}
