use super::*;

impl SceneRuntime {
    pub fn text_sprite_content(&self, sprite_id: &str) -> Option<&str> {
        if let Some(&layer_idx) = self.sprite_id_to_layer.get(sprite_id) {
            if let Some(layer) = self.scene.layers.get(layer_idx) {
                if let Some(content) = find_text_content(&layer.sprites, sprite_id) {
                    return Some(content);
                }
            }
        }
        for layer in &self.scene.layers {
            if let Some(content) = find_text_content(&layer.sprites, sprite_id) {
                return Some(content);
            }
        }
        None
    }

    pub fn object_text_snapshot(&mut self) -> std::sync::Arc<HashMap<String, String>> {
        if let Some(cached) = &self.cached_object_text {
            if self.cached_object_text_gen == self.object_mutation_gen {
                return std::sync::Arc::clone(cached);
            }
        }
        let mut out = HashMap::new();
        for (object_id, object) in &self.objects {
            let Some(sprite_id) = object.aliases.first() else {
                continue;
            };
            let Some(content) = self.text_sprite_content(sprite_id) else {
                continue;
            };
            out.insert(object_id.clone(), content.to_string());
        }
        let arc = std::sync::Arc::new(out);
        self.cached_object_text = Some(std::sync::Arc::clone(&arc));
        self.cached_object_text_gen = self.object_mutation_gen;
        arc
    }

    pub fn object_props_snapshot(&mut self) -> std::sync::Arc<HashMap<String, JsonValue>> {
        if let Some(cached) = &self.cached_object_props {
            if self.cached_object_props_gen == self.object_mutation_gen {
                return std::sync::Arc::clone(cached);
            }
        }
        let mut out = HashMap::new();
        for (object_id, object) in &self.objects {
            let Some(sprite_id) = object.aliases.first() else {
                continue;
            };
            let mut props = JsonMap::new();
            if let Some((font, fg, bg)) = self.text_sprite_style(sprite_id) {
                let mut text = JsonMap::new();
                if let Some(value) = font {
                    text.insert("font".to_string(), JsonValue::String(value));
                }
                if let Some(value) = fg {
                    text.insert("fg".to_string(), term_colour_to_json(&value));
                }
                if let Some(value) = bg {
                    text.insert("bg".to_string(), term_colour_to_json(&value));
                }
                if !text.is_empty() {
                    props.insert("text".to_string(), JsonValue::Object(text.clone()));
                    props.insert("style".to_string(), JsonValue::Object(text));
                }
            }
            if let Some(obj) = self.obj_sprite_properties(sprite_id) {
                let mut obj_props = JsonMap::new();
                if let Some(value) = obj.scale {
                    obj_props.insert("scale".to_string(), JsonValue::from(value));
                }
                if let Some(value) = obj.yaw {
                    obj_props.insert("yaw".to_string(), JsonValue::from(value));
                }
                if let Some(value) = obj.pitch {
                    obj_props.insert("pitch".to_string(), JsonValue::from(value));
                }
                if let Some(value) = obj.roll {
                    obj_props.insert("roll".to_string(), JsonValue::from(value));
                }
                if let Some(value) = obj.orbit_speed {
                    obj_props.insert("orbit_speed".to_string(), JsonValue::from(value));
                }
                if let Some(value) = obj.surface_mode {
                    obj_props.insert("surface_mode".to_string(), JsonValue::String(value));
                }
                if !obj_props.is_empty() {
                    props.insert("obj".to_string(), JsonValue::Object(obj_props));
                }
            }
            if !props.is_empty() {
                out.insert(object_id.clone(), JsonValue::Object(props));
            }
        }
        let arc = std::sync::Arc::new(out);
        self.cached_object_props = Some(std::sync::Arc::clone(&arc));
        self.cached_object_props_gen = self.object_mutation_gen;
        arc
    }

    fn text_sprite_style(
        &self,
        sprite_id: &str,
    ) -> Option<(Option<String>, Option<TermColour>, Option<TermColour>)> {
        self.scene
            .layers
            .iter()
            .find_map(|layer| find_text_style_recursive(&layer.sprites, sprite_id))
    }

    fn obj_sprite_properties(&self, sprite_id: &str) -> Option<ObjSpritePropertySnapshot> {
        self.scene
            .layers
            .iter()
            .find_map(|layer| find_obj_properties_recursive(&layer.sprites, sprite_id))
    }

