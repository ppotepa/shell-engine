# Next — Planet Generator Handover

## Stan Projektu (Project State)

Planet Generator (`mods/planet-generator`) to interaktywny mod do generowania planet proceduralnych w silniku Shell Engine — terminalowym renderze 3D opartym o SDL2 pixel canvas. Mod pozwala w czasie rzeczywistym kręcić parametrami (slidery GUI) i oglądać jak planeta się zmienia.

### Co zostało zrobione (commits od `9f846cc6..HEAD`)

| Commit | Co |
|--------|----|
| `1e8f66e6` | **App launcher fallback** — `cargo run -p app` bez `--mod` uruchamia interaktywne TUI menu (launcher `se`) |
| `b19e45f2` | **Revert do per-pixel atmosphere (870131af)** — usunięcie geometry shell, powrót do prostego rim/haze |
| `5f14057d` | **Restore atmosphere shell + boost** — tymczasowe przywrócenie (potem nadpisane przez b19e45f2) |
| `4f522b10` | **Usunięcie geometry shell atmosphere** — zostaje tylko per-pixel rim/haze glow |
| `8174beb9` | **Mouse scroll wheel zoom** — dodanie `MouseWheel` event end-to-end (SDL2→engine-events→orbit camera) |
| `ea2ca304` | **Sphere-only** — usunięcie selektora base mesh, planeta zawsze jest sferą |
| `a1d0307a` | **Fix artefaktu "dwóch obiektów"** — atmosfera shell używała smooth normals + stały promień |
| `e39eced6` | **Orbit camera + base mesh rework + bugfixy** — drag mysz → obrót kamery, Ctrl+F → free-look |

### Obecna architektura Planet Generatora

**Scena:** `mods/planet-generator/scenes/main/` (scene.yml + layers/planet.yml + main.rhai)

**Sterowanie:**
- **Mouse drag** → orbit camera (obrót planety)
- **Scroll** → zoom in/out
- **+/-** → zoom (klawiatura)
- **Ctrl+F** → toggle free-look camera (WASD/QE do latania)
- **R** → toggle auto-rotacja planety
- **1-5** → przełączanie tabów (Continents / Mountains / Climate / Visual / Atmosphere)
- **F1-F7** → presety (Earth / Mars / Ocean / Desert / Ice / Volcanic / Archipelago)
- **Delete** → reset do Earth

**5 tabów z parametrami (slidery GUI, 0-1 normalized, Rhai mapuje na domeny):**

| Tab | Parametry |
|-----|-----------|
| 0 — Continents | Seed, Ocean%, Continent Size, Coast Chaos, Octaves |
| 1 — Mountains | Mtn Spacing, Mtn Height, Ridge Detail |
| 2 — Climate | Moisture, Ice Caps, Alt Cooling, Rain Shadow |
| 3 — Visual | Resolution, Displacement, Coloring Mode, Rotation Speed, Sun Az/El, Ambient |
| 4 — Atmosphere | Color (discrete 6), Density, Rim Power, Haze Density, Haze Power |

**7 presetów:** Earth, Mars, Ocean, Desert, Ice, Volcanic, Archipelago — każdy to Rhai map ze wszystkimi parametrami.

### Atmosfera — obecny model

Geometry shell (atmo_shell.rs) został **całkowicie usunięty** (~325 linii). Jedyny system atmosfery to:

- **Per-pixel rim/haze glow** (`apply_atmosphere_overlay_barycentric` w `engine-render-3d/src/effects/atmosphere.rs`)
- Działa wewnątrz rasteryzera trójkątów Gouraud — każdy piksel dostaje rim factor `(1 - N·V)^power`
- Sun-aware: dzień/noc z smoothstep, wyższa luminancja po stronie oświetlonej
- Kontrolowane przez: `atmo_color`, `atmo_strength`, `atmo_rim_power`, `atmo_haze_strength`, `atmo_haze_power`
- **Nie tworzy dodatkowej siatki** — glow podąża za kształtem planety pixel-perfect

**Znany problem:** Atmosfera jest aktualnie niewidoczna runtime. Debug wykazał, że `view_dir` w `PlanetBiomeParams` wynosi `(0.022, 1.0, 0.0)` zamiast oczekiwanego `(0, 0, -1)`. Prawdopodobna przyczyna: orbit camera (dodana po `870131af`) ustawia `camera_look_yaw/pitch` na sprite, co obraca `view_forward` w `ObjRenderParams`, ale `CameraSource::Local` (default) nie synchronizuje `view_forward` z faktyczną pozycją kamery. Wymaga dalszej analizy i fixa.

### App Startup — zmiana domyślnego zachowania

- `cargo run -p app` (bez `--mod`) → uruchamia launcher TUI (`se` binary)
- `cargo run -p app -- --mod planet-generator` → bezpośrednio uruchamia mod
- `cargo run -p app -- --mod-source mods/planet-generator` → j.w. z pełną ścieżką
- Wcześniej default był hardcodowany na "asteroids"/"shell-quest"

### Engine — kluczowe zmiany w silniku

1. **`MouseWheel` event** — nowy event w `EngineEvent` i `InputEvent` (`engine-events/src/lib.rs`), przechwycony z SDL2
2. **`apply_orbit_camera_scroll()`** — nowa metoda w `camera_3d.rs`
3. **Usunięte pola z `ObjRenderParams`**: `atmo_shell_scale`, `atmo_scale_height`
4. **Usunięte settery** z `materialization.rs`: `obj.atmo.shell_scale`, `obj.atmo.scale_height`
5. **App launcher fallback** — `--mod` jest teraz opcjonalny w app CLI, bez niego startuje launcher `se`

