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

This is adjacent tooling/experimentation rather than the primary shipped
`cognitOS` implementation under `mods/shell-engine/os/cognitOS/`, but it shares
the same overall sidecar/terminal simulation direction.