    pub(crate) fn set_text_sprite_content(
        &mut self,
        sprite_id: &str,
        next_content: String,
    ) -> bool {
        let mut updated = false;
        for layer in &mut self.scene.layers {
            set_text_content_recursive(&mut layer.sprites, sprite_id, &next_content, &mut updated);
        }
        updated
    }

    pub(crate) fn set_text_sprite_font(&mut self, sprite_id: &str, next_font: String) -> bool {
        let mut updated = false;
        for layer in &mut self.scene.layers {
            set_text_font_recursive(&mut layer.sprites, sprite_id, &next_font, &mut updated);
        }
        updated
    }

    pub(crate) fn set_text_sprite_fg_colour(
        &mut self,
        sprite_id: &str,
        next_colour: TermColour,
    ) -> bool {
        let mut updated = false;
        if let Some(&layer_idx) = self.sprite_id_to_layer.get(sprite_id) {
            if let Some(layer) = self.scene.layers.get_mut(layer_idx) {
                set_text_fg_recursive(&mut layer.sprites, sprite_id, &next_colour, &mut updated);
                if updated { return true; }
            }
        }
        for layer in &mut self.scene.layers {
            set_text_fg_recursive(&mut layer.sprites, sprite_id, &next_colour, &mut updated);
            if updated { return true; }
        }
        false
    }

    pub(crate) fn set_text_sprite_bg_colour(
        &mut self,
        sprite_id: &str,
        next_colour: TermColour,
    ) -> bool {
        let mut updated = false;
        if let Some(&layer_idx) = self.sprite_id_to_layer.get(sprite_id) {
            if let Some(layer) = self.scene.layers.get_mut(layer_idx) {
                set_text_bg_recursive(&mut layer.sprites, sprite_id, &next_colour, &mut updated);
                if updated { return true; }
            }
        }
        for layer in &mut self.scene.layers {
            set_text_bg_recursive(&mut layer.sprites, sprite_id, &next_colour, &mut updated);
            if updated { return true; }
        }
        false
    }

    pub(crate) fn set_obj_sprite_property(
        &mut self,
        sprite_id: &str,
        path: &str,
        value: &JsonValue,
    ) -> bool {
        let mut updated = false;
        if let Some(&layer_idx) = self.sprite_id_to_layer.get(sprite_id) {
            if let Some(layer) = self.scene.layers.get_mut(layer_idx) {
                set_obj_property_recursive(&mut layer.sprites, sprite_id, path, value, &mut updated);
                if updated { return true; }
            }
        }
        for layer in &mut self.scene.layers {
            set_obj_property_recursive(&mut layer.sprites, sprite_id, path, value, &mut updated);
            if updated { return true; }
        }
        false
    }

    pub(crate) fn set_vector_sprite_property(
        &mut self,
        sprite_id: &str,
        path: &str,
        value: &JsonValue,
    ) -> bool {
        let mut updated = false;
        // Fast path: use sprite_id_to_layer index for O(1) layer lookup
        if let Some(&layer_idx) = self.sprite_id_to_layer.get(sprite_id) {
            if let Some(layer) = self.scene.layers.get_mut(layer_idx) {
                set_vector_property_recursive(&mut layer.sprites, sprite_id, path, value, &mut updated);
                if updated {
                    return true;
                }
            }
        }
        // Fallback: scan all layers (handles unindexed sprites)
        for layer in &mut self.scene.layers {
            set_vector_property_recursive(&mut layer.sprites, sprite_id, path, value, &mut updated);
            if updated {
                return true;
            }
        }
        false
    }

    pub(crate) fn set_scene3d_sprite_frame(&mut self, sprite_id: &str, next_frame: &str) -> bool {
        let mut updated = false;
        if let Some(&layer_idx) = self.sprite_id_to_layer.get(sprite_id) {
            if let Some(layer) = self.scene.layers.get_mut(layer_idx) {
                set_scene3d_frame_recursive(&mut layer.sprites, sprite_id, next_frame, &mut updated);
                if updated { return true; }
            }
        }
        for layer in &mut self.scene.layers {
            set_scene3d_frame_recursive(&mut layer.sprites, sprite_id, next_frame, &mut updated);
            if updated { return true; }
        }
        false
    }

