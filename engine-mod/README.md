# engine-mod

Mod manifest loading from directories and zip archives.

## Purpose

Loads mod manifests from either unpacked directory trees or packaged
zip files. Provides a unified `ModSource` abstraction so the rest of
the engine does not need to know whether assets come from disk or an
archive.

## Key Types

- `ModManifest` — deserialized mod metadata (name, version, entry scene)
- `ModSource` — enum: `Dir(PathBuf)` or `Zip(PathBuf)`
- `load_mod()` — entry point that detects source type and loads the manifest

## Dependencies

- `engine-error` — shared error types
- `serde_yaml` — YAML deserialization of manifest files
- `zip` — reading assets from zip-packaged mods

## Usage

```rust
let manifest = load_mod("mods/shell-quest")?;
```
