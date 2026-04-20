# Vehicle Playground

Standalone sandbox for tuning the vehicle domain around a generated sphere.

The mod keeps the same core loop as the current planet handoff flow:

- generated body patched into the celestial runtime,
- controlled ship starting grounded on the rotating surface and then flying in a local horizon frame,
- compact vehicle HUD with body, telemetry, assist/profile state, and an on-screen control cheat sheet.

The mod boots directly into the vehicle runtime when started standalone, but it can
also consume the canonical cross-mod vehicle launch packet produced by `planet-generator` at
`/mods/planet-generator/vehicle/handoff`.

## Running

```bash
cargo run -p app -- --mod-source=mods/vehicle-playground
```

## Controls

| Key / Input | Action |
|-------------|--------|
| `W` / `S` or `Up` / `Down` | Forward / reverse thrust on the local horizon |
| `A` / `D` or `Left` / `Right` | Yaw left / right around the local up vector |
| `Q` / `E` | Lateral strafe left / right in the tangent frame |
| `Space` | Lift away from the surface; the ship starts grounded and co-rotating with the planet |
| `X` | Main engine boost |
| `H` | Toggle altitude hold |
| `J` | Toggle heading hold |
| `F9` / `C` | Toggle vehicle profile: `arcade` / `sim-lite` |
| `Esc` | Return to the producer scene when launched from a handoff packet; otherwise reset ship to the default spawn |

## Producer handoff

- `planet-generator` is the current producer for this scene.
- The consumer reads the producer packet on scene boot, normalizes it through
  `engine-vehicle`, hydrates the generated planet render state from the packet
  environment, restores profile/assist state from `vehicle`, restores
  motion/camera state from `telemetry`, then clears the consumed packet paths.
- On `Esc`, if the packet carried a `return_scene_id`, the vehicle scene writes
  a canonical vehicle return packet back to the producer handoff path with the
  same vehicle-domain payloads, then jumps cross-mod back to the producer mod + scene.
  Without a packet, `Esc` stays a local reset only.
- `engine-vehicle` owns launch/return packet kind, version, telemetry basis,
  and compatibility normalization; this mod stays a consumer/returner of
  vehicle-domain state rather than co-defining the packet wire shape.
- `planet-generator` owns the target through:
  - `/mods/planet-generator/vehicle/target_mod_ref`
  - `/mods/planet-generator/vehicle/target_scene_id`

## Layout

- `catalogs/celestial/*.yaml` defines one local generated body and system.
- `catalogs/prefabs.yaml` and `objects/ship.yml` define the controlled ship.
- `scenes/vehicle/scene.yml` is the mod entrypoint.
- The package uses scene id `vehicle-playground-vehicle` as the canonical
  producer/consumer target.
- `scenes/vehicle/main.rhai` is now a thin orchestrator that wires bootstrap,
  body sync, flight step, HUD, and return flow.
- `scripts/std/` holds shared Rhai helpers:
  - `math3.rhai` for shared angle/orientation math
  - `runtime_scene.rhai` for the small scene-object write seam
- `scripts/vehicle/` holds the domain split:
  - `state.rhai` owns bootstrap and `local -> local.state` persistence
  - `control.rhai` owns control/profile/assist persistence and input shaping
  - `environment.rhai` owns body patching and planet render pushes
  - `handoff.rhai` owns launch/return packet consume/build helpers
  - `flight.rhai` owns the local-horizon ship step and camera rig
  - `hud.rhai` owns HUD formatting and writes
- `scenes/vehicle/layers/*.yml` define the sphere view, vehicle slot, and HUD.
