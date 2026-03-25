# engine-render-terminal

Crossterm-based terminal RenderBackend implementation.

## Purpose

Implements the `RenderBackend` trait using crossterm to write composed
frames to the terminal. This is the default output backend used when
running Shell Quest in a terminal emulator.

## Key Types

- `TerminalRenderBackend` — struct implementing `RenderBackend` via crossterm

## Dependencies

- `engine-core` — buffer and cell types
- `engine-render` — `RenderBackend` trait definition
- `crossterm` — terminal output, cursor control, and style commands

## Usage

Created automatically by the runtime when no alternative backend is
specified. Handles raw mode setup, alternate screen, and cleanup on
shutdown.
