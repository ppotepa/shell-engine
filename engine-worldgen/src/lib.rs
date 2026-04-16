use engine_terrain::{
    moisture_color, GeneratedPlanet, HeightmapCell, WorldBase, WorldColoring, WorldGenParams,
    WorldShape,
};

#[derive(Debug, Clone)]
pub struct GeneratedWorldMesh {
    pub mesh: engine_mesh::Mesh,
    pub face_colors: Vec<[u8; 3]>,
}

/// Parse a `world://N?...` URI into `WorldGenParams`.
pub fn parse_world_params_from_uri(uri: &str) -> WorldGenParams {
    let rest = uri.strip_prefix("world://").unwrap_or(uri);
    let (subdiv_str, query) = rest.split_once('?').unwrap_or((rest, ""));
    let subdivisions: u32 = subdiv_str.trim().parse().unwrap_or(32);
    let mut p = WorldGenParams::default();
    p.subdivisions = subdivisions;
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            match k {
                "shape" => {
                    p.shape = parse_world_shape(v);
                }
                "base" => {
                    p.base = parse_world_base(v);
                }
                "coloring" => {
                    p.coloring = parse_world_coloring(v);
                }
                "seed" => {
                    if let Ok(n) = v.parse::<u64>() {
                        p.planet.seed = n;
                    }
                }
                "ocean" => {
                    if let Ok(f) = v.parse::<f64>() {
                        p.planet.ocean_fraction = f.clamp(0.0, 1.0);
                    }
                }
                "cscale" => {
                    if let Ok(f) = v.parse::<f64>() {
                        p.planet.continent_scale = f.clamp(0.5, 10.0);
                    }
                }
                "cwarp" => {
                    if let Ok(f) = v.parse::<f64>() {
                        p.planet.continent_warp = f.clamp(0.0, 2.0);
                    }
                }
                "coct" => {
                    if let Ok(n) = v.parse::<u8>() {
                        p.planet.continent_octaves = n.clamp(2, 8);
                    }
                }
                "mscale" => {
                    if let Ok(f) = v.parse::<f64>() {
                        p.planet.mountain_scale = f.clamp(1.0, 20.0);
                    }
                }
                "mstr" => {
                    if let Ok(f) = v.parse::<f64>() {
                        p.planet.mountain_strength = f.clamp(0.0, 1.0);
                    }
                }
                "mroct" => {
                    if let Ok(n) = v.parse::<u8>() {
                        p.planet.mountain_ridge_octaves = n.clamp(2, 8);
                    }
                }
                "moistscale" => {
                    if let Ok(f) = v.parse::<f64>() {
                        p.planet.moisture_scale = f.clamp(0.5, 10.0);
                    }
                }
                "ice" => {
                    if let Ok(f) = v.parse::<f64>() {
                        p.planet.ice_cap_strength = f.clamp(0.0, 3.0);
                    }
                }
                "lapse" => {
                    if let Ok(f) = v.parse::<f64>() {
                        p.planet.lapse_rate = f.clamp(0.0, 1.0);
                    }
                }
                "rainshadow" => {
                    if let Ok(f) = v.parse::<f64>() {
                        p.planet.rain_shadow = f.clamp(0.0, 1.0);
                    }
                }
                "disp" => {
                    if let Ok(f) = v.parse::<f32>() {
                        p.displacement_scale = f.clamp(0.0, 1.0);
                    }
                }
                _ => {}
            }
        }
    }
    p
}

pub fn parse_world_shape(s: &str) -> WorldShape {
    match s {
        "flat" => WorldShape::Flat,
        _ => WorldShape::Sphere,
    }
}

pub fn parse_world_base(s: &str) -> WorldBase {
    match s {
        "uv" => WorldBase::Uv,
        "tetra" => WorldBase::Tetra,
        "octa" => WorldBase::Octa,
        "icosa" => WorldBase::Icosa,
        _ => WorldBase::Cube,
    }
}

pub fn parse_world_coloring(s: &str) -> WorldColoring {
    match s {
        "altitude" | "elevation" => WorldColoring::Altitude,
        "moisture" => WorldColoring::Moisture,
        "none" => WorldColoring::None,
        _ => WorldColoring::Biome,
    }
}

pub fn world_shape_str(shape: WorldShape) -> &'static str {
    match shape {
        WorldShape::Flat => "flat",
        WorldShape::Sphere => "sphere",
    }
}

pub fn world_base_str(base: WorldBase) -> &'static str {
    match base {
        WorldBase::Cube => "cube",
        WorldBase::Uv => "uv",
        WorldBase::Tetra => "tetra",
        WorldBase::Octa => "octa",
        WorldBase::Icosa => "icosa",
    }
}

