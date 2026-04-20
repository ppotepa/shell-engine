# Vehicle Playground Flight Model Implementation Brief

## Goal

Refactor `mods/vehicle-playground` so the vehicle feels like a **3D lunar-lander-style craft** launching from and flying around a small rotating planet, not like a ship that is spawned directly into an orbital tangent frame.

The desired feel is:

- start **stationary on the surface**
- ship is **upright relative to the local surface normal**
- press `lift` to **take off vertically**
- hover and translate in a readable local-horizon frame
- fly low around the rotating planet
- land deliberately and cleanly
- only later transition into more detached/free-flight feel

This should feel closer to a classic **2D lunar lander**, but in **3D around a small planet**.

## Repository Reality

The current repo already contains most of the pieces needed.

### What already exists

- `mods/vehicle-playground/scripts/vehicle/state.rhai`
  - `default_state(vehicle)` already starts with:
    - `vehicle_grounded: true`
    - `vehicle_heading: 0.0`
    - `spawn_altitude_km: 0.0`
    - `vehicle_speed_kms: 0.0`
    - `vehicle_vrad_kms: 0.0`
- `engine-vehicle` already exposes typed runtime attachment modes:
  - `Grounded`
  - `SurfaceLocked`
  - `Detached`
- typed runtime input already supports:
  - `surface_contact`
  - `request_detach`
  - `prefer_grounded_on_contact`

### What is wrong today

- `mods/vehicle-playground/scripts/vehicle/flight.rhai`
  - `seed_vehicle_orbit_state(...)` seeds the craft into tangent/orbit orientation
  - the default step path calls this seeding on first boot
  - the non-packet reset path calls it again
  - grounded behavior mostly freezes motion and waits for lift, but the overall frame logic still inherits orbit-style basis assumptions

So the ship starts with surface defaults in state, but the flight script immediately reinterprets the craft as orbit-framed from frame one.

## Flight Model Definition

For this task, the flight model is the combination of:

1. state machine
2. reference-frame policy
3. control meaning per phase
4. transition rules
5. camera behavior
6. HUD phase reporting

The target model is:

- `PAD_GROUNDED`
- `SURFACE_HOVER`
- `FREE_FLIGHT`
- `ORBITAL_FEEL`

`ORBITAL_FEEL` is not a new hard physics mode in pass one. It is mainly a high-altitude presentation/tuning regime layered on top of free flight.

## Target Behavior

### PAD_GROUNDED

- spawn state and landing state
- ship rests at `surface_radius + clearance`
- `vfwd = 0`
- `vright = 0`
- `vrad = 0`
- body up-axis follows local surface normal
- forward axis is tangent, derived from heading
- yaw rotates around the surface normal only
- no orbital/carrier drift is injected into local motion identity
- HUD label: `PAD`
- camera: close, stable, low sway

### SURFACE_HOVER

- first airborne state after lift
- runtime mode should resolve to `SurfaceLocked`
- local up remains aligned with the surface normal
- local forward/right remain tangent to the planet
- `lift` controls radial authority
- `thrust` and `strafe` move on the tangent plane
- planet rotation is real, but should read as world motion below the craft
- HUD label: `HOVER`
- camera remains readable and surface-biased

### FREE_FLIGHT

- begins after meaningful altitude/speed or explicit detach intent
- less constrained motion
- detached-like semantics
- less stabilization
- HUD label: `FLIGHT`

### ORBITAL_FEEL

- high-altitude/high-speed presentation regime
- no special spawn behavior
- more distant camera
- less visible surface readability bias
- HUD label: `ORBIT`

## High-Level Design Rules

- Do **not** rewrite the whole engine.
- Solve this in `mods/vehicle-playground` first.
- Reuse `Grounded / SurfaceLocked / Detached`.
- Do **not** replace typed runtime semantics with hidden script hacks.
- Keep packet/handoff restore behavior explicit and intact.
- Prefer readable control feel over “perfect” realism.

## In Scope

- `mods/vehicle-playground/scripts/vehicle/flight.rhai`
- `mods/vehicle-playground/scripts/vehicle/state.rhai`
- `mods/vehicle-playground/scripts/vehicle/hud.rhai`
- phase-aware camera behavior
- grounded/hover/free-flight transitions
- optional small typed tuning additions if absolutely necessary

## Out Of Scope

- full aerospace simulation
- giant physics rewrite
- rewriting all other mods
- replacing existing typed vehicle runtime architecture

## Full Implementation Plan

### Step 1. Freeze semantics in the script

At the top of `mods/vehicle-playground/scripts/vehicle/flight.rhai`, define the meaning of:

- `PAD_GROUNDED`
- `SURFACE_HOVER`
- `FREE_FLIGHT`
- `ORBITAL_FEEL`

Add a script-owned phase field:

- `state.vehicle_phase = "pad" | "hover" | "flight" | "orbit"`

Important:

- this does **not** replace typed runtime `surface_mode`
- typed runtime continues to own `Grounded / SurfaceLocked / Detached`
- script-owned phase is for:
  - coordination
  - camera
  - HUD
  - human-facing transition policy

### Step 2. Replace orbit seeding with surface seeding

In `flight.rhai`, split the current default seeding behavior into:

- `seed_vehicle_surface_state(...)`
- packet/handoff restore path, which remains separate

`seed_vehicle_surface_state(...)` should:

- read `vehicle_spawn_angle_deg`
- derive local normal from spawn angle
- derive tangent forward from the normal
- rotate forward/right around the normal by `vehicle_heading`
- set `radius = surface_radius + surface_clearance_wu`
- set `vfwd = 0`
- set `vright = 0`
- set `vrad = 0`
- set `yaw_rate = 0`
- set `vehicle_grounded = true`
- initialize camera basis from surface pose
- set `vehicle_phase = "pad"`

Do **not** break packet restore:

- default boot uses surface seeding
- local manual reset uses surface seeding
- handoff/packet restore keeps restored basis/runtime data

### Step 3. Remove default orbit seeding from the normal boot path

Today the top-level step does:

- body context
- `if !vehicle_orbit_seeded -> seed_vehicle_orbit_state(...)`

Change this to:

- if packet restore already provided a valid basis/state, keep it
- otherwise use `seed_vehicle_surface_state(...)`

Do the same in the Escape reset path:

- reset must return the player to `PAD_GROUNDED`
- it must **not** bounce the craft back into tangent/orbit framing

### Step 4. Split `flight.rhai` into explicit phase steps

Refactor the monolithic flow into:

- `resolve_vehicle_phase(...)`
- `step_grounded(...)`
- `step_surface_hover(...)`
- `step_free_flight(...)`
- `sync_phase_camera(...)`
- `sync_phase_hud_state(...)`
- keep one public `step(...)`

Do **not** change `main.rhai` structure unless necessary. `main.rhai` already delegates through `flight::step(...)`.

### Step 5. Implement `step_grounded(...)`

Responsibilities:

- clamp radius to `surface_radius + clearance`
- zero `vfwd`, `vright`, `vrad`
- yaw only
- rotate forward/right around surface normal by yaw input
- keep target altitude at `0`
- no orbital/carrier drift in local motion identity
- update ship transform to surface clearance
- sync runtime with:
  - `surface_contact = true`
  - `prefer_grounded_on_contact = true`
  - `request_detach = false`

Transition rule:

- if `lift > takeoff_lift_threshold`
  - set `vehicle_phase = "hover"`
  - set `vehicle_grounded = false`
  - do not inject fake tangential drift

### Step 6. Implement `step_surface_hover(...)`

This is the most important phase.

Responsibilities:

- preserve local-horizon basis
- preserve local surface normal as up
- forward/strafe move on tangent plane
- lift controls radial authority
- reuse existing altitude-hold and heading-hold logic
- keep strong low-altitude readability

Runtime sync for hover:

- `surface_contact = true` while near ground
- `prefer_grounded_on_contact = false`
- `request_detach = false`

That should naturally make typed runtime report `SurfaceLocked`.

Landing rule from hover:

- altitude <= surface-contact threshold
- `abs(vrad)` <= grounded threshold
- tangent speed <= grounded threshold
- lift not currently requested

Then:

- `vehicle_phase = "pad"`
- `vehicle_grounded = true`
- zero drift
- snap only to surface clearance
- do not reseed into orbit basis

### Step 7. Keep planet rotation, but stop using it as spawn identity

The existing surface spin/carrier math is useful and should remain.

Keep:

- planet rotation
- carrier/world movement
- basis advancement around the planet

Change the feel:

- in `PAD_GROUNDED`, no local drift identity
- in `SURFACE_HOVER`, world rotation is present but readable
- in `FREE_FLIGHT`, carrier/inertial effects can dominate more

The player should feel:

1. I launched from the pad
2. I am hovering/flying around a rotating planet

not:

1. I spawned already moving like I am in orbit

### Step 8. Implement `step_free_flight(...)`

This is where most of the current open-flight behavior can live.

Entry conditions:

- altitude above hover exit threshold
- or speed above hover exit threshold
- or explicit detach / inertial intent

Runtime sync:

- use `request_detach = true` when explicitly transitioning to detached behavior
- or stop reporting `surface_contact` once clearly out of the surface regime

