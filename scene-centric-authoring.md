# Scene-Centric Authoring

Ten dokument opisuje aktualny kontrakt authoringu YAML w Shell Quest.

## 1) Struktura modu

Standardowy układ:

```text
mods/<mod>/
├── mod.yaml
├── objects/
│   └── *.yml
├── stages/
│   └── *.yml
└── scenes/
    ├── foo.yml
    └── bar/
        ├── scene.yml
        ├── layers/*.yml
        ├── templates/*.yml
        └── objects/*.yml
```

Scena może być:

- pojedynczym plikiem `scenes/*.yml`,
- pakietem `scenes/<name>/scene.yml` z partialami.

## 2) Kontrakt sceny

`scene.yml` odpowiada za:

- identyfikację (`id`, `title`),
- lifecycle (`stages` lub `stages-ref`),
- kolejność kompozycji (`layers`),
- post-process pipeline (`postfx`),
- kontrakt UI (`ui.enabled`, `ui.persist`, `ui.theme`, `ui.focus-order`),
- routing (`next`, `menu-options`),
- profile wejścia (`input`).

Przykład:

```yaml
id: playground-menu
title: Playground Menu
bg: black
layers:
  - ref: main
stages-ref: /stages/anykey-loop-1200-fade-250-180.yml
next: playground-3d-scene
menu-options:
  - { key: "1", label: "3D SCENE", to: playground-3d-scene }
  - { key: "2", label: "TERMINAL", to: playground-terminal-shell }
```

## 2.1) PostFX (screen-space passy)

`postfx` to lista passów wykonywanych po compositorze i przed rendererem terminala.

Aktualny runtime:

- etap pipeline: `compositor_system -> postfx_system -> renderer_system`,
- `postfx` działa na finalnym buforze sceny (screen-space),
- pass ma dostęp do historii poprzedniej klatki (`previous_output`) i licznika klatek,
- passy są wykonywane sekwencyjnie w kolejności wpisów `postfx`.

Przykład:

```yaml
postfx:
  - name: crt-underlay
    params:
      intensity: 1.0
      transparency: 0.35
      brightness: 1.1
      speed: 0.45
  - name: crt-distort
    params:
      intensity: 0.3
      sphericality: 0.3
      transparency: 0.25
      brightness: 1.0
      speed: 0.3
  - name: crt-scan-glitch
    params:
      intensity: 0.25
      transparency: 0.25
      brightness: 1.0
      speed: 0.8
  - name: crt-ruby
    params:
      intensity: 0.2
      transparency: 0.2
      brightness: 0.98
      speed: 0.55
```

Uwagi:

- kolejność wpisów `postfx` jest kolejnością wykonania pipeline i wpływa na finalny obraz,
- `crt-underlay`, `crt-distort`, `crt-scan-glitch`, `crt-ruby` działają jako dedykowane passy runtime,
- `terminal-crt` działa jako alias wsteczny (`params.coverage` mapuje do nowych passów),
- pozostałe nazwy passów są traktowane jako fallback do zwykłego dispatcher-a efektów na pełnym ekranie.

## 3) Kontrakt warstwy

Warstwa opisuje render:

- `name`, `z_index`, `visible`,
- `ui` (oznacza warstwę jako interfejs użytkownika),
- `sprites`,
- opcjonalnie `behaviors`, `stages`, `objects`.

W partialu `layers/*.yml` plik jest tablicą warstw.

Przykład minimalnego kontraktu UI:

```yaml
id: ui-demo
title: UI Demo
ui:
  enabled: true
  persist: scene
  theme: terminal
  focus-order: [terminal-prompt]
layers:
  - name: world
    z_index: 0
    sprites: []
  - name: hud
    z_index: 10
    ui: true
    sprites: []
```

## 4) Kontrakt obiektu

Obiekt (`objects/*.yml`) jest prefabem wielokrotnego użycia:

- `name`,
- opcjonalne `exports`,
- `sprites`,
- opcjonalne `logic`.

Instancjonowanie:

- na poziomie sceny: `scene.objects`,
- na poziomie warstwy: `layer.objects`.

Obsługiwane aliasy instancji:

- `ref` lub `use`,
- `as` lub `id`,
- `with` dla override eksportów.

## 5) Reużywalne presety sceny

`stages-ref` pozwala wyciągać lifecycle do osobnych plików:

```yaml
stages-ref: /stages/anykey-fade-250-200.yml
```

