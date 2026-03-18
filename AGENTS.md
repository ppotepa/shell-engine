
 ~/git/shell-quest │ on main !19  micro aAGENTS.md                                                                                                                 ✔ │ with ppotepa@ppotepa-dev2 │ at 21:31:05
 ~/git/shell-quest │ on main !19  micro AGENTS.md                                                                                                                  ✔ │ with ppotepa@ppotepa-dev2 │ at 21:31:05
233 - `schemas/` (generated fragments)
234
235 Scenes can be:
236 - single YAML files
237 - packaged scene dirs (`scene.yml` + partials)
238
239 Asset loading supports both unpacked directories and zip-packaged mods.
240
241 ## 6) Editor architecture (short map)
242
243 `editor/src`:
244 - `app.rs`: terminal lifecycle and main editor loop
245 - `cli.rs`: CLI options (`--mod-source`)
246 - `domain/`: scene/effect/asset indexes, diagnostics
247 - `io/`: file scanning, yaml, recents
248 - `input/`: command + key mapping
249 - `state/`: app state
250 - `ui/`: draw/layout/focus/filters/theme
251
252 Editor shares model/metadata from `engine-core` and `engine-authoring`.
253
254 ## 7) Tooling and schema workflow
255
256 Schema generation:
257 - `cargo run -p schema-gen -- --all-mods`
258
259 Schema verification in CI/local checks:
260 - `cargo run -p schema-gen -- --all-mods --check`
261
262 Helper script:
263 - `./refresh-schemas.sh` (single run or loop mode)
264
265 ## 8) Operational commands
266
267 - Run game:
268   - `cargo run -p app`
269 - Run editor:
270   - `cargo run -p editor`
271 - Run with playground mod:
272   - `SHELL_QUEST_MOD_SOURCE=mods/playground cargo run -p app`
273 - Core runtime tests:
274   - `cargo test -p engine`
275   - `cargo test -p engine-core`
276
277 ## 9) Critical invariants (must preserve)
278
279 - Keep system order stable unless explicit architecture change is requested.
280 - Preserve resolver correctness against sorted layer/sprite runtime order.
281 - Apply scene `virtual-size-override` on transitions.
282 - Keep virtual buffer synced with terminal resize in max-available mode.
283 - Do not reintroduce animator freeze for empty/0ms stages.
284 - Reset per-frame behavior runtime state before behavior application.
285 - Maintain compatibility with existing YAML mod structure.
286
287 ## 10) Change playbook for AI agents
288
289 When changing:
290 - scene model/fields:
291   - update `engine-core` model + runtime consumption + schema/authoring surfaces
292 - effect params:
293   - update effect metadata + schema generation path + editor consumption
294 - render pipeline:
295   - verify compositor + renderer + virtual buffer interactions
296 - transitions/lifecycle:
297   - verify scoped reset behavior and scene loader ref resolution
298
299 Bias:
300 - prefer minimal, local, type-safe changes
301 - avoid hidden fallback behavior
302 - test changed surfaces with existing crate tests
303 q
304
