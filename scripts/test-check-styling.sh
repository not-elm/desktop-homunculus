#!/usr/bin/env bash
# Tests for scripts/check-styling.sh.
# Each test creates a tmp git repo, drops fixture files, runs check-styling.sh,
# and asserts exit code + stdout contains expected substrings.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CHECK="$SCRIPT_DIR/check-styling.sh"
# Subshells cannot mutate parent variables, so persist counters via temp files.
COUNTER_DIR="$(mktemp -d)"
trap 'rm -rf "$COUNTER_DIR"' EXIT
echo 0 > "$COUNTER_DIR/pass"
echo 0 > "$COUNTER_DIR/fail"

bump_pass() {
  local n
  n="$(cat "$COUNTER_DIR/pass")"
  echo $((n + 1)) > "$COUNTER_DIR/pass"
}

bump_fail() {
  local n
  n="$(cat "$COUNTER_DIR/fail")"
  echo $((n + 1)) > "$COUNTER_DIR/fail"
}

assert_exit() {
  local expected="$1"
  local actual="$2"
  local name="$3"
  if [ "$actual" -eq "$expected" ]; then
    echo "  PASS: $name"
    bump_pass
  else
    echo "  FAIL: $name (expected exit $expected, got $actual)"
    bump_fail
  fi
}

assert_contains() {
  local needle="$1"
  local haystack="$2"
  local name="$3"
  if echo "$haystack" | grep -qF "$needle"; then
    echo "  PASS: $name"
    bump_pass
  else
    echo "  FAIL: $name (output did not contain '$needle')"
    echo "    Output: $haystack"
    bump_fail
  fi
}

setup_tmp_repo() {
  local dir
  dir="$(mktemp -d)"
  (
    cd "$dir"
    git init -q -b main
    git config user.email t@t
    git config user.name t
    mkdir -p packages/ui/src mods/menu/ui/src scripts
    cp "$CHECK" scripts/
    chmod +x scripts/check-styling.sh
    git add -A
    git commit -q -m initial
  )
  echo "$dir"
}

echo "=== test_full_scope_passes_with_only_allowlisted_files ==="
DIR="$(setup_tmp_repo)"
(
  cd "$DIR"
  echo "@import 'tailwindcss';" > packages/ui/src/index.css
  echo "@keyframes a { 0% { opacity: 0 } }" > packages/ui/src/animation.css
  output="$(./scripts/check-styling.sh --full 2>&1)" && exit_code=0 || exit_code=$?
  assert_exit 0 "$exit_code" "exits 0 when only allowlisted CSS exists"
)
rm -rf "$DIR"

echo "=== test_full_scope_fails_on_disallowed_css ==="
DIR="$(setup_tmp_repo)"
(
  cd "$DIR"
  echo ".btn { color: red }" > packages/ui/src/extra.css
  output="$(./scripts/check-styling.sh --full 2>&1)" && exit_code=0 || exit_code=$?
  assert_exit 1 "$exit_code" "exits 1 when a disallowed CSS file exists"
  assert_contains "extra.css" "$output" "error message names the offending file"
)
rm -rf "$DIR"

echo "=== test_diff_scope_only_checks_added_files ==="
DIR="$(setup_tmp_repo)"
(
  cd "$DIR"
  echo ".pre-existing { color: blue }" > packages/ui/src/legacy.css
  git add -A && git commit -q -m "add legacy"
  git checkout -q -b feature
  echo ".new { color: green }" > packages/ui/src/new-bad.css
  git add -A && git commit -q -m "add new bad"
  output="$(./scripts/check-styling.sh --diff main 2>&1)" && exit_code=0 || exit_code=$?
  assert_exit 1 "$exit_code" "diff scope flags newly added disallowed CSS"
  assert_contains "new-bad.css" "$output" "diff error names the new file"
  if echo "$output" | grep -qF "legacy.css"; then
    echo "  FAIL: diff scope incorrectly flagged pre-existing file"
    bump_fail
  else
    echo "  PASS: diff scope ignored pre-existing file"
    bump_pass
  fi
)
rm -rf "$DIR"

