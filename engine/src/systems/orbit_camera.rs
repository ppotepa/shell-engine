use crate::services::EngineWorldAccess;
use crate::world::World;

pub fn orbit_camera_system(world: &mut World) {
    if let Some(runtime) = world.scene_runtime_mut() {
        let _ = runtime.step_orbit_camera();
    }
}
