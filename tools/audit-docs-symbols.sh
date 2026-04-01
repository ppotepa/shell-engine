#!/usr/bin/env bash
# audit-docs-symbols.sh — scan markdown docs for legacy/removed symbols.
#
# Usage:
#   ./tools/audit-docs-symbols.sh
#   ./tools/audit-docs-symbols.sh "world.ship_set_turn" "TopDownShipController"
#   ./tools/audit-docs-symbols.sh --root docs "attach_ship_controller"
#
# Exit codes:
#   0 - no matches
#   1 - matches found
#   2 - invalid usage or missing dependency

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
SEARCH_ROOT="${REPO_ROOT}"

declare -a DEFAULT_PATTERNS=(
    "TopDownShipController"
    "attach_ship_controller"
    "world.ship_set_turn"
    "world.ship_set_thrust"
    "world.ship_heading"
    "world.ship_heading_vector"
    "world.ship_velocity"
)

declare -a USER_PATTERNS=()

print_help() {
    cat <<'EOF'
Scan markdown docs for legacy/removed symbols.

Usage:
  audit-docs-symbols.sh [--root <path>] [pattern...]

Options:
  --root <path>   Limit scan to this directory (default: repo root)
  -h, --help      Show this help

Behavior:
  - If patterns are provided, scans for those patterns.
  - If no patterns are provided, scans for built-in legacy API patterns.
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --root)
            if [[ $# -lt 2 ]]; then
                echo "ERROR: --root requires a path" >&2
                exit 2
            fi
            SEARCH_ROOT="$2"
            shift 2
            ;;
        -h|--help)
            print_help
            exit 0
            ;;
        *)
            USER_PATTERNS+=("$1")
            shift
            ;;
    esac
done

if ! command -v rg >/dev/null 2>&1; then
    echo "ERROR: ripgrep (rg) is required but not installed." >&2
    exit 2
fi

if [[ ! -d "${SEARCH_ROOT}" ]]; then
    echo "ERROR: search root does not exist: ${SEARCH_ROOT}" >&2
    exit 2
fi

declare -a PATTERNS=()
if [[ ${#USER_PATTERNS[@]} -gt 0 ]]; then
    PATTERNS=("${USER_PATTERNS[@]}")
else
    PATTERNS=("${DEFAULT_PATTERNS[@]}")
fi

echo "Scanning markdown files in: ${SEARCH_ROOT}"
echo "Patterns: ${PATTERNS[*]}"

found=0
for p in "${PATTERNS[@]}"; do
    if rg -nF --glob '**/*.md' --glob '!target/**' --glob '!.git/**' "$p" "${SEARCH_ROOT}"; then
        found=1
    fi
done

if [[ ${found} -eq 1 ]]; then
    echo
    echo "Legacy symbol matches found."
    exit 1
fi

echo "No matches found."
exit 0
