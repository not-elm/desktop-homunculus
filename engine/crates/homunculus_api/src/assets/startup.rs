//! Restores imported assets from the database at engine startup.
//!
//! Reads all rows from the `imported_assets` table and registers them in
//! the [`AssetRegistry`] so that dynamically imported assets survive
//! engine restarts.

use bevy::prelude::*;
use homunculus_core::prelude::{AssetEntry, AssetId, AssetRegistry, AssetType};
use homunculus_prefs::PrefsDatabase;
use homunculus_utils::path::homunculus_dir;
use std::path::PathBuf;

/// Startup system that loads all previously imported assets from the
/// database and registers them in the `AssetRegistry`.
pub fn restore_imported_assets(mut registry: ResMut<AssetRegistry>, prefs: NonSend<PrefsDatabase>) {
    let assets = match prefs.list_imported_assets() {
        Ok(a) => a,
        Err(e) => {
            warn!("Failed to load imported assets from DB: {e}");
            return;
        }
    };

    for asset in assets {
        let Some(asset_type) = parse_asset_type(&asset.asset_type) else {
            warn!(
                "Skipping imported asset '{}': unknown type '{}'",
                asset.id, asset.asset_type
            );
            continue;
        };

        let stored_path = PathBuf::from(&asset.path);
        let filename = stored_path
            .file_name()
            .map(PathBuf::from)
            .unwrap_or_else(|| stored_path.clone());
        let entry = AssetEntry::imported(
            AssetId::new(&asset.id),
            filename,
            &homunculus_dir().join("assets"),
            asset_type,
            asset.description,
        );
        if !registry.register_imported(entry) {
            warn!(
                "Skipping imported asset '{}': the ID is reserved by a built-in asset",
                asset.id
            );
        }
    }

    info!(
        "Restored {} imported asset(s) from database",
        prefs.list_imported_assets().map(|a| a.len()).unwrap_or(0)
    );
}

/// Parses a lowercase string into an [`AssetType`].
fn parse_asset_type(s: &str) -> Option<AssetType> {
    serde_json::from_value(serde_json::Value::String(s.to_string())).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use homunculus_core::prelude::AssetOrigin;

    #[test]
    fn parse_known_types() {
        assert_eq!(parse_asset_type("vrm"), Some(AssetType::Vrm));
        assert_eq!(parse_asset_type("vrma"), Some(AssetType::Vrma));
        assert_eq!(parse_asset_type("sound"), Some(AssetType::Sound));
        assert_eq!(parse_asset_type("image"), Some(AssetType::Image));
        assert_eq!(parse_asset_type("html"), Some(AssetType::Html));
    }

    #[test]
    fn parse_unknown_type() {
        assert_eq!(parse_asset_type("unknown"), None);
    }

    #[test]
    fn restore_registers_imported_assets() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<AssetRegistry>()
            .insert_non_send(PrefsDatabase::open_in_memory())
            .add_systems(Startup, restore_imported_assets);

        // Insert an imported asset into the database before startup
        {
            let prefs = app.world().non_send::<PrefsDatabase>();
            prefs
                .upsert_imported_asset(
                    "vrm:local:test",
                    None,
                    "/home/test/.homunculus/assets/test.vrm",
                    "vrm",
                    Some("Test model"),
                    Some("/original/test.vrm"),
                )
                .unwrap();
        }

        app.update();

        let registry = app.world().resource::<AssetRegistry>();
        let entry = registry.get("vrm:local:test");
        assert!(entry.is_some(), "Imported asset should be in the registry");
        let entry = entry.unwrap();
        assert_eq!(entry.asset_type, AssetType::Vrm);
        assert_eq!(entry.origin, AssetOrigin::Local);
        assert_eq!(entry.description.as_deref(), Some("Test model"));
    }

    #[test]
    fn restore_skips_unknown_types() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<AssetRegistry>()
            .insert_non_send(PrefsDatabase::open_in_memory())
            .add_systems(Startup, restore_imported_assets);

        {
            let prefs = app.world().non_send::<PrefsDatabase>();
            prefs
                .upsert_imported_asset("bad:type", None, "/some/path", "unknown", None, None)
                .unwrap();
        }

        app.update();

        let registry = app.world().resource::<AssetRegistry>();
        assert!(
            registry.get("bad:type").is_none(),
            "Asset with unknown type should not be registered"
        );
    }

    /// Asserts that restoring imported assets skips a row whose ID belongs to
    /// a built-in asset, leaving the built-in asset and the database row in
    /// place.
    ///
    /// Case: before upgrading, a user imported their own model as `vrm:elmer`,
    /// which is now a built-in asset ID.
    #[test]
    fn restore_skips_rows_that_collide_with_builtin_ids() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<AssetRegistry>()
            .insert_non_send(PrefsDatabase::open_in_memory())
            .add_systems(Startup, restore_imported_assets);
        app.world_mut()
            .resource_mut::<AssetRegistry>()
            .register_builtin(AssetEntry {
                id: AssetId::new("vrm:elmer"),
                path: PathBuf::from("vrm/Elmer.vrm"),
                absolute_path: PathBuf::from("/engine/assets/vrm/Elmer.vrm"),
                asset_type: AssetType::Vrm,
                description: None,
                origin: AssetOrigin::Builtin,
            });
        app.world()
            .non_send::<PrefsDatabase>()
            .upsert_imported_asset(
                "vrm:elmer",
                None,
                "/home/test/.homunculus/assets/vrm_elmer.vrm",
                "vrm",
                None,
                Some("/original/my-model.vrm"),
            )
            .unwrap();

        app.update();

        let registry = app.world().resource::<AssetRegistry>();
        assert_eq!(
            registry.get("vrm:elmer").map(|entry| &entry.origin),
            Some(&AssetOrigin::Builtin)
        );
        let rows = app
            .world()
            .non_send::<PrefsDatabase>()
            .list_imported_assets()
            .unwrap();
        assert!(rows.iter().any(|row| row.id == "vrm:elmer"));
    }
}
