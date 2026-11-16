//! The startup migrations, in the order they run.

mod remove_app_exit_mod;

use crate::migration::Migration;

/// Every startup migration, in the order they run.
pub(crate) static MIGRATIONS: &[Migration] = &[remove_app_exit_mod::MIGRATION];

#[cfg(test)]
mod tests {
    use super::*;

    /// Asserts that the app-exit removal is part of the startup migrations.
    ///
    /// Case: the engine starts after an upgrade and runs every registered
    /// migration.
    #[test]
    fn startup_migrations_include_app_exit_removal() {
        assert!(
            MIGRATIONS
                .iter()
                .any(|migration| migration.name == "remove-app-exit-mod")
        );
    }
}
