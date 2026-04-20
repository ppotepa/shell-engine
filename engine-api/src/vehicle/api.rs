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
    ShipModel, ShipMotionState, ShipReferenceFrameState, ShipRuntimeInput, ShipRuntimeModel,
    ShipRuntimeOutput, ShipRuntimeState, ShipRuntimeStepReport, VehicleAssistState, VehicleBasis3,
    VehicleBodySnapshot, VehicleButtonInput, VehicleControlState, VehicleEnvironmentBinding,
    VehicleEnvironmentSnapshot, VehicleInputIntent, VehicleLaunchPacket, VehiclePacketEnvelope,
    VehiclePacketTelemetry, VehiclePacketVehicle, VehicleReferenceFrame, VehicleReturnPacket,
    VehicleSessionState, VehicleShipProfileTuning, VehicleTelemetrySnapshot,
    DEFAULT_VEHICLE_PROFILE_ID,
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

fn json_map_to_rhai_map(map: &serde_json::Map<String, serde_json::Value>) -> RhaiMap {
    json_to_rhai_dynamic(&serde_json::Value::Object(map.clone()))
        .try_cast::<RhaiMap>()
        .unwrap_or_default()
}

fn register_vehicle_value_types(engine: &mut RhaiEngine) {
    engine.register_type_with_name::<VehicleAssistState>("VehicleAssistState");
    engine.register_type_with_name::<VehicleButtonInput>("VehicleButtonInput");
    engine.register_type_with_name::<VehicleShipProfileTuning>("VehicleShipProfileTuning");
    engine.register_type_with_name::<VehicleInputIntent>("VehicleInputIntent");
    engine.register_type_with_name::<VehicleControlState>("VehicleControlState");
    engine.register_type_with_name::<VehicleReferenceFrame>("VehicleReferenceFrame");
    engine.register_type_with_name::<VehicleEnvironmentBinding>("VehicleEnvironmentBinding");
    engine.register_type_with_name::<VehicleTelemetrySnapshot>("VehicleTelemetrySnapshot");
    engine.register_type_with_name::<VehicleBasis3>("VehicleBasis3");
    engine.register_type_with_name::<VehicleBodySnapshot>("VehicleBodySnapshot");
    engine.register_type_with_name::<VehicleEnvironmentSnapshot>("VehicleEnvironmentSnapshot");
    engine.register_type_with_name::<ShipMotionState>("ShipMotionState");
    engine.register_type_with_name::<ShipReferenceFrameState>("ShipReferenceFrameState");
    engine.register_type_with_name::<ShipRuntimeState>("ShipRuntimeState");
    engine.register_type_with_name::<ShipRuntimeInput>("ShipRuntimeInput");
    engine.register_type_with_name::<ShipRuntimeStepReport>("ShipRuntimeStepReport");
    engine.register_type_with_name::<ShipRuntimeOutput>("ShipRuntimeOutput");
    engine.register_type_with_name::<VehicleSessionState>("VehicleSessionState");
    engine.register_type_with_name::<VehiclePacketEnvelope>("VehiclePacketEnvelope");
    engine.register_type_with_name::<VehiclePacketTelemetry>("VehiclePacketTelemetry");
    engine.register_type_with_name::<VehiclePacketVehicle>("VehiclePacketVehicle");
    engine.register_type_with_name::<VehicleLaunchPacket>("VehicleLaunchPacket");
    engine.register_type_with_name::<VehicleReturnPacket>("VehicleReturnPacket");
}

