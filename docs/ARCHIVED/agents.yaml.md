## 1) Cel dokumentu

Ten plik jest praktycznym przewodnikiem authoringu YAML dla modow Shell Quest.
Opisuje, jak dziala pipeline, jakie sa kontrakty schema, jak projektowac sceny/obiekty/efekty
oraz jak przechodzic walidacje runtime i compile-time.

To jest dokument roboczy dla autorow contentu i agentow pracujacych w repo.

---

## 2) Szybki model projektu

- `app` uruchamia `engine` z wybranym modem.
- `engine` wykonuje runtime: loop, lifecycle, systems, render, audio.
- `engine-core` trzyma runtime model scen/effectow oraz metadata.
- `engine-authoring` trzyma granice authoringu (compile/validate/schema/package/repository).
- `mods/*` to source content (YAML-first).

Najwazniejsze: edytowalny source to YAML moda, runtime konsumuje skompilowany model.

---

## 3) Jak dziala schema per mod

## 3.1 Zasada

Nie generujemy jednego globalnego monolitu schema dla calego repo.
Generujemy zestaw schema **per mod** pod `mods/<mod>/schemas/`, zeby intellisense byl
zawsze zgodny z konkretnym contentem moda (scene ids, object refs, effect refs, itp.).

## 3.2 Standardowe pliki schema w modzie

- `schemas/mod.yaml`
- `schemas/scenes.yaml`
- `schemas/object.yaml`
- `schemas/objects.yaml`
- `schemas/layers.yaml`
- `schemas/templates.yaml`
- `schemas/sprites.yaml`
- `schemas/effects.yaml`

Wazne: to jest **jeden zestaw na mod** (nie globalny monolit na cale repo).

## 3.3 Co jest w `catalog.yaml`

Dynamiczne enumy, np.:

- `scene_ids`, `scene_paths`, `scene_refs`
- `object_names`, `object_refs`

- `effect_names` (content moda + built-in)
- `layer_refs`, `sprite_refs`, `template_refs`, `effect_refs`
- `image_paths`, `model_paths`
- `font_names`, `font_specs`
- `sprite_ids`, `template_names`

To daje YAML intellisense "pod ten konkretny mod".

---

## 4) Naglowki `$schema` - gotowa sciagnka

## 4.1 Manifest moda

Plik:

- `mods/<mod>/mod.yaml`

Naglowek:

```yaml
# yaml-language-server: $schema=./schemas/mod.yaml
```

## 4.2 Scene root (single-file pod `scenes/`)

Plik:

- `mods/<mod>/scenes/foo.yml`

Naglowek:

```yaml
# yaml-language-server: $schema=../schemas/scenes.yaml
```

## 4.3 Scene package root (`scenes/<name>/scene.yml`)

Plik:

- `mods/<mod>/scenes/<name>/scene.yml`

Naglowek:

```yaml
# yaml-language-server: $schema=../../schemas/scenes.yaml
```

## 4.4 Scene package partials

Pliki:

- `scenes/<name>/layers/*.yml` -> `../../../schemas/layers.yaml`
- `scenes/<name>/templates/*.yml` -> `../../../schemas/templates.yaml`
- `scenes/<name>/objects/*.yml` -> `../../../schemas/objects.yaml`
- `scenes/<name>/sprites/*.yml` -> `../../../schemas/sprites.yaml` (gdy uzywasz)
- `scenes/<name>/effects/*.yml` -> `../../../schemas/effects.yaml` (opis ponizej: ograniczenia runtime)

Przyklad:

```yaml
# yaml-language-server: $schema=../../../schemas/layers.yaml
```

## 4.5 Reusable object prefab

Plik:

- `mods/<mod>/objects/*.yml`

Naglowek:

```yaml
# yaml-language-server: $schema=../schemas/object.yaml
```

---

## 5) Scene discovery i scene package semantics

## 5.1 Ktore pliki sa indeksowane jako sceny

Indeksowane sa pliki YAML pod `scenes/`, ale **z wykluczeniami**:

- `scenes/shared/**` - nie jest scena entry,
- `scenes/<scene>/layers/**` - partiale,
- `scenes/<scene>/sprites/**` - partiale,
- `scenes/<scene>/templates/**` - partiale,
- `scenes/<scene>/objects/**` - partiale,
- `scenes/<scene>/effects/**` - partiale/rezerwacja.

