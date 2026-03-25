# Shell Quest Authoring: Metadata-First Rollout

## 1) Why this exists

We want one coherent authoring pipeline where runtime behavior, schema hints, CLI (`devtool`), and editor UI are all driven by the same source of truth.

Current state works, but still has hardcoded seams that slow development and create drift risk.

## 2) Current state (snapshot)

### Effects
- Runtime effects are trait-based (`Effect`) and expose metadata (`metadata()`).
- Per-mod effect overlays (`mods/*/schemas/effects.yaml`) are generated from metadata.
- `schema-gen` now validates shared effect schemas (`schemas/effect.schema.yaml`, `schemas/effect-params.schema.yaml`) against builtin effect metadata.
- Editor effects browser uses one descriptor registry for param read/write/labels (`editor/src/domain/effect_params.rs`).

### PostFX (implemented)
- Scene model supports `postfx: []` at scene root.
- Runtime executes `postfx_system` after compositing and before terminal flush.
- `postfx` pass runtime keeps frame history (`previous_output`) and frame counter.
- Built-in registered postfx variants:
  - `crt-underlay` (soft glow under content),
  - `crt-distort` (tube-like curvature + CRT margins),
  - `crt-scan-glitch` (random scanline sweep with right-shift/chroma glitch),
  - `crt-ruby` (ruby tint + center darkening + edge reveal sweep).
- Legacy compatibility remains for `terminal-crt` + `params.coverage`.
- Runtime postfx code path is split per pass into dedicated modules (`engine/src/systems/postfx/`).
- Runtime postfx applies passes through a compiled ordered pipeline with ping-pong scratch buffers
  (no full re-allocation clone per pass).
- Scene schema overlays include `postfx` in `scenes.yaml` generation path.

### Scene logic + presets (implemented contracts)
- Scene logic is explicit: compiler requires `logic:` block.
- `logic.kind: script` requires explicit `src`.
- No auto-sidecar discovery without `logic:`.
- `logic.kind: graph` is rejected as experimental.
- `effect-presets` / `effect_presets` are supported with strict validation and deterministic errors.

### What is still hardcoded / partially manual
- Effect registration list is duplicated manually in `engine-core/src/effects/mod.rs`:
  - `register_builtins()`
  - `builtin_names()`
- Shared effect schema files are still hand-authored (validated, not generated).
- `devtool` scaffold templates are hardcoded strings (`render_*_yaml` helpers).

### Maturity estimate
- Effects automation: ~80%
- PostFX pipeline contract: ~70%
- Whole authoring metadata-first automation: ~45-55%

## 3) Target architecture

## Principle
**Metadata first, codegen second, tools as consumers.**

Introduce a shared descriptor model and registry pattern:

- `AuthoringDescriptor` trait (or equivalent static descriptor struct)
- Domain descriptors:
  - effects
  - sprite/layout nodes (grid/flex/image/text/obj, etc.)
  - behaviors
  - input profiles
  - future authoring surfaces
- Single registry per domain (prefer macro-based declaration).

All tooling consumes those descriptors:
- runtime validation
- schema generation (shared + per-mod overlays)
- `devtool` create/edit forms and defaults
- editor contextual controls

## 4) Rollout phases

### Phase A: Unify effect registry (low-medium effort)
- Replace duplicated effect lists with one declaration source.
- Generate both registration and exported names from one registry declaration.

Outcome: no more missing `builtin_names()` updates when adding effects.

### Phase B: Generate shared effect schemas (medium effort)
- Replace hand-maintained `schemas/effect.schema.yaml` and `schemas/effect-params.schema.yaml` with generated output from effect metadata.
- Keep friendly descriptions; generate deterministic ordering.

Outcome: effect hints become fully automatic.

### Phase C: Devtool metadata-backed scaffolding (medium effort)
- Replace hardcoded scaffold text in `devtool` with descriptor-driven templates.
- Add strong `create` and `edit` commands based on descriptor constraints.
- Keep backward-compatible aliases (`new` -> `create`).

Outcome: CLI authoring becomes safer, richer, less brittle.

### Phase D: Layout + sprite descriptor expansion (medium-high effort)
- Model layout/sprite authoring constraints in metadata (fields, required-if, defaults, enum sources).
- Generate schema constraints and CLI forms from the same descriptors.

Outcome: fewer manual schema edits when authoring model changes.

### Phase E: Full consistency guardrails (medium effort)
- Add consistency tests:
  - registry <-> generated schemas
  - registry <-> `devtool` command capability
  - registry <-> editor field rendering

Outcome: drift caught in CI before users see broken hints.

## 5) Development impact (what we gain)

- Faster feature delivery: adding new builtins/domain fields touches fewer files.
- Lower regression risk: less copy-paste and duplicate lists.
- Better UX for content authors: always-correct completion and validation.
- Cleaner architecture: runtime/tooling/editor stay aligned by construction.

## 5.1) Operational rule for authoring changes

When changing any authored YAML contract, think in full-stack authoring terms rather than one file at a time.

Usually the change is not done until all of these are checked:

