use crate::mod_service::ModService;
use bevy::prelude::*;
use homunculus_core::prelude::{
    AssetEntry, AssetId, AssetOrigin, AssetRegistry, HomunculusConfig, ModInfo, ModMenuMetadata,
    ModMenuMetadataList, ModRegistry, create_dir_all_if_need,
};
use homunculus_utils::runtime::RuntimeResolver;
use std::path::Path;

pub(crate) struct ModLoadPlugin;

impl Plugin for ModLoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, discover_mods);
    }
}

fn discover_mods(
    mut commands: Commands,
    mut menus: ResMut<ModMenuMetadataList>,
    mut registry: ResMut<AssetRegistry>,
    mut mod_registry: ResMut<ModRegistry>,
    config: Res<HomunculusConfig>,
    runtime: Res<RuntimeResolver>,
) {
    let mods_root = config.mods_dir.clone();
    info!("Mods root: {}", mods_root.display());
    create_dir_all_if_need(&mods_root);

    let mods = match homunculus_utils::mods::list::list_installation_mods_with_runtime(&runtime) {
        Ok(mods) => mods,
        Err(e) => {
            error!("{e}");
            return;
        }
    };
    for m in loadable_mods(mods) {
        schedule_service(&m, &mut commands, &mods_root);
        load_assets(&m, &mut registry);
        load_menus(&m, &mut menus);
        info!("Loaded mod: [{}]", m.name);
        mod_registry.register(m);
    }
}

fn loadable_mods(mods: Vec<ModInfo>) -> Vec<ModInfo> {
    mods.into_iter()
        .filter(|m| {
            let reserved = AssetOrigin::is_reserved_mod_name(&m.name);
            if reserved {
                warn!(
                    "Skipping mod '{}': the name is reserved for the engine's own assets",
                    m.name
                );
            }
            !reserved
        })
        .collect()
}

fn schedule_service(info: &ModInfo, commands: &mut Commands, mods_dir: &Path) {
    if let Some(service_script_path) = &info.service_script_path {
        if service_script_path.exists() {
            commands.spawn(ModService {
                mod_name: info.name.clone(),
                script_path: service_script_path.clone(),
                mods_dir: mods_dir.to_path_buf(),
            });
        } else {
            warn!(
                "Service script not found: {}",
                service_script_path.display()
            );
        }
    }
}

fn load_assets(info: &ModInfo, registry: &mut ResMut<AssetRegistry>) {
    for (asset_id, decl) in &info.assets {
        registry.register(AssetEntry::from_declaration(
            AssetId::new(asset_id.clone()),
            decl,
            &info.mod_dir,
            AssetOrigin::Mod(info.name.clone()),
        ));
    }
}

fn load_menus(info: &ModInfo, menus: &mut ResMut<ModMenuMetadataList>) {
    menus.extend(info.menus.iter().map(|m| ModMenuMetadata {
        id: m.id.clone(),
        mod_name: info.name.clone(),
        text: m.text.clone(),
        command: m.command.clone(),
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn mod_info(name: &str) -> ModInfo {
        ModInfo {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            license: None,
            service_script_path: None,
            commands: vec![],
            assets: HashMap::new(),
            menus: vec![],
            tray: None,
            mod_dir: PathBuf::from("/mods").join(name),
        }
    }

    /// Asserts that MODs named `local` or `builtin` are excluded from loading
    /// while other MODs are kept.
    ///
    /// Case: a user installs packages named `local` and `builtin`, which would
    /// collide with the labels the engine reports for imported and built-in
    /// assets.
    #[test]
    fn reserved_mod_names_are_not_loaded() {
        let mods = vec![
            mod_info("local"),
            mod_info("builtin"),
            mod_info("@hmcs/persona"),
        ];

        let names: Vec<String> = loadable_mods(mods).into_iter().map(|m| m.name).collect();

        assert_eq!(names, vec!["@hmcs/persona".to_string()]);
    }
}
