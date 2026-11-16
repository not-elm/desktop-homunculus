//! The migration type, its context, and the runner that executes migrations
//! in order.

use bevy::log::error;
use homunculus_utils::runtime::RuntimeResolver;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// A startup migration that brings one part of an older user environment up
/// to date.
///
/// Every migration runs on every startup, so `run` must be idempotent: it
/// detects whether the environment still needs the change and otherwise does
/// nothing. It must also leave the `sandbox/` mods directory that `make debug`
/// uses unchanged.
pub(crate) struct Migration {
    /// The name that identifies the migration in failure logs.
    pub(crate) name: &'static str,
    /// Applies the migration, or returns `Ok(())` without changes when the
    /// environment does not need it.
    pub(crate) run: fn(&MigrationContext) -> Result<(), MigrationError>,
}

/// The environment a migration inspects and changes.
pub(crate) struct MigrationContext {
    /// The MOD installation directory.
    pub(crate) mods_dir: PathBuf,
    /// Resolves the Node.js and pnpm executables to run.
    pub(crate) runtime: RuntimeResolver,
}

/// A failure of one migration.
#[derive(Debug, Error)]
pub(crate) enum MigrationError {
    /// pnpm could not be started.
    #[error("failed to spawn pnpm: {0}")]
    Spawn(#[from] io::Error),
    /// pnpm ran and exited unsuccessfully.
    #[error("pnpm exited with code {code:?}: {stderr}")]
    Pnpm {
        /// The exit code, or `None` when pnpm was terminated by a signal.
        code: Option<i32>,
        /// What pnpm wrote to its standard error.
        stderr: String,
    },
}

impl MigrationContext {
    /// Returns a context for the given mods directory and runtime.
    pub(crate) fn new(mods_dir: PathBuf, runtime: RuntimeResolver) -> Self {
        Self { mods_dir, runtime }
    }
}

/// Runs every migration in order and returns the names of those that failed.
///
/// Each failure is logged with `error!` and does not stop the migrations
/// after it.
pub(crate) fn run_all(migrations: &[Migration], context: &MigrationContext) -> Vec<&'static str> {
    let mut failed = Vec::new();
    for migration in migrations {
        if let Err(error) = (migration.run)(context) {
            error!("Migration `{}` failed: {error}", migration.name);
            failed.push(migration.name);
        }
    }
    failed
}

#[cfg(test)]
pub(crate) mod test_support {
    use homunculus_utils::runtime::RuntimeResolver;
    use std::path::{Path, PathBuf};

    pub(crate) fn fallback_runtime() -> RuntimeResolver {
        RuntimeResolver::new(Path::new("/nonexistent/bin/app"))
    }

    pub(crate) fn scratch_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("hmcs-migration-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir should be creatable");
        dir
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::fallback_runtime;
    use super::*;
    use std::path::PathBuf;

    fn context() -> MigrationContext {
        MigrationContext::new(PathBuf::from("/nonexistent/hmcs-mods"), fallback_runtime())
    }

    fn always_fails(_: &MigrationContext) -> Result<(), MigrationError> {
        Err(MigrationError::Pnpm {
            code: Some(1),
            stderr: "boom".to_string(),
        })
    }

    fn always_succeeds(_: &MigrationContext) -> Result<(), MigrationError> {
        Ok(())
    }

    /// Asserts that a failed migration is reported and skipped rather than
    /// stopping the migrations after it.
    ///
    /// Case: pnpm cannot be started on the user's machine, so the first
    /// migration fails at startup while a later one still needs to run.
    #[test]
    fn run_all_continues_after_a_failed_migration() {
        let migrations = [
            Migration {
                name: "first",
                run: always_fails,
            },
            Migration {
                name: "second",
                run: always_fails,
            },
        ];
        assert_eq!(run_all(&migrations, &context()), vec!["first", "second"]);
    }

    /// Asserts that migrations that succeed are not reported as failures.
    ///
    /// Case: the user starts an environment that is already up to date.
    #[test]
    fn run_all_reports_nothing_when_every_migration_succeeds() {
        let migrations = [Migration {
            name: "only",
            run: always_succeeds,
        }];
        assert!(run_all(&migrations, &context()).is_empty());
    }
}
