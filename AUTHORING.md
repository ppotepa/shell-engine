# Content Authoring Guide

Shell Quest content is metadata-driven. Effect metadata, scene schemas, CLI
tools, and the TUI editor all derive from a single source of truth defined in
`engine-core`. When metadata is correct, everything else follows.

Current metadata maturity:

| Area    | Coverage |
|---------|----------|
| Effects | ~80%     |
| PostFX  | ~70%     |
| Overall | ~45-55%  |

---

## Mod Structure

```
mods/<mod>/
+-- mod.yaml                 Mod manifest
+-- levels/                  Level payloads (*.yml|*.yaml|*.json)
+-- audio/
|   +-- sfx.yaml             Semantic SFX bank (events -> cue variants)
|   +-- songs/               Sequenced song files (*.yml|*.yaml)
|   +-- synth/               Note-sheet cue definitions (*.yml|*.yaml)
+-- assets/
|   +-- images/              Image assets (PNG, GIF)
|   +-- fonts/               Rasterized font manifests
|   +-- 3d/                  OBJ/MTL meshes
|   +-- raw/                 Local staging (gitignored)
+-- objects/                 Reusable prefabs (*.yml)
+-- stages/                  Reusable stage presets
+-- behaviors/               Mod-level behaviors (`script:` inline or `src:` external Rhai)
+-- scenes/
    +-- <scene>.yml          Single-file scene
    +-- <scene>/             Scene package
        +-- scene.yml
        +-- layers/*.yml
        +-- templates/*.yml
        +-- objects/*.yml
        +-- behaviors/*.yml
```

A scene is either a single YAML file or a package directory containing
`scene.yml` plus partials. Both forms are interchangeable at runtime.

Object instances support both explicit entries and a repeat shorthand. The
`repeat` form expands at compile time and supports `{i}` token substitution
in `as`/`id` and string values inside `with`.

```yaml
objects:
  - repeat:
      count: 8
      ref: bullet-vector
      as: bullet-{i}
      with:
        id: bullet-{i}
        x: 0
        y: 0
```

---

## Asset System

Mental model (each level builds on the previous):

```
Asset (file data)
  -> Sprite (drawable node)
    -> Object (reusable prefab)
      -> Layer (visual slice)
        -> Scene (flow + composition)
```

Asset paths use a leading `/` and resolve relative to the mod root. The same
paths work for both unpacked directories and zip-packaged mods.

### Asset Categories

| Category     | Location           | Loader              | Notes                          |
|--------------|--------------------|----------------------|--------------------------------|
| Images       | assets/images/     | image_loader.rs      | PNG, GIF (animated), static    |
| Fonts        | assets/fonts/      | font_loader.rs       | Manifest-based, generic:* built-in |
| OBJ meshes   | assets/3d/ or scenes/ | obj_loader.rs     | Wavefront OBJ + MTL           |
| YAML prefabs | objects/, layers/  | engine-authoring     | Reusable authored resources    |

`mod.yaml` can define `terminal.default_font`. Then any text sprite using
`font: "default"` resolves to that spec; if unset, engine fallback generic is used.

Audio authoring is mod-root based:
- `audio/sfx.yaml` defines semantic events such as `ui.menu.select`
- `audio/songs/*.yml` defines sequenced tracks/patterns
- `audio/synth/*.yml` defines generated cues; the engine synthesizes these into in-memory buffers at startup

---

## Sprite Types

### Core Types

| Type  | Purpose             | Key Fields                                      | Asset-backed? |
|-------|---------------------|-------------------------------------------------|---------------|
| text  | Terminal/raster text | content, font, fg, bg, reveal_ms, glow          | Only with named fonts |
| image | Display image       | source, width, height, stretch-to-area           | Yes           |
| obj   | 3D mesh render      | source, scale, yaw/pitch/roll, surface-mode      | Yes           |
| grid  | Layout container    | columns, rows, gap-x/y, children                 | No            |
| flex  | Stack container     | direction, gap, children                         | No            |

### Sugar Types

These compile down to core types during the authoring pipeline:

| Sugar          | Compiles To  | Purpose                                 |
|----------------|--------------|------------------------------------------|
| window         | panel        | UI window with title/body/footer slots   |
| terminal-input | window       | Prompt widget with hint/input slots      |
| scroll-list    | grid         | Scrollable list with menu-carousel       |
| frame-sequence | timed images | Stop-motion animation                    |

---

## Scene Contract

A `scene.yml` controls the following concerns:

