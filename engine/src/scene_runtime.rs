use crate::behavior::{
    built_in_behavior, Behavior, BehaviorCommand, BehaviorContext, SceneAudioBehavior,
};
use crate::effects::Region;
use crate::game_object::{GameObject, GameObjectKind};
use crate::scene::{BehaviorSpec, Scene, Sprite};
use crate::systems::animator::SceneStage;
use std::collections::BTreeMap;

pub struct SceneRuntime {
    scene: Scene,
    root_id: String,
    objects: BTreeMap<String, GameObject>,
    object_states: BTreeMap<String, ObjectRuntimeState>,
    layer_ids: BTreeMap<usize, String>,
    sprite_ids: BTreeMap<String, String>,
    behaviors: Vec<ObjectBehaviorRuntime>,
    resolver_cache: TargetResolver,
}

#[derive(Debug, Clone, Default)]
pub struct TargetResolver {
    scene_object_id: String,
    aliases: BTreeMap<String, String>,
    layer_ids: BTreeMap<usize, String>,
    sprite_ids: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectRuntimeState {
    pub visible: bool,
    pub offset_x: i32,
    pub offset_y: i32,
}

impl Default for ObjectRuntimeState {
    fn default() -> Self {
        Self {
            visible: true,
            offset_x: 0,
            offset_y: 0,
        }
    }
}

impl SceneRuntime {
    pub fn new(scene: Scene) -> Self {
        let root_id = format!("scene:{}", sanitize_fragment(&scene.id));
        let mut objects = BTreeMap::new();
        let mut object_states = BTreeMap::new();
        let mut layer_ids = BTreeMap::new();
        let mut sprite_ids = BTreeMap::new();
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

        let mut runtime = Self {
            scene,
            root_id,
            objects,
            object_states,
            layer_ids,
            sprite_ids,
            behaviors: Vec::new(),
            resolver_cache: TargetResolver::default(),
        };
        runtime.attach_default_behaviors();
        runtime.attach_declared_behaviors(behavior_bindings);
        // Pre-sort layers and sprites once at load time so the compositor hot path skips sorting.
        runtime.scene.layers.sort_by_key(|l| l.z_index);
        for layer in &mut runtime.scene.layers {
            layer.sprites.sort_by_key(|s| s.z_index());
        }
        runtime.resolver_cache = runtime.build_target_resolver();
        runtime
    }

    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    pub fn object(&self, id: &str) -> Option<&GameObject> {
        self.objects.get(id)
    }

    pub fn objects(&self) -> impl Iterator<Item = &GameObject> {
        self.objects.values()
    }

    pub fn object_count(&self) -> usize {
        self.objects.len()
    }

    pub fn object_state(&self, id: &str) -> Option<&ObjectRuntimeState> {
        self.object_states.get(id)
    }

    pub fn object_states_snapshot(&self) -> BTreeMap<String, ObjectRuntimeState> {
        self.object_states.clone()
    }

    pub fn effective_object_state(&self, id: &str) -> Option<ObjectRuntimeState> {
        let mut state = self.object_states.get(id)?.clone();
        let mut parent_id = self
            .objects
            .get(id)
            .and_then(|object| object.parent_id.as_deref());

        while let Some(current_parent_id) = parent_id {
            let parent_state = self
                .object_states
                .get(current_parent_id)
                .cloned()
                .unwrap_or_default();
            state.visible &= parent_state.visible;
            state.offset_x = state.offset_x.saturating_add(parent_state.offset_x);
            state.offset_y = state.offset_y.saturating_add(parent_state.offset_y);
            parent_id = self
                .objects
                .get(current_parent_id)
                .and_then(|object| object.parent_id.as_deref());
        }

        Some(state)
    }

    pub fn effective_object_states_snapshot(&self) -> BTreeMap<String, ObjectRuntimeState> {
        self.objects
            .keys()
            .filter_map(|object_id| {
                self.effective_object_state(object_id)
                    .map(|state| (object_id.clone(), state))
            })
            .collect()
    }

    pub fn target_resolver(&self) -> TargetResolver {
        self.resolver_cache.clone()
    }

    fn build_target_resolver(&self) -> TargetResolver {
        let mut aliases = BTreeMap::new();

        for (object_id, object) in &self.objects {
            aliases.insert(object_id.clone(), object_id.clone());
            aliases.insert(object.name.clone(), object_id.clone());
            for alias in &object.aliases {
                aliases.insert(alias.clone(), object_id.clone());
            }
        }

        TargetResolver {
            scene_object_id: self.root_id.clone(),
            aliases,
            layer_ids: self.layer_ids.clone(),
            sprite_ids: self.sprite_ids.clone(),
        }
    }

