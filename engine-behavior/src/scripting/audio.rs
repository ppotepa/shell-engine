//! Audio domain API: ScriptAudioApi for audio cues and songs, ScriptFxApi for effects.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use rhai::{Engine as RhaiEngine, Map as RhaiMap};
use serde_json::Value as JsonValue;

use crate::{catalog, geometry, BehaviorCommand};
use engine_game::{GameplayWorld, LifecyclePolicy};

use super::ephemeral::{spawn_ephemeral_visual, EphemeralSpawn};
use crate::EmitterState;

// ── ScriptAudioApi ───────────────────────────────────────────────────────

#[derive(Clone)]
pub(crate) struct ScriptAudioApi {
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

impl ScriptAudioApi {
    pub(crate) fn new(queue: Arc<Mutex<Vec<BehaviorCommand>>>) -> Self {
        Self { queue }
    }

    fn cue(&mut self, cue: &str, volume: Option<f32>) -> bool {
        let cue = cue.trim();
        if cue.is_empty() {
            return false;
        }
        let Ok(mut queue) = self.queue.lock() else {
            return false;
        };
        queue.push(BehaviorCommand::PlayAudioCue {
            cue: cue.to_string(),
            volume,
        });
        true
    }

    fn event(&mut self, event: &str, gain: Option<f32>) -> bool {
        let event = event.trim();
        if event.is_empty() {
            return false;
        }
        let Ok(mut queue) = self.queue.lock() else {
            return false;
        };
        queue.push(BehaviorCommand::PlayAudioEvent {
            event: event.to_string(),
            gain,
        });
        true
    }

    fn play_song(&mut self, song_id: &str) -> bool {
        let song_id = song_id.trim();
        if song_id.is_empty() {
            return false;
        }
        let Ok(mut queue) = self.queue.lock() else {
            return false;
        };
        queue.push(BehaviorCommand::PlaySong {
            song_id: song_id.to_string(),
        });
        true
    }

    fn stop_song(&mut self) -> bool {
        let Ok(mut queue) = self.queue.lock() else {
            return false;
        };
        queue.push(BehaviorCommand::StopSong);
        true
    }
}

// ── ScriptFxApi ──────────────────────────────────────────────────────────

#[derive(Clone)]
pub(crate) struct ScriptFxApi {
    world: Option<GameplayWorld>,
    emitter_state: Option<EmitterState>,
    catalogs: Arc<catalog::ModCatalogs>,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

impl ScriptFxApi {
    pub(crate) fn new(
        world: Option<GameplayWorld>,
        emitter_state: Option<EmitterState>,
        catalogs: Arc<catalog::ModCatalogs>,
        queue: Arc<Mutex<Vec<BehaviorCommand>>>,
    ) -> Self {
        Self {
            world,
            emitter_state,
            catalogs,
            queue,
        }
    }

    pub(crate) fn emit(&mut self, effect_name: &str, args: RhaiMap) -> rhai::Array {
        let effect_name = effect_name.trim();
        if effect_name.is_empty() {
            return rhai::Array::new();
        }

        let Some(world) = self.world.clone() else {
            return rhai::Array::new();
        };

        // Clone config out to avoid borrow conflict
        let catalog_config = self.catalogs.emitters.get(effect_name).cloned();
        let is_disintegration = effect_name.ends_with("disintegration");

        if is_disintegration {
            self.emit_disintegration(effect_name, &world, &args, catalog_config.as_ref())
        } else {
            self.emit_thrust_smoke(effect_name, &world, &args, catalog_config.as_ref())
        }
    }

