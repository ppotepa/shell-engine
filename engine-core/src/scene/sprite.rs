use super::{color::TermColour, BehaviorSpec, LayerStages};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum VerticalAlign {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CameraSource {
    #[default]
    Local,
    Scene,
}

/// Controls uppercase conversion of text sprite content before glyph lookup.
#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum TextTransform {
    /// Preserve authored case (default). Glyphs are looked up as written.
    #[default]
    None,
    /// Force all characters to uppercase before glyph lookup.
    /// Use for retro/block-title rendering with the generic bitmap font.
    Uppercase,
}

/// Glow halo effect for a text sprite.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Glow {
    /// Glow colour. If None, uses a dimmed version of the sprite's fg_colour.
    pub colour: Option<TermColour>,
    /// Radius in cells. 1 = 8-neighbor halo. Default 1.
    #[serde(default = "default_glow_radius")]
    pub radius: u16,
}

fn default_glow_radius() -> u16 {
    1
}

fn default_true() -> bool {
    true
}

fn default_grid_line() -> u16 {
    1
}

fn default_grid_span() -> u16 {
    1
}

fn default_panel_padding() -> u16 {
    1
}

fn default_panel_border_width() -> u16 {
    1
}

fn default_panel_radius() -> u16 {
    1
}

fn default_panel_shadow_x() -> i32 {
    1
}

fn default_panel_shadow_y() -> i32 {
    1
}

fn default_scale() -> f32 {
    1.0
}

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
/// Declares the main axis used by a flex container sprite.
pub enum FlexDirection {
    /// Stacks children top-to-bottom.
    #[default]
    Column,
    /// Stacks children left-to-right.
    Row,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpriteSizePreset {
    Small,
    Medium,
    Large,
    XLarge,
}

impl SpriteSizePreset {
    pub const fn generic_mode(self) -> &'static str {
        match self {
            Self::Small => "1",
            Self::Medium => "2",
            Self::Large => "3",
            Self::XLarge => "4",
        }
    }

    pub const fn image_scale_ratio(self) -> (u16, u16) {
        match self {
            Self::Small => (1, 3),
            Self::Medium => (1, 2),
            Self::Large => (2, 3),
            Self::XLarge => (3, 4),
        }
    }

    pub const fn obj_dimensions(self) -> (u16, u16) {
        match self {
            Self::Small => (32, 12),
            Self::Medium => (64, 24),
            Self::Large => (96, 36),
            Self::XLarge => (128, 48),
        }
    }
}

impl TryFrom<u8> for SpriteSizePreset {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Small),
            2 => Ok(Self::Medium),
            3 => Ok(Self::Large),
            4 => Ok(Self::XLarge),
            other => Err(format!("unsupported sprite size preset: {other}")),
        }
    }
}

impl<'de> Deserialize<'de> for SpriteSizePreset {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = u8::deserialize(deserializer)?;
        Self::try_from(raw).map_err(serde::de::Error::custom)
    }
}

