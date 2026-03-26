# engine-terminal

Terminal capability detection and manifest requirement checks.

## Purpose

`engine-terminal` detects the host terminal’s size and colour support, parses
terminal requirements from mod manifests, and reports requirement violations
before runtime startup continues.

## Key types

- `TerminalCaps`
- `TerminalRequirements`
- `TerminalViolation`

## Main helpers

- `TerminalCaps::detect()`
- `TerminalCaps::validate()`
- `TerminalRequirements::from_manifest()`
- `target_fps_from_manifest()`

## Working with this crate

- keep environment-based colour detection conservative and predictable,
- preserve manifest compatibility for terminal settings such as `target-fps`,
- if terminal requirement fields change, update example mods and launcher docs too.