Czyli scena entry to np.:

- `scenes/menu.yml` (single-file),
- `scenes/menu/scene.yml` (folder package root).

## 5.2 Merge order dla scene package

Przy `scenes/<name>/scene.yml` engine sklada dokument w stalej kolejnosci:

1. root `scene.yml`
2. `layers/*.yml` (append do `layers`)
3. `templates/*.yml` (merge mapy `templates`)
4. `objects/*.yml` (append do `objects`)

Dopiero potem idzie normalizacja i compile do runtime `Scene`.

## 5.3 Uwaga o `effects/` partials

Katalog `scenes/<name>/effects/` jest obecnie zarezerwowany w discovery/schema, ale
**nie jest automatycznie merge'owany** przez assembler package (w runtime merge sa tylko layers/templates/objects).

W praktyce:

- efekty authoruj bezposrednio w `stages.*.steps[*].effects`.
- `effects.yaml` i `effect_refs` sa przygotowaniem pod dalszy etap, ale nie daja jeszcze automatycznego compose `effects/` do sceny.

---

## 6) Authoring pipeline: YAML -> runtime

Pipeline (skrocony):

1. Repo (`engine/src/repositories.rs`) laduje scene root i ewentualnie sklada package.
2. `engine-authoring::compile::scene`:
   - rozwija `objects:` (`ref/use`, `as/id`, `with`),
   - podstawia eksporty (`$arg`) z object doc.
3. `SceneDocument::compile()` (`engine-authoring/src/document/scene.rs`):
   - normalizuje aliasy/shorthand,
   - rozwija template `use + args`,
   - mapuje expressions, ktore sa aktualnie wspierane,
   - deserializuje do runtime `engine_core::scene::Scene`.
4. Runtime systemy w `engine` renderuja i obsluguja lifecycle.

To jest granica authored/runtime: YAML nie jest juz "wprost serde do Scene" na wejsciu repo.

---

## 7) `mod.yaml` - pelna spec praktyczna

Pola wymagane:

- `name: string`
- `version: string`
- `entrypoint: "/scenes/...yml"`

`terminal` (opcjonalne):

- `min_colours`, `min_width`, `min_height`
- `target_fps`
- `use_virtual_buffer`
- `virtual_size` (`WxH` lub `max-available`)
- `virtual_policy` (`strict` lub `fit`)
- `renderer_mode` (`cell|halfblock|quadblock|braille`)

Aliasy kebab/snake dla niektorych pol sa wspierane schema'owo.

Praktyka w tym repo:

- `shell-quest` uzywa mod-level `virtual_size: max-available`.
- scene-level fixed override zostawiamy tylko dla celowych cinematic/letterbox.

---

## 8) Scene YAML - pola i semantyka

## 8.1 Najwazniejsze top-level pola

- `id` (wymagane)
- `title` (wymagane)
- `cutscene`
- `target-fps`
- `rendered-mode`
- `virtual-size-override`
- `bg_colour` oraz alias `bg`
- `stages`
- `layers`
- `next`
- `menu-options` (alias `menu_options`)
- `input`
- `templates`
- `objects`
- `behaviors` (scene-level)
- `audio` (`on_enter|on_idle|on_leave`)

## 8.2 Lifecycle stages

Struktura:

- `stages.on_enter`
- `stages.on_idle`
- `stages.on_leave`

Kazda stage ma:

- `steps: []`
- opcjonalnie `looping`
- `trigger` (glownie `on_idle`: `any-key | timeout | none`)

Semantyka:

- kroki (`steps`) ida sekwencyjnie,
- efekty wewnatrz kroku ida rownolegle,
- realny czas kroku to `max(duration, najdluzszy effect)`.

## 8.3 `pause` shorthand

Mozesz pisac:

```yaml
- pause: 1200ms
```

Normalizer zamienia to na:

- `duration: 1200`
- `effects: []`

Akceptowane jednostki: `ms` i `s` (oraz integer ms).

## 8.4 `menu-options` i alias `to`

W `menu-options`:

- `to:` jest aliasem routingowym,
- compiler dopelnia `scene` + `next`.

`key/label/to` to preferowany zapis authoringowy.

## 8.5 `stages-ref`

Scena moze ladowac preset lifecycle:

```yaml
stages-ref: /stages/my-preset.yml
```

Resolver obsluguje sciezki absolutne, wzgledne i nazwy bez rozszerzenia.

## 8.6 `sprite-defaults` i `frame-sequence`

`sprite-defaults` dziala jako domyslny zestaw pol dziedziczony przez sprite.

`type: frame-sequence` rozwija sekwencje klatek do timed image sprites
(`source-pattern`, `from/to`, `delay-ms`, `last-delay-ms`).

`menu-ui` moze generowac strukture sprite dla menu (grid + itemy + arrows)
na bazie `menu-options` i parametrow layoutu.

## 8.7 `input` profiles

Obecnie w schemie:

- `obj-viewer.sprite_id`
- `terminal-size-tester.presets`

Przyklady sa w `mods/playground/scenes/3d-scene/scene.yml` i `terminal-size-test/scene.yml`.

## 8.8 `audio` hooks

`audio.on_enter|on_idle|on_leave` przyjmuje liste:

- `cue` (required),
- `at_ms` (default 0),
- `volume` (0.0..1.0).

To jest deklaratywny hook; runtime emituje cue eventy.

---

## 9) Layer YAML

Layer:

- `name`
- `z_index`
- `visible`
- `stages` (layer lifecycle)
- `behaviors`
- `sprites`

W package partialu (`layers/*.yml`) plik jest **tablica layerow**.

---

## 10) Sprite YAML (wszystkie typy)

## 10.1 Typy sprite

- `text`
- `image`
- `obj`
- `grid`
- `flex`

## 10.2 Wspolne pola

- pozycja/alignment: `x`, `y`, `align_x`, `align_y`, alias `at`
- kolor: `fg_colour`/`bg_colour` + aliasy `fg`/`bg`
- lifecycle: `stages`
- animacje: `animations`
- zachowania runtime: `behaviors`
- timing: `appear_at_ms`, `disappear_at_ms`, `reveal_ms`, `hide_on_leave`
- `id` dla targetowania i resolvera

## 10.3 Specyficzne pola

- `text`: `content`, `font`, `force-font-mode`
- `image`: `source`, `width/height/size`
- `obj`: `source`, `scale`, rotacje, kamera (`camera-distance`, `fov-degrees`, `near-clip`), `surface-mode`, `draw-char`
- `grid`: `columns`, `rows`, `gap-x`, `gap-y`, `children`
- `flex`: `direction`, `gap`, `children`

## 10.4 Shorthand/alias dla sprite

Normalizer obsluguje m.in.:

- `fg` -> `fg_colour`
- `bg` -> `bg_colour`
- `at` -> (`align_x`, `align_y`)

---

## 11) Templates (`templates:` + `use:` + `args:`)

Scene moze miec lokalna mape template:

```yaml
templates:
  menu-item:
    type: text
    content: "$label"
```

Uzycie:

```yaml
- use: menu-item
  args:
    label: START
```

To jest reuse na poziomie authored YAML (makro), nie osobny runtime byt.

---

## 12) Objects/prefaby

## 12.1 Object document (`objects/*.yml`)

Docelowy plik reusable:

- `kind: object` (opcjonalny marker)
- `name` (required)
- `exports` (domyslne argumenty)
- `state` (poczatkowy stan, obecnie glownie kontrakt schema)
- `logic` (`native|graph|script`)
- `sprites` lub `layers` (material runtime)

Przyklad referencyjny:

- `mods/playground/objects/suzan.yml`

## 12.2 Instancja object w scenie (`scene.objects[]`)

Wspierane pola:

- `ref` (preferowane) lub `use`
- `as` (preferowane) lub `id`
- `with` (override `exports`)
- `state`
- `tags`

Przyklad:

```yaml
objects:
  - ref: suzan
    as: demo
    with:
      id: demo-sprite
      source: /scenes/3d/helsinki-university/city_scene_horizontal_front_yup.obj
```

## 12.3 Resolution rules dla `ref/use`

- absolutna sciezka (`/objects/x.yml`) -> bezposrednio,
- relatywna (`./`, `../`) -> wzgledem pliku sceny (`scene_source_path`),
- sama nazwa (`suzan`) -> `/objects/suzan.yml`.

## 12.4 `logic` w object

