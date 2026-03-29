use engine_game::CollisionHit;

/// Simple event buffer to make collisions visible to behaviors without direct coupling.
#[derive(Default)]
pub struct GameplayEventBuffer {
    pub collisions: Vec<CollisionHit>,
}

pub fn push_collisions(
    world: &mut engine_core::world::World,
    hits: Vec<CollisionHit>,
) {
    if hits.is_empty() {
        return;
    }
    if let Some(buf) = world.get_mut::<GameplayEventBuffer>() {
        buf.collisions.extend(hits);
    } else {
        let mut buf = GameplayEventBuffer::default();
        buf.collisions = hits;
        world.register(buf);
    }
}

/// Clears the buffer after behaviors have consumed it.
pub fn clear(world: &mut engine_core::world::World) {
    if let Some(buf) = world.get_mut::<GameplayEventBuffer>() {
        buf.collisions.clear();
    }
}
