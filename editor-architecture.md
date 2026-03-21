# Editor Architecture (Current State)

Ten dokument opisuje aktualny stan architektury `editor/` po refaktorze.

## 1) Warstwy

- `editor/src/state` trzyma stan aplikacji i routing komend.
- `editor/src/ui` renderuje TUI i konsumuje stan.
- `editor/src/domain` trzyma model indeksu, preview i mapowanie parametrów efektów.
- `editor/src/io` skanuje projekt, ładuje YAML i waliduje layout projektu.

Aktualny podział feature-state w `editor/src/state`:

- `start_screen.rs`: launch flow (recents/actions), schema picker, directory browser, open/close projektu.
- `project_explorer.rs`: drzewo projektu i wejście do edit mode.
- `editor_pane.rs`: lifecycle edit mode i routing komend panelu edytora.
- `effects_browser.rs`: selection/params/live preview lifecycle dla efektów.
- `scenes_browser.rs`: selection/layers/fullscreen/preview lifecycle dla scen.
- `cutscene.rs`: walidacja źródeł cutsceny i status feature.
- `watch.rs`: polling zmian FS i synchronizacja indeksu/preview.

## 2) Preview pipeline

Preview scen i efektów korzysta ze wspólnego modułu:

- `editor/src/domain/preview_renderer.rs`

UI nie składa już runtime preview bezpośrednio w komponentach; komponenty
wołają wspólny renderer i dostają gotowy `Buffer` do wyświetlenia.

Aktualny przepływ:

1. UI buduje request preview (scena/efekt + viewport/progress).
2. `domain/preview_renderer` uruchamia wspólny runtime preview.
3. Renderer zwraca `Buffer`.
4. UI renderuje `Buffer` jako linie TUI.

Uwaga o zgodności z runtime:

- Ponieważ preview używa runtime render path, scena z `postfx` renderuje się w preview po tych samych etapach (`compositor -> postfx -> renderer buffer`).
- Oznacza to, że authoring `postfx` jest widoczny w podglądzie scen bez osobnej ścieżki UI-only.

## 3) Efekty: source of truth parametrów

Parametry efektów są obsługiwane przez descriptor registry w:

- `editor/src/domain/effect_params.rs`

Registry centralizuje:

- odczyt wartości z `EffectParams`,
- zapis override do `EffectParams`,
- etykiety parametrów dla UI.

## 4) Walidacja i indeksacja projektu

Walidacja katalogu projektu i indeksacja używają typed manifestu (`ModManifestSummary`)
oraz wspólnego przepływu walidacji (`validate_project_dir_with_manifest`), zamiast
dwóch niezależnych ścieżek parsowania.

## 5) Authoring contract (logic + presets)

W `engine-authoring`:

- logika sceny wymaga jawnego bloku `logic:`,
- `logic.kind: script` wymaga jawnego `src`,
- brak auto-detekcji sidecarów bez `logic:`,
- `logic.kind: graph` jest odrzucane jako eksperymentalne,
- wspierane są scene-level `effect-presets` + `use/preset/ref` z `overrides` (deep merge),
- aliasy presetów są walidowane deterministycznie (bez konfliktów aliasów root/entry).

## 6) Obszary centralne (stan bieżący)

`editor/src/state/mod.rs` pełni rolę shell-a:

- definicje typów stanu,
- globalne helpery UI (`current_screen_name`, `current_shortcuts`, `current_help`),
- root routing (`apply_command` + mode dispatch),
- wspólne metody przekrojowe (`update_transition`, time helpers).

## 7) Logging runtime/editor

Aktualny logging run-level jest opisany w osobnym pliku:

- `logging.md`

W skrócie:

- logger jest współdzielony (`engine-core/src/logging.rs`),
- `app` i `editor` inicjalizują run log przy starcie,
- w debug build logi są domyślnie włączone.