### Kamera — dwa tryby

| Tryb | Sterowanie | Co kontroluje |
|------|-----------|---------------|
| **Orbit** (domyślny) | Mouse drag, scroll, +/- | `obj.yaw`, `obj.pitch`, `camera-distance` na sprite |
| **Free-look** (Ctrl+F toggle) | WASD/QE + mouse look | Scena 3D camera (eye/look-at), `camera_look_yaw/pitch` |

`free_look_camera_engaged()` blokuje input orbit camera kiedy free-look jest aktywny.

---

## Co dalej (Potential Next Steps)

### Priorytet — Atmosphere Fix
- [ ] **Fix invisible atmosphere** — `view_dir` w biome params jest nieprawidłowy przy orbit camera. Trzeba albo:
  - Dodać `camera-source: scene` do planet.yml (żeby `view_forward` brane było ze `scene_camera_3d`)
  - Albo naprawić propagację orbit camera → view_forward w `CameraSource::Local` path
  - Albo hardcodować view_dir jako `-camera_eye_normalized` w `setup.rs`

### Krótkoterminowe
- [ ] **Slider snapping / discrete indicators** — slidery GUI nie mają wizualnych markerów dla wartości dyskretnych (np. atmo color ma 6 opcji). Przydałby się system ticków/snap w `engine-gui`
- [ ] **Schema regeneration** — po usunięciu `atmo-shell-scale` i `atmo-scale-height` z modelu, warto przebudować schematy: `cargo run -p schema-gen -- --all-mods`
- [ ] **Atmosphere glow beyond silhouette** — per-pixel rim jest ograniczony do pikseli na powierzchni planety. Prawdziwa atmosfera widoczna jest też poza dyskiem (korona). Opcje:
  - Post-process bloom/glow pass na krawędziach planety
  - Screenspace ring/halo rasteryzowany po planecie
- [ ] **Cloud layer** — planeta ma już support na `below_threshold_transparent` + `cloud_alpha_softness` w RGBA path

### Średnioterminowe
- [ ] **Terrain LOD** — przy dużych subdivisions (256-512) rasteryzacja jest ciężka; adaptacyjny LOD per-face mógłby pomóc
- [ ] **Biome map export** — export wygenerowanej planety do texture/heightmap
- [ ] **Multiple light sources** — `ObjRenderParams` ma już pola na 2 point lights + 2 directional lights, ale planet generator używa tylko 1 directional (sun)
- [ ] **Ring system** — Saturn-style rings jako oddzielny flat-disc OBJ z transparency

### Znane problemy
- **Atmosphere invisible** — view_dir mismatch (patrz wyżej). Atmosfera renderuje się ale z błędnym kątem kamery, więc rim factor jest ~0 na widocznych pikselach.
- 3 testy w `engine` failują — `shell_engine_intro_logo_renders_non_black_cells`, `real_shell_engine_mod_manifest_and_entrypoint_load`, `real_shell_engine_scenes_all_load` — bo `mods/shell-engine` nie istnieje lokalnie. **Nie nasze bugi.**
- `engine-compositor` lib tests mają failing test z powodu brakującego pola `space` w `Layer` test initializers — **pre-existing, nie nasze.**

---

## Pliki kluczowe

| Plik | Rola |
|------|------|
| `app/src/main.rs` | CLI app — launcher fallback, mod resolution |
| `mods/planet-generator/scenes/main/main.rhai` | Cała logika GUI, sliderów, presetów, push do engine (~580 linii) |
| `mods/planet-generator/scenes/main/layers/planet.yml` | YAML sprite z planet-mesh, canvas 420×420, orbit camera |
| `engine-render-3d/src/effects/atmosphere.rs` | Per-pixel rim/haze atmosphere overlay (~98 linii) |
| `engine-compositor/src/obj_render.rs` | Główny dispatch OBJ rendering |
| `engine-compositor/src/obj_render/params.rs` | `ObjRenderParams` — wszystkie parametry renderowania 3D |
| `engine-compositor/src/obj_render/setup.rs` | `build_biome_params()` + `normalized_light_and_view_dirs()` |
| `engine-scene-runtime/src/camera_3d.rs` | Orbit camera + free-look + scroll zoom |
| `engine-scene-runtime/src/materialization.rs` | Runtime settery `scene.set()` → params pipeline |
| `engine-events/src/lib.rs` | `EngineEvent` / `InputEvent` (w tym `MouseWheel`) |
| `engine-worldgen/src/lib.rs` | Proceduralna generacja: noise, biomy, klimat, heightmap |

## Komendy

```bash
# Uruchom launcher (menu TUI)
cargo run -p app

# Uruchom planet generator bezpośrednio
cargo run -p app -- --mod planet-generator

# Z debug overlay (F1=stats, ~/`=logs)
cargo run -p app -- --mod planet-generator --dev

# Scene check
cargo run -q -p app -- --check-scenes --mod-source=mods/planet-generator

# Testy
cargo test -q -p engine -p engine-scene-runtime -p engine-events

# Schema regen
cargo run -p schema-gen -- --all-mods
```
