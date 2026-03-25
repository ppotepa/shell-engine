# engine-game

Persistent game state with path-based key-value access.

## Purpose

Manages runtime game state as a nested key-value store. Supports
dot-separated path access (get/set/has/remove/push) so Rhai scripts
and scene behaviors can read and write persistent flags, counters,
and inventory data across scene transitions.

## Key Types

- `GameState` — the state container with path-based `get`, `set`, `has`, `remove`, and `push` operations

## Dependencies

- `serde` / `serde_json` — serialization for save/load and JSON value representation

## Usage

Rhai scripts interact with game state through scope variables:

```rhai
state.set("player.health", 100);
let hp = state.get("player.health");
```
