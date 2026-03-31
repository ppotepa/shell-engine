//! Menu behaviors: selection highlighting, carousel scrolling, and object grouping.

use std::collections::BTreeMap;

use engine_core::scene::BehaviorParams;

use crate::{
    emit_offset, emit_visibility, resolve_target, wrapped_menu_distance, Behavior, BehaviorCommand,
    BehaviorContext, GameObject, Scene,
};

/// Shows the object only while it is the currently selected menu option.
pub struct MenuSelectedBehavior {
    target: Option<String>,
    index: usize,
}

impl MenuSelectedBehavior {
    pub fn from_params(params: &BehaviorParams) -> Self {
        Self {
            target: params.target.clone(),
            index: params.index.unwrap_or(0),
        }
    }
}

impl Behavior for MenuSelectedBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        _scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        emit_visibility(
            commands,
            resolve_target(&self.target, object),
            ctx.menu_selected_index == self.index,
        );
    }
}

/// Repositions menu items into a centered rolling window around selected index.
pub struct MenuCarouselBehavior {
    target: Option<String>,
    index: usize,
    count: Option<usize>,
    window: usize,
    step_y: i32,
    endless: bool,
    last_dy: i32,
}

impl MenuCarouselBehavior {
    pub fn from_params(params: &BehaviorParams) -> Self {
        Self {
            target: params.target.clone(),
            index: params.index.unwrap_or(0),
            count: params.count,
            window: params.window.unwrap_or(5).max(1),
            step_y: params.step_y.unwrap_or(2).max(1),
            endless: params.endless.unwrap_or(true),
            last_dy: 0,
        }
    }

    fn hide_and_reset(&mut self, object: &GameObject, commands: &mut Vec<BehaviorCommand>) {
        self.last_dy = 0;
        emit_visibility(commands, object.id.clone(), false);
    }
}

impl Behavior for MenuCarouselBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        let total = self.count.unwrap_or(scene.menu_options.len());
        if total == 0 || self.index >= total {
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

        let selected = ctx.menu_selected_index % total;
        let relative = if self.endless {
            wrapped_menu_distance(self.index, selected, total)
        } else {
            self.index as i32 - selected as i32
        };
        let half_window = ((self.window.saturating_sub(1)) / 2) as i32;
        if relative.abs() > half_window {
            self.hide_and_reset(object, commands);
            return;
        }

        emit_visibility(commands, object.id.clone(), true);

        let Some(own_region) = ctx.object_region(&object.id) else {
            // First frame after becoming visible: wait for compositor to discover own region.
            return;
        };

        // Keep menu items from collapsing into each other when authored `step_y`
        // is too small for the current rendered item height.
        let item_height = own_region.height.max(1) as i32;
        let effective_step_y = self.step_y.max(item_height.saturating_add(1));
        let center_y = target_region.y as i32 + (target_region.height.saturating_sub(1) as i32 / 2);
        let desired_y = center_y.saturating_add(relative.saturating_mul(effective_step_y));
        let base_y = own_region.y as i32 - self.last_dy;
        let new_dy = desired_y - base_y;
        self.last_dy = new_dy;
        emit_offset(commands, object.id.clone(), 0, new_dy);
    }
}

/// Repositions a group of menu items from one controller behavior attached to
/// the parent object/layer.
pub struct MenuCarouselObjectBehavior {
    target: Option<String>,
    item_prefix: String,
    count: Option<usize>,
    window: usize,
    step_y: i32,
    endless: bool,
    last_dy_by_index: BTreeMap<usize, i32>,
}

impl MenuCarouselObjectBehavior {
    pub fn from_params(params: &BehaviorParams) -> Self {
        Self {
            target: params.target.clone(),
            item_prefix: params
                .item_prefix
                .clone()
                .unwrap_or_else(|| "menu-item-".to_string()),
            count: params.count,
            window: params.window.unwrap_or(5).max(1),
            step_y: params.step_y.unwrap_or(2).max(1),
            endless: params.endless.unwrap_or(true),
            last_dy_by_index: BTreeMap::new(),
        }
    }

    fn item_alias(&self, index: usize) -> String {
        if self.item_prefix.contains("{}") {
            self.item_prefix.replace("{}", &index.to_string())
        } else {
            format!("{}{}", self.item_prefix, index)
        }
    }
}

impl Behavior for MenuCarouselObjectBehavior {
    fn update(
        &mut self,
        _object: &GameObject,
        scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        let total = self.count.unwrap_or(scene.menu_options.len());
        if total == 0 {
            self.last_dy_by_index.clear();
            return;
        }
        let Some(target_alias) = self.target.as_deref() else {
            self.last_dy_by_index.clear();
            return;
        };
        let Some(target_region) = ctx.resolved_object_region(target_alias) else {
            self.last_dy_by_index.clear();
            return;
        };

        let selected = ctx.menu_selected_index % total;
        let half_window = ((self.window.saturating_sub(1)) / 2) as i32;
        for index in 0..total {
            let item_alias = self.item_alias(index);
            let relative = if self.endless {
                wrapped_menu_distance(index, selected, total)
            } else {
                index as i32 - selected as i32
            };
            if relative.abs() > half_window {
                self.last_dy_by_index.insert(index, 0);
                emit_visibility(commands, item_alias, false);
                continue;
            }
            emit_visibility(commands, item_alias.clone(), true);

            let Some(item_region) = ctx.resolved_object_region(&item_alias) else {
                // First visible frame can happen before compositor reports regions.
                continue;
            };
            let last_dy = self.last_dy_by_index.get(&index).copied().unwrap_or(0);
            let item_height = item_region.height.max(1) as i32;
            let effective_step_y = self.step_y.max(item_height.saturating_add(1));
            let center_y =
                target_region.y as i32 + (target_region.height.saturating_sub(1) as i32 / 2);
            let desired_y = center_y.saturating_add(relative.saturating_mul(effective_step_y));
            let base_y = item_region.y as i32 - last_dy;
            let new_dy = desired_y - base_y;
            self.last_dy_by_index.insert(index, new_dy);
            emit_offset(commands, item_alias, 0, new_dy);
        }
    }
}