Do not over-polish this first. The first pass should prioritize:

- pad
- hover
- landing

### Step 9. Centralize transition policy

Do not let transition policy leak through random side effects.

Recommended rules:

- `PAD -> HOVER`
  - lift above takeoff threshold

- `HOVER -> PAD`
  - near surface
  - low radial speed
  - low tangent speed
  - no lift request

- `HOVER -> FLIGHT`
  - altitude above hover exit threshold
  - or speed above hover exit threshold
  - or explicit detach intent

- `FLIGHT -> ORBIT`
  - mostly label/camera regime in pass one

- optional later:
  - `FLIGHT -> HOVER`
    - descending into low-altitude, low-speed, surface-readable regime

### Step 10. Make camera phase-aware

Current camera logic is a good base, but is too altitude-only.

Implement:

- `PAD` camera
  - smallest pullback
  - smallest lift
  - minimal sway
  - strong local-up stability

- `HOVER` camera
  - slightly more pullback
  - modest smoothing
  - still strongly surface-readable

- `FLIGHT` camera
  - can use most current behavior

- `ORBIT` camera
  - longest pullback
  - least surface readability bias

Phase should matter more than altitude alone.

### Step 11. Fix HUD mode reporting

The current HUD uses heuristics like:

- `SURFACE`
- `LOCKED`
- `ORBIT`
- `CLIMB`
- `DECAY`
- `CRUISE`

For this refactor, phase reporting should instead use the script-owned phase:

- `PAD`
- `HOVER`
- `FLIGHT`
- `ORBIT`

Keep existing telemetry lines:

- altitude
- speed
- radial speed
- profile
- assists
- body type
- gravity
- sway

### Step 12. Preserve current control bindings

Do not invent a new input model in pass one.

Keep:

- thrust/brake
- yaw
- strafe
- boost
- lift
- assist logic already routed through `control.rhai`

Only change what those controls mean in each phase.

### Step 13. Keep handoff semantics explicit

This rule is critical:

- default scene boot -> surface seed
- manual local reset -> surface seed
- packet/handoff restore -> restore packet basis/runtime and normalize into new phase system

Do not destroy legitimate handoff state by forcing packet return through default surface seed.

## File-By-File Rewrite Outline

### `mods/vehicle-playground/scripts/vehicle/state.rhai`

Add and maintain:

- `vehicle_phase: "pad"` in `default_state(vehicle)`

Keep:

- grounded/stationary defaults
- current handoff fields
- current world/body defaults

Do not overcomplicate state here. This file should own defaults, not physics.

### `mods/vehicle-playground/scripts/vehicle/flight.rhai`

This is the primary file to change.

Recommended function structure:

1. `seed_vehicle_surface_state(state, surface_radius, km_per_wu, surface_clearance_wu)`
2. `seed_vehicle_packet_state(...)` or keep packet restoration outside if already clean
3. `resolve_vehicle_phase(state, body_snapshot, ship_tuning, altitude_km, tangent_speed, radial_speed, detach_intent)`
4. `step_grounded(...)`
5. `step_surface_hover(...)`
6. `step_free_flight(...)`
7. `sync_phase_camera(...)`
8. `sync_phase_hud_state(...)`
9. `step(...)`

### `mods/vehicle-playground/scripts/vehicle/hud.rhai`

Change mode label logic to trust `state.vehicle_phase`.

Suggested output mapping:

- `"pad"` -> `PAD`
- `"hover"` -> `HOVER`
- `"flight"` -> `FLIGHT`
- `"orbit"` -> `ORBIT`

### `mods/vehicle-playground/scripts/vehicle/control.rhai`

Likely no large change needed.

Only touch if you want:

- an explicit detach intent
- phase-specific control helpers

### Optional engine-side follow-up

Only if script-side thresholds are clearly not enough:

- `engine-vehicle/src/input.rs`
- `engine-vehicle/src/models/ship.rs`
- `engine-vehicle/src/runtime.rs`

Do not start here.

## `flight.rhai` Pseudocode

### Surface seed

