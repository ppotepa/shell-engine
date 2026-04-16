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
| 4 — Atmosphere | Color (discrete 6), Density, Halo Width, Halo Bright, Veil Density, Rim Power, Haze Density, Haze Power |

**7 presetów:** Earth, Mars, Ocean, Desert, Ice, Volcanic, Archipelago — każdy to Rhai map ze wszystkimi parametrami.

### Atmosfera — obecny model

Geometry shell (atmo_shell.rs) pozostaje usunięty. Aktualny model atmosfery jest 2-warstwowy:

- **Inner atmosphere overlay** (`engine-render-3d/src/effects/atmosphere.rs`)
  - działa per-pixel na powierzchni planety
  - liczy `view_dir` z rzeczywistej pozycji kamery per piksel (`camera_pos - world_pos`), więc nie opiera się już na jednym globalnym kierunku widoku
  - składa się z 3 wkładów: `rim`, `haze`, `veil`
- **Outer halo pass** (`engine-compositor/src/obj_render.rs`)
  - działa jako prosty screenspace pass po renderze planety
  - dodaje świecącą koronę tuż poza sylwetką
  - wzmacnia stronę dzienną względem nocnej na podstawie projekcji kierunku światła

Kontrolowane przez:

- `atmo_color`
- `atmo_strength`
- `atmo_rim_power`
- `atmo_haze_strength`
- `atmo_haze_power`
- `atmo_veil_strength`
- `atmo_veil_power`
- `atmo_halo_strength`
- `atmo_halo_width`
- `atmo_halo_power`

**Status:** problem z niewidzialną atmosferą został naprawiony przez przejście z globalnego `view_dir` na per-pixel `camera_pos - world_pos`. Atmosfera jest nadal modelem uproszczonym: nie ma prawdziwego volumetric scattering ani osobnej geometrii shell, ale ma już wizualną furtkę pod gęstszą atmosferę i halo poza dyskiem.

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

### Priorytet — Atmosphere Polish
- [ ] **Tune halo shape** — obecny outer halo to prosty screenspace ring. Warto dopracować szerokość/falloff i stronę nocną, żeby mniej przypominał bloom, a bardziej cienką warstwę rozpraszającą.
- [ ] **Tune veil vs surface contrast** — dla gęstych presetów (`Ocean`, przyszły `Venus-like`) veil powinien mocniej obniżać kontrast powierzchni na tarczy planety.
- [ ] **Promote atmosphere profile** — obecne parametry są już rozdzielone logicznie; kolejnym krokiem może być `PlanetAtmosphereVisualProfile`, żeby obecny mod i przyszły realistic generator korzystały z tego samego adaptera.

### Krótkoterminowe
- [ ] **Slider snapping / discrete indicators** — slidery GUI nie mają wizualnych markerów dla wartości dyskretnych (np. atmo color ma 6 opcji). Przydałby się system ticków/snap w `engine-gui`
- [ ] **Schema regeneration** — po usunięciu `atmo-shell-scale` i `atmo-scale-height` z modelu, warto przebudować schematy: `cargo run -p schema-gen -- --all-mods`
- [ ] **Cloud layer** — planeta ma już support na `below_threshold_transparent` + `cloud_alpha_softness` w RGBA path

### Średnioterminowe
- [ ] **Terrain LOD** — przy dużych subdivisions (256-512) rasteryzacja jest ciężka; adaptacyjny LOD per-face mógłby pomóc
- [ ] **Biome map export** — export wygenerowanej planety do texture/heightmap
- [ ] **Multiple light sources** — `ObjRenderParams` ma już pola na 2 point lights + 2 directional lights, ale planet generator używa tylko 1 directional (sun)
- [ ] **Ring system** — Saturn-style rings jako oddzielny flat-disc OBJ z transparency

### Znane problemy
- **Atmosphere is still stylized, not physical** — obecny model daje halo + veil + haze, ale nie symuluje jeszcze składu chemicznego, wysokości skali, anisotropic scattering ani oddzielnej warstwy chmur.
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