`logic.type: native` + `behavior` jest mapowane do runtime `behaviors` na warstwie/sprite po ekspansji.

`graph`/`script` sa obecnie zachowane jako granica authored API (przygotowanie pod kolejne etapy), ale nie sa pelnym wykonaniem skryptu.

---

## 13) Expressions - co jest realnie wspierane teraz

Aktualnie normalizer obsluguje:

- `x: oscillate(min,max,period)` -> bazowe `x` + animacja `float`,
- `y: oscillate(...)` analogicznie,
- `rotation-y: animate(start,end,duration,...)` -> statyczny start + `rotate-y-deg-per-sec`.

To sa kontrolowane, konkretne transformacje; nie ma jeszcze pelnego parsera uniwersalnych wyrazen.

---

## 14) Effects - zasady authoringu

## 14.1 Wspolny shape effect

W kroku stage:

```yaml
effects:
  - name: fade-in
    duration: 300
    params: { easing: ease-out }
```

Bazowe pole:

- `name` (enum built-in),
- `duration`,
- `looping` (opcjonalne),
- `params` (typy i ograniczenia z `effect-params.schema.yaml` + overlay per effect).

## 14.2 Target compatibility

Startup check `effect-registry` weryfikuje:

- czy effect istnieje,
- czy placement/target kind jest kompatybilny (scene/layer/sprite, bitmap/text, itd.).

To jest twarda walidacja startupowa - engine zatrzyma start przy bledzie.

## 14.3 Pelna lista built-in efektow (aktualny runtime)

Wbudowane efekty:

- `crt-on`
- `power-off`
- `fade-in`
- `fade-out`
- `fade-to-black`
- `clear-to-colour`
- `scanlines`
- `shine`
- `brighten`
- `lightning-flash`
- `lightning-branch`
- `lightning-growth`
- `lightning-optical-80s`
- `lightning-fbm`
- `lightning-ambient`
- `lightning-natural`
- `tesla-orb`
- `screen-shake`
- `whiteout`
- `glitch-out`
- `devour-out`
- `artifact-out`
- `shatter-glitch`

Praktyczne grupy (dla authora scen):

- Intro/CRT: `crt-on`, `power-off`, `scanlines`
- Fade/cleanup: `fade-in`, `fade-out`, `fade-to-black`, `clear-to-colour`, `whiteout`
- Lightning/cinematic: `lightning-*`, `tesla-orb`
- Distortion/glitch transition: `glitch-out`, `devour-out`, `artifact-out`, `shatter-glitch`
- Emphasis/motion: `shine`, `brighten`, `screen-shake`

## 14.4 Ktore efekty dzialaja gdzie (scope targetow)

**Scene-only** (nie wolno ich wieszac na sprite/layer target niekompatybilny):

- `crt-on`
- `power-off`
- `fade-to-black`
- `scanlines`
- `glitch-out`

**Scene + Layer**:

- `lightning-flash`
- `lightning-branch`
- `lightning-growth`
- `lightning-optical-80s`
- `lightning-fbm`
- `lightning-ambient`
- `lightning-natural`
- `tesla-orb`
- `screen-shake`

**Any target** (scene/layer/sprite):

- `fade-in`, `fade-out`
- `clear-to-colour`
- `shine`
- `brighten`
- `whiteout`
- `devour-out`
- `artifact-out`
- `shatter-glitch`

Jesli scope jest zly, startup check `effect-registry` zwroci blad i mod nie ruszy.

## 14.5 Parametry efektow - skad brac prawde

Single source of truth dla efektow to metadata efektu w kodzie (`engine-core/src/effects/metadata.rs` + `builtin/*`).

W praktyce author korzysta z:

- `schemas/effect.schema.yaml` (lista names + opis),
- `schemas/effect-params.schema.yaml` (wspolne params),
- `mods/<mod>/schemas/effects.yaml` (overlay z konkretnymi wariantami i podpowiedziami).

Uwaga praktyczna: `catalog.yaml` zbiera nazwy efektow znalezione w contentcie moda, wiec literowka w nazwie moze pojawic sie lokalnie w enumie/sugestii. Finalna walidacja i tak jest po stronie startup check (`effect-registry`).

## 14.6 Tranzycje scen - jak to naprawde dziala

Przejscie do kolejnej sceny dzieje sie po zakonczeniu `on_leave` i wykorzystuje:

