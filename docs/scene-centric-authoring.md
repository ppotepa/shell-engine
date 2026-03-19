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

## 6) Skróty authoringu wspierane przez compiler

- `pause: 1200ms` w `steps`.
- `menu-options[].to` jako alias routingu.
- `sprite-defaults` (dziedziczenie pól sprite między poziomami).
- `type: frame-sequence` (rozwinięcie do timed image sprites).
- `type: window` (rozwinięcie do framed `grid` z sekcjami title/body/footer).
- `type: scroll-list` (rozwinięcie do `grid` z itemami listy, opcjonalnie z `menu-carousel`).
- `cutscene-ref` (rozwinięcie do timed image sprites przez manifest cutsceny).

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

Dla skryptów wykrywane są sidecary przy scenie:

- `*.rhai`,
- `*.logic.rhai`,
- `*.logic.yml`.

Dla scen pakietowych (`scenes/<name>/scene.yml`) wykrywanie sprawdza kolejno:

- `<name>.rhai`,
- `scene.rhai`,
- `<name>.logic.rhai`,
- `scene.logic.rhai`,
- `<name>.logic.yml`,
- `scene.logic.yml`.

Przykład działających scen demonstracyjnych:

- `mods/playground/scenes/rhai-lab/scene.yml`,
- `mods/playground/scenes/rhai-time/scene.yml`,
- `mods/playground/scenes/rhai-focus/scene.yml`,
- `mods/playground/scenes/rhai-object/scene.yml`.

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

## 13) Known Limits: `rhai-script`

Aktualnie behavior `rhai-script` obsługuje tylko komendy:

- `visibility` (`target`, `visible`)
- `offset` (`target`, `dx`, `dy`)

Zakres danych w scope obejmuje:

- `selected_index`,
- `menu_count`,
- `scene_elapsed_ms`,
- `stage_elapsed_ms`,
- `params`,
- `regions`.
- `ui` (`focused_target`, `has_submit`, `submit_target`, `submit_text`, `has_change`, `change_target`, `change_text`)
- aliasy flat: `ui_focused_target`, `ui_submit_target`, `ui_submit_text`, `ui_change_target`, `ui_change_text`, `ui_has_submit`, `ui_has_change`.

`ui.theme`:

- opcjonalny identyfikator motywu UI dla sceny (np. `terminal`, `win98`, `jrpg`),
- w kolejnych etapach będzie rozwiązywany przez rejestr motywów runtime.

`ui.focus-order`:

- definiuje kolejność focusu dla targetów UI,
- `Tab` przechodzi do następnego targetu, `Shift+Tab` do poprzedniego,
- przy `input.terminal-shell` edycja promptu i `Esc`-back działają tylko gdy fokus jest na `prompt-sprite-id`,
- gdy `focus-order` jest puste i scena ma `terminal-shell`, focus domyślnie trafia na prompt.

Brak wsparcia dla wykonywania dowolnego kodu gameplay/API poza tym kontraktem komend i danymi scope.
