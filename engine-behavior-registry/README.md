# engine-behavior-registry

Behavior definition loading and lookup from YAML.

## Purpose

Loads named behavior definitions from YAML files and provides a
registry for the runtime to resolve behavior references at scene
compile time.

## Key Types

- `BehaviorRegistry` — collection of named behavior definitions, keyed by ID
- `BehaviorDefinition` — a single behavior's YAML-defined parameters and script reference

## Dependencies

- `engine-core` — shared behavior model types
- `serde_yaml` — YAML deserialization of behavior definition files

## Usage

The scene compiler queries the `BehaviorRegistry` to resolve behavior
references found in scene YAML. Definitions are loaded once at mod
initialization.