- `next` z aktualnej sceny, albo
- `menu-options` (ustawia `next_scene_override`), gdy scena menu aktywuje wybor.

`scene_ref` moze byc:

- id sceny (np. `mainmenu`),
- sciezka (np. `/scenes/mainmenu/scene.yml`).

Resolver runtime obsluguje obie formy.

## 14.7 Typowe wzorce tranzycji (gotowe recipe)

### A) Auto cutscene -> nastepna scena po czasie

```yaml
stages:
  on_idle:
    trigger: timeout
    steps:
      - pause: 1800ms
next: intro.prologue
```

### B) "Press any key" -> wyjscie

```yaml
stages:
  on_idle:
    trigger: any-key
    looping: true
    steps:
      - pause: 1200ms
  on_leave:
    steps:
      - effects:
          - name: fade-out
            duration: 300
next: mainmenu
```

### C) Menu route (strzalki + enter + skroty key)

```yaml
stages:
  on_idle:
    trigger: any-key
menu-options:
  - key: "1"
    label: START
    to: game.start
  - key: "2"
    label: EXIT
    to: game.exit
```

### D) Brak przejscia

```yaml
next: null
```

## 14.8 Jak zrobic "swoja tranzycje" bez pisania nowego efektu

Najczesciej nie trzeba nowego efektu - wystarczy kompozycja `on_leave`:

1. W `on_leave.steps` laczysz kilka efektow rownolegle (`effects: [...]`) albo sekwencyjnie (kolejne kroki).
2. Dodajesz timing (`pause`, `duration`) pod dramaturgie.
3. Ustawiasz `next` albo `menu-options[].to`.

Przyklad "custom feel" z gotowych klockow:

```yaml
on_leave:
  steps:
    - effects:
        - name: screen-shake
          duration: 120
          params: { amplitude_x: 1.6, amplitude_y: 0.8, frequency: 18 }
        - name: whiteout
          duration: 120
    - effects:
        - name: glitch-out
          duration: 220
    - effects:
        - name: fade-to-black
          duration: 260
next: next-scene
```

To zwykle daje "wlasna tranzycje" bez ruszania Rusta.

## 14.9 Jak zrobic wlasny efekt (nowy built-in) - krok po kroku

W obecnym stanie projektu "wlasny efekt" = rozszerzenie engine w Rust (nie dynamiczny plugin YAML-only).

### Krok 1 - implementacja efektu

Dodaj plik:

- `engine-core/src/effects/builtin/<twoj_efekt>.rs`

Zaimplementuj:

- `struct TwojEfekt;`
- trait `Effect` (`apply(...)`)
- statyczne `METADATA: EffectMetadata` (name, summary, category, params, sample, compatible_targets).

### Krok 2 - ekspozycja modułu

W `engine-core/src/effects/builtin/mod.rs`:

- `pub mod <twoj_efekt>;`
- `pub use <twoj_efekt>::TwojEfekt;`

### Krok 3 - rejestracja w dispatcherze

W `engine-core/src/effects/mod.rs`:

- dopisz `self.registry.insert("twoj-efekt", Box::new(TwojEfekt));`
- dopisz nazwe do `EffectDispatcher::builtin_names()`.

### Krok 4 - aktualizacja bazowych schem

W `schemas/effect.schema.yaml`:

- dopisz nazwe do `properties.name.enum`,
- dopisz opis efektu.

W `schemas/effect-params.schema.yaml`:

- jesli dodajesz nowe klucze params, dopisz je tu (bo schema ma `additionalProperties: false`).

### Krok 5 - regeneracja overlay per mod

```bash
cargo run -p schema-gen -- --all-mods
cargo run -p schema-gen -- --all-mods --check
```

### Krok 6 - test i walidacja runtime

```bash
cargo test -p engine-core
cargo test -p engine
cargo test -p editor
```

Potem dodaj scene showcase w `mods/playground` z nowym efektem.

## 14.10 Minimalny szablon YAML dla nowego efektu

```yaml
stages:
  on_enter:
    steps:
      - effects:
          - name: twoj-efekt
            duration: 420
            params:
              easing: ease-in-out
```

Jesli nazwa/params sa poprawnie wpiete w kod + schemy, intellisense i startup checks beda spojne.

