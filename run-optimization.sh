#!/usr/bin/env bash
# Run shell-quest with ALL pipeline optimizations enabled.
# Use plain `cargo run -p app` for the safe baseline (no experimental optimizations).
#
# The --opt flag includes:
#   --opt-comp     Compositor: layer-scratch skip (#4), dirty-halfblock narrowing (#5)
#   --opt-present  Present: hash-based static frame skip (#13)
#   --opt-diff     Diff: dirty-region scan instead of full-buffer scan (experimental)
#   --opt-skip     Frame-skip oracle: unified skipping (prevents animation flicker)
#   --opt-rowdiff  Row-level dirty skip in diff scan (experimental)
#   --opt-async    Async display sink: offload terminal I/O to background thread

exec cargo run -p app -- --opt "$@"
