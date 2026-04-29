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

# Allowlist of CSS paths permitted to exist. Globs match via case statement.
is_allowlisted_css() {
  local path="$1"
  case "$path" in
    packages/ui/src/index.css) return 0 ;;
    packages/ui/src/animation.css) return 0 ;;
    packages/ui/.storybook/storybook.css) return 0 ;;
    mods/menu/ui/.storybook/storybook.css) return 0 ;;
    mods/*/ui/src/index.css) return 0 ;;
    mods/persona/shared/*.css) return 0 ;;
    *) return 1 ;;
  esac
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
  read_lines css_candidates < <(find packages mods -type f -name "*.css" 2>/dev/null | sort)
  read_lines tsx_candidates < <(find packages mods -type f \( -name "*.tsx" -o -name "*.ts" \) 2>/dev/null | sort)
else
  if ! git rev-parse --verify "$BASE" >/dev/null 2>&1; then
    echo "warn: base ref '$BASE' not found; skipping diff-scope checks" >&2
  else
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
# Heuristic: match `style={{ ... }}` where every value is a string or number literal.
# Multi-line objects are out of scope (false negatives ok).
for f in "${tsx_candidates[@]+"${tsx_candidates[@]}"}"; do
  [ -z "$f" ] && continue
  if [ ! -f "$f" ]; then
    continue
  fi
  while IFS= read -r line; do
    if ! echo "$line" | grep -qE 'style=\{\{'; then
      continue
    fi
    after="$(echo "$line" | sed -E 's/^.*style=\{\{//')"
    # Strip string literals so their contents don't count as identifiers.
    stripped="$(echo "$after" | sed -E "s/'[^']*'//g; s/\"[^\"]*\"//g")"
    # Template literal or substitution -> dynamic.
    if echo "$stripped" | grep -qE '\$\{|\`'; then
      continue
    fi
    # Take only the chunk inside the first style={{...}}.
    inside="$(echo "$stripped" | sed -E 's/\}\}.*$//')"
    # Strip property-name identifiers (followed by colon) and numeric literals.
    cleaned="$(echo "$inside" | sed -E 's/[a-zA-Z_$][a-zA-Z0-9_$]*[[:space:]]*://g; s/-?[0-9]+(\.[0-9]+)?//g')"
    # Any remaining identifier means a non-literal value (variable, fn call, spread, etc.).
    if echo "$cleaned" | grep -qE '[a-zA-Z_$]'; then
      continue
    fi
    errors+=("static inline style in $f: $line")
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