Rozwiązywanie ścieżek:

- absolutna: `/stages/foo.yml`,
- względna: `./foo.yml`, `../foo.yml`,
- nazwa: `foo` -> `/stages/foo.yml`.

Scena może jednocześnie mieć lokalne `stages`; lokalne pola nadpisują preset.

`effect-presets` (alias: `effect_presets`) pozwala definiować reusable konfiguracje efektów
na poziomie sceny i używać ich w `effects` przez `use`/`preset`/`ref`.

Aktualne reguły walidacji presetów:

- nie można jednocześnie użyć `effect-presets` i `effect_presets` w jednym dokumencie,
- wpis efektu nie może mieszać aliasów `use`/`preset`/`ref` jednocześnie,
- brakująca referencja presetu zwraca błąd kompilacji sceny,
- tabela presetów musi być mapą, a alias presetu musi być stringiem.

Przykład:

```yaml
effect-presets:
  lightning-soft:
    name: lightning-natural
    duration: 800ms
    params:
      intensity: 0.8
      strikes: 2

stages:
  on_idle:
    steps:
      - effects:
          - use: lightning-soft
            overrides:
              params:
                intensity: 1.1
```

## 6) Skróty authoringu wspierane przez compiler

- `pause: 1200ms` w `steps`.
- `menu-options[].to` jako alias routingu.
- `sprite-defaults` (dziedziczenie pól sprite między poziomami).
- `type: frame-sequence` (rozwinięcie do timed image sprites).
- `type: window` (rozwinięcie do `panel` z sekcjami title/body/footer).
- `type: terminal-input` (rozwinięcie do `window` z semantycznymi slotami hint/prompt dla terminalowego inputu).
- `type: scroll-list` (rozwinięcie do `grid` z itemami listy, opcjonalnie z `menu-carousel`).
- `cutscene-ref` (rozwinięcie do timed image sprites przez manifest cutsceny).

Szczegółowy opis kontraktu `window`/`terminal-input` i terminalowego HUD:

- `docs/terminal-hud-authoring.md`

## 7) Menu i kompozycja UI

Menu jest opisane przez:

- `menu-options` (routing i etykiety),
- sprite/object layout (grid/flex/text),
- behaviors sterujące offset/visibility.
- `stages.on_idle.trigger: any-key` (warunek działania nawigacji menu w runtime).

W praktyce używane są dwa warianty:

- behavior wbudowany (`menu-carousel-object`, `selected-arrows`),
- behavior skryptowy Rhai (`rhai-script`) z sidecar `menu.rhai`.

Praktyczna zasada layoutu:

- jeśli scena ma kilka warstw UI, nie ustawiaj wszystkich root kontenerów na `at: cc` bez offsetów,
- niezależnie centrowane rooty różnych warstw nie „wiedzą” o sobie i będą się nakładać,
- dla czytelnego HUD najlepiej:
  - użyć jednego root layoutu dla całej kompozycji UI, albo
  - stosować jawne anchory (`ct`/`cb`/`lt`/`rt`) i separację `y/x` między warstwami.

## 8) Logika sceny (native/script)

`scene.logic` wspiera:

- `type: native` z `behavior`,
- `type: script` z `src`.

Logika sceny jest ładowana tylko przez jawny blok `logic:`. Compiler nie wykonuje
automatycznego wykrywania sidecarów `*.rhai` / `*.logic.rhai` / `*.logic.yml`
jeśli `logic:` nie jest zdefiniowane.

`logic.kind: graph` jest obecnie traktowane jako tryb eksperymentalny i jest
odrzucane przez compiler.

`logic.kind: script` wymaga jawnego `src` (brak fallbacku do auto-odkrywania plików skryptu).

Przykład działających scen demonstracyjnych:

- `mods/playground/scenes/rhai-lab/scene.yml`,
- `mods/playground/scenes/rhai-time/scene.yml`,
- `mods/playground/scenes/rhai-focus/scene.yml`,
- `mods/playground/scenes/rhai-object/scene.yml`,
- `mods/playground/scenes/rhai-text-lab/scene.yml`,
- `mods/playground/scenes/rhai-image-lab/scene.yml`.

## 9) Ścieżka kompilacji

Pipeline:

