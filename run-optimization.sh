#!/usr/bin/env bash
# Run shell-quest with all experimental pipeline optimizations enabled.
# Use plain `cargo run -p app` for the safe baseline (no experimental optimizations).
#
# Flags:
#   --opt-comp     Compositor: layer-scratch skip (#4), dirty-halfblock narrowing (#5)
#   --opt-present  Present: hash-based static frame skip (#13)
#   --opt-diff     Diff: dirty-region scan instead of full-buffer scan (experimental)

exec cargo run -p app -- --opt-comp --opt-present "$@"