    pub(crate) fn set_image_sprite_frame_index(
        &mut self,
        sprite_id: &str,
        next_frame: u16,
    ) -> bool {
        let mut updated = false;
        if let Some(&layer_idx) = self.sprite_id_to_layer.get(sprite_id) {
            if let Some(layer) = self.scene.layers.get_mut(layer_idx) {
                set_image_frame_index_recursive(&mut layer.sprites, sprite_id, next_frame, &mut updated);
                if updated { return true; }
            }
        }
        for layer in &mut self.scene.layers {
            set_image_frame_index_recursive(
                &mut layer.sprites,
                sprite_id,
                next_frame,
                &mut updated,
            );
            if updated { return true; }
        }
        false
    }

    pub(crate) fn object_alias_candidates(&self, object_id: &str, target: &str) -> Vec<String> {
        let mut out = vec![target.to_string()];
        if let Some(object) = self.objects.get(object_id) {
            for alias in &object.aliases {
                if alias.trim().is_empty() || out.iter().any(|current| current == alias) {
                    continue;
                }
                out.push(alias.clone());
            }
        }
        out
    }

    pub(crate) fn apply_text_property_for_target(
        &mut self,
        object_id: &str,
        target: &str,
        mut apply: impl FnMut(&mut Self, &str) -> bool,
    ) -> bool {
        for alias in self.object_alias_candidates(object_id, target) {
            if apply(self, &alias) {
                return true;
            }
        }
        false
    }

    // =========================================================================
    // Direct particle mutation — bypasses BehaviorCommand pipeline entirely
    // =========================================================================

    /// Directly apply position (x, y) and heading updates for entities with
    /// visual bindings.  Each entry is `(visual_id, x, y, heading)`.
    ///
    /// This is the zero-allocation fast path for `visual_sync_system`:
    /// - No BehaviorCommand creation (no String clone/alloc for target/path)
    /// - No JsonValue allocation
    /// - No resolve_alias (we resolve once via the Arc resolver)
    /// - Heading child cascade uses index iteration (no Vec<String> clone)
    pub fn apply_particle_visual_sync(&mut self, sync_data: &[(String, f32, f32, f32)]) {
        if sync_data.is_empty() {
            return;
        }
        self.effective_states_dirty = true;
        self.object_mutation_gen = self.object_mutation_gen.wrapping_add(1);
        self.cached_object_states = None;
        self.cached_object_props = None;

        let resolver = std::sync::Arc::clone(&self.resolver_cache);

        for (visual_id, x, y, heading) in sync_data {
            let Some(object_id) = resolver.resolve_alias(visual_id) else {
                continue;
            };

            // Round (not truncate) to the nearest pixel — truncation causes ±1px
            // jitter on fast-moving entities when the fractional part crosses 0.5.
            if let Some(state) = self.object_states.get_mut(object_id) {
                state.offset_x = x.round() as i32;
                state.offset_y = y.round() as i32;
                state.heading = *heading;
            }

            // Cascade heading to child sprites (avoid Vec clone).
            // Particles are typically single-sprite layers, so children.len() == 1.
            if let Some(obj) = self.objects.get(object_id) {
                if matches!(obj.kind, GameObjectKind::Layer) {
                    // Index-based iteration: borrow children slice, then mutate states.
                    let n = obj.children.len();
                    for i in 0..n {
                        let child_id = &self.objects.get(object_id).unwrap().children[i];
                        // Need to clone the child_id to satisfy borrow checker
                        // (objects is borrowed immutably for child_id, states mutably).
                        // For single-child particles this is one clone vs N clones before.
                        let cid = child_id.clone();
                        if let Some(state) = self.object_states.get_mut(&cid) {
                            state.heading = *heading;
                        }
                    }
                }
            }
        }
    }