/// A drawable object within a layer. Has its own local bitmap position and lifecycle.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum Sprite {
    /// Terminal-native text sprite.
    /// If `font` is None: characters written directly to buffer (fast path).
    /// If `font` is Some: text is rasterized using a bitmap font definition (rasterizer path).
    Text {
        /// Optional stable identifier for script-side lookups (e.g. intro.rs logic hooks).
        #[serde(default)]
        id: Option<String>,
        content: String,
        #[serde(default)]
        x: i32,
        #[serde(default)]
        y: i32,
        #[serde(default)]
        z_index: i32,
        #[serde(default = "default_grid_line", rename = "grid-row")]
        grid_row: u16,
        #[serde(default = "default_grid_line", rename = "grid-col")]
        grid_col: u16,
        #[serde(default = "default_grid_span", rename = "row-span")]
        row_span: u16,
        #[serde(default = "default_grid_span", rename = "col-span")]
        col_span: u16,
        #[serde(default)]
        size: Option<SpriteSizePreset>,
        font: Option<String>,
        /// Optional per-sprite font mode override (e.g. ascii/raster).
        #[serde(default, rename = "force-font-mode")]
        force_font_mode: Option<String>,
        #[serde(default, rename = "align-x")]
        align_x: Option<HorizontalAlign>,
        #[serde(default, rename = "align-y")]
        align_y: Option<VerticalAlign>,
        fg_colour: Option<TermColour>,
        bg_colour: Option<TermColour>,
        /// Delay from scene start (ms) before sprite becomes visible.
        #[serde(default)]
        appear_at_ms: Option<u64>,
        /// Optional absolute scene time (ms) after which sprite is hidden.
        #[serde(default)]
        disappear_at_ms: Option<u64>,
        /// Optional reveal duration (ms). When set, text appears left->right.
        #[serde(default)]
        reveal_ms: Option<u64>,
        /// Hide the sprite immediately when scene enters on_leave.
        #[serde(default)]
        hide_on_leave: bool,
        /// Initial visibility state. Can be toggled at runtime via scene.set(id, "visible", bool).
        #[serde(default = "default_true")]
        visible: bool,
        #[serde(default)]
        stages: LayerStages,
        #[serde(default)]
        animations: Vec<crate::scene::Animation>,
        #[serde(default)]
        behaviors: Vec<BehaviorSpec>,
        /// Optional glow halo rendered behind the sprite.
        #[serde(default)]
        glow: Option<Glow>,
        /// Text case transformation applied before glyph lookup.
        /// Default: `none` (preserve authored case).
        #[serde(default)]
        text_transform: TextTransform,
        /// Horizontal scale factor applied when blitting the rasterized text.
        /// 1.0 = no change, 1.5 = 50% wider, 0.5 = half width.
        /// Only affects bitmap/raster font paths; has no effect on native terminal text.
        #[serde(default = "default_scale", rename = "scale-x")]
        scale_x: f32,
        /// Vertical scale factor applied when blitting the rasterized text.
        /// 1.0 = no change, 1.5 = 50% taller, 0.5 = half height.
        /// Only affects bitmap/raster font paths; has no effect on native terminal text.
        #[serde(default = "default_scale", rename = "scale-y")]
        scale_y: f32,
    },
    /// Bitmap image sprite rendered on terminal grid in selected mode.
    Image {
        #[serde(default)]
        id: Option<String>,
        source: String,
        /// Optional spritesheet column count. When >1, image is treated as a sheet.
        #[serde(default, rename = "spritesheet-columns")]
        spritesheet_columns: Option<u16>,
        /// Optional spritesheet row count. When >1, image is treated as a sheet.
        #[serde(default, rename = "spritesheet-rows")]
        spritesheet_rows: Option<u16>,
        /// Optional 0-based frame index selected from spritesheet cells.
        #[serde(default, rename = "frame-index")]
        frame_index: Option<u16>,
        #[serde(default)]
        x: i32,
        #[serde(default)]
        y: i32,
        #[serde(default)]
        z_index: i32,
        #[serde(default = "default_grid_line", rename = "grid-row")]
        grid_row: u16,
        #[serde(default = "default_grid_line", rename = "grid-col")]
        grid_col: u16,
        #[serde(default = "default_grid_span", rename = "row-span")]
        row_span: u16,
        #[serde(default = "default_grid_span", rename = "col-span")]
        col_span: u16,
        #[serde(default)]
        size: Option<SpriteSizePreset>,
        #[serde(default)]
        width: Option<u16>,
        #[serde(default)]
        height: Option<u16>,
        /// When true, scales the image to exactly fill its resolved draw area.
        #[serde(default, rename = "stretch-to-area")]
        stretch_to_area: bool,
        #[serde(default, rename = "align-x")]
        align_x: Option<HorizontalAlign>,
        #[serde(default, rename = "align-y")]
        align_y: Option<VerticalAlign>,
        #[serde(default)]
        appear_at_ms: Option<u64>,
        #[serde(default)]
        disappear_at_ms: Option<u64>,
        #[serde(default)]
        hide_on_leave: bool,
        /// Initial visibility state. Can be toggled at runtime via scene.set(id, "visible", bool).
        #[serde(default = "default_true")]
        visible: bool,
        #[serde(default)]
        stages: LayerStages,
        #[serde(default)]
        animations: Vec<crate::scene::Animation>,
        #[serde(default)]
        behaviors: Vec<BehaviorSpec>,
    },
    /// Vector polyline/polygon sprite rendered as line segments.
    Vector {
        #[serde(default)]
        id: Option<String>,
        /// List of points in local sprite space.
        #[serde(default)]
        points: Vec<[i32; 2]>,
        /// When true, closes the shape by connecting last->first.
        #[serde(default)]
        closed: bool,
        /// Glyph used for line rasterization.
        #[serde(default, rename = "draw-char")]
        draw_char: Option<String>,
        #[serde(default)]
        x: i32,
        #[serde(default)]
        y: i32,
        #[serde(default)]
        z_index: i32,
        #[serde(default = "default_grid_line", rename = "grid-row")]
        grid_row: u16,
        #[serde(default = "default_grid_line", rename = "grid-col")]
        grid_col: u16,
        #[serde(default = "default_grid_span", rename = "row-span")]
        row_span: u16,
        #[serde(default = "default_grid_span", rename = "col-span")]
        col_span: u16,
        #[serde(default, rename = "align-x")]
        align_x: Option<HorizontalAlign>,
        #[serde(default, rename = "align-y")]
        align_y: Option<VerticalAlign>,
        fg_colour: Option<TermColour>,
        bg_colour: Option<TermColour>,
        #[serde(default)]
        appear_at_ms: Option<u64>,
        #[serde(default)]
        disappear_at_ms: Option<u64>,
        #[serde(default)]
        hide_on_leave: bool,
        /// Initial visibility state. Can be toggled at runtime via scene.set(id, "visible", bool).
        #[serde(default = "default_true")]
        visible: bool,
        #[serde(default)]
        stages: LayerStages,
        #[serde(default)]
        animations: Vec<crate::scene::Animation>,
        #[serde(default)]
        behaviors: Vec<BehaviorSpec>,
    },
    /// Wavefront OBJ mesh rendered as terminal wireframe/material.
    Obj {
        #[serde(default)]
        id: Option<String>,
        source: String,
        #[serde(default)]
        x: i32,
        #[serde(default)]
        y: i32,
        #[serde(default)]
        z_index: i32,
        #[serde(default = "default_grid_line", rename = "grid-row")]
        grid_row: u16,
        #[serde(default = "default_grid_line", rename = "grid-col")]
        grid_col: u16,
        #[serde(default = "default_grid_span", rename = "row-span")]
        row_span: u16,
        #[serde(default = "default_grid_span", rename = "col-span")]
        col_span: u16,
        #[serde(default)]
        size: Option<SpriteSizePreset>,
        #[serde(default)]
        width: Option<u16>,
        #[serde(default)]
        height: Option<u16>,
        #[serde(default, rename = "surface-mode")]
        surface_mode: Option<String>,
        #[serde(default, rename = "backface-cull")]
        backface_cull: Option<bool>,
        /// Vertical clip region top edge (0.0 = top of sprite, 1.0 = bottom). Default 0.0.
        #[serde(default, rename = "clip-y-min")]
        clip_y_min: Option<f32>,
        /// Vertical clip region bottom edge (0.0 = top of sprite, 1.0 = bottom). Default 1.0.
        #[serde(default, rename = "clip-y-max")]
        clip_y_max: Option<f32>,
        #[serde(default)]
        scale: Option<f32>,
        #[serde(default, rename = "yaw-deg")]
        yaw_deg: Option<f32>,
        #[serde(default, rename = "pitch-deg")]
        pitch_deg: Option<f32>,
        #[serde(default, rename = "roll-deg")]
        roll_deg: Option<f32>,
        /// Static initial rotation of the mesh around the X axis (degrees).
        #[serde(default, rename = "rotation-x")]
        rotation_x: Option<f32>,
        /// Static initial rotation of the mesh around the Y axis (degrees).
        #[serde(default, rename = "rotation-y")]
        rotation_y: Option<f32>,
        /// Static initial rotation of the mesh around the Z axis (degrees).
        #[serde(default, rename = "rotation-z")]
        rotation_z: Option<f32>,
        #[serde(default, rename = "rotate-y-deg-per-sec")]
        rotate_y_deg_per_sec: Option<f32>,
        #[serde(default, rename = "camera-distance")]
        camera_distance: Option<f32>,
        #[serde(default, rename = "camera-source")]
        camera_source: CameraSource,
        #[serde(default, rename = "fov-degrees")]
        fov_degrees: Option<f32>,
        #[serde(default, rename = "near-clip")]
        near_clip: Option<f32>,
        #[serde(default, rename = "light-direction-x")]
        light_direction_x: Option<f32>,
        #[serde(default, rename = "light-direction-y")]
        light_direction_y: Option<f32>,
        #[serde(default, rename = "light-direction-z")]
        light_direction_z: Option<f32>,
        #[serde(default, rename = "light-2-direction-x")]
        light_2_direction_x: Option<f32>,
        #[serde(default, rename = "light-2-direction-y")]
        light_2_direction_y: Option<f32>,
        #[serde(default, rename = "light-2-direction-z")]
        light_2_direction_z: Option<f32>,
        #[serde(default, rename = "light-2-intensity")]
        light_2_intensity: Option<f32>,
        #[serde(default, rename = "light-point-x")]
        light_point_x: Option<f32>,
        #[serde(default, rename = "light-point-y")]
        light_point_y: Option<f32>,
        #[serde(default, rename = "light-point-z")]
        light_point_z: Option<f32>,
        #[serde(default, rename = "light-point-intensity")]
        light_point_intensity: Option<f32>,
        #[serde(default, rename = "light-point-colour")]
        light_point_colour: Option<TermColour>,
        #[serde(default, rename = "light-point-flicker-depth")]
        light_point_flicker_depth: Option<f32>,
        #[serde(default, rename = "light-point-flicker-hz")]
        light_point_flicker_hz: Option<f32>,
        /// Orbit angular speed (Hz) for point light 1 around the Y axis (smooth).
        #[serde(default, rename = "light-point-orbit-hz")]
        light_point_orbit_hz: Option<f32>,
        /// Teleport snap rate (Hz) for point light 1: instant jump to pseudo-random position at this frequency.
        #[serde(default, rename = "light-point-snap-hz")]
        light_point_snap_hz: Option<f32>,
        #[serde(default, rename = "light-point-2-x")]
        light_point_2_x: Option<f32>,
        #[serde(default, rename = "light-point-2-y")]
        light_point_2_y: Option<f32>,
        #[serde(default, rename = "light-point-2-z")]
        light_point_2_z: Option<f32>,
        #[serde(default, rename = "light-point-2-intensity")]
        light_point_2_intensity: Option<f32>,
        #[serde(default, rename = "light-point-2-colour")]
        light_point_2_colour: Option<TermColour>,
        #[serde(default, rename = "light-point-2-flicker-depth")]
        light_point_2_flicker_depth: Option<f32>,
        #[serde(default, rename = "light-point-2-flicker-hz")]
        light_point_2_flicker_hz: Option<f32>,
        /// Orbit angular speed (Hz) for point light 2 around the Y axis (smooth).
        #[serde(default, rename = "light-point-2-orbit-hz")]
        light_point_2_orbit_hz: Option<f32>,
        /// Teleport snap rate (Hz) for point light 2: instant jump to pseudo-random position at this frequency.
        #[serde(default, rename = "light-point-2-snap-hz")]
        light_point_2_snap_hz: Option<f32>,
        #[serde(default, rename = "cel-levels")]
        cel_levels: Option<u8>,
        #[serde(default, rename = "shadow-colour")]
        shadow_colour: Option<TermColour>,
        #[serde(default, rename = "midtone-colour")]
        midtone_colour: Option<TermColour>,
        #[serde(default, rename = "highlight-colour")]
        highlight_colour: Option<TermColour>,
        #[serde(default, rename = "tone-mix")]
        tone_mix: Option<f32>,
        /// Ambient light floor — brightness of unlit surfaces (0.0–1.0). Default 0.15.
        /// Set at runtime via `scene.set(id, "obj.ambient", v)`.
        #[serde(default, rename = "ambient")]
        ambient: Option<f32>,
        #[serde(default, rename = "smooth-shading")]
        smooth_shading: Option<bool>,
        /// Number of procedural latitude bands (sine-wave along world-Y). 0 = disabled.
        #[serde(default, rename = "latitude-bands")]
        latitude_bands: Option<u8>,
        /// Strength of latitude band modulation (0.0–1.0). Controls how much bands alter the shading.
        #[serde(default, rename = "latitude-band-depth")]
        latitude_band_depth: Option<f32>,
        /// RGB hex color for terrain (land) surface. When set, enables 3-D Perlin noise terrain.
        #[serde(default, rename = "terrain-color")]
        terrain_color: Option<TermColour>,
        /// Noise threshold for land vs. ocean (0.0–1.0). Default 0.5; higher = less land coverage.
        #[serde(default, rename = "terrain-threshold")]
        terrain_threshold: Option<f32>,
        /// Scale of 3-D noise for terrain features. Higher = more/smaller continents. Default 2.5.
        #[serde(default, rename = "terrain-noise-scale")]
        terrain_noise_scale: Option<f32>,
        /// Number of fBm octaves for terrain noise (1 = fast, 4 = detail-rich). Default 2.
        #[serde(default, rename = "terrain-noise-octaves")]
        terrain_noise_octaves: Option<u8>,
        /// Strength of marble turbulence on ocean pixels. 0.0 = flat ocean color.
        #[serde(default, rename = "marble-depth")]
        marble_depth: Option<f32>,
        /// When true, pixels below `terrain-threshold` are rendered transparent instead of
        /// using `fg_colour`. Use for cloud/overlay layers where non-cloud areas must be clear.
        #[serde(default, rename = "below-threshold-transparent")]
        below_threshold_transparent: bool,
        /// Polar ice cap color. When set, applies smooth ice coverage at high latitudes.
        #[serde(default, rename = "polar-ice-color")]
        polar_ice_color: Option<TermColour>,
        /// Latitude |y| (0=equator, 1=pole) where ice coverage begins. Default 0.78.
        #[serde(default, rename = "polar-ice-start")]
        polar_ice_start: Option<f32>,
        /// Latitude |y| where ice coverage is full. Default 0.92.
        #[serde(default, rename = "polar-ice-end")]
        polar_ice_end: Option<f32>,
        /// Desert/dry zone color for equatorial land regions.
        #[serde(default, rename = "desert-color")]
        desert_color: Option<TermColour>,
        /// Strength of desert biome blending (0.0–1.0). Default 0.0.
        #[serde(default, rename = "desert-strength")]
        desert_strength: Option<f32>,
        /// Atmosphere rim/glow color. When set, renders a thin halo at the planet limb.
        #[serde(default, rename = "atmo-color")]
        atmo_color: Option<TermColour>,
        /// Overall atmosphere blend strength (0.0–1.0). Default 0.0.
        #[serde(default, rename = "atmo-strength")]
        atmo_strength: Option<f32>,
        /// Rim falloff power for atmosphere effect (higher = thinner rim). Default 4.5.
        #[serde(default, rename = "atmo-rim-power")]
        atmo_rim_power: Option<f32>,
        /// Broad atmosphere haze strength (0.0–1.0). Default 0.0.
        #[serde(default, rename = "atmo-haze-strength")]
        atmo_haze_strength: Option<f32>,
        /// Haze falloff power for atmosphere effect (lower = broader). Default 1.8.
        #[serde(default, rename = "atmo-haze-power")]
        atmo_haze_power: Option<f32>,
        /// Atmospheric veil strength across the visible disk (0.0–1.0). Higher values can soften or obscure surface detail.
        #[serde(default, rename = "atmo-veil-strength")]
        atmo_veil_strength: Option<f32>,
        /// Veil falloff power (lower = broader coverage across the disk). Default 1.6.
        #[serde(default, rename = "atmo-veil-power")]
        atmo_veil_power: Option<f32>,
        /// Outer halo strength outside the planet silhouette (0.0–1.0). Default 0.0.
        #[serde(default, rename = "atmo-halo-strength")]
        atmo_halo_strength: Option<f32>,
        /// Outer halo width as a fraction of apparent planet radius (0.0–1.0). Default 0.12.
        #[serde(default, rename = "atmo-halo-width")]
        atmo_halo_width: Option<f32>,
        /// Outer halo falloff power (higher = tighter halo). Default 2.2.
        #[serde(default, rename = "atmo-halo-power")]
        atmo_halo_power: Option<f32>,
        /// Night-side city lights color. When set, renders procedural light clusters on the dark side.
        #[serde(default, rename = "night-light-color")]
        night_light_color: Option<TermColour>,
        /// Noise threshold for city light clusters (0.0–1.0). Default 0.82.
        #[serde(default, rename = "night-light-threshold")]
        night_light_threshold: Option<f32>,
        /// Brightness of night-side city light clusters. Default 0.0 (disabled).
        #[serde(default, rename = "night-light-intensity")]
        night_light_intensity: Option<f32>,
        #[serde(default, rename = "draw-char")]
        draw_char: Option<String>,
        #[serde(default, rename = "align-x")]
        align_x: Option<HorizontalAlign>,
        #[serde(default, rename = "align-y")]
        align_y: Option<VerticalAlign>,
        fg_colour: Option<TermColour>,
        bg_colour: Option<TermColour>,
        #[serde(default)]
        appear_at_ms: Option<u64>,
        #[serde(default)]
        disappear_at_ms: Option<u64>,
        #[serde(default)]
        hide_on_leave: bool,
        /// Initial visibility state. Can be toggled at runtime via scene.set(id, "visible", bool).
        #[serde(default = "default_true")]
        visible: bool,
        #[serde(default)]
        stages: LayerStages,
        #[serde(default)]
        animations: Vec<crate::scene::Animation>,
        #[serde(default)]
        behaviors: Vec<BehaviorSpec>,
        /// When `true` (default) and the scene has `prerender: true`, this sprite will be
        /// pre-rendered once at its initial pose during scene transition.
        /// Set to `false` to opt out (e.g. for sprites that animate continuously).
        #[serde(default = "default_true")]
        prerender: bool,
        /// When `true`, pre-bake all 72 rotation keyframes (every 5°) at scene load.
        /// Eliminates per-frame 3D rasterisation for slowly-rotating objects (e.g. a planet).
        /// Requires the sprite to have a nonzero `rotate-y-deg-per-sec`.
        #[serde(default, rename = "prerender-anim")]
        prerender_anim: bool,
        /// Terrain-plane geometry override: height amplitude (default: 1.0).
        /// Set at runtime via `scene.set(id, "terrain.amplitude", v)`.
        #[serde(default, rename = "terrain-amplitude")]
        terrain_plane_amplitude: Option<f32>,
        /// Terrain-plane geometry override: noise frequency (default: 1.0).
        #[serde(default, rename = "terrain-frequency")]
        terrain_plane_frequency: Option<f32>,
        /// Terrain-plane geometry override: fBm roughness 0.0–1.0 (default: 1.0).
        #[serde(default, rename = "terrain-roughness")]
        terrain_plane_roughness: Option<f32>,
        /// Terrain-plane geometry override: fBm octave count 1–3 (default: 3).
        #[serde(default, rename = "terrain-octaves")]
        terrain_plane_octaves: Option<u8>,
        /// Terrain-plane geometry override: X seed offset, shifts noise region (default: 0.0).
        #[serde(default, rename = "terrain-seed-x")]
        terrain_plane_seed_x: Option<f32>,
        /// Terrain-plane geometry override: Z seed offset, shifts noise region (default: 0.0).
        #[serde(default, rename = "terrain-seed-z")]
        terrain_plane_seed_z: Option<f32>,
        /// Terrain-plane geometry override: lacunarity — freq multiplier between octaves (default: 2.0).
        #[serde(default, rename = "terrain-lacunarity")]
        terrain_plane_lacunarity: Option<f32>,
        /// Terrain-plane geometry override: ridge mode — abs() each octave for sharp peaks (default: false).
        #[serde(default, rename = "terrain-ridge")]
        terrain_plane_ridge: Option<bool>,
        /// Terrain-plane geometry override: plateau strength 0.0–1.0 — flatten peaks (default: 0.0).
        #[serde(default, rename = "terrain-plateau")]
        terrain_plane_plateau: Option<f32>,
        /// Terrain-plane geometry override: sea level 0.0–1.0 — clamp floor upward (default: 0.0).
        #[serde(default, rename = "terrain-sea-level")]
        terrain_plane_sea_level: Option<f32>,
        /// Terrain-plane geometry override: anisotropic X stretch (default: 1.0).
        #[serde(default, rename = "terrain-scale-x")]
        terrain_plane_scale_x: Option<f32>,
        /// Terrain-plane geometry override: anisotropic Z stretch (default: 1.0).
        #[serde(default, rename = "terrain-scale-z")]
        terrain_plane_scale_z: Option<f32>,

        // ── World generator params (world:// URI) ──────────────────────────
        // These drive `engine_terrain::WorldGenParams` → full biome/climate pipeline.
        // Changed at runtime via `scene.set(id, "world.<field>", v)`.

        /// World shape: "sphere" (default) or "flat".
        #[serde(default, rename = "world-shape")]
        world_gen_shape: Option<String>,
        /// Sphere base primitive: "cube" (default), "uv", "tetra", "octa", "icosa".
        #[serde(default, rename = "world-base")]
        world_gen_base: Option<String>,
        /// World coloring strategy: "biome" (default), "altitude", or "none".
        #[serde(default, rename = "world-coloring")]
        world_gen_coloring: Option<String>,
        /// World generator seed (integer). Default 0.
        #[serde(default, rename = "world-seed")]
        world_gen_seed: Option<u64>,
        /// Target ocean fraction 0.0–1.0 (default 0.55).
        #[serde(default, rename = "world-ocean-fraction")]
        world_gen_ocean_fraction: Option<f64>,
        /// Continent noise frequency scale, larger = smaller continents (default 2.5).
        #[serde(default, rename = "world-continent-scale")]
        world_gen_continent_scale: Option<f64>,
        /// Domain warp strength 0.0–1.5, higher = more chaotic coastlines (default 0.65).
        #[serde(default, rename = "world-continent-warp")]
        world_gen_continent_warp: Option<f64>,
        /// fBm octaves for continent noise 3–7 (default 5).
        #[serde(default, rename = "world-continent-octaves")]
        world_gen_continent_octaves: Option<u8>,
        /// Mountain ridge frequency, higher = narrower chains (default 6.0).
        #[serde(default, rename = "world-mountain-scale")]
        world_gen_mountain_scale: Option<f64>,
        /// Mountain elevation contribution over land 0.0–1.0 (default 0.45).
        #[serde(default, rename = "world-mountain-strength")]
        world_gen_mountain_strength: Option<f64>,
        /// Ridged noise octave count for mountain detail 2–8 (default 5).
        #[serde(default, rename = "world-mountain-ridge-octaves")]
        world_gen_mountain_ridge_octaves: Option<u8>,
        /// Regional moisture noise frequency (default 3.0).
        #[serde(default, rename = "world-moisture-scale")]
        world_gen_moisture_scale: Option<f64>,
        /// Polar cold zone strength 0.0–2.0 (default 1.0).
        #[serde(default, rename = "world-ice-cap-strength")]
        world_gen_ice_cap_strength: Option<f64>,
        /// Temperature reduction per unit elevation 0.0–1.0 (default 0.6).
        #[serde(default, rename = "world-lapse-rate")]
        world_gen_lapse_rate: Option<f64>,
        /// Moisture reduction above mountains 0.0–1.0 (default 0.35).
        #[serde(default, rename = "world-rain-shadow")]
        world_gen_rain_shadow: Option<f64>,
        /// Radial vertex displacement range ±N (sphere only, default 0.22).
        #[serde(default, rename = "world-displacement-scale")]
        world_gen_displacement_scale: Option<f32>,
        /// Mesh subdivision count; higher = smoother sphere (default 32).
        #[serde(default, rename = "world-subdivisions")]
        world_gen_subdivisions: Option<u32>,

        /// Object world-space translation (applied before view/projection).
        /// Useful when driving multiple OBJ sprites from one shared scene camera.
        /// Set per frame via `scene.set(id, "obj.world.x/y/z", value)`.
        #[serde(default, rename = "world-x")]
        world_x: Option<f32>,
        #[serde(default, rename = "world-y")]
        world_y: Option<f32>,
        #[serde(default, rename = "world-z")]
        world_z: Option<f32>,
        /// Cockpit camera world-space position override.
        /// When set, replaces the camera_distance-derived default position (0, 0, -camera_distance).
        /// Set per frame via `scene.set(id, "obj.cam.wx"/"obj.cam.wy"/"obj.cam.wz", f)`.
        #[serde(default)]
        cam_world_x: Option<f32>,
        #[serde(default)]
        cam_world_y: Option<f32>,
        #[serde(default)]
        cam_world_z: Option<f32>,
        /// Camera view-basis vectors override. Default: right=(1,0,0), up=(0,1,0), forward=(0,0,1).
        /// Set per frame via `scene.set(id, "obj.view.rx/.../...", f)` to drive from ship 3D basis.
        #[serde(default)]
        view_right_x: Option<f32>,
        #[serde(default)]
        view_right_y: Option<f32>,
        #[serde(default)]
        view_right_z: Option<f32>,
        #[serde(default)]
        view_up_x: Option<f32>,
        #[serde(default)]
        view_up_y: Option<f32>,
        #[serde(default)]
        view_up_z: Option<f32>,
        #[serde(default)]
        view_fwd_x: Option<f32>,
        #[serde(default)]
        view_fwd_y: Option<f32>,
        #[serde(default)]
        view_fwd_z: Option<f32>,
    },
    /// Body-backed planet sprite rendered from mod body + preset catalogs.
    Planet {
        #[serde(default)]
        id: Option<String>,
        #[serde(rename = "body-id")]
        body_id: String,
        #[serde(default)]
        preset: Option<String>,
        /// Optional sphere mesh override. Defaults to `/assets/3d/sphere.obj`.
        #[serde(default, rename = "mesh-source")]
        mesh_source: Option<String>,
        #[serde(default)]
        x: i32,
        #[serde(default)]
        y: i32,
        #[serde(default)]
        z_index: i32,
        #[serde(default = "default_grid_line", rename = "grid-row")]
        grid_row: u16,
        #[serde(default = "default_grid_line", rename = "grid-col")]
        grid_col: u16,
        #[serde(default = "default_grid_span", rename = "row-span")]
        row_span: u16,
        #[serde(default = "default_grid_span", rename = "col-span")]
        col_span: u16,
        #[serde(default)]
        size: Option<SpriteSizePreset>,
        #[serde(default)]
        width: Option<u16>,
        #[serde(default)]
        height: Option<u16>,
        #[serde(default)]
        scale: Option<f32>,
        #[serde(default, rename = "yaw-deg")]
        yaw_deg: Option<f32>,
        #[serde(default, rename = "pitch-deg")]
        pitch_deg: Option<f32>,
        #[serde(default, rename = "roll-deg")]
        roll_deg: Option<f32>,
        #[serde(default, rename = "spin-deg")]
        spin_deg: Option<f32>,
        #[serde(default, rename = "cloud-spin-deg")]
        cloud_spin_deg: Option<f32>,
        #[serde(default, rename = "cloud2-spin-deg")]
        cloud2_spin_deg: Option<f32>,
        #[serde(default, rename = "observer-altitude-km")]
        observer_altitude_km: Option<f32>,
        #[serde(default, rename = "camera-distance")]
        camera_distance: Option<f32>,
        #[serde(default, rename = "camera-source")]
        camera_source: CameraSource,
        #[serde(default, rename = "fov-degrees")]
        fov_degrees: Option<f32>,
        #[serde(default, rename = "near-clip")]
        near_clip: Option<f32>,
        #[serde(default, rename = "sun-dir-x")]
        sun_dir_x: Option<f32>,
        #[serde(default, rename = "sun-dir-y")]
        sun_dir_y: Option<f32>,
        #[serde(default, rename = "sun-dir-z")]
        sun_dir_z: Option<f32>,
        #[serde(default, rename = "align-x")]
        align_x: Option<HorizontalAlign>,
        #[serde(default, rename = "align-y")]
        align_y: Option<VerticalAlign>,
        #[serde(default)]
        appear_at_ms: Option<u64>,
        #[serde(default)]
        disappear_at_ms: Option<u64>,
        #[serde(default)]
        hide_on_leave: bool,
        /// Initial visibility state. Can be toggled at runtime via scene.set(id, "visible", bool).
        #[serde(default = "default_true")]
        visible: bool,
        #[serde(default)]
        stages: LayerStages,
        #[serde(default)]
        animations: Vec<crate::scene::Animation>,
        #[serde(default)]
        behaviors: Vec<BehaviorSpec>,
    },
    /// UI panel container rendered as a themed box with optional border, corner radius and shadow.
    #[serde(rename = "panel")]
    Panel {
        #[serde(default)]
        id: Option<String>,
        #[serde(default)]
        x: i32,
        #[serde(default)]
        y: i32,
        #[serde(default)]
        z_index: i32,
        #[serde(default = "default_grid_line", rename = "grid-row")]
        grid_row: u16,
        #[serde(default = "default_grid_line", rename = "grid-col")]
        grid_col: u16,
        #[serde(default = "default_grid_span", rename = "row-span")]
        row_span: u16,
        #[serde(default = "default_grid_span", rename = "col-span")]
        col_span: u16,
        #[serde(default)]
        width: Option<u16>,
        #[serde(default, rename = "width-percent")]
        width_percent: Option<u16>,
        #[serde(default)]
        height: Option<u16>,
        #[serde(default = "default_panel_padding")]
        padding: u16,
        #[serde(default = "default_panel_border_width", rename = "border-width")]
        border_width: u16,
        #[serde(default = "default_panel_radius", rename = "corner-radius")]
        corner_radius: u16,
        #[serde(default = "default_panel_shadow_x", rename = "shadow-x")]
        shadow_x: i32,
        #[serde(default = "default_panel_shadow_y", rename = "shadow-y")]
        shadow_y: i32,
        #[serde(default, rename = "align-x")]
        align_x: Option<HorizontalAlign>,
        #[serde(default, rename = "align-y")]
        align_y: Option<VerticalAlign>,
        fg_colour: Option<TermColour>,
        bg_colour: Option<TermColour>,
        #[serde(rename = "border-colour")]
        border_colour: Option<TermColour>,
        #[serde(rename = "shadow-colour")]
        shadow_colour: Option<TermColour>,
        #[serde(default)]
        appear_at_ms: Option<u64>,
        #[serde(default)]
        disappear_at_ms: Option<u64>,
        #[serde(default)]
        hide_on_leave: bool,
        /// Initial visibility state. Can be toggled at runtime via scene.set(id, "visible", bool).
        #[serde(default = "default_true")]
        visible: bool,
        #[serde(default)]
        stages: LayerStages,
        #[serde(default)]
        animations: Vec<crate::scene::Animation>,
        #[serde(default)]
        behaviors: Vec<BehaviorSpec>,
        #[serde(default)]
        children: Vec<Sprite>,
    },
    /// Grid layout container. Children are renderable sprites arranged in rows/columns.
    Grid {
        #[serde(default)]
        id: Option<String>,
        #[serde(default)]
        x: i32,
        #[serde(default)]
        y: i32,
        #[serde(default)]
        z_index: i32,
        #[serde(default = "default_grid_line", rename = "grid-row")]
        grid_row: u16,
        #[serde(default = "default_grid_line", rename = "grid-col")]
        grid_col: u16,
        #[serde(default = "default_grid_span", rename = "row-span")]
        row_span: u16,
        #[serde(default = "default_grid_span", rename = "col-span")]
        col_span: u16,
        #[serde(default)]
        width: Option<u16>,
        #[serde(default)]
        height: Option<u16>,
        #[serde(default, rename = "gap-x")]
        gap_x: u16,
        #[serde(default, rename = "gap-y")]
        gap_y: u16,
        #[serde(default, rename = "align-x")]
        align_x: Option<HorizontalAlign>,
        #[serde(default, rename = "align-y")]
        align_y: Option<VerticalAlign>,
        #[serde(default)]
        appear_at_ms: Option<u64>,
        #[serde(default)]
        disappear_at_ms: Option<u64>,
        #[serde(default)]
        hide_on_leave: bool,
        /// Initial visibility state. Can be toggled at runtime via scene.set(id, "visible", bool).
        #[serde(default = "default_true")]
        visible: bool,
        #[serde(default)]
        stages: LayerStages,
        #[serde(default)]
        animations: Vec<crate::scene::Animation>,
        #[serde(default)]
        behaviors: Vec<BehaviorSpec>,
        columns: Vec<String>,
        rows: Vec<String>,
        #[serde(default)]
        children: Vec<Sprite>,
    },
    /// Flex layout container. Children are auto-stacked vertically (column) or horizontally (row).
    #[serde(rename = "flex")]
    Flex {
        #[serde(default)]
        id: Option<String>,
        #[serde(default)]
        x: i32,
        #[serde(default)]
        y: i32,
        #[serde(default)]
        z_index: i32,
        #[serde(default = "default_grid_line", rename = "grid-row")]
        grid_row: u16,
        #[serde(default = "default_grid_line", rename = "grid-col")]
        grid_col: u16,
        #[serde(default = "default_grid_span", rename = "row-span")]
        row_span: u16,
        #[serde(default = "default_grid_span", rename = "col-span")]
        col_span: u16,
        #[serde(default)]
        width: Option<u16>,
        #[serde(default)]
        height: Option<u16>,
        #[serde(default)]
        gap: u16,
        #[serde(default)]
        direction: FlexDirection,
        #[serde(default, rename = "align-x")]
        align_x: Option<HorizontalAlign>,
        #[serde(default, rename = "align-y")]
        align_y: Option<VerticalAlign>,
        #[serde(default)]
        appear_at_ms: Option<u64>,
        #[serde(default)]
        disappear_at_ms: Option<u64>,
        #[serde(default)]
        hide_on_leave: bool,
        /// Initial visibility state. Can be toggled at runtime via scene.set(id, "visible", bool).
        #[serde(default = "default_true")]
        visible: bool,
        #[serde(default)]
        stages: LayerStages,
        #[serde(default)]
        animations: Vec<crate::scene::Animation>,
        #[serde(default)]
        behaviors: Vec<BehaviorSpec>,
        #[serde(default)]
        children: Vec<Sprite>,
    },
    /// A pre-rendered 3D scene blitted from the Scene3D atlas.
    /// The atlas is populated during scene preparation (before first frame) from a `.scene3d.yml`.
    Scene3D {
        #[serde(default)]
        id: Option<String>,
        /// Path to `.scene3d.yml` relative to the mod root.
        src: String,
        /// Which named frame from the atlas to display. Changed at runtime via `scene.set`.
        #[serde(default)]
        frame: String,
        #[serde(default, rename = "camera-source")]
        camera_source: CameraSource,
        #[serde(default)]
        x: i32,
        #[serde(default)]
        y: i32,
        #[serde(default)]
        z_index: i32,
        #[serde(default = "default_grid_line", rename = "grid-row")]
        grid_row: u16,
        #[serde(default = "default_grid_line", rename = "grid-col")]
        grid_col: u16,
        #[serde(default = "default_grid_span", rename = "row-span")]
        row_span: u16,
        #[serde(default = "default_grid_span", rename = "col-span")]
        col_span: u16,
        #[serde(default)]
        appear_at_ms: Option<u64>,
        #[serde(default)]
        disappear_at_ms: Option<u64>,
        #[serde(default)]
        hide_on_leave: bool,
        /// Initial visibility state. Can be toggled at runtime via scene.set(id, "visible", bool).
        #[serde(default = "default_true")]
        visible: bool,
        #[serde(default)]
        stages: LayerStages,
        #[serde(default)]
        animations: Vec<crate::scene::Animation>,
        #[serde(default)]
        behaviors: Vec<BehaviorSpec>,
    },
}

