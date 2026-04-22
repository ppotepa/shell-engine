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
        materialize_runtime_object_documents(
            &mut objects,
            &mut object_states,
            &root_id,
            &scene.runtime_objects,
        );

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
            camera_director: camera::CameraDirectorRuntime::default(),
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
        runtime.camera_director.free_look = runtime
            .scene
            .input
            .free_look_camera
            .as_ref()
            .map(camera::FreeLookCameraState::from_controls);
        runtime.camera_director.orbit = runtime
            .scene
            .input
            .orbit_camera
            .as_ref()
            .map(camera::ObjOrbitCameraState::from_controls);
        let camera_preset = runtime
            .scene
            .controller_defaults
            .camera_preset
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if camera_preset.is_none() {
            runtime
                .camera_director
                .select_active_controller_from_camera_preset(None);
        }
        runtime.initialize_ui_state();
        runtime.sync_widget_visuals();
        runtime.attach_default_behaviors();
        runtime.attach_declared_behaviors(behavior_bindings, None);
        runtime.resolver_cache = std::sync::Arc::new(runtime.build_target_resolver());
        runtime.rebuild_sprite_id_to_layer();
        runtime.clamp_orbit_camera_bootstrap();
        runtime
    }

    /// Removes a materialized authored `runtime-object` subtree and keeps the
    /// retained `scene.runtime_objects` bridge payload in sync.
    ///
    /// This is a narrow bridge seam for prefab-first runtime objects. It does
    /// not attempt full runtime instancing or graph rebuild parity; it only
    /// handles direct subtree removal for already-materialized authored nodes.
    pub fn remove_runtime_object_subtree(&mut self, target: &str) -> bool {
        let target = target.trim();
        if target.is_empty() {
            return false;
        }
        let resolver = self.target_resolver();
        let Some(object_id) = resolver.resolve_alias(target).map(str::to_string) else {
            return false;
        };
        let Some(path) = runtime_object_document_path(&self.root_id, &object_id) else {
            return false;
        };

        if !remove_runtime_object_document_at_path(&mut self.scene.runtime_objects, &path) {
            return false;
        }
        if !remove_runtime_object_graph_subtree(
            &mut self.objects,
            &mut self.object_states,
            &mut self.obj_camera_states,
            &object_id,
        ) {
            return false;
        }
        remove_empty_runtime_object_container(&mut self.objects, &mut self.object_states, &self.root_id);
        self.refresh_runtime_object_bridge_indexes();
        self.apply_runtime_mutation_impact(RuntimeMutationImpact::graph());
        true
    }

    fn refresh_runtime_object_bridge_indexes(&mut self) {
        self.cached_object_kinds = std::sync::Arc::new(
            self.objects
                .iter()
                .map(|(id, object)| (id.clone(), object_kind_name(&object.kind).to_string()))
                .collect::<HashMap<_, _>>(),
        );
        self.resolver_cache = std::sync::Arc::new(self.build_target_resolver());
        self.rebuild_sprite_id_to_layer();
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
            world_model: Default::default(),
            controller_defaults: Default::default(),
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
            planet_spec: None,
            planet_spec_ref: None,
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
            runtime_objects: Vec::new(),
            gui: Default::default(),
        };

        let runtime = SceneRuntime::new(scene);

        assert_eq!(
            runtime.resolved_view_profile().environment_policy,
            engine_core::scene::ViewEnvironmentPolicy::ThreeDCelestial
        );
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

    #[test]
    fn runtime_materializes_prefab_first_runtime_object_tree_with_path_aliases() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: runtime-object-bridge
title: Runtime Object Bridge
layers:
  - name: hud
    sprites:
      - type: text
        id: title
        content: TITLE
runtime-objects:
  - name: carrier
    prefab: /prefabs/carrier.yml
    preset: capital-ship
    transform:
      space: 3d
      translation: [1.0, 2.0, 3.0]
    children:
      - name: cockpit
        prefab: /prefabs/cockpit.yml
        transform:
          space: 3d
          translation: [0.0, 1.0, 0.0]
"#,
        )
        .expect("scene parse");

        let runtime = SceneRuntime::new(scene);
        let resolver = runtime.target_resolver();
        let carrier_id = resolver
            .resolve_alias("carrier")
            .expect("carrier alias")
            .to_string();
        let cockpit_id = resolver
            .resolve_alias("runtime-objects/carrier/cockpit")
            .expect("cockpit path alias")
            .to_string();
        let title_id = resolver.resolve_alias("title").expect("title alias").to_string();

        let carrier = runtime.object(&carrier_id).expect("carrier object");
        let cockpit = runtime.object(&cockpit_id).expect("cockpit object");
        assert_eq!(cockpit.parent_id.as_deref(), Some(carrier_id.as_str()));
        assert!(carrier.children.iter().any(|child| child == &cockpit_id));
        assert_eq!(runtime.object(&title_id).expect("title object").name, "text:title");

        let runtime_objects_container = runtime
            .object(carrier.parent_id.as_deref().expect("runtime object container"))
            .expect("container object");
        assert_eq!(runtime_objects_container.name, "runtime-objects");
        assert_eq!(
            runtime_objects_container.parent_id.as_deref(),
            Some(runtime.root_id())
        );
    }

    #[test]
    fn runtime_object_path_aliases_remain_targetable_when_bare_name_conflicts() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: runtime-object-conflict