| Concern     | Fields                                                    |
|-------------|-----------------------------------------------------------|
| Identity    | id, title                                                 |
| Lifecycle   | stages, stages-ref                                        |
| Composition | layers (ordered list of visual slices)                    |
| PostFX      | postfx (ordered list of post-processing passes)           |
| UI          | ui.enabled, ui.persist, ui.theme, ui.focus-order          |
| Routing     | next, menu-options (each with `to`)                       |
| Input       | input profiles (terminal-shell, menu, custom)             |
| Prerender   | prerender hooks                                           |

---

## PostFX Pipeline

PostFX passes execute after the compositor and before terminal flush.
Order matters — passes apply sequentially to the composited buffer.

| Pass            | Purpose                           |
|-----------------|-----------------------------------|
| crt-underlay    | Soft glow under content           |
| crt-distort     | Tube curvature + margins          |
| crt-scan-glitch | Scanline sweep + chroma glitch    |
| crt-ruby        | Ruby tint + edge reveal           |
| terminal-crt    | Legacy alias                      |

---

## OBJ Lighting

Scenes can define directional lights (primary + secondary), point lights, and
cel shading for 3D objects.

### Light Types

| Type        | Fields                        | Behavior                        |
|-------------|-------------------------------|---------------------------------|
| Directional | direction, color, intensity   | Infinite parallel rays          |
| Point       | position, color, radius       | Orbit or snap animation         |
| Cel shading | steps, edge-threshold         | Posterized shading bands        |

### Point Light Animation

| Mode     | Field    | Behavior                                      |
|----------|----------|-----------------------------------------------|
| Snap     | snap-hz  | Instant position jumps (deterministic hash)   |
| Orbit    | orbit-hz | Smooth continuous rotation                    |
| Static   | (none)   | Fixed position                                |

Priority: snap > orbit > static. When `snap-hz` is set, `orbit-hz` is ignored.

---

## Terminal HUD Authoring

### Window

`type: window` compiles to a panel with three slots: title, body, footer.
Slot layout respects the active font height for vertical sizing.

### Terminal Input

`type: terminal-input` is a specialized window for interactive prompts.

### Shell Input Profile

The `input.terminal-shell` section binds a shell prompt to sprites:

| Field           | Purpose                                    |
|-----------------|--------------------------------------------|
| prompt-sprite-id | Sprite displaying the prompt text         |
| output-sprite-id | Sprite displaying command output          |
| prompt-panel-id  | Panel containing the prompt               |
| prompt-wrap      | Enable line wrapping in prompt            |
| prompt-auto-grow | Panel grows with input length             |

In scripted mode the engine skips built-in commands entirely; Rhai handles
all input processing and output rendering.

### Action Map

The `action_map` section in `mod.yaml` defines named input actions that scripts can query:

```yaml
action_map:
  actions:
    move_left:
      key: a
      repeat: true
    move_right:
      key: d
      repeat: true
    jump:
      key: space
      repeat: false
```

Action names must be valid identifiers (start with letter or `_`, contain only alphanumeric or `_`).
The `key` property is required and specifies the input key code.
The optional `repeat` property (default `false`) indicates whether the action repeats while held.

---

## Rhai Scripting

### Scope Variables

| Variable   | Contents                                  |
|------------|-------------------------------------------|
| menu.*     | Menu state (index, items, selection)      |
| time.*     | Elapsed time, delta, stage progress       |
| params     | Effect/behavior parameters                |
| regions    | Named regions from layout                 |
| objects    | Object instances in the scene             |
| state      | Persistent key-value state                |
| ui         | UI state (focus, visibility)              |
| game       | Global game state                         |
| level      | Active level payload + level catalog      |
| world      | Gameplay entity world                     |
| key        | Current key event                         |
| collisions | Collision hits for the current frame      |

Gameplay helpers on `world` (component-backed):
- `world.set_transform(id, x, y, heading_rad)` and `world.transform(id)`
- `world.set_physics(id, vx, vy, ax, ay, drag, max_speed)` and `world.physics(id)`
- `world.set_collider_circle(id, radius, layer, mask)`
- `world.set_lifetime(id, ttl_ms)`
- `world.set_visual(id, visual_id)` to bind scene runtime target for cleanup on entity expiration
- `world.collisions()` → array of `{ a, b }` entity id pairs detected this frame

### Commands

Scripts emit commands to mutate the scene:

- Visibility: show/hide sprites and layers
- Set-text: update sprite content
- Position: move sprites
- Style: change fg/bg/font/glow

### Object API