    pub fn update_behaviors(
        &mut self,
        stage: SceneStage,
        scene_elapsed_ms: u64,
        stage_elapsed_ms: u64,
    ) -> Vec<BehaviorCommand> {
        let mut commands = Vec::new();
        for idx in 0..self.behaviors.len() {
            let object_id = self.behaviors[idx].object_id.clone();
            let Some(object) = self.objects.get(&object_id).cloned() else {
                continue;
            };
            let ctx = BehaviorContext {
                stage: stage.clone(),
                scene_elapsed_ms,
                stage_elapsed_ms,
                target_resolver: self.resolver_cache.clone(),
                object_states: self.effective_object_states_snapshot(),
            };
            let mut local_commands = Vec::new();
            self.behaviors[idx]
                .behavior
                .update(&object, &self.scene, &ctx, &mut local_commands);
            self.apply_behavior_commands(&self.resolver_cache.clone(), &local_commands);
            commands.extend(local_commands.iter().cloned());
        }
        commands
    }

    pub fn reset_frame_state(&mut self) {
        for state in self.object_states.values_mut() {
            *state = ObjectRuntimeState::default();
        }
    }

    pub fn apply_behavior_commands(
        &mut self,
        resolver: &TargetResolver,
        commands: &[BehaviorCommand],
    ) {
        for command in commands {
            match command {
                BehaviorCommand::PlayAudioCue { .. } => {}
                BehaviorCommand::SetVisibility { target, visible } => {
                    let Some(object_id) = resolver.resolve_alias(target) else {
                        continue;
                    };
                    if let Some(state) = self.object_states.get_mut(object_id) {
                        state.visible = *visible;
                    }
                }
                BehaviorCommand::SetOffset { target, dx, dy } => {
                    let Some(object_id) = resolver.resolve_alias(target) else {
                        continue;
                    };
                    if let Some(state) = self.object_states.get_mut(object_id) {
                        state.offset_x = state.offset_x.saturating_add(*dx);
                        state.offset_y = state.offset_y.saturating_add(*dy);
                    }
                }
            }
        }
    }

    fn attach_default_behaviors(&mut self) {
        if has_scene_audio(&self.scene) {
            self.behaviors.push(ObjectBehaviorRuntime {
                object_id: self.root_id.clone(),
                behavior: Box::new(SceneAudioBehavior::default()),
            });
        }
    }

    fn attach_declared_behaviors(&mut self, bindings: Vec<BehaviorBinding>) {
        for binding in bindings {
            for spec in binding.specs {
                if let Some(behavior) = built_in_behavior(&spec) {
                    self.behaviors.push(ObjectBehaviorRuntime {
                        object_id: binding.object_id.clone(),
                        behavior,
                    });
                }
            }
        }
    }
}

struct ObjectBehaviorRuntime {
    object_id: String,
    behavior: Box<dyn Behavior + Send + Sync>,
}

struct BehaviorBinding {
    object_id: String,
    specs: Vec<BehaviorSpec>,
}

fn has_scene_audio(scene: &Scene) -> bool {
    !scene.audio.on_enter.is_empty()
        || !scene.audio.on_idle.is_empty()
        || !scene.audio.on_leave.is_empty()
}

impl TargetResolver {
    pub fn scene_object_id(&self) -> &str {
        &self.scene_object_id
    }

    pub fn resolve_alias(&self, target: &str) -> Option<&str> {
        self.aliases.get(target).map(String::as_str)
    }

    pub fn register_alias(&mut self, alias: String, object_id: String) {
        self.aliases.insert(alias, object_id);
    }

    pub fn layer_object_id(&self, layer_idx: usize) -> Option<&str> {
        self.layer_ids.get(&layer_idx).map(String::as_str)
    }

    pub fn sprite_object_id(&self, layer_idx: usize, sprite_path: &[usize]) -> Option<&str> {
        self.sprite_ids
            .get(&path_key(layer_idx, sprite_path))
            .map(String::as_str)
    }

