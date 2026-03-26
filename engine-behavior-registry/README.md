# engine-behavior-registry

Compatibility crate re-exporting behavior registry types from `engine-behavior`.

## Purpose

This crate exists only to preserve older dependency paths after the behavior
registry implementation was absorbed into `engine-behavior`.

New code should depend on `engine-behavior` directly. This crate remains as a
thin re-export layer so the workspace can evolve without breaking every caller
at once.

## What lives here

- `pub use engine_behavior::registry::*;`

That is the entire public contract.

## Working with this crate

- do not add new logic here unless temporary compatibility absolutely requires it,
- prefer moving callers to `engine-behavior` instead of expanding this crate,
- if the compatibility layer is ever removed, update dependent READMEs and
  Cargo manifests together.
