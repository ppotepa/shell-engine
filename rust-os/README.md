# rust-os

Rust sidecar operating-system simulation built on `engine-io`.

## Purpose

`rust-os/` is a standalone Rust project that experiments with or implements a
sidecar terminal OS environment using the same IPC bridge concepts as the main
game’s sidecar integrations.

## Main areas

- `kernel/` — core OS-like services and resource management
- `commands/` — command handlers
- `apps/` — built-in applications
- `exec/` — command execution and pipeline logic
- `hosts/` — host integration/registry
- `vfs/` — virtual filesystem support

## Relation to the main game

This is the current in-tree sidecar / terminal-sandbox experiment. It shares
the same overall IPC and terminal-simulation direction as the engine, but there
is no separate shipped `mods/.../cognitOS/` tree in the current repo layout.
