use crate::services::EngineWorldAccess;
use crate::world::World;

pub fn free_look_camera_system(world: &mut World, dt_ms: u64) {
    if let Some(runtime) = world.scene_runtime_mut() {
        let _ = runtime.step_free_look_camera(dt_ms);
    }
}