1. runtime/core model accepts the field or behavior
2. `engine-authoring` compiles and normalizes it
3. base schemas and per-mod overlays describe it
4. `./refresh-schemas.sh` and `schema-gen --check` stay green
5. editor and `devtool` understand the new surface, or the gap is explicitly documented
6. real mod content is migrated if the new contract becomes preferred

This is especially important for scene-centric work:

- `scene.yml` orchestration changes
- `scene.layers[].ref` resolution
- `layer.objects`
- effect parameter surfaces
- asset/source semantics

## 6) Effort profile

- Near-term win (A+B): medium, high ROI.
- Full metadata-first rollout (A->E): medium-large refactor.

Recommended strategy:
1. Finish A+B first (effects end-to-end automation).
2. Then expand descriptor system to `devtool` and layout/sprite surfaces incrementally.

## 7) Definition of done (long-term)

- New builtin (effect/layout primitive/behavior/input profile) can be added once in metadata/registry.
- Schemas, CLI hints, and editor controls update automatically.
- CI fails on any mismatch between runtime descriptors and authoring surfaces.

## 8) Execution-ready checklist (file-level)

### Phase A — single effect registry source

Target files:
- `engine-core/src/effects/mod.rs`
- `engine-core/src/effects/builtin/mod.rs`
- (new) `engine-core/src/effects/registry.rs` (or macro module)

Tasks:
- Introduce one declarative registry list for builtins.
- Drive both registration and `builtin_names()` from that list.
- Keep effect order deterministic for stable schema output.

Acceptance:
- Adding one new effect requires one registry entry only.
- No duplicated hardcoded names in multiple lists.

### Phase B — generate shared effect schemas from metadata

Target files:
- `tools/schema-gen/src/main.rs`
- (new) `engine-authoring/src/schema/shared_effects.rs` (or equivalent)
- generated outputs:
  - `schemas/effect.schema.yaml`
  - `schemas/effect-params.schema.yaml`

Tasks:
- Build generators for shared effect enum and param property set from `EffectMetadata`.
- Replace manual maintenance with generated output step.
- Keep human-readable descriptions (summary/sample/param descriptions).

Acceptance:
- Running schema generation fully rewrites shared effect schemas from metadata.
- CI fails if generated shared schema is stale.

### Phase C — metadata-backed devtool create/edit

Target files:
- `tools/devtool/src/main.rs` (split)
- (new) `tools/devtool/src/cli.rs`
- (new) `tools/devtool/src/create.rs`
- (new) `tools/devtool/src/edit.rs`
- (new) `tools/devtool/src/schema.rs`

Tasks:
- Introduce `create` as primary command and keep `new` alias.
- Implement:
  - `create mod <name>`
  - `create scene <name> --mod <mod>`
  - `create layer <name> --mod <mod> --scene <scene>`
  - `create sprite <source> --mod <mod> --scene <scene> --layer <layer>`
- Add safe edit operations (`set`, `rename`, `remove`) with validation.
- Refresh touched mod schemas after mutations.

Acceptance:
- User examples work exactly:
  - `./devtool.sh create mod shell-quest-2`
  - `./devtool.sh create sprite path.png --mod shell-quest-2 --scene intro --layer main`
- Errors are explicit (missing mod/scene/layer, conflicting ids).

### Phase D — layout/sprite descriptor expansion

Target files:
- `engine-core/src/scene/*` (descriptor definitions)
- `engine-authoring/src/schema/mod.rs`
- editor forms/components consuming those descriptors

Tasks:
- Define descriptor metadata for layout/sprite primitives (grid/flex/image/text/obj).
- Encode required-if constraints, enum/value sources, defaults.
- Use descriptors for both schema generation and CLI constraints.

Acceptance:
- New sprite/layout field appears in schema + devtool + editor from one descriptor update.

### Phase E — consistency guardrails

Target files:
- `tools/schema-gen/src/main.rs` tests
- `tools/devtool/src/main.rs` tests (or split test modules)
- CI command docs / scripts

Tasks:
- Add tests for:
  - registry vs generated schemas
  - registry vs devtool command availability
  - descriptor fields vs editor field wiring (where testable)
- Add clear failure messages pointing to source of drift.

Acceptance:
- Drift is caught automatically before merge.

## 9) Suggested PR slicing

1. PR-1: Phase A only (registry unification).
2. PR-2: Phase B only (shared effect schema generation).
3. PR-3: Phase C create commands (`create`, `new` alias, mod/scene/layer/sprite).
4. PR-4: Phase C edit commands + tests.
5. PR-5+: Phase D/E incremental descriptor rollout by domain.

## 10) Daily authoring workflow

For normal content work the expected loop is:

1. edit YAML under `mods/<mod>/`
2. run `./refresh-schemas.sh`
3. continue authoring with updated completions/validation
4. run `cargo run -q -p schema-gen -- --all-mods --check` before merge or when touching schema-sensitive surfaces

If the change affects scene/layer/object structure, also verify:

- package scene assembly still behaves as intended
- the editor preview reflects the compiled scene correctly
- no legacy fallback is accidentally doing hidden work for the new content
