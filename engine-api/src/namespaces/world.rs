//! world.* namespace for Rhai scripting.
//!
//! Provides world-level operations: spawning, querying, emitting effects, managing bounds, etc.
//! Accessible as `world.spawn_visual()`, `world.emit()`, etc.
//!
//! All methods are also registered as flat names for backward compatibility:
//! - `world.spawn_visual(...)` works AND `spawn_visual(...)` works

use rhai::Engine as RhaiEngine;
use crate::gameplay::api::ScriptWorldContext;
use std::sync::{Arc, Mutex};

/// World namespace API for Rhai scripting.
/// Wraps ScriptWorldContext and provides all world-level operations.
#[derive(Clone)]
pub struct WorldApi {
    context: Arc<Mutex<ScriptWorldContext>>,
}

impl WorldApi {
    /// Create a new WorldApi wrapper.
    pub(crate) fn new(context: ScriptWorldContext) -> Self {
        Self {
            context: Arc::new(Mutex::new(context)),
        }
    }

    /// Expose the inner context for use by other namespaces.
    pub(crate) fn context(&self) -> Arc<Mutex<ScriptWorldContext>> {
        Arc::clone(&self.context)
    }
}

/// World namespace marker for Rhai registration.
/// This struct is not used at runtime but serves as a type anchor for namespace methods.
#[derive(Clone)]
pub struct WorldNamespace;

/// Register the `world.*` namespace and backward-compatible flat names.
///
/// # Pattern for Implementing Dual Names
///
/// To add world.* namespace support while maintaining backward compatibility:
///
/// 1. **For each world method**, register it twice:
///    ```ignore
///    // Namespaced version (new API)
///    engine.register_fn("world.spawn_visual",
///        |world: &mut ScriptGameplayApi, kind: &str, template: &str, data: RhaiMap| {
///            world.spawn_visual(kind, template, data)
///        }
///    );
///
///    // Flat version (backward compatible)
///    engine.register_fn("spawn_visual",
///        |world: &mut ScriptGameplayApi, kind: &str, template: &str, data: RhaiMap| {
///            world.spawn_visual(kind, template, data)
///        }
///    );
///    ```
///
/// 2. **Organize by domain** in engine-behavior/src/scripting/gameplay.rs:
///    - World operations: spawn_visual, spawn_prefab, emit, query, count, bounds
///    - Entity operations: entity, exists, kind, tags, ids (will be entity.* in Phase 3c)
///    - Transform/Physics/Tags: handled by entity wrapper (will be entity.* in Phase 3c)
///    - Collision: collision_enters_between, etc. (will be collision.* in Phase 3c)
///
/// 3. **Test both interfaces**:
///    - Old: `spawn_visual(kind, template, data)` must work
///    - New: `world.spawn_visual(kind, template, data)` must work
///    - Both in same script should work
///
/// # Current Implementation Strategy
///
/// Rather than duplicate registration code in engine-api, the implementation follows:
///
/// 1. **Single source of truth**: engine-behavior/src/scripting/gameplay.rs
///    - All registration logic lives here for now
///    - Maintains both flat and namespaced names
///    - Reduces risk of divergence
///
/// 2. **Pattern established**: WorldApi and WorldNamespace types exist
///    - Ready for future crate-level registration
///    - Type structure supports domain organization
///
/// 3. **Incremental migration**: This allows gradual adoption
///    - Old scripts work unchanged (flat names)
///    - New scripts can use world.* immediately
///    - Future: Remove flat names if desired (backward compat break)
///
pub fn register_world_namespace(_engine: &mut RhaiEngine) {
    // NOTE: This function is intentionally minimal.
    // 
    // The actual world.* registration happens in:
    // engine-behavior/src/scripting/gameplay.rs:register_with_rhai()
    //
    // Future phases can move registration here as engine-behavior is refactored.
    // For now, keeping it in gameplay.rs maintains the single source of truth.
}