    /// Directly apply color ramp and radius updates for particle visuals.
    /// Each entry is `(visual_id, colour_str, radius)`.
    ///
    /// This is the zero-allocation fast path for `particle_ramp_system`:
    /// - No BehaviorCommand creation
    /// - No JsonValue round-trip (typed TermColour + direct points mutation)
    /// - No object_alias_candidates fallback (particles always hit sprite_id_to_layer)
    pub fn apply_particle_ramps(&mut self, ramp_data: &[(String, String, i32)]) {
        if ramp_data.is_empty() {
            return;
        }
        self.effective_states_dirty = true;
        self.object_mutation_gen = self.object_mutation_gen.wrapping_add(1);
        self.cached_object_states = None;
        self.cached_object_props = None;

        for (visual_id, colour_str, radius) in ramp_data {
            let r = (*radius).max(0);
            let next_points = vec![[0, 0], [r, 0]];
            let next_colour = engine_core::scene::color::parse_colour_str(colour_str);

            // Fast path: use sprite_id_to_layer index for O(1) lookup.
            // Particles are always indexed, so fallback scan is not needed.
            if let Some(&layer_idx) = self.sprite_id_to_layer.get(visual_id.as_str()) {
                if let Some(layer) = self.scene.layers.get_mut(layer_idx) {
                    set_particle_ramp_recursive(
                        &mut layer.sprites,
                        visual_id,
                        next_colour.as_ref(),
                        &next_points,
                    );
                }
            }
        }
    }
}

fn find_text_content<'a>(sprites: &'a [Sprite], sprite_id: &str) -> Option<&'a str> {
    for sprite in sprites {
        match sprite {
            Sprite::Text {
                id: Some(id),
                content,
                ..
            } if id == sprite_id => return Some(content.as_str()),
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                if let Some(content) = find_text_content(children, sprite_id) {
                    return Some(content);
                }
            }
            _ => {}
        }
    }
    None
}

pub(crate) fn find_text_layout_recursive(
    sprites: &[Sprite],
    sprite_id: &str,
) -> Option<TextLayoutSpec> {
    for sprite in sprites {
        match sprite {
            Sprite::Text {
                id: Some(id),
                x,
                y,
                font,
                ..
            } if id == sprite_id => {
                return Some(TextLayoutSpec {
                    x: *x,
                    y: *y,
                    font: font.clone(),
                });
            }
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                if let Some(layout) = find_text_layout_recursive(children, sprite_id) {
                    return Some(layout);
                }
            }
            _ => {}
        }
    }
    None
}

fn find_text_style_recursive(
    sprites: &[Sprite],
    sprite_id: &str,
) -> Option<(Option<String>, Option<TermColour>, Option<TermColour>)> {
    for sprite in sprites {
        match sprite {
            Sprite::Text {
                id: Some(id),
                font,
                fg_colour,
                bg_colour,
                ..
            } if id == sprite_id => {
                return Some((font.clone(), fg_colour.clone(), bg_colour.clone()));
            }
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                if let Some(style) = find_text_style_recursive(children, sprite_id) {
                    return Some(style);
                }
            }
            _ => {}
        }
    }
    None
}

fn find_obj_properties_recursive(
    sprites: &[Sprite],
    sprite_id: &str,
) -> Option<ObjSpritePropertySnapshot> {
    for sprite in sprites {
        match sprite {
            Sprite::Obj {
                id: Some(id),
                scale,
                yaw_deg,
                pitch_deg,
                roll_deg,
                rotate_y_deg_per_sec,
                surface_mode,
                clip_y_min,
                clip_y_max,
                ..
            } if id == sprite_id => {
                return Some(ObjSpritePropertySnapshot {
                    scale: *scale,
                    yaw: *yaw_deg,
                    pitch: *pitch_deg,
                    roll: *roll_deg,
                    orbit_speed: *rotate_y_deg_per_sec,
                    surface_mode: surface_mode.clone(),
                    clip_y_min: *clip_y_min,
                    clip_y_max: *clip_y_max,
                });
            }
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                if let Some(props) = find_obj_properties_recursive(children, sprite_id) {
                    return Some(props);
                }
            }
            _ => {}
        }
    }
    None
}

fn set_text_content_recursive(
    sprites: &mut [Sprite],
    sprite_id: &str,
    next_content: &str,
    updated: &mut bool,
) {
    for sprite in sprites.iter_mut() {
        match sprite {
            Sprite::Text {
                id: Some(id),
                content,
                ..
            } if id == sprite_id => {
                if content.as_str() != next_content {
                    *content = next_content.to_string();
                    *updated = true;
                }
            }
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                set_text_content_recursive(children, sprite_id, next_content, updated)
            }
            _ => {}
        }
    }
}

