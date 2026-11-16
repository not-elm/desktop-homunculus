//! Helpers shared by the migration tests.

use homunculus_utils::runtime::RuntimeResolver;
use std::path::{Path, PathBuf};

pub(crate) fn fallback_runtime() -> RuntimeResolver {
    RuntimeResolver::new(Path::new("/nonexistent/bin/app"))
}

pub(crate) fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("hmcs-migration-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir should be creatable");
    dir
}
