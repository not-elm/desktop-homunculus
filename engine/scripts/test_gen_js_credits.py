"""Tests for gen_js_credits."""

import json
import tempfile
import unittest
from pathlib import Path

import gen_js_credits


def write_list(path: Path, entries: list[dict]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(entries), encoding="utf-8")


def add_ui(root: Path, name: str) -> None:
    ui_dir = root / "engine" / "ui" / name
    ui_dir.mkdir(parents=True, exist_ok=True)
    (ui_dir / "package.json").write_text(json.dumps({"name": f"@hmcs/{name}-ui"}), encoding="utf-8")


def write_css_packages(root: Path) -> None:
    for name, workspace in gen_js_credits.CSS_PACKAGES:
        package_dir = root / workspace / "node_modules" / name
        package_dir.mkdir(parents=True, exist_ok=True)
        manifest = {"name": name, "version": "1.0.0", "license": "MIT"}
        (package_dir / "package.json").write_text(json.dumps(manifest), encoding="utf-8")
        (package_dir / "LICENSE").write_text(f"{name} licence text\n", encoding="utf-8")


class GenJsCreditsTest(unittest.TestCase):
    def test_duplicates_are_merged_and_sorted(self) -> None:
        react = {"name": "react", "version": "19.2.4", "identifier": "MIT", "text": "MIT text"}
        radix = {"name": "@radix-ui/react-menu", "version": "2.1.0", "identifier": "MIT"}
        scheduler = {"name": "scheduler", "version": "0.27.0", "identifier": "MIT"}
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            add_ui(root, "menu")
            add_ui(root, "settings")
            menu_list, settings_list, ui_list = gen_js_credits.license_lists(root)
            write_list(menu_list, [scheduler, react])
            write_list(settings_list, [react])
            write_list(ui_list, [react, radix])
            write_css_packages(root)

            entries = gen_js_credits.collect(root)

        names = [f"{entry['name']}@{entry['version']}" for entry in entries]
        self.assertEqual(
            names,
            [
                "@radix-ui/react-menu@2.1.0",
                "react@19.2.4",
                "scheduler@0.27.0",
                "tailwindcss@1.0.0",
                "tw-animate-css@1.0.0",
            ],
        )

    def test_css_only_packages_are_credited(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            add_ui(root, "menu")
            for path in gen_js_credits.license_lists(root):
                write_list(path, [])
            write_css_packages(root)

            entries = gen_js_credits.collect(root)

        self.assertIn(
            {
                "name": "tailwindcss",
                "version": "1.0.0",
                "identifier": "MIT",
                "text": "tailwindcss licence text",
            },
            entries,
        )
        self.assertIn("tw-animate-css", [entry["name"] for entry in entries])

    def test_render_matches_vite_markdown_layout(self) -> None:
        entries = [
            {"name": "react", "version": "19.2.4", "identifier": "MIT", "text": "MIT text"},
        ]

        rendered = gen_js_credits.render(entries)

        self.assertTrue(rendered.startswith("# JavaScript Third-Party Licenses\n"))
        self.assertIn("\n## react - 19.2.4 (MIT)\n\nMIT text\n", rendered)

    def test_missing_list_names_the_build_command(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            add_ui(root, "menu")
            with self.assertRaises(FileNotFoundError) as raised:
                gen_js_credits.collect(root)

        self.assertIn("pnpm run build:builtin-ui", str(raised.exception))

    def test_license_lists_cover_each_built_in_ui(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            add_ui(root, "settings")
            add_ui(root, "menu")

            lists = gen_js_credits.license_lists(root)

        self.assertEqual(
            lists,
            [
                root / "engine/ui/menu/dist/licenses.json",
                root / "engine/ui/settings/dist/licenses.json",
                root / "packages/ui/dist/licenses.json",
            ],
        )

    def test_license_lists_skip_directories_without_a_package(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            add_ui(root, "menu")
            (root / "engine/ui/stray").mkdir(parents=True)

            lists = gen_js_credits.license_lists(root)

        self.assertNotIn(root / "engine/ui/stray/dist/licenses.json", lists)
        self.assertEqual(len(lists), 2)


if __name__ == "__main__":
    unittest.main()
