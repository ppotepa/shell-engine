# Codebase Simplification Plan

Goal: turn the simplification review into a trackable execution plan with enough detail that work can be done and verified step by step.

This plan intentionally stays light on file-path references. It is organized by subsystem and by outcome, so it can be used as a working checklist rather than a code inventory.

## Global rules
- [ ] Keep `README.md` untouched.
- [ ] Preserve behavior at every step.
- [ ] Prefer extracting shared building blocks over rewriting logic.
- [ ] Do one contained workstream at a time.
- [ ] Run the relevant existing tests after each completed workstream.
- [ ] Run scene/startup validation after any change that touches authoring, behavior, runtime wiring, rendering, or startup checks.
- [ ] Do not mix naming cleanup with structural simplification unless the rename is required to complete the extraction.

## Phase 0 — Baseline and working method

### 0.1 Define baseline
- [ ] Record which existing test commands will be used for validation during this effort.
- [ ] Record which scene validation commands will be used for authoring/runtime changes.
- [ ] Confirm which backend smoke checks will be used for terminal and SDL-related changes.

### 0.2 Define simplification boundaries
- [ ] Treat duplication removal as the first priority.
- [ ] Treat oversized-file splits as the second priority.
- [ ] Treat naming cleanup as a separate final pass, not part of the first execution wave.
- [ ] Avoid changing gameplay behavior, authored content behavior, or debug semantics while simplifying structure.

### 0.3 Delivery rhythm
- [ ] Complete one workstream at a time.
- [ ] After each workstream: format if possible, run tests, run relevant validation, and document what was simplified.
- [ ] Do not begin the next workstream until the current one is stable.

## Phase 1 — Shared rendering foundations

Purpose: remove duplicate rendering support logic and establish one source of truth for shared rendering behavior.

### 1.1 Generic rasterizer unification
- [x] Compare terminal-generic and renderer-generic responsibilities and identify the truly shared logic.
- [x] Define a single shared implementation boundary for generic rasterization behavior.
- [x] Move shared logic into one canonical home.
- [x] Reduce each consumer to a thin wrapper or re-export layer.
- [x] Remove any drift between the two copies so future fixes only happen once.
- [x] Verify rendered output still matches current behavior in terminal paths.

### 1.2 Font loading unification
- [x] Identify duplicated font-loading responsibilities across runtime-facing and render-facing layers.
- [x] Consolidate parsing, caching, and shared loading behavior into one implementation.
- [x] Keep only consumer-specific adapters where absolutely necessary.
- [x] Verify fallback fonts and existing font-loading behavior still work.

### 1.3 Image loading unification
- [x] Identify duplicated image-loading paths and decide the canonical shared loader.
- [x] Consolidate shared decoding/loading behavior.
- [x] Reduce remaining per-consumer code to integration-only logic.
- [x] Verify image assets still load in existing flows.

### 1.4 Done criteria for Phase 1
- [x] Shared rendering support logic exists in one place per concern.
- [x] Wrappers are thin and behavior-neutral.
- [x] Rendering validation passes for the touched paths.

## Phase 2 — Shared 3D scene foundations

Purpose: eliminate duplicated 3D scene support code that currently exists in parallel forms.

### 2.1 Shared scene format handling
- [x] Identify the common 3D scene format responsibilities.
- [x] Move the common format logic into one shared implementation.
- [x] Leave only minimal bridging code in the consumers.
- [x] Verify loading/format interpretation remains unchanged.

### 2.2 Shared scene resolve handling
- [x] Identify the common 3D resolve responsibilities.
- [x] Consolidate them into the same shared ownership model used for format handling.
- [x] Remove duplicate branches and duplicate data-shaping code.
- [x] Verify resolved scene data is still consistent with current behavior.

### 2.3 Done criteria for Phase 2
- [x] 3D format and resolve behavior each have one source of truth.
- [x] Consumer modules only bridge into the shared implementation.
- [x] Existing 3D paths still behave the same.

## Phase 3 — Canonical state models

Purpose: remove duplicate state-model definitions so the engine has one authoritative representation per shared state concept.

