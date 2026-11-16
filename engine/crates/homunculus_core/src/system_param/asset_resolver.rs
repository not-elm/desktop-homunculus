use crate::prelude::{AssetEntry, AssetOrigin, AssetRegistry};
use bevy::asset::{Asset, AssetPath, AssetServer};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

/// Resolves asset IDs to Bevy asset handles.
///
/// Use [`load()`](AssetResolver::load) for VRM/VRMA/Effects (returns a Handle).
/// Use [`resolve()`](AssetResolver::resolve) for Webviews (returns the AssetEntry for path resolution).
///
/// # Example
///
/// ```ignore
/// fn my_system(resolver: AssetResolver) {
///     let handle: Handle<Scene> = resolver.load("elmer:vrm").unwrap();
/// }
/// ```
#[derive(SystemParam)]
pub struct AssetResolver<'w> {
    registry: Res<'w, AssetRegistry>,
    asset_server: Res<'w, AssetServer>,
}

impl AssetResolver<'_> {
    /// Resolve an asset ID and load it through the Bevy AssetServer.
    ///
    /// Returns a typed Handle for the asset. A built-in asset loads from the
    /// engine's asset directory; a MOD or imported asset loads from the
    /// `asset://` source.
    pub fn load<A: Asset>(&self, asset_id: &str) -> Result<Handle<A>, AssetResolveError> {
        Ok(self
            .asset_server
            .load(asset_path_of(self.resolve(asset_id)?)))
    }

    /// Resolve an asset ID to its registry entry without loading.
    ///
    /// Useful for Webview source resolution where Bevy AssetServer isn't used.
    pub fn resolve(&self, asset_id: &str) -> Result<&AssetEntry, AssetResolveError> {
        self.registry
            .get(asset_id)
            .ok_or_else(|| AssetResolveError::NotFound(asset_id.to_string()))
    }
}

/// Error type for asset resolution failures.
#[derive(Debug)]
pub enum AssetResolveError {
    /// The asset ID was not found in the registry.
    NotFound(String),
}

impl std::fmt::Display for AssetResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetResolveError::NotFound(id) => write!(f, "Asset not found: {id}"),
        }
    }
}

impl std::error::Error for AssetResolveError {}

fn asset_path_of(entry: &AssetEntry) -> AssetPath<'static> {
    match &entry.origin {
        AssetOrigin::Builtin => AssetPath::from(entry.path.clone()),
        AssetOrigin::Mod(_) | AssetOrigin::Local => AssetPath::from(format!(
            "asset://{}/{}",
            entry.origin.label(),
            entry.path.display()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::{AssetId, AssetType};
    use bevy::asset::io::AssetSourceId;
    use std::path::{Path, PathBuf};

    fn entry(path: &str, origin: AssetOrigin) -> AssetEntry {
        AssetEntry {
            id: AssetId::new("test:asset"),
            path: PathBuf::from(path),
            absolute_path: PathBuf::from("/abs").join(path),
            asset_type: AssetType::Vrm,
            description: None,
            origin,
        }
    }

    /// Asserts that a built-in asset loads from Bevy's default source by its
    /// path relative to the engine's asset directory.
    ///
    /// Case: the engine spawns the default character from the built-in
    /// `vrm:elmer` asset.
    #[test]
    fn builtin_entries_load_from_the_default_source() {
        let path = asset_path_of(&entry("vrm/Elmer.vrm", AssetOrigin::Builtin));

        assert_eq!(path.source(), &AssetSourceId::Default);
        assert_eq!(path.path(), Path::new("vrm/Elmer.vrm"));
    }

    /// Asserts that a MOD asset loads from the `asset` source under the MOD's
    /// directory.
    ///
    /// Case: a MOD shows a stamp effect with an image it declares.
    #[test]
    fn mod_entries_load_from_the_mod_source() {
        let path = asset_path_of(&entry("img/a.png", AssetOrigin::Mod("my-mod".into())));

        assert_eq!(path.source(), &AssetSourceId::from("asset"));
        assert_eq!(path.path(), Path::new("my-mod/img/a.png"));
    }

    /// Asserts that an imported asset loads from the `asset` source under the
    /// `local` directory.
    ///
    /// Case: a persona uses a VRM file the user imported.
    #[test]
    fn local_entries_load_from_the_local_directory() {
        let path = asset_path_of(&entry("vrm_local_x.vrm", AssetOrigin::Local));

        assert_eq!(path.source(), &AssetSourceId::from("asset"));
        assert_eq!(path.path(), Path::new("local/vrm_local_x.vrm"));
    }
}