1. repo ładuje scenę (single-file albo package),
2. `engine-authoring` rozwija `layers ref`, `objects`, `stages-ref`, `cutscene-ref`,
3. normalizer rozwija aliasy i shorthandy,
4. wynik deserializuje się do runtime `Scene` (w tym `postfx`),
5. **walidacja timeline** sprawdza sprite timing vs scene duration (debug mode),
6. runtime wykonuje lifecycle/input/compositor/postfx/render na tym modelu.

## 9.1) Timeline i sprite visibility

### Timing absolutny względem sceny

Sprite timing (`appear_at_ms`, `disappear_at_ms`) jest **absolutny względem startu sceny** (początku `on_enter`), NIE relatywny do layer lifecycle:

```yaml
stages:
  on_enter:
    steps:
      - duration: 6000  # Scene trwa 6s

layers:
  - name: intro
    sprites:
      - type: text
        id: title
        content: "Welcome"
        appear_at_ms: 1000    # 1s od STARTU SCENY
        disappear_at_ms: 5000 # 5s od STARTU SCENY
```

**Ważne**: Jeśli sprite ma `appear_at_ms >= scene_duration`, **NIGDY nie będzie widoczny** podczas `on_enter`. Walidacja w debug mode ostrzeże o tym błędzie authoringu.

### Layer visibility

Layer ma **statyczny boolean** `visible`:

```yaml
layers:
  - name: overlay
    visible: false  # Layer skipowany przez compositor
    sprites: []
```

**Ograniczenia**:
- Layer NIE MA pól `appear_at_ms` / `disappear_at_ms`
- `layer.visible` nie kaskaduje kontroli timeline do children
- Sprite timing jest niezależny od layer visibility (każdy sprite sprawdza swoje okno)

Runtime control przez Rhai:
```rhai
scene.set("overlay-layer", "visible", false);
```

### Walidacja timeline (debug mode)

Kompilator sprawdza sprite timing w `engine-authoring` (tylko debug builds):

```
⚠️  Scene 'intro-cpu-on': sprite #2 in layer 'terminal' has appear_at_ms=8200 
    but on_enter ends at 6000ms (sprite will never be visible)
```

Checky:
1. **SpriteAppearsAfterSceneEnd**: sprite pojawia się po zakończeniu `on_enter`
2. **SpriteDisappearsBeforeAppear**: sprite znika przed pojawieniem się

Uruchom debug build aby zobaczyć ostrzeżenia:
```bash
cargo build  # debug: walidacja + ostrzeżenia
cargo build --release  # release: bez walidacji
```

### Best practices

1. **Sprite timing w ramach scene duration**:
   ```yaml
   # ✅ Dobrze
   stages:
     on_enter:
       steps:
         - duration: 6000
   layers:
     - sprites:
         - appear_at_ms: 1000
           disappear_at_ms: 5500  # < 6000
   ```

2. **Layer visibility dla runtime control**:
   - Użyj `visible: false` dla static layerów
   - Użyj Rhai `scene.set(layer, "visible", ...)` dla dynamic control

3. **Scene duration = on_enter**:
   - `on_enter` to główna faza cutscene z timed sprites
   - `on_idle` to event-driven (any-key, timeout)
   - `on_leave` to transition effects (zwykle krótkie)

Pełna dokumentacja: `timeline-architecture.md`

## 10) Minimalna checklista autora

1. Każdy nowy YAML ma poprawny `$schema`.
2. Referencje `next` i `menu-options[].to` wskazują istniejące sceny.
3. Referencje `ref/use` wskazują istniejące pliki/obiekty.
4. `./refresh-schemas.sh` i `schema-gen --check` przechodzą.
5. Smoke run moda startuje bez błędów kompilacji scen.
6. **Sprite timing w ramach scene duration** (walidacja ostrzeże w debug builds).

## 11) Quick Start Rhai (minimalny przykład)

Pakiet sceny:

```text
mods/<mod>/scenes/demo-rhai/
├── scene.yml
├── demo-rhai.rhai
└── layers/main.yml
```

`scene.yml`:

```yaml
# yaml-language-server: $schema=../../schemas/scenes.yaml
id: demo-rhai
title: Demo Rhai
bg: black
layers:
  - ref: main
stages-ref: /stages/anykey-fade-250-200.yml
next: demo-rhai
menu-options:
  - { key: "1", label: "SELF", to: demo-rhai }
```

`layers/main.yml`:

```yaml
# yaml-language-server: $schema=../../../schemas/layers.yaml
- name: main
  z_index: 0
  sprites:
    - { type: text, id: demo-label, content: "RHAI", at: cc, font: "generic:1", fg: white }
```