    pub fn effect_region(
        &self,
        target: Option<&str>,
        default_region: Region,
        object_regions: &BTreeMap<String, Region>,
    ) -> Region {
        let Some(target) = target.filter(|value| !value.trim().is_empty()) else {
            return default_region;
        };
        self.resolve_alias(target)
            .and_then(|object_id| object_regions.get(object_id).copied())
            .unwrap_or(default_region)
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

fn build_sprite_objects(
    objects: &mut BTreeMap<String, GameObject>,
    object_states: &mut BTreeMap<String, ObjectRuntimeState>,
    sprite_ids: &mut BTreeMap<String, String>,
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
    if !sprite.behaviors().is_empty() {
        behavior_bindings.push(BehaviorBinding {
            object_id: sprite_id.clone(),
            specs: sprite.behaviors().to_vec(),
        });
    }
    if let Some(parent) = objects.get_mut(parent_id) {
        parent.children.push(sprite_id.clone());
    }

    if let Sprite::Grid { children, .. } = sprite {
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
    objects: &mut BTreeMap<String, GameObject>,
    object_states: &mut BTreeMap<String, ObjectRuntimeState>,
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
        Sprite::Grid { id, .. } => (
            GameObjectKind::GridSprite,
            sprite_name("grid", id.as_deref(), sprite_idx),
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

#[cfg(test)]
mod tests {
    use super::SceneRuntime;
    use crate::behavior::BehaviorCommand;
    use crate::game_object::GameObjectKind;
    use crate::scene::Scene;

    #[test]
    fn builds_object_hierarchy_for_layers_and_nested_sprites() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: intro
title: Intro
bg_colour: black
layers:
  - name: UI
    sprites:
      - type: grid
        id: root-grid
        width: 10
        height: 5
        columns: ["1fr"]
        rows: ["1fr"]
        children:
          - type: text
            id: title
            content: HELLO
"#,
        )
        .expect("scene should parse");
        let runtime = SceneRuntime::new(scene);

        assert_eq!(runtime.object_count(), 4);
        let root = runtime
            .object(runtime.root_id())
            .expect("scene root should exist");
        assert_eq!(root.kind, GameObjectKind::Scene);
        assert_eq!(root.children.len(), 1);

        let grid = runtime
            .objects()
            .find(|object| object.kind == GameObjectKind::GridSprite)
            .expect("grid object");
        assert_eq!(grid.children.len(), 1);

        let text = runtime
            .objects()
            .find(|object| object.kind == GameObjectKind::TextSprite)
            .expect("text object");
        assert_eq!(text.parent_id.as_deref(), Some(grid.id.as_str()));
    }

    #[test]
    fn target_resolver_supports_alias_lookup_and_sprite_paths() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: intro
title: Intro
bg_colour: black
layers:
  - name: HUD
    sprites:
      - type: grid
        id: root-grid
        columns: ["1fr"]
        rows: ["1fr"]
        children:
          - type: text
            id: title
            content: HELLO
"#,
        )
        .expect("scene should parse");
        let runtime = SceneRuntime::new(scene);
        let resolver = runtime.target_resolver();

        let title_id = resolver.resolve_alias("title").expect("title alias");
        assert_eq!(resolver.resolve_alias("HUD"), resolver.layer_object_id(0));
        assert_eq!(resolver.sprite_object_id(0, &[0, 0]), Some(title_id));
    }

    #[test]
    fn effective_object_state_accumulates_parent_visibility_and_offsets() {
        let scene: Scene = serde_yaml::from_str(
            r#"
id: intro
title: Intro
bg_colour: black
layers:
  - name: UI
    sprites:
      - type: grid
        id: root-grid
        width: 10
        height: 5
        columns: ["1fr"]
        rows: ["1fr"]
        children:
          - type: text
            id: title
            content: HELLO
"#,
        )
        .expect("scene should parse");
        let mut runtime = SceneRuntime::new(scene);
        let resolver = runtime.target_resolver();
        runtime.apply_behavior_commands(
            &resolver,
            &[
                BehaviorCommand::SetOffset {
                    target: "intro".to_string(),
                    dx: 1,
                    dy: 0,
                },
                BehaviorCommand::SetVisibility {
                    target: "UI".to_string(),
                    visible: false,
                },
                BehaviorCommand::SetOffset {
                    target: "UI".to_string(),
                    dx: 2,
                    dy: 0,
                },
                BehaviorCommand::SetOffset {
                    target: "root-grid".to_string(),
                    dx: 3,
                    dy: 0,
                },
                BehaviorCommand::SetOffset {
                    target: "title".to_string(),
                    dx: 4,
                    dy: 0,
                },
            ],
        );

        let title_id = resolver.resolve_alias("title").expect("title id");
        let state = runtime
            .effective_object_state(title_id)
            .expect("effective state");

        assert!(!state.visible);
        assert_eq!(state.offset_x, 10);
        assert_eq!(state.offset_y, 0);
    }
}