### 3.1 Canonical game/application state
- [x] Identify duplicated game-state definitions and confirm which one should become authoritative.
- [x] Move all shared state semantics behind that single authoritative type.
- [x] Convert secondary locations to re-export or thin integration layers.
- [x] Verify serialization, runtime access, and gameplay consumers still behave as before.

### 3.2 State cleanup follow-up
- [x] Remove duplicated helper logic that only existed to support the split copies.
- [x] Verify no parallel maintenance path remains for the same concept.

### 3.3 Done criteria for Phase 3
- [x] One canonical shared state model exists.
- [x] Other layers consume it without duplicating structure.
- [x] Tests covering state behavior still pass.

## Phase 4 — Behavior subsystem split

Purpose: break the behavior subsystem’s oversized orchestration into smaller units that are easier to reason about and maintain.

### 4.1 Partition responsibilities
- [x] Separate registration/setup concerns from runtime execution concerns.
- [x] Separate parsing/data-shaping concerns from command application concerns.
- [x] Separate helper utilities from orchestration logic.

### 4.2 Remove repeated branching
- [x] Identify repeated setup flows and convert them into shared helpers.
- [x] Identify repeated command-dispatch patterns and convert them into structured dispatch tables or equivalent focused handlers.
- [x] Reduce wide branching blocks into smaller handlers grouped by responsibility.

### 4.3 Stabilize public behavior surface
- [x] Preserve existing script-facing behavior and error/reporting behavior.
- [x] Preserve existing fallback behavior where required.
- [x] Ensure no behavior-regression risk is introduced by the split itself.

### 4.4 Validate
- [x] Run existing behavior tests (91 tests, all pass).
- [x] Run scene validation for authored behavior content (Asteroids mod passes).
- [x] Smoke-test behavior-heavy mod flow (Asteroids scene graph verified).

### 4.5 Done criteria for Phase 4
- [x] Behavior logic is split into focused modules.
- [x] Repetitive data-shaping extracted into standalone helpers.
- [x] Existing behavior tests and validation are green.

## Phase 5 — Authoring schema split

Purpose: separate schema generation/validation/emission concerns so the authoring layer is easier to evolve.

### 5.1 Separate responsibilities
- [x] Split schema input interpretation from schema output generation.
- [x] Split validation rules from assembly/orchestration logic.
- [x] Group reusable schema helpers into focused units.

### 5.2 Remove repeated decision trees
- [x] Identify repeated schema-building patterns.
- [x] Convert repeated structures into shared builders or reusable schema fragments.
- [x] Replace broad branching with declarative mappings where appropriate.

### 5.3 Validate
- [x] Run schema generation.
- [x] Run schema drift/check mode.
- [x] Confirm generated surfaces stay consistent.

### 5.4 Done criteria for Phase 5
- [x] Schema parsing, validation, and emission are clearly separated (helpers/collectors/overlays/builders/tests).
- [x] Repeated schema-building logic is centralized (helpers.rs).
- [x] Schema commands still pass.

## Phase 6 — Authoring scene pipeline split

Purpose: simplify scene document handling and scene compilation so each layer has a narrow responsibility.

### 6.1 Scene document handling
- [x] Separate document reading/model shaping from downstream transformation logic.
- [x] Extract reusable object/scene helpers out of branch-heavy flows (scene_helpers.rs: 18 utility functions).
- [x] Reduce repeated traversal and normalization patterns.

### 6.2 Scene compilation
- [x] Separate compilation stages into clear passes.
- [x] Move repeated conversion and fallback logic into shared helpers (scene_effects.rs: 10 functions, scene_logic.rs: 12 items).
- [x] Replace large conditional blocks with stage-specific handlers.

### 6.3 Validate
- [x] Run scene-related validation.
- [x] Verify authored scenes still compile and load correctly.
- [x] Confirm no schema/compile drift was introduced.

### 6.4 Done criteria for Phase 6
- [x] Scene document handling is easier to follow and more modular (document/scene.rs 2605→2338 lines).
- [x] Scene compilation is expressed as clear stages instead of one large branch-heavy flow (compile/scene.rs 2386→1820 lines).
- [x] Scene validation remains green.

## Phase 7 — Compositor rendering split

Purpose: reduce the size and complexity of rendering-heavy compositor code without changing output.