`demo-rhai.rhai` (auto-detekcja sidecar):

```rhai
let commands = [];
let visible = (time.scene_elapsed_ms / 300) % 2 == 0;
commands.push(#{ op: "visibility", target: "demo-label", visible: visible });
commands
```

## 11) Menu Troubleshooting Checklist

Jeśli menu nie reaguje na `Enter`, strzałki lub `menu-options[].key`, sprawdź:

1. Scena jest w `on_idle`.
2. `on_idle.trigger` ma wartość `any-key`.
3. `menu-options` nie jest puste i ma poprawne klucze `key`.
4. Każda pozycja ma poprawny cel routingu (`to`, `next` lub `scene`).
5. Docelowe sceny istnieją i mają poprawne `id`.
6. Po zmianach został odświeżony schemat (`./refresh-schemas.sh`) i walidacja jest zielona.

## 11) Rhai Scope (skrót)

Zakres danych w scope obejmuje:

- `menu.selected_index`,
- `menu.count`,
- `time.scene_elapsed_ms`,
- `time.stage_elapsed_ms`,
- `params`,
- `regions`.
- `objects` (snapshot obiektów po runtime `id` i aliasach target resolvera):
  - `id`, `kind`,
  - `state.visible`, `state.offset_x`, `state.offset_y`,
  - `props.visible`, `props.offset.x`, `props.offset.y`,
  - `props.text.content`, `props.text.font`, `props.text.fg`, `props.text.bg` (dla text sprite),
  - `props.style.fg`, `props.style.bg`,
  - `props.obj.scale`, `props.obj.yaw`, `props.obj.pitch`, `props.obj.roll`, `props.obj.orbit_speed`, `props.obj.surface_mode` (dla obj sprite),
  - `region` (jeśli dostępny),
  - `text.content`/`props.text.content` dla text sprite.
- `state` (persistowany między tickami, gdy skrypt zwróci `#{ state: ... }`)
- `ui` (`focused_target`, `theme`, `has_submit`, `submit_target`, `submit_text`, `has_change`, `change_target`, `change_text`)
- `game` (`get(path)`, `set(path, value)`, `has(path)`, `remove(path)`, `push(path, value)`)

Kompatybilność tymczasowa:

- flat aliasy UI: `ui_focused_target`, `ui_theme`, `ui_submit_target`, `ui_submit_text`, `ui_change_target`, `ui_change_text`, `ui_has_submit`, `ui_has_change`
- flat globale czasu/menu: `selected_index`, `menu_count`, `scene_elapsed_ms`, `stage_elapsed_ms`
- helpery `scene_get(target)` i `scene_set(target, path, value)`
- helpery `game_get(path)`, `game_set(path, value)`, `game_has(path)`, `game_remove(path)`, `game_push(path, value)`
- powyższe helpery są utrzymywane tylko jako warstwa compatibility; nowy kod powinien używać API obiektowego.

Pełny kontrakt komend, `scene.get/set`, `obj.get/set` i path-based mutacji:
- `scene.get(target)` -> uchwyt obiektu,
- `scene.set(target, path, value)` -> zapis property,
- `obj.get(path)` / `obj.set(path, value)` na uchwycie obiektu.

Wspierane ścieżki `set(path, value)`:

- `visible` (`bool`),
- `position.x` / `offset.x` (`int`),
- `position.y` / `offset.y` (`int`),
- `text.content` (`string`),
- `text.font` (`string`),
- `style.fg` / `text.fg` (`string`, kolor nazwany lub `#rrggbb`),
- `style.bg` / `text.bg` (`string`, kolor nazwany lub `#rrggbb`),
- `obj.scale` (`number`),
- `obj.yaw` (`number`),
- `obj.pitch` (`number`),
- `obj.roll` (`number`),
- `obj.orbit_speed` (`number`),
- `obj.surface_mode` (`string`).

Kompatybilność:

- stare helpery `scene_get(target)` i `scene_set(target, path, value)` nadal działają, ale nowy kod powinien używać `scene.get/set`,
- mapy komend `#{ op: "...", ... }` nadal działają.

Przykład obiektowego API:

```rhai
let core = scene.get("object-core");
let y = core.get("state.offset_y");
core.set("position.y", y + 1);
scene.set("object-status", "text.content", "READY");
```

`ui.theme`:

