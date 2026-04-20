//! Neutral vehicle-domain API surface.
//!
//! This module intentionally stays thin. The runtime currently exposes only a
//! generic controlled-entity slot through `GameplayWorld`, so the vehicle
//! domain is expressed as a facade over that slot until dedicated vehicle
//! accessors exist. Once `engine-vehicle` lands in the workspace, its runtime
//! selection handle can implement `VehicleSelectionApi` directly without
//! changing the Rhai surface.
//!
//! The current script-facing slice stops at:
//! - active-vehicle selection,
//! - typed vehicle value construction / normalization,
//! - thin profile/helper lookups that forward directly to `engine-vehicle`,
//! - required ship-runtime passthrough that delegates into `engine-vehicle`,
//! - canonical handoff DTO helpers.
//!
//! Concrete ship-runtime semantics still stay in `engine-vehicle`; this module
//! only forwards the minimal typed constructors / step hook that existing mods
//! already use.

use engine_game::GameplayWorld;
use engine_vehicle::{
    builtin_ship_profile_tuning, next_builtin_ship_profile_id, normalize_vehicle_profile_id,
    ShipModel, ShipRuntimeInput, ShipRuntimeModel, ShipRuntimeOutput, ShipRuntimeState,
    VehicleAssistState, VehicleButtonInput, VehicleControlState, VehicleInputIntent,
    VehicleLaunchPacket, VehiclePacketEnvelope, VehiclePacketTelemetry, VehiclePacketVehicle,
    VehicleReturnPacket, VehicleShipProfileTuning, DEFAULT_VEHICLE_PROFILE_ID,
};
use rhai::{Dynamic as RhaiDynamic, Engine as RhaiEngine, Map as RhaiMap};
use serde::{de::DeserializeOwned, Serialize};

use crate::gameplay::api::ScriptWorldContext;
use crate::rhai::conversion::{json_to_rhai_dynamic, rhai_dynamic_to_json};

/// Minimal runtime-selection seam required by the vehicle facade.
///
/// `GameplayWorld` implements this today via its generic controlled-entity
/// slot. A dedicated `engine-vehicle` runtime handle can implement the same
/// trait later so the script-facing `vehicle.*` API stays unchanged.
pub trait VehicleSelectionApi: Clone + 'static {
    fn set_active_vehicle(&self, id: u64) -> bool;
    fn active_vehicle(&self) -> Option<u64>;
    fn clear_active_vehicle(&self) -> bool;
}

impl VehicleSelectionApi for GameplayWorld {
    fn set_active_vehicle(&self, id: u64) -> bool {
        self.set_controlled_entity(id)
    }

    fn active_vehicle(&self) -> Option<u64> {
        self.controlled_entity()
    }

    fn clear_active_vehicle(&self) -> bool {
        self.clear_controlled_entity()
    }
}

/// Minimal runtime contract for a vehicle-domain script API.
///
/// The canonical vehicle surface is intentionally small:
/// - choose the active vehicle entity,
/// - query the currently active vehicle entity,
/// - clear the active vehicle selection.
///
/// Runtime crates can implement this trait directly or reuse
/// `ScriptVehicleApi`, which adapts a runtime selection handle. The raw
/// `controlled_entity` naming remains part of the generic gameplay API; the
/// vehicle domain keeps its own neutral `active` vocabulary.
pub trait VehicleCoreApi: Clone + 'static {
    fn set_active(&mut self, id: rhai::INT) -> bool;
    fn active(&mut self) -> rhai::INT;
    fn clear_active(&mut self) -> bool;
}

fn typed_to_rhai_map<T>(value: &T) -> RhaiMap
where
    T: Serialize,
{
    serde_json::to_value(value)
        .ok()
        .and_then(|value| json_to_rhai_dynamic(&value).try_cast::<RhaiMap>())
        .unwrap_or_default()
}

fn rhai_map_to_typed<T>(map: RhaiMap) -> Option<T>
where
    T: DeserializeOwned,
{
    let json = rhai_dynamic_to_json(&RhaiDynamic::from_map(map))?;
    serde_json::from_value::<T>(json).ok()
}

fn rhai_map_to_typed_or_default<T>(map: RhaiMap) -> T
where
    T: DeserializeOwned + Default,
{
    rhai_map_to_typed(map).unwrap_or_default()
}

fn register_vehicle_value_types(engine: &mut RhaiEngine) {
    engine.register_type_with_name::<VehicleAssistState>("VehicleAssistState");
    engine.register_type_with_name::<VehicleButtonInput>("VehicleButtonInput");
    engine.register_type_with_name::<VehicleShipProfileTuning>("VehicleShipProfileTuning");
    engine.register_type_with_name::<VehicleInputIntent>("VehicleInputIntent");
    engine.register_type_with_name::<VehicleControlState>("VehicleControlState");
    engine.register_type_with_name::<ShipRuntimeState>("ShipRuntimeState");
    engine.register_type_with_name::<ShipRuntimeInput>("ShipRuntimeInput");
    engine.register_type_with_name::<ShipRuntimeOutput>("ShipRuntimeOutput");
    engine.register_type_with_name::<VehiclePacketEnvelope>("VehiclePacketEnvelope");
    engine.register_type_with_name::<VehiclePacketTelemetry>("VehiclePacketTelemetry");
    engine.register_type_with_name::<VehiclePacketVehicle>("VehiclePacketVehicle");
    engine.register_type_with_name::<VehicleLaunchPacket>("VehicleLaunchPacket");
    engine.register_type_with_name::<VehicleReturnPacket>("VehicleReturnPacket");
}