    fn emit_thrust_smoke(
        &mut self,
        effect_name: &str,
        world: &GameplayWorld,
        args: &RhaiMap,
        config: Option<&catalog::EmitterConfig>,
    ) -> rhai::Array {
        let ship_id = args
            .get("ship_id")
            .and_then(|v| v.clone().try_cast::<rhai::INT>())
            .unwrap_or(0) as u64;

        if ship_id == 0 || !world.exists(ship_id) {
            return rhai::Array::new();
        }

        // Throttle via cooldown — dynamic rate based on thrust duration
        let cooldown_name = config
            .and_then(|c| c.cooldown_name.as_deref())
            .unwrap_or("smoke");
        if !world.cooldown_ready(ship_id, cooldown_name) {
            return rhai::Array::new();
        }
        let base_cooldown = config.and_then(|c| c.cooldown_ms).unwrap_or(48) as f32;
        let min_cooldown = config.and_then(|c| c.min_cooldown_ms).unwrap_or(base_cooldown as i64) as f32;
        let ramp_ms = config.and_then(|c| c.ramp_ms).unwrap_or(2000) as f32;

        // Compute effective cooldown: lerp from base → min over ramp_ms of sustained thrust
        let thrust_ms = args
            .get("thrust_ms")
            .and_then(|v| v.clone().try_cast::<rhai::INT>())
            .unwrap_or(0) as f32;
        let ramp_t = if ramp_ms > 0.0 {
            (thrust_ms / ramp_ms).min(1.0)
        } else {
            1.0
        };
        let effective_cooldown = (base_cooldown + (min_cooldown - base_cooldown) * ramp_t) as i32;
        world.cooldown_start(ship_id, cooldown_name, effective_cooldown.max(1));

        let max_count = config.and_then(|c| c.max_count).unwrap_or(10) as usize;
        if let Some(state) = &self.emitter_state {
            while state.active_count(effect_name, Some(ship_id)) >= max_count {
                let Some(oldest_id) = state.evict_oldest(effect_name, Some(ship_id)) else {
                    break;
                };
                if let Some(binding) = world.visual(oldest_id) {
                    for target in binding.all_visual_ids() {
                        if let Ok(mut commands) = self.queue.lock() {
                            commands.push(BehaviorCommand::SceneDespawn {
                                target: target.to_string(),
                            });
                        }
                    }
                }
                let _ = world.despawn(oldest_id);
                state.remove_entity(oldest_id);
            }
        } else if world.count_kind("smoke") as i64 >= max_count as i64 {
            return rhai::Array::new();
        }

        let Some(transform) = world.transform(ship_id) else {
            return rhai::Array::new();
        };
        let Some(controller) = world.controller(ship_id) else {
            return rhai::Array::new();
        };

        let spawn_offset = config.and_then(|c| c.spawn_offset).unwrap_or(6.0) as f32;
        let heading = controller.current_heading;
        let (dir_x, dir_y) = geometry::heading_vector_i32(heading);
        let spawn_x = transform.x - (dir_x * spawn_offset);
        let spawn_y = transform.y - (dir_y * spawn_offset);

        let physics = world.physics(ship_id);
        let ship_vx = physics.map(|p| p.vx).unwrap_or(0.0);
        let ship_vy = physics.map(|p| p.vy).unwrap_or(0.0);
        let backward_speed = config.and_then(|c| c.backward_speed).unwrap_or(0.35) as f32;
        let velocity_scale = config.and_then(|c| c.velocity_scale).unwrap_or(60.0) as f32;

        let vx = ship_vx - (dir_x * backward_speed * velocity_scale);
        let vy = ship_vy - (dir_y * backward_speed * velocity_scale);
        let ttl_ms = config.and_then(|c| c.ttl_ms).unwrap_or(520) as i32;
        let radius = config.and_then(|c| c.radius).unwrap_or(3) as i64;

        if let Some(id) = self.spawn_smoke_entity(
            world,
            Some(ship_id),
            spawn_x,
            spawn_y,
            vx,
            vy,
            ttl_ms,
            radius,
        ) {
            if let Some(state) = &self.emitter_state {
                state.track_spawn(effect_name, Some(ship_id), id);
            }
            vec![(id as rhai::INT).into()]
        } else {
            rhai::Array::new()
        }
    }

