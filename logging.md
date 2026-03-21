# Logging (Current State)

Dokument opisuje aktualnie zaimplementowany runtime logging dla `app` i `editor`.

## 1) Lokalizacja logów

Domyślny layout katalogów:

`logs/<dd-mm-yy>/run-XXX/run.log`

Przykład:

`logs/20-03-26/run-001/run.log`

Każde uruchomienie tworzy nowy katalog `run-XXX` (inkrementowany per dzień).

## 2) Kiedy logowanie jest aktywne

Aktywacja logowania (priorytet):

1. `--no-logs` → wyłącza logi.
2. `--logs` → włącza logi.
3. build debug (`cfg!(debug_assertions)`) → logi włączone domyślnie.
4. release: `SHELL_QUEST_LOGS=1|true|yes|on` → włącza logi.

Root katalogu logów można nadpisać flagą:

- `--log-root <path>`

## 3) Format wpisu

Każda linia ma format:

`timestamp [pid] [app] [level] [target] message`

`app` to nazwa procesu (`app` albo `editor`).

## 4) Abstrakcja loggera

Wspólna implementacja jest w:

- `engine-core/src/logging.rs`

API:

- `init_run_logger(...)`
- `resolve_enabled(...)`
- `install_panic_hook(...)`
- `debug/info/warn/error(...)`
- `run_log_info()`
- `tail_recent(limit: usize) -> Vec<LogOverlayLine>` ← **nowe**

Logger jest globalny per proces (`OnceLock`) i thread-safe (`Mutex`).

## 5) In-memory ring buffer (overlay)

Każde wywołanie `logging::*` oprócz zapisu do pliku dołącza wpis do globalnego ring buffera (cap 500 wpisów). Najstarsze wpisy są usuwane automatycznie.

```rust
pub struct LogOverlayLine {
    pub level: &'static str,   // "DEBUG" | "INFO" | "WARN" | "ERROR"
    pub target: String,
    pub message: String,
}
```

`tail_recent(n)` zwraca ostatnie N wpisów — używane przez debug overlay.

## 6) Debug overlay — tylda (`~`)

Uruchomienie z `--debug-feature` aktywuje overlay:

```bash
SHELL_QUEST_MOD_SOURCE=mods/shell-quest cargo run -p app -- --debug-feature
```

Klawisze:
- **F1** — toggle overlay Stats (scene id, virtual size, ostatnie błędy Rhai)
- **~** / **`** — toggle overlay Logs (ostatnie N wpisów loggera)
- **F3 / F4** — prev/next scena (debug navigation)

Overlay Logs renderuje linie z kolorowaniem:
- `WARN` → żółty tekst
- `ERROR` → czerwony tekst
- pozostałe → biały tekst

## 7) Aktualne punkty emisji

`app`:
- start launchera i resolved config,
- init/failed init `ShellEngine`,
- wynik `engine.run()`.

`engine`:
- start runa z entrypointem,
- warningi startup checks,
- załadowanie sceny wejściowej,
- transition request/apply,
- błędy backendu audio,
- **błędy kompilacji/runtime Rhai** (`engine.debug.overlay`),
- **toggle overlay** (`engine.debug.input`).

`editor`:
- start pętli aplikacji,
- dispatch komend i zmiany mode/sidebar/focus,
- open/close projektu,
- start/stop Scene Run,
- przejścia scen podczas hard run.

## 8) DebugLogBuffer (Rhai errors)

Niezależnie od file/ring loggera, `DebugLogBuffer` (cap 64) przechowuje błędy Rhai:

- błędy kompilacji skryptu (preflight przy ładowaniu sceny)
- błędy runtime eval

Wyświetlane w overlay Stats na czerwonym tle. Gdy aktywny błąd + debug mode → renderer zachowuje ostatni dobry frame zamiast czarnego ekranu.
