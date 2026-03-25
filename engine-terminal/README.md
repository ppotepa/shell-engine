# engine-terminal

Terminal detection and configuration.

## Purpose

Detects the host terminal's capabilities (color depth, Unicode
support, dimensions) and builds a configuration struct the engine
uses to adapt its rendering output accordingly.

## Key Types

- `TerminalConfig` — resolved terminal settings (size, color mode, features)
- `TerminalCapabilities` — detected feature flags (truecolor, halfblock, etc.)
- `detect_terminal()` — probes the environment and returns `TerminalConfig`

## Dependencies

- `crossterm` — terminal size queries and capability detection
- `serde_yaml` / `serde` — optional config file deserialization

## Usage

Called once at startup before the render backend is initialized:

```rust
let config = detect_terminal()?;
```
