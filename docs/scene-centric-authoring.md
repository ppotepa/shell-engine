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
4. wynik deserializuje się do runtime `Scene`,
5. runtime wykonuje lifecycle/input/render na tym modelu.

## 10) Minimalna checklista autora

1. Każdy nowy YAML ma poprawny `$schema`.
2. Referencje `next` i `menu-options[].to` wskazują istniejące sceny.
3. Referencje `ref/use` wskazują istniejące pliki/obiekty.
4. `./refresh-schemas.sh` i `schema-gen --check` przechodzą.
5. Smoke run moda startuje bez błędów kompilacji scen.

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
let visible = (scene_elapsed_ms / 300) % 2 == 0;
commands.push(#{ op: "visibility", target: "demo-label", visible: visible });
commands
```

## 12) Menu Troubleshooting Checklist

Jeśli menu nie reaguje na `Enter`, strzałki lub `menu-options[].key`, sprawdź:

1. Scena jest w `on_idle`.
2. `on_idle.trigger` ma wartość `any-key`.
3. `menu-options` nie jest puste i ma poprawne klucze `key`.
4. Każda pozycja ma poprawny cel routingu (`to`, `next` lub `scene`).
5. Docelowe sceny istnieją i mają poprawne `id`.
6. Po zmianach został odświeżony schemat (`./refresh-schemas.sh`) i walidacja jest zielona.

## 13) Rhai Scope (skrót)

Zakres danych w scope obejmuje:

- `selected_index`,
- `menu_count`,
- `scene_elapsed_ms`,
- `stage_elapsed_ms`,
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
- aliasy flat: `ui_focused_target`, `ui_theme`, `ui_submit_target`, `ui_submit_text`, `ui_change_target`, `ui_change_text`, `ui_has_submit`, `ui_has_change`.

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

- helpery `scene_get(target)` i `scene_set(target, path, value)` nadal działają,
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

- `prompt-panel-id` wiąże prompt z panelem UI i aktywuje layout-aware wrapping,
- `prompt-shadow-panel-id` (opcjonalnie) synchronizuje wysokość panelu cienia przy auto-grow,
- `prompt-wrap` włącza zawijanie linii do szerokości panelu,
- `prompt-auto-grow` + `prompt-min-lines`/`prompt-max-lines` rozszerzają panel wraz z liczbą linii,
- `prompt-growth-ms` ustala czas animacji wzrostu wysokości panelu.
- `type: terminal-input` nie renderuje tytułu domyślnie; pasek tytułu pojawia się tylko po podaniu `title-bar`.
- `type: terminal-input` układa sloty sekwencyjnie wg wysokości fontu; prompt rezerwuje minimum jedną linię nawet gdy jest pusty.
- renderer tekstu zachowuje transparentne tło (`bg=Reset`) i nie nadpisuje kompozytowego tła pustymi komórkami.

Brak wsparcia dla wykonywania dowolnego kodu gameplay/API poza tym kontraktem komend i danymi scope.

## 14) Intro CPU-On: paged boot logs

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
