use bevy::prelude::*;
use std::collections::HashMap;

// Re-export types that moved to homunculus_utils for backward compatibility.
#[allow(unused_imports)]
pub use homunculus_utils::schema::asset::{
    AssetDeclaration, AssetEntry, AssetId, AssetOrigin, AssetType,
};

/// Component to track the asset ID loaded on an entity.
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetIdComponent(pub AssetId);

/// Registry of every asset that can be referenced by ID: built-in assets, MOD
/// assets, and imported files.
///
/// # Invariants
///
/// A built-in asset's ID is never taken over by a MOD asset or an imported file.
#[derive(Resource, Debug, Default)]
pub struct AssetRegistry {
    entries: HashMap<AssetId, AssetEntry>,
}

impl AssetRegistry {
    /// Registers a built-in asset.
    ///
    /// A built-in asset replaces any MOD or imported entry with the same ID, and
    /// logs a warning when it does.
    pub fn register_builtin(&mut self, entry: AssetEntry) {
        if let Some(existing) = self
            .entries
            .get(&entry.id)
            .filter(|existing| existing.origin != AssetOrigin::Builtin)
        {
            warn!(
                "Asset ID '{}' from '{}' is replaced by the built-in asset '{}'",
                entry.id,
                existing.origin.label(),
                entry.origin.label()
            );
        }
        self.entries.insert(entry.id.clone(), entry);
    }

    /// Registers an asset declared by a MOD.
    ///
    /// The first entry for an ID wins. An entry whose ID is already registered,
    /// including by a built-in asset, is skipped with a warning.
    pub fn register(&mut self, entry: AssetEntry) {
        if let Some(existing) = self.entries.get(&entry.id) {
            warn!(
                "Duplicate asset ID '{}': already registered by '{}', skipping from '{}'",
                entry.id,
                existing.origin.label(),
                entry.origin.label()
            );
            return;
        }
        self.entries.insert(entry.id.clone(), entry);
    }

    /// Registers an imported asset, replacing a MOD or imported entry with the
    /// same ID.
    ///
    /// Returns `false` and leaves the registry unchanged when a built-in asset
    /// owns the ID.
    #[must_use]
    pub fn register_imported(&mut self, entry: AssetEntry) -> bool {
        if self.is_builtin(&entry.id) {
            warn!(
                "Asset ID '{}' is reserved by a built-in asset; skipping the imported entry",
                entry.id
            );
            return false;
        }
        self.entries.insert(entry.id.clone(), entry);
        true
    }

    /// Returns `true` when a built-in asset owns `id`.
    pub fn is_builtin(&self, id: &str) -> bool {
        self.entries
            .get(id)
            .is_some_and(|entry| entry.origin == AssetOrigin::Builtin)
    }

    /// Look up an asset entry by ID.
    pub fn get(&self, id: &str) -> Option<&AssetEntry> {
        self.entries.get(id)
    }

    /// Returns all registered assets.
    pub fn all(&self) -> impl Iterator<Item = &AssetEntry> {
        self.entries.values()
    }

    /// Returns all registered assets filtered by type.
    pub fn by_type(&self, asset_type: &AssetType) -> impl Iterator<Item = &AssetEntry> {
        self.entries
            .values()
            .filter(move |e| &e.asset_type == asset_type)
    }

    /// Returns all registered assets filtered by mod name.
    pub fn by_mod(&self, mod_name: &str) -> impl Iterator<Item = &AssetEntry> {
        self.entries
            .values()
            .filter(move |e| e.origin.label() == mod_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn entry(id: &str, origin: AssetOrigin) -> AssetEntry {
        AssetEntry {
            id: AssetId::new(id),
            path: PathBuf::from("file.vrm"),
            absolute_path: PathBuf::from("/abs/file.vrm"),
            asset_type: AssetType::Vrm,
            description: None,
            origin,
        }
    }

    fn origin_of<'a>(registry: &'a AssetRegistry, id: &str) -> &'a AssetOrigin {
        &registry.get(id).expect("entry is registered").origin
    }

    /// Asserts that a built-in asset replaces a MOD asset already registered
    /// under the same ID.
    ///
    /// Case: an existing installation still has the retired assets MOD, and
    /// it is discovered before the engine registers its built-in assets.
    #[test]
    fn builtin_wins_when_registered_after_mod() {
        let mut registry = AssetRegistry::default();
        registry.register(entry("vrm:elmer", AssetOrigin::Mod("@hmcs/assets".into())));

        registry.register_builtin(entry("vrm:elmer", AssetOrigin::Builtin));

        assert_eq!(origin_of(&registry, "vrm:elmer"), &AssetOrigin::Builtin);
    }

    /// Asserts that a MOD asset is rejected rather than registered when a
    /// built-in asset already owns the ID.
    ///
    /// Case: an existing installation still has the retired assets MOD, and
    /// it is discovered after the engine registers its built-in assets.
    #[test]
    fn mod_is_rejected_when_registered_after_builtin() {
        let mut registry = AssetRegistry::default();
        registry.register_builtin(entry("vrm:elmer", AssetOrigin::Builtin));

        registry.register(entry("vrm:elmer", AssetOrigin::Mod("@hmcs/assets".into())));

        assert_eq!(origin_of(&registry, "vrm:elmer"), &AssetOrigin::Builtin);
    }

    /// Asserts that an imported file cannot replace a built-in asset, and that
    /// the rejection is reported to the caller.
    ///
    /// Case: a user imports their own model under the built-in ID `vrm:elmer`.
    #[test]
    fn imported_entry_cannot_replace_builtin() {
        let mut registry = AssetRegistry::default();
        registry.register_builtin(entry("vrm:elmer", AssetOrigin::Builtin));

        let registered = registry.register_imported(entry("vrm:elmer", AssetOrigin::Local));

        assert!(!registered);
        assert_eq!(origin_of(&registry, "vrm:elmer"), &AssetOrigin::Builtin);
    }

    /// Asserts that an imported file still replaces a MOD asset with the same
    /// ID.
    ///
    /// Case: a user imports a file under an ID that an installed MOD also
    /// declares.
    #[test]
    fn imported_entry_still_overwrites_mod_entry() {
        let mut registry = AssetRegistry::default();
        registry.register(entry("vrm:shared", AssetOrigin::Mod("my-mod".into())));

        let registered = registry.register_imported(entry("vrm:shared", AssetOrigin::Local));

        assert!(registered);
        assert_eq!(origin_of(&registry, "vrm:shared"), &AssetOrigin::Local);
    }

    /// Asserts that only IDs owned by built-in assets are reported as
    /// built-in.
    ///
    /// Case: the import API checks whether a requested ID belongs to the
    /// engine before it copies the user's file.
    #[test]
    fn is_builtin_reports_only_builtin_ids() {
        let mut registry = AssetRegistry::default();
        registry.register_builtin(entry("vrm:elmer", AssetOrigin::Builtin));
        registry.register(entry("vrm:other", AssetOrigin::Mod("my-mod".into())));

        assert!(registry.is_builtin("vrm:elmer"));
        assert!(!registry.is_builtin("vrm:other"));
        assert!(!registry.is_builtin("vrm:missing"));
    }
}
