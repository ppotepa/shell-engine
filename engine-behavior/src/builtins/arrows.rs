//! Selected arrows behavior: flank the selected menu item with directional arrows.

use engine_core::scene::BehaviorParams;

use crate::{BehaviorCommand, Behavior, BehaviorContext, GameObject, Scene, emit_offset, emit_visibility, rounded_sine_wave};

/// Shows directional arrow sprites flanking the selected menu option.
pub struct SelectedArrowsBehavior {
    target: Option<String>,
    index: usize,
    side: ArrowSide,
    padding: i32,
    amplitude_x: i32,
    period_ms: u64,
    phase_ms: u64,
    autoscale_height: bool,
    pub last_dx: i32,
    pub last_dy: i32,
}

#[derive(Clone, Copy)]
enum ArrowSide {
    Left,
    Right,
}

impl SelectedArrowsBehavior {
    pub fn from_params(params: &BehaviorParams) -> Self {
        let side_str = params.side.as_deref().unwrap_or("");
        let side = if side_str.trim().eq_ignore_ascii_case("right") {
            ArrowSide::Right
        } else {
            ArrowSide::Left
        };
        Self {
            target: params.target.clone(),
            index: params.index.unwrap_or(0),
            side,
            padding: params.padding.unwrap_or(1),
            amplitude_x: params.amplitude_x.unwrap_or(1).abs(),
            period_ms: params.period_ms.unwrap_or(900).max(1),
            phase_ms: params.phase_ms.unwrap_or(0),
            autoscale_height: params.autoscale_height.unwrap_or(false),
            last_dx: 0,
            last_dy: 0,
        }
    }

    fn hide_and_reset(&mut self, object: &GameObject, commands: &mut Vec<BehaviorCommand>) {
        self.last_dx = 0;
        self.last_dy = 0;
        emit_visibility(commands, object.id.clone(), false);
    }
}

impl Behavior for SelectedArrowsBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        _scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        if ctx.menu_selected_index != self.index {
            self.hide_and_reset(object, commands);
            return;
        }

        let Some(target_alias) = self.target.as_deref() else {
            self.hide_and_reset(object, commands);
            return;
        };
        let Some(target_region) = ctx.resolved_object_region(target_alias) else {
            self.hide_and_reset(object, commands);
            return;
        };
        let Some(own_region) = ctx.object_region(&object.id) else {
            emit_visibility(commands, object.id.clone(), true);
            // First frame after becoming visible: wait for compositor to discover own region.
            return;
        };

        let wave = rounded_sine_wave(ctx.scene_elapsed_ms, self.phase_ms, self.period_ms);
        let signed_wave = match self.side {
            ArrowSide::Left => wave,
            ArrowSide::Right => -wave,
        } * self.amplitude_x;
        let auto_pad = if self.autoscale_height {
            (target_region.height.saturating_sub(1) as i32) / 2
        } else {
            0
        };
        let effective_padding = self.padding.saturating_add(auto_pad).max(0);
        let arrow_w = own_region.width.max(1) as i32;
        let arrow_h = own_region.height.max(1) as i32;
        let target_w = target_region.width.max(1) as i32;
        let target_center_y =
            target_region.y as i32 + (target_region.height.saturating_sub(1) as i32 / 2);

        let target_x = match self.side {
            ArrowSide::Left => target_region.x as i32 - effective_padding - arrow_w + signed_wave,
            ArrowSide::Right => target_region.x as i32 + target_w + effective_padding + signed_wave,
        };
        let target_y = target_center_y.saturating_sub((arrow_h.saturating_sub(1)) / 2);

        emit_visibility(commands, object.id.clone(), true);

        let base_x = own_region.x as i32 - self.last_dx;
        let base_y = own_region.y as i32 - self.last_dy;
        let new_dx = target_x - base_x;
        let new_dy = target_y - base_y;
        self.last_dx = new_dx;
        self.last_dy = new_dy;

        emit_offset(commands, object.id.clone(), new_dx, new_dy);
    }
}