title: Runtime Object Conflict
layers:
  - name: hud
    sprites:
      - type: text
        id: title
        content: TITLE
runtime-objects:
  - name: title
    prefab: /prefabs/title-probe.yml
    transform:
      space: 3d
      translation: [0.0, 0.0, 0.0]
"#,
        )
        .expect("scene parse");

        let runtime = SceneRuntime::new(scene);
        let resolver = runtime.target_resolver();
        let title_id = resolver.resolve_alias("title").expect("title alias").to_string();
        let runtime_title_id = resolver
            .resolve_alias("runtime-objects/title")
            .expect("runtime object path alias")
            .to_string();

        assert_ne!(title_id, runtime_title_id);
        assert_eq!(runtime.object(&title_id).expect("title object").name, "text:title");
        assert_eq!(
            runtime.object(&runtime_title_id).expect("runtime object").name,
            "title"
        );
    }

    #[test]
    fn runtime_object_effective_state_inherits_parent_visibility_and_offsets() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: runtime-object-state
title: Runtime Object State
layers: []
runtime-objects:
  - name: carrier
    prefab: /prefabs/carrier.yml
    transform:
      space: 3d
      translation: [0.0, 0.0, 0.0]
    children:
      - name: cockpit
        prefab: /prefabs/cockpit.yml
        transform:
          space: 3d
          translation: [0.0, 1.0, 0.0]
"#,
        )
        .expect("scene parse");

        let mut runtime = SceneRuntime::new(scene);
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[
                BehaviorCommand::SetProps {
                    target: "runtime-objects/carrier".to_string(),
                    visible: Some(false),
                    dx: Some(4),
                    dy: Some(-2),
                    text: None,
                },
                BehaviorCommand::SetProps {
                    target: "runtime-objects/carrier/cockpit".to_string(),
                    visible: None,
                    dx: Some(1),
                    dy: Some(3),
                    text: None,
                },
            ],
        );

        let cockpit_id = resolver
            .resolve_alias("runtime-objects/carrier/cockpit")
            .expect("cockpit alias")
            .to_string();
        let effective = runtime
            .effective_object_state(&cockpit_id)
            .expect("effective state");
        assert!(!effective.visible);
        assert_eq!(effective.offset_x, 5);
        assert_eq!(effective.offset_y, 2);
    }

    #[test]
    fn runtime_object_initial_state_reflects_authored_transform_scaffold() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: runtime-object-transform
title: Runtime Object Transform
layers: []
runtime-objects:
  - name: map-pin
    prefab: /prefabs/map-pin.yml
    transform:
      space: 2d
      x: 12.4
      y: -3.6
      rotation-deg: 90.0
  - name: shuttle
    prefab: /prefabs/shuttle.yml
    transform:
      space: 3d
      translation: [4.4, -2.2, 7.7]
  - name: orbital-site
    prefab: /prefabs/site.yml
    transform:
      space: celestial
      frame: local-horizon
      translation: [1.2, 3.4, -5.6]
"#,
        )
        .expect("scene parse");

        let runtime = SceneRuntime::new(scene);
        let resolver = runtime.target_resolver();

        let map_pin = runtime
            .object_state(resolver.resolve_alias("map-pin").expect("map-pin alias"))
            .expect("map-pin state");
        assert_eq!(map_pin.offset_x, 12);
        assert_eq!(map_pin.offset_y, -4);
        assert!((map_pin.heading - std::f32::consts::FRAC_PI_2).abs() < 0.0001);

        let shuttle = runtime
            .object_state(resolver.resolve_alias("shuttle").expect("shuttle alias"))
            .expect("shuttle state");
        assert_eq!(shuttle.offset_x, 4);
        assert_eq!(shuttle.offset_y, -2);
        assert_eq!(shuttle.offset_z, 8);

        let orbital_site = runtime
            .object_state(
                resolver
                    .resolve_alias("orbital-site")
                    .expect("orbital-site alias"),
            )
            .expect("orbital-site state");
        assert_eq!(orbital_site.offset_x, 1);
        assert_eq!(orbital_site.offset_y, 3);
        assert_eq!(orbital_site.offset_z, -6);
    }

    #[test]
    fn runtime_object_subtree_removal_updates_scene_documents_and_preserves_siblings() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: runtime-object-remove-child