impl Sprite {
    pub fn id(&self) -> Option<&str> {
        match self {
            Sprite::Text { id, .. }
            | Sprite::Image { id, .. }
            | Sprite::Obj { id, .. }
            | Sprite::Planet { id, .. }
            | Sprite::Panel { id, .. }
            | Sprite::Grid { id, .. }
            | Sprite::Flex { id, .. }
            | Sprite::Scene3D { id, .. }
            | Sprite::Vector { id, .. } => id.as_deref(),
        }
    }

    pub fn z_index(&self) -> i32 {
        match self {
            Sprite::Text { z_index, .. }
            | Sprite::Image { z_index, .. }
            | Sprite::Obj { z_index, .. }
            | Sprite::Planet { z_index, .. }
            | Sprite::Panel { z_index, .. }
            | Sprite::Grid { z_index, .. }
            | Sprite::Flex { z_index, .. }
            | Sprite::Scene3D { z_index, .. }
            | Sprite::Vector { z_index, .. } => *z_index,
        }
    }

    pub fn stages(&self) -> &LayerStages {
        match self {
            Sprite::Text { stages, .. }
            | Sprite::Image { stages, .. }
            | Sprite::Obj { stages, .. }
            | Sprite::Planet { stages, .. }
            | Sprite::Panel { stages, .. }
            | Sprite::Grid { stages, .. }
            | Sprite::Flex { stages, .. }
            | Sprite::Scene3D { stages, .. }
            | Sprite::Vector { stages, .. } => stages,
        }
    }

