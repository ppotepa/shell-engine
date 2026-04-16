use engine_core::color::Color;

#[derive(Debug, Clone)]
pub struct ObjRenderParams {
    pub scale: f32,
    pub yaw_deg: f32,
    pub pitch_deg: f32,
    pub roll_deg: f32,
    /// Static initial rotation offsets (x=pitch, y=yaw, z=roll) from `rotation-x/y/z` YAML.
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub rotation_z: f32,
    pub rotate_y_deg_per_sec: f32,
    pub camera_distance: f32,
    pub fov_degrees: f32,
    pub near_clip: f32,
    pub light_direction_x: f32,
    pub light_direction_y: f32,
    pub light_direction_z: f32,
    pub light_2_direction_x: f32,
    pub light_2_direction_y: f32,
    pub light_2_direction_z: f32,
    pub light_2_intensity: f32,
    pub light_point_x: f32,
    pub light_point_y: f32,
    pub light_point_z: f32,
    pub light_point_intensity: f32,
    pub light_point_colour: Option<Color>,
    pub light_point_flicker_depth: f32,
    pub light_point_flicker_hz: f32,
    pub light_point_orbit_hz: f32,
    pub light_point_snap_hz: f32,
    pub light_point_2_x: f32,
    pub light_point_2_y: f32,
    pub light_point_2_z: f32,
    pub light_point_2_intensity: f32,
    pub light_point_2_colour: Option<Color>,
    pub light_point_2_flicker_depth: f32,
    pub light_point_2_flicker_hz: f32,
    pub light_point_2_orbit_hz: f32,
    pub light_point_2_snap_hz: f32,
    pub cel_levels: u8,
    pub shadow_colour: Option<Color>,
    pub midtone_colour: Option<Color>,
    pub highlight_colour: Option<Color>,
    pub tone_mix: f32,
    pub scene_elapsed_ms: u64,
    /// Camera pan offset in view-space units (applied before projection).
    pub camera_pan_x: f32,
    pub camera_pan_y: f32,
    /// Additional camera look rotation (accumulated from mouse). Yaw = horizontal, pitch = vertical.
    pub camera_look_yaw: f32,
    pub camera_look_pitch: f32,
    /// Object-space/view-space translation applied after rotation and scale.
    pub object_translate_x: f32,
    pub object_translate_y: f32,
    pub object_translate_z: f32,
    /// Vertical clip region (normalised 0.0–1.0). Rows outside [min, max) are skipped.
    pub clip_y_min: f32,
    pub clip_y_max: f32,
    /// Camera world-space position for look_at view transform.
    /// Default [0,0,-camera_distance] reproduces the legacy +z-forward camera.
    pub camera_world_x: f32,
    pub camera_world_y: f32,
    pub camera_world_z: f32,
    /// View-space basis vectors (right, up, forward). Identity reproduces legacy behavior.
    pub view_right_x: f32,
    pub view_right_y: f32,
    pub view_right_z: f32,
    pub view_up_x: f32,
    pub view_up_y: f32,
    pub view_up_z: f32,
    pub view_forward_x: f32,
    pub view_forward_y: f32,
    pub view_forward_z: f32,
    /// Skip all lighting; render each face at its intrinsic `fg` color (for nebula, stars, etc.).
    pub unlit: bool,
    /// Ambient light intensity: minimum diffuse floor (prevents pitch-black dark sides).
    pub ambient: f32,
    /// Quadratic attenuation coefficient for point light 1: `1 / (1 + k * dist²)`.
    pub light_point_falloff: f32,
    /// Quadratic attenuation coefficient for point light 2.
    pub light_point_2_falloff: f32,
    /// When true, uses per-vertex Gouraud shading with smooth normals instead of flat per-face shading.
    pub smooth_shading: bool,
    /// Number of procedural latitude bands (sine-wave modulation along world-Y). 0 = disabled.
    pub latitude_bands: u8,
    /// Strength of latitude band modulation (0.0–1.0). Controls how much bands brighten/darken the surface.
    pub latitude_band_depth: f32,
    /// Optional terrain (land) color in RGB. When set, 3-D noise is used to split the surface into
    /// terrain (above `terrain_threshold`) and ocean (below). `None` disables the terrain system.
    pub terrain_color: Option<[u8; 3]>,
    /// Noise threshold for land vs. ocean classification. Typical range 0.4–0.6 (default 0.5).
    pub terrain_threshold: f32,
    /// 3-D noise frequency scale for terrain features. Higher = more/smaller continents.
    pub terrain_noise_scale: f32,
    /// Number of fBm octaves for terrain noise (1 = fast, 4 = detail-rich). Default 2.
    pub terrain_noise_octaves: u8,
    /// Strength of marble turbulence on ocean pixels. 0.0 = flat ocean color.
    pub marble_depth: f32,
    /// Elevation-based shade modulation for land pixels (0.0 = off, 0.35 = strong relief).
    /// High terrain (noise well above threshold) is brightened; low terrain (near threshold) is darkened.
    /// Gives terrain a sense of height without per-pixel normal perturbation.
    pub terrain_relief: f32,
    /// Seed offset for terrain noise. Different seeds give different continent shapes.
    pub noise_seed: f32,
    /// Domain warp strength for organic coastlines (0.0–2.0).
    pub warp_strength: f32,
    /// Octaves for domain warp field.
    pub warp_octaves: u8,
    /// FBM lacunarity (frequency multiplier per octave). Default 2.0.
    pub noise_lacunarity: f32,
    /// FBM persistence (amplitude decay per octave). Default 0.5.
    pub noise_persistence: f32,
    /// Per-pixel normal perturbation strength for fake bumps (0.0–1.0).
    pub normal_perturb_strength: f32,
    /// Ocean specular highlight strength (0.0–1.0).
    pub ocean_specular: f32,
    /// Crater density scale (0.0 = off, higher = more/smaller craters).
    pub crater_density: f32,
    /// Crater rim brightness boost.
    pub crater_rim_height: f32,
    /// Altitude (0–1 above threshold) where snow appears. 0.0 = disabled.
    pub snow_line_altitude: f32,
    /// Vertex displacement along sphere normal (fraction of sphere radius).
    /// 0.0 = flat sphere, 0.12–0.22 = visible mountains at silhouette.
    /// Applied before rotation so displaced geometry is correct from all angles.
    pub terrain_displacement: f32,
    /// When true, below-threshold pixels are left transparent (canvas `None`) instead of
    /// written with `fg_colour`. Used for cloud overlay layers.
    pub below_threshold_transparent: bool,
    /// Alpha softness width for cloud threshold edges (0.0 = binary cutoff).
    /// When > 0.0, pixels near `terrain_threshold` get a smooth alpha gradient
    /// instead of hard on/off.  Only used by the RGBA cloud render path.
    pub cloud_alpha_softness: f32,
    /// Polar ice cap color. When Some, enables smooth ice coverage at high latitudes.
    pub polar_ice_color: Option<[u8; 3]>,
    /// Latitude |y| (0=equator, 1=pole) where ice coverage begins. Default 0.78.
    pub polar_ice_start: f32,
    /// Latitude |y| where ice coverage is full. Default 0.92.
    pub polar_ice_end: f32,
    /// Desert/dry zone color for equatorial land. When None, desert effect is disabled.
    pub desert_color: Option<[u8; 3]>,
    /// Strength of desert biome blending (0.0–1.0). Default 0.0.
    pub desert_strength: f32,
    /// Atmosphere rim/glow color. When None, atmosphere rim is disabled.
    pub atmo_color: Option<[u8; 3]>,
    /// Relative atmosphere shell height (0.0–1.0 of apparent radius).
    pub atmo_height: f32,
    /// Global atmosphere optical density (0.0–1.0).
    pub atmo_density: f32,
    /// Overall atmosphere blend strength (0.0–1.0). Default 0.0.
    pub atmo_strength: f32,
    /// Rayleigh-like molecular scattering amount (0.0–1.0).
    pub atmo_rayleigh_amount: f32,
    /// Rayleigh scattering tint.
    pub atmo_rayleigh_color: Option<[u8; 3]>,
    /// Rayleigh vertical falloff control (0.0–1.0).
    pub atmo_rayleigh_falloff: f32,
    /// Mie/haze scattering amount (0.0–1.0).
    pub atmo_haze_amount: f32,
    /// Mie/haze scattering tint.
    pub atmo_haze_color: Option<[u8; 3]>,
    /// Mie/haze vertical falloff control (0.0–1.0).
    pub atmo_haze_falloff: f32,
    /// Absorption amount (0.0–1.0).
    pub atmo_absorption_amount: f32,
    /// Absorption tint.
    pub atmo_absorption_color: Option<[u8; 3]>,
    /// Absorption profile center height (0.0–1.0).
    pub atmo_absorption_height: f32,
    /// Absorption profile width (0.0–1.0).
    pub atmo_absorption_width: f32,
    /// Forward-scatter anisotropy control (0.0–1.0).
    pub atmo_forward_scatter: f32,
    /// Limb brightness multiplier.
    pub atmo_limb_boost: f32,
    /// Day/night transition softness around the terminator.
    pub atmo_terminator_softness: f32,
    /// Night-side atmospheric emission amount.
    pub atmo_night_glow: f32,
    /// Night-side atmospheric emission tint.
    pub atmo_night_glow_color: Option<[u8; 3]>,
    /// Rim falloff power for atmosphere effect (higher = thinner). Default 4.5.
    pub atmo_rim_power: f32,
    /// Broad haze contribution for atmosphere volume (0.0–1.0). Default 0.0.
    pub atmo_haze_strength: f32,
    /// Haze falloff power (lower = broader). Default 1.8.
    pub atmo_haze_power: f32,
    /// Veil strength across the visible planet disk.
    pub atmo_veil_strength: f32,
    /// Veil falloff power (lower = broader disk tint/occlusion).
    pub atmo_veil_power: f32,
    /// Strength of the outer halo rendered beyond the silhouette.
    pub atmo_halo_strength: f32,
    /// Halo width as a fraction of apparent disk radius.
    pub atmo_halo_width: f32,
    /// Halo falloff power (higher = tighter halo).
    pub atmo_halo_power: f32,
    /// Scale for ocean surface noise(higher = finer waves). Default 4.0.
    pub ocean_noise_scale: f32,
    /// Ocean base color override (RGB). When Some, replaces OBJ face color for ocean pixels.
    pub ocean_color_rgb: Option<[u8; 3]>,
    /// Night-side city lights color. When None, city lights are disabled.
    pub night_light_color: Option<[u8; 3]>,
    /// Noise threshold for city light clusters (0.0–1.0). Default 0.82.
    pub night_light_threshold: f32,
    /// Brightness of night-side city light clusters. Default 0.0.
    pub night_light_intensity: f32,
    // ── Tectonic heightmap ────────────────────────────────────────────────────
    /// Tectonic elevation grid (0..1, 0.5=sea level). Row-major, row 0 = south pole.
    pub heightmap: Option<std::sync::Arc<Vec<f32>>>,
    /// Heightmap grid width.
    pub heightmap_w: u32,
    /// Heightmap grid height.
    pub heightmap_h: u32,
    /// Blend: 0=pure fBm, 1=pure heightmap. Default 0.
    pub heightmap_blend: f32,
    /// When true, faces are sorted back-to-front (painter's algorithm) before rasterization.
    /// Only needed for semi-transparent geometry that cannot rely on the depth buffer alone.
    /// Default false — opaque objects use the depth buffer for correct occlusion, no sort needed.
    pub depth_sort_faces: bool,
}
