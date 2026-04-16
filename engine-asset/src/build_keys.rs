use engine_core::asset_source::SourceRef;
use engine_core::scene::Sprite;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MeshBuildKey(String);

impl MeshBuildKey {
    pub fn from_source(source: impl Into<String>) -> Self {
        Self(source.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for MeshBuildKey {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for MeshBuildKey {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl AsRef<str> for MeshBuildKey {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MaterialBuildKey(String);

impl MaterialBuildKey {
    pub fn from_source(source: impl Into<String>) -> Self {
        Self(source.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for MaterialBuildKey {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for MaterialBuildKey {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl AsRef<str> for MaterialBuildKey {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageAssetKey(String);

impl ImageAssetKey {
    pub fn from_asset_path(asset_path: &str) -> Self {
        let source = SourceRef::mod_asset(asset_path);
        Self(source.normalized_value().to_string())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for ImageAssetKey {
    fn from(value: String) -> Self {
        Self::from_asset_path(value.as_str())
    }
}

impl From<&str> for ImageAssetKey {
    fn from(value: &str) -> Self {
        Self::from_asset_path(value)
    }
}

impl AsRef<str> for ImageAssetKey {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Resolves a canonical image key used across 2D and 3D consumers.
pub fn resolve_image_asset_key(asset_path: &str) -> ImageAssetKey {
    ImageAssetKey::from_asset_path(asset_path)
}

/// Resolves a canonical mesh build key for an `obj` sprite source and any
/// sprite-level procedural overrides.
pub fn resolve_obj_mesh_build_key(source: &str, sprite: &Sprite) -> MeshBuildKey {
    let Sprite::Obj {
        terrain_plane_amplitude,
        terrain_plane_frequency,
        terrain_plane_roughness,
        terrain_plane_octaves,
        terrain_plane_seed_x,
        terrain_plane_seed_z,
        terrain_plane_lacunarity,
        terrain_plane_ridge,
        terrain_plane_plateau,
        terrain_plane_sea_level,
        terrain_plane_scale_x,
        terrain_plane_scale_z,
        world_gen_shape,
        world_gen_base,
        world_gen_coloring,
        world_gen_seed,
        world_gen_ocean_fraction,
        world_gen_continent_scale,
        world_gen_continent_warp,
        world_gen_continent_octaves,
        world_gen_mountain_scale,
        world_gen_mountain_strength,
        world_gen_mountain_ridge_octaves,
        world_gen_moisture_scale,
        world_gen_ice_cap_strength,
        world_gen_lapse_rate,
        world_gen_rain_shadow,
        world_gen_displacement_scale,
        world_gen_subdivisions,
        ..
    } = sprite
    else {
        return MeshBuildKey::from_source(source);
    };

    if (source.starts_with("terrain-plane://")
        || source.starts_with("terrain-sphere://")
        || source.starts_with("earth-sphere://"))
        && (terrain_plane_amplitude.is_some()
            || terrain_plane_frequency.is_some()
            || terrain_plane_roughness.is_some()
            || terrain_plane_octaves.is_some()
            || terrain_plane_seed_x.is_some()
            || terrain_plane_seed_z.is_some()
            || terrain_plane_lacunarity.is_some()
            || terrain_plane_ridge.is_some()
            || terrain_plane_plateau.is_some()
            || terrain_plane_sea_level.is_some()
            || terrain_plane_scale_x.is_some()
            || terrain_plane_scale_z.is_some())
    {
        let scheme = if source.starts_with("terrain-sphere://") {
            "terrain-sphere"
        } else if source.starts_with("earth-sphere://") {
            "earth-sphere"
        } else {
            "terrain-plane"
        };
        let mut params = parse_terrain_params_from_uri(source);
        if let Some(v) = terrain_plane_amplitude {
            params.amplitude = *v;
        }
        if let Some(v) = terrain_plane_frequency {
            params.frequency = *v;
        }
        if let Some(v) = terrain_plane_roughness {
            params.roughness = *v;
        }
        if let Some(v) = terrain_plane_octaves {
            params.octaves = *v;
        }
        if let Some(v) = terrain_plane_seed_x {
            params.seed_x = *v;
        }
        if let Some(v) = terrain_plane_seed_z {
            params.seed_z = *v;
        }
        if let Some(v) = terrain_plane_lacunarity {
            params.lacunarity = *v;
        }
        if let Some(v) = terrain_plane_ridge {
            params.ridge = *v;
        }
        if let Some(v) = terrain_plane_plateau {
            params.plateau = *v;
        }
        if let Some(v) = terrain_plane_sea_level {
            params.sea_level = *v;
        }
        if let Some(v) = terrain_plane_scale_x {
            params.scale_x = *v;
        }
        if let Some(v) = terrain_plane_scale_z {
            params.scale_z = *v;
        }
        let grid = source
            .splitn(3, "//")
            .nth(1)
            .unwrap_or("32")
            .split('?')
            .next()
            .unwrap_or("32");
        return MeshBuildKey::from_source(format!(
            "{scheme}://{}?amp={}&freq={}&oct={}&rough={}&sx={}&sz={}&lac={}&ridge={}&plat={}&sea={}&scx={}&scz={}",
            grid,
            params.amplitude,
            params.frequency,
            params.octaves,
            params.roughness,
            params.seed_x,
            params.seed_z,
            params.lacunarity,
            if params.ridge { 1 } else { 0 },
            params.plateau,
            params.sea_level,
            params.scale_x,
            params.scale_z
        ));
    }

    if source.starts_with("world://")
        && (world_gen_seed.is_some()
            || world_gen_ocean_fraction.is_some()
            || world_gen_continent_scale.is_some()
            || world_gen_continent_warp.is_some()
            || world_gen_continent_octaves.is_some()
            || world_gen_mountain_scale.is_some()
            || world_gen_mountain_strength.is_some()
            || world_gen_mountain_ridge_octaves.is_some()
            || world_gen_moisture_scale.is_some()
            || world_gen_ice_cap_strength.is_some()
            || world_gen_lapse_rate.is_some()
            || world_gen_rain_shadow.is_some()
            || world_gen_displacement_scale.is_some()
            || world_gen_subdivisions.is_some()
            || world_gen_shape.is_some()
            || world_gen_base.is_some()
            || world_gen_coloring.is_some())
    {
        let mut p = engine_worldgen::parse_world_params_from_uri(source);
        if let Some(v) = world_gen_shape {
            p.shape = engine_worldgen::parse_world_shape(v);
        }
        if let Some(v) = world_gen_base {
            p.base = engine_worldgen::parse_world_base(v);
        }
        if let Some(v) = world_gen_coloring {
            p.coloring = engine_worldgen::parse_world_coloring(v);
        }
        if let Some(v) = world_gen_subdivisions {
            p.subdivisions = *v;
        }
        if let Some(v) = world_gen_seed {
            p.planet.seed = *v;
        }
        if let Some(v) = world_gen_ocean_fraction {
            p.planet.ocean_fraction = *v;
        }
        if let Some(v) = world_gen_continent_scale {
            p.planet.continent_scale = *v;
        }
        if let Some(v) = world_gen_continent_warp {
            p.planet.continent_warp = *v;
        }
        if let Some(v) = world_gen_continent_octaves {
            p.planet.continent_octaves = *v;
        }
        if let Some(v) = world_gen_mountain_scale {
            p.planet.mountain_scale = *v;
        }
        if let Some(v) = world_gen_mountain_strength {
            p.planet.mountain_strength = *v;
        }
        if let Some(v) = world_gen_mountain_ridge_octaves {
            p.planet.mountain_ridge_octaves = *v;
        }
        if let Some(v) = world_gen_moisture_scale {
            p.planet.moisture_scale = *v;
        }
        if let Some(v) = world_gen_ice_cap_strength {
            p.planet.ice_cap_strength = *v;
        }
        if let Some(v) = world_gen_lapse_rate {
            p.planet.lapse_rate = *v;
        }
        if let Some(v) = world_gen_rain_shadow {
            p.planet.rain_shadow = *v;
        }
        if let Some(v) = world_gen_displacement_scale {
            p.displacement_scale = *v;
        }
        return MeshBuildKey::from_source(engine_worldgen::world_mesh_build_key_from_params(&p));
    }

    if source.starts_with("world://") {
        return MeshBuildKey::from_source(engine_worldgen::world_mesh_build_key_from_uri(source));
    }

    MeshBuildKey::from_source(source)
}

/// Resolves generated-world mesh source key using a stable canonical form.
pub fn resolve_generated_world_mesh_build_key(
    mesh_source: Option<&str>,
    default_source: &str,
) -> MeshBuildKey {
    let source = mesh_source.unwrap_or(default_source);
    if source.starts_with("world://") {
        MeshBuildKey::from_source(engine_worldgen::world_mesh_build_key_from_uri(source))
    } else {
        MeshBuildKey::from_source(source)
    }
}

fn parse_terrain_params(query: &str) -> engine_mesh::primitives::TerrainParams {
    let mut p = engine_mesh::primitives::TerrainParams::default();
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            match k {
                "amp" => {
                    if let Ok(f) = v.parse::<f32>() {
                        p.amplitude = f.clamp(0.01, 10.0);
                    }
                }
                "freq" => {
                    if let Ok(f) = v.parse::<f32>() {
                        p.frequency = f.clamp(0.01, 16.0);
                    }
                }
                "oct" => {
                    if let Ok(n) = v.parse::<u8>() {
                        p.octaves = n.clamp(1, 3);
                    }
                }
                "rough" => {
                    if let Ok(f) = v.parse::<f32>() {
                        p.roughness = f.clamp(0.0, 1.0);
                    }
                }
                "sx" => {
                    if let Ok(f) = v.parse::<f32>() {
                        p.seed_x = f;
                    }
                }
                "sz" => {
                    if let Ok(f) = v.parse::<f32>() {
                        p.seed_z = f;
                    }
                }
                "lac" => {
                    if let Ok(f) = v.parse::<f32>() {
                        p.lacunarity = f.clamp(1.0, 4.0);
                    }
                }
                "ridge" => {
                    p.ridge = v == "1";
                }
                "plat" => {
                    if let Ok(f) = v.parse::<f32>() {
                        p.plateau = f.clamp(0.0, 1.0);
                    }
                }
                "sea" => {
                    if let Ok(f) = v.parse::<f32>() {
                        p.sea_level = f.clamp(0.0, 1.0);
                    }
                }
                "scx" => {
                    if let Ok(f) = v.parse::<f32>() {
                        p.scale_x = f.clamp(0.25, 4.0);
                    }
                }
                "scz" => {
                    if let Ok(f) = v.parse::<f32>() {
                        p.scale_z = f.clamp(0.25, 4.0);
                    }
                }
                _ => {}
            }
        }
    }
    p
}

fn parse_terrain_params_from_uri(uri: &str) -> engine_mesh::primitives::TerrainParams {
    let query = uri.split_once('?').map(|(_, q)| q).unwrap_or("");
    parse_terrain_params(query)
}

#[cfg(test)]
mod tests {
    use super::{
        resolve_generated_world_mesh_build_key, resolve_image_asset_key,
        resolve_obj_mesh_build_key, MeshBuildKey,
    };
    use engine_core::scene::Sprite;

    #[test]
    fn canonicalizes_world_mesh_key_from_source() {
        let source = "world://48?seed=9&shape=sphere&base=cube&coloring=biome";
        let key = resolve_generated_world_mesh_build_key(Some(source), "cube-sphere://64");
        assert_eq!(
            key.as_str(),
            engine_worldgen::world_mesh_build_key_from_uri(source)
        );
    }

    #[test]
    fn resolves_obj_world_key_with_sprite_overrides() {
        let sprite: Sprite = serde_yaml::from_str(
            r#"
type: obj
source: world://32?seed=1
world-seed: 42
"#,
        )
        .expect("obj sprite should parse");

        let key = resolve_obj_mesh_build_key("world://32?seed=1", &sprite);
        assert!(key.as_str().contains("seed=42"));
    }

    #[test]
    fn resolves_obj_terrain_key_with_sprite_overrides() {
        let sprite: Sprite = serde_yaml::from_str(
            r#"
type: obj
source: terrain-plane://32?amp=1.0
terrain-amplitude: 2.5
"#,
        )
        .expect("obj sprite should parse");

        let key = resolve_obj_mesh_build_key("terrain-plane://32?amp=1.0", &sprite);
        assert!(key.as_str().starts_with("terrain-plane://32?"));
        assert!(key.as_str().contains("amp=2.5"));
    }

    #[test]
    fn non_obj_sprite_keeps_source_key() {
        let sprite: Sprite = serde_yaml::from_str(
            r#"
type: text
content: hi
"#,
        )
        .expect("text sprite should parse");
        let key = resolve_obj_mesh_build_key("/assets/3d/a.obj", &sprite);
        assert_eq!(key, MeshBuildKey::from_source("/assets/3d/a.obj"));
    }

    #[test]
    fn image_asset_key_normalizes_equivalent_paths() {
        let with_leading = resolve_image_asset_key("/assets/images/logo.png");
        let without_leading = resolve_image_asset_key("assets/images/logo.png");
        assert_eq!(with_leading, without_leading);
        assert_eq!(with_leading.as_str(), "assets/images/logo.png");
    }
}