title: Runtime Object Remove Child
layers: []
runtime-objects:
  - name: carrier
    prefab: /prefabs/carrier.yml
    transform:
      space: 3d
    children:
      - name: cockpit
        prefab: /prefabs/cockpit.yml
        transform:
          space: 3d
      - name: escort
        prefab: /prefabs/escort.yml
        transform:
          space: 3d
  - name: beacon
    prefab: /prefabs/beacon.yml
    transform:
      space: 3d
"#,
        )
        .expect("scene parse");

        let mut runtime = SceneRuntime::new(scene);
        let resolver = runtime.target_resolver();
        let carrier_id = resolver
            .resolve_alias("runtime-objects/carrier")
            .expect("carrier alias")
            .to_string();
        let cockpit_id = resolver
            .resolve_alias("runtime-objects/carrier/cockpit")
            .expect("cockpit alias")
            .to_string();
        let escort_id = resolver
            .resolve_alias("runtime-objects/carrier/escort")
            .expect("escort alias")
            .to_string();
        let beacon_id = resolver
            .resolve_alias("runtime-objects/beacon")
            .expect("beacon alias")
            .to_string();

        assert!(runtime.remove_runtime_object_subtree("runtime-objects/carrier/cockpit"));

        assert!(runtime.object(&cockpit_id).is_none());
        assert!(runtime.object(&carrier_id).is_some());
        assert!(runtime.object(&escort_id).is_some());
        assert!(runtime.object(&beacon_id).is_some());
        assert!(runtime
            .target_resolver()
            .resolve_alias("runtime-objects/carrier/cockpit")
            .is_none());
        assert_eq!(runtime.scene().runtime_objects.len(), 2);
        assert_eq!(runtime.scene().runtime_objects[0].children.len(), 1);
        assert_eq!(runtime.scene().runtime_objects[0].children[0].name, "escort");
    }

    #[test]
    fn runtime_object_root_removal_drops_empty_container_and_scene_payload() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: runtime-object-remove-root
title: Runtime Object Remove Root
layers: []
runtime-objects:
  - name: carrier
    prefab: /prefabs/carrier.yml
    transform:
      space: 3d
