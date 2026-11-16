use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Type of mod asset.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "lowercase")]
pub enum AssetType {
    Vrm,
    Vrma,
    Sound,
    Image,
    Html,
}

/// A single declared asset entry from a MOD's package.json.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AssetDeclaration {
    /// File path relative to the MOD root directory.
    pub path: String,
    /// Asset type.
    #[serde(rename = "type")]
    pub asset_type: AssetType,
    /// Human-readable description (used by LLM agents for asset selection).
    #[serde(default)]
    pub description: Option<String>,
}

/// Where a registered asset comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetOrigin {
    /// An asset shipped with the engine in its asset directory.
    Builtin,
    /// An asset declared by the named MOD.
    Mod(String),
    /// A file the user imported into `~/.homunculus/assets/`.
    Local,
}

impl AssetOrigin {
    /// The label reported for built-in assets.
    pub const BUILTIN_LABEL: &'static str = "builtin";
    /// The label reported for imported assets.
    pub const LOCAL_LABEL: &'static str = "local";

    /// Returns the label the HTTP API reports in the `mod` field.
    ///
    /// A MOD asset's label is the MOD's package name.
    pub fn label(&self) -> &str {
        match self {
            Self::Builtin => Self::BUILTIN_LABEL,
            Self::Mod(name) => name,
            Self::Local => Self::LOCAL_LABEL,
        }
    }

    /// Returns `true` when `name` is a label no MOD may use as its package name.
    pub fn is_reserved_mod_name(name: &str) -> bool {
        name == Self::BUILTIN_LABEL || name == Self::LOCAL_LABEL
    }
}

/// A resolved asset entry with full path information.
#[derive(Debug, Clone)]
pub struct AssetEntry {
    /// The asset ID (key from package.json homunculus.assets).
    pub id: AssetId,
    /// File path relative to the MOD root directory.
    pub path: PathBuf,
    /// Absolute asset path.
    pub absolute_path: PathBuf,
    /// Asset type.
    pub asset_type: AssetType,
    /// Human-readable description.
    pub description: Option<String>,
    /// Where this asset comes from.
    pub origin: AssetOrigin,
}

impl AssetEntry {
    /// Builds an entry from a declaration whose `path` is relative to `root`.
    pub fn from_declaration(
        id: AssetId,
        declaration: &AssetDeclaration,
        root: &Path,
        origin: AssetOrigin,
    ) -> Self {
        Self {
            id,
            path: PathBuf::from(&declaration.path),
            absolute_path: root.join(&declaration.path),
            asset_type: declaration.asset_type.clone(),
            description: declaration.description.clone(),
            origin,
        }
    }

    /// Builds an entry for a file imported into `assets_dir` as `file_name`.
    pub fn imported(
        id: AssetId,
        file_name: PathBuf,
        assets_dir: &Path,
        asset_type: AssetType,
        description: Option<String>,
    ) -> Self {
        Self {
            id,
            absolute_path: assets_dir.join(&file_name),
            path: file_name,
            asset_type,
            description,
            origin: AssetOrigin::Local,
        }
    }
}

/// A strongly-typed asset identifier.
///
/// Wraps a `String` but provides type safety so arbitrary strings cannot be
/// passed where an asset ID is expected. Serializes transparently as a plain
/// JSON string so the HTTP/JSON API schema is unchanged.
#[repr(transparent)]
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(transparent)]
#[derive(Default)]
pub struct AssetId(String);

impl AssetId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AssetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::ops::Deref for AssetId {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for AssetId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for AssetId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for AssetId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::borrow::Borrow<str> for AssetId {
    fn borrow(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Asserts that each origin's label is the value the HTTP API reports in
    /// the `mod` field.
    ///
    /// Case: a client lists assets and groups them by the `mod` field to tell
    /// built-in assets, imported files, and MOD assets apart.
    #[test]
    fn origin_labels_match_http_mod_values() {
        assert_eq!(AssetOrigin::Builtin.label(), "builtin");
        assert_eq!(AssetOrigin::Local.label(), "local");
        assert_eq!(AssetOrigin::Mod("my-mod".into()).label(), "my-mod");
    }

    /// Asserts that only `builtin` and `local` are reserved as MOD names.
    ///
    /// Case: a user installs MOD packages whose names resemble the labels the
    /// engine uses for its own assets and for imported files.
    #[test]
    fn reserved_mod_names_are_builtin_and_local() {
        assert!(AssetOrigin::is_reserved_mod_name("builtin"));
        assert!(AssetOrigin::is_reserved_mod_name("local"));
        assert!(!AssetOrigin::is_reserved_mod_name("@hmcs/persona"));
        assert!(!AssetOrigin::is_reserved_mod_name("builtin-extra"));
    }

    /// Asserts that an entry built from a declaration keeps the declared
    /// relative path and resolves its absolute path under the given root.
    ///
    /// Case: the engine registers a declared asset whose files live under a
    /// MOD directory or the engine's own asset directory.
    #[test]
    fn from_declaration_joins_root_and_declared_path() {
        let declaration = AssetDeclaration {
            path: "vrm/Elmer.vrm".to_string(),
            asset_type: AssetType::Vrm,
            description: Some("VRM model named Elmer".to_string()),
        };

        let entry = AssetEntry::from_declaration(
            AssetId::new("vrm:elmer"),
            &declaration,
            Path::new("/r"),
            AssetOrigin::Builtin,
        );

        assert_eq!(entry.id, AssetId::new("vrm:elmer"));
        assert_eq!(entry.path, PathBuf::from("vrm/Elmer.vrm"));
        assert_eq!(entry.absolute_path, PathBuf::from("/r/vrm/Elmer.vrm"));
        assert_eq!(entry.origin, AssetOrigin::Builtin);
        assert_eq!(entry.asset_type, AssetType::Vrm);
        assert_eq!(entry.description.as_deref(), Some("VRM model named Elmer"));
    }
}