fn register_vehicle_value_methods(engine: &mut RhaiEngine) {
    engine.register_fn("to_map", |assists: &mut VehicleAssistState| {
        typed_to_rhai_map(assists)
    });
    engine.register_fn("any_enabled", |assists: &mut VehicleAssistState| {
        assists.any_enabled()
    });

    engine.register_fn("normalized", |buttons: &mut VehicleButtonInput| *buttons);
    engine.register_fn("to_map", |buttons: &mut VehicleButtonInput| {
        typed_to_rhai_map(buttons)
    });
    engine.register_fn("is_idle", |buttons: &mut VehicleButtonInput| {
        buttons.is_idle()
    });
    engine.register_fn("intent", |buttons: &mut VehicleButtonInput| {
        buttons.intent().normalized()
    });
    engine.register_fn(
        "control_state",
        |buttons: &mut VehicleButtonInput, profile_id: &str, assists: VehicleAssistState| {
            buttons.control_state(profile_id, assists).normalized()
        },
    );

    engine.register_fn("normalized", |tuning: &mut VehicleShipProfileTuning| {
        tuning.normalized()
    });
    engine.register_fn("to_map", |tuning: &mut VehicleShipProfileTuning| {
        typed_to_rhai_map(tuning)
    });

    engine.register_fn("normalized", |intent: &mut VehicleInputIntent| {
        intent.normalized()
    });
    engine.register_fn("to_map", |intent: &mut VehicleInputIntent| {
        typed_to_rhai_map(intent)
    });
    engine.register_fn("is_idle", |intent: &mut VehicleInputIntent| {
        intent.is_idle()
    });
    engine.register_fn("has_translation", |intent: &mut VehicleInputIntent| {
        intent.has_translation()
    });
    engine.register_fn("has_rotation", |intent: &mut VehicleInputIntent| {
        intent.has_rotation()
    });

    engine.register_fn("normalized", |control: &mut VehicleControlState| {
        control.clone().normalized()
    });
    engine.register_fn("to_map", |control: &mut VehicleControlState| {
        typed_to_rhai_map(control)
    });
    engine.register_fn("intent", |control: &mut VehicleControlState| {
        control.intent()
    });
    engine.register_fn("uses_assists", |control: &mut VehicleControlState| {
        control.uses_assists()
    });
    engine.register_fn(
        "profile_matches",
        |control: &mut VehicleControlState, profile_id: &str| control.profile_matches(profile_id),
    );
    engine.register_fn("ship_profile_id", |control: &mut VehicleControlState| {
        control
            .ship_profile()
            .map(|profile| profile.profile_id().to_string())
    });
    engine.register_fn("cycle_ship_profile", |control: &mut VehicleControlState| {
        control.cycle_ship_profile().to_string()
    });
    engine.register_fn(
        "toggle_altitude_hold",
        |control: &mut VehicleControlState, target_altitude_km: rhai::FLOAT| {
            control.toggle_altitude_hold(Some(target_altitude_km as f32))
        },
    );
    engine.register_fn(
        "toggle_altitude_hold",
        |control: &mut VehicleControlState| control.toggle_altitude_hold(None),
    );
    engine.register_fn(
        "toggle_heading_hold",
        |control: &mut VehicleControlState, target_heading_rad: rhai::FLOAT| {
            control.toggle_heading_hold(Some(target_heading_rad as f32))
        },
    );
    engine.register_fn(
        "toggle_heading_hold",
        |control: &mut VehicleControlState| control.toggle_heading_hold(None),
    );

    engine.register_fn("normalized", |state: &mut ShipRuntimeState| {
        state.clone().normalized()
    });
    engine.register_fn("to_map", |state: &mut ShipRuntimeState| {
        typed_to_rhai_map(state)
    });
    engine.register_fn("is_grounded", |state: &mut ShipRuntimeState| {
        state.is_grounded()
    });
    engine.register_fn("is_surface_locked", |state: &mut ShipRuntimeState| {
        state.is_surface_locked()
    });
    engine.register_fn("is_detached", |state: &mut ShipRuntimeState| {
        state.is_detached()
    });

    engine.register_fn("normalized", |input: &mut ShipRuntimeInput| {
        input.clone().normalized()
    });
    engine.register_fn("to_map", |input: &mut ShipRuntimeInput| {
        typed_to_rhai_map(input)
    });

    engine.register_fn("to_map", |output: &mut ShipRuntimeOutput| {
        typed_to_rhai_map(output)
    });

    engine.register_fn("normalized", |envelope: &mut VehiclePacketEnvelope| {
        envelope.clone().normalized()
    });
    engine.register_fn(
        "normalized_for_launch",
        |envelope: &mut VehiclePacketEnvelope| {
            let mut envelope = envelope.clone();
            envelope.normalize_for_launch();
            envelope
        },
    );
    engine.register_fn(
        "normalized_for_return",
        |envelope: &mut VehiclePacketEnvelope| {
            let mut envelope = envelope.clone();
            envelope.normalize_for_return();
            envelope
        },
    );
    engine.register_fn("to_map", |envelope: &mut VehiclePacketEnvelope| {
        typed_to_rhai_map(envelope)
    });

    engine.register_fn("normalized", |telemetry: &mut VehiclePacketTelemetry| {
        telemetry.clone().normalized()
    });
    engine.register_fn("to_map", |telemetry: &mut VehiclePacketTelemetry| {
        typed_to_rhai_map(telemetry)
    });

    engine.register_fn("normalized", |vehicle: &mut VehiclePacketVehicle| {
        vehicle.clone().normalized()
    });
    engine.register_fn("to_map", |vehicle: &mut VehiclePacketVehicle| {
        typed_to_rhai_map(vehicle)
    });
    engine.register_fn("to_control_state", |vehicle: &mut VehiclePacketVehicle| {
        vehicle.to_control_state().normalized()
    });

    engine.register_fn("normalized", |packet: &mut VehicleLaunchPacket| {
        packet.clone().normalized()
    });
    engine.register_fn("to_map", |packet: &mut VehicleLaunchPacket| {
        typed_to_rhai_map(packet)
    });
    engine.register_fn("is_vehicle_handoff", |packet: &mut VehicleLaunchPacket| {
        packet.is_vehicle_handoff()
    });

    engine.register_fn("normalized", |packet: &mut VehicleReturnPacket| {
        packet.clone().normalized()
    });
    engine.register_fn("to_map", |packet: &mut VehicleReturnPacket| {
        typed_to_rhai_map(packet)
    });
    engine.register_fn("is_vehicle_return", |packet: &mut VehicleReturnPacket| {
        packet.is_vehicle_return()
    });
}

