// Asteroids shared helpers
// This module is imported by asteroids-game-loop.rhai and asteroids-render-sync.rhai

// TODO: Extract from game-loop.rhai:
// - wrap_heading32(heading)
// - fragment_heading_offset(fragment_idx)
// - asteroid_radius(size)
// - asteroid_score(size)

// TODO: Extract from render-sync.rhai:
// - sync_transform(entity_ref, scene, sprite_id)
// - sync_physics(entity_ref, scene, sprite_id)

// Example helper function to verify module loading works
pub fn test_module_loading() {
    print("Asteroids shared module loaded successfully!");
}

// Placeholder for now - will be filled in during script migration (phase 5)
