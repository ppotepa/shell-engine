# engine-asset

Scene and asset repository abstractions for mod directories and zip archives.

## Purpose

`engine-asset` hides where authored content comes from. It gives the runtime a
single way to:

- discover scenes,
- load scene YAML from unpacked mods or packaged `.zip` mods,
- assemble scene packages before compilation,
- read arbitrary asset bytes through the same source abstraction.

This crate is the bridge between raw mod storage and the typed scene model
produced by `engine-authoring`.

## Key Types

- `SceneRepository` — loads compiled `Scene` values and discovers scene paths
- `AssetRepository` — reads asset bytes, existence checks, and prefix listings
- `FsSceneRepository` — repository for unpacked mod directories
- `ZipSceneRepository` — repository for packaged mod archives
- `AnySceneRepository` / `AnyAssetRepository` — runtime-selected wrappers
- `ModAssetSourceLoader` — `SourceLoader` implementation for asset refs inside authored content
- `ImageAssetKey` — canonical key for shared image lookup across 2D and 3D consumers
- scene-view profile hydration helpers — load asset-backed `view-profile`,
  `lighting-profile`, and `space-environment-profile` data into compiled scenes
- `compile_scene_document_with_loader_and_source()` — compiles merged authored YAML into a runtime `Scene`

## How it works

1. Select a repository implementation from the mod source path.
2. Load a scene file or scene package root.
3. If the scene is a package, merge partials from `layers/`, `templates/`, and
   `objects/`.
4. Resolve referenced authored fragments through the provided loader.
5. Hand the final YAML to `engine-authoring` for normalization and typed compilation.
6. Hydrate any asset-backed scene-view profile refs so runtime can resolve one
   effective scene-wide look contract without duplicating storage concerns.

The engine should treat this crate as the place for storage format concerns.
Directory-vs-zip logic belongs here, not in higher-level runtime systems.

## Working with this crate

When changing mod loading behavior:

- update both `FsSceneRepository` and `ZipSceneRepository`,
- preserve normalized absolute-style asset paths like `/assets/...`,
- keep scene package assembly behavior identical across directory and zip sources,
- keep compilation delegated to `engine-authoring` rather than duplicating parsing here.

If a change affects how authored assets are resolved, update
`ModAssetSourceLoader` as well.

Conventional scene-view profile directories are:

- `/view-profiles/<id>.yml|yaml`
- `/lighting-profiles/<id>.yml|yaml`
- `/space-environment-profiles/<id>.yml|yaml`

## Image seam (2D + 3D)

Use `resolve_image_asset_key()` to normalize authored image refs once (leading
slash, separator style, and `./` prefix differences), then load through:

- `load_image_asset_with_key()` for decoded cached image assets
- `load_rgba_image_with_key()` for first-frame RGBA access
- `has_image_asset_with_key()` for existence checks

Path-based helpers (`load_image_asset`, `load_rgba_image`, `has_image_asset`)
delegate to the same key-based seam and are equivalent.

## Integration points

- `engine-mod` decides which mod source to load
- `engine-authoring` performs the actual authored-scene compilation
- `engine-compositor`, font loading, audio loading, and other asset consumers use
  the repository-backed asset access patterns
