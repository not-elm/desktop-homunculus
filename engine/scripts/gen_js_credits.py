"""Generate the licence credits for npm packages bundled into the built-in UIs.

Reads the `licenses.json` files that Vite's `build.license` writes next to each
build, adds the packages that reach the UIs only through CSS, and renders
`engine/credits/licenses/JS_THIRD_PARTY.md`.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

BUILD_COMMAND = "pnpm --filter '@hmcs/menu-ui...' run build"

# Repo-relative licence lists. A new built-in UI adds its own list here.
LICENSE_LISTS = [
    "engine/ui/menu/dist/licenses.json",
    "packages/ui/dist/licenses.json",
]

# Packages inlined into the UIs through a CSS `@import`, as (package name,
# workspace whose node_modules resolves it). Vite's licence lists only cover JS
# modules, so these are read from node_modules directly.
CSS_PACKAGES = [
    ("tailwindcss", "engine/ui/menu"),
    ("tw-animate-css", "packages/ui"),
]

OUTPUT = "engine/credits/licenses/JS_THIRD_PARTY.md"

HEADER = """# JavaScript Third-Party Licenses

The npm packages below are bundled into the engine's built-in UIs.

This file is auto-generated from the Vite licence lists and the CSS-imported packages by
`engine/scripts/gen_js_credits.py`.
Do not edit manually.
"""


def collect(repo_root: Path) -> list[dict]:
    """Merge every licence list, deduplicated and sorted by `name@version`."""
    entries = [entry for relative in LICENSE_LISTS for entry in read_list(repo_root / relative)]
    entries += [read_css_package(repo_root, name, workspace) for name, workspace in CSS_PACKAGES]
    merged: dict[str, dict] = {}
    for entry in entries:
        merged.setdefault(entry_key(entry), entry)
    return [merged[key] for key in sorted(merged)]


def render(entries: list[dict]) -> str:
    """Render the entries in Vite's per-package Markdown layout."""
    sections = [render_entry(entry) for entry in entries]
    return HEADER + "".join(sections)


def write_credits(repo_root: Path) -> Path:
    """Write `JS_THIRD_PARTY.md` under `repo_root` and return its path."""
    output = repo_root / OUTPUT
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes(render(collect(repo_root)).encode("utf-8"))
    return output


def read_list(path: Path) -> list[dict]:
    if not path.is_file():
        raise FileNotFoundError(
            f"{path} is missing. Run `{BUILD_COMMAND}` at the repository root first."
        )
    return json.loads(path.read_text(encoding="utf-8"))


def read_css_package(repo_root: Path, name: str, workspace: str) -> dict:
    package_dir = repo_root / workspace / "node_modules" / name
    manifest_path = package_dir / "package.json"
    if not manifest_path.is_file():
        raise FileNotFoundError(
            f"{package_dir} is missing. Run `pnpm install` at the repository root first."
        )
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    entry = {"name": manifest["name"], "version": manifest["version"]}
    if manifest.get("license"):
        entry["identifier"] = manifest["license"]
    text = license_text(package_dir)
    if text:
        entry["text"] = text
    return entry


def license_text(package_dir: Path) -> str | None:
    for candidate in sorted(package_dir.iterdir()):
        if candidate.is_file() and candidate.name.lower().startswith(("license", "licence")):
            return candidate.read_text(encoding="utf-8").strip()
    return None


def entry_key(entry: dict) -> str:
    return f"{entry['name']}@{entry['version']}"


def render_entry(entry: dict) -> str:
    identifier = f" ({entry['identifier']})" if entry.get("identifier") else ""
    section = f"\n## {entry['name']} - {entry['version']}{identifier}\n"
    if entry.get("text"):
        section += f"\n{entry['text']}\n"
    return section


def main() -> None:
    repo_root = Path(__file__).resolve().parents[2]
    try:
        output = write_credits(repo_root)
    except FileNotFoundError as error:
        print(f"error: {error}", file=sys.stderr)
        sys.exit(1)
    print(f"==> Wrote {output.relative_to(repo_root)}")


if __name__ == "__main__":
    main()
