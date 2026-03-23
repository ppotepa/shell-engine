#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MOD_NAME="${SHELL_QUEST_MOD:-shell-quest}"
MOD_SOURCE="$ROOT_DIR/mods/$MOD_NAME"
MOD_MANIFEST="$MOD_SOURCE/mod.yaml"
# Default to a regular-sized terminal window. Fullscreen terminals can make
# the current authored scenes look effectively blank because they render into
# a much larger character grid than the authored minimum.
WINDOW_MODE="${SHELL_QUEST_WINDOW_MODE:-normal}" # game | normal
FORCED_TERMINAL="${SHELL_QUEST_TERMINAL:-}"
START_SCENE="${SHELL_QUEST_START_SCENE:-}"
SKIP_SPLASH=0
HOLD_ON_EXIT=1
AUDIO=0

print_usage() {
  cat <<'EOF'
Usage: ./launcher.sh [options]

Options:
  --mod <name>             Mod to load (resolves to mods/<name>/). Default: shell-quest
  --mod-source <path>      Explicit mod source path (directory or .zip)
  --start-scene <path>    Jump directly to a specific scene (overrides entrypoint)
  --in-place              Run in current terminal (no new window)
  --terminal <name>       Force terminal binary (e.g. konsole, gnome-terminal)
  --game-window           Try fullscreen / minimal chrome game-like window
  --normal-window         Open regular terminal window
  --no-hold               Do not wait for Enter after process exit
  --skip-splash           Skip the engine splash screen
  --audio                 Enable audio playback
  -h, --help              Show this help
EOF
}

IN_PLACE=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --in-place)
      IN_PLACE=1
      shift
      ;;
    --mod)
      MOD_NAME="${2:-}"
      if [[ -z "$MOD_NAME" ]]; then
        echo "[launcher] --mod requires a value" >&2
        exit 2
      fi
      MOD_SOURCE="$ROOT_DIR/mods/$MOD_NAME"
      MOD_MANIFEST="$MOD_SOURCE/mod.yaml"
      shift 2
      ;;
    --mod-source)
      MOD_SOURCE="${2:-}"
      if [[ -z "$MOD_SOURCE" ]]; then
        echo "[launcher] --mod-source requires a value" >&2
        exit 2
      fi
      MOD_MANIFEST="$MOD_SOURCE/mod.yaml"
      shift 2
      ;;
    --start-scene)
      START_SCENE="${2:-}"
      if [[ -z "$START_SCENE" ]]; then
        echo "[launcher] --start-scene requires a value" >&2
        exit 2
      fi
      shift 2
      ;;
    --terminal)
      FORCED_TERMINAL="${2:-}"
      if [[ -z "$FORCED_TERMINAL" ]]; then
        echo "[launcher] --terminal requires a value" >&2
        exit 2
      fi
      shift 2
      ;;
    --game-window)
      WINDOW_MODE="game"
      shift
      ;;
    --normal-window)
      WINDOW_MODE="normal"
      shift
      ;;
    --no-hold)
      HOLD_ON_EXIT=0
      shift
      ;;
    --skip-splash)
      SKIP_SPLASH=1
      shift
      ;;
    --audio)
      AUDIO=1
      shift
      ;;
    -h|--help)
      print_usage
      exit 0
      ;;
    *)
      echo "[launcher] unknown option: $1" >&2
      print_usage >&2
      exit 2
      ;;
  esac
done

extract_terminal_value() {
  local key="$1"
  awk -v key="$key" '
    /^[[:space:]]*terminal:[[:space:]]*$/ { in_terminal=1; next }
    in_terminal && /^[^[:space:]]/ { in_terminal=0 }
    in_terminal {
      pattern = "^[[:space:]]*" key ":[[:space:]]*([0-9]+)"
      if (match($0, pattern, m)) { print m[1]; exit }
    }
  ' "$MOD_MANIFEST"
}

MIN_WIDTH=120
MIN_HEIGHT=40
MIN_COLOURS=256

if [[ -f "$MOD_MANIFEST" ]]; then
  MIN_WIDTH="$(extract_terminal_value "min_width" || true)"
  MIN_HEIGHT="$(extract_terminal_value "min_height" || true)"
  MIN_COLOURS="$(extract_terminal_value "min_colours" || true)"
fi

MIN_WIDTH="${MIN_WIDTH:-120}"
MIN_HEIGHT="${MIN_HEIGHT:-40}"
MIN_COLOURS="${MIN_COLOURS:-256}"

build_game_cmd() {
  local shell_mod_name shell_root hold_line
  shell_mod_name="$(printf "%q" "$MOD_NAME")"
  shell_root="$(printf "%q" "$ROOT_DIR")"
  hold_line=""
  if [[ "$HOLD_ON_EXIT" == "1" ]]; then
    hold_line=$'if [[ -t 0 ]]; then\n  read -r -p "Press Enter to close..." _\nfi'
  fi

  cat <<EOF
cd $shell_root
export COLUMNS=$MIN_WIDTH
export LINES=$MIN_HEIGHT
if [[ ${MIN_COLOURS} -ge 16777216 ]]; then
  export COLORTERM=truecolor
elif [[ ${MIN_COLOURS} -ge 256 ]]; then
  case "\${TERM:-}" in
    *256color*) ;;
    *) export TERM=xterm-256color ;;
  esac
