# engine-animation

Scene animation timing, lifecycle stages, and menu-action evaluation.

## Purpose

`engine-animation` owns scene progression mechanics: stage timing, step
advancement, lifecycle transitions, and menu-action evaluation that happens as
scenes animate and advance.

## Key modules

- `animator` — `Animator` and `SceneStage`
- `systems` — frame-driven animator system wiring
- `menu` — menu action evaluation helpers
- `provider` / `access` — traits used to decouple animation logic from concrete world types

## Main exports

- `Animator`
- `SceneStage`
- `animator_system()`
- `MenuAction`
- `evaluate_menu_action()`
- `LifecycleProvider`

## Working with this crate

- preserve stage/step progression semantics carefully, especially around zero-duration steps,
- keep lifecycle abstractions generic so animation logic does not collapse back into `engine`,
- when animation behavior changes, verify intro/menu pacing in real content and tests.