---

## 15) Startup walidacje (runtime gate)

Przy starcie moda `engine` uruchamia checki:

1. `terminal-requirements`
2. `scene-graph`
3. `effect-registry`
4. `image-assets`
5. `font-manifest`
6. `font-glyph-coverage`

Co to daje:

- brak nieistniejacych przejsc scen,
- brak nieistniejacych assetow/fontow,
- brak nieznanych lub zle targetowanych efektow,
- sprawdzenie wymagan terminala.

To jest kluczowe dla "blokuj build/run gdy YAML jest semantycznie zly".

---

## 16) Compile-time / CI gating (rekomendowany zestaw)

Minimalna bramka:

```bash
cargo run -p schema-gen -- --all-mods --check
cargo test -p engine real_playground_mod_manifest_and_entrypoint_load
cargo test -p engine real_shell_quest_mod_manifest_and_entrypoint_load
```

Pelna bramka (zalecana):

```bash
cargo run -p schema-gen -- --all-mods --check
cargo test -p engine-core
cargo test -p engine
cargo test -p editor
```

Dla lokalnego iterowania:

```bash
./refresh-schemas.sh --loop
```

---

## 17) Rola `playground` vs `shell-quest`

## 17.1 `mods/playground` (learning/reference mod)

Tu sa sceny "pokazowe":

- menu z `menu-options` + `to`,
- profile input (`obj-viewer`, `terminal-size-tester`),
- `flex` i `grid`,
- object reuse (`ref/as/with`) z `objects/suzan.yml`,
- rozne `target-fps` i `rendered-mode`.

To jest najlepsze miejsce do szybkich eksperymentow authoringowych.

## 17.2 `mods/shell-quest` (main mod)

Tu sa sceny produkcyjne:

- intro/logo/prologue/mainmenu jako package scenes,
- nacisk na czytelnosc flow, timing, efekty i comments,
- virtual viewport z mod-level `max-available`.

To jest wzorzec utrzymywania contentu scen gry.

---

## 18) Ograniczenia i "known caveats" (stan biezacy)

1. Scena uzywa jawnego `menu-options`; osobny top-level `menu` nie jest kontraktem sceny.
2. `scenes/<name>/effects/` nie jest merge'owane automatycznie do scene root.
3. Expressions sa ograniczone do konkretnych wzorcow (`oscillate` dla osi i `animate` dla `rotation-y`).
4. `state` i `tags` w instancji object sa obecne w schemie kontraktowej, ale runtime nie konsumuje wszystkich przypadkow.

To nie blokuje codziennej pracy, ale warto o tym pamietac przy projektowaniu nowych feature'ow.

---

## 19) Checklist autora przed commit

1. Czy kazdy nowy YAML ma poprawny `$schema`?
2. Czy scene refs (`next`, `menu-options[].to/next`) prowadza do istniejacych scen?
3. Czy `ref` do object ma poprawna sciezke/nazwe?
4. Czy wszystkie `source` (image/obj) istnieja?
5. Czy fonty custom maja manifest i potrzebne glify?
6. Czy `schema-gen --check` przechodzi?
7. Czy startup smoke testy moda przechodza?

Jak tak, to authoring jest gotowy do dalszej pracy.

---

## 20) Komendy operacyjne (skrót)

Uruchomienie gry (default shell-quest):

```bash
cargo run -p app
```

Uruchomienie gry na playground:

```bash
SHELL_QUEST_MOD_SOURCE=mods/playground cargo run -p app
```

Uruchomienie edytora:

```bash
cargo run -p editor -- --mod-source mods/shell-quest
```

Regeneracja schem:

```bash
cargo run -p schema-gen -- --all-mods
```

Sprawdzenie driftu schem:

```bash
cargo run -p schema-gen -- --all-mods --check
```

Watch co 5s:

```bash
./refresh-schemas.sh --loop
```

---

## 21) Finalna zasada pracy

W tym projekcie YAML jest zrodlem prawdy dla contentu moda, ale:

- ergonomie daje warstwa authored (aliasy/shorthand/templates/objects),
- spojnosc daje schema overlay per mod,
- bezpieczenstwo daje startup + test gates.

Dlatego pracujemy zawsze cyklem:

**zmiana YAML -> regeneracja schem -> check -> startup/testy -> dalej.**
