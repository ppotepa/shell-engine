# Input Profiles (Current Behavior)

Ten dokument opisuje aktualne profile wejścia sceny oraz ich kontrakt runtime.

## 1) Model eventu klawiatury

Silnik używa pełnego `KeyEvent` (nie tylko `KeyCode`), więc input uwzględnia:

- kod klawisza,
- modyfikatory (`Ctrl`, `Alt`, `Shift`),
- typ zdarzenia (`Press/Repeat/Release`).

## 2) Profile `scene.input`

Obsługiwane profile:

- `obj-viewer`
- `terminal-size-tester`
- `terminal-shell`

Przykład:

```yaml
input:
  terminal-shell:
    prompt-sprite-id: terminal-prompt
    output-sprite-id: terminal-output
    prompt-prefix: "λ "
    max-lines: 120
```

## 3) `terminal-shell`

`terminal-shell` zapisuje dane do zwykłych text sprite'ów:

- `prompt-sprite-id` wskazuje linię inputu,
- `output-sprite-id` wskazuje panel wyjścia.

Edytor linii działa na `tui-input` (core API), a mapowanie klawiszy jest realizowane po stronie silnika.

Skróty obejmują m.in.:

- ruch kursora,
- kasowanie znaków/słów,
- `Ctrl-A/E/K/U/W`.

Wbudowane komendy shella:

- `help`, `clear`, `ls`, `pwd`, `echo`, `whoami`.

## 4) `obj-viewer`

Profil steruje sprite typu `obj` przez:

- `sprite_id` (target).

## 5) `terminal-size-tester`

Profil przyjmuje listę presetów:

- `presets: ["80x24", "100x30", ...]`.

## 6) Runtime contract

Input jest konsumowany w `scene_lifecycle` i przekazywany do aktywnych profilów sceny.
Zmiany stanu wejścia synchronizują się z runtime scene state i trafiają do renderu w tej samej klatce.

## 7) Debug feature mode

Generyczne helpery debug można włączyć przez:

- CLI: `cargo run -p app -- --debug-feature`
- env: `SHELL_QUEST_DEBUG_FEATURE=1`

W trybie debug:

- `F1` przełącza overlay debug (scene id + virtual buffer info),
- `F3` przełącza na poprzednią scenę (kolejność discover),
- `F4` przełącza na następną scenę (kolejność discover).

## 8) Playground routing policy

W scenach `playground-*`:

- `Esc` wraca do `playground-menu` (zamiast zamykać aplikację),
- wyjście z aplikacji odbywa się przez pozycję `Exit` w `playground-menu`.

Globalny hard quit pozostaje pod `Ctrl+C`.

## 9) Referencyjne sceny

- `mods/playground/scenes/terminal-shell/scene.yml`
- `mods/playground/scenes/3d-scene/scene.yml`
- `mods/playground/scenes/terminal-size-test/scene.yml`

## 10) Playground terminal-shell (HUD showcase)

Scena `playground-terminal-shell` jest referencyjnym układem HUD/interfejsu:

- tło (`bg`) renderuje strumień outputu terminala (`output-sprite-id`),
- warstwa `windows` renderuje osobne okno inputu użytkownika (`prompt-sprite-id`) przez `type: terminal-input` oraz niezależny tooltip komend przez `type: window`,
- slot `title` dla `type: window` renderuje się w nagłówku (top-center),
- sloty `title/body/footer` są układane sekwencyjnie wg wysokości fontu (bez nakładania),
- prompt używa `prompt-panel-id` + `prompt-wrap` + `prompt-auto-grow`, więc tekst wejściowy pozostaje w obrębie okna i panel rośnie płynnie do wielu linii,
- opcjonalne `prompt-shadow-panel-id` synchronizuje wysokość panelu cienia z głównym input-window (gdy scena faktycznie używa osobnego panelu cienia).
- renderer tekstu zachowuje przezroczystość (`bg=Reset`) i nie nadpisuje tła czarną łatą.

Scena używa `font: "generic:half"` i `ui.theme: terminal`; gdy motyw nie jest zdefiniowany, runtime używa fallbacku `engine-default`.

Szczegółowy kontrakt YAML i renderingu:

- `docs/terminal-hud-authoring.md`
