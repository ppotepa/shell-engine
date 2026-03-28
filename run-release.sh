#!/usr/bin/env bash
set -euo pipefail

# Release run helper:
# - `--sdl2` first (requested)
# - `--opt` enables all optimization flags
# - script-level `--no-cap` unlocks pacing as much as current engine limits allow:
#   sets `--target-fps` to the engine max (240).
#   (VSync is left unchanged to avoid tearing/flicker by default.)

script_no_cap=0
has_target_fps=0
pass_args=()

for arg in "$@"; do
  case "$arg" in
    --no-cap)
      script_no_cap=1
      ;;
    --target-fps|--target-fps=*)
      has_target_fps=1
      pass_args+=("$arg")
      ;;
    *)
      pass_args+=("$arg")
      ;;
  esac
done

app_args=(--sdl2 --opt)
if [[ "$script_no_cap" -eq 1 ]]; then
  if [[ "$has_target_fps" -eq 0 ]]; then
    app_args+=(--target-fps 240)
  fi
fi

exec cargo run -p app --release -- "${app_args[@]}" "${pass_args[@]}"
