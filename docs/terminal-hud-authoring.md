# Terminal HUD Authoring (Playground + Runtime)

Ten dokument opisuje aktualny kontrakt silnika dla komponowania terminalowego UI/HUD w YAML.
Zakres: `type: window`, `type: terminal-input`, `input.terminal-shell`, style, layout i reguły renderingu.

## 1) Cel i model kompozycji

Docelowy układ terminalowy jest składany z warstw:

1. Tło świata/output (`type: text` lub inne sprite'y) na warstwie bazowej.
2. Okna UI (`type: window` / `type: terminal-input`) na warstwie `ui: true`.
3. Input profile `terminal-shell`, który zapisuje prompt/output do wskazanych sprite'ów.

Praktyczny wzorzec referencyjny:

- `mods/playground/scenes/terminal-shell/scene.yml`
- `mods/playground/scenes/terminal-shell/layers/bg.yml`
- `mods/playground/scenes/terminal-shell/layers/windows.yml`

## 2) `type: window` (sugar -> `panel`)

`type: window` kompiluje się do `type: panel` z trzema slotami tekstowymi:

- `title` (`title-bar` alias),
- `body` (`body-content` aliasy),
- `footer` (`footer-content` aliasy).

### 2.1 Reguły układu slotów

- `title` renderuje się w nagłówku okna (`at: ct`, top-center).
- `body` startuje poniżej `title`.
- `footer` startuje poniżej `body`.
- Offsety pionowe są liczone z realnej wysokości slotu i fontu (np. `generic:half` = 4 linie na 1 wiersz tekstu), więc sloty nie nakładają się.

To dotyczy też fontów `generic:*` i eliminuje przypadki, gdzie `body` pisało po `title`.

### 2.2 Styling i theme

Można podać bezpośrednio:

- `border-fg`, `border-bg`, `panel-bg`,
- `title-fg`, `body-fg`, `footer-fg`,
- `corner-radius`, `shadow-x`, `shadow-y`.

Jeśli pole nie jest ustawione, używane są domyślne wartości z `ui.theme` (albo fallback `engine-default`).

## 3) `type: terminal-input` (sugar)

`terminal-input` to wyspecjalizowany `window` dla promptu:

- wspiera `title-bar`,
- wspiera `hint-content`,
- wspiera `prompt-id` (slot promptu),
- domyślnie ustawia layout pod dolny pasek inputu (`at: cb`, `width-percent: 95`, `padding: 1`).

Reguły slotów:

- gdy `title-bar` nie jest podany, slot tytułu nie jest renderowany,
- gdy `hint-content` nie jest podany, slot hintu nie jest renderowany,
- slot promptu zawsze rezerwuje minimum jedną linię (nawet gdy pusty), żeby input nie znikał.

## 4) `input.terminal-shell` (runtime)

Profil `terminal-shell` obsługuje interakcję i wpisuje tekst do sprite'ów:

- `prompt-sprite-id`: sprite z linią inputu,
- `output-sprite-id`: sprite z transcriptem/outputem,
- `prompt-panel-id`: panel/okno promptu dla wrap + autosize,
- `prompt-shadow-panel-id` (opcjonalnie): synchronizacja wysokości panelu cienia.

Parametry layout/input:

- `prompt-wrap`: zawijanie linii promptu do szerokości panelu,
- `prompt-auto-grow`: dynamiczny wzrost wysokości panelu,
- `prompt-min-lines` / `prompt-max-lines`: limity liczby linii,
- `prompt-growth-ms`: czas animacji zmiany wysokości.

## 5) Rendering tekstu i przezroczystość

Aktualne reguły renderu:

- glyphy `generic:*` są rasteryzowane jako piksele z przezroczystym tłem (`bg=Reset`),
- blit tekstu zachowuje tło spod spodu dla komórek z `bg=Reset`,
- puste, transparentne komórki nie nadpisują tła.

Efekt praktyczny:

- tekst w oknach (`prompt`, `hint`) nie rysuje niechcianej czarnej „łatki” tła,
- nadal używany jest ten sam pipeline rasteryzacji fontów.

## 6) Minimalny przykład (window + terminal-shell)

```yaml
id: playground-terminal-shell
title: Playground Terminal Shell
bg: "#202020"
ui:
  enabled: true
  persist: scene
  theme: terminal
  focus-order: [ui-terminal-prompt]
layers:
  - ref: bg
  - ref: windows
input:
  terminal-shell:
    prompt_sprite_id: ui-terminal-prompt
    output_sprite_id: ui-terminal-output
    prompt_panel_id: ui-terminal-input-window
    prompt_prefix: ">"
    prompt_wrap: true
    prompt_auto_grow: true
    prompt_min_lines: 1
    prompt_max_lines: 3
    prompt_growth_ms: 140
```

Warstwa okien:

```yaml
- name: terminal-shell-windows
  z_index: 2
  ui: true
  sprites:
    - type: terminal-input
      id: ui-terminal-input-window
      at: cb
      width-percent: 95
      font: "generic:half"
      prompt-id: ui-terminal-prompt
      title-bar: ""
      border-fg: "#3A3A3A"
      border-bg: "#1C1C1C"
      panel-bg: "#5E5E5E"
      corner-radius: 1
      shadow-x: 1
      shadow-y: 1
    - type: window
      id: ui-terminal-help-tooltip
      at: rt
      x: -2
      y: 1
      font: "generic:half"
      title-bar: "HINTS"
      body-content: |
        TYPE LS
        TYPE STATUS
        TYPE LOGS
```

## 7) Troubleshooting

Jeśli sloty się nakładają:

1. Sprawdź, czy używasz `type: window` / `type: terminal-input` (a nie ręcznie ustawionych trzech text sprite'ów na tych samych `y`).
2. Sprawdź, czy wszystkie sloty mają ten sam `font` (albo jawnie różne, jeśli to intencjonalne).
3. Sprawdź, czy nie masz dodatkowych efektów `glow`/`fade` na tych samych sprite'ach, które dają wrażenie duplikacji.

Jeśli prompt wychodzi poza okno:

1. Ustaw `prompt-panel-id` na właściwy panel.
2. Włącz `prompt-wrap` i `prompt-auto-grow`.
3. Ustaw sensowne `prompt-min-lines`/`prompt-max-lines`.

Jeśli tekst ma niechciane tło:

1. Usuń jawne `bg` z text sprite'a, jeśli ma być przezroczysty.
2. Zostaw render fontu przez `generic:*` lub właściwy raster font.