fn register_vehicle_value_methods(engine: &mut RhaiEngine) {
    engine.register_get("alt_hold", |assists: &mut VehicleAssistState| {
        assists.alt_hold
    });
    engine.register_get("heading_hold", |assists: &mut VehicleAssistState| {
        assists.heading_hold
    });
    engine.register_fn("to_map", |assists: &mut VehicleAssistState| {
        typed_to_rhai_map(assists)
    });
    engine.register_fn("any_enabled", |assists: &mut VehicleAssistState| {
        assists.any_enabled()
    });
    engine.register_fn(
        "assist_state_with",
        |_assists: &mut VehicleAssistState, alt_hold: bool, heading_hold: bool| {
            VehicleAssistState::from_flags(alt_hold, heading_hold)
        },
    );

    engine.register_fn("normalized", |buttons: &mut VehicleButtonInput| *buttons);
    engine.register_fn("to_map", |buttons: &mut VehicleButtonInput| {
        typed_to_rhai_map(buttons)
    });
    engine.register_get("forward", |buttons: &mut VehicleButtonInput| {
        buttons.forward
    });
    engine.register_get("reverse", |buttons: &mut VehicleButtonInput| {
        buttons.reverse
    });
    engine.register_get("strafe_left", |buttons: &mut VehicleButtonInput| {
        buttons.strafe_left
    });
    engine.register_get("strafe_right", |buttons: &mut VehicleButtonInput| {
        buttons.strafe_right
    });
    engine.register_get("lift_up", |buttons: &mut VehicleButtonInput| {
        buttons.lift_up
    });
    engine.register_get("lift_down", |buttons: &mut VehicleButtonInput| {
        buttons.lift_down
    });
    engine.register_get("yaw_left", |buttons: &mut VehicleButtonInput| {
        buttons.yaw_left
    });
    engine.register_get("yaw_right", |buttons: &mut VehicleButtonInput| {
        buttons.yaw_right
    });
    engine.register_get("boost", |buttons: &mut VehicleButtonInput| buttons.boost);
    engine.register_get("main_engine", |buttons: &mut VehicleButtonInput| {
        buttons.main_engine
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
    engine.register_get("normal", |basis: &mut VehicleBasis3| {
        basis.normal
            .iter()
            .map(|value| rhai::Dynamic::from_float(*value as rhai::FLOAT))
            .collect::<rhai::Array>()
    });
    engine.register_get("forward", |basis: &mut VehicleBasis3| {
        basis.forward
            .iter()
            .map(|value| rhai::Dynamic::from_float(*value as rhai::FLOAT))
            .collect::<rhai::Array>()
    });
    engine.register_get("right", |basis: &mut VehicleBasis3| {
        basis.right
            .iter()
            .map(|value| rhai::Dynamic::from_float(*value as rhai::FLOAT))
            .collect::<rhai::Array>()
    });
    engine.register_get("normal_x", |basis: &mut VehicleBasis3| basis.normal[0] as rhai::FLOAT);
    engine.register_get("normal_y", |basis: &mut VehicleBasis3| basis.normal[1] as rhai::FLOAT);
    engine.register_get("normal_z", |basis: &mut VehicleBasis3| basis.normal[2] as rhai::FLOAT);
    engine.register_get(
        "forward_x",
        |basis: &mut VehicleBasis3| basis.forward[0] as rhai::FLOAT,
    );
    engine.register_get(
        "forward_y",
        |basis: &mut VehicleBasis3| basis.forward[1] as rhai::FLOAT,
    );
    engine.register_get(
        "forward_z",
        |basis: &mut VehicleBasis3| basis.forward[2] as rhai::FLOAT,
    );
    engine.register_get("right_x", |basis: &mut VehicleBasis3| basis.right[0] as rhai::FLOAT);
    engine.register_get("right_y", |basis: &mut VehicleBasis3| basis.right[1] as rhai::FLOAT);
    engine.register_get("right_z", |basis: &mut VehicleBasis3| basis.right[2] as rhai::FLOAT);

    engine.register_fn("normalized", |tuning: &mut VehicleShipProfileTuning| {
        tuning.normalized()
    });
    engine.register_fn("to_map", |tuning: &mut VehicleShipProfileTuning| {
        typed_to_rhai_map(tuning)
    });
    engine.register_get(
        "forward_accel_g",
        |tuning: &mut VehicleShipProfileTuning| tuning.forward_accel_g as rhai::FLOAT,
    );
    engine.register_get("side_accel_g", |tuning: &mut VehicleShipProfileTuning| {
        tuning.side_accel_g as rhai::FLOAT
    });
    engine.register_get("lift_accel_g", |tuning: &mut VehicleShipProfileTuning| {
        tuning.lift_accel_g as rhai::FLOAT
    });
    engine.register_get("main_engine_g", |tuning: &mut VehicleShipProfileTuning| {
        tuning.main_engine_g as rhai::FLOAT
    });
    engine.register_get(
        "max_speed_ratio",
        |tuning: &mut VehicleShipProfileTuning| tuning.max_speed_ratio as rhai::FLOAT,
    );
    engine.register_get("max_vrad_ratio", |tuning: &mut VehicleShipProfileTuning| {
        tuning.max_vrad_ratio as rhai::FLOAT
    });
    engine.register_get("yaw_response", |tuning: &mut VehicleShipProfileTuning| {
        tuning.yaw_response as rhai::FLOAT
    });
    engine.register_get("yaw_damp", |tuning: &mut VehicleShipProfileTuning| {
        tuning.yaw_damp as rhai::FLOAT
    });
    engine.register_get("yaw_max", |tuning: &mut VehicleShipProfileTuning| {
        tuning.yaw_max as rhai::FLOAT
    });
    engine.register_get(
        "surface_contact_altitude_threshold_km",
        |tuning: &mut VehicleShipProfileTuning| {
            tuning.surface_contact_altitude_threshold_km as rhai::FLOAT
        },
    );
    engine.register_get(
        "surface_clearance_km",
        |tuning: &mut VehicleShipProfileTuning| tuning.surface_clearance_km as rhai::FLOAT,
    );
    engine.register_get(
        "surface_clearance_min_wu",
        |tuning: &mut VehicleShipProfileTuning| tuning.surface_clearance_min_wu as rhai::FLOAT,
    );
    engine.register_get(
        "takeoff_lift_threshold",
        |tuning: &mut VehicleShipProfileTuning| tuning.takeoff_lift_threshold as rhai::FLOAT,
    );
    engine.register_get(
        "grounded_speed_threshold_wu_s",
        |tuning: &mut VehicleShipProfileTuning| tuning.grounded_speed_threshold_wu_s as rhai::FLOAT,
    );
    engine.register_get("linear_damp", |tuning: &mut VehicleShipProfileTuning| {
        tuning.linear_damp as rhai::FLOAT
    });
    engine.register_get("side_trim", |tuning: &mut VehicleShipProfileTuning| {
        tuning.side_trim as rhai::FLOAT
    });
    engine.register_get("side_thrust_trim", |tuning: &mut VehicleShipProfileTuning| {
        tuning.side_thrust_trim as rhai::FLOAT
    });
    engine.register_get("camera_sway_tau", |tuning: &mut VehicleShipProfileTuning| {
        tuning.camera_sway_tau as rhai::FLOAT
    });
    engine.register_get("camera_sway_gain", |tuning: &mut VehicleShipProfileTuning| {
        tuning.camera_sway_gain as rhai::FLOAT
    });
    engine.register_get("heading_hold_kp", |tuning: &mut VehicleShipProfileTuning| {
        tuning.heading_hold_kp as rhai::FLOAT
    });
    engine.register_get("alt_hold_kp", |tuning: &mut VehicleShipProfileTuning| {
        tuning.alt_hold_kp as rhai::FLOAT
    });
    engine.register_get("alt_hold_kd", |tuning: &mut VehicleShipProfileTuning| {
        tuning.alt_hold_kd as rhai::FLOAT
    });
    engine.register_fn(
        "forward_accel_wu_s2",
        |tuning: &mut VehicleShipProfileTuning, surface_gravity_wu_s2: rhai::FLOAT| {
            tuning.forward_accel_wu_s2(surface_gravity_wu_s2 as f32) as rhai::FLOAT
        },
    );
    engine.register_fn(
        "side_accel_wu_s2",
        |tuning: &mut VehicleShipProfileTuning, surface_gravity_wu_s2: rhai::FLOAT| {
            tuning.side_accel_wu_s2(surface_gravity_wu_s2 as f32) as rhai::FLOAT
        },
    );
    engine.register_fn(
        "lift_accel_wu_s2",
        |tuning: &mut VehicleShipProfileTuning, surface_gravity_wu_s2: rhai::FLOAT| {
            tuning.lift_accel_wu_s2(surface_gravity_wu_s2 as f32) as rhai::FLOAT
        },
    );
    engine.register_fn(
        "main_engine_accel_wu_s2",
        |tuning: &mut VehicleShipProfileTuning, surface_gravity_wu_s2: rhai::FLOAT| {
            tuning.main_engine_accel_wu_s2(surface_gravity_wu_s2 as f32) as rhai::FLOAT
        },
    );
    engine.register_fn(
        "max_speed_wu_s",
        |tuning: &mut VehicleShipProfileTuning, surface_circular_speed_wu_s: rhai::FLOAT| {
            tuning.max_speed_wu_s(surface_circular_speed_wu_s as f32) as rhai::FLOAT
        },
    );
    engine.register_fn(
        "max_radial_speed_wu_s",
        |tuning: &mut VehicleShipProfileTuning, surface_circular_speed_wu_s: rhai::FLOAT| {
            tuning.max_radial_speed_wu_s(surface_circular_speed_wu_s as f32) as rhai::FLOAT
        },
    );
    engine.register_fn(
        "surface_clearance_wu",
        |tuning: &mut VehicleShipProfileTuning, km_per_wu: rhai::FLOAT| {
            tuning.surface_clearance_wu(km_per_wu as f32) as rhai::FLOAT
        },
    );
    engine.register_fn(
        "yaw_blend",
        |tuning: &mut VehicleShipProfileTuning, dt_s: rhai::FLOAT| {
            tuning.yaw_blend(dt_s as f32) as rhai::FLOAT
        },
    );
    engine.register_fn(
        "camera_sway_alpha",
        |tuning: &mut VehicleShipProfileTuning, dt_s: rhai::FLOAT| {
            tuning.camera_sway_alpha(dt_s as f32) as rhai::FLOAT
        },
    );
    engine.register_fn(
        "camera_sway_target",
        |tuning: &mut VehicleShipProfileTuning, yaw_rate_rad_s: rhai::FLOAT| {
            tuning.camera_sway_target(yaw_rate_rad_s as f32) as rhai::FLOAT
        },
    );

    engine.register_fn("normalized", |intent: &mut VehicleInputIntent| {
        intent.normalized()
    });
    engine.register_fn("to_map", |intent: &mut VehicleInputIntent| {
        typed_to_rhai_map(intent)
    });
    engine.register_get("throttle", |intent: &mut VehicleInputIntent| {
        intent.throttle as rhai::FLOAT
    });
    engine.register_get("yaw", |intent: &mut VehicleInputIntent| {
        intent.yaw as rhai::FLOAT
    });
    engine.register_get("strafe", |intent: &mut VehicleInputIntent| {
        intent.strafe as rhai::FLOAT
    });
    engine.register_get("lift", |intent: &mut VehicleInputIntent| {
        intent.lift as rhai::FLOAT
    });
    engine.register_get("pitch", |intent: &mut VehicleInputIntent| {
        intent.pitch as rhai::FLOAT
    });
    engine.register_get("roll", |intent: &mut VehicleInputIntent| {
        intent.roll as rhai::FLOAT
    });
    engine.register_get("brake", |intent: &mut VehicleInputIntent| intent.brake);
    engine.register_get("boost", |intent: &mut VehicleInputIntent| intent.boost);
    engine.register_get("stabilize", |intent: &mut VehicleInputIntent| {
        intent.stabilize
    });
    engine.register_get("main_engine", |intent: &mut VehicleInputIntent| {
        intent.main_engine
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
    engine.register_get("profile_id", |control: &mut VehicleControlState| {
        control.profile_id.clone()
    });
    engine.register_get("throttle", |control: &mut VehicleControlState| {
        control.throttle as rhai::FLOAT
    });
    engine.register_get("yaw", |control: &mut VehicleControlState| {
        control.yaw as rhai::FLOAT
    });
    engine.register_get("strafe", |control: &mut VehicleControlState| {
        control.strafe as rhai::FLOAT
    });
    engine.register_get("lift", |control: &mut VehicleControlState| {
        control.lift as rhai::FLOAT
    });
    engine.register_get("pitch", |control: &mut VehicleControlState| {
        control.pitch as rhai::FLOAT
    });
    engine.register_get("roll", |control: &mut VehicleControlState| {
        control.roll as rhai::FLOAT
    });
    engine.register_get("boost_scale", |control: &mut VehicleControlState| {
        control.boost_scale as rhai::FLOAT
    });
    engine.register_get("brake_active", |control: &mut VehicleControlState| {
        control.brake_active
    });
    engine.register_get("stabilize_active", |control: &mut VehicleControlState| {
        control.stabilize_active
    });
    engine.register_get("main_engine_active", |control: &mut VehicleControlState| {
        control.main_engine_active
    });
    engine.register_get("assists", |control: &mut VehicleControlState| {
        control.assists
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
    engine.register_fn(
        "has_target_altitude_km",
        |control: &mut VehicleControlState| control.target_altitude_km.is_some(),
    );
    engine.register_fn(
        "target_altitude_km",
        |control: &mut VehicleControlState, fallback: rhai::FLOAT| {
            control.target_altitude_km.unwrap_or(fallback as f32) as rhai::FLOAT
        },
    );
    engine.register_fn(
        "has_target_heading_rad",
        |control: &mut VehicleControlState| control.target_heading_rad.is_some(),
    );
    engine.register_fn(
        "target_heading_rad",
        |control: &mut VehicleControlState, fallback: rhai::FLOAT| {
            control.target_heading_rad.unwrap_or(fallback as f32) as rhai::FLOAT
        },
    );
    engine.register_fn(
        "set_target_altitude_km",
        |control: &mut VehicleControlState, target_altitude_km: rhai::FLOAT| {
            control.set_altitude_hold(true, Some(target_altitude_km as f32));
            control.clone().normalized()
        },
    );
    engine.register_fn(
        "clear_target_altitude_km",
        |control: &mut VehicleControlState| {
            control.target_altitude_km = None;
            control.clone().normalized()
        },
    );
    engine.register_fn(
        "set_target_heading_rad",
        |control: &mut VehicleControlState, target_heading_rad: rhai::FLOAT| {
            control.set_heading_hold(true, Some(target_heading_rad as f32));
            control.clone().normalized()
        },
    );
    engine.register_fn(
        "clear_target_heading_rad",
        |control: &mut VehicleControlState| {
            control.target_heading_rad = None;
            control.clone().normalized()
        },
    );
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

    engine.register_get("heading_rad", |reference: &mut VehicleReferenceFrame| {
        reference.heading_rad as rhai::FLOAT
    });
    engine.register_get("heading_deg", |reference: &mut VehicleReferenceFrame| {
        reference.heading_deg() as rhai::FLOAT
    });
    engine.register_get("forward_x", |reference: &mut VehicleReferenceFrame| {
        reference.forward_x as rhai::FLOAT
    });
    engine.register_get("forward_y", |reference: &mut VehicleReferenceFrame| {
        reference.forward_y as rhai::FLOAT
    });
    engine.register_get("right_x", |reference: &mut VehicleReferenceFrame| {
        reference.right_x as rhai::FLOAT
    });
    engine.register_get("right_y", |reference: &mut VehicleReferenceFrame| {
        reference.right_y as rhai::FLOAT
    });

    engine.register_fn(
        "normalized",
        |environment: &mut VehicleEnvironmentBinding| environment.clone().normalized(),
    );
    engine.register_fn("to_map", |environment: &mut VehicleEnvironmentBinding| {
        typed_to_rhai_map(environment)
    });
    engine.register_get("extras", |environment: &mut VehicleEnvironmentBinding| {
        json_map_to_rhai_map(&environment.extras)
    });
    engine.register_get("body_extras", |environment: &mut VehicleEnvironmentBinding| {
        json_map_to_rhai_map(&environment.body_extras)
    });
    engine.register_get("body_id", |environment: &mut VehicleEnvironmentBinding| {
        environment.body_id.clone()
    });
    engine.register_get(
        "body_kind",
        |environment: &mut VehicleEnvironmentBinding| environment.body_kind.clone(),
    );
    engine.register_get(
        "surface_radius_wu",
        |environment: &mut VehicleEnvironmentBinding| environment.surface_radius_wu as rhai::FLOAT,
    );
    engine.register_get(
        "render_radius_wu",
        |environment: &mut VehicleEnvironmentBinding| environment.render_radius_wu as rhai::FLOAT,
    );
    engine.register_get(
        "real_radius_km",
        |environment: &mut VehicleEnvironmentBinding| environment.real_radius_km as rhai::FLOAT,
    );
    engine.register_get(
        "scale_divisor",
        |environment: &mut VehicleEnvironmentBinding| environment.scale_divisor as rhai::FLOAT,
    );
    engine.register_get(
        "gravity_mu_km3_s2",
        |environment: &mut VehicleEnvironmentBinding| environment.gravity_mu_km3_s2 as rhai::FLOAT,
    );
    engine.register_get(
        "surface_gravity_mps2",
        |environment: &mut VehicleEnvironmentBinding| {
            environment.surface_gravity_mps2 as rhai::FLOAT
        },
    );
    engine.register_get(
        "atmosphere_top_km",
        |environment: &mut VehicleEnvironmentBinding| environment.atmosphere_top_km as rhai::FLOAT,
    );
    engine.register_get(
        "atmosphere_dense_start_km",
        |environment: &mut VehicleEnvironmentBinding| {
            environment.atmosphere_dense_start_km as rhai::FLOAT
        },
    );
    engine.register_get(
        "atmosphere_drag_max",
        |environment: &mut VehicleEnvironmentBinding| {
            environment.atmosphere_drag_max as rhai::FLOAT
        },
    );
    engine.register_fn("has_body", |environment: &mut VehicleEnvironmentBinding| {
        environment.has_body()
    });
    engine.register_fn(
        "has_atmosphere",
        |environment: &mut VehicleEnvironmentBinding| environment.has_atmosphere(),
    );
    engine.register_fn(
        "km_per_wu",
        |environment: &mut VehicleEnvironmentBinding| environment.km_per_wu() as rhai::FLOAT,
    );
    engine.register_fn(
        "surface_gravity_wu_s2",
        |environment: &mut VehicleEnvironmentBinding| {
            environment.surface_gravity_wu_s2() as rhai::FLOAT
        },
    );
    engine.register_fn(
        "circular_orbit_speed_wu_s",
        |environment: &mut VehicleEnvironmentBinding| {
            environment.circular_orbit_speed_wu_s() as rhai::FLOAT
        },
    );
    engine.register_fn(
        "altitude_km_from_radius_wu",
        |environment: &mut VehicleEnvironmentBinding, radius_wu: rhai::FLOAT| {
            environment.altitude_km_from_radius_wu(radius_wu as f32) as rhai::FLOAT
        },
    );
    engine.register_fn(
        "radius_wu_from_altitude_km",
        |environment: &mut VehicleEnvironmentBinding, altitude_km: rhai::FLOAT| {
            environment.radius_wu_from_altitude_km(altitude_km as f32) as rhai::FLOAT
        },
    );
    engine.register_fn(
        "atmosphere_drag_factor",
        |environment: &mut VehicleEnvironmentBinding, altitude_wu: rhai::FLOAT| {
            environment.atmosphere_drag_factor(altitude_wu as f32) as rhai::FLOAT
        },
    );
    engine.register_fn(
        "carrier_speed_wu_s",
        |environment: &mut VehicleEnvironmentBinding, radius_wu: rhai::FLOAT| {
            environment.carrier_speed_wu_s(radius_wu as f32) as rhai::FLOAT
        },
    );
    engine.register_fn(
        "to_snapshot",
        |environment: &mut VehicleEnvironmentBinding| environment.to_snapshot(),
    );

    engine.register_fn("normalized", |telemetry: &mut VehicleTelemetrySnapshot| {
        telemetry.clone().normalized()
    });
    engine.register_fn("to_map", |telemetry: &mut VehicleTelemetrySnapshot| {
        typed_to_rhai_map(telemetry)
    });
    engine.register_get("basis", |telemetry: &mut VehicleTelemetrySnapshot| {
        telemetry.basis.unwrap_or_default()
    });
    engine.register_get("reference", |telemetry: &mut VehicleTelemetrySnapshot| {
        telemetry.reference
    });
    engine.register_get("heading_deg", |telemetry: &mut VehicleTelemetrySnapshot| {
        telemetry.heading_deg as rhai::FLOAT
    });
    engine.register_get(
        "surface_mode",
        |telemetry: &mut VehicleTelemetrySnapshot| {
            serde_json::to_value(telemetry.surface_mode)
                .ok()
                .and_then(|value| value.as_str().map(str::to_string))
                .unwrap_or_else(|| "detached".to_string())
        },
    );
    engine.register_get(
        "ship_reference",
        |telemetry: &mut VehicleTelemetrySnapshot| telemetry.ship_reference.clone(),
    );
    engine.register_get("position_x", |telemetry: &mut VehicleTelemetrySnapshot| {
        telemetry.position_x as rhai::FLOAT
    });
    engine.register_get("position_y", |telemetry: &mut VehicleTelemetrySnapshot| {
        telemetry.position_y as rhai::FLOAT
    });
    engine.register_get("altitude_km", |telemetry: &mut VehicleTelemetrySnapshot| {
        telemetry.altitude_km as rhai::FLOAT
    });
    engine.register_get(
        "tangent_speed_kms",
        |telemetry: &mut VehicleTelemetrySnapshot| telemetry.tangent_speed_kms as rhai::FLOAT,
    );
    engine.register_get(
        "radial_speed_kms",
        |telemetry: &mut VehicleTelemetrySnapshot| telemetry.radial_speed_kms as rhai::FLOAT,
    );
    engine.register_get(
        "spawn_angle_deg",
        |telemetry: &mut VehicleTelemetrySnapshot| telemetry.spawn_angle_deg as rhai::FLOAT,
    );
    engine.register_get("camera_sway", |telemetry: &mut VehicleTelemetrySnapshot| {
        telemetry.camera_sway as rhai::FLOAT
    });
    engine.register_get("radius_wu", |telemetry: &mut VehicleTelemetrySnapshot| {
        telemetry.radius_wu as rhai::FLOAT
    });
    engine.register_get(
        "forward_speed_wu_s",
        |telemetry: &mut VehicleTelemetrySnapshot| telemetry.forward_speed_wu_s as rhai::FLOAT,
    );
    engine.register_get(
        "lateral_speed_wu_s",
        |telemetry: &mut VehicleTelemetrySnapshot| telemetry.lateral_speed_wu_s as rhai::FLOAT,
    );
    engine.register_get(
        "radial_speed_wu_s",
        |telemetry: &mut VehicleTelemetrySnapshot| telemetry.radial_speed_wu_s as rhai::FLOAT,
    );
    engine.register_get(
        "yaw_rate_rad_s",
        |telemetry: &mut VehicleTelemetrySnapshot| telemetry.yaw_rate_rad_s as rhai::FLOAT,
    );
    engine.register_get("grounded", |telemetry: &mut VehicleTelemetrySnapshot| {
        telemetry.grounded
    });
    engine.register_fn("heading_rad", |telemetry: &mut VehicleTelemetrySnapshot| {
        telemetry.heading_rad() as rhai::FLOAT
    });
    engine.register_fn("has_basis", |telemetry: &mut VehicleTelemetrySnapshot| {
        telemetry.has_basis()
    });
    engine.register_fn(
        "has_environment",
        |telemetry: &mut VehicleTelemetrySnapshot| telemetry.has_environment(),
    );
    engine.register_fn(
        "with_environment",
        |telemetry: &mut VehicleTelemetrySnapshot, environment: VehicleEnvironmentBinding| {
            telemetry.clone().with_environment(environment)
        },
    );
    engine.register_fn(
        "with_ship_runtime_state",
        |telemetry: &mut VehicleTelemetrySnapshot, state: ShipRuntimeState| {
            telemetry.clone().with_ship_runtime_state(&state)
        },
    );
    engine.register_fn("to_packet", |telemetry: &mut VehicleTelemetrySnapshot| {
        telemetry.to_packet()
    });

    engine.register_get("radius_wu", |motion: &mut ShipMotionState| {
        motion.radius_wu as rhai::FLOAT
    });
    engine.register_get("forward_speed_wu_s", |motion: &mut ShipMotionState| {
        motion.forward_speed_wu_s as rhai::FLOAT
    });
    engine.register_get("lateral_speed_wu_s", |motion: &mut ShipMotionState| {
        motion.lateral_speed_wu_s as rhai::FLOAT
    });
    engine.register_get("radial_speed_wu_s", |motion: &mut ShipMotionState| {
        motion.radial_speed_wu_s as rhai::FLOAT
    });
    engine.register_get("yaw_rate_rad_s", |motion: &mut ShipMotionState| {
        motion.yaw_rate_rad_s as rhai::FLOAT
    });
    engine.register_get("camera_sway", |motion: &mut ShipMotionState| {
        motion.camera_sway as rhai::FLOAT
    });
    engine.register_fn("is_configured", |motion: &mut ShipMotionState| {
        motion.is_configured()
    });

    engine.register_get("reference", |state: &mut ShipReferenceFrameState| {
        state.reference
    });
    engine.register_get("body_id", |state: &mut ShipReferenceFrameState| {
        state.body_id.clone()
    });
    engine.register_get("body_kind", |state: &mut ShipReferenceFrameState| {
        state.body_kind.clone()
    });
    engine.register_get("anchor_angle_deg", |state: &mut ShipReferenceFrameState| {
        state.anchor_angle_deg as rhai::FLOAT
    });
    engine.register_get("radius_wu", |state: &mut ShipReferenceFrameState| {
        state.radius_wu as rhai::FLOAT
    });
    engine.register_get("altitude_km", |state: &mut ShipReferenceFrameState| {
        state.altitude_km as rhai::FLOAT
    });
    engine.register_get("normal_x", |state: &mut ShipReferenceFrameState| {
        state.normal_x as rhai::FLOAT
    });
    engine.register_get("normal_y", |state: &mut ShipReferenceFrameState| {
        state.normal_y as rhai::FLOAT
    });
    engine.register_get("tangent_x", |state: &mut ShipReferenceFrameState| {
        state.tangent_x as rhai::FLOAT
    });
    engine.register_get("tangent_y", |state: &mut ShipReferenceFrameState| {
        state.tangent_y as rhai::FLOAT
    });
    engine.register_get(
        "co_rotation_enabled",
        |state: &mut ShipReferenceFrameState| state.co_rotation_enabled,
    );
    engine.register_get(
        "carrier_speed_wu_s",
        |state: &mut ShipReferenceFrameState| state.carrier_speed_wu_s as rhai::FLOAT,
    );
    engine.register_get("spin_omega_rad_s", |state: &mut ShipReferenceFrameState| {
        state.spin_omega_rad_s as rhai::FLOAT
    });
    engine.register_fn("has_body", |state: &mut ShipReferenceFrameState| {
        state.has_body()
    });
    engine.register_fn(
        "has_surface_anchor",
        |state: &mut ShipReferenceFrameState| state.has_surface_anchor(),
    );
    engine.register_fn(
        "uses_local_horizon",
        |state: &mut ShipReferenceFrameState| state.uses_local_horizon(),
    );
    engine.register_fn("uses_co_rotation", |state: &mut ShipReferenceFrameState| {
        state.uses_co_rotation()
    });
    engine.register_fn("is_configured", |state: &mut ShipReferenceFrameState| {
        state.is_configured()
    });

    engine.register_fn("normalized", |state: &mut ShipRuntimeState| {
        state.clone().normalized()
    });
    engine.register_fn("to_map", |state: &mut ShipRuntimeState| {
        typed_to_rhai_map(state)
    });
    engine.register_get("surface_mode", |state: &mut ShipRuntimeState| {
        serde_json::to_value(state.surface_mode)
            .ok()
            .and_then(|value| value.as_str().map(str::to_string))
            .unwrap_or_else(|| "detached".to_string())
    });
    engine.register_get("reference_frame", |state: &mut ShipRuntimeState| {
        state.reference_frame.clone()
    });
    engine.register_get("motion", |state: &mut ShipRuntimeState| state.motion);
    engine.register_get("control", |state: &mut ShipRuntimeState| {
        state.control.clone()
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

    engine.register_fn("has_environment", |state: &mut ShipRuntimeState| {
        state.environment.is_some()
    });

    engine.register_fn("normalized", |input: &mut ShipRuntimeInput| {
        input.clone().normalized()
    });
    engine.register_fn("to_map", |input: &mut ShipRuntimeInput| {
        typed_to_rhai_map(input)
    });
    engine.register_get("dt_s", |input: &mut ShipRuntimeInput| {
        input.dt_s as rhai::FLOAT
    });
    engine.register_get("control", |input: &mut ShipRuntimeInput| {
        input.control.clone()
    });
    engine.register_get("telemetry", |input: &mut ShipRuntimeInput| {
        input.telemetry.clone()
    });
    engine.register_get("request_surface_lock", |input: &mut ShipRuntimeInput| {
        input.request_surface_lock
    });
    engine.register_get("request_detach", |input: &mut ShipRuntimeInput| {
        input.request_detach
    });
    engine.register_get("request_local_horizon", |input: &mut ShipRuntimeInput| {
        input.request_local_horizon
    });
    engine.register_get("request_inertial_frame", |input: &mut ShipRuntimeInput| {
        input.request_inertial_frame
    });
    engine.register_fn("has_environment", |input: &mut ShipRuntimeInput| {
        input.environment.is_some()
    });
    engine.register_fn(
        "with_dt_s",
        |input: &mut ShipRuntimeInput, dt_s: rhai::FLOAT| input.clone().with_dt_s(dt_s as f32),
    );
    engine.register_fn(
        "with_environment",
        |input: &mut ShipRuntimeInput, environment: VehicleEnvironmentBinding| {
            input.clone().with_environment(environment)
        },
    );
    engine.register_fn(
        "with_surface_contact",
        |input: &mut ShipRuntimeInput, surface_contact: bool| {
            input.clone().with_surface_contact(surface_contact)
        },
    );
    engine.register_fn(
        "with_surface_lock_request",
        |input: &mut ShipRuntimeInput, enabled: bool| {
            input.clone().with_surface_lock_request(enabled)
        },
    );
    engine.register_fn(
        "with_prefer_grounded_on_contact",
        |input: &mut ShipRuntimeInput, enabled: bool| {
            input.clone().with_prefer_grounded_on_contact(enabled)
        },
    );
    engine.register_fn(
        "with_detach_request",
        |input: &mut ShipRuntimeInput, enabled: bool| input.clone().with_detach_request(enabled),
    );
    engine.register_fn(
        "with_local_horizon_request",
        |input: &mut ShipRuntimeInput, enabled: bool| {
            input.clone().with_local_horizon_request(enabled)
        },
    );
    engine.register_fn(
        "with_inertial_frame_request",
        |input: &mut ShipRuntimeInput, enabled: bool| {
            input.clone().with_inertial_frame_request(enabled)
        },
    );

    engine.register_get("state", |output: &mut ShipRuntimeOutput| {
        output.state.clone()
    });
    engine.register_get("telemetry", |output: &mut ShipRuntimeOutput| {
        output.telemetry.clone()
    });
    engine.register_get("report", |output: &mut ShipRuntimeOutput| output.report);
    engine.register_get("surface_mode_changed", |output: &mut ShipRuntimeOutput| {
        output.surface_mode_changed
    });
    engine.register_get(
        "reference_frame_changed",
        |output: &mut ShipRuntimeOutput| output.reference_frame_changed,
    );
    engine.register_get("motion_changed", |output: &mut ShipRuntimeOutput| {
        output.motion_changed
    });
    engine.register_fn("to_map", |output: &mut ShipRuntimeOutput| {
        typed_to_rhai_map(output)
    });
    engine.register_fn("has_environment", |output: &mut ShipRuntimeOutput| {
        output.has_environment()
    });

    engine.register_get("dt_s", |report: &mut ShipRuntimeStepReport| {
        report.dt_s as rhai::FLOAT
    });
    engine.register_get("surface_radius_wu", |report: &mut ShipRuntimeStepReport| {
        report.surface_radius_wu as rhai::FLOAT
    });
    engine.register_get(
        "surface_clearance_wu",
        |report: &mut ShipRuntimeStepReport| report.surface_clearance_wu as rhai::FLOAT,
    );
    engine.register_get(
        "surface_gravity_wu_s2",
        |report: &mut ShipRuntimeStepReport| report.surface_gravity_wu_s2 as rhai::FLOAT,
    );
    engine.register_get(
        "surface_circular_speed_wu_s",
        |report: &mut ShipRuntimeStepReport| report.surface_circular_speed_wu_s as rhai::FLOAT,
    );
    engine.register_get("altitude_wu", |report: &mut ShipRuntimeStepReport| {
        report.altitude_wu as rhai::FLOAT
    });
    engine.register_get("atmosphere_drag", |report: &mut ShipRuntimeStepReport| {
        report.atmosphere_drag as rhai::FLOAT
    });
    engine.register_get(
        "carrier_speed_wu_s",
        |report: &mut ShipRuntimeStepReport| report.carrier_speed_wu_s as rhai::FLOAT,
    );
    engine.register_get(
        "signed_tangential_speed_wu_s",
        |report: &mut ShipRuntimeStepReport| report.signed_tangential_speed_wu_s as rhai::FLOAT,
    );

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
    engine.register_get("packet_kind", |envelope: &mut VehiclePacketEnvelope| {
        envelope.packet_kind.clone()
    });
    engine.register_get("packet_version", |envelope: &mut VehiclePacketEnvelope| {
        envelope.packet_version as rhai::INT
    });
    engine.register_get("producer_mod_id", |envelope: &mut VehiclePacketEnvelope| {
        envelope.producer_mod_id.clone()
    });
    engine.register_get("source_scene_id", |envelope: &mut VehiclePacketEnvelope| {
        envelope.source_scene_id.clone()
    });
    engine.register_get("target_mod_ref", |envelope: &mut VehiclePacketEnvelope| {
        envelope.target_mod_ref.clone()
    });
    engine.register_get("target_scene_id", |envelope: &mut VehiclePacketEnvelope| {
        envelope.target_scene_id.clone()
    });
    engine.register_get("return_scene_id", |envelope: &mut VehiclePacketEnvelope| {
        envelope.return_scene_id.clone()
    });
    engine.register_get("consumer_hint", |envelope: &mut VehiclePacketEnvelope| {
        envelope.consumer_hint.clone()
    });

    engine.register_fn("normalized", |telemetry: &mut VehiclePacketTelemetry| {
        telemetry.clone().normalized()
    });
    engine.register_fn("to_map", |telemetry: &mut VehiclePacketTelemetry| {
        typed_to_rhai_map(telemetry)
    });
    engine.register_get("basis", |telemetry: &mut VehiclePacketTelemetry| {
        telemetry.basis.unwrap_or_default()
    });
    engine.register_get("heading_deg", |telemetry: &mut VehiclePacketTelemetry| {
        telemetry.heading_deg as rhai::FLOAT
    });
    engine.register_get("altitude_km", |telemetry: &mut VehiclePacketTelemetry| {
        telemetry.altitude_km as rhai::FLOAT
    });
    engine.register_get(
        "tangent_speed_kms",
        |telemetry: &mut VehiclePacketTelemetry| telemetry.tangent_speed_kms as rhai::FLOAT,
    );
    engine.register_get(
        "radial_speed_kms",
        |telemetry: &mut VehiclePacketTelemetry| telemetry.radial_speed_kms as rhai::FLOAT,
    );
    engine.register_get(
        "spawn_angle_deg",
        |telemetry: &mut VehiclePacketTelemetry| telemetry.spawn_angle_deg as rhai::FLOAT,
    );
    engine.register_get("camera_sway", |telemetry: &mut VehiclePacketTelemetry| {
        telemetry.camera_sway as rhai::FLOAT
    });
    engine.register_get("radius_wu", |telemetry: &mut VehiclePacketTelemetry| {
        telemetry.radius_wu as rhai::FLOAT
    });
    engine.register_get("vfwd_wu_s", |telemetry: &mut VehiclePacketTelemetry| {
        telemetry.vfwd_wu_s as rhai::FLOAT
    });
    engine.register_get("vright_wu_s", |telemetry: &mut VehiclePacketTelemetry| {
        telemetry.vright_wu_s as rhai::FLOAT
    });
    engine.register_get("vrad_wu_s", |telemetry: &mut VehiclePacketTelemetry| {
        telemetry.vrad_wu_s as rhai::FLOAT
    });
    engine.register_get(
        "yaw_rate_rad_s",
        |telemetry: &mut VehiclePacketTelemetry| telemetry.yaw_rate_rad_s as rhai::FLOAT,
    );
    engine.register_get("grounded", |telemetry: &mut VehiclePacketTelemetry| {
        telemetry.grounded
    });
    engine.register_fn("to_snapshot", |telemetry: &mut VehiclePacketTelemetry| {
        VehicleTelemetrySnapshot::from_packet(telemetry)
    });

    engine.register_fn("normalized", |vehicle: &mut VehiclePacketVehicle| {
        vehicle.clone().normalized()
    });
    engine.register_fn("to_map", |vehicle: &mut VehiclePacketVehicle| {
        typed_to_rhai_map(vehicle)
    });
    engine.register_get("profile_id", |vehicle: &mut VehiclePacketVehicle| {
        vehicle.profile_id.clone()
    });
    engine.register_get("assist_alt_hold", |vehicle: &mut VehiclePacketVehicle| {
        vehicle.assist_alt_hold
    });
    engine.register_get(
        "assist_heading_hold",
        |vehicle: &mut VehiclePacketVehicle| vehicle.assist_heading_hold,
    );
    engine.register_get("spawn_altitude_km", |vehicle: &mut VehiclePacketVehicle| {
        vehicle.spawn_altitude_km as rhai::FLOAT
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
    engine.register_get("envelope", |packet: &mut VehicleLaunchPacket| {
        packet.envelope.clone()
    });
    engine.register_get("environment", |packet: &mut VehicleLaunchPacket| {
        packet.environment.clone()
    });
    engine.register_get("vehicle", |packet: &mut VehicleLaunchPacket| {
        packet.vehicle.clone()
    });
    engine.register_get("ui", |packet: &mut VehicleLaunchPacket| {
        json_map_to_rhai_map(&packet.ui)
    });
    engine.register_fn("is_vehicle_handoff", |packet: &mut VehicleLaunchPacket| {
        packet.is_vehicle_handoff()
    });
    engine.register_fn("control_state", |packet: &mut VehicleLaunchPacket| {
        packet.control_state()
    });
    engine.register_fn("environment_binding", |packet: &mut VehicleLaunchPacket| {
        packet.environment_binding()
    });
    engine.register_fn("has_telemetry", |packet: &mut VehicleLaunchPacket| {
        packet.telemetry.is_some()
    });
    engine.register_fn("telemetry_snapshot", |packet: &mut VehicleLaunchPacket| {
        packet.telemetry_snapshot_or_default()
    });

    engine.register_fn("normalized", |packet: &mut VehicleReturnPacket| {
        packet.clone().normalized()
    });
    engine.register_fn("to_map", |packet: &mut VehicleReturnPacket| {
        typed_to_rhai_map(packet)
    });
    engine.register_get("envelope", |packet: &mut VehicleReturnPacket| {
        packet.envelope.clone()
    });
    engine.register_get("environment", |packet: &mut VehicleReturnPacket| {
        packet.environment.clone()
    });
    engine.register_get("vehicle", |packet: &mut VehicleReturnPacket| {
        packet.vehicle.clone()
    });
    engine.register_get("telemetry", |packet: &mut VehicleReturnPacket| {
        packet.telemetry.clone()
    });
    engine.register_get("ui", |packet: &mut VehicleReturnPacket| {
        json_map_to_rhai_map(&packet.ui)
    });
    engine.register_fn("is_vehicle_return", |packet: &mut VehicleReturnPacket| {
        packet.is_vehicle_return()
    });
    engine.register_fn("control_state", |packet: &mut VehicleReturnPacket| {
        packet.control_state()
    });
    engine.register_fn("environment_binding", |packet: &mut VehicleReturnPacket| {
        packet.environment_binding()
    });
    engine.register_fn("telemetry_snapshot", |packet: &mut VehicleReturnPacket| {
        packet.telemetry_snapshot()
    });

    engine.register_fn("normalized", |body: &mut VehicleBodySnapshot| {
        body.clone().normalized()
    });
    engine.register_fn("to_map", |body: &mut VehicleBodySnapshot| {
        typed_to_rhai_map(body)
    });
    engine.register_get("extras", |body: &mut VehicleBodySnapshot| {
        json_map_to_rhai_map(&body.extras)
    });
    engine.register_get("body_id", |body: &mut VehicleBodySnapshot| {
        body.body_id.clone()
    });
    engine.register_get("body_kind", |body: &mut VehicleBodySnapshot| {
        body.body_kind.clone()
    });
    engine.register_get("surface_radius_wu", |body: &mut VehicleBodySnapshot| {
        body.surface_radius_wu as rhai::FLOAT
    });
    engine.register_get("render_radius_wu", |body: &mut VehicleBodySnapshot| {
        body.render_radius_wu as rhai::FLOAT
    });
    engine.register_get("radius_km", |body: &mut VehicleBodySnapshot| {
        body.radius_km as rhai::FLOAT
    });

    engine.register_fn(
        "normalized",
        |environment: &mut VehicleEnvironmentSnapshot| environment.clone().normalized(),
    );
    engine.register_fn("to_map", |environment: &mut VehicleEnvironmentSnapshot| {
        typed_to_rhai_map(environment)
    });
    engine.register_get("extras", |environment: &mut VehicleEnvironmentSnapshot| {
        json_map_to_rhai_map(&environment.extras)
    });
    engine.register_get("body", |environment: &mut VehicleEnvironmentSnapshot| {
        environment.body.clone()
    });
    engine.register_get(
        "real_radius_km",
        |environment: &mut VehicleEnvironmentSnapshot| environment.real_radius_km as rhai::FLOAT,
    );
    engine.register_get(
        "scale_divisor",
        |environment: &mut VehicleEnvironmentSnapshot| environment.scale_divisor as rhai::FLOAT,
    );
    engine.register_get(
        "surface_gravity_mps2",
        |environment: &mut VehicleEnvironmentSnapshot| {
            environment.surface_gravity_mps2 as rhai::FLOAT
        },
    );
    engine.register_get(
        "atmosphere_top_km",
        |environment: &mut VehicleEnvironmentSnapshot| environment.atmosphere_top_km as rhai::FLOAT,
    );
    engine.register_get(
        "atmosphere_dense_start_km",
        |environment: &mut VehicleEnvironmentSnapshot| {
            environment.atmosphere_dense_start_km as rhai::FLOAT
        },
    );
    engine.register_get(
        "atmosphere_drag_max",
        |environment: &mut VehicleEnvironmentSnapshot| {
            environment.atmosphere_drag_max as rhai::FLOAT
        },
    );

    engine.register_fn("normalized", |session: &mut VehicleSessionState| {
        session.clone().normalized()
    });
    engine.register_fn("to_map", |session: &mut VehicleSessionState| {
        typed_to_rhai_map(session)
    });
    engine.register_get("control", |session: &mut VehicleSessionState| {
        session.control.clone()
    });
    engine.register_get("runtime", |session: &mut VehicleSessionState| {
        session.runtime.clone()
    });
    engine.register_get("telemetry", |session: &mut VehicleSessionState| {
        session.telemetry.clone()
    });
    engine.register_get("spawn_altitude_km", |session: &mut VehicleSessionState| {
        session.spawn_altitude_km as rhai::FLOAT
    });
    engine.register_get("spawn_angle_deg", |session: &mut VehicleSessionState| {
        session.spawn_angle_deg as rhai::FLOAT
    });
    engine.register_fn("has_environment", |session: &mut VehicleSessionState| {
        session.environment.is_some()
    });
    engine.register_fn(
        "with_control",
        |session: &mut VehicleSessionState, control: VehicleControlState| {
            session.clone().with_control(control)
        },
    );
    engine.register_fn(
        "with_runtime",
        |session: &mut VehicleSessionState, runtime: ShipRuntimeState| {
            session.clone().with_runtime(runtime)
        },
    );
    engine.register_fn(
        "with_telemetry",
        |session: &mut VehicleSessionState, telemetry: VehicleTelemetrySnapshot| {
            session.clone().with_telemetry(telemetry)
        },
    );
    engine.register_fn(
        "with_environment",
        |session: &mut VehicleSessionState, environment: VehicleEnvironmentBinding| {
            session.clone().with_environment(environment)
        },
    );
    engine.register_fn(
        "with_spawn",
        |session: &mut VehicleSessionState,
         spawn_altitude_km: rhai::FLOAT,
         spawn_angle_deg: rhai::FLOAT| {
            session
                .clone()
                .with_spawn(spawn_altitude_km as f32, spawn_angle_deg as f32)
        },
    );
    engine.register_fn(
        "apply_runtime_output",
        |session: &mut VehicleSessionState, output: ShipRuntimeOutput| {
            let mut session = session.clone();
            session.apply_runtime_output(&output);
            session
        },
    );
    engine.register_fn("packet_vehicle", |session: &mut VehicleSessionState| {
        session.packet_vehicle()
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
        "assist_state_with",
        |_vehicle: &mut TVehicle, alt_hold: bool, heading_hold: bool| {
            VehicleAssistState::from_flags(alt_hold, heading_hold)
        },
    );
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
        "button_input_pressed",
        |_vehicle: &mut TVehicle,
         forward: bool,
         reverse: bool,
         strafe_left: bool,
         strafe_right: bool,
         lift_up: bool,
         yaw_left: bool,
         yaw_right: bool,
         boost: bool,
         main_engine: bool| {
            VehicleButtonInput {
                forward,
                reverse,
                strafe_left,
                strafe_right,
                lift_up,
                lift_down: false,
                yaw_left,
                yaw_right,
                boost,
                main_engine,
                ..VehicleButtonInput::default()
            }
        },
    );
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
        "control_state_with",
        |_vehicle: &mut TVehicle, profile_id: &str, alt_hold: bool, heading_hold: bool| {
            let mut control = VehicleControlState::with_profile_id(profile_id);
            control.assists = VehicleAssistState::from_flags(alt_hold, heading_hold);
            control.normalized()
        },
    );
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
        "ship_runtime_state_from_telemetry",
        |_vehicle: &mut TVehicle, telemetry: VehicleTelemetrySnapshot| {
            ShipModel::new(DEFAULT_VEHICLE_PROFILE_ID).ship_runtime_state_from_telemetry(telemetry)
        },
    );
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
        "ship_runtime_input_from_parts",
        |_vehicle: &mut TVehicle,
         control: VehicleControlState,
         telemetry: VehicleTelemetrySnapshot| {
            ShipRuntimeInput::from_parts(control, telemetry)
        },
    );
    engine.register_fn(
        "ship_runtime_input_from_parts",
        |_vehicle: &mut TVehicle,
         control: VehicleControlState,
         telemetry: VehicleTelemetrySnapshot,
         environment: VehicleEnvironmentBinding| {
            ShipRuntimeInput::from_parts_with_environment(control, telemetry, environment)
        },
    );
    engine.register_fn(
        "ship_runtime_input_from_parts",
        |_vehicle: &mut TVehicle,
         dt_s: rhai::FLOAT,
         control: VehicleControlState,
         telemetry: VehicleTelemetrySnapshot,
         environment: VehicleEnvironmentBinding| {
            ShipRuntimeInput::from_parts_with_environment(control, telemetry, environment)
                .with_dt_s(dt_s as f32)
        },
    );
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
        "packet_telemetry_from_snapshot",
        |_vehicle: &mut TVehicle, telemetry: VehicleTelemetrySnapshot| telemetry.to_packet(),
    );
    engine.register_fn(
        "packet_telemetry_from",
        |_vehicle: &mut TVehicle, args: RhaiMap| {
            rhai_map_to_typed_or_default::<VehiclePacketTelemetry>(args).normalized()
        },
    );

    engine.register_fn("environment_binding", |_vehicle: &mut TVehicle| {
        VehicleEnvironmentBinding::default().normalized()
    });
    engine.register_fn(
        "environment_binding_from",
        |_vehicle: &mut TVehicle, args: RhaiMap| {
            rhai_map_to_typed_or_default::<VehicleEnvironmentBinding>(args).normalized()
        },
    );
    engine.register_fn(
        "environment_binding_from_snapshot",
        |_vehicle: &mut TVehicle, snapshot: VehicleEnvironmentSnapshot| {
            VehicleEnvironmentBinding::from_snapshot(&snapshot)
        },
    );

    engine.register_fn("telemetry_snapshot", |_vehicle: &mut TVehicle| {
        VehicleTelemetrySnapshot::default().normalized()
    });
    engine.register_fn(
        "telemetry_snapshot_from",
        |_vehicle: &mut TVehicle, args: RhaiMap| {
            rhai_map_to_typed_or_default::<VehicleTelemetrySnapshot>(args).normalized()
        },
    );
    engine.register_fn(
        "telemetry_snapshot_from_packet",
        |_vehicle: &mut TVehicle, packet: VehiclePacketTelemetry| {
            VehicleTelemetrySnapshot::from_packet(&packet)
        },
    );
    engine.register_fn(
        "telemetry_snapshot_from_launch_packet",
        |_vehicle: &mut TVehicle, packet: VehicleLaunchPacket| {
            VehicleTelemetrySnapshot::from_launch_packet(&packet)
        },
    );
    engine.register_fn(
        "telemetry_snapshot_from_return_packet",
        |_vehicle: &mut TVehicle, packet: VehicleReturnPacket| {
            VehicleTelemetrySnapshot::from_return_packet(&packet)
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

    engine.register_fn("session_state", |_vehicle: &mut TVehicle| {
        VehicleSessionState::default().normalized()
    });
    engine.register_fn(
        "session_state_from",
        |_vehicle: &mut TVehicle, args: RhaiMap| {
            rhai_map_to_typed_or_default::<VehicleSessionState>(args).normalized()
        },
    );
    engine.register_fn(
        "session_state_from_parts",
        |_vehicle: &mut TVehicle,
         control: VehicleControlState,
         runtime: ShipRuntimeState,
         telemetry: VehicleTelemetrySnapshot| {
            VehicleSessionState::default()
                .with_control(control)
                .with_runtime(runtime)
                .with_telemetry(telemetry)
        },
    );
    engine.register_fn(
        "session_state_from_parts",
        |_vehicle: &mut TVehicle,
         control: VehicleControlState,
         runtime: ShipRuntimeState,
         telemetry: VehicleTelemetrySnapshot,
         environment: VehicleEnvironmentBinding| {
            VehicleSessionState::default()
                .with_control(control)
                .with_runtime(runtime)
                .with_telemetry(telemetry)
                .with_environment(environment)
        },
    );
    engine.register_fn(
        "session_state_from_launch_packet",
        |_vehicle: &mut TVehicle, packet: VehicleLaunchPacket| {
            VehicleSessionState::from_launch_packet(&packet)
        },
    );
    engine.register_fn(
        "session_state_from_return_packet",
        |_vehicle: &mut TVehicle, packet: VehicleReturnPacket| {
            VehicleSessionState::from_return_packet(&packet)
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

    #[test]
    fn vehicle_registration_supports_typed_runtime_flow_without_primary_map_roundtrip() {
        let world = GameplayWorld::new();
        let mut engine = RhaiEngine::new();
        register_vehicle_api(&mut engine);

        let mut scope = rhai::Scope::new();
        scope.push("vehicle", test_vehicle_api(Some(world)));

        let result: RhaiMap = engine
            .eval_with_scope(
                &mut scope,
                r#"
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
                                surface_radius: 212.0,
                                radius_px: 212.0,
                                gravity_mu_km3_s2: 4410.0
                            },
                            real_radius_km: 6371.0,
                            scale_divisor: 30.0,
                            surface_gravity_mps2: 9.81,
                            atmo_top_km: 80.0,
                            atmo_dense_start_km: 12.0,
                            atmo_drag_max: 2.0
                        },
                        vehicle: #{
                            profile_id: " sim_lite ",
                            assist_alt_hold: true,
                            spawn_altitude_km: 4.0
                        },
                        telemetry: #{
                            heading_deg: -90.0,
                            altitude_km: 4.0,
                            tangent_speed_kms: 1.25,
                            radial_speed_kms: -0.1,
                            spawn_angle_deg: -45.0,
                            grounded: true,
                            radius_wu: 216.0
                        }
                    });

                    let env = launch.environment_binding();
                    let telemetry = launch.telemetry_snapshot();
                    let control = launch.control_state();
                    let runtime = vehicle.ship_runtime_state_from_telemetry(telemetry);
                    let input = vehicle
                        .ship_runtime_input_from_parts(0.016, control, telemetry, env)
                        .with_surface_lock_request(true);
                    let step = vehicle.ship_runtime_step(" sim_lite ", runtime, input);
                    let session = vehicle
                        .session_state_from_launch_packet(launch)
                        .apply_runtime_output(step);

                    #{
                        env_body: env.body_id,
                        env_scale: env.scale_divisor,
                        env_gravity: env.surface_gravity_wu_s2(),
                        telemetry_heading: telemetry.heading_deg,
                        telemetry_grounded: telemetry.grounded,
                        runtime_mode: runtime.surface_mode,
                        input_dt: input.dt_s,
                        input_surface_lock: input.request_surface_lock,
                        step_mode: step.state.surface_mode,
                        step_radius: step.report.surface_radius_wu,
                        session_profile: session.control.profile_id,
                        session_spawn_altitude: session.spawn_altitude_km,
                        session_packet_profile: session.packet_vehicle().profile_id
                    }
                "#,
            )
            .expect("typed runtime flow should resolve without maps");

        let env_body = result
            .get("env_body")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("env_body");
        let env_scale = result
            .get("env_scale")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("env_scale");
        let env_gravity = result
            .get("env_gravity")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("env_gravity");
        let telemetry_heading = result
            .get("telemetry_heading")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("telemetry_heading");
        let telemetry_grounded = result
            .get("telemetry_grounded")
            .and_then(|value| value.clone().try_cast::<bool>())
            .expect("telemetry_grounded");
        let runtime_mode = result
            .get("runtime_mode")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("runtime_mode");
        let input_dt = result
            .get("input_dt")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("input_dt");
        let input_surface_lock = result
            .get("input_surface_lock")
            .and_then(|value| value.clone().try_cast::<bool>())
            .expect("input_surface_lock");
        let step_mode = result
            .get("step_mode")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("step_mode");
        let step_radius = result
            .get("step_radius")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("step_radius");
        let session_profile = result
            .get("session_profile")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("session_profile");
        let session_spawn_altitude = result
            .get("session_spawn_altitude")
            .and_then(|value| value.clone().try_cast::<rhai::FLOAT>())
            .expect("session_spawn_altitude");
        let session_packet_profile = result
            .get("session_packet_profile")
            .and_then(|value| value.clone().try_cast::<String>())
            .expect("session_packet_profile");

        assert_eq!(env_body, "generated-planet");
        assert_eq!(env_scale, 30.0);
        assert!(env_gravity > 0.0);
        assert_eq!(telemetry_heading, 270.0);
        assert!(telemetry_grounded);
        assert_eq!(runtime_mode, "grounded");
        assert!((input_dt - 0.016).abs() < 1.0e-6);
        assert!(input_surface_lock);
        assert_eq!(step_mode, "grounded");
        assert!(step_radius >= 0.0);
        assert_eq!(session_profile, "sim-lite");
        assert_eq!(session_spawn_altitude, 4.0);
        assert_eq!(session_packet_profile, "sim-lite");
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
