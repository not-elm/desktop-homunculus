//! Copies the built menu UI into `OUT_DIR` so the crate can embed it.

use std::path::{Path, PathBuf};

const MENU_UI_HTML: &str = "../../ui/menu/dist/index.html";

fn main() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join(MENU_UI_HTML);
    println!("cargo::rerun-if-changed={}", source.display());
    if !is_built(&source) {
        println!(
            "cargo::error=The menu UI is not built. Run `pnpm install` and `pnpm run build:builtin-ui` at the repository root."
        );
        return;
    }
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    std::fs::copy(&source, out_dir.join("menu.html")).expect("the built menu UI is copied");
}

fn is_built(html: &Path) -> bool {
    std::fs::metadata(html).is_ok_and(|metadata| metadata.len() > 0)
}