- opcjonalny identyfikator motywu UI dla sceny (np. `terminal`, `win98`, `jrpg`),
- aktualnie wpływa na domyślne wartości sugar `type: window`, `type: terminal-input` i `type: scroll-list` (kolory panelu, obramowania, cienia, sekcji/listy),
- runtime rozwiązuje `ui.theme` przez wspólny registry presetów (`engine-core`),
- gdy `ui.theme` nie jest podany lub jest nieznany, używany jest fallback `engine-default`,
- jawne pola sprite (`border-fg`, `border-bg`, `panel-bg`, `title-fg`, `body-fg`, `footer-fg`, `fg-selected`, `fg-alt-a`, `fg-alt-b`) zawsze mają priorytet nad theme defaults.
- `type: window` domyślnie kompiluje się do kompaktowego `panel` z lekkim `corner-radius` i cieniem; bez tekstowych ramek ASCII.
- `type: window` zachowuje `width-percent` (nie jest nadpisywane przez fallback `width`).
- `type: window` wspiera alias `title-bar` (`title_bar`) dla paska tytułu.
- `type: window` renderuje `title` w nagłówku (`top-center`), a `body/footer` poniżej.
- pionowe offsety slotów `title/body/footer` są liczone wg wysokości tekstu i fontu (np. `generic:half`), więc sloty nie powinny się nakładać.
- `type: window` ma domyślne `padding: 0`, żeby układ `title/body/footer` mieścił się w kompaktowych wysokościach (np. `height: 5`).
- jeśli `width`/`width-percent` oraz `height` nie są podane, panel jest traktowany jako `autosize` (rozmiar wynika z zawartości i insetu zamiast rozciągania na cały obszar).

`ui.focus-order`:

- definiuje kolejność focusu dla targetów UI,
- `Tab` przechodzi do następnego targetu, `Shift+Tab` do poprzedniego,
- przy `input.terminal-shell` edycja promptu i `Esc`-back działają tylko gdy fokus jest na `prompt-sprite-id`,
- gdy `focus-order` jest puste i scena ma `terminal-shell`, focus domyślnie trafia na prompt.

`input.terminal-shell` (prompt widget):

- `mode` (opcjonalnie): `builtin` (domyślnie) lub `scripted`.
  - `builtin`: silnik wykonuje wbudowane komendy (help, clear, ls, pwd, echo, whoami).
  - `scripted`: silnik pomija wykonanie komend; tylko emituje UI submit/change events dla skryptów.
- `prompt-panel-id` wiąże prompt z panelem UI i aktywuje layout-aware wrapping,
- `prompt-shadow-panel-id` (opcjonalnie) synchronizuje wysokość panelu cienia przy auto-grow,
- `prompt-wrap` włącza zawijanie linii do szerokości panelu,
- `prompt-auto-grow` + `prompt-min-lines`/`prompt-max-lines` rozszerzają panel wraz z liczbą linii,
- `prompt-growth-ms` ustala czas animacji wzrostu wysokości panelu.
- `type: terminal-input` nie renderuje tytułu domyślnie; pasek tytułu pojawia się tylko po podaniu `title-bar`.
- `type: terminal-input` układa sloty sekwencyjnie wg wysokości fontu; prompt rezerwuje minimum jedną linię nawet gdy jest pusty.
- renderer tekstu zachowuje transparentne tło (`bg=Reset`) i nie nadpisuje kompozytowego tła pustymi komórkami.

W trybie `scripted` dostępne są Rhai API dla kontroli transkryptu:

- `terminal.push(line)` - dodaje linię do transkryptu wyjściowego
- `terminal.clear()` - czyści transkrypt wyjściowy

Dla scen `terminal-shell` w trybie `sidecar` dostępny jest także obiekt Rhai `ipc`
(snapshot eventów z bridge `engine-io`, tylko do odczytu):

- `ipc.has_output` (`bool`) — czy w tej klatce sidecar dostarczył nowe linie
- `ipc.output_lines` (`array<string>`) — linie `out` / `screen-diff`
- `ipc.clear_count` (`int`) — ile razy sidecar wyemitował `clear` w tej klatce
- `ipc.has_screen_full` (`bool`) — czy sidecar wysłał `screen-full`
- `ipc.screen_full_lines` (`array<string>`) — payload `screen-full`
- `ipc.custom_events` (`array<string>`) — surowe payloady zdarzeń `custom`

Przykład:

```rhai
if ipc.has_output {
  for line in ipc.output_lines {
    if line.contains("ALERT") {
      game.set("/session/alert", true);
    }
  }
}
if ipc.clear_count > 0 {
  terminal.push("screen was cleared by sidecar");
}
```

Przykład użycia trybu scriptowanego (zobacz `mods/playground/scenes/terminal-shell-scripted`):

```rhai
if ui.has_submit && ui.focused_target == "prompt-sprite-id" {
  let cmd = ui.submit_text.trim();
  if cmd == "status" {
    terminal.push("power: online");
    terminal.push("hull: 92%");
  } else {
    terminal.push("unknown command: " + cmd);
  }
}
```

Brak wsparcia dla wykonywania dowolnego kodu gameplay/API poza tym kontraktem komend i danymi scope.

## 11) Intro CPU-On: paged boot logs

Scena `mods/shell-quest/scenes/04-intro-cpu-on` używa skryptu `scene.rhai` do stronicowania
logów bootowania, żeby długie sekwencje nie nakładały się na siebie i mieściły się w obszarze
BIOS-owego panelu.

Aktualny podział:

- `Page 1/4`: boot sector + secondary loader
- `Page 2/4`: kernel init + zegar
- `Page 3/4`: storage + fs check
- `Page 4/4`: init + login

Dla statusów używany jest inline markup w `set-text`:

- `[#55ff55]SUCCESS[/]`
- `[#ff5555]WARNING[/]`
- `[#ff5555]FAIL[/]`

Sugerowany wzorzec dla podobnych cutscenek:

1. Trzymać stały layout (`bios-line-*`, `bios-footer-*`) w YAML.
2. Stronicowanie i treść robić wyłącznie w `scene.rhai`.
3. Na każdą klatkę strony najpierw czyścić sloty, potem wpisywać aktywną stronę.

## 11) Rhai — pułapki i guardrails

### Stringi wieloliniowe — ZAWSZE backticki

Rhai **nie** obsługuje dosłownych nowych linii w `"..."` ani sekwencji `\n` w `"..."`.
Jedyne bezpieczne podejście dla tekstu wieloliniowego to backtick template:

```rhai
// ❌ ŹLE — kompiluje się, ale \n nie jest zamieniane na nową linię
let msg = "Linia 1\nLinia 2";

// ❌ ŹLE — błąd kompilacji w Rhai (literalna nowa linia w "...")
let msg = "Linia 1
Linia 2";

// ✅ DOBRZE — backtick template ze znakiem nowej linii
let msg = `Linia 1
Linia 2`;

// ✅ DOBRZE — backtick template z \n
let msg = `Linia 1\nLinia 2`;
```

**Reguła:** jeśli treść tekstu ma więcej niż jedną linię → backtick.

### Błędy składniowe są teraz widoczne w debug overlay

Silnik kompiluje skrypt przy starcie sceny (`from_params`).
Jeśli jest błąd składniowy, pojawi się **czerwona linia** w debug overlay (uruchom z `--debug-feature`):

```
[ERR ] scena-id | ./scene.rhai | compile error: ...
```

Scena nadal ładuje się, ale skrypt nie wykonuje się — na ekranie zostaje ostatni dobry frame.

### Tryb debugowania

```bash
SHELL_QUEST_MOD_SOURCE=mods/shell-quest cargo run -p app -- --debug-feature
```

- **F1** — toggle overlay (pokazuje scene id, virtual size, ostatnie 5 błędów)
- Błędy Rhai (compile i runtime) pojawiają się w overlay z czerwonym tłem
- Przy aktywnym błędzie silnik zachowuje ostatni dobry frame zamiast pokazywać czarny ekran

### Dual-prompt pattern (sceny z terminal-shell)

Sceny używające `input-profile: terminal-shell` mają **ukryty prawdziwy prompt** (np. `login-hidden-prompt`)
i **rysowany ręcznie fake prompt** w text sprite. Jeśli skrypt failuje:
- ukryty prompt pozostaje niewidoczny
- fake prompt nigdy się nie aktualizuje
- efekt: czarny / zamrożony ekran

Rozwiązanie: uruchomić z `--debug-feature` i sprawdzić overlay po błędach.

### Canonical example

`mods/shell-quest/scenes/06-intro-login/scene.rhai` — pełny przykład:
- backtick strings dla komunikatów terminalowych
- state machine (`init` / `login` / `password` / `shell`)
- obsługa `ui.has_submit` i `ui.has_change`
- terminal API (`terminal.push_output`, `terminal.clear`)