fn set_text_font_recursive(
    sprites: &mut [Sprite],
    sprite_id: &str,
    next_font: &str,
    updated: &mut bool,
) {
    for sprite in sprites.iter_mut() {
        match sprite {
            Sprite::Text {
                id: Some(id), font, ..
            } if id == sprite_id => {
                if font.as_deref() != Some(next_font) {
                    *font = Some(next_font.to_string());
                    *updated = true;
                }
            }
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                set_text_font_recursive(children, sprite_id, next_font, updated)
            }
            _ => {}
        }
    }
}

fn set_text_fg_recursive(
    sprites: &mut [Sprite],
    sprite_id: &str,
    next_colour: &TermColour,
    updated: &mut bool,
) {
    for sprite in sprites.iter_mut() {
        match sprite {
            Sprite::Text {
                id: Some(id),
                fg_colour,
                ..
            } if id == sprite_id => {
                if fg_colour.as_ref() != Some(next_colour) {
                    *fg_colour = Some(next_colour.clone());
                    *updated = true;
                }
            }
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                set_text_fg_recursive(children, sprite_id, next_colour, updated)
            }
            _ => {}
        }
    }
}

fn set_text_bg_recursive(
    sprites: &mut [Sprite],
    sprite_id: &str,
    next_colour: &TermColour,
    updated: &mut bool,
) {
    for sprite in sprites.iter_mut() {
        match sprite {
            Sprite::Text {
                id: Some(id),
                bg_colour,
                ..
            } if id == sprite_id => {
                if bg_colour.as_ref() != Some(next_colour) {
                    *bg_colour = Some(next_colour.clone());
                    *updated = true;
                }
            }
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                set_text_bg_recursive(children, sprite_id, next_colour, updated)
            }
            _ => {}
        }
    }
}

fn set_image_frame_index_recursive(
    sprites: &mut [Sprite],
    sprite_id: &str,
    next_frame: u16,
    updated: &mut bool,
) {
    for sprite in sprites.iter_mut() {
        match sprite {
            Sprite::Image {
                id: Some(id),
                frame_index,
                ..
            } if id == sprite_id => {
                if frame_index.unwrap_or(0) != next_frame {
                    *frame_index = Some(next_frame);
                    *updated = true;
                }
            }
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                set_image_frame_index_recursive(children, sprite_id, next_frame, updated)
            }
            _ => {}
        }
    }
}

fn set_scene3d_frame_recursive(
    sprites: &mut [Sprite],
    sprite_id: &str,
    next_frame: &str,
    updated: &mut bool,
) {
    for sprite in sprites.iter_mut() {
        match sprite {
            Sprite::Scene3D {
                id: Some(id),
                frame,
                ..
            } if id == sprite_id => {
                if frame != next_frame {
                    *frame = next_frame.to_string();
                    *updated = true;
                }
            }
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                set_scene3d_frame_recursive(children, sprite_id, next_frame, updated);
            }
            _ => {}
        }
    }
}