    pub fn grid_position(&self) -> (u16, u16, u16, u16) {
        let (row, col, row_span, col_span) = match self {
            Sprite::Text {
                grid_row,
                grid_col,
                row_span,
                col_span,
                ..
            }
            | Sprite::Image {
                grid_row,
                grid_col,
                row_span,
                col_span,
                ..
            }
            | Sprite::Obj {
                grid_row,
                grid_col,
                row_span,
                col_span,
                ..
            }
            | Sprite::Planet {
                grid_row,
                grid_col,
                row_span,
                col_span,
                ..
            }
            | Sprite::Panel {
                grid_row,
                grid_col,
                row_span,
                col_span,
                ..
            }
            | Sprite::Grid {
                grid_row,
                grid_col,
                row_span,
                col_span,
                ..
            }
            | Sprite::Flex {
                grid_row,
                grid_col,
                row_span,
                col_span,
                ..
            }
            | Sprite::Scene3D {
                grid_row,
                grid_col,
                row_span,
                col_span,
                ..
            }
            | Sprite::Vector {
                grid_row,
                grid_col,
                row_span,
                col_span,
                ..
            } => (*grid_row, *grid_col, *row_span, *col_span),
        };
        (row.max(1), col.max(1), row_span.max(1), col_span.max(1))
    }

