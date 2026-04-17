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

impl ObjMaterialParam {
    pub(crate) fn from_full_path(path: &str) -> Option<Self> {
        match path {
            "obj.source" => Some(Self::Source),
            "obj.scale" => Some(Self::Scale),
            "obj.yaw" => Some(Self::Yaw),
            "obj.pitch" => Some(Self::Pitch),
            "obj.roll" => Some(Self::Roll),
            "obj.orbit_speed" => Some(Self::OrbitSpeed),
            "obj.rotation-speed" => Some(Self::RotationSpeed),
            "obj.ambient" => Some(Self::Ambient),
            "obj.camera-distance" => Some(Self::CameraDistance),
            "obj.surface_mode" => Some(Self::SurfaceMode),
            "obj.clip_y_min" => Some(Self::ClipYMin),
            "obj.clip_y_max" => Some(Self::ClipYMax),
            "obj.light.x" => Some(Self::LightDirectionX),
            "obj.light.y" => Some(Self::LightDirectionY),
            "obj.light.z" => Some(Self::LightDirectionZ),
            "obj.world.x" => Some(Self::WorldX),
            "obj.world.y" => Some(Self::WorldY),
            "obj.world.z" => Some(Self::WorldZ),
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
pub enum Render3DCompatProperty {
    Scene3dFrame { frame: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Render3DMutation {
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
    SetCompatProperty {
        target: String,
        property: Render3DCompatProperty,
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
    SpawnObject { template: String, target: String },
    DespawnObject { target: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum SetSpritePropertyMutation {
    Heading {
        heading: f32,
    },
    TextFont {
        font: String,
    },
    TextColour {
        fg: bool,
        value: JsonValue,
    },
    VectorProperty {
        path: String,
        value: JsonValue,
    },
    ImageFrame {
        frame_index: u16,
    },
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
        let mutation = Render3DMutation::SetObjMaterialParam {
            target: "ship".to_string(),
            param: ObjMaterialParam::Yaw,
            value: MaterialValue::Scalar(45.0),
        };

        match mutation {
            Render3DMutation::SetObjMaterialParam {
                target,
                param,
                value,
            } => {
                assert_eq!(target, "ship");
                assert_eq!(param, ObjMaterialParam::Yaw);
                assert_eq!(value, MaterialValue::Scalar(45.0));
            }
            _ => panic!("unexpected mutation shape"),
        }
    }

    #[test]
    fn typed_atmosphere_param_mutation() {
        let mutation = Render3DMutation::SetAtmosphereParamTyped {
            target: "planet".to_string(),
            param: AtmosphereParam::Height,
            value: MaterialValue::Scalar(0.15),
        };

        match mutation {
            Render3DMutation::SetAtmosphereParamTyped {
                target,
                param,
                value,
            } => {
                assert_eq!(target, "planet");
                assert_eq!(param, AtmosphereParam::Height);
                assert_eq!(value, MaterialValue::Scalar(0.15));
            }
            _ => panic!("unexpected mutation shape"),
        }
    }
}
