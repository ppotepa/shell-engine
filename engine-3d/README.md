# engine-3d

OBJ mesh loading, Scene3D YAML definitions, and 3D asset resolution.

## Purpose

Provides a pipeline for loading 3D assets (OBJ meshes and MTL materials)
and resolving them against Scene3D definitions authored in YAML. This crate
bridges raw 3D geometry with the engine's scene model.

## Key Types

- `Scene3D` — YAML-defined 3D scene model with camera, lights, and mesh references
- `Scene3DAssetResolver` — trait for locating and loading 3D assets from mod sources
- `ObjLoader` / `MtlLoader` — parsers for Wavefront OBJ and MTL file formats

## Dependencies

- `engine-core` — scene model and shared types
- `serde` / `serde_yaml` — YAML deserialization of Scene3D definitions

## Usage

Scene3D definitions live in mod YAML files. The asset resolver locates
referenced OBJ/MTL files from the mod's asset directory at load time.
