//! # Homunculus Migration
//!
//! Startup migrations that bring a user environment created by an older
//! version of Desktop Homunculus up to date with the current version.

mod migrations;
#[cfg(test)]
mod test_support;

use crate::migrations::RemoveAppExitModPlugin;
use bevy::app::MainScheduleOrder;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;

/// The startup schedule that runs the migrations.
///
/// It runs once, before `PreStartup`, so the migrations finish before MOD
/// discovery reads the mods directory. A migration system added to it runs on
/// every startup, so it must be idempotent: it detects whether the environment
/// still needs the change and otherwise does nothing. It must also leave the
/// `sandbox/` mods directory that `make debug` uses unchanged, and it logs a
/// failure with `error!` instead of panicking.
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MigrationStartup;

/// Plugin that runs the startup migrations before any other startup work.
///
/// Requires the `HomunculusConfig` and `RuntimeResolver` resources. A failed
/// migration is logged and does not stop the app from starting.
pub struct HomunculusMigrationPlugin;

impl Plugin for HomunculusMigrationPlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(MigrationStartup);
        app.world_mut()
            .resource_mut::<MainScheduleOrder>()
            .insert_startup_before(PreStartup, MigrationStartup);
        app.add_plugins(RemoveAppExitModPlugin);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{fallback_runtime, scratch_dir};
    use bevy::ecs::schedule::InternedScheduleLabel;
    use homunculus_utils::config::HomunculusConfig;

    /// Asserts that the migration schedule runs before `PreStartup`.
    ///
    /// Case: the engine starts, and migrations must finish before MOD
    /// discovery reads the mods directory.
    #[test]
    fn migration_startup_runs_before_pre_startup() {
        let mut app = App::new();
        app.add_plugins(HomunculusMigrationPlugin);
        let labels = &app.world().resource::<MainScheduleOrder>().startup_labels;
        let position = |label: InternedScheduleLabel| labels.iter().position(|l| *l == label);
        assert!(
            position(MigrationStartup.intern()).expect("registered")
                < position(PreStartup.intern()).expect("registered")
        );
        assert!(app.get_schedule(MigrationStartup).is_some());
    }

    /// Asserts that startup completes when the mods directory is empty.
    ///
    /// Case: a fresh install starts for the first time with no MODs yet.
    #[test]
    fn plugin_starts_up_with_an_empty_mods_dir() {
        let config = HomunculusConfig {
            mods_dir: scratch_dir("plugin-startup"),
            ..HomunculusConfig::default()
        };
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(config)
            .insert_resource(fallback_runtime())
            .add_plugins(HomunculusMigrationPlugin);
        app.update();
    }

    /// Asserts that the migration plugin adds the app-exit removal.
    ///
    /// Case: the engine starts after an upgrade from a version that shipped
    /// the Exit MOD.
    #[test]
    fn migration_plugin_adds_app_exit_removal() {
        let mut app = App::new();
        app.add_plugins(HomunculusMigrationPlugin);
        assert!(app.is_plugin_added::<RemoveAppExitModPlugin>());
    }
}