fi
stty cols "$MIN_WIDTH" rows "$MIN_HEIGHT" 2>/dev/null || true
cargo run -q -p app -- --mod $shell_mod_name${START_SCENE:+ --start-scene "$START_SCENE"}$( [[ "$SKIP_SPLASH" == "1" ]] && echo " --skip-splash" )$( [[ "$AUDIO" == "1" ]] && echo " --audio" )
status=\$?
printf "\\n[launcher] Shell Quest exited with code %s\\n" "\$status"
$hold_line
exit "\$status"
EOF
}

GAME_CMD="$(build_game_cmd)"

pick_terminal() {
  if [[ -n "$FORCED_TERMINAL" ]]; then
    if command -v "$FORCED_TERMINAL" >/dev/null 2>&1; then
      echo "$FORCED_TERMINAL"
      return 0
    fi
    return 1
  fi
  for t in kgx gnome-terminal konsole xfce4-terminal kitty alacritty wezterm xterm; do
    if command -v "$t" >/dev/null 2>&1; then
      echo "$t"
      return 0
    fi
  done
  return 1
}

has_flag() {
  local bin="$1"
  local flag="$2"
  "$bin" --help 2>&1 | grep -q -- "$flag"
}

launch_in_current_terminal() {
  echo "[launcher] GUI terminal not available. Running in current terminal..."
  exec bash -lc "$GAME_CMD"
}

if [[ "$IN_PLACE" == "1" ]]; then
  launch_in_current_terminal
fi

if [[ -z "${DISPLAY:-}" && -z "${WAYLAND_DISPLAY:-}" ]]; then
  launch_in_current_terminal
fi

if ! TERMINAL_BIN="$(pick_terminal)"; then
  if [[ -n "$FORCED_TERMINAL" ]]; then
    echo "[launcher] forced terminal not found: $FORCED_TERMINAL" >&2
  fi
  launch_in_current_terminal
fi

echo "[launcher] launching Shell Quest in new terminal: ${TERMINAL_BIN} (${WINDOW_MODE} mode)"

case "$TERMINAL_BIN" in
  kgx)
    args=(--title "Shell Quest")
    if [[ "$WINDOW_MODE" == "game" ]] && has_flag kgx "--maximize"; then
      args+=(--maximize)
    fi
    kgx "${args[@]}" -- bash -lc "$GAME_CMD" >/dev/null 2>&1 &
    ;;
  gnome-terminal)
    args=(--title="Shell Quest" --geometry="${MIN_WIDTH}x${MIN_HEIGHT}")
    if [[ "$WINDOW_MODE" == "game" ]]; then
      has_flag gnome-terminal "--full-screen" && args+=(--full-screen)
      has_flag gnome-terminal "--hide-menubar" && args+=(--hide-menubar)
    fi
    gnome-terminal "${args[@]}" -- bash -lc "$GAME_CMD" >/dev/null 2>&1 &
    ;;
  konsole)
    args=(--noclose --workdir "$ROOT_DIR")
    if [[ "$WINDOW_MODE" == "game" ]]; then
      has_flag konsole "--fullscreen" && args+=(--fullscreen)
      has_flag konsole "--hide-menubar" && args+=(--hide-menubar)
      has_flag konsole "--notabbar" && args+=(--notabbar)
      has_flag konsole "--notoolbar" && args+=(--notoolbar)
    fi
    konsole "${args[@]}" -e bash -lc "$GAME_CMD" >/dev/null 2>&1 &
    ;;
  xfce4-terminal)
    args=(--title="Shell Quest" --geometry="${MIN_WIDTH}x${MIN_HEIGHT}")
    if [[ "$WINDOW_MODE" == "game" ]] && has_flag xfce4-terminal "--fullscreen"; then
      args+=(--fullscreen)
    fi
    xfce4-terminal "${args[@]}" --command "bash -lc \"$GAME_CMD\"" >/dev/null 2>&1 &
    ;;
  kitty)
    args=(--title "Shell Quest")
    if [[ "$WINDOW_MODE" == "game" ]]; then
      args+=(--start-as=fullscreen)
    fi
    kitty "${args[@]}" bash -lc "$GAME_CMD" >/dev/null 2>&1 &
    ;;
  alacritty)
    args=(-t "Shell Quest" -e bash -lc "$GAME_CMD")
    if [[ "$WINDOW_MODE" == "game" ]]; then
      args=(-o window.startup_mode=Fullscreen -o window.decorations=None "${args[@]}")
    else
      args=(-o window.dimensions.columns="$MIN_WIDTH" -o window.dimensions.lines="$MIN_HEIGHT" "${args[@]}")
    fi
    alacritty "${args[@]}" >/dev/null 2>&1 &
    ;;
  wezterm)
    args=(start --cwd "$ROOT_DIR" -- bash -lc "$GAME_CMD")
    if [[ "$WINDOW_MODE" == "game" ]] && has_flag wezterm "--fullscreen"; then
      args=(start --fullscreen --cwd "$ROOT_DIR" -- bash -lc "$GAME_CMD")
    fi
    wezterm "${args[@]}" >/dev/null 2>&1 &
    ;;
  xterm)
    args=(-T "Shell Quest" -geometry "${MIN_WIDTH}x${MIN_HEIGHT}" -e bash -lc "$GAME_CMD")
    xterm "${args[@]}" >/dev/null 2>&1 &
    ;;
  *)
    launch_in_current_terminal
    ;;
esac

disown || true
