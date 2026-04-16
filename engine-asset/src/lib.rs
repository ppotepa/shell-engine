//! Asset loading and repository abstractions.
//!
//! Provides scene and asset repository implementations for loading from
//! mod sources (unpacked directories or ZIP archives), and scene compilation.

pub mod build_keys;
pub mod material_repository;
pub mod mesh_repository;
pub mod repositories;
pub mod scene_compiler;

pub use repositories::{
    create_asset_repository, create_scene_repository, AnyAssetRepository, AnySceneRepository,
    AssetRepository, FsSceneRepository, SceneRepository, ZipSceneRepository,
};
pub use scene_compiler::compile_scene_document_with_loader_and_source;
pub use {
    build_keys::{
        resolve_generated_world_mesh_build_key, resolve_obj_mesh_build_key, MaterialBuildKey,
        MeshBuildKey,
    },
    material_repository::MaterialRepository,
    mesh_repository::MeshRepository,
};

pub mod source_loader;

pub use source_loader::ModAssetSourceLoader;