    pub fn walk_recursive<'a, F>(&'a self, visit: &mut F)
    where
        F: FnMut(&'a Sprite),
    {
        visit(self);
        match self {
            Sprite::Panel { children, .. }
            | Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. } => {
                for child in children {
                    child.walk_recursive(visit);
                }
            }
            _ => {}
        }
    }

    pub fn behaviors(&self) -> &[BehaviorSpec] {
        match self {
            Sprite::Text { behaviors, .. }
            | Sprite::Image { behaviors, .. }
            | Sprite::Obj { behaviors, .. }
            | Sprite::Planet { behaviors, .. }
            | Sprite::Panel { behaviors, .. }
            | Sprite::Grid { behaviors, .. }
            | Sprite::Flex { behaviors, .. }
            | Sprite::Scene3D { behaviors, .. }
            | Sprite::Vector { behaviors, .. } => behaviors,
        }
    }

    pub fn hide_on_leave(&self) -> bool {
        match self {
            Sprite::Text { hide_on_leave, .. }
            | Sprite::Image { hide_on_leave, .. }
            | Sprite::Obj { hide_on_leave, .. }
            | Sprite::Planet { hide_on_leave, .. }
            | Sprite::Panel { hide_on_leave, .. }
            | Sprite::Grid { hide_on_leave, .. }
            | Sprite::Flex { hide_on_leave, .. }
            | Sprite::Scene3D { hide_on_leave, .. }
            | Sprite::Vector { hide_on_leave, .. } => *hide_on_leave,
        }
    }

    pub fn visible(&self) -> bool {
        match self {
            Sprite::Text { visible, .. }
            | Sprite::Image { visible, .. }
            | Sprite::Obj { visible, .. }
            | Sprite::Planet { visible, .. }
            | Sprite::Panel { visible, .. }
            | Sprite::Grid { visible, .. }
            | Sprite::Flex { visible, .. }
            | Sprite::Scene3D { visible, .. }
            | Sprite::Vector { visible, .. } => *visible,
        }
    }

    pub fn appear_at_ms(&self) -> Option<u64> {
        match self {
            Sprite::Text { appear_at_ms, .. }
            | Sprite::Image { appear_at_ms, .. }
            | Sprite::Obj { appear_at_ms, .. }
            | Sprite::Planet { appear_at_ms, .. }
            | Sprite::Panel { appear_at_ms, .. }
            | Sprite::Grid { appear_at_ms, .. }
            | Sprite::Flex { appear_at_ms, .. }
            | Sprite::Scene3D { appear_at_ms, .. }
            | Sprite::Vector { appear_at_ms, .. } => *appear_at_ms,
        }
    }

    pub fn disappear_at_ms(&self) -> Option<u64> {
        match self {
            Sprite::Text {
                disappear_at_ms, ..
            }
            | Sprite::Image {
                disappear_at_ms, ..
            }
            | Sprite::Obj {
                disappear_at_ms, ..
            }
            | Sprite::Planet {
                disappear_at_ms, ..
            }
            | Sprite::Panel {
                disappear_at_ms, ..
            }
            | Sprite::Grid {
                disappear_at_ms, ..
            }
            | Sprite::Flex {
                disappear_at_ms, ..
            }
            | Sprite::Scene3D {
                disappear_at_ms, ..
            }
            | Sprite::Vector {
                disappear_at_ms, ..
            } => *disappear_at_ms,
        }
    }

    pub fn animations(&self) -> &[crate::scene::Animation] {
        match self {
            Sprite::Text { animations, .. }
            | Sprite::Image { animations, .. }
            | Sprite::Obj { animations, .. }
            | Sprite::Planet { animations, .. }
            | Sprite::Panel { animations, .. }
            | Sprite::Grid { animations, .. }
            | Sprite::Flex { animations, .. }
            | Sprite::Scene3D { animations, .. }
            | Sprite::Vector { animations, .. } => animations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Sprite, SpriteSizePreset};

    #[test]
    fn supports_negative_sprite_offsets() {
        let raw = r#"
type: text
content: "TEST"
x: -2
y: -8
"#;
        let sprite: Sprite = serde_yaml::from_str(raw).expect("sprite should parse");
        match sprite {
            Sprite::Text { x, y, .. } => {
                assert_eq!(x, -2);
                assert_eq!(y, -8);
            }
            Sprite::Image { .. }
            | Sprite::Obj { .. }
            | Sprite::Planet { .. }
            | Sprite::Vector { .. }
            | Sprite::Panel { .. }
            | Sprite::Grid { .. }
            | Sprite::Flex { .. }
            | Sprite::Scene3D { .. } => {
                panic!("expected text sprite")
            }
        }
    }

    #[test]
    fn parses_force_font_mode() {
        let raw = r#"
type: text
content: "TEST"
font: "generic:2"
force-font-mode: ascii
"#;
        let sprite: Sprite = serde_yaml::from_str(raw).expect("sprite should parse");
        match sprite {
            Sprite::Text {
                force_font_mode,
                behaviors,
                ..
            } => {
                assert_eq!(force_font_mode.as_deref(), Some("ascii"));
                assert!(behaviors.is_empty());
            }
            Sprite::Image { .. }
            | Sprite::Obj { .. }
            | Sprite::Planet { .. }
            | Sprite::Vector { .. }
            | Sprite::Panel { .. }
            | Sprite::Grid { .. }
            | Sprite::Flex { .. }
            | Sprite::Scene3D { .. } => {
                panic!("expected text sprite")
            }
        }
    }

    #[test]
    fn parses_image_sprite() {
        let raw = r#"
type: image
source: "/assets/images/tux.png"
size: 3
stretch-to-area: true
"#;
        let sprite: Sprite = serde_yaml::from_str(raw).expect("image sprite should parse");
        match sprite {
            Sprite::Image {
                source,
                spritesheet_columns,
                spritesheet_rows,
                frame_index,
                size,
                width,
                height,
                stretch_to_area,
                ..
            } => {
                assert_eq!(source, "/assets/images/tux.png");
                assert_eq!(spritesheet_columns, None);
                assert_eq!(spritesheet_rows, None);
                assert_eq!(frame_index, None);
                assert_eq!(size, Some(SpriteSizePreset::Large));
                assert_eq!(width, None);
                assert_eq!(height, None);
                assert!(stretch_to_area);
            }
            Sprite::Text { .. }
            | Sprite::Obj { .. }
            | Sprite::Planet { .. }
            | Sprite::Vector { .. }
            | Sprite::Panel { .. }
            | Sprite::Grid { .. }
            | Sprite::Flex { .. }
            | Sprite::Scene3D { .. } => {
                panic!("expected image sprite")
            }
        }
    }

    #[test]
    fn parses_image_spritesheet_fields() {
        let raw = r#"
type: image
source: "/assets/images/difficulty1.png"
spritesheet-columns: 5
spritesheet-rows: 1
frame-index: 3
"#;
        let sprite: Sprite = serde_yaml::from_str(raw).expect("image sprite should parse");
        match sprite {
            Sprite::Image {
                spritesheet_columns,
                spritesheet_rows,
                frame_index,
                ..
            } => {
                assert_eq!(spritesheet_columns, Some(5));
                assert_eq!(spritesheet_rows, Some(1));
                assert_eq!(frame_index, Some(3));
            }
            _ => panic!("expected image sprite"),
        }
    }

    #[test]
    fn rejects_unsupported_size_preset() {
        let raw = r#"
type: image
source: "/assets/images/tux.png"
size: 5
"#;
        let error = serde_yaml::from_str::<Sprite>(raw).expect_err("size should be rejected");
        assert!(
            error.to_string().contains("unsupported sprite size preset"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn parses_grid_sprite_with_children() {
        let raw = r#"
type: grid
columns: ["1fr","1fr","1fr"]
rows: ["auto","1fr"]
children:
  - type: text
    content: "A"
    grid-col: 1
    grid-row: 1
  - type: image
    source: "/assets/images/tux.png"
    grid-col: 2
    grid-row: 2
"#;
        let sprite: Sprite = serde_yaml::from_str(raw).expect("grid sprite should parse");
        match sprite {
            Sprite::Grid {
                columns,
                rows,
                children,
                ..
            } => {
                assert_eq!(columns.len(), 3);
                assert_eq!(rows.len(), 2);
                assert_eq!(children.len(), 2);
            }
            Sprite::Text { .. }
            | Sprite::Image { .. }
            | Sprite::Obj { .. }
            | Sprite::Planet { .. }
            | Sprite::Vector { .. }
            | Sprite::Panel { .. }
            | Sprite::Flex { .. }
            | Sprite::Scene3D { .. } => {
                panic!("expected grid sprite")
            }
        }
    }

    #[test]
    fn parses_sprite_behaviors() {
        let raw = r#"
type: text
id: title
content: "TEST"
behaviors:
  - name: blink
    params:
      visible_ms: 100
      hidden_ms: 150
  - name: bob
    params:
      amplitude_y: 2
      period_ms: 1000
"#;
        let sprite: Sprite = serde_yaml::from_str(raw).expect("sprite should parse");
        match sprite {
            Sprite::Text { behaviors, .. } => {
                assert_eq!(behaviors.len(), 2);
                assert_eq!(behaviors[0].name, "blink");
                assert_eq!(behaviors[1].params.amplitude_y, Some(2));
            }
            Sprite::Image { .. }
            | Sprite::Obj { .. }
            | Sprite::Planet { .. }
            | Sprite::Vector { .. }
            | Sprite::Panel { .. }
            | Sprite::Grid { .. }
            | Sprite::Flex { .. }
            | Sprite::Scene3D { .. } => {
                panic!("expected text sprite")
            }
        }
    }
}
