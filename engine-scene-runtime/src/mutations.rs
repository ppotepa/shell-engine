use engine_core::render_types::{Camera3DState, Light3D, MaterialValue, Transform3D};

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
    PlanetParam { path: String, value: MaterialValue },
    ObjParam { path: String, value: MaterialValue },
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
    SetMaterialParam {
        target: String,
        param: String,
        value: MaterialValue,
    },
    SetAtmosphereParam {
        target: String,
        param: String,
        value: MaterialValue,
    },
    SetWorldgenParam {
        target: String,
        param: String,
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
    SetCamera2D(SetCamera2DMutation),
    SetCamera3D(Camera3DState),
    SetRender3D(Render3DMutation),
    SpawnObject { template: String, target: String },
    DespawnObject { target: String },
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
}