fn set_obj_property_recursive(
    sprites: &mut [Sprite],
    sprite_id: &str,
    path: &str,
    value: &JsonValue,
    updated: &mut bool,
) {
    for sprite in sprites.iter_mut() {
        match sprite {
            Sprite::Obj {
                id: Some(id),
                scale,
                yaw_deg,
                pitch_deg,
                roll_deg,
                rotate_y_deg_per_sec,
                surface_mode,
                clip_y_min,
                clip_y_max,
                cam_world_x,
                cam_world_y,
                cam_world_z,
                view_right_x,
                view_right_y,
                view_right_z,
                view_up_x,
                view_up_y,
                view_up_z,
                view_fwd_x,
                view_fwd_y,
                view_fwd_z,
                ..
            } if id == sprite_id => match path {
                "obj.scale" => {
                    let Some(next) = json_value_to_f32(value) else {
                        continue;
                    };
                    let next = next.clamp(0.1, 8.0);
                    if (scale.unwrap_or(1.0) - next).abs() > f32::EPSILON {
                        *scale = Some(next);
                        *updated = true;
                    }
                }
                "obj.yaw" => {
                    let Some(next) = json_value_to_f32(value) else {
                        continue;
                    };
                    if (yaw_deg.unwrap_or(0.0) - next).abs() > f32::EPSILON {
                        *yaw_deg = Some(next);
                        *updated = true;
                    }
                }
                "obj.pitch" => {
                    let Some(next) = json_value_to_f32(value) else {
                        continue;
                    };
                    if (pitch_deg.unwrap_or(0.0) - next).abs() > f32::EPSILON {
                        *pitch_deg = Some(next);
                        *updated = true;
                    }
                }
                "obj.roll" => {
                    let Some(next) = json_value_to_f32(value) else {
                        continue;
                    };
                    if (roll_deg.unwrap_or(0.0) - next).abs() > f32::EPSILON {
                        *roll_deg = Some(next);
                        *updated = true;
                    }
                }
                "obj.orbit_speed" => {
                    let Some(next) = json_value_to_f32(value) else {
                        continue;
                    };
                    if (rotate_y_deg_per_sec.unwrap_or(0.0) - next).abs() > f32::EPSILON {
                        *rotate_y_deg_per_sec = Some(next);
                        *updated = true;
                    }
                }
                "obj.surface_mode" => {
                    let Some(next) = value.as_str() else {
                        continue;
                    };
                    if surface_mode.as_deref() != Some(next) {
                        *surface_mode = Some(next.to_string());
                        *updated = true;
                    }
                }
                "obj.clip_y_min" => {
                    let Some(next) = json_value_to_f32(value) else {
                        continue;
                    };
                    let next = next.clamp(0.0, 1.0);
                    if (clip_y_min.unwrap_or(0.0) - next).abs() > f32::EPSILON {
                        *clip_y_min = Some(next);
                        *updated = true;
                    }
                }
                "obj.clip_y_max" => {
                    let Some(next) = json_value_to_f32(value) else {
                        continue;
                    };
                    let next = next.clamp(0.0, 1.0);
                    if (clip_y_max.unwrap_or(1.0) - next).abs() > f32::EPSILON {
                        *clip_y_max = Some(next);
                        *updated = true;
                    }
                }
                // ── Cockpit camera world position ──────────────────────────
                "obj.cam.wx" => {
                    if let Some(v) = json_value_to_f32(value) {
                        if cam_world_x.map_or(true, |c| (c - v).abs() > f32::EPSILON) {
                            *cam_world_x = Some(v);
                            *updated = true;
                        }
                    }
                }
                "obj.cam.wy" => {
                    if let Some(v) = json_value_to_f32(value) {
                        if cam_world_y.map_or(true, |c| (c - v).abs() > f32::EPSILON) {
                            *cam_world_y = Some(v);
                            *updated = true;
                        }
                    }
                }
                "obj.cam.wz" => {
                    if let Some(v) = json_value_to_f32(value) {
                        if cam_world_z.map_or(true, |c| (c - v).abs() > f32::EPSILON) {
                            *cam_world_z = Some(v);
                            *updated = true;
                        }
                    }
                }
                // ── Cockpit camera view basis ──────────────────────────────
                "obj.view.rx" => {
                    if let Some(v) = json_value_to_f32(value) {
                        if view_right_x.map_or(true, |c| (c - v).abs() > f32::EPSILON) {
                            *view_right_x = Some(v);
                            *updated = true;
                        }
                    }
                }
                "obj.view.ry" => {
                    if let Some(v) = json_value_to_f32(value) {
                        if view_right_y.map_or(true, |c| (c - v).abs() > f32::EPSILON) {
                            *view_right_y = Some(v);
                            *updated = true;
                        }
                    }
                }
                "obj.view.rz" => {
                    if let Some(v) = json_value_to_f32(value) {
                        if view_right_z.map_or(true, |c| (c - v).abs() > f32::EPSILON) {
                            *view_right_z = Some(v);
                            *updated = true;
                        }
                    }
                }
                "obj.view.ux" => {
                    if let Some(v) = json_value_to_f32(value) {
                        if view_up_x.map_or(true, |c| (c - v).abs() > f32::EPSILON) {
                            *view_up_x = Some(v);
                            *updated = true;
                        }
                    }
                }
                "obj.view.uy" => {
                    if let Some(v) = json_value_to_f32(value) {
                        if view_up_y.map_or(true, |c| (c - v).abs() > f32::EPSILON) {
                            *view_up_y = Some(v);
                            *updated = true;
                        }
                    }
                }
                "obj.view.uz" => {
                    if let Some(v) = json_value_to_f32(value) {
                        if view_up_z.map_or(true, |c| (c - v).abs() > f32::EPSILON) {
                            *view_up_z = Some(v);
                            *updated = true;
                        }
                    }
                }
                "obj.view.fx" => {
                    if let Some(v) = json_value_to_f32(value) {
                        if view_fwd_x.map_or(true, |c| (c - v).abs() > f32::EPSILON) {
                            *view_fwd_x = Some(v);
                            *updated = true;
                        }
                    }
                }
                "obj.view.fy" => {
                    if let Some(v) = json_value_to_f32(value) {
                        if view_fwd_y.map_or(true, |c| (c - v).abs() > f32::EPSILON) {
                            *view_fwd_y = Some(v);
                            *updated = true;
                        }
                    }
                }
                "obj.view.fz" => {
                    if let Some(v) = json_value_to_f32(value) {
                        if view_fwd_z.map_or(true, |c| (c - v).abs() > f32::EPSILON) {
                            *view_fwd_z = Some(v);
                            *updated = true;
                        }
                    }
                }
                _ => {}
            },
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                set_obj_property_recursive(children, sprite_id, path, value, updated);
            }
            _ => {}
        }
    }
}

