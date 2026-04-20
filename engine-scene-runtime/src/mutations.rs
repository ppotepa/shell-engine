use engine_core::render_types::{Camera3DState, Light3D, MaterialValue, Transform3D};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, PartialEq)]
pub enum ObjMaterialParam {
    Source,
    Scale,
    Yaw,
    Pitch,
    Roll,
    OrbitSpeed,
    RotationSpeed,
    Ambient,
    CameraDistance,
    SurfaceMode,
    ClipYMin,
    ClipYMax,
    LightDirectionX,
    LightDirectionY,
    LightDirectionZ,
    WorldX,
    WorldY,
    WorldZ,
    CamWorldX,
    CamWorldY,
    CamWorldZ,
    ViewRightX,
    ViewRightY,
    ViewRightZ,
    ViewUpX,
    ViewUpY,
    ViewUpZ,
    ViewFwdX,
    ViewFwdY,
    ViewFwdZ,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AtmosphereParam {
    Color,
    Height,
    Density,
    Strength,
    RayleighAmount,
    RayleighColor,
    RayleighFalloff,
    HazeAmount,
    HazeColor,
    HazeFalloff,
    AbsorptionAmount,
    AbsorptionColor,
    AbsorptionHeight,
    AbsorptionWidth,
    ForwardScatter,
    LimbBoost,
    TerminatorSoftness,
    NightGlow,
    NightGlowColor,
    RimPower,
    HazeStrength,
    HazePower,
    VeilStrength,
    VeilPower,
    HaloStrength,
    HaloWidth,
    HaloPower,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TerrainParam {
    Amplitude,
    Frequency,
    Roughness,
    Octaves,
    SeedX,
    SeedZ,
    Lacunarity,
    Ridge,
    Plateau,
    SeaLevel,
    ScaleX,
    ScaleZ,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorldgenParam {
    Seed,
    HasOcean,
    OceanFraction,
    ContinentScale,
    ContinentWarp,
    ContinentOctaves,
    MountainScale,
    MountainStrength,
    MountainRidgeOctaves,
    MoistureScale,
    IceCapStrength,
    LapseRate,
    RainShadow,
    Subdivisions,
    DisplacementScale,
    Coloring,
    Base,
    Shape,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlanetParam {
    SpinDeg,
    CloudSpinDeg,
    Cloud2SpinDeg,
    ObserverAltitudeKm,
    SunDirX,
    SunDirY,
    SunDirZ,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LightingProfileParam {
    AmbientIntensity,
    KeyLightIntensity,
    FillLightIntensity,
    RimLightIntensity,
    BlackLevel,
    ShadowContrast,
    Exposure,
    Tonemap,
    Gamma,
    NightGlowScale,
    HazeNightLeak,
    SpecularFloor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Render3DProfileSlot {
    View,
    Lighting,
    SpaceEnvironment,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Render3DProfileParam {
    Lighting(LightingProfileParam),
    SpaceEnvironment(SpaceEnvironmentParam),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpaceEnvironmentParam {
    BackgroundColor,
    BackgroundFloor,
    StarfieldDensity,
    StarfieldBrightness,
    StarfieldSizeMin,
    StarfieldSizeMax,
    PrimaryStarColor,
    PrimaryStarGlareStrength,
    PrimaryStarGlareWidth,
    NebulaStrength,
    DustBandStrength,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Render3DGroupedParam {
    Material(ObjMaterialParam),
    Atmosphere(AtmosphereParam),
    Surface(TerrainParam),
    Generator(WorldgenParam),
    Body(PlanetParam),
    View(ViewParam),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewParam {
    Distance,
    Yaw,
    Pitch,
    Roll,
}

impl ObjMaterialParam {
    pub(crate) fn from_full_path(path: &str) -> Option<Self> {
        match path {
            "obj.source" => Some(Self::Source),
            "obj.scale" => Some(Self::Scale),
            "obj.yaw" => Some(Self::Yaw),
            "obj.pitch" => Some(Self::Pitch),
            "obj.roll" => Some(Self::Roll),
            "obj.orbit_speed" => Some(Self::OrbitSpeed),
            "obj.rotation-speed" | "obj.rotation_speed" => Some(Self::RotationSpeed),
            "obj.ambient" => Some(Self::Ambient),
            "obj.camera-distance" | "obj.camera_distance" => Some(Self::CameraDistance),
            "obj.surface_mode" | "obj.surface-mode" => Some(Self::SurfaceMode),
            "obj.clip_y_min" | "obj.clip-y-min" => Some(Self::ClipYMin),
            "obj.clip_y_max" | "obj.clip-y-max" => Some(Self::ClipYMax),
            "obj.light.x" => Some(Self::LightDirectionX),
            "obj.light.y" => Some(Self::LightDirectionY),
            "obj.light.z" => Some(Self::LightDirectionZ),
            "obj.world.x" => Some(Self::WorldX),
            "obj.world.y" => Some(Self::WorldY),
            "obj.world.z" => Some(Self::WorldZ),
            "obj.cam.wx" | "obj.cam.world.x" | "obj.cam.world_x" => Some(Self::CamWorldX),
            "obj.cam.wy" | "obj.cam.world.y" | "obj.cam.world_y" => Some(Self::CamWorldY),
            "obj.cam.wz" | "obj.cam.world.z" | "obj.cam.world_z" => Some(Self::CamWorldZ),
            "obj.view.rx" | "obj.view.right.x" | "obj.view.right_x" => Some(Self::ViewRightX),
            "obj.view.ry" | "obj.view.right.y" | "obj.view.right_y" => Some(Self::ViewRightY),
            "obj.view.rz" | "obj.view.right.z" | "obj.view.right_z" => Some(Self::ViewRightZ),
            "obj.view.ux" | "obj.view.up.x" | "obj.view.up_x" => Some(Self::ViewUpX),
            "obj.view.uy" | "obj.view.up.y" | "obj.view.up_y" => Some(Self::ViewUpY),
            "obj.view.uz" | "obj.view.up.z" | "obj.view.up_z" => Some(Self::ViewUpZ),
            "obj.view.fx" | "obj.view.fwd.x" | "obj.view.fwd_x" => Some(Self::ViewFwdX),
            "obj.view.fy" | "obj.view.fwd.y" | "obj.view.fwd_y" => Some(Self::ViewFwdY),
            "obj.view.fz" | "obj.view.fwd.z" | "obj.view.fwd_z" => Some(Self::ViewFwdZ),
            _ => None,
        }
    }
}

impl AtmosphereParam {
    pub(crate) fn from_full_path(path: &str) -> Option<Self> {
        let bare = path.strip_prefix("obj.atmo.").unwrap_or(path);
        match bare {
            "color" => Some(Self::Color),
            "height" => Some(Self::Height),
            "density" => Some(Self::Density),
            "strength" => Some(Self::Strength),
            "rayleigh_amount" => Some(Self::RayleighAmount),
            "rayleigh_color" => Some(Self::RayleighColor),
            "rayleigh_falloff" => Some(Self::RayleighFalloff),
            "haze_amount" => Some(Self::HazeAmount),
            "haze_color" => Some(Self::HazeColor),
            "haze_falloff" => Some(Self::HazeFalloff),
            "absorption_amount" => Some(Self::AbsorptionAmount),
            "absorption_color" => Some(Self::AbsorptionColor),
            "absorption_height" => Some(Self::AbsorptionHeight),
            "absorption_width" => Some(Self::AbsorptionWidth),
            "forward_scatter" => Some(Self::ForwardScatter),
            "limb_boost" => Some(Self::LimbBoost),
            "terminator_softness" => Some(Self::TerminatorSoftness),
            "night_glow" => Some(Self::NightGlow),
            "night_glow_color" => Some(Self::NightGlowColor),
            "rim_power" => Some(Self::RimPower),
            "haze_strength" => Some(Self::HazeStrength),
            "haze_power" => Some(Self::HazePower),
            "veil_strength" => Some(Self::VeilStrength),
            "veil_power" => Some(Self::VeilPower),
            "halo_strength" => Some(Self::HaloStrength),
            "halo_width" => Some(Self::HaloWidth),
            "halo_power" => Some(Self::HaloPower),
            _ => None,
        }
    }
}

impl TerrainParam {
    pub(crate) fn from_full_path(path: &str) -> Option<Self> {
        let bare = path.strip_prefix("terrain.").unwrap_or(path);
        match bare {
            "amplitude" => Some(Self::Amplitude),
            "frequency" => Some(Self::Frequency),
            "roughness" => Some(Self::Roughness),
            "octaves" => Some(Self::Octaves),
            "seed_x" => Some(Self::SeedX),
            "seed_z" => Some(Self::SeedZ),
            "lacunarity" => Some(Self::Lacunarity),
            "ridge" => Some(Self::Ridge),
            "plateau" => Some(Self::Plateau),
            "sea_level" => Some(Self::SeaLevel),
            "scale_x" => Some(Self::ScaleX),
            "scale_z" => Some(Self::ScaleZ),
            _ => None,
        }
    }
}

impl WorldgenParam {
    pub(crate) fn from_full_path(path: &str) -> Option<Self> {
        let bare = path.strip_prefix("world.").unwrap_or(path);
        match bare {
            "seed" => Some(Self::Seed),
            "has_ocean" | "has-ocean" => Some(Self::HasOcean),
            "ocean_fraction" => Some(Self::OceanFraction),
            "continent_scale" => Some(Self::ContinentScale),
            "continent_warp" => Some(Self::ContinentWarp),
            "continent_octaves" => Some(Self::ContinentOctaves),
            "mountain_scale" => Some(Self::MountainScale),
            "mountain_strength" => Some(Self::MountainStrength),
            "mountain_ridge_octaves" => Some(Self::MountainRidgeOctaves),
            "moisture_scale" => Some(Self::MoistureScale),
            "ice_cap_strength" => Some(Self::IceCapStrength),
            "lapse_rate" => Some(Self::LapseRate),
            "rain_shadow" => Some(Self::RainShadow),
            "subdivisions" => Some(Self::Subdivisions),
            "displacement_scale" => Some(Self::DisplacementScale),
            "coloring" => Some(Self::Coloring),
            "base" => Some(Self::Base),
            "shape" => Some(Self::Shape),
            _ => None,
        }
    }
}

impl PlanetParam {
    pub(crate) fn from_full_path(path: &str) -> Option<Self> {
        let bare = path.strip_prefix("planet.").unwrap_or(path);
        match bare {
            "spin_deg" => Some(Self::SpinDeg),
            "cloud_spin_deg" => Some(Self::CloudSpinDeg),
            "cloud2_spin_deg" => Some(Self::Cloud2SpinDeg),
            "observer_altitude_km" => Some(Self::ObserverAltitudeKm),
            "sun_dir.x" | "sun_dir_x" => Some(Self::SunDirX),
            "sun_dir.y" | "sun_dir_y" => Some(Self::SunDirY),
            "sun_dir.z" | "sun_dir_z" => Some(Self::SunDirZ),
            _ => None,
        }
    }
}

impl LightingProfileParam {
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "ambient_intensity" | "ambient-intensity" => Some(Self::AmbientIntensity),
            "key_light_intensity" | "key-light-intensity" => Some(Self::KeyLightIntensity),
            "fill_light_intensity" | "fill-light-intensity" => Some(Self::FillLightIntensity),
            "rim_light_intensity" | "rim-light-intensity" => Some(Self::RimLightIntensity),
            "black_level" | "black-level" => Some(Self::BlackLevel),
            "shadow_contrast" | "shadow-contrast" => Some(Self::ShadowContrast),
            "exposure" => Some(Self::Exposure),
            "tonemap" => Some(Self::Tonemap),
            "gamma" => Some(Self::Gamma),
            "night_glow_scale" | "night-glow-scale" => Some(Self::NightGlowScale),
            "haze_night_leak" | "haze-night-leak" => Some(Self::HazeNightLeak),
            "specular_floor" | "specular-floor" => Some(Self::SpecularFloor),
            _ => None,
        }
    }
}

impl SpaceEnvironmentParam {
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "background_color" | "background-color" => Some(Self::BackgroundColor),
            "background_floor" | "background-floor" => Some(Self::BackgroundFloor),
            "starfield_density" | "starfield-density" => Some(Self::StarfieldDensity),
            "starfield_brightness" | "starfield-brightness" => Some(Self::StarfieldBrightness),
            "starfield_size_min" | "starfield-size-min" => Some(Self::StarfieldSizeMin),
            "starfield_size_max" | "starfield-size-max" => Some(Self::StarfieldSizeMax),
            "primary_star_color" | "primary-star-color" => Some(Self::PrimaryStarColor),
            "primary_star_glare_strength" | "primary-star-glare-strength" => {
                Some(Self::PrimaryStarGlareStrength)
            }
            "primary_star_glare_width" | "primary-star-glare-width" => {
                Some(Self::PrimaryStarGlareWidth)
            }
            "nebula_strength" | "nebula-strength" => Some(Self::NebulaStrength),
            "dust_band_strength" | "dust-band-strength" => Some(Self::DustBandStrength),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Set2DPropsMutation {
    pub target: String,
    pub visible: Option<bool>,
    pub dx: Option<i32>,
    pub dy: Option<i32>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SetCamera2DMutation {
    pub x: i32,
    pub y: i32,
    pub zoom: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Render3DMutation {
    SetGroupedParams {
        target: Option<String>,
        params: Vec<(Render3DGroupedParam, MaterialValue)>,
    },
    SetProfile {
        slot: Render3DProfileSlot,
        profile: String,
    },
    SetViewProfile {
        profile: String,
    },
    SetLightingProfile {
        profile: String,
    },
    SetSpaceEnvironmentProfile {
        profile: String,
    },
    SetProfileParam {
        param: Render3DProfileParam,
        value: MaterialValue,
    },
    SetLightingParam {
        param: LightingProfileParam,
        value: MaterialValue,
    },
    SetSpaceEnvironmentParam {
        param: SpaceEnvironmentParam,
        value: MaterialValue,
    },
    SetNodeTransform {
        target: String,
        transform: Transform3D,
    },
    SetNodeVisibility {
        target: String,
        visible: bool,
    },
    SetObjMaterialParam {
        target: String,
        param: ObjMaterialParam,
        value: MaterialValue,
    },
    SetAtmosphereParamTyped {
        target: String,
        param: AtmosphereParam,
        value: MaterialValue,
    },
    SetTerrainParamTyped {
        target: String,
        param: TerrainParam,
        value: MaterialValue,
    },
    SetWorldgenParamTyped {
        target: String,
        param: WorldgenParam,
        value: MaterialValue,
    },
    SetPlanetParamTyped {
        target: String,
        param: PlanetParam,
        value: MaterialValue,
    },
    SetScene3DFrame {
        target: String,
        frame: String,
    },
    SetSceneCamera {
        camera: Camera3DState,
    },
    SetLight {
        index: usize,
        light: Light3D,
    },
    RebuildMesh {
        target: String,
    },
    RebuildWorldgen {
        target: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum SceneMutation {
    Set2DProps(Set2DPropsMutation),
    SetSpriteProperty {
        target: String,
        mutation: SetSpritePropertyMutation,
    },
    SetCamera2D(SetCamera2DMutation),
    SetCamera3D(Camera3DState),
    SetRender3D(Render3DMutation),
    SpawnObject {
        template: String,
        target: String,
    },
    DespawnObject {
        target: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum SetSpritePropertyMutation {
    Heading { heading: f32 },
    TextFont { font: String },
    TextColour { fg: bool, value: JsonValue },
    VectorProperty { path: String, value: JsonValue },
    ImageFrame { frame_index: u16 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_mutation_wraps_render3d_mutation() {
        let mutation = SceneMutation::SetRender3D(Render3DMutation::SetNodeVisibility {
            target: "planet".to_string(),
            visible: true,
        });

        match mutation {
            SceneMutation::SetRender3D(Render3DMutation::SetNodeVisibility { target, visible }) => {
                assert_eq!(target, "planet");
                assert!(visible);
            }
            _ => panic!("unexpected mutation shape"),
        }
    }

    #[test]
    fn typed_obj_material_param_mutation() {
        let mutation = Render3DMutation::SetGroupedParams {
            target: Some("ship".to_string()),
            params: vec![(
                Render3DGroupedParam::Material(ObjMaterialParam::Yaw),
                MaterialValue::Scalar(45.0),
            )],
        };

        match mutation {
            Render3DMutation::SetGroupedParams { target, params } => {
                assert_eq!(target.as_deref(), Some("ship"));
                assert_eq!(
                    params,
                    vec![(
                        Render3DGroupedParam::Material(ObjMaterialParam::Yaw),
                        MaterialValue::Scalar(45.0),
                    )]
                );
            }
            _ => panic!("unexpected mutation shape"),
        }
    }

    #[test]
    fn obj_material_param_recognizes_object_camera_aliases() {
        assert_eq!(
            ObjMaterialParam::from_full_path("obj.cam.wx"),
            Some(ObjMaterialParam::CamWorldX)
        );
        assert_eq!(
            ObjMaterialParam::from_full_path("obj.cam.world.x"),
            Some(ObjMaterialParam::CamWorldX)
        );
        assert_eq!(
            ObjMaterialParam::from_full_path("obj.cam.world_y"),
            Some(ObjMaterialParam::CamWorldY)
        );
        assert_eq!(
            ObjMaterialParam::from_full_path("obj.view.rx"),
            Some(ObjMaterialParam::ViewRightX)
        );
        assert_eq!(
            ObjMaterialParam::from_full_path("obj.view.right_y"),
            Some(ObjMaterialParam::ViewRightY)
        );
        assert_eq!(
            ObjMaterialParam::from_full_path("obj.view.up_z"),
            Some(ObjMaterialParam::ViewUpZ)
        );
        assert_eq!(
            ObjMaterialParam::from_full_path("obj.view.up.z"),
            Some(ObjMaterialParam::ViewUpZ)
        );
        assert_eq!(
            ObjMaterialParam::from_full_path("obj.view.fwd.z"),
            Some(ObjMaterialParam::ViewFwdZ)
        );
        assert_eq!(
            ObjMaterialParam::from_full_path("obj.view.fwd_x"),
            Some(ObjMaterialParam::ViewFwdX)
        );
    }

    #[test]
    fn typed_atmosphere_param_mutation() {
        let mutation = Render3DMutation::SetGroupedParams {
            target: Some("planet".to_string()),
            params: vec![(
                Render3DGroupedParam::Atmosphere(AtmosphereParam::Height),
                MaterialValue::Scalar(0.15),
            )],
        };

        match mutation {
            Render3DMutation::SetGroupedParams { target, params } => {
                assert_eq!(target.as_deref(), Some("planet"));
                assert_eq!(
                    params,
                    vec![(
                        Render3DGroupedParam::Atmosphere(AtmosphereParam::Height),
                        MaterialValue::Scalar(0.15),
                    )]
                );
            }
            _ => panic!("unexpected mutation shape"),
        }
    }

    #[test]
    fn typed_lighting_profile_param_mutation() {
        let mutation = Render3DMutation::SetLightingParam {
            param: LightingProfileParam::Exposure,
            value: MaterialValue::Scalar(0.9),
        };

        match mutation {
            Render3DMutation::SetLightingParam { param, value } => {
                assert_eq!(param, LightingProfileParam::Exposure);
                assert_eq!(value, MaterialValue::Scalar(0.9));
            }
            _ => panic!("unexpected mutation shape"),
        }
    }

    #[test]
    fn neutral_profile_mutation_keeps_slot_and_profile() {
        let mutation = Render3DMutation::SetProfile {
            slot: Render3DProfileSlot::Lighting,
            profile: "lab-neutral".to_string(),
        };

        match mutation {
            Render3DMutation::SetProfile { slot, profile } => {
                assert_eq!(slot, Render3DProfileSlot::Lighting);
                assert_eq!(profile, "lab-neutral");
            }
            _ => panic!("unexpected mutation shape"),
        }
    }

    #[test]
    fn neutral_profile_param_mutation_keeps_typed_payload() {
        let mutation = Render3DMutation::SetGroupedParams {
            target: None,
            params: vec![(
                Render3DGroupedParam::View(ViewParam::Distance),
                MaterialValue::Text("#010203".to_string()),
            )],
        };

        match mutation {
            Render3DMutation::SetGroupedParams { target, params } => {
                assert_eq!(target, None);
                assert_eq!(
                    params,
                    vec![(
                        Render3DGroupedParam::View(ViewParam::Distance),
                        MaterialValue::Text("#010203".to_string()),
                    )]
                );
            }
            _ => panic!("unexpected mutation shape"),
        }
    }

    #[test]
    fn lighting_param_parses_supported_names() {
        assert_eq!(
            LightingProfileParam::from_name("shadow_contrast"),
            Some(LightingProfileParam::ShadowContrast)
        );
        assert_eq!(
            SpaceEnvironmentParam::from_name("primary-star-glare-width"),
            Some(SpaceEnvironmentParam::PrimaryStarGlareWidth)
        );
    }
}
