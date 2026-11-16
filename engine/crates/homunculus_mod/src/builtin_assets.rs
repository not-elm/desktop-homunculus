//! Assets shipped with the engine and registered under fixed asset IDs.

use bevy::asset::io::file::FileAssetReader;
use bevy::prelude::*;
use homunculus_core::prelude::{AssetDeclaration, AssetEntry, AssetId, AssetOrigin, AssetRegistry};
use std::collections::HashMap;
use std::path::PathBuf;

const BUILTIN_MANIFEST: &str = include_str!("../../../assets/assets.json");

/// The engine's built-in asset directory, as an absolute path.
#[derive(Resource, Debug, Clone)]
pub struct BuiltinAssetsDir(pub PathBuf);

impl BuiltinAssetsDir {
    /// Resolves the engine's asset directory for the running executable.
    ///
    /// The directory is resolved against Bevy's asset base directory
    /// (`BEVY_ASSET_ROOT`, then `CARGO_MANIFEST_DIR`, then the executable's
    /// directory): `assets` in development builds, `../Resources/assets` inside
    /// a macOS app bundle, and the `assets` directory next to the executable on
    /// Windows.
    pub fn resolve() -> Self {
        Self(FileAssetReader::get_base_path().join(relative_asset_dir()))
    }

    /// Returns the directory in the form `AssetPlugin::file_path` expects.
    pub fn as_file_path(&self) -> String {
        self.0.to_string_lossy().into_owned()
    }

    /// Registers every asset declared in the embedded manifest as a built-in
    /// asset.
    ///
    /// A declared asset is registered even when its file is missing, so its ID
    /// stays reserved; the missing file is logged as an error.
    pub fn register_assets(&self, registry: &mut AssetRegistry) {
        self.register_from_manifest(BUILTIN_MANIFEST, registry);
    }

    fn register_from_manifest(&self, manifest_json: &str, registry: &mut AssetRegistry) {
        let declarations: HashMap<String, AssetDeclaration> =
            match serde_json::from_str(manifest_json) {
                Ok(declarations) => declarations,
                Err(e) => {
                    error!("Failed to parse the built-in asset manifest: {e}");
                    return;
                }
            };
        for (id, declaration) in &declarations {
            let entry = AssetEntry::from_declaration(
                AssetId::new(id.clone()),
                declaration,
                &self.0,
                AssetOrigin::Builtin,
            );
            report_missing_file(&entry);
            registry.register_builtin(entry);
        }
    }
}

pub(crate) fn register_builtin_assets(
    dir: Res<BuiltinAssetsDir>,
    mut registry: ResMut<AssetRegistry>,
) {
    dir.register_assets(&mut registry);
}

fn relative_asset_dir() -> PathBuf {
    if std::env::var("CARGO_MANIFEST_DIR").is_ok() {
        return PathBuf::from("assets");
    }
    if cfg!(target_os = "macos") {
        return PathBuf::from("../Resources/assets");
    }
    if cfg!(target_os = "windows")
        && let Some(dir) = exe_adjacent_assets_dir()
    {
        return dir;
    }
    PathBuf::from("assets")
}

fn exe_adjacent_assets_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?.join("assets");
    dir.exists().then_some(dir)
}

