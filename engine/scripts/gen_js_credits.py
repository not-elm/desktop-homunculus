"""Generate the licence credits for npm packages bundled into the built-in UIs.

Reads the `licenses.json` files that Vite's `build.license` writes next to each
build and renders `engine/credits/licenses/JS_THIRD_PARTY.md`.
"""

import json
import sys
from pathlib import Path

BUILD_COMMAND = "pnpm --filter '@hmcs/menu-ui...' run build"

# Repo-relative licence lists. A new built-in UI adds its own list here.
LICENSE_LISTS = [
    "engine/ui/menu/dist/licenses.json",
    "packages/ui/dist/licenses.json",
]

OUTPUT = "engine/credits/licenses/JS_THIRD_PARTY.md"

HEADER = """# JavaScript Third-Party Licenses

The npm packages below are bundled into the engine's built-in UIs.

This file is auto-generated from the Vite licence lists by `engine/scripts/gen_js_credits.py`.
Do not edit manually.
"""


def collect(repo_root: Path) -> list[dict]:
    """Merge every licence list, deduplicated and sorted by `name@version`."""
    merged: dict[str, dict] = {}
    for relative in LICENSE_LISTS:
        for entry in read_list(repo_root / relative):
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
    output.write_text(render(collect(repo_root)), encoding="utf-8")
    return output


def read_list(path: Path) -> list[dict]:
    if not path.is_file():
        raise FileNotFoundError(
            f"{path} is missing. Run `{BUILD_COMMAND}` at the repository root first."
        )
    return json.loads(path.read_text(encoding="utf-8"))


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