"#,
        )
        .expect("scene parse");

        let mut runtime = SceneRuntime::new(scene);
        let resolver = runtime.target_resolver();
        let carrier_id = resolver
            .resolve_alias("carrier")
            .expect("carrier alias")
            .to_string();
        let container_id = runtime
            .object(&carrier_id)
            .and_then(|object| object.parent_id.clone())
            .expect("runtime object container");

        assert!(runtime.remove_runtime_object_subtree("carrier"));

        assert!(runtime.scene().runtime_objects.is_empty());
        assert!(runtime.object(&carrier_id).is_none());
        assert!(runtime.object(&container_id).is_none());
        assert!(runtime.target_resolver().resolve_alias("carrier").is_none());
        assert!(
            !runtime
                .object(runtime.root_id())
                .expect("root object")
                .children
                .iter()
                .any(|child| child == &container_id)
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

fn materialize_runtime_object_documents(
    objects: &mut HashMap<String, GameObject>,
    object_states: &mut HashMap<String, ObjectRuntimeState>,
    root_id: &str,
    docs: &[engine_core::scene::model::RuntimeObjectDocument],
) {
    if docs.is_empty() {
        return;
    }

    let container_id = format!("{root_id}/runtime-objects");
    insert_object(
        objects,
        object_states,
        GameObject {
            id: container_id.clone(),
            name: "runtime-objects".to_string(),
            kind: GameObjectKind::Layer,
            aliases: Vec::new(),
            parent_id: Some(root_id.to_string()),
            children: Vec::new(),
        },
    );
    if let Some(root) = objects.get_mut(root_id) {
        root.children.push(container_id.clone());
    }

    for (idx, doc) in docs.iter().enumerate() {
        materialize_runtime_object_node(
            objects,
            object_states,
            &container_id,
            idx,
            doc,
            "runtime-objects",
        );
    }
}

fn materialize_runtime_object_node(
    objects: &mut HashMap<String, GameObject>,
    object_states: &mut HashMap<String, ObjectRuntimeState>,
    parent_id: &str,
    node_idx: usize,
    doc: &engine_core::scene::model::RuntimeObjectDocument,
    parent_path_alias: &str,
) {
    let display_name = if doc.name.trim().is_empty() {
        format!("runtime-object-{node_idx}")
    } else {
        doc.name.clone()
    };
    let segment = sanitize_fragment(&display_name);
    let node_id = format!("{parent_id}/runtime-object:{node_idx}:{segment}");
    let path_alias = format!("{parent_path_alias}/{segment}");
    let mut aliases = vec![path_alias.clone()];
    if runtime_object_bare_alias_available(objects, &display_name) {
        aliases.insert(0, display_name.clone());
    }
    insert_object(
        objects,
        object_states,
        GameObject {
            id: node_id.clone(),
            name: display_name,
            kind: GameObjectKind::Layer,
            aliases,
            parent_id: Some(parent_id.to_string()),
            children: Vec::new(),
        },
    );
    if let Some(state) = object_states.get_mut(&node_id) {
        *state = runtime_object_initial_state(&doc.transform);
    }
    if let Some(parent) = objects.get_mut(parent_id) {
        parent.children.push(node_id.clone());
    }

    for (child_idx, child) in doc.children.iter().enumerate() {
        materialize_runtime_object_node(
            objects,
            object_states,
            &node_id,
            child_idx,
            child,
            &path_alias,
        );
    }
}

fn runtime_object_bare_alias_available(objects: &HashMap<String, GameObject>, alias: &str) -> bool {
    let alias = alias.trim();
    !alias.is_empty()
        && !objects.values().any(|object| {
            object.name == alias || object.aliases.iter().any(|existing| existing == alias)
        })
}

fn runtime_object_initial_state(
    transform: &engine_core::scene::model::RuntimeObjectTransform,
) -> ObjectRuntimeState {
    let mut state = ObjectRuntimeState::default();
    match transform {
        engine_core::scene::model::RuntimeObjectTransform::TwoD {
            x,
            y,
            rotation_deg,
            ..
        } => {
            state.offset_x = x.round() as i32;
            state.offset_y = y.round() as i32;
            state.heading = rotation_deg.to_radians();
        }
        engine_core::scene::model::RuntimeObjectTransform::ThreeD { translation, .. }
        | engine_core::scene::model::RuntimeObjectTransform::Celestial { translation, .. } => {
            state.offset_x = translation[0].round() as i32;
            state.offset_y = translation[1].round() as i32;
            state.offset_z = translation[2].round() as i32;
        }
    }
    state
}

fn runtime_object_document_path(root_id: &str, object_id: &str) -> Option<Vec<usize>> {
    let prefix = format!("{root_id}/runtime-objects/");
    let suffix = object_id.strip_prefix(&prefix)?;
    let mut path = Vec::new();
    for segment in suffix.split('/') {
        let rest = segment.strip_prefix("runtime-object:")?;
        let idx_text = rest.split(':').next()?;
        let idx = idx_text.parse::<usize>().ok()?;
        path.push(idx);
    }
    if path.is_empty() {
        None
    } else {
        Some(path)
    }
}

fn remove_runtime_object_document_at_path(
    docs: &mut Vec<engine_core::scene::model::RuntimeObjectDocument>,
    path: &[usize],
) -> bool {
    let Some((&head, tail)) = path.split_first() else {
        return false;
    };
    if tail.is_empty() {
        return if head < docs.len() {
            docs.remove(head);
            true
        } else {
            false
        };
    }
    let Some(node) = docs.get_mut(head) else {
        return false;
    };
    remove_runtime_object_document_at_path(&mut node.children, tail)
}

fn remove_runtime_object_graph_subtree(
    objects: &mut HashMap<String, GameObject>,
    object_states: &mut HashMap<String, ObjectRuntimeState>,
    obj_camera_states: &mut HashMap<String, ObjCameraState>,
    root_id: &str,
) -> bool {
    let Some(root_object) = objects.get(root_id).cloned() else {
        return false;
    };
    if let Some(parent_id) = root_object.parent_id.as_deref() {
        if let Some(parent) = objects.get_mut(parent_id) {
            parent.children.retain(|child| child != root_id);
        }
    }

    let mut stack = vec![root_id.to_string()];
    while let Some(current_id) = stack.pop() {
        if let Some(object) = objects.remove(&current_id) {
            stack.extend(object.children);
        }
        object_states.remove(&current_id);
        obj_camera_states.remove(&current_id);
    }
    true
}

fn remove_empty_runtime_object_container(
    objects: &mut HashMap<String, GameObject>,
    object_states: &mut HashMap<String, ObjectRuntimeState>,
    root_id: &str,
) {
    let container_id = format!("{root_id}/runtime-objects");
    let should_remove = objects
        .get(&container_id)
        .is_some_and(|container| container.children.is_empty());
    if !should_remove {
        return;
    }

    objects.remove(&container_id);
    object_states.remove(&container_id);
    if let Some(root) = objects.get_mut(root_id) {
        root.children.retain(|child| child != &container_id);
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