fn register_vehicle_value_api<TVehicle>(engine: &mut RhaiEngine)
where
    TVehicle: VehicleCoreApi,
{
    engine.register_fn("default_profile_id", |_vehicle: &mut TVehicle| {
        DEFAULT_VEHICLE_PROFILE_ID.to_string()
    });
    engine.register_fn(
        "normalize_profile_id",
        |_vehicle: &mut TVehicle, profile_id: &str| normalize_vehicle_profile_id(profile_id),
    );

    engine.register_fn("assist_state", |_vehicle: &mut TVehicle| {
        VehicleAssistState::default()
    });
    engine.register_fn(
        "assist_state_from",
        |_vehicle: &mut TVehicle, args: RhaiMap| {
            rhai_map_to_typed_or_default::<VehicleAssistState>(args)
        },
    );

    engine.register_fn("input_intent", |_vehicle: &mut TVehicle| {
        VehicleInputIntent::default()
    });
    engine.register_fn("button_input", |_vehicle: &mut TVehicle| {
        VehicleButtonInput::default()
    });
    engine.register_fn(
        "button_input_from",
        |_vehicle: &mut TVehicle, args: RhaiMap| {
            rhai_map_to_typed_or_default::<VehicleButtonInput>(args)
        },
    );
    engine.register_fn(
        "next_ship_profile_id",
        |_vehicle: &mut TVehicle, profile_id: &str| {
            next_builtin_ship_profile_id(profile_id).to_string()
        },
    );
    engine.register_fn(
        "ship_profile_tuning",
        |_vehicle: &mut TVehicle, profile_id: &str| builtin_ship_profile_tuning(profile_id),
    );
    engine.register_fn(
        "input_intent_from",
        |_vehicle: &mut TVehicle, args: RhaiMap| {
            rhai_map_to_typed_or_default::<VehicleInputIntent>(args).normalized()
        },
    );
    engine.register_fn(
        "input_intent_from_buttons",
        |_vehicle: &mut TVehicle, buttons: VehicleButtonInput| buttons.intent().normalized(),
    );

    engine.register_fn("control_state", |_vehicle: &mut TVehicle| {
        VehicleControlState::default()
    });
    engine.register_fn(
        "control_state_from",
        |_vehicle: &mut TVehicle, args: RhaiMap| {
            rhai_map_to_typed_or_default::<VehicleControlState>(args).normalized()
        },
    );
    engine.register_fn(
        "control_from_intent",
        |_vehicle: &mut TVehicle,
         intent: VehicleInputIntent,
         alt_hold: bool,
         heading_hold: bool| {
            VehicleControlState::from_intent(
                intent,
                VehicleAssistState::from_flags(alt_hold, heading_hold),
            )
            .normalized()
        },
    );
    engine.register_fn(
        "control_from_intent",
        |_vehicle: &mut TVehicle, intent: VehicleInputIntent, assists: VehicleAssistState| {
            VehicleControlState::from_intent(intent, assists).normalized()
        },
    );
    engine.register_fn(
        "control_from_buttons",
        |_vehicle: &mut TVehicle,
         profile_id: &str,
         buttons: VehicleButtonInput,
         assists: VehicleAssistState| {
            VehicleControlState::from_button_input(profile_id, buttons, assists).normalized()
        },
    );
    engine.register_fn("ship_runtime_state", |_vehicle: &mut TVehicle| {
        ShipRuntimeState::default().normalized()
    });
    engine.register_fn(
        "ship_runtime_state_from",
        |_vehicle: &mut TVehicle, args: RhaiMap| {
            rhai_map_to_typed_or_default::<ShipRuntimeState>(args).normalized()
        },
    );
    engine.register_fn("ship_runtime_input", |_vehicle: &mut TVehicle| {
        ShipRuntimeInput::default().normalized()
    });
    engine.register_fn(
        "ship_runtime_input_from",
        |_vehicle: &mut TVehicle, args: RhaiMap| {
            rhai_map_to_typed_or_default::<ShipRuntimeInput>(args).normalized()
        },
    );
    engine.register_fn(
        "ship_runtime_step",
        |_vehicle: &mut TVehicle,
         profile_id: &str,
         previous: ShipRuntimeState,
         input: ShipRuntimeInput| {
            ShipModel::new(profile_id).ship_runtime_step(previous, input)
        },
    );

    engine.register_fn("packet_envelope", |_vehicle: &mut TVehicle| {
        VehiclePacketEnvelope::default().normalized()
    });
    engine.register_fn(
        "packet_envelope_from",
        |_vehicle: &mut TVehicle, args: RhaiMap| {
            rhai_map_to_typed_or_default::<VehiclePacketEnvelope>(args).normalized()
        },
    );

    engine.register_fn("packet_telemetry", |_vehicle: &mut TVehicle| {
        VehiclePacketTelemetry::default()
    });
    engine.register_fn(
        "packet_telemetry_from",
        |_vehicle: &mut TVehicle, args: RhaiMap| {
            rhai_map_to_typed_or_default::<VehiclePacketTelemetry>(args).normalized()
        },
    );

    engine.register_fn("packet_vehicle", |_vehicle: &mut TVehicle| {
        VehiclePacketVehicle::default()
    });
    engine.register_fn(
        "packet_vehicle_from",
        |_vehicle: &mut TVehicle, args: RhaiMap| {
            rhai_map_to_typed_or_default::<VehiclePacketVehicle>(args).normalized()
        },
    );
    engine.register_fn(
        "packet_vehicle_from_control",
        |_vehicle: &mut TVehicle, control: VehicleControlState, spawn_altitude_km: rhai::FLOAT| {
            VehiclePacketVehicle::from_control_state(&control, spawn_altitude_km as f32)
                .normalized()
        },
    );
    engine.register_fn(
        "control_from_packet_vehicle",
        |_vehicle: &mut TVehicle, packet_vehicle: VehiclePacketVehicle| {
            packet_vehicle.to_control_state().normalized()
        },
    );

    engine.register_fn("launch_packet", |_vehicle: &mut TVehicle| {
        VehicleLaunchPacket::default()
    });
    engine.register_fn(
        "launch_packet_from",
        |_vehicle: &mut TVehicle, args: RhaiMap| {
            rhai_map_to_typed_or_default::<VehicleLaunchPacket>(args).normalized()
        },
    );

    engine.register_fn("return_packet", |_vehicle: &mut TVehicle| {
        VehicleReturnPacket::default()
    });
    engine.register_fn(
        "return_packet_from",
        |_vehicle: &mut TVehicle, args: RhaiMap| {
            rhai_map_to_typed_or_default::<VehicleReturnPacket>(args).normalized()
        },
    );
}