pub fn world_coloring_str(coloring: WorldColoring) -> &'static str {
    match coloring {
        WorldColoring::Altitude => "altitude",
        WorldColoring::Biome => "biome",
        WorldColoring::Moisture => "moisture",
        WorldColoring::None => "none",
    }
}

/// Canonical URI serialization for cache keys and runtime updates.
pub fn world_uri_from_params(p: &WorldGenParams) -> String {
    format!(
        "world://{}?shape={}&base={}&coloring={}&seed={}&ocean={}&cscale={}&cwarp={}&coct={}&mscale={}&mstr={}&mroct={}&moistscale={}&ice={}&lapse={}&rainshadow={}&disp={}",
        p.subdivisions,
        world_shape_str(p.shape),
        world_base_str(p.base),
        world_coloring_str(p.coloring),
        p.planet.seed,
        p.planet.ocean_fraction,
        p.planet.continent_scale,
        p.planet.continent_warp,
        p.planet.continent_octaves,
        p.planet.mountain_scale,
        p.planet.mountain_strength,
        p.planet.mountain_ridge_octaves,
        p.planet.moisture_scale,
        p.planet.ice_cap_strength,
        p.planet.lapse_rate,
        p.planet.rain_shadow,
        p.displacement_scale
    )
}

/// Stable mesh build key for world-generated geometry.
#[inline]
pub fn world_mesh_build_key_from_params(p: &WorldGenParams) -> String {
    world_uri_from_params(p)
}

/// Stable mesh build key normalized from a `world://` URI.
#[inline]
pub fn world_mesh_build_key_from_uri(uri: &str) -> String {
    let params = parse_world_params_from_uri(uri);
    world_uri_from_params(&params)
}

/// Generate world mesh + face colors from high-level params.
pub fn build_world_mesh(p: &WorldGenParams) -> GeneratedWorldMesh {
    let planet = engine_terrain::generate(&p.planet);
    match p.shape {
        WorldShape::Sphere => {
            let base = build_world_base_mesh(p.base, p.subdivisions);
            let verts: Vec<[f32; 3]> = base
                .vertices
                .iter()
                .map(|v| {
                    let cell = sample_planet_xyz(v, &planet);
                    let disp = (cell.elevation - 0.5) * 2.0 * p.displacement_scale;
                    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-6);
                    let nx = v[0] / len;
                    let ny = v[1] / len;
                    let nz = v[2] / len;
                    let r = 1.0 + disp;
                    [nx * r, ny * r, nz * r]
                })
                .collect();

            let normals = engine_mesh::mesh::compute_smooth_normals(&verts, &base.faces);
            let colors: Vec<[u8; 3]> = base
                .faces
                .iter()
                .map(|&[a, b, c]| {
                    let cx = (verts[a][0] + verts[b][0] + verts[c][0]) / 3.0;
                    let cy = (verts[a][1] + verts[b][1] + verts[c][1]) / 3.0;
                    let cz = (verts[a][2] + verts[b][2] + verts[c][2]) / 3.0;
                    let cell = sample_planet_xyz(&[cx, cy, cz], &planet);
                    match p.coloring {
                        WorldColoring::Biome => engine_terrain::biome_color(cell.biome),
                        WorldColoring::Altitude => engine_terrain::altitude_color(cell.elevation),
                        WorldColoring::Moisture => moisture_color(cell.moisture),
                        WorldColoring::None => [200, 200, 200],
                    }
                })
                .collect();
            let mesh = engine_mesh::Mesh::new(verts, normals, base.faces);
            GeneratedWorldMesh {
                mesh,
                face_colors: colors,
            }
        }
        WorldShape::Flat => {
            // Build a flat terrain grid driven by the planet heightmap.
            // UV maps XZ ∈ [-1,1] linearly onto the heightmap so every planet
            // parameter (ocean fraction, continents, climate) affects the flat view
            // and all three coloring modes (biome/altitude/moisture) work correctly.
            let subdiv = p.subdivisions.clamp(8, 256) as usize;
            let cols = subdiv;
            let rows = subdiv;

            let mut vertices = Vec::with_capacity((rows + 1) * (cols + 1));
            for row in 0..=rows {
                for col in 0..=cols {
                    let u = col as f32 / cols as f32;
                    let v = row as f32 / rows as f32;
                    let x = u * 2.0 - 1.0;
                    let z = v * 2.0 - 1.0;
                    let gx =
                        ((u * (planet.width - 1) as f32).round() as usize).min(planet.width - 1);
                    let gy =
                        ((v * (planet.height - 1) as f32).round() as usize).min(planet.height - 1);
                    let cell = planet.cell(gx, gy);
                    let y = (cell.elevation - 0.5) * 2.0 * p.displacement_scale;
                    vertices.push([x, y, z]);
                }
            }

            let mut faces = Vec::with_capacity(rows * cols * 2);
            for row in 0..rows {
                for col in 0..cols {
                    let i00 = row * (cols + 1) + col;
                    let i10 = i00 + 1;
                    let i01 = (row + 1) * (cols + 1) + col;
                    let i11 = i01 + 1;
                    faces.push([i00, i01, i10]);
                    faces.push([i10, i01, i11]);
                }
            }

            let normals = engine_mesh::mesh::compute_smooth_normals(&vertices, &faces);

            let colors: Vec<[u8; 3]> = faces
                .iter()
                .map(|&[a, b, c]| {
                    let u_avg =
                        (vertices[a][0] + vertices[b][0] + vertices[c][0]) / 3.0 * 0.5 + 0.5;
                    let v_avg =
                        (vertices[a][2] + vertices[b][2] + vertices[c][2]) / 3.0 * 0.5 + 0.5;
                    let gx = ((u_avg * (planet.width - 1) as f32).round() as usize)
                        .min(planet.width - 1);
                    let gy = ((v_avg * (planet.height - 1) as f32).round() as usize)
                        .min(planet.height - 1);
                    let cell = planet.cell(gx, gy);
                    match p.coloring {
                        WorldColoring::Biome => engine_terrain::biome_color(cell.biome),
                        WorldColoring::Altitude => engine_terrain::altitude_color(cell.elevation),
                        WorldColoring::Moisture => moisture_color(cell.moisture),
                        WorldColoring::None => [200, 200, 200],
                    }
                })
                .collect();

            let mesh = engine_mesh::Mesh::new(vertices, normals, faces);
            GeneratedWorldMesh {
                mesh,
                face_colors: colors,
            }
        }
    }
}