    fn emit_disintegration(
        &mut self,
        effect_name: &str,
        world: &GameplayWorld,
        args: &RhaiMap,
        config: Option<&catalog::EmitterConfig>,
    ) -> rhai::Array {
        let x = args
            .get("x")
            .and_then(|v| {
                v.clone()
                    .try_cast::<rhai::FLOAT>()
                    .or_else(|| v.clone().try_cast::<rhai::INT>().map(|i| i as rhai::FLOAT))
            })
            .unwrap_or(0.0) as f32;
        let y = args
            .get("y")
            .and_then(|v| {
                v.clone()
                    .try_cast::<rhai::FLOAT>()
                    .or_else(|| v.clone().try_cast::<rhai::INT>().map(|i| i as rhai::FLOAT))
            })
            .unwrap_or(0.0) as f32;

        let ttl_ms = config.and_then(|c| c.ttl_ms).unwrap_or(800) as i32;
        let radius = config.and_then(|c| c.radius).unwrap_or(4) as i64;
        let velocity_scale = config.and_then(|c| c.velocity_scale).unwrap_or(60.0) as f32;
        let count = 12;

        let mut ids = rhai::Array::new();
        for i in 0..count {
            let max_count = config.and_then(|c| c.max_count).unwrap_or(20) as usize;
            if let Some(state) = &self.emitter_state {
                while state.active_count(effect_name, None) >= max_count {
                    let Some(oldest_id) = state.evict_oldest(effect_name, None) else {
                        break;
                    };
                    if let Some(binding) = world.visual(oldest_id) {
                        for target in binding.all_visual_ids() {
                            if let Ok(mut commands) = self.queue.lock() {
                                commands.push(BehaviorCommand::SceneDespawn {
                                    target: target.to_string(),
                                });
                            }
                        }
                    }
                    let _ = world.despawn(oldest_id);
                    state.remove_entity(oldest_id);
                }
            }
            let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
            let speed = 0.5 * velocity_scale;
            let vx = angle.cos() * speed;
            let vy = angle.sin() * speed;
            if let Some(id) = self.spawn_smoke_entity(world, None, x, y, vx, vy, ttl_ms, radius) {
                if let Some(state) = &self.emitter_state {
                    state.track_spawn(effect_name, None, id);
                }
                ids.push((id as rhai::INT).into());
            }
        }
        ids
    }

    fn spawn_smoke_entity(
        &mut self,
        world: &GameplayWorld,
        owner_id: Option<u64>,
        x: f32,
        y: f32,
        vx: f32,
        vy: f32,
        ttl_ms: i32,
        radius: i64,
    ) -> Option<u64> {
        let mut extra_data = BTreeMap::new();
        extra_data.insert("ttl_ms".to_string(), JsonValue::from(ttl_ms as i64));
        extra_data.insert("max_ttl_ms".to_string(), JsonValue::from(ttl_ms as i64));
        extra_data.insert("radius".to_string(), JsonValue::from(radius));

        spawn_ephemeral_visual(
            world,
            &self.queue,
            EphemeralSpawn {
                kind: "smoke",
                template: "smoke-template",
                x,
                y,
                heading: 0.0,
                vx,
                vy,
                drag: 0.04,
                max_speed: 0.0,
                ttl_ms: Some(ttl_ms),
                owner_id,
                lifecycle: if owner_id.is_some() {
                    LifecyclePolicy::TtlOwnerBound
                } else {
                    LifecyclePolicy::Ttl
                },
                extra_data,
            },
        )
    }
}

pub(crate) fn register_with_rhai(engine: &mut RhaiEngine) {
    engine.register_type_with_name::<ScriptAudioApi>("AudioApi");
    engine.register_type_with_name::<ScriptFxApi>("FxApi");

    engine.register_fn("cue", |audio: &mut ScriptAudioApi, cue: &str| {
        audio.cue(cue, None)
    });
    engine.register_fn(
        "cue",
        |audio: &mut ScriptAudioApi, cue: &str, volume: rhai::FLOAT| {
            audio.cue(cue, Some(volume as f32))
        },
    );
    engine.register_fn("event", |audio: &mut ScriptAudioApi, event: &str| {
        audio.event(event, None)
    });
    engine.register_fn(
        "event",
        |audio: &mut ScriptAudioApi, event: &str, gain_scale: rhai::FLOAT| {
            audio.event(event, Some(gain_scale as f32))
        },
    );
    engine.register_fn("play_song", |audio: &mut ScriptAudioApi, song_id: &str| {
        audio.play_song(song_id)
    });
    engine.register_fn("stop_song", |audio: &mut ScriptAudioApi| audio.stop_song());

    engine.register_fn(
        "emit",
        |fx: &mut ScriptFxApi, effect_name: &str, args: RhaiMap| fx.emit(effect_name, args),
    );
}
