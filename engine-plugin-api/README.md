# engine-plugin-api

Shared extension and plugin contracts for the Shell Engine workspace.

This crate is intentionally data-first. It defines:

- plugin taxonomy (`content`, `script`, `native engine`)
- extension-point descriptors
- capability sets
- typed asset handles
- service/interface descriptors

It does not implement dynamic loading or a stable native plugin ABI. Those are
later concerns once the contracts stop moving.
