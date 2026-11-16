//! Uninstalls the legacy `@hmcs/app-exit` MOD, whose Exit item is now built
//! into the engine's tray menu.

use crate::migration::{Migration, MigrationContext, MigrationError};
use homunculus_utils::process::CommandNoWindow;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::Command;

const PACKAGE_NAME: &str = "@hmcs/app-exit";

/// Removes `@hmcs/app-exit` from the mods directory when its `package.json`
/// still declares it.
pub(crate) const MIGRATION: Migration = Migration {
    name: "remove-app-exit-mod",
    run,
};

fn run(context: &MigrationContext) -> Result<(), MigrationError> {
    let Some(mut command) = removal_command(context) else {
        return Ok(());
    };
    let output = command.output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(MigrationError::Pnpm {
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

fn removal_command(context: &MigrationContext) -> Option<Command> {
    if !is_declared(&context.mods_dir) {
        return None;
    }
    let mut command = context.runtime.pnpm_command();
    command
        .arg("-C")
        .arg(&context.mods_dir)
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
    use crate::migration::MigrationContext;
    use crate::migration::test_support::{fallback_runtime, scratch_dir};
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
        let command = removal_command(&MigrationContext::new(dir.clone(), fallback_runtime()))
            .expect("the package is declared");
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
        assert!(removal_command(&MigrationContext::new(dir, fallback_runtime())).is_none());
    }
}
