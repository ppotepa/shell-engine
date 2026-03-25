# engine-error

Shared error types for the engine crate family.

## Purpose

Defines a unified error enum used across engine crates so that I/O,
YAML parsing, zip extraction, configuration, and asset-not-found
errors have a single, consistent representation.

## Key Types

- `EngineError` — enum with variants: `IoError`, `YamlError`, `ZipError`, `ConfigError`, `AssetNotFound`, and others

## Dependencies

- `thiserror` — derive macro for `std::error::Error` implementation
- `zip` — error type for zip archive operations
- `serde_yaml` — error type for YAML parse failures

## Usage

Engine crates return `Result<T, EngineError>` from fallible operations.
Use the `?` operator to propagate errors across crate boundaries.
