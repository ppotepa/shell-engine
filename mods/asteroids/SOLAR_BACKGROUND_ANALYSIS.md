# Asteroids Solar Background — Step-by-Step Design

## Goal

Replace fragmented background layers with one coherent `scene3_d` prefab that
looks like a believable solar-system slice, while staying stable at runtime and
compatible with free camera movement.

## Visual Target

- One authored background object: sun + planets + saturn-style ring system.
- Asteroid belt is a true annulus (disk with a center hole), orbiting the
  ringed planet.
- Belt has visible discrete asteroids on top of the ring disk (higher density,
  at least ~4x the previous rock count baseline).
- Composition reads as one scene, not random independent props.

## Simulation Model (Pragmatic, Not Full N-Body)

- Use precomputed clip frames for orbital motion (`solar-orbit-0..N`) instead of
  per-frame procedural transforms in Rhai.
- Keep world gameplay deterministic; background is cinematic and decorative.
- Drive only two runtime values from Rhai:
  - selected clip frame index based on elapsed time
  - small camera-relative drift offset for depth/parallax feel

This avoids jitter from complex per-object runtime writes and keeps frame cost
predictable.

## Implementation Steps

1. Author a single Scene3D asset (`solar-system.scene3d.yml`).
2. Define object groups inside it:
   - deep nebula shell
   - sun core + glow
   - planets/moon set
   - ring disk back/front meshes (annulus with center hole)
   - visible belt rocks distributed around the same orbital center
3. Define one static frame + one clip frame with keyframed yaw offsets.
4. Expose the Scene3D via a single gameplay layer (`solar-scene3d-layer.yml`).
5. In Rhai, select prerendered clip frame and apply minor drift each frame.
6. Keep HUD and gameplay entities as separate layers above the background.
7. Validate with:
   - `--check-scenes`
   - short bench run on the gameplay scene (`--bench 2 --logs`)

## Quality Gates (What “Looks Correct” Means)

- No flicker on top gameplay layer while camera moves freely.
- Ring disk reads clearly as a single annulus around the ringed planet.
- Visible belt rocks follow the same orbital band and do not look random.
- Background motion is smooth and slow (astronomical feel, no “spinning toy”).
- 60 FPS target remains intact on SDL2 benchmark runs.

## Next Iteration (Optional)

- Add a second wider belt in far depth for extra scale.
- Tie gameplay asteroid spawn zones to the same belt center/radii.
- Add subtle light flicker/noise on sun glow material for more life.
