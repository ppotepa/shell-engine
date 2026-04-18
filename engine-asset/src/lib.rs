//! Asset loading and repository abstractions.
//!
//! Provides scene and asset repository implementations for loading from
//! mod sources (unpacked directories or ZIP archives), and scene compilation.
//! Image entry points (`load_image_asset`, `load_rgba_image`) and key-based
//! variants (`*_with_key`, `ImageAssetKey`) are the shared seam used by both
//! 2D sprite rendering and 3D-domain consumers that need decoded image data.

pub mod build_keys;
pub mod image_assets;
pub mod material_repository;
pub mod mesh_assets;
pub mod mesh_repository;
pub mod profile_assets;
pub mod repositories;
pub mod scene_compiler;

pub use repositories::{
    create_asset_repository, create_scene_repository, AnyAssetRepository, AnySceneRepository,
    AssetRepository, FsSceneRepository, SceneRepository, ZipSceneRepository,
};
pub use scene_compiler::compile_scene_document_with_loader_and_source;
pub use {
    build_keys::{
        resolve_generated_world_mesh_build_key, resolve_image_asset_key,
        resolve_obj_mesh_build_key, ImageAssetKey, MaterialBuildKey, MeshBuildKey,
    },
    image_assets::{
        has_image_asset, has_image_asset_with_key, load_image_asset, load_image_asset_with_key,
        load_rgba_image, load_rgba_image_with_key, AnimatedImageAsset, AnimatedImageAssetFrame,
        ImageAsset, RgbaImageAsset,
    },
    material_repository::MaterialRepository,
    mesh_assets::{
        colored_mesh_to_obj_mesh, load_obj_mesh, load_obj_mesh_from_root, load_render_mesh,
        mesh_to_obj_mesh, ObjFace, ObjMesh,
    },
    mesh_repository::MeshRepository,
    profile_assets::hydrate_scene_view_profiles,
};

pub mod source_loader;

pub use source_loader::ModAssetSourceLoader;