echo "=== test_apply_outside_reset_block_fails ==="
DIR="$(setup_tmp_repo)"
(
  cd "$DIR"
  cat > packages/ui/src/index.css <<'CSS'
@import 'tailwindcss';
.foo {
  @apply bg-primary;
}
CSS
  output="$(./scripts/check-styling.sh --full 2>&1)" && exit_code=0 || exit_code=$?
  assert_exit 1 "$exit_code" "non-reset @apply fails"
  assert_contains "@apply" "$output" "error mentions @apply"
)
rm -rf "$DIR"

echo "=== test_apply_in_known_reset_passes ==="
DIR="$(setup_tmp_repo)"
(
  cd "$DIR"
  cat > packages/ui/src/index.css <<'CSS'
@import 'tailwindcss';
* {
  @apply border-border outline-ring/50;
}
body {
  @apply bg-transparent text-foreground antialiased;
}
CSS
  output="$(./scripts/check-styling.sh --full 2>&1)" && exit_code=0 || exit_code=$?
  assert_exit 0 "$exit_code" "exempted @apply lines pass"
)
rm -rf "$DIR"

echo "=== test_inline_style_object_literal_with_only_literals_fails ==="
DIR="$(setup_tmp_repo)"
(
  cd "$DIR"
  cat > mods/menu/ui/src/Bad.tsx <<'TSX'
export const Bad = () => <div style={{ width: '100%', display: 'flex' }} />;
TSX
  output="$(./scripts/check-styling.sh --full 2>&1)" && exit_code=0 || exit_code=$?
  assert_exit 1 "$exit_code" "static inline style object literal fails"
  assert_contains "Bad.tsx" "$output" "error names the file"
)
rm -rf "$DIR"

echo "=== test_inline_style_with_dynamic_value_passes ==="
DIR="$(setup_tmp_repo)"
(
  cd "$DIR"
  cat > mods/menu/ui/src/Good.tsx <<'TSX'
export const Good = ({ w }: { w: number }) => <div style={{ width: `${w}%` }} />;
TSX
  output="$(./scripts/check-styling.sh --full 2>&1)" && exit_code=0 || exit_code=$?
  assert_exit 0 "$exit_code" "dynamic inline style passes"
)
rm -rf "$DIR"

echo "=== test_allowlist_rejects_deeply_nested_css ==="
DIR="$(setup_tmp_repo)"
(
  cd "$DIR"
  mkdir -p mods/persona/shared/sub/dir
  echo ".sneaky { color: red }" > mods/persona/shared/sub/dir/anything.css
  output="$(./scripts/check-styling.sh --full 2>&1)" && exit_code=0 || exit_code=$?
  assert_exit 1 "$exit_code" "deeply-nested CSS under allowlisted prefix is rejected"
  assert_contains "mods/persona/shared/sub/dir/anything.css" "$output" "error names the deeply-nested file"
)
rm -rf "$DIR"

echo "=== test_full_scope_ignores_dist_and_node_modules ==="
DIR="$(setup_tmp_repo)"
(
  cd "$DIR"
  mkdir -p packages/ui/dist packages/ui/node_modules/something
  echo ".leak { color: red }" > packages/ui/dist/leak.css
  echo ".vendor { color: blue }" > packages/ui/node_modules/something/foo.css
  output="$(./scripts/check-styling.sh --full 2>&1)" && exit_code=0 || exit_code=$?
  assert_exit 0 "$exit_code" "build artifacts under dist/ and node_modules/ are pruned"
)
rm -rf "$DIR"

echo "=== test_diff_scope_fails_when_base_missing ==="
DIR="$(setup_tmp_repo)"
(
  cd "$DIR"
  output="$(./scripts/check-styling.sh --diff nonexistent-ref 2>&1)" && exit_code=0 || exit_code=$?
  assert_exit 2 "$exit_code" "missing diff base ref exits 2"
  assert_contains "nonexistent-ref" "$output" "error names the missing ref"
)
rm -rf "$DIR"

PASS="$(cat "$COUNTER_DIR/pass")"
FAIL="$(cat "$COUNTER_DIR/fail")"
echo
echo "=== Summary: $PASS passed, $FAIL failed ==="
[ "$FAIL" -eq 0 ]