/// Script-facing vehicle facade backed by a runtime-selection handle.
#[derive(Clone)]
pub struct ScriptVehicleApi<TSelection = GameplayWorld> {
    selection: Option<TSelection>,
}

impl<TSelection> ScriptVehicleApi<TSelection>
where
    TSelection: VehicleSelectionApi,
{
    pub fn from_selection(selection: Option<TSelection>) -> Self {
        Self { selection }
    }

    fn selection(&self) -> Option<&TSelection> {
        self.selection.as_ref()
    }
}

impl ScriptVehicleApi<GameplayWorld> {
    pub fn new(ctx: ScriptWorldContext) -> Self {
        Self::from_gameplay_world(ctx.world)
    }

    pub fn from_gameplay_world(world: Option<GameplayWorld>) -> Self {
        Self::from_selection(world)
    }
}

impl<TSelection> VehicleCoreApi for ScriptVehicleApi<TSelection>
where
    TSelection: VehicleSelectionApi,
{
    fn set_active(&mut self, id: rhai::INT) -> bool {
        let Ok(id) = u64::try_from(id) else {
            return false;
        };
        if id == 0 {
            return false;
        }
        self.selection()
            .map(|selection| selection.set_active_vehicle(id))
            .unwrap_or(false)
    }

    fn active(&mut self) -> rhai::INT {
        self.selection()
            .and_then(|selection| selection.active_vehicle())
            .and_then(|id| rhai::INT::try_from(id).ok())
            .unwrap_or_default()
    }

    fn clear_active(&mut self) -> bool {
        self.selection()
            .map(VehicleSelectionApi::clear_active_vehicle)
            .unwrap_or(false)
    }
}

/// Register the minimal vehicle-domain method surface for any runtime adapter.
///
/// This does not register a concrete type name so runtime crates can layer the
/// vehicle API onto an already-registered scripting type without conflicts.
pub fn register_vehicle_core_api<TVehicle>(engine: &mut RhaiEngine)
where
    TVehicle: VehicleCoreApi,
{
    register_vehicle_value_types(engine);
    register_vehicle_value_methods(engine);

    engine.register_fn("set_active", |vehicle: &mut TVehicle, id: rhai::INT| {
        vehicle.set_active(id)
    });
    engine.register_fn("active", |vehicle: &mut TVehicle| vehicle.active());
    engine.register_fn("clear_active", |vehicle: &mut TVehicle| {
        vehicle.clear_active()
    });
    register_vehicle_value_api::<TVehicle>(engine);
}

