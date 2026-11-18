//! Copies the built panel UIs into `OUT_DIR` so the crate can embed them.

use std::path::{Path, PathBuf};

const BUILTIN_UIS: [&str; 1] = ["settings"];

fn main() {
    let ui_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ui");
    let sources: Vec<(&str, PathBuf)> = BUILTIN_UIS
        .iter()
        .map(|name| (*name, ui_dir.join(name).join("dist/index.html")))
        .collect();
    for (_, source) in &sources {
        println!("cargo::rerun-if-changed={}", source.display());
    }
    let missing: Vec<&str> = sources
        .iter()
        .filter(|(_, source)| !is_built(source))
        .map(|(name, _)| *name)
        .collect();
    if !missing.is_empty() {
        missing.iter().for_each(|name| report_missing(name));
        return;
    }
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    for (name, source) in &sources {
        std::fs::copy(source, out_dir.join(format!("{name}.html")))
            .expect("the built panel UI is copied");
    }
}

fn is_built(html: &Path) -> bool {
    std::fs::metadata(html).is_ok_and(|metadata| metadata.len() > 0)
}

fn report_missing(name: &str) {
    println!(
        "cargo::error=The {name} UI is not built (engine/ui/{name}/dist/index.html is missing or empty). Run `pnpm install` and `pnpm run build:builtin-ui` at the repository root."
    );
}