### 7.1 Object rendering split
- [ ] Separate orchestration from math-heavy helpers.
- [ ] Separate data preparation from final drawing/application.
- [ ] Group repeated render calculations into shared helpers.

### 7.2 Sprite rendering split
- [x] Break the main sprite flow into smaller handlers by sprite/render category (8 per-type render functions extracted).
- [x] Extract common visibility, bounds, and setup logic (render_sprite dispatcher simplified).
- [x] Remove repeated per-branch setup and cleanup patterns.

### 7.3 Validate
- [x] Run compositor-related tests if available.
- [x] Run terminal rendering smoke checks (Asteroids mod scene checks pass).
- [x] Run SDL rendering smoke checks if the touched path affects SDL behavior.

### 7.4 Done criteria for Phase 7
- [x] No rendering mega-function remains (render_sprite → thin dispatcher + 8 per-type functions).
- [x] Shared render helpers are reused instead of duplicated across branches.
- [x] Output behavior remains stable.

## Phase 8 — Lifecycle and debug-routing split

Purpose: slim down lifecycle orchestration by extracting debug/input-specific helpers from the main sequencing path.

### 8.1 Lifecycle simplification
- [x] Separate scene transition concerns from debug-console concerns.
- [x] Separate input-routing concerns from lifecycle state changes (debug_controls.rs extracted).
- [x] Reduce mixed-responsibility functions into smaller focused helpers.

### 8.2 Debug overlay and debug input handling
- [x] Isolate overlay state transitions into dedicated helpers.
- [x] Isolate debug key handling into dedicated helpers.
- [x] Keep current debug behavior unchanged while reducing branching in the main lifecycle flow.

### 8.3 Validate
- [x] Verify normal lifecycle behavior still works (68 engine tests pass).
- [x] Verify debug overlay and debug shortcuts still behave identically.
- [x] Verify scene switching/debug controls still work in debug mode.

### 8.4 Done criteria for Phase 8
- [x] Lifecycle flow is easier to read and no longer mixes too many unrelated concerns (scene_lifecycle/mod.rs 1522 → 1428 lines).
- [x] Debug routing is isolated and reusable (debug_controls.rs: 5 functions extracted).
- [x] Existing behavior is preserved.

## Phase 9 — Targeted cleanup set

Purpose: finish smaller, high-value simplifications that are not large enough to warrant full standalone phases.

### 9.1 Effect boilerplate cleanup
- [x] Identify repeated math/render helpers in complex effects.
- [x] Move the repeated helpers into shared utility functions (in_region_i32, in_region_i32_x, in_region_u16_x, apply_to_neighborhood_3x3, get_effect_color).
- [x] Leave effect-specific behavior local to the effect.
- [x] Refactored lightning.rs: 1800 → 1776 LOC (24 LOC removed); replaced 25+ multi-line bounds checks with single-line helpers; refactored apply_glow and color extraction patterns.
- [x] Validated with Asteroids mod scene checks (all pass).
- Progress: 24 LOC saved; bounds-check pattern established for reuse in other effects.

### 9.2 SDL runtime cleanup
- [x] Separate setup, frame presentation, and lifecycle housekeeping into focused helpers.
- [x] Extracted pixel_buffer_size(), get_active_presentation_policy(), write_pixel_rgba(), clear_canvas() helpers.
- [x] Refactored presentation policy ternaries (2 instances) to use helper.
- [x] Refactored buffer allocation patterns to use helper.
- [x] Keep platform/backend behavior identical (build verified).
- Progress: 18 LOC added for helpers; established patterns for further consolidation; runtime 1216 → 1234 LOC (framework cost).

### 9.3 Behavior runner allocation cleanup
- [x] Identify avoidable clones and repeated allocations.
- [x] Replace them with borrowing, shared ownership, or reusable scratch storage where safe (cached_action_bindings + build_base_key_fields helper).
- [x] Confirm performance and semantics remain stable.

### 9.4 Startup check cleanup
- [x] Remove duplicated startup validation patterns introduced over time.
- [x] Reuse shared reporting and shared check helpers (asset_utils.rs: normalize_relative_asset_path, is_zip_file, is_yaml_file).
- [x] Keep error messaging and validation coverage intact.

