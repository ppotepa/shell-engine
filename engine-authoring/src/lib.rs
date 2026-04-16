//! Shared authoring pipeline for Shell Engine YAML assets.
//!
//! This crate is the future source of truth for authored scene/object input:
//! parsing, package assembly, normalization, validation, and schema metadata.
//! Runtime execution and rendering stay in `engine`; shared runtime data stays
//! in `engine-core`.

pub mod compile;
pub mod document;
pub mod package;
pub mod repository;
pub mod schema;
pub mod validate;

/// Common result type used by authoring pipeline APIs.
pub type AuthoringResult<T> = anyhow::Result<T>;
