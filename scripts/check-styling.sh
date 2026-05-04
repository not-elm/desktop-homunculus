#!/usr/bin/env bash
# Tailwind utility-first enforcement check.
#
# Usage:
#   scripts/check-styling.sh                # diff scope vs origin/main
#   scripts/check-styling.sh --diff <base>  # diff scope vs <base>
#   scripts/check-styling.sh --full         # full-repo scope (local opt-in)
#
# Exits 0 if all checks pass, 1 if any fail.

set -euo pipefail

MODE="diff"
BASE="${BASE:-origin/main}"

while [ $# -gt 0 ]; do
  case "$1" in
    --full) MODE="full"; shift ;;
    --diff)
      MODE="diff"
      shift
      if [ $# -gt 0 ] && [ "${1:0:2}" != "--" ]; then
        BASE="$1"
        shift
      fi
      ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
done

# Allowlist of CSS paths permitted to exist.
# Exact paths use `case`; segment-bounded globs use `=~` regex with [^/]+
# to ensure exactly one path segment (no nesting).
is_allowlisted_css() {
  local path="$1"
  case "$path" in
    packages/ui/src/index.css) return 0 ;;
    packages/ui/src/animation.css) return 0 ;;
    packages/ui/.storybook/storybook.css) return 0 ;;
    mods/menu/ui/.storybook/storybook.css) return 0 ;;
    mods/persona/management/src/index.css) return 0 ;;
  esac
  if [[ "$path" =~ ^mods/[^/]+/ui/src/index\.css$ ]]; then return 0; fi
  if [[ "$path" =~ ^mods/persona/shared/[^/]+\.css$ ]]; then return 0; fi
  return 1
}

# Allowed @apply lines (substring match after trimming).
is_exempt_apply() {
  local line="$1"
  case "$line" in
    *"@apply border-border outline-ring/50"*) return 0 ;;
    *"@apply bg-transparent text-foreground antialiased"*) return 0 ;;
    *) return 1 ;;
  esac
}

errors=()

# read_lines reads stdin into the named array variable, one element per line.
# Compat shim for bash 3.2 which lacks `mapfile`.
read_lines() {
  local __arr_name="$1"
  local __line
  eval "$__arr_name=()"
  while IFS= read -r __line; do
    eval "$__arr_name+=(\"\$__line\")"
  done
}

# Step 1: collect candidate files for the active scope.
css_candidates=()
tsx_candidates=()
if [ "$MODE" = "full" ]; then
  read_lines css_candidates < <(
    find packages mods \
      \( -name node_modules -o -name dist -o -name build -o -name .next -o -name .turbo \) -prune -o \
      -type f -name "*.css" -print 2>/dev/null | sort
  )
  read_lines tsx_candidates < <(
    find packages mods \
      \( -name node_modules -o -name dist -o -name build -o -name .next -o -name .turbo \) -prune -o \
      -type f \( -name "*.tsx" -o -name "*.ts" \) -print 2>/dev/null | sort
  )
else
  if ! git rev-parse --verify "$BASE" >/dev/null 2>&1; then
    echo "error: base ref '$BASE' not found. Run 'git fetch' or pass --diff <existing-ref>." >&2
    echo "error: refusing to silently skip enforcement." >&2
    exit 2
  fi
  read_lines css_candidates < <(
    git diff --name-only --diff-filter=AM "$BASE"...HEAD 2>/dev/null \
      | grep -E '\.css$' \
      | sort || true
  )
  read_lines tsx_candidates < <(
    git diff --name-only --diff-filter=AM "$BASE"...HEAD 2>/dev/null \
      | grep -E '\.(tsx|ts)$' \
      | sort || true
  )
fi

# Step 2: check each CSS file is allowlisted; if so, scan @apply usage.
for f in "${css_candidates[@]+"${css_candidates[@]}"}"; do
  [ -z "$f" ] && continue
  if ! is_allowlisted_css "$f"; then
    errors+=("disallowed CSS file: $f")
    continue
  fi
  if [ ! -f "$f" ]; then
    continue
  fi
  while IFS= read -r line; do
    case "$line" in
      *"@apply"*)
        if ! is_exempt_apply "$line"; then
          errors+=("non-exempt @apply in $f: $(echo "$line" | sed 's/^[[:space:]]*//')")
        fi
        ;;
    esac
  done < "$f"
done

# Step 3: scan TSX/TS files for static inline style attributes.
#
# Heuristic: match `style={{ ... }}` on a single line where every value is a
# string literal or numeric literal (no template literals, identifiers,
# function calls, member access, spreads, etc.).
#
# Implemented in awk for portable, deterministic regex behavior across
# bash 3.2 (macOS) and bash 5 (Linux CI). The earlier sed-pipeline version
# diverged between BSD sed and GNU sed in subtle ways.
#
# Heuristic limitations (acceptable false negatives):
# - Only the first style={{...}} per line is scanned
# - Multi-line object literals are not analysed
# - JS comments inside style objects may be misread
# The .claude/rules/tailwind-style.md doc is the source of truth; this
# script is a heuristic safety net that biases toward false negatives.
classify_inline_style() {
  # Reads stdin (the line). Prints "static" if the line contains a static
  # inline style={{...}}, "dynamic" if it has a dynamic one, "none" otherwise.
  awk '
    {
      line = $0
      # Find style={{ marker
      if (!match(line, /style=\{\{/)) { print "none"; exit }
      rest = substr(line, RSTART + RLENGTH)
      # Find closing }} (first occurrence)
      if (!match(rest, /\}\}/)) { print "none"; exit }
      content = substr(rest, 1, RSTART - 1)

      # Dynamic indicators inside the object literal.
      # Template-literal substitution or backtick.
      if (content ~ /\$\{/ || content ~ /`/) { print "dynamic"; exit }
      # Spread.
      if (content ~ /\.\.\./) { print "dynamic"; exit }

      # Strip string literals (single-quoted and double-quoted, non-greedy).
      gsub(/'\''[^'\'']*'\''/, "", content)
      gsub(/"[^"]*"/, "", content)
      # Strip numeric literals (incl. negatives, decimals).
      gsub(/-?[0-9]+(\.[0-9]+)?/, "", content)
      # Strip property-name identifiers followed by colon
      # (key may be plain identifier; quoted-key was already stripped above).
      gsub(/[A-Za-z_$][A-Za-z0-9_$]*[[:space:]]*:/, "", content)
      # Strip whitespace, commas, parentheses around the colon stripping.
      gsub(/[[:space:],]/, "", content)

      if (length(content) == 0) { print "static"; exit }
      print "dynamic"
    }
  '
}

for f in "${tsx_candidates[@]+"${tsx_candidates[@]}"}"; do
  [ -z "$f" ] && continue
  if [ ! -f "$f" ]; then
    continue
  fi
  while IFS= read -r line; do
    case "$line" in
      *"style={{"*)
        verdict="$(printf '%s\n' "$line" | classify_inline_style)"
        if [ "$verdict" = "static" ]; then
          errors+=("static inline style in $f: $line")
        fi
        ;;
    esac
  done < "$f"
done

errors_count="${#errors[@]}"
if [ "$errors_count" -gt 0 ]; then
  echo "check-styling: $errors_count violation(s):"
  for e in "${errors[@]+"${errors[@]}"}"; do
    echo "  - $e"
  done
  exit 1
fi

echo "check-styling: OK ($MODE scope)"
exit 0
