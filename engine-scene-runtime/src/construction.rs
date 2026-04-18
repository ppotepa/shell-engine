use super::*;

impl SceneRuntime {
    /// Materializes a runtime scene graph from the authored [`Scene`] model.
    ///
    /// # Invariants
    ///
    /// The runtime assigns stable ids to the scene root, layers, and sprites,
    /// attaches declared behaviors, and keeps resolver indices aligned with the
    /// z-sorted layer and sprite order used by the compositor.
    pub fn new(mut scene: Scene) -> Self {
        scene.layers.sort_by_key(|l| l.z_index);
        for layer in &mut scene.layers {
            layer.sprites.sort_by_key(|s| s.z_index());
        }
        let root_id = format!("scene:{}", sanitize_fragment(&scene.id));
        let mut objects = HashMap::new();
        let mut object_states = HashMap::new();
        let mut layer_ids = BTreeMap::new();
        let mut sprite_ids = HashMap::new();
        let mut behavior_bindings = Vec::new();
        insert_object(
            &mut objects,
            &mut object_states,
            GameObject {
                id: root_id.clone(),
                name: scene.id.clone(),
                kind: GameObjectKind::Scene,
                aliases: vec![scene.id.clone()],
                parent_id: None,
                children: Vec::new(),
            },
        );
        if !scene.behaviors.is_empty() {
            behavior_bindings.push(BehaviorBinding {
                object_id: root_id.clone(),
                specs: scene.behaviors.clone(),
            });
        }

        for (layer_idx, layer) in scene.layers.iter().enumerate() {
            let layer_name = if layer.name.trim().is_empty() {
                format!("layer-{layer_idx}")
            } else {
                layer.name.clone()
            };
            let layer_id = format!(
                "{root_id}/layer:{}:{}",
                layer_idx,
                sanitize_fragment(&layer_name)
            );
            insert_object(
                &mut objects,
                &mut object_states,
                GameObject {
                    id: layer_id.clone(),
                    name: layer_name,
                    kind: GameObjectKind::Layer,
                    aliases: layer_aliases(scene.layers[layer_idx].name.as_str()),
                    parent_id: Some(root_id.clone()),
                    children: Vec::new(),
                },
            );
            layer_ids.insert(layer_idx, layer_id.clone());

            if let Some(root) = objects.get_mut(&root_id) {
                root.children.push(layer_id.clone());
            }
            if !layer.behaviors.is_empty() {
                behavior_bindings.push(BehaviorBinding {
                    object_id: layer_id.clone(),
                    specs: layer.behaviors.clone(),
                });
            }

            for (sprite_idx, sprite) in layer.sprites.iter().enumerate() {
                build_sprite_objects(
                    &mut objects,
                    &mut object_states,
                    &mut sprite_ids,
                    &mut behavior_bindings,
                    layer_idx,
                    &[sprite_idx],
                    &layer_id,
                    sprite,
                    sprite_idx,
                );
            }
        }

        let cached_object_kinds = std::sync::Arc::new(
            objects
                .iter()
                .map(|(id, object)| (id.clone(), object_kind_name(&object.kind).to_string()))
                .collect::<HashMap<_, _>>(),
        );
        let spatial_context = scene.spatial.to_context();
        let resolved_view_profile = engine_core::scene::resolve_scene_view_profile(&scene);

        let mut runtime = Self {
            scene,
            root_id,
            objects,
            object_states,
            layer_ids,
            sprite_ids,
            behaviors: Vec::new(),
            resolver_cache: std::sync::Arc::new(TargetResolver::default()),
            object_regions: std::sync::Arc::new(HashMap::new()),
            cached_object_kinds,
            object_mutation_gen: 0,
            cached_object_states_gen: 0,
            cached_effective_states_gen: 0,
            cached_object_props_gen: 0,
            cached_object_text_gen: 0,
            cached_object_states: None,
            cached_effective_states: None,
            effective_states_dirty: true,
            cached_object_props: None,
            cached_object_text: None,
            cached_sidecar_io: None,
            cached_object_regions: std::sync::Arc::new(HashMap::new()),
            obj_orbit_default_speed: HashMap::new(),
            obj_camera_states: HashMap::new(),
            cached_obj_camera_states: None,
            free_look_camera: None,
            orbit_camera: None,
            ui_state: UiRuntimeState::default(),
            pending_bindings: Vec::new(),
            action_bindings: HashMap::new(),
            cached_action_bindings: None,
            prev_collision_pairs: std::collections::HashSet::new(),
            prev_keys_down: std::collections::HashSet::new(),
            prev_scene_elapsed_ms: 0,
            palette_applied_version: 0,
            game_state_applied_version: 0,
            sprite_id_to_layer: HashMap::new(),
            spawn_batch_depth: 0,
            camera_x: 0,
            camera_y: 0,
            camera_zoom: 1.0,
            spatial_context,
            scene_camera_3d: SceneCamera3D::default(),
            resolved_view_profile,
            runtime_lighting_profile_override: None,
            runtime_space_environment_override: None,
            render3d_dirty_mask: engine_core::render_types::DirtyMask3D::empty(),
            render3d_rebuild_diagnostics: Render3dRebuildDiagnostics::default(),
            gui_widgets: Vec::new(),
            gui_state: engine_gui::GuiRuntimeState::new(),
            cached_gui_state: None,
        };
        runtime.gui_widgets = runtime
            .scene
            .gui
            .widgets
            .clone()
            .into_iter()
            .map(|widget| scene_gui_widget_to_control(widget, runtime.scene.ui.scale))
            .collect();
        runtime.obj_orbit_default_speed = collect_obj_orbit_defaults(&runtime.scene);
        runtime.free_look_camera = runtime
            .scene
            .input
            .free_look_camera
            .as_ref()
            .map(FreeLookCameraState::from_controls);
        runtime.orbit_camera = runtime
            .scene
            .input
            .orbit_camera
            .as_ref()
            .map(ObjOrbitCameraState::from_controls);
        runtime.initialize_ui_state();
        runtime.sync_widget_visuals();
        runtime.attach_default_behaviors();
        runtime.attach_declared_behaviors(behavior_bindings, None);
        runtime.resolver_cache = std::sync::Arc::new(runtime.build_target_resolver());
        runtime.rebuild_sprite_id_to_layer();
        runtime.clamp_orbit_camera_bootstrap();
        runtime
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_materializes_resolved_view_profile_from_scene() {
        let scene = Scene {
            id: "test".to_string(),
            title: "Test".to_string(),
            cutscene: false,
            target_fps: None,
            space: Default::default(),
            spatial: Default::default(),
            celestial: Default::default(),
            lighting: Some(engine_core::scene::SceneLighting {
                ambient_floor: Some(0.11),
            }),
            view: Some(engine_core::scene::SceneView {
                profile: Some("orbit-realistic".to_string()),
                lighting_profile: None,
                space_environment_profile: None,
                resolved_view_profile_asset: None,
                resolved_lighting_profile_asset: None,
                resolved_space_environment_profile_asset: None,
            }),
            virtual_size_override: None,
            bg_colour: None,
            stages: Default::default(),
            behaviors: Vec::new(),
            audio: Default::default(),
            ui: Default::default(),
            layers: Vec::new(),
            menu_options: Vec::new(),
            input: Default::default(),
            postfx: Vec::new(),
            next: None,
            prerender: false,
            palette_bindings: Vec::new(),
            game_state_bindings: Vec::new(),
            gui: Default::default(),
        };

        let runtime = SceneRuntime::new(scene);

        assert_eq!(
            runtime.resolved_view_profile().lighting.black_level,
            Some(0.11)
        );
        assert_eq!(
            runtime
                .resolved_view_profile()
                .environment
                .starfield_brightness,
            Some(0.7)
        );
    }
}

fn path_key(layer_idx: usize, sprite_path: &[usize]) -> String {
    let mut key = layer_idx.to_string();
    for idx in sprite_path {
        key.push('/');
        key.push_str(&idx.to_string());
    }
    key
}

fn scale_i32(value: i32, scale: f32) -> i32 {
    ((value as f32) * scale.max(0.01)).round() as i32
}

fn scene_gui_widget_to_control(
    def: engine_core::scene::model::SceneGuiWidgetDef,
    ui_scale: f32,
) -> Box<dyn engine_gui::GuiControl> {
    use engine_core::scene::model::SceneGuiWidgetDef as Src;
    let ui_scale = ui_scale.max(0.01);
    let to_choice = |choice: engine_core::scene::model::SceneGuiChoiceDef| {
        engine_gui::ChoiceOption::new(choice.value.clone(), choice.label.unwrap_or(choice.value))
    };
    match def {
        Src::Slider {
            id,
            sprite,
            x,
            y,
            w,
            h,
            min,
            max,
            value,
            hit_padding,
            handle,
            follow_layout,
        } => Box::new(engine_gui::SliderControl {
            id,
            sprite,
            x: scale_i32(x, ui_scale),
            y: scale_i32(y, ui_scale),
            w: scale_i32(w, ui_scale).max(1),
            h: scale_i32(h, ui_scale).max(1),
            min,
            max,
            value,
            hit_padding: scale_i32(hit_padding, ui_scale),
            handle,
            follow_layout,
        }),
        Src::Button {
            id,
            sprite,
            x,
            y,
            w,
            h,
            follow_layout,
        } => Box::new(engine_gui::ButtonControl {
            id,
            sprite,
            x: scale_i32(x, ui_scale),
            y: scale_i32(y, ui_scale),
            w: scale_i32(w, ui_scale).max(1),
            h: scale_i32(h, ui_scale).max(1),
            follow_layout,
        }),
        Src::Toggle {
            id,
            sprite,
            x,
            y,
            w,
            h,
            on,
            follow_layout,
        } => Box::new(engine_gui::ToggleControl {
            id,
            sprite,
            x: scale_i32(x, ui_scale),
            y: scale_i32(y, ui_scale),
            w: scale_i32(w, ui_scale).max(1),
            h: scale_i32(h, ui_scale).max(1),
            initial_on: on,
            follow_layout,
        }),
        Src::Panel {
            id,
            sprite,
            visible,
        } => Box::new(engine_gui::PanelControl {
            id,
            sprite,
            visible,
        }),
        Src::RadioGroup {
            id,
            sprite,
            x,
            y,
            w,
            h,
            options,
            selected,
            selected_sprites,
            follow_layout,
        } => Box::new(engine_gui::RadioGroupControl {
            id,
            sprite,
            x: scale_i32(x, ui_scale),
            y: scale_i32(y, ui_scale),
            w: scale_i32(w, ui_scale).max(1),
            h: scale_i32(h, ui_scale).max(1),
            options: options.into_iter().map(to_choice).collect(),
            selected,
            selected_sprites,
            follow_layout,
        }),
        Src::SegmentedControl {
            id,
            sprite,
            x,
            y,
            w,
            h,
            options,
            selected,
            selected_sprites,
            follow_layout,
        } => Box::new(engine_gui::RadioGroupControl {
            id,
            sprite,
            x: scale_i32(x, ui_scale),
            y: scale_i32(y, ui_scale),
            w: scale_i32(w, ui_scale).max(1),
            h: scale_i32(h, ui_scale).max(1),
            options: options.into_iter().map(to_choice).collect(),
            selected,
            selected_sprites,
            follow_layout,
        }),
        Src::Tabs {
            id,
            sprite,
            x,
            y,
            w,
            h,
            options,
            selected,
            selected_sprites,
            follow_layout,
        } => Box::new(engine_gui::RadioGroupControl {
            id,
            sprite,
            x: scale_i32(x, ui_scale),
            y: scale_i32(y, ui_scale),
            w: scale_i32(w, ui_scale).max(1),
            h: scale_i32(h, ui_scale).max(1),
            options: options
                .into_iter()
                .map(|opt| engine_gui::ChoiceOption {
                    value: opt.value,
                    label: opt.label.unwrap_or_default(),
                })
                .collect(),
            selected,
            selected_sprites,
            follow_layout,
        }),
        Src::Dropdown {
            id,
            sprite,
            x,
            y,
            w,
            h,
            options,
            selected,
            popup_sprite,
            label_sprite,
            option_sprites,
            popup_above,
            follow_layout,
        } => Box::new(engine_gui::DropdownControl {
            id,
            sprite,
            x: scale_i32(x, ui_scale),
            y: scale_i32(y, ui_scale),
            w: scale_i32(w, ui_scale).max(1),
            h: scale_i32(h, ui_scale).max(1),
            options: options.into_iter().map(to_choice).collect(),
            selected,
            popup_sprite,
            label_sprite,
            option_sprites,
            popup_above,
            follow_layout,
        }),
        Src::TextInput {
            id,
            sprite,
            x,
            y,
            w,
            h,
            text_sprite,
            placeholder,
            value,
            max_length,
            follow_layout,
        } => Box::new(engine_gui::TextInputControl {
            id,
            sprite,
            x: scale_i32(x, ui_scale),
            y: scale_i32(y, ui_scale),
            w: scale_i32(w, ui_scale).max(1),
            h: scale_i32(h, ui_scale).max(1),
            text_sprite,
            placeholder,
            value,
            max_length,
            follow_layout,
        }),
        Src::NumberInput {
            id,
            sprite,
            x,
            y,
            w,
            h,
            text_sprite,
            placeholder,
            value,
            min,
            max,
            step,
            max_length,
            follow_layout,
        } => Box::new(engine_gui::NumberInputControl {
            id,
            sprite,
            x: scale_i32(x, ui_scale),
            y: scale_i32(y, ui_scale),
            w: scale_i32(w, ui_scale).max(1),
            h: scale_i32(h, ui_scale).max(1),
            text_sprite,
            placeholder,
            value,
            min,
            max,
            step,
            max_length,
            follow_layout,
        }),
    }
}

#[allow(clippy::too_many_arguments)]
fn build_sprite_objects(
    objects: &mut HashMap<String, GameObject>,
    object_states: &mut HashMap<String, ObjectRuntimeState>,
    sprite_ids: &mut HashMap<String, String>,
    behavior_bindings: &mut Vec<BehaviorBinding>,
    layer_idx: usize,
    sprite_path: &[usize],
    parent_id: &str,
    sprite: &Sprite,
    sprite_idx: usize,
) {
    let (kind, name, aliases) = sprite_descriptor(sprite, sprite_idx);
    let sprite_id = format!("{parent_id}/{name}");
    insert_object(
        objects,
        object_states,
        GameObject {
            id: sprite_id.clone(),
            name: name.clone(),
            kind,
            aliases,
            parent_id: Some(parent_id.to_string()),
            children: Vec::new(),
        },
    );
    sprite_ids.insert(path_key(layer_idx, sprite_path), sprite_id.clone());
    // Apply authored initial visibility (default is true; sprites may declare visible: false).
    if !sprite.visible() {
        if let Some(state) = object_states.get_mut(&sprite_id) {
            state.visible = false;
        }
    }
    if !sprite.behaviors().is_empty() {
        behavior_bindings.push(BehaviorBinding {
            object_id: sprite_id.clone(),
            specs: sprite.behaviors().to_vec(),
        });
    }
    if let Some(parent) = objects.get_mut(parent_id) {
        parent.children.push(sprite_id.clone());
    }

    if let Sprite::Grid { children, .. }
    | Sprite::Flex { children, .. }
    | Sprite::Panel { children, .. } = sprite
    {
        for (child_idx, child) in children.iter().enumerate() {
            let mut child_path = sprite_path.to_vec();
            child_path.push(child_idx);
            build_sprite_objects(
                objects,
                object_states,
                sprite_ids,
                behavior_bindings,
                layer_idx,
                &child_path,
                &sprite_id,
                child,
                child_idx,
            );
        }
    }
}

fn insert_object(
    objects: &mut HashMap<String, GameObject>,
    object_states: &mut HashMap<String, ObjectRuntimeState>,
    object: GameObject,
) {
    object_states.insert(object.id.clone(), ObjectRuntimeState::default());
    objects.insert(object.id.clone(), object);
}

fn sprite_descriptor(sprite: &Sprite, sprite_idx: usize) -> (GameObjectKind, String, Vec<String>) {
    match sprite {
        Sprite::Text { id, .. } => (
            GameObjectKind::TextSprite,
            sprite_name("text", id.as_deref(), sprite_idx),
            sprite_aliases(id.as_deref()),
        ),
        Sprite::Image { id, .. } => (
            GameObjectKind::ImageSprite,
            sprite_name("image", id.as_deref(), sprite_idx),
            sprite_aliases(id.as_deref()),
        ),
        Sprite::Obj { id, .. } => (
            GameObjectKind::ObjSprite,
            sprite_name("obj", id.as_deref(), sprite_idx),
            sprite_aliases(id.as_deref()),
        ),
        Sprite::Planet { id, .. } => (
            GameObjectKind::ObjSprite,
            sprite_name("planet", id.as_deref(), sprite_idx),
            sprite_aliases(id.as_deref()),
        ),
        Sprite::Vector { id, .. } => (
            GameObjectKind::VectorSprite,
            sprite_name("vector", id.as_deref(), sprite_idx),
            sprite_aliases(id.as_deref()),
        ),
        Sprite::Panel { id, .. } => (
            GameObjectKind::PanelSprite,
            sprite_name("panel", id.as_deref(), sprite_idx),
            sprite_aliases(id.as_deref()),
        ),
        Sprite::Grid { id, .. } => (
            GameObjectKind::GridSprite,
            sprite_name("grid", id.as_deref(), sprite_idx),
            sprite_aliases(id.as_deref()),
        ),
        Sprite::Flex { id, .. } => (
            GameObjectKind::FlexSprite,
            sprite_name("flex", id.as_deref(), sprite_idx),
            sprite_aliases(id.as_deref()),
        ),
        Sprite::Scene3D { id, .. } => (
            GameObjectKind::ObjSprite,
            sprite_name("scene3d", id.as_deref(), sprite_idx),
            sprite_aliases(id.as_deref()),
        ),
    }
}

fn sprite_name(prefix: &str, explicit_id: Option<&str>, sprite_idx: usize) -> String {
    if let Some(id) = explicit_id.filter(|value| !value.trim().is_empty()) {
        format!("{prefix}:{}", sanitize_fragment(id))
    } else {
        format!("{prefix}:{sprite_idx}")
    }
}

fn sprite_aliases(explicit_id: Option<&str>) -> Vec<String> {
    explicit_id
        .filter(|value| !value.trim().is_empty())
        .map(|value| vec![value.to_string()])
        .unwrap_or_default()
}

fn layer_aliases(name: &str) -> Vec<String> {
    if name.trim().is_empty() {
        Vec::new()
    } else {
        vec![name.to_string()]
    }
}

fn sanitize_fragment(input: &str) -> String {
    let sanitized = input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':') {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    let collapsed = sanitized
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if collapsed.is_empty() {
        "unnamed".to_string()
    } else {
        collapsed
    }
}

fn object_kind_name(kind: &GameObjectKind) -> &'static str {
    match kind {
        GameObjectKind::Scene => "scene",
        GameObjectKind::Layer => "layer",
        GameObjectKind::TextSprite => "text",
        GameObjectKind::ImageSprite => "image",
        GameObjectKind::ObjSprite => "obj",
        GameObjectKind::VectorSprite => "vector",
        GameObjectKind::PanelSprite => "panel",
        GameObjectKind::GridSprite => "grid",
        GameObjectKind::FlexSprite => "flex",
    }
}

fn collect_obj_orbit_defaults(scene: &Scene) -> HashMap<String, f32> {
    let mut out = HashMap::new();
    for layer in &scene.layers {
        for_each_obj(&layer.sprites, &mut |sprite| {
            if let Sprite::Obj {
                id: Some(id),
                rotate_y_deg_per_sec,
                ..
            } = sprite
            {
                out.entry(id.to_string())
                    .or_insert(rotate_y_deg_per_sec.unwrap_or(20.0));
            }
        });
    }
    out
}

/// Visit every [`Sprite::Obj`] in a tree, recursing into grids.
fn for_each_obj(sprites: &[Sprite], f: &mut impl FnMut(&Sprite)) {
    for sprite in sprites {
        match sprite {
            Sprite::Obj { .. } => f(sprite),
            Sprite::Grid { children, .. } => for_each_obj(children, f),
            _ => {}
        }
    }
}