```rhai
fn seed_vehicle_surface_state(state, surface_radius, km_per_wu, surface_clearance_wu) {
    let spawn_angle = deg_to_rad(state.vehicle_spawn_angle_deg ?? 0.0);
    let heading = state.vehicle_heading ?? 0.0;

    let normal = surface_normal_from_angle(spawn_angle);
    let tangent = tangent_forward_from_normal(normal);
    let right = tangent_right_from_normal_and_forward(normal, tangent);

    let forward = rotate_around_normal(tangent, normal, heading);
    let right = rotate_around_normal(right, normal, heading);

    state.radius = surface_radius + surface_clearance_wu;
    state.vfwd = 0.0;
    state.vright = 0.0;
    state.vrad = 0.0;
    state.yaw_rate = 0.0;
    state.vehicle_grounded = true;
    state.vehicle_phase = "pad";

    state.snx = normal.x;
    state.sny = normal.y;
    state.snz = normal.z;
    state.sfx = forward.x;
    state.sfy = forward.y;
    state.sfz = forward.z;
    state.srx = right.x;
    state.sry = right.y;
    state.srz = right.z;

    init_camera_from_surface_pose(state, normal, forward);
    state
}
```

### Grounded step

```rhai
fn step_grounded(state, ...) {
    clamp_radius_to_surface_clearance();
    zero_local_motion();
    apply_yaw_around_surface_normal_only();

    if lift_requested() {
        state.vehicle_grounded = false;
        state.vehicle_phase = "hover";
    }

    sync_runtime(surface_contact: true, prefer_grounded_on_contact: true, request_detach: false);
    state
}
```

### Hover step

```rhai
fn step_surface_hover(state, ...) {
    apply_yaw_in_local_horizon();
    apply_tangent_thrust_and_strafe();
    apply_radial_lift_and_gravity();
    apply_drag_and_hover_damping();
    advance_basis_around_rotating_planet();

    if should_land() {
        snap_to_surface_clearance();
        zero_local_motion();
        state.vehicle_grounded = true;
        state.vehicle_phase = "pad";
    } else if should_detach_to_free_flight() {
        state.vehicle_phase = "flight";
    }

    sync_runtime(surface_contact: near_surface, prefer_grounded_on_contact: false, request_detach: false);
    state
}
```

### Free-flight step

```rhai
fn step_free_flight(state, ...) {
    run_existing_open_flight_logic();

    if high_altitude_or_speed() {
        state.vehicle_phase = "orbit";
    } else {
        state.vehicle_phase = "flight";
    }

    sync_runtime(surface_contact: false_or_contextual, prefer_grounded_on_contact: false, request_detach: true_if_needed);
    state
}
```

### Transition resolver

```rhai
fn resolve_vehicle_phase(state, altitude_km, tangent_speed, radial_speed, detach_intent, lift_on) {
    if (state.vehicle_phase ?? "pad") == "pad" {
        if lift_on { return "hover"; }
        return "pad";
    }

    if state.vehicle_phase == "hover" {
        if should_land(...) { return "pad"; }
        if should_detach(...) { return "flight"; }
        return "hover";
    }

    if state.vehicle_phase == "flight" {
        if high_altitude(...) { return "orbit"; }
        return "flight";
    }

    if state.vehicle_phase == "orbit" {
        if lower_altitude_and_surface_readable(...) { return "flight"; }
        return "orbit";
    }

    "pad"
}
```

## Concrete Current Code Anchors

These are the main places the next agent should inspect first:

- `mods/vehicle-playground/scripts/vehicle/flight.rhai`
  - current runtime sync
  - current orbit seeding
  - current grounded branch
  - current carrier rotation math
  - current camera block
- `mods/vehicle-playground/scripts/vehicle/state.rhai`
  - current default boot state
- `engine-vehicle/src/models/ship.rs`
  - current `Grounded / SurfaceLocked / Detached` transition behavior
- `engine-vehicle/src/runtime.rs`
  - typed runtime surface mode semantics
- `engine-vehicle/src/input.rs`
  - tuning values and thresholds

## Acceptance Criteria

The first pass is correct when:

1. boot starts on the surface, upright and still
2. lift causes a clean vertical break from the pad
3. low-altitude movement feels like hover/lander motion
4. the rotating planet is visible through world motion, not preloaded orbital drift
5. landing is threshold-driven and readable
6. camera is stable on pad and readable in hover
7. HUD says `PAD`, `HOVER`, `FLIGHT`, `ORBIT` truthfully
8. packet return still works
9. scene checks still pass

## Validation

Minimum:

```powershell
cargo run -p app -- --mod-source=mods/vehicle-playground --check-scenes
```

If engine-side runtime semantics are touched:

```powershell
cargo test -p engine-vehicle
cargo test -p engine-behavior --lib scripting::
```

If new runtime semantics or transition rules are introduced, add regression tests where appropriate.

## Suggested First Deliverable

Implement only this first pass:

1. add `vehicle_phase`
2. add `seed_vehicle_surface_state(...)`
3. remove default orbit seeding from boot/reset
4. split grounded and hover flow
5. make camera phase-aware for pad/hover
6. update HUD labels

Do **not** spend the first pass polishing orbital realism.
