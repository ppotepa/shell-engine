# Shell Quest Assets Guide

## 1) Purpose

This document explains how Shell Quest thinks about assets, sprites, objects, and authored content.

It is intentionally project-level, not just schema-level:

- what kinds of assets exist,
- where they live,
- how runtime loads them,
- how they are referenced from YAML,
- what an "object" really is,
- which invariants you should preserve when extending the system.

If you are authoring a new mod, changing runtime asset loading, extending `devtool`, or wiring editor support, this is the document to start with.

## 2) Mental model

The project has three different layers of "content building blocks":

- **assets** = raw files such as images, fonts, and mesh files
- **sprites** = runtime renderable nodes that may consume assets
- **objects** = reusable prefabs that expand into sprites and/or layers

The shortest version is:

- **Asset** = file data
- **Sprite** = something drawable/layoutable in a layer
- **Object** = reusable authored composition
- **Layer** = visual slice of a scene
- **Scene** = flow + composition order

## 3) Mod structure and where assets live

At the top level, a mod is either:

- an unpacked directory under `mods/<mod>/`
- or a `.zip` archive loaded the same way by runtime

Typical structure:

```text
mods/<mod>/
├── mod.yaml
├── assets/
│   ├── images/
│   ├── fonts/
│   └── ...
├── objects/
│   └── *.yml
└── scenes/
    └── <scene>/
        ├── scene.yml
        ├── layers/
        ├── templates/
        └── objects/
```

Important references:

- mod entrypoint and terminal config: `mods/shell-quest/mod.yaml`
- runtime directory/zip source selection: `engine/src/repositories.rs`
- mod root asset anchor: `engine/src/assets.rs`

### Raw staging folders

`mods/*/assets/raw/` should be treated as a **local working area** for source files such as:

- stop-motion frame sequences
- raw captures
- intermediate exports
- input material for later conversion

Those raw folders are ignored by git on purpose.

Rule of thumb:

- keep source frames in `assets/raw/`
- commit usable converted assets outside `raw/`

For example:

- local source frames: `mods/shell-quest/assets/raw/intro/*.png`
- committed converted output: `mods/shell-quest/assets/images/intro/cutscene.gif`

## 4) Asset paths and source semantics

Today authored asset references are primarily **mod-local paths** such as:

- `/assets/images/tux.png`
- `/assets/fonts/.../manifest.yaml`
- `/scenes/3d/helsinki-university/city_scene_horizontal_front_yup.obj`

The runtime convention is:

- paths are written with a leading `/`
- runtime strips the leading slash and resolves them relative to the mod source
- the same authored path works for both unpacked directories and `.zip` mod packages

Key code:

- path anchoring: `engine/src/assets.rs`
- asset repositories: `engine/src/repositories.rs`
- source abstraction foundation: `engine/src/asset_source.rs`

### Current source abstraction status

The engine now has a clean runtime foundation for asset sources:

- `SourceRef`
- `SourceKind`
- `SourceLoader`
- `SourceAdapter<T>`
- `ModAssetSourceLoader`

Right now only **mod-local assets** are enabled as a source kind in runtime.

That is intentional: the abstraction is now in place so future source kinds such as URL, generated data, or additional packaged sources can be added without rewriting image/OBJ/font consumers.

## 5) Asset categories

### 5.1 Images

Images are currently used by `image` sprites.

Typical location:

- `assets/images/...`

Example:

- `mods/shell-quest/scenes/mainmenu/layers/main.yml`
- `mods/shell-quest/scenes/02-intro-boot/layers/main.yml`

Runtime loader:

- `engine/src/image_loader.rs`

Conversion helper for stop-motion/raw frame sequences:

- `tools/devtool/stop_animation_converter.py`

That helper:

- preserves frame size 1:1
- builds one global 256-colour palette for the whole sequence
- applies mild auto-contrast/contrast/sharpness tuning
- uses Floyd-Steinberg dithering by default
- writes a GIF suitable for preview/reference/export workflows

Current behavior:

- image bytes are loaded through the shared source abstraction
- static images are decoded into `LoadedRgbaImage`
- GIFs are decoded into an animated image asset with per-frame delays
- both are then rendered into terminal cells by the compositor through the same pixel-buffer path

Rendering path:

- `engine/src/systems/compositor/image_render.rs`

Current animation behavior:

- `image` sprites may point at `.gif` files
- the compositor selects the active GIF frame from sprite elapsed time
- playback loops over the total GIF duration
- there is no dedicated YAML playback policy yet (`once`, `ping-pong`, `hold-last-frame`, etc.)

### 5.2 Fonts

Fonts are asset-backed only for rasterized/custom text rendering.

Typical location:

- `assets/fonts/<font-label>/<size>/<mode>/manifest.yaml`

Example:

- `mods/shell-quest/assets/fonts/abril-fatface/24px/ascii/manifest.yaml`

Runtime font loader:

- `engine/src/rasterizer/font_loader.rs`

Font manifests describe:

- font label/family
- mode
- glyph entries
- file paths under `glyphs/`
- width/height per glyph

Schema:

- `schemas/font-manifest.schema.yaml`

Important note:

- built-in generic fonts such as `generic:1`, `generic:2`, `generic:half`, `generic:quad`, `generic:braille` do **not** require asset files
- named fonts such as `"Abril Fatface:ascii"` do require assets under `assets/fonts`

### 5.3 OBJ meshes

OBJ assets are used by `obj` sprites.

Typical sources:

- `/assets/.../*.obj`
- or scene-relative mesh paths such as `/scenes/3d/.../*.obj`

Runtime loader:

- `engine/src/systems/compositor/obj_loader.rs`

Current behavior:

- OBJ bytes are loaded through the shared source abstraction
- the OBJ parser may resolve `mtllib` side files relative to the OBJ source
- parsed meshes are cached and then rendered as terminal wireframes

### 5.4 YAML-authored assets/prefabs

Not every "asset" in the practical sense is binary.

These YAML-authored things behave like project assets too:

- `objects/*.yml`
- `scenes/<scene>/layers/*.yml`
- `scenes/<scene>/templates/*.yml`
- `scenes/<scene>/objects/*.yml`

They are not raw render assets, but they are still reusable authored resources consumed by scene assembly and runtime compilation.

## 6) Sprite types

Sprites are defined in:

- `engine-core/src/scene/sprite.rs`
- `schemas/scene.schema.yaml`

Current sprite families:

- `text`
- `image`
- `obj`
- `grid`
- `flex`

### 6.1 Text sprite

Purpose:

- terminal-native text
- or rasterized text when `font` is set

Key fields:

- `content`
- `font`
- `fg`, `bg`
- `align_x`, `align_y`
- `reveal_ms`
- `glow`
- `force-renderer-mode`
- `force-font-mode`

### 6.2 Image sprite

Purpose:

- display an image asset on the terminal grid

Key fields:

- `source`
- `width`
- `height`
- `size`
- `stretch-to-area`
- `force-renderer-mode`

Important runtime fact:

- image rendering is cell-mode aware and can render in `cell`, `halfblock`, `quadblock`, or `braille`
- image alpha is thresholded, not blended, so semi-transparent overlays usually need dithering or precomposed assets

### 6.3 Obj sprite

Purpose:

- render a Wavefront OBJ mesh as terminal wireframe/surface

Key fields:

- `source`
- `scale`
- `width`, `height`
- `yaw-deg`, `pitch-deg`, `roll-deg`
- `rotation-x`, `rotation-y`, `rotation-z`
- `surface-mode`

### 6.4 Grid sprite

Purpose:

- container layout with explicit tracks

Key fields:

- `columns`
- `rows`
- `gap-x`, `gap-y`
- `children`

### 6.5 Flex sprite

Purpose:

- container layout that stacks children in a row/column

Key fields:

- `direction`
- `gap`
- `children`

## 7) Which sprites consume external assets

Asset-backed sprite types today:

- `image` -> image file bytes
- `obj` -> OBJ text + optional MTL side files
- `text` -> only when using named bitmap fonts under `assets/fonts`

Pure authored/container sprite types:

- `grid`
- `flex`

## 8) Objects: what they are and what they can contain

Objects are reusable YAML prefabs.

Schema:

- `schemas/object.schema.yaml`

Example:

- any object file under `mods/<mod>/objects/*.yml`

An object can define:

- `name`
- `exports`
- `state`
- `logic`
- `sprites`
- `layers`

That means an object is not "just one sprite".
It can be:

- a small visual prefab
- a multi-layer composition
- a logic-bearing reusable entity

### Example

An object can expand into multiple layers and sprites, not only one node.
That is a good example of the intended meaning of "object":

- reusable
- parameterized by `exports`
- able to expand into multiple render elements

### Important usage model

From the current stable project model, scene-level object refs are the supported baseline.

You should think of objects as:

- reusable scene building blocks
- optionally scene-local via package structure
- expanded by authoring/runtime compilation rather than loaded as separate runtime file handles

## 9) Scene-centric authoring relationship

The project direction is scene-centric authoring:

- scene chooses composition order
- layer owns visual composition
- object is a reusable prefab

Reference:

- `docs/scene-centric-authoring.md`

This is important because "assets" in Shell Quest are not only files in `assets/`.
Reusable YAML building blocks are also part of the authoring asset system.

## 10) Packaging and discovery

Mods can be loaded from:

- directory
- `.zip`

The same authored paths should keep working in both modes.

That is a core invariant of the system.

Scene package assembly merges:

- root `scene.yml`
- `layers/`
- `templates/`
- `objects/`

Reference points:

- `engine/src/repositories.rs`
- `engine-authoring/src/package/mod.rs`

Important caveat:

- `scenes/<name>/effects/` is reserved in discovery/schema discussions, but scene package assembly does not currently auto-merge it the way layers/templates/objects are merged

## 11) Runtime loading pipeline

The practical loading flow today is:

1. authored YAML references a `source`
2. runtime builds a `SourceRef`
3. `ModAssetSourceLoader` reads bytes through the asset repository
4. a typed adapter decodes bytes into a runtime structure
5. typed result is cached
6. compositor/rasterizer consumes the decoded result

Examples:

- image path -> `engine/src/image_loader.rs`
- OBJ path -> `engine/src/systems/compositor/obj_loader.rs`
- font assets -> `engine/src/rasterizer/font_loader.rs`

## 12) Format support today

### Images

Current image support is effectively tied to what is enabled in `engine/Cargo.toml`.

At the moment:

- the engine `image` crate is built with `png` and `gif` support enabled
- remote URLs are not supported at runtime
- additional image formats should be introduced via adapters, not ad-hoc conditionals

So if you are asking "do PNG/JPEG/GIF/etc all work right now?" the correct answer is:

- **PNG and GIF are the currently supported authored image formats**
- GIF works as an animated `image` sprite rendered through the same terminal pixel-buffer pipeline as static images
- broader format support should be added through the new source/adapter architecture

### Fonts

Current manifest schema supports:

- `terminal-pixels`
- `ascii`

Runtime loader also treats `raster` and `cell` as aliases/preferences when resolving manifests, but the project-level canonical structure should still follow the manifest/layout used in the repo.

### OBJ

Current mesh support is:

- Wavefront OBJ
- optional MTL side files

### glTF / GLB

Current mesh support does **not** include glTF/glb.

That means:

- `type: obj` is the current 3D authoring path
- `scene.gltf` files are not directly renderable by runtime today
- using glTF content requires either offline conversion, prerendered images, or new engine work

## 13) Reflection and compositing constraints

Some content ideas sound simple at YAML level but are actually renderer features.

Today the engine does **not** have a dedicated pipeline like:

`3D model -> offscreen framebuffer -> warped reflection texture -> scene composite`

So "reflection" in current authored scenes is typically faked via:

- bitmap overlays
- dithered semi-transparency
- blur/distortion effects
- positioning a sprite/object where the reflection should appear

For practical authoring this means:

- smooth half-transparent CRT overlays should usually be precomposed or dithered
- reflective 3D content should usually be authored as OBJ or prerendered PNG, not expected from glTF + postprocess magic
- if a design truly needs dynamic projected reflections, that should be tracked as an engine feature, not hidden in scene YAML

## 14) What to preserve when extending assets

If you add new asset kinds, source kinds, or tool support, preserve these invariants:

- authored asset paths remain portable across directory and zip sources
- cache keys stay stable per mod source + asset identity
- scene/layer/object YAML compatibility is preserved
- renderer order and sprite/layer sorting remain stable
- virtual buffer behavior stays unchanged
- object expansion stays deterministic and cycle-safe
- schema-gen, runtime, editor, and `devtool` should converge on the same authoring model

## 15) Recommended extension strategy

If you want to extend assets the clean way:

1. add/extend a source abstraction, not a one-off loader
2. add a typed adapter/decoder for the new payload
3. keep authored references explicit and portable
4. update schema hints and validation
5. update `devtool`
6. update editor affordances

This is the same metadata-first direction described in:

- `authoring.md`

## 16) Practical authoring examples

### Image sprite

```yaml
- type: image
  id: logo
  source: /assets/images/logo.png
  at: cc
  width: 24
  force-renderer-mode: halfblock
```

### Text sprite using built-in generic font

```yaml
- type: text
  id: title
  content: "SHELL QUEST"
  at: ct
  font: "generic:1"
  fg: white
```

### Text sprite using bitmap font assets

```yaml
- type: text
  id: title
  content: "SHELL QUEST"
  at: ct
  font: "Abril Fatface:ascii"
  fg: white
```

### OBJ sprite

```yaml
- type: obj
  id: demo
  source: /scenes/3d/helsinki-university/city_scene_horizontal_front_yup.obj
  width: 64
  height: 24
  rotation-y: 180
```

### Object prefab

```yaml
name: ui-title
exports:
  text: DEFAULT
sprites:
  - type: text
    content: "$text"
    at: cc
    font: "generic:1"
    fg: white
```

## 16) Cleanup note

No mandatory cleanup is needed before documenting this area.

The main gap was documentation, not an urgent structural fix.

The runtime source foundation is now clean enough to document honestly:

- current behavior is stable
- limitations are explicit
- future extension points are now real rather than hypothetical