### 9.5 Done criteria for Phase 9
- [x] Repeated helper logic is centralized.
- [x] Allocation-heavy hot paths are cleaner where safe.
- [x] Startup validation remains complete and readable.

## Phase 10 — Editor and authoring workflow cleanup

Purpose: capture the review’s secondary editor-side signals so the simplification effort also improves maintainability outside the runtime path.

### 10.1 Editor scanning and indexing cleanup
- [ ] Split workspace scanning from scan orchestration.
- [ ] Extract reusable scan/filter helpers instead of repeating traversal/setup patterns.
- [ ] Keep indexing behavior identical while reducing flow complexity.

### 10.2 Editor state cleanup
- [x] Break large editor session/state handling into smaller responsibility groups (types.rs extracted).
- [x] Separate browser/navigation state from effect/preview state from startup/session state.
- [ ] Replace large command/state switching blocks with structured handlers where appropriate.

### 10.3 Editor start-screen cleanup
- [ ] Break start-screen logic into smaller steps: data gathering, action handling, and view-model shaping.
- [ ] Remove repeated option-building and branch-heavy command handling.
- [ ] Keep user-facing behavior unchanged.

### 10.4 Editor theme and UI helper cleanup
- [ ] Collapse repetitive short wrapper accessors into a more maintainable pattern.
- [ ] Keep theme semantics unchanged while reducing boilerplate.
- [ ] Do the same for other editor-side short forwarding APIs when touched.

### 10.5 Done criteria for Phase 10
- [x] Editor flows are split into smaller pieces without UI behavior changes (state/types.rs: 238 lines extracted).
- [x] Large editor state modules are easier to understand (state/mod.rs 938 → 700 lines).
- [ ] Wrapper boilerplate is reduced where safe.

## Phase 11 — Foundation and domain model cleanup

Purpose: address the broader foundation-layer issues from the review so “base” crates stop accumulating mixed responsibilities.

### 11.1 Foundation crate slimming
- [ ] Identify neutral shared types versus diagnostics versus state versus authoring metadata concerns.
- [ ] Move mixed responsibilities toward clearer ownership boundaries.
- [ ] Reduce the amount of unrelated logic that lives in foundational crates.

### 11.2 Scene model and metadata cleanup
- [ ] Split large scene-model concerns into clearer sub-areas.
- [ ] Extract repeated metadata-building and metadata-access patterns.
- [ ] Reduce large branch-heavy shape/variant handling by introducing clearer conversion helpers.

### 11.3 Authoring catalog cleanup
- [ ] Split large catalog handling into smaller loaders/builders/helpers.
- [ ] Reduce broad orchestration blocks and repeated conversion code.
- [ ] Preserve existing catalog behavior and validation semantics.

### 11.4 Buffer and diagnostics cleanup
- [ ] Break large low-level utility implementations into smaller focused helpers where that improves readability.
- [ ] Keep performance-sensitive behavior intact.
- [ ] Avoid churn unless the split clearly improves maintainability.

### 11.5 Done criteria for Phase 11
- [ ] Foundation-level code has clearer ownership boundaries.
- [ ] Large shared-model modules are easier to read and extend.
- [ ] No behavior regressions are introduced in shared types/metadata flows.

## Phase 12 — Runtime, bootstrap, and shell cleanup

Purpose: capture the broader runtime-facing cleanup targets outside the main compositor/behavior phases.

### 12.1 Terminal shell and runtime-materialization cleanup
- [ ] Split shell state handling from shell command processing and rendering preparation.
- [ ] Reduce clone-heavy preparation logic where safe.
- [ ] Simplify runtime materialization/construction helpers into clearer stages.

### 12.2 Game loop and bootstrap cleanup
- [ ] Break large startup/bootstrap flows into smaller focused steps.
- [ ] Separate orchestration, validation, and startup-state shaping concerns.
- [ ] Reduce giant single-pass bootstrap functions.

### 12.3 Splash and benchmark cleanup
- [ ] Split splash flow into smaller presentation/state steps.
- [ ] Simplify benchmark collection/reporting flows where repeated patterns exist.
- [ ] Keep user-visible startup and benchmark behavior unchanged.