```
scene.get(target)              // read a value
scene.set(target, path, value) // write a value
scene.spawn_object(template, target)   // clone a scene object/layer template at runtime
scene.despawn_object(target)           // soft-despawn (hide) a scene object/layer
audio.cue(cue_id)              // play direct cue id (asset stem)
audio.cue(cue_id, volume)      // play direct cue id with volume scale
audio.event(event_id)          // play semantic sfx event from audio/sfx.yaml
audio.event(event_id, gain)    // semantic event with gain scale
audio.play_song(song_id)       // start sequenced song from audio/songs/*.yml
audio.stop_song()              // stop currently active sequenced song

game.get(path)                 // read global game state
game.set(path, value)          // write global game state
persist.get(path)              // read on-disk persistent state
persist.set(path, value)       // write on-disk persistent state
level.current()                // active level id
level.ids()                    // available level ids
level.select(level_id)         // switch active level
level.get(path)                // read active level payload

world.spawn_object(kind, payload) // create a gameplay entity and return its id
world.despawn_object(id)       // remove a gameplay entity
world.clear()                  // remove all gameplay entities
world.count()                  // number of gameplay entities
world.count_kind(kind)         // count entities by kind
world.count_tag(tag)           // count entities by tag
world.first_kind(kind)         // first entity id by kind
world.first_tag(tag)           // first entity id by tag
world.exists(id)               // check if an entity exists
world.kind(id)                 // read entity kind
world.tags(id)                 // read entity tags
world.ids()                    // list gameplay entity ids
world.query_kind(kind)         // find ids by kind
world.query_tag(tag)           // find ids by tag
world.get(id, path)            // read entity payload by JSON pointer
world.set(id, path, value)     // write entity payload by JSON pointer
world.has(id, path)            // test entity payload path
world.remove(id, path)         // delete entity payload path
world.push(id, path, value)    // append entity payload path
world.entity(id)               // entity handle API for cleaner repeated access

// entity handle API
entity.exists()                // check entity existence
entity.get(path)               // read path
entity.get_i(path, fallback)   // read integer with fallback
entity.get_bool(path, fallback)// read bool with fallback
entity.set(path, value)        // write path

// Math / geometry helpers (engine-provided, useful for gameplay scripts)
abs_i(v)                       // absolute value for integers
sign_i(v, fallback)            // sign with fallback when v == 0
clamp_i(v, min_v, max_v)       // clamp integer
wrap(v, min_v, max_v)          // toroidal integer wrap
wrap_fp(v, min_v, max_v, scale)// toroidal fixed-point wrap
wrap_heading32(v)              // wrap heading to [0, 31]
rng_next_i(seed, modulus)      // deterministic integer RNG helper

// Geometry helpers
sin32(idx)                     // 32-step integer sine lookup (-1024..1024)
ship_points(heading)           // ship polygon points for heading [ [x,y], ... ]
asteroid_points(shape, size)   // asteroid polygon points for shape/size
rotate_points(points, heading) // rotate a point array around 0,0 using 32-step heading
asteroid_fragment_points(shape, size, fragment_idx) // one of three closed asteroid wedges
asteroid_radius(size)          // helper radius by asteroid size tier
asteroid_score(size)           // score value by asteroid size tier
```

Mod-level named behaviors can live in `behaviors/*.yml` and reference external Rhai:

```yaml
kind: behavior
name: asteroids-game-loop
src: ./asteroids-game-loop.rhai
```

Use `script:` for short inline behaviors and `src:` for reusable or larger scripts.

**Important:** Always use backtick strings for multiline text in Rhai:

```rhai
// correct
let msg = `line one
line two`;

// wrong — do not use escaped newlines
let msg = "line one\nline two";
```

---

## Compilation Pipeline

```
1. Load       scene YAML (single-file or package)
       |
2. Expand     refs, objects, stages-ref, cutscene-ref (engine-authoring)
       |
3. Normalize  expand aliases and shorthands
       |
4. Deserialize  into runtime Scene struct
       |
5. Validate   timeline checks (debug mode)
       |
6. Execute    lifecycle -> input -> compositor -> postfx -> render
```

---

## Author Checklist

1. Every YAML file has a correct `$schema` reference.
2. `next` and each `menu-options[].to` point to existing scenes.
3. All `ref` / `use` references resolve to valid targets.
4. `./refresh-schemas.sh` and `cargo run -p schema-gen -- --all-mods --check` pass.
5. Sprite timing falls within scene duration.
6. A smoke run (`cargo run -p app`) starts without compile errors.
7. `cargo run -p app -- --mod-source=mods/<mod> --check-scenes` reports zero warnings before merge.

---

## Daily Workflow

1. Edit YAML files under `mods/<mod>/`.
2. Run `./refresh-schemas.sh` to regenerate schemas.
3. Continue authoring — editor completions reflect the updated schemas.
4. Run `cargo run -p schema-gen -- --all-mods --check` before merge.