/// Register the concrete `ScriptVehicleApi` adapter into the Rhai engine.
pub fn register_vehicle_api(engine: &mut RhaiEngine) {
    engine.register_type_with_name::<ScriptVehicleApi>("VehicleApi");
    register_vehicle_core_api::<ScriptVehicleApi>(engine);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    use crate::commands::BehaviorCommand;

    fn test_vehicle_api(world: Option<GameplayWorld>) -> ScriptVehicleApi {
        ScriptVehicleApi::from_gameplay_world(world)
    }

    #[test]
    fn vehicle_api_tracks_active_entity_via_controlled_slot() {
        let world = GameplayWorld::new();
        let vehicle_id = world
            .spawn("vehicle", serde_json::json!({}))
            .expect("vehicle id");
        let mut api = test_vehicle_api(Some(world.clone()));

        assert_eq!(api.active(), 0);
        assert!(api.set_active(vehicle_id as rhai::INT));
        assert_eq!(api.active(), vehicle_id as rhai::INT);
        assert!(api.clear_active());
        assert_eq!(api.active(), 0);
    }

    #[test]
    fn vehicle_api_rejects_missing_or_invalid_ids() {
        let world = GameplayWorld::new();
        let mut api = test_vehicle_api(Some(world));

        assert!(!api.set_active(0));
        assert!(!api.set_active(42));
        assert_eq!(api.active(), 0);
    }

    #[test]
    fn vehicle_registration_exposes_neutral_methods() {
        let world = GameplayWorld::new();
        let vehicle_id = world
            .spawn("vehicle", serde_json::json!({}))
            .expect("vehicle id");
        let mut engine = RhaiEngine::new();
        register_vehicle_api(&mut engine);

        let mut scope = rhai::Scope::new();
        scope.push("vehicle", test_vehicle_api(Some(world)));
        scope.push_constant("vehicle_id", vehicle_id as rhai::INT);

        let active: rhai::INT = engine
            .eval_with_scope(
                &mut scope,
                "vehicle.set_active(vehicle_id); vehicle.active()",
            )
            .expect("vehicle.active should resolve");
        assert_eq!(active, vehicle_id as rhai::INT);

        let cleared: bool = engine
            .eval_with_scope(&mut scope, "vehicle.clear_active()")
            .expect("vehicle.clear_active should resolve");
        assert!(cleared);

        let after_clear: rhai::INT = engine
            .eval_with_scope(&mut scope, "vehicle.active()")
            .expect("vehicle.active should resolve");
        assert_eq!(after_clear, 0);
    }

    #[test]
    fn vehicle_registration_exposes_typed_input_and_handoff_helpers() {
        let world = GameplayWorld::new();
        let mut engine = RhaiEngine::new();
        register_vehicle_api(&mut engine);

        let mut scope = rhai::Scope::new();
        scope.push("vehicle", test_vehicle_api(Some(world)));

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let intent = vehicle.input_intent_from(#{
                        throttle: 2.0,
                        yaw: -2.0,
                        strafe: 0.5,
                        lift: -3.0,
                        boost: true,
                        main_engine: true
                    });
                    let control = vehicle.control_from_intent(intent, true, false);
                    let packet_vehicle = vehicle.packet_vehicle_from_control(control, 12.0);
                    let launch = vehicle.launch_packet_from(#{
                        producer_mod_id: " planet-generator ",
                        source_scene_id: " planet-generator-main ",
                        target_mod_ref: " vehicle-playground ",
                        target_scene_id: " vehicle-playground-vehicle ",
                        return_scene_id: " planet-generator-main ",
                        consumer_hint: " vehicle-runtime ",
                        planet: #{
                            body: #{
                                id: " generated-planet ",
                                planet_type: " earth_like ",
                                atmosphere_top_km: 80.0,
                                atmosphere_dense_start_km: 120.0
                            },
                            real_radius_km: 6371.0,
                            scale_divisor: 0.0,
                            surface_gravity_mps2: 9.81,
                            atmo_top_km: 80.0,
                            atmo_dense_start_km: 120.0,
                            atmo_drag_max: 2.0
                        },
                        vehicle: packet_vehicle.to_map(),
                        telemetry: #{
                            heading_deg: -45.0,
                            altitude_km: 12.0,
                            tangent_speed_kms: 1.5
                        }
                    });

                    #{
                        default_profile: vehicle.default_profile_id(),
                        normalized_profile: vehicle.normalize_profile_id(" sim_lite "),
                        intent: intent.to_map(),
                        control: control.to_map(),
                        packet_vehicle: packet_vehicle.to_map(),
                        launch: launch.to_map(),
                        launch_ok: launch.is_vehicle_handoff()
                    }
                "#,
            )
            .expect("vehicle typed helpers should resolve");

        let default_profile = result
            .get("default_profile")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("default profile");
        assert_eq!(default_profile, "arcade");

        let normalized_profile = result
            .get("normalized_profile")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("normalized profile");
        assert_eq!(normalized_profile, "sim-lite");

        let intent = result
            .get("intent")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("intent map");
        let throttle = intent
            .get("throttle")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("intent throttle");
        let lift = intent
            .get("lift")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("intent lift");
        assert_eq!(throttle, 1.0);
        assert_eq!(lift, -1.0);

        let control = result
            .get("control")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("control map");
        let profile_id = control
            .get("profile_id")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("control profile_id");
        let assists = control
            .get("assists")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("control assists");
        let alt_hold = assists
            .get("alt_hold")
            .and_then(|value| value.clone().try_cast::<bool>())
            .expect("assist alt_hold");
        assert_eq!(profile_id, "arcade");
        assert!(alt_hold);

        let packet_vehicle = result
            .get("packet_vehicle")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("packet vehicle map");
        let spawn_altitude = packet_vehicle
            .get("spawn_altitude_km")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("spawn altitude");
        assert_eq!(spawn_altitude, 12.0);

        let launch = result
            .get("launch")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("launch map");
        let packet_kind = launch
            .get("packet_kind")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("launch packet kind");
        let planet = launch
            .get("planet")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("launch planet");
        let scale_divisor = planet
            .get("scale_divisor")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("launch scale_divisor");
        assert_eq!(packet_kind, "vehicle_launch");
        assert!((scale_divisor - 0.0001).abs() < 1.0e-6);

        let launch_ok = result
            .get("launch_ok")
            .and_then(|value| value.clone().try_cast::<bool>())
            .expect("launch ok");
        assert!(launch_ok);
    }

    #[test]
    fn vehicle_registration_exposes_typed_assist_and_packet_helpers() {
        let world = GameplayWorld::new();
        let mut engine = RhaiEngine::new();
        register_vehicle_api(&mut engine);

        let mut scope = rhai::Scope::new();
        scope.push("vehicle", test_vehicle_api(Some(world)));

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let assists = vehicle.assist_state_from(#{
                        alt_hold: true,
                        heading_hold: true
                    });
                    let control = vehicle.control_from_intent(
                        vehicle.input_intent_from(#{
                            throttle: 0.5,
                            yaw: -0.25,
                            boost: true
                        }),
                        assists
                    );
                    let envelope = vehicle.packet_envelope_from(#{
                        producer_mod_id: " planet-generator ",
                        source_scene_id: " main ",
                        target_mod_ref: " vehicle-playground ",
                        target_scene_id: " vehicle-scene ",
                        return_scene_id: " main ",
                        consumer_hint: " vehicle-runtime "
                    }).normalized_for_launch();
                    let telemetry = vehicle.packet_telemetry_from(#{
                        heading_deg: -45.0,
                        altitude_km: -2.0,
                        tangent_speed_kms: 1.5
                    });

                    #{
                        assists: assists.to_map(),
                        assists_any: assists.any_enabled(),
                        control: control.to_map(),
                        envelope: envelope.to_map(),
                        telemetry: telemetry.to_map()
                    }
                "#,
            )
            .expect("vehicle assist and packet helpers should resolve");

        let assists_any = result
            .get("assists_any")
            .and_then(|value| value.clone().try_cast::<bool>())
            .expect("assists_any");
        assert!(assists_any);

        let control = result
            .get("control")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("control map");
        let assists = control
            .get("assists")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("control assists");
        let heading_hold = assists
            .get("heading_hold")
            .and_then(|value| value.clone().try_cast::<bool>())
            .expect("heading hold");
        assert!(heading_hold);

        let envelope = result
            .get("envelope")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("envelope map");
        let packet_kind = envelope
            .get("packet_kind")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("envelope packet kind");
        let packet_version = envelope
            .get("packet_version")
            .and_then(|value| value.clone().try_cast::<rhai::INT>())
            .expect("envelope packet version");
        assert_eq!(packet_kind, "vehicle_launch");
        assert_eq!(packet_version, 1);

        let telemetry = result
            .get("telemetry")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("telemetry map");
        let heading_deg = telemetry
            .get("heading_deg")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("telemetry heading");
        let altitude_km = telemetry
            .get("altitude_km")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("telemetry altitude");
        assert_eq!(heading_deg, 315.0);
        assert_eq!(altitude_km, 0.0);
    }

    #[test]
    fn vehicle_registration_roundtrips_surface_runtime_telemetry_through_launch_and_return_packets()
    {
        let world = GameplayWorld::new();
        let mut engine = RhaiEngine::new();
        register_vehicle_api(&mut engine);

        let mut scope = rhai::Scope::new();
        scope.push("vehicle", test_vehicle_api(Some(world)));

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let telemetry = vehicle.packet_telemetry_from(#{
                        heading_deg: -45.0,
                        altitude_km: 0.0,
                        tangent_speed_kms: 1.25,
                        radial_speed_kms: -0.15,
                        spawn_angle_deg: -30.0,
                        camera_sway: 0.4,
                        radius_wu: 140.0,
                        vfwd_wu_s: 3.25,
                        vright_wu_s: -0.75,
                        vrad_wu_s: -0.2,
                        yaw_rate_rad_s: 0.8,
                        grounded: true
                    });
                    let launch = vehicle.launch_packet_from(#{
                        producer_mod_id: " vehicle-playground ",
                        source_scene_id: " vehicle-scene ",
                        target_mod_ref: " planet-generator ",
                        target_scene_id: " generator-scene ",
                        return_scene_id: " vehicle-scene ",
                        consumer_hint: " runtime-launch ",
                        planet: #{
                            body: #{
                                id: "generated-planet",
                                planet_type: "earth_like"
                            },
                            real_radius_km: 6371.0,
                            scale_divisor: 50.0,
                            surface_gravity_mps2: 9.81,
                            atmo_top_km: 80.0,
                            atmo_dense_start_km: 12.0,
                            atmo_drag_max: 1.2
                        },
                        telemetry: telemetry.to_map()
                    });
                    let ret = vehicle.return_packet_from(#{
                        producer_mod_id: " vehicle-playground ",
                        source_scene_id: " vehicle-scene ",
                        target_mod_ref: " planet-generator ",
                        target_scene_id: " generator-scene ",
                        return_scene_id: " vehicle-scene ",
                        consumer_hint: " runtime-return ",
                        planet: #{
                            body: #{
                                id: "generated-planet",
                                planet_type: "earth_like"
                            },
                            real_radius_km: 6371.0,
                            scale_divisor: 50.0,
                            surface_gravity_mps2: 9.81,
                            atmo_top_km: 80.0,
                            atmo_dense_start_km: 12.0,
                            atmo_drag_max: 1.2
                        },
                        telemetry: telemetry.to_map()
                    });

                    #{
                        telemetry: telemetry.to_map(),
                        launch: launch.to_map(),
                        return_packet: ret.to_map()
                    }
                "#,
            )
            .expect("surface telemetry handoff helpers should resolve");

        let telemetry = result
            .get("telemetry")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("telemetry map");
        let heading_deg = telemetry
            .get("heading_deg")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("telemetry heading");
        let spawn_angle_deg = telemetry
            .get("spawn_angle_deg")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("telemetry spawn angle");
        let grounded = telemetry
            .get("grounded")
            .and_then(|value| value.clone().try_cast::<bool>())
            .expect("telemetry grounded");
        let radius_wu = telemetry
            .get("radius_wu")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("telemetry radius");
        assert_eq!(heading_deg, 315.0);
        assert_eq!(spawn_angle_deg, 330.0);
        assert!(grounded);
        assert_eq!(radius_wu, 140.0);

        let launch = result
            .get("launch")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("launch map");
        let launch_telemetry = launch
            .get("telemetry")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("launch telemetry");
        let launch_grounded = launch_telemetry
            .get("grounded")
            .and_then(|value| value.clone().try_cast::<bool>())
            .expect("launch grounded");
        let launch_spawn_angle = launch_telemetry
            .get("spawn_angle_deg")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("launch spawn angle");
        assert!(launch_grounded);
        assert_eq!(launch_spawn_angle, 330.0);

        let return_packet = result
            .get("return_packet")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("return packet map");
        let return_telemetry = return_packet
            .get("telemetry")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("return telemetry");
        let return_heading = return_telemetry
            .get("heading_deg")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("return heading");
        let return_radius = return_telemetry
            .get("radius_wu")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("return radius");
        assert_eq!(return_heading, 315.0);
        assert_eq!(return_radius, 140.0);
    }

    #[test]
    fn vehicle_registration_exposes_button_input_helpers_without_runtime_ownership() {
        let world = GameplayWorld::new();
        let mut engine = RhaiEngine::new();
        register_vehicle_api(&mut engine);

        let mut scope = rhai::Scope::new();
        scope.push("vehicle", test_vehicle_api(Some(world)));

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let buttons = vehicle.button_input_from(#{
                        forward: true,
                        yaw_left: true,
                        lift_up: true,
                        boost: true
                    });
                    let intent = vehicle.input_intent_from_buttons(buttons);
                    let control = vehicle.control_from_buttons(
                        vehicle.next_ship_profile_id("arcade"),
                        buttons,
                        vehicle.assist_state_from(#{ alt_hold: true })
                    );
                    let tuning = vehicle.ship_profile_tuning("sim_lite");

                    #{
                        intent: intent.to_map(),
                        control: control.to_map(),
                        tuning: tuning.to_map()
                    }
                "#,
            )
            .expect("button input helpers should resolve");

        let intent = result
            .get("intent")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("intent map");
        let throttle = intent
            .get("throttle")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("throttle");
        let yaw = intent
            .get("yaw")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("yaw");
        assert_eq!(throttle, 1.0);
        assert_eq!(yaw, 1.0);

        let control = result
            .get("control")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("control map");
        let profile_id = control
            .get("profile_id")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("profile id");
        assert_eq!(profile_id, "sim-lite");

        let tuning = result
            .get("tuning")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("tuning map");
        let yaw_max = tuning
            .get("yaw_max")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("yaw max");
        let main_engine_g = tuning
            .get("main_engine_g")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("main engine g");
        assert!((yaw_max - 1.1).abs() < 1.0e-6);
        assert!((main_engine_g - 1.46).abs() < 1.0e-6);
    }

    #[test]
    fn vehicle_registration_preserves_required_ship_runtime_passthrough() {
        let world = GameplayWorld::new();
        let mut engine = RhaiEngine::new();
        register_vehicle_api(&mut engine);

        let mut scope = rhai::Scope::new();
        scope.push("vehicle", test_vehicle_api(Some(world)));

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let state = vehicle.ship_runtime_state_from(#{
                        surface_mode: "surface_locked",
                        control: #{
                            profile_id: " sim_lite ",
                            throttle: 1.5
                        }
                    });
                    let input = vehicle.ship_runtime_input_from(#{
                        control: #{
                            profile_id: " sim_lite ",
                            throttle: 0.75
                        },
                        telemetry: #{
                            heading_deg: -90.0,
                            altitude_km: 0.0,
                            tangent_speed_kms: 1.1,
                            radial_speed_kms: -0.05,
                            spawn_angle_deg: -45.0,
                            grounded: true
                        },
                        request_local_horizon: true,
                        request_inertial_frame: true
                    });
                    let step = vehicle.ship_runtime_step(" sim_lite ", state, input);

                    #{
                        state: state.to_map(),
                        input: input.to_map(),
                        step: step.to_map()
                    }
                "#,
            )
            .expect("ship runtime passthrough should resolve");

        let state = result
            .get("state")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("state map");
        let control = state
            .get("control")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("state control");
        let profile_id = control
            .get("profile_id")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("state profile id");
        let grounded = state
            .get("surface_mode")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("surface mode");
        assert_eq!(profile_id, "sim-lite");
        assert_eq!(grounded, "surface_locked");

        let input = result
            .get("input")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("input map");
        let request_local_horizon = input
            .get("request_local_horizon")
            .and_then(|value| value.clone().try_cast::<bool>())
            .expect("request local horizon");
        let request_inertial_frame = input
            .get("request_inertial_frame")
            .and_then(|value| value.clone().try_cast::<bool>())
            .expect("request inertial frame");
        assert!(!request_local_horizon);
        assert!(request_inertial_frame);

        let step = result
            .get("step")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("step map");
        let step_state = step
            .get("state")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("step state");
        let step_control = step_state
            .get("control")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("step control");
        let step_profile = step_control
            .get("profile_id")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("step profile id");
        let step_telemetry = step
            .get("telemetry")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("step telemetry");
        let heading_deg = step_telemetry
            .get("heading_deg")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("step heading");
        assert_eq!(step_profile, "sim-lite");
        assert_eq!(heading_deg, 270.0);
    }

    #[derive(Clone)]
    struct FakeVehicleSelection {
        slot: Arc<Mutex<Option<u64>>>,
    }

    impl VehicleSelectionApi for FakeVehicleSelection {
        fn set_active_vehicle(&self, id: u64) -> bool {
            if id == 0 {
                return false;
            }
            *self.slot.lock().expect("fake vehicle slot") = Some(id);
            true
        }

        fn active_vehicle(&self) -> Option<u64> {
            *self.slot.lock().expect("fake vehicle slot")
        }

        fn clear_active_vehicle(&self) -> bool {
            self.slot
                .lock()
                .expect("fake vehicle slot")
                .take()
                .is_some()
        }
    }

    #[test]
    fn vehicle_core_registration_supports_non_gameplay_selection_adapters() {
        let selection = FakeVehicleSelection {
            slot: Arc::new(Mutex::new(None)),
        };
        let mut engine = RhaiEngine::new();
        engine.register_type_with_name::<ScriptVehicleApi<FakeVehicleSelection>>("VehicleApi");
        register_vehicle_core_api::<ScriptVehicleApi<FakeVehicleSelection>>(&mut engine);

        let mut scope = rhai::Scope::new();
        scope.push(
            "vehicle",
            ScriptVehicleApi::from_selection(Some(selection.clone())),
        );

        let active: rhai::INT = engine
            .eval_with_scope(&mut scope, "vehicle.set_active(41); vehicle.active()")
            .expect("generic vehicle adapter should resolve");
        assert_eq!(active, 41);

        let cleared: bool = engine
            .eval_with_scope(&mut scope, "vehicle.clear_active()")
            .expect("generic vehicle adapter should clear");
        assert!(cleared);
        assert_eq!(selection.active_vehicle(), None);
    }

    #[test]
    fn vehicle_core_registration_exposes_typed_helpers_for_generic_selection_adapters() {
        let selection = FakeVehicleSelection {
            slot: Arc::new(Mutex::new(None)),
        };
        let mut engine = RhaiEngine::new();
        engine.register_type_with_name::<ScriptVehicleApi<FakeVehicleSelection>>("VehicleApi");
        register_vehicle_core_api::<ScriptVehicleApi<FakeVehicleSelection>>(&mut engine);

        let mut scope = rhai::Scope::new();
        scope.push("vehicle", ScriptVehicleApi::from_selection(Some(selection)));

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
                    let control = vehicle.control_state_from(#{
                        profile_id: "sim_lite",
                        throttle: 0.5,
                        assists: #{ alt_hold: true, heading_hold: true },
                        target_altitude_km: 12.0,
                        target_heading_rad: -0.25
                    });
                    let packet_vehicle = vehicle.packet_vehicle_from_control(control, 8.0);
                    #{
                        control: control.to_map(),
                        packet_vehicle: packet_vehicle.to_map()
                    }
                "#,
            )
            .expect("generic selection helpers should resolve");

        let control = result
            .get("control")
            .and_then(|value| value.clone().try_cast::<RhaiMap>())
            .expect("control map");
        let profile_id = control
            .get("profile_id")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("profile id");
        assert_eq!(profile_id, "sim-lite");
    }

    #[test]
    fn legacy_script_world_context_constructor_still_builds_vehicle_api() {
        let world = GameplayWorld::new();
        let api = ScriptVehicleApi::new(ScriptWorldContext::new(
            Some(world),
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
            None,
            Arc::new(Mutex::new(Vec::<BehaviorCommand>::new())),
        ));

        assert!(api.selection().is_some());
    }
}
