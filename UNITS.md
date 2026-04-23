# Units

This is the project-wide unit contract for Shell Engine.

## Spaces

1. `screen_px`
- Final output/window pixels.
- UI and presentation only (independent of software or hardware backend).

2. `virtual_px`
- Authored 2D canvas space.
- Used by composition and output scaling across backend paths.

3. `world_units` (`wu`)
- Runtime gameplay/scene space.
- Shared by 2D and 3D runtime systems.

4. `physical` (`m` / `km`)
- Celestial and physics domain values.
- Used for gravity, planet radius, altitude, atmosphere, orbits.

## Canonical Scale

- `meters_per_world_unit` is the primary conversion.
- Optional: `virtual_pixels_per_world_unit` for 2D projection policy.

Conversions:
- `wu <-> m`
- `wu <-> km`
- optionally `wu <-> virtual_px`

## Ownership

- `engine-core::spatial`: shared types and conversion helpers.
- `engine-scene-runtime`: active per-scene `SpatialContext`.
- `engine-authoring`: compiles `scene.spatial` authored policy.
- `engine-game`: simulation in `wu`.
- `engine-celestial`: physical values and resolver mapping through scale.
- render crates consume resolved world-space values; they do not own physical semantics.

## Authoring

Scene-level block:

```yaml
spatial:
  meters-per-world-unit: 2000.0
  virtual-pixels-per-world-unit: 1.0
  handedness: right
  up-axis: y
```

## Rules

- Do not mix `screen_px` with physics/celestial math.
- Do not hardcode one global `1 unit = 1 meter` assumption for all scenes.
- New physics/celestial features should use shared resolvers, not local conversion formulas.
