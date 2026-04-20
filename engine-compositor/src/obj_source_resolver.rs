use crate::obj_render::parse_terrain_params_from_uri;
use engine_core::scene::Sprite;

pub(crate) fn resolve_effective_obj_source(source: &str, sprite: &Sprite) -> String {
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
        world_gen_has_ocean,
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
        return source.to_string();
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
        return format!(
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
        );
    }

    if source.starts_with("world://")
        && (world_gen_seed.is_some()
            || world_gen_has_ocean.is_some()
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
        if let Some(v) = world_gen_has_ocean {
            p.planet.has_ocean = *v;
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
        return engine_worldgen::world_uri_from_params(&p);
    }

    source.to_string()
}