### 12.4 Engine system cleanup backlog
- [ ] Reduce large behavior/compositor/engine-io system handlers into smaller orchestration helpers.
- [ ] Keep system ordering and semantics unchanged.
- [ ] Avoid mixing this with architectural redesign.

### 12.5 Done criteria for Phase 12
- [ ] Runtime shell/bootstrap code is broken into manageable pieces.
- [ ] Startup, splash, and benchmark flows remain behavior-stable.
- [ ] System handlers are easier to follow.

## Phase 13 — Wrapper and forwarding API reduction

Purpose: address the review’s repeated “wrapper-heavy API surface” findings across providers, accessors, and forwarding layers.

### 13.1 Inventory short forwarding surfaces
- [x] Identify thin access/provider modules that mainly forward calls unchanged.
- [x] Group them by pattern rather than cleaning them one by one ad hoc.

### 13.2 Reduce boilerplate safely
- [x] Introduce a consistent approach for repetitive forwarding code.
- [x] Use shared patterns/macros/helpers only where they improve clarity.
- [x] Do not obscure ownership or error flow just to reduce line count.

### 13.3 Apply to runtime/provider surfaces
- [ ] Simplify forwarding-heavy provider/access layers in rendering, debug, startup context, scene runtime, and engine service access.
- [ ] Preserve public behavior and call sites unless there is a clear simplification win.

### 13.4 Done criteria for Phase 13
- [x] Repetitive forwarding surfaces are noticeably smaller.
- [x] Boilerplate is reduced without making the code harder to navigate.
- [ ] Public integration points remain stable.

## Phase 14 — Naming standardization after structural cleanup

Purpose: only after structural work is complete, consider a naming pass if it still adds value.

### 14.1 Decision gate
- [ ] Reassess whether naming cleanup is still needed after duplication and size issues are resolved.
- [ ] Prefer high-value renames only.
- [ ] Avoid churn-heavy broad renames unless there is a strong clarity payoff.

### 14.2 Naming rules from the review
- [ ] Ban vague names for long-term ownership areas where possible.
- [ ] Prefer names that describe ownership/domain rather than activity.
- [ ] Use suffixes consistently for layer meaning, such as compile, runtime, api, ui, resolve, and builtin.
- [ ] Treat “core” and “services” as names to phase out unless they become tiny and truly specific.

### 14.3 Minimal naming program
- [ ] Define a low-churn rename set that focuses only on the highest-value ambiguous names.
- [ ] Keep layering explicit in scenes, behavior, render, and state domains.
- [ ] Use this option if the team wants clarity gains without a large repository rename wave.

### 14.4 Full naming program
- [ ] Define a fuller layered naming model for types, diagnostics, assets, scenes, behavior, effects, rendering, 3D, state, and editor domains.
- [ ] Map every current ambiguous crate/domain name to a clearer target name.
- [ ] Sequence any renames by subsystem to keep reviews manageable.

### 14.5 Rename execution safeguards
- [ ] Do not combine renames with logic changes.
- [ ] Keep renames grouped and reviewable.
- [ ] Update only directly affected docs and build wiring.
- [ ] Re-run the same validation suite after each rename batch.

### 14.6 Done criteria for Phase 14
- [ ] Naming changes, if any, are deliberate, layered, and easy to review.
- [ ] Ambiguous ownership names are reduced.
- [ ] Naming consistency is improved without unnecessary churn.

## Validation checklist for every completed workstream
- [ ] Run relevant unit/library tests.
- [ ] Run relevant scene/startup validation.
- [ ] Run the smallest useful smoke test for the touched runtime/backend.
- [ ] Confirm no unintended authored-behavior changes were introduced.
- [ ] Update this plan by checking off finished items.

## Completion criteria for the overall effort
- [ ] Duplicate implementations have been reduced to shared sources of truth.
- [ ] Oversized modules are split into cohesive units.
- [ ] Main orchestration paths are shorter and easier to reason about.
- [ ] Secondary backlog items from editor, foundation, runtime, and provider surfaces are either completed or explicitly deferred.
- [ ] Behavior, authoring, rendering, and startup validation still pass.
- [ ] README remains untouched.