fn set_vector_property_recursive(
    sprites: &mut [Sprite],
    sprite_id: &str,
    path: &str,
    value: &JsonValue,
    updated: &mut bool,
) {
    for sprite in sprites.iter_mut() {
        match sprite {
            Sprite::Vector {
                id: Some(id),
                points,
                closed,
                draw_char,
                fg_colour,
                bg_colour,
                ..
            } if id == sprite_id => match path {
                "vector.points" => {
                    let Some(next_points) = json_to_vector_points(value) else {
                        continue;
                    };
                    if *points != next_points {
                        *points = next_points;
                        *updated = true;
                    }
                }
                "vector.closed" => {
                    let Some(next_closed) = value.as_bool() else {
                        continue;
                    };
                    if *closed != next_closed {
                        *closed = next_closed;
                        *updated = true;
                    }
                }
                "vector.draw_char" => {
                    let Some(raw) = value.as_str() else {
                        continue;
                    };
                    let next_draw = raw.chars().next().map(|ch| ch.to_string());
                    if *draw_char != next_draw {
                        *draw_char = next_draw;
                        *updated = true;
                    }
                }
                "vector.fg" | "style.fg" => {
                    let Some(next_colour) = parse_term_colour(value) else {
                        continue;
                    };
                    if fg_colour.as_ref() != Some(&next_colour) {
                        *fg_colour = Some(next_colour);
                        *updated = true;
                    }
                }
                "vector.bg" | "style.bg" => {
                    let Some(next_colour) = parse_term_colour(value) else {
                        continue;
                    };
                    if bg_colour.as_ref() != Some(&next_colour) {
                        *bg_colour = Some(next_colour);
                        *updated = true;
                    }
                }
                _ => {}
            },
            Sprite::Grid { children, .. } | Sprite::Flex { children, .. } => {
                set_vector_property_recursive(children, sprite_id, path, value, updated);
            }
            Sprite::Panel {
                id,
                fg_colour,
                bg_colour,
                border_colour,
                shadow_colour,
                children,
                ..
            } => {
                if id.as_deref() == Some(sprite_id) {
                    match path {
                        "style.fg" => {
                            if let Some(next_colour) = parse_term_colour(value) {
                                if fg_colour.as_ref() != Some(&next_colour) {
                                    *fg_colour = Some(next_colour);
                                    *updated = true;
                                }
                            }
                        }
                        "style.bg" => {
                            if let Some(next_colour) = parse_term_colour(value) {
                                if bg_colour.as_ref() != Some(&next_colour) {
                                    *bg_colour = Some(next_colour);
                                    *updated = true;
                                }
                            }
                        }
                        "style.border" => {
                            if let Some(next_colour) = parse_term_colour(value) {
                                if border_colour.as_ref() != Some(&next_colour) {
                                    *border_colour = Some(next_colour);
                                    *updated = true;
                                }
                            }
                        }
                        "style.shadow" => {
                            if let Some(next_colour) = parse_term_colour(value) {
                                if shadow_colour.as_ref() != Some(&next_colour) {
                                    *shadow_colour = Some(next_colour);
                                    *updated = true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                set_vector_property_recursive(children, sprite_id, path, value, updated);
            }
            _ => {}
        }
    }
}

pub(crate) fn parse_term_colour(value: &JsonValue) -> Option<TermColour> {
    let raw = value.as_str()?;
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "black" => Some(TermColour::Black),
        "white" => Some(TermColour::White),
        "gray" | "grey" => Some(TermColour::Gray),
        "silver" => Some(TermColour::Silver),
        "red" => Some(TermColour::Red),
        "green" => Some(TermColour::Green),
        "blue" => Some(TermColour::Blue),
        "yellow" => Some(TermColour::Yellow),
        "cyan" => Some(TermColour::Cyan),
        "magenta" => Some(TermColour::Magenta),
        _ => {
            let hex = normalized.strip_prefix('#')?;
            if hex.len() != 6 {
                return None;
            }
            let Ok(r) = u8::from_str_radix(&hex[0..2], 16) else {
                return None;
            };
            let Ok(g) = u8::from_str_radix(&hex[2..4], 16) else {
                return None;
            };
            let Ok(b) = u8::from_str_radix(&hex[4..6], 16) else {
                return None;
            };
            Some(TermColour::Rgb(r, g, b))
        }
    }
}

fn term_colour_to_json(colour: &TermColour) -> JsonValue {
    match colour {
        TermColour::Black => JsonValue::String("black".to_string()),
        TermColour::White => JsonValue::String("white".to_string()),
        TermColour::Gray => JsonValue::String("gray".to_string()),
        TermColour::Silver => JsonValue::String("silver".to_string()),
        TermColour::Red => JsonValue::String("red".to_string()),
        TermColour::Green => JsonValue::String("green".to_string()),
        TermColour::Blue => JsonValue::String("blue".to_string()),
        TermColour::Yellow => JsonValue::String("yellow".to_string()),
        TermColour::Cyan => JsonValue::String("cyan".to_string()),
        TermColour::Magenta => JsonValue::String("magenta".to_string()),
        TermColour::Rgb(r, g, b) => JsonValue::String(format!("#{r:02x}{g:02x}{b:02x}")),
    }
}

pub(crate) fn json_value_to_f32(value: &JsonValue) -> Option<f32> {
    value
        .as_f64()
        .map(|number| number as f32)
        .or_else(|| value.as_i64().map(|number| number as f32))
}

fn json_value_to_i32(value: &JsonValue) -> Option<i32> {
    if let Some(number) = value.as_i64() {
        return i32::try_from(number).ok();
    }
    value.as_u64().and_then(|number| i32::try_from(number).ok())
}

fn json_to_vector_points(value: &JsonValue) -> Option<Vec<[i32; 2]>> {
    let raw = value.as_array()?;
    let mut points = Vec::with_capacity(raw.len());
    for point in raw {
        if let Some(pair) = point.as_array() {
            if pair.len() != 2 {
                return None;
            }
            let x = json_value_to_i32(&pair[0])?;
            let y = json_value_to_i32(&pair[1])?;
            points.push([x, y]);
            continue;
        }
        if let Some(map) = point.as_object() {
            let x = map.get("x").and_then(json_value_to_i32)?;
            let y = map.get("y").and_then(json_value_to_i32)?;
            points.push([x, y]);
            continue;
        }
        return None;
    }
    Some(points)
}

/// Direct typed mutation of fg_colour and points on a vector sprite.
/// Avoids the JsonValue round-trip used by the generic `set_vector_property_recursive`.
fn set_particle_ramp_recursive(
    sprites: &mut [Sprite],
    sprite_id: &str,
    next_colour: Option<&TermColour>,
    next_points: &[[i32; 2]],
) {
    for sprite in sprites.iter_mut() {
        match sprite {
            Sprite::Vector {
                id: Some(id),
                points,
                fg_colour,
                ..
            } if id == sprite_id => {
                if let Some(colour) = next_colour {
                    if fg_colour.as_ref() != Some(colour) {
                        *fg_colour = Some(colour.clone());
                    }
                }
                if *points != next_points {
                    *points = next_points.to_vec();
                }
                return;
            }
            Sprite::Grid { children, .. }
            | Sprite::Flex { children, .. }
            | Sprite::Panel { children, .. } => {
                set_particle_ramp_recursive(children, sprite_id, next_colour, next_points);
            }
            _ => {}
        }
    }
}
