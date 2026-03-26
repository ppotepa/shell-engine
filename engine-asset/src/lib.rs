//! Asset loading and repository abstractions.
//!
//! Provides scene and asset repository implementations for loading from
//! mod sources (unpacked directories or ZIP archives), and scene compilation.

pub mod repositories;
pub mod scene_compiler;

pub use repositories::{
    create_asset_repository, create_scene_repository, AnyAssetRepository, AnySceneRepository,
    AssetRepository, FsSceneRepository, SceneRepository, ZipSceneRepository,
};
pub use scene_compiler::compile_scene_document_with_loader_and_source;

pub mod source_loader;

pub use source_loader::ModAssetSourceLoader;