fn report_missing_file(entry: &AssetEntry) {
    if !entry.absolute_path.is_file() {
        error!(
            "Built-in asset file is missing: {} ({})",
            entry.absolute_path.display(),
            entry.id
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use homunculus_core::prelude::AssetType;
    use std::path::Path;

    fn engine_assets_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets")
    }

    fn embedded_manifest() -> HashMap<String, AssetDeclaration> {
        serde_json::from_str(BUILTIN_MANIFEST).expect("the embedded manifest parses")
    }

    fn relative_files_under(root: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let mut pending = vec![root.to_path_buf()];
        while let Some(dir) = pending.pop() {
            for entry in std::fs::read_dir(&dir).expect("the directory is readable") {
                let path = entry.expect("the directory entry is readable").path();
                if is_hidden(&path) {
                    continue;
                }
                if path.is_dir() {
                    pending.push(path);
                } else {
                    let relative = path.strip_prefix(root).expect("the file is under root");
                    files.push(relative.to_path_buf());
                }
            }
        }
        files
    }

    fn is_hidden(path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with('.'))
    }

    fn empty_temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "homunculus-builtin-assets-{name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("the temp directory is created");
        dir
    }

    fn mod_entry(id: &str) -> AssetEntry {
        AssetEntry {
            id: AssetId::new(id),
            path: PathBuf::from("vrm/Elmer.vrm"),
            absolute_path: PathBuf::from("/mods/node_modules/@hmcs/assets/vrm/Elmer.vrm"),
            asset_type: AssetType::Vrm,
            description: None,
            origin: AssetOrigin::Mod("@hmcs/assets".to_string()),
        }
    }

    /// Asserts that every entry of the embedded manifest points at a file in
    /// the engine's asset directory and carries a non-empty description.
    ///
    /// Case: an LLM agent lists assets through MCP and picks an animation by
    /// its description.
    #[test]
    fn embedded_manifest_entries_exist_and_are_described() {
        let dir = engine_assets_dir();

        for (id, declaration) in embedded_manifest() {
            assert!(
                dir.join(&declaration.path).is_file(),
                "{id}: {} is missing",
                declaration.path
            );
            assert!(
                declaration
                    .description
                    .as_deref()
                    .is_some_and(|text| !text.trim().is_empty()),
                "{id} has no description"
            );
        }
    }

    /// Asserts that every file under the engine's asset directory is either
    /// declared in the manifest or allowlisted.
    ///
    /// Case: a developer adds a file to the engine's asset directory and
    /// forgets to declare it in the manifest.
    #[test]
    fn every_file_under_assets_is_declared_or_allowlisted() {
        let declared: Vec<PathBuf> = embedded_manifest()
            .into_values()
            .map(|declaration| PathBuf::from(declaration.path))
            .collect();

        for file in relative_files_under(&engine_assets_dir()) {
            let allowlisted = file.starts_with("icons") || file == Path::new("assets.json");
            assert!(
                allowlisted || declared.contains(&file),
                "{} is neither declared nor allowlisted",
                file.display()
            );
        }
    }

    /// Asserts that the assets moved from the retired assets MOD keep their
    /// IDs, paths, and types.
    ///
    /// Case: an existing user's persona refers to `vrm:elmer`, and its saved
    /// position is keyed by that ID.
    #[test]
    fn migrated_asset_ids_paths_and_types_are_preserved() {
        let expected = HashMap::from([
            ("vrm:elmer", ("vrm/Elmer.vrm", AssetType::Vrm)),
            ("vrma:idle-maid", ("vrma/idle-maid.vrma", AssetType::Vrma)),
            ("vrma:grabbed", ("vrma/grabbed.vrma", AssetType::Vrma)),
            (
                "vrma:idle-sitting",
                ("vrma/idle-sitting.vrma", AssetType::Vrma),
            ),
            ("se:open", ("se/open.mp3", AssetType::Sound)),
            ("se:close", ("se/close.mp3", AssetType::Sound)),
        ]);

        let actual: HashMap<String, (String, AssetType)> = embedded_manifest()
            .into_iter()
            .map(|(id, declaration)| (id, (declaration.path, declaration.asset_type)))
            .collect();

        assert_eq!(actual.len(), expected.len());
        for (id, (path, asset_type)) in expected {
            assert_eq!(
                actual.get(id),
                Some(&(path.to_string(), asset_type)),
                "{id} changed"
            );
        }
    }

    /// Asserts that a declared built-in ID is reserved even when its file is
    /// missing, so a MOD cannot take it over.
    ///
    /// Case: the built-in files are missing from an installation, and the
    /// retired assets MOD that declares the same IDs is still installed.
    #[test]
    fn missing_builtin_file_still_reserves_the_id() {
        let mut registry = AssetRegistry::default();
        let dir = empty_temp_dir("missing-files");

        BuiltinAssetsDir(dir.clone()).register_assets(&mut registry);
        registry.register(mod_entry("vrm:elmer"));

        for id in embedded_manifest().keys() {
            assert!(registry.is_builtin(id), "{id} is not reserved");
        }
        std::fs::remove_dir_all(dir).ok();
    }

    /// Asserts that a manifest that fails to parse registers nothing instead
    /// of panicking.
    ///
    /// Case: a broken manifest reaches a build, and the engine still starts.
    #[test]
    fn invalid_manifest_registers_nothing() {
        let mut registry = AssetRegistry::default();

        BuiltinAssetsDir(engine_assets_dir()).register_from_manifest("{", &mut registry);

        assert_eq!(registry.all().count(), 0);
    }
}
