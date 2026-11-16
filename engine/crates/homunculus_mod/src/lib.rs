//! MOD system for Desktop Homunculus: discovers MODs installed as npm packages,
//! runs their services, and registers their assets alongside the engine's
//! built-in assets.

mod builtin_assets;
mod load;
pub mod managed_process;
mod mod_asset_reader;
pub mod mod_service;
pub mod node_process;

pub use crate::builtin_assets::BuiltinAssetsDir;

use crate::builtin_assets::register_builtin_assets;
use crate::load::ModLoadPlugin;
use crate::mod_asset_reader::ModAssetReader;
use crate::mod_service::ModServicePlugin;
use crate::node_process::NodeProcessPlugin;
use bevy::asset::io::{AssetSourceBuilder, AssetSourceId};
use bevy::prelude::*;
use homunculus_core::prelude::{AssetRegistry, HomunculusConfig, ModRegistry};
use homunculus_utils::path::homunculus_dir;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

/// Plugin that discovers installed MODs and registers built-in and MOD assets.
///
/// The app must insert [`HomunculusConfig`] and [`BuiltinAssetsDir`] before
/// adding this plugin.
pub struct HomunculusModPlugin;

impl Plugin for HomunculusModPlugin {
    fn build(&self, app: &mut App) {
        let config = app.world().resource::<HomunculusConfig>();
        let node_modules = config.mods_dir.join("node_modules");
        let dir_map = Arc::new(RwLock::new(HashMap::new()));
        register_local_assets_dir(&dir_map);
        let dir_map_for_reader = dir_map.clone();

        app.register_asset_source(
            AssetSourceId::from("asset"),
            AssetSourceBuilder::new(move || {
                Box::new(ModAssetReader::new(
                    node_modules.clone(),
                    dir_map_for_reader.clone(),
                ))
            }),
        );

        app.init_resource::<AssetRegistry>();
        app.init_resource::<ModRegistry>();
        app.add_plugins((NodeProcessPlugin, ModLoadPlugin, ModServicePlugin));
        app.add_systems(PreStartup, register_builtin_assets);
    }
}

/// Registers `~/.homunculus/assets/` in the dir_map under key `"local"`,
/// so `ModAssetReader` can resolve imported asset paths like `asset://local/filename.vrm`.
fn register_local_assets_dir(dir_map: &Arc<RwLock<HashMap<String, PathBuf>>>) {
    let assets_dir = homunculus_dir().join("assets");
    if let Ok(mut map) = dir_map.write() {
        map.insert("local".to_string(), assets_dir);
    }
}