fn build_world_base_mesh(base: WorldBase, subdivisions: u32) -> engine_mesh::Mesh {
    use engine_mesh::primitives::{
        cube_sphere, icosa_sphere, octa_sphere, tetra_sphere, uv_sphere,
    };
    match base {
        // Cube and UV sphere face counts scale as O(N²) — cap at 256 subdivisions to avoid
        // generating millions of sub-pixel triangles that cost CPU time but add no visible detail.
        // Face counts: cube_sphere(128)≈196K, cube_sphere(256)≈786K, cube_sphere(512)≈3.1M.
        // The terrain heightmap (512×256 grid) provides the quality; mesh res just samples it.
        WorldBase::Cube => cube_sphere(subdivisions.min(256)),
        WorldBase::Uv => {
            let lat = subdivisions.clamp(8, 128);
            let lon = (lat * 2).clamp(16, 256);
            uv_sphere(lat, lon)
        }
        // Poly spheres scale as O(4^levels) — already naturally bounded by poly_levels_from_subdivisions.
        WorldBase::Tetra => tetra_sphere(poly_levels_from_subdivisions(subdivisions)),
        WorldBase::Octa => octa_sphere(poly_levels_from_subdivisions(subdivisions)),
        WorldBase::Icosa => icosa_sphere(poly_levels_from_subdivisions(subdivisions)),
    }
}

fn poly_levels_from_subdivisions(subdivisions: u32) -> u32 {
    // Level 0/1 produce too few faces for usable planet topology (icosa level 0 = 20 triangles).
    // Start at level 2 (320 faces for icosa) to ensure a reasonable minimum.
    match subdivisions {
        0..=32 => 2,
        33..=64 => 3,
        65..=128 => 4,
        _ => 5,
    }
}

fn sample_planet_xyz(v: &[f32; 3], planet: &GeneratedPlanet) -> HeightmapCell {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-6);
    let nx = v[0] / len;
    let ny = v[1] / len;
    let nz = v[2] / len;

    let lat = ny.clamp(-1.0, 1.0).acos();
    let lon = nz.atan2(nx);
    let lon_pos = if lon < 0.0 {
        lon + std::f32::consts::TAU
    } else {
        lon
    };

    let gx = ((lon_pos / std::f32::consts::TAU) * planet.width as f32) as usize;
    let gy = ((lat / std::f32::consts::PI) * planet.height as f32) as usize;
    let gx = gx.min(planet.width - 1);
    let gy = gy.min(planet.height - 1);
    *planet.cell(gx, gy)
}
