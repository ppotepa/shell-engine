use std::collections::HashSet;
use std::f32::consts::TAU;

use crate::game_object::GameObject;
use crate::effects::Region;
use crate::scene::{AudioCue, BehaviorParams, BehaviorSpec, Scene};
use crate::scene_runtime::{ObjectRuntimeState, TargetResolver};
use crate::systems::animator::SceneStage;

#[derive(Debug, Clone)]
pub struct BehaviorContext {
    pub stage: SceneStage,
    pub scene_elapsed_ms: u64,
    pub stage_elapsed_ms: u64,
    pub menu_selected_index: usize,
    pub target_resolver: TargetResolver,
    pub object_states: std::collections::BTreeMap<String, ObjectRuntimeState>,
    pub object_regions: std::collections::BTreeMap<String, Region>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BehaviorCommand {
    PlayAudioCue { cue: String, volume: Option<f32> },
    SetVisibility { target: String, visible: bool },
    SetOffset { target: String, dx: i32, dy: i32 },
}

pub trait Behavior: Send + Sync {
    fn update(
        &mut self,
        object: &GameObject,
        scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    );
}

pub fn built_in_behavior(spec: &BehaviorSpec) -> Option<Box<dyn Behavior + Send + Sync>> {
    match spec.name.trim().to_ascii_lowercase().as_str() {
        "blink" => Some(Box::new(BlinkBehavior::from_params(&spec.params))),
        "bob" => Some(Box::new(BobBehavior::from_params(&spec.params))),
        "follow" => Some(Box::new(FollowBehavior::from_params(&spec.params))),
        "menu-selected" => Some(Box::new(MenuSelectedBehavior::from_params(&spec.params))),
        "selected-arrows" => Some(Box::new(SelectedArrowsBehavior::from_params(&spec.params))),
        "stage-visibility" => Some(Box::new(StageVisibilityBehavior::from_params(&spec.params))),
        "timed-visibility" => Some(Box::new(TimedVisibilityBehavior::from_params(&spec.params))),
        _ => None,
    }
}

#[derive(Default)]
pub struct SceneAudioBehavior {
    emitted: HashSet<String>,
}

impl Behavior for SceneAudioBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        for cue in cues_for_stage(scene, &ctx.stage) {
            if ctx.scene_elapsed_ms < cue.at_ms || cue.cue.trim().is_empty() {
                continue;
            }
            let key = cue_key(&scene.id, &object.id, &ctx.stage, cue);
            if self.emitted.insert(key) {
                commands.push(BehaviorCommand::PlayAudioCue {
                    cue: cue.cue.clone(),
                    volume: cue.volume,
                });
            }
        }
    }
}

pub struct BlinkBehavior {
    target: Option<String>,
    visible_ms: u64,
    hidden_ms: u64,
    phase_ms: u64,
}

impl BlinkBehavior {
    fn from_params(params: &BehaviorParams) -> Self {
        Self {
            target: params.target.clone(),
            visible_ms: params.visible_ms.unwrap_or(250),
            hidden_ms: params.hidden_ms.unwrap_or(250),
            phase_ms: params.phase_ms.unwrap_or(0),
        }
    }
}

impl Behavior for BlinkBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        _scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        let cycle = self.visible_ms.saturating_add(self.hidden_ms);
        let visible = if cycle == 0 {
            true
        } else {
            let t = ctx.scene_elapsed_ms.saturating_add(self.phase_ms) % cycle;
            t < self.visible_ms || self.hidden_ms == 0
        };

        commands.push(BehaviorCommand::SetVisibility {
            target: resolve_target(&self.target, object),
            visible,
        });
    }
}

pub struct BobBehavior {
    target: Option<String>,
    amplitude_x: i32,
    amplitude_y: i32,
    period_ms: u64,
    phase_ms: u64,
}

impl BobBehavior {
    fn from_params(params: &BehaviorParams) -> Self {
        Self {
            target: params.target.clone(),
            amplitude_x: params.amplitude_x.unwrap_or(0),
            amplitude_y: params.amplitude_y.unwrap_or(1),
            period_ms: params.period_ms.unwrap_or(2000).max(1),
            phase_ms: params.phase_ms.unwrap_or(0),
        }
    }
}

impl Behavior for BobBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        _scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        let phase = (ctx.scene_elapsed_ms.saturating_add(self.phase_ms) % self.period_ms) as f32
            / self.period_ms as f32;
        let wave = (phase * TAU).sin();
        commands.push(BehaviorCommand::SetOffset {
            target: resolve_target(&self.target, object),
            dx: (self.amplitude_x as f32 * wave).round() as i32,
            dy: (self.amplitude_y as f32 * wave).round() as i32,
        });
    }
}

pub struct FollowBehavior {
    target: Option<String>,
    offset_x: i32,
    offset_y: i32,
}

impl FollowBehavior {
    fn from_params(params: &BehaviorParams) -> Self {
        Self {
            target: params.target.clone(),
            offset_x: params.amplitude_x.unwrap_or(0),
            offset_y: params.amplitude_y.unwrap_or(0),
        }
    }
}

impl Behavior for FollowBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        _scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        let Some(target) = self.target.as_deref() else {
            return;
        };
        let Some(target_id) = ctx.resolve_target(target) else {
            return;
        };
        let Some(target_state) = ctx.object_state(target_id) else {
            return;
        };

        commands.push(BehaviorCommand::SetVisibility {
            target: object.id.clone(),
            visible: target_state.visible,
        });
        commands.push(BehaviorCommand::SetOffset {
            target: object.id.clone(),
            dx: target_state.offset_x.saturating_add(self.offset_x),
            dy: target_state.offset_y.saturating_add(self.offset_y),
        });
    }
}

pub struct StageVisibilityBehavior {
    target: Option<String>,
    stages: Vec<SceneStage>,
}

pub struct MenuSelectedBehavior {
    target: Option<String>,
    index: usize,
}

impl MenuSelectedBehavior {
    fn from_params(params: &BehaviorParams) -> Self {
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
        commands.push(BehaviorCommand::SetVisibility {
            target: resolve_target(&self.target, object),
            visible: ctx.menu_selected_index == self.index,
        });
    }
}

pub struct SelectedArrowsBehavior {
    target: Option<String>,
    index: usize,
    side: ArrowSide,
    padding: i32,
    amplitude_x: i32,
    period_ms: u64,
    phase_ms: u64,
    autoscale_height: bool,
    last_dx: i32,
    last_dy: i32,
}

#[derive(Clone, Copy)]
enum ArrowSide {
    Left,
    Right,
}

impl SelectedArrowsBehavior {
    fn from_params(params: &BehaviorParams) -> Self {
        let side = match params
            .side
            .as_deref()
            .map(str::trim)
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("right") => ArrowSide::Right,
            _ => ArrowSide::Left,
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
            commands.push(BehaviorCommand::SetVisibility {
                target: object.id.clone(),
                visible: false,
            });
            return;
        }

        let Some(target_alias) = self.target.as_deref() else {
            commands.push(BehaviorCommand::SetVisibility {
                target: object.id.clone(),
                visible: false,
            });
            return;
        };
        let Some(target_id) = ctx.resolve_target(target_alias) else {
            commands.push(BehaviorCommand::SetVisibility {
                target: object.id.clone(),
                visible: false,
            });
            return;
        };
        let Some(target_region) = ctx.object_regions.get(target_id) else {
            commands.push(BehaviorCommand::SetVisibility {
                target: object.id.clone(),
                visible: false,
            });
            return;
        };
        let own_region = ctx.object_regions.get(&object.id);

        let wave_phase = (ctx.scene_elapsed_ms.saturating_add(self.phase_ms) % self.period_ms) as f32
            / self.period_ms as f32;
        let wave = (wave_phase * TAU).sin().round() as i32;
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

        let target_x = match self.side {
            ArrowSide::Left => target_region.x as i32 - effective_padding + signed_wave,
            ArrowSide::Right => {
                target_region.x as i32
                    + target_region.width as i32
                    - 1
                    + effective_padding
                    + signed_wave
            }
        };
        let target_y = target_region.y as i32 + (target_region.height.saturating_sub(1) as i32 / 2);

        commands.push(BehaviorCommand::SetVisibility {
            target: object.id.clone(),
            visible: true,
        });

        let Some(own_region) = own_region else {
            // First frame after becoming visible: wait for compositor to discover own region.
            return;
        };

        let base_x = own_region.x as i32 - self.last_dx;
        let base_y = own_region.y as i32 - self.last_dy;
        let new_dx = target_x - base_x;
        let new_dy = target_y - base_y;
        self.last_dx = new_dx;
        self.last_dy = new_dy;

        commands.push(BehaviorCommand::SetOffset {
            target: object.id.clone(),
            dx: new_dx,
            dy: new_dy,
        });
    }
}

impl StageVisibilityBehavior {
    fn from_params(params: &BehaviorParams) -> Self {
        let stages = params
            .stages
            .iter()
            .filter_map(|value| parse_stage_name(value))
            .collect::<Vec<_>>();
        Self {
            target: params.target.clone(),
            stages,
        }
    }
}

impl Behavior for StageVisibilityBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        _scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        let visible = if self.stages.is_empty() {
            true
        } else {
            self.stages.iter().any(|stage| stage == &ctx.stage)
        };
        commands.push(BehaviorCommand::SetVisibility {
            target: resolve_target(&self.target, object),
            visible,
        });
    }
}

pub struct TimedVisibilityBehavior {
    target: Option<String>,
    start_ms: Option<u64>,
    end_ms: Option<u64>,
    time_scope: TimeScope,
}

impl TimedVisibilityBehavior {
    fn from_params(params: &BehaviorParams) -> Self {
        Self {
            target: params.target.clone(),
            start_ms: params.start_ms,
            end_ms: params.end_ms,
            time_scope: TimeScope::from_params(params),
        }
    }
}

impl Behavior for TimedVisibilityBehavior {
    fn update(
        &mut self,
        object: &GameObject,
        _scene: &Scene,
        ctx: &BehaviorContext,
        commands: &mut Vec<BehaviorCommand>,
    ) {
        let elapsed_ms = self.time_scope.elapsed_ms(ctx);
        let after_start = self
            .start_ms
            .map(|start_ms| elapsed_ms >= start_ms)
            .unwrap_or(true);
        let before_end = self
            .end_ms
            .map(|end_ms| elapsed_ms < end_ms)
            .unwrap_or(true);
        commands.push(BehaviorCommand::SetVisibility {
            target: resolve_target(&self.target, object),
            visible: after_start && before_end,
        });
    }
}

fn resolve_target(target: &Option<String>, object: &GameObject) -> String {
    target
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| object.id.clone())
}

#[derive(Clone, Copy)]
enum TimeScope {
    Scene,
    Stage,
}

impl TimeScope {
    fn from_params(params: &BehaviorParams) -> Self {
        match params
            .time_scope
            .as_deref()
            .map(str::trim)
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("stage") => Self::Stage,
            _ => Self::Scene,
        }
    }

    fn elapsed_ms(self, ctx: &BehaviorContext) -> u64 {
        match self {
            Self::Scene => ctx.scene_elapsed_ms,
            Self::Stage => ctx.stage_elapsed_ms,
        }
    }
}

fn parse_stage_name(raw: &str) -> Option<SceneStage> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "on-enter" | "enter" => Some(SceneStage::OnEnter),
        "on-idle" | "idle" => Some(SceneStage::OnIdle),
        "on-leave" | "leave" => Some(SceneStage::OnLeave),
        "done" => Some(SceneStage::Done),
        _ => None,
    }
}

fn cues_for_stage<'a>(scene: &'a Scene, stage: &SceneStage) -> &'a [AudioCue] {
    match stage {
        SceneStage::OnEnter => &scene.audio.on_enter,
        SceneStage::OnIdle => &scene.audio.on_idle,
        SceneStage::OnLeave => &scene.audio.on_leave,
        SceneStage::Done => &[],
    }
}

impl BehaviorContext {
    pub fn resolve_target(&self, target: &str) -> Option<&str> {
        self.target_resolver.resolve_alias(target)
    }

    pub fn object_state(&self, object_id: &str) -> Option<&ObjectRuntimeState> {
        self.object_states.get(object_id)
    }
}

fn cue_key(scene_id: &str, object_id: &str, stage: &SceneStage, cue: &AudioCue) -> String {
    format!("{scene_id}:{object_id}:{stage:?}:{}:{}", cue.at_ms, cue.cue)
}

#[cfg(test)]
mod tests {
    use super::{
        built_in_behavior, Behavior, BehaviorCommand, BehaviorContext, BlinkBehavior, BobBehavior,
        FollowBehavior, MenuSelectedBehavior, SceneAudioBehavior, SelectedArrowsBehavior,
        StageVisibilityBehavior, TimedVisibilityBehavior,
    };
    use crate::effects::Region;
    use crate::game_object::{GameObject, GameObjectKind};
    use crate::scene::{
        AudioCue, BehaviorParams, BehaviorSpec, Scene, SceneAudio, SceneRenderedMode, SceneStages,
        TermColour,
    };
    use crate::scene_runtime::{ObjectRuntimeState, TargetResolver};
    use crate::systems::animator::SceneStage;

    fn scene_object() -> GameObject {
        GameObject {
            id: "scene:intro".to_string(),
            name: "intro".to_string(),
            kind: GameObjectKind::Scene,
            aliases: vec!["intro".to_string()],
            parent_id: None,
            children: Vec::new(),
        }
    }

    #[test]
    fn scene_audio_behavior_emits_each_cue_once() {
        let scene = Scene {
            id: "intro".to_string(),
            title: "Intro".to_string(),
            cutscene: true,
            rendered_mode: SceneRenderedMode::Cell,
            virtual_size_override: None,
            bg_colour: Some(TermColour::Black),
            stages: SceneStages::default(),
            behaviors: Vec::new(),
            audio: SceneAudio {
                on_enter: vec![AudioCue {
                    at_ms: 100,
                    cue: "thunder".to_string(),
                    volume: Some(0.7),
                }],
                on_idle: Vec::new(),
                on_leave: Vec::new(),
            },
            layers: Vec::new(),
            menu_options: Vec::new(),
            next: None,
        };
        let object = scene_object();
        let ctx = BehaviorContext {
            stage: SceneStage::OnEnter,
            scene_elapsed_ms: 100,
            stage_elapsed_ms: 100,
            menu_selected_index: 0,
            target_resolver: TargetResolver::default(),
            object_states: Default::default(),
            object_regions: Default::default(),
        };
        let mut behavior = SceneAudioBehavior::default();
        let mut commands = Vec::new();

        behavior.update(&object, &scene, &ctx, &mut commands);
        behavior.update(&object, &scene, &ctx, &mut commands);

        assert_eq!(
            commands,
            vec![BehaviorCommand::PlayAudioCue {
                cue: "thunder".to_string(),
                volume: Some(0.7)
            }]
        );
    }

    #[test]
    fn blink_behavior_toggles_visibility() {
        let mut behavior = BlinkBehavior::from_params(&BehaviorParams {
            visible_ms: Some(100),
            hidden_ms: Some(100),
            ..BehaviorParams::default()
        });
        let object = scene_object();
        let scene = Scene {
            id: "intro".to_string(),
            title: "Intro".to_string(),
            cutscene: true,
            rendered_mode: SceneRenderedMode::Cell,
            virtual_size_override: None,
            bg_colour: Some(TermColour::Black),
            stages: SceneStages::default(),
            behaviors: Vec::new(),
            audio: SceneAudio::default(),
            layers: Vec::new(),
            menu_options: Vec::new(),
            next: None,
        };

        let mut commands = Vec::new();
        behavior.update(
            &object,
            &scene,
            &BehaviorContext {
                stage: SceneStage::OnIdle,
                scene_elapsed_ms: 150,
                stage_elapsed_ms: 150,
                menu_selected_index: 0,
                target_resolver: TargetResolver::default(),
                object_states: Default::default(),
                object_regions: Default::default(),
            },
            &mut commands,
        );

        assert_eq!(
            commands,
            vec![BehaviorCommand::SetVisibility {
                target: "scene:intro".to_string(),
                visible: false
            }]
        );
    }

    #[test]
    fn bob_behavior_emits_offset() {
        let mut behavior = BobBehavior::from_params(&BehaviorParams {
            amplitude_x: Some(2),
            amplitude_y: Some(0),
            period_ms: Some(1000),
            phase_ms: Some(250),
            ..BehaviorParams::default()
        });
        let object = scene_object();
        let scene = Scene {
            id: "intro".to_string(),
            title: "Intro".to_string(),
            cutscene: true,
            rendered_mode: SceneRenderedMode::Cell,
            virtual_size_override: None,
            bg_colour: Some(TermColour::Black),
            stages: SceneStages::default(),
            behaviors: Vec::new(),
            audio: SceneAudio::default(),
            layers: Vec::new(),
            menu_options: Vec::new(),
            next: None,
        };

        let mut commands = Vec::new();
        behavior.update(
            &object,
            &scene,
            &BehaviorContext {
                stage: SceneStage::OnIdle,
                scene_elapsed_ms: 0,
                stage_elapsed_ms: 0,
                menu_selected_index: 0,
                target_resolver: TargetResolver::default(),
                object_states: Default::default(),
                object_regions: Default::default(),
            },
            &mut commands,
        );

        assert_eq!(
            commands,
            vec![BehaviorCommand::SetOffset {
                target: "scene:intro".to_string(),
                dx: 2,
                dy: 0
            }]
        );
    }

    #[test]
    fn builds_known_behavior_from_spec() {
        let behavior = built_in_behavior(&BehaviorSpec {
            name: "blink".to_string(),
            params: BehaviorParams::default(),
        });

        assert!(behavior.is_some());
    }

    #[test]
    fn stage_visibility_behavior_shows_only_selected_stage() {
        let mut behavior = StageVisibilityBehavior::from_params(&BehaviorParams {
            stages: vec!["on-idle".to_string()],
            ..BehaviorParams::default()
        });
        let object = scene_object();
        let scene = Scene {
            id: "intro".to_string(),
            title: "Intro".to_string(),
            cutscene: true,
            rendered_mode: SceneRenderedMode::Cell,
            virtual_size_override: None,
            bg_colour: Some(TermColour::Black),
            stages: SceneStages::default(),
            behaviors: Vec::new(),
            audio: SceneAudio::default(),
            layers: Vec::new(),
            menu_options: Vec::new(),
            next: None,
        };

        let mut commands = Vec::new();
        behavior.update(
            &object,
            &scene,
            &BehaviorContext {
                stage: SceneStage::OnEnter,
                scene_elapsed_ms: 0,
                stage_elapsed_ms: 0,
                menu_selected_index: 0,
                target_resolver: TargetResolver::default(),
                object_states: Default::default(),
                object_regions: Default::default(),
            },
            &mut commands,
        );

        assert_eq!(
            commands,
            vec![BehaviorCommand::SetVisibility {
                target: "scene:intro".to_string(),
                visible: false
            }]
        );
    }

    #[test]
    fn timed_visibility_behavior_uses_elapsed_time_window() {
        let mut behavior = TimedVisibilityBehavior::from_params(&BehaviorParams {
            target: Some("title".to_string()),
            start_ms: Some(100),
            end_ms: Some(200),
            ..BehaviorParams::default()
        });
        let object = scene_object();
        let scene = Scene {
            id: "intro".to_string(),
            title: "Intro".to_string(),
            cutscene: true,
            rendered_mode: SceneRenderedMode::Cell,
            virtual_size_override: None,
            bg_colour: Some(TermColour::Black),
            stages: SceneStages::default(),
            behaviors: Vec::new(),
            audio: SceneAudio::default(),
            layers: Vec::new(),
            menu_options: Vec::new(),
            next: None,
        };

        let mut commands = Vec::new();
        behavior.update(
            &object,
            &scene,
            &BehaviorContext {
                stage: SceneStage::OnIdle,
                scene_elapsed_ms: 150,
                stage_elapsed_ms: 150,
                menu_selected_index: 0,
                target_resolver: TargetResolver::default(),
                object_states: Default::default(),
                object_regions: Default::default(),
            },
            &mut commands,
        );

        assert_eq!(
            commands,
            vec![BehaviorCommand::SetVisibility {
                target: "title".to_string(),
                visible: true,
            }]
        );
    }

    #[test]
    fn timed_visibility_behavior_can_use_stage_clock() {
        let mut behavior = TimedVisibilityBehavior::from_params(&BehaviorParams {
            target: Some("title".to_string()),
            time_scope: Some("stage".to_string()),
            start_ms: Some(100),
            end_ms: Some(200),
            ..BehaviorParams::default()
        });
        let object = scene_object();
        let scene = Scene {
            id: "intro".to_string(),
            title: "Intro".to_string(),
            cutscene: true,
            rendered_mode: SceneRenderedMode::Cell,
            virtual_size_override: None,
            bg_colour: Some(TermColour::Black),
            stages: SceneStages::default(),
            behaviors: Vec::new(),
            audio: SceneAudio::default(),
            layers: Vec::new(),
            menu_options: Vec::new(),
            next: None,
        };

        let mut commands = Vec::new();
        behavior.update(
            &object,
            &scene,
            &BehaviorContext {
                stage: SceneStage::OnIdle,
                scene_elapsed_ms: 500,
                stage_elapsed_ms: 150,
                menu_selected_index: 0,
                target_resolver: TargetResolver::default(),
                object_states: Default::default(),
                object_regions: Default::default(),
            },
            &mut commands,
        );

        assert_eq!(
            commands,
            vec![BehaviorCommand::SetVisibility {
                target: "title".to_string(),
                visible: true,
            }]
        );
    }

    #[test]
    fn follow_behavior_copies_target_state() {
        let mut behavior = FollowBehavior::from_params(&BehaviorParams {
            target: Some("leader".to_string()),
            amplitude_x: Some(1),
            amplitude_y: Some(-1),
            ..BehaviorParams::default()
        });
        let object = scene_object();
        let scene = Scene {
            id: "intro".to_string(),
            title: "Intro".to_string(),
            cutscene: true,
            rendered_mode: SceneRenderedMode::Cell,
            virtual_size_override: None,
            bg_colour: Some(TermColour::Black),
            stages: SceneStages::default(),
            behaviors: Vec::new(),
            audio: SceneAudio::default(),
            layers: Vec::new(),
            menu_options: Vec::new(),
            next: None,
        };
        let mut resolver = TargetResolver::default();
        resolver.register_alias("leader".to_string(), "obj:leader".to_string());
        let mut object_states = std::collections::BTreeMap::new();
        object_states.insert(
            "obj:leader".to_string(),
            ObjectRuntimeState {
                visible: false,
                offset_x: 3,
                offset_y: 2,
            },
        );

        let mut commands = Vec::new();
        behavior.update(
            &object,
            &scene,
            &BehaviorContext {
                stage: SceneStage::OnIdle,
                scene_elapsed_ms: 0,
                stage_elapsed_ms: 0,
                menu_selected_index: 0,
                target_resolver: resolver,
                object_states,
                object_regions: Default::default(),
            },
            &mut commands,
        );

        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "scene:intro".to_string(),
                    visible: false
                },
                BehaviorCommand::SetOffset {
                    target: "scene:intro".to_string(),
                    dx: 4,
                    dy: 1
                }
            ]
        );
    }

    #[test]
    fn menu_selected_behavior_visibility_matches_selected_index() {
        let mut behavior = MenuSelectedBehavior::from_params(&BehaviorParams {
            index: Some(1),
            ..BehaviorParams::default()
        });
        let object = scene_object();
        let scene = Scene {
            id: "intro".to_string(),
            title: "Intro".to_string(),
            cutscene: true,
            rendered_mode: SceneRenderedMode::Cell,
            virtual_size_override: None,
            bg_colour: Some(TermColour::Black),
            stages: SceneStages::default(),
            behaviors: Vec::new(),
            audio: SceneAudio::default(),
            layers: Vec::new(),
            menu_options: Vec::new(),
            next: None,
        };
        let mut commands = Vec::new();
        behavior.update(
            &object,
            &scene,
            &BehaviorContext {
                stage: SceneStage::OnIdle,
                scene_elapsed_ms: 0,
                stage_elapsed_ms: 0,
                menu_selected_index: 1,
                target_resolver: TargetResolver::default(),
                object_states: Default::default(),
                object_regions: Default::default(),
            },
            &mut commands,
        );

        assert_eq!(
            commands,
            vec![BehaviorCommand::SetVisibility {
                target: "scene:intro".to_string(),
                visible: true
            }]
        );
    }

    #[test]
    fn selected_arrows_hides_when_target_region_missing() {
        let mut behavior = SelectedArrowsBehavior::from_params(&BehaviorParams {
            target: Some("menu-item-0".to_string()),
            index: Some(0),
            side: Some("left".to_string()),
            ..BehaviorParams::default()
        });
        let object = scene_object();
        let scene = Scene {
            id: "intro".to_string(),
            title: "Intro".to_string(),
            cutscene: true,
            rendered_mode: SceneRenderedMode::Cell,
            virtual_size_override: None,
            bg_colour: Some(TermColour::Black),
            stages: SceneStages::default(),
            behaviors: Vec::new(),
            audio: SceneAudio::default(),
            layers: Vec::new(),
            menu_options: Vec::new(),
            next: None,
        };

        let mut commands = Vec::new();
        behavior.update(
            &object,
            &scene,
            &BehaviorContext {
                stage: SceneStage::OnIdle,
                scene_elapsed_ms: 0,
                stage_elapsed_ms: 0,
                menu_selected_index: 0,
                target_resolver: TargetResolver::default(),
                object_states: Default::default(),
                object_regions: Default::default(),
            },
            &mut commands,
        );

        assert_eq!(
            commands,
            vec![BehaviorCommand::SetVisibility {
                target: "scene:intro".to_string(),
                visible: false
            }]
        );
    }

    #[test]
    fn selected_arrows_uses_target_region_and_padding() {
        let mut behavior = SelectedArrowsBehavior::from_params(&BehaviorParams {
            target: Some("menu-item-0".to_string()),
            index: Some(0),
            side: Some("left".to_string()),
            padding: Some(1),
            autoscale_height: Some(true),
            amplitude_x: Some(0),
            period_ms: Some(1000),
            ..BehaviorParams::default()
        });
        let object = scene_object();
        let scene = Scene {
            id: "intro".to_string(),
            title: "Intro".to_string(),
            cutscene: true,
            rendered_mode: SceneRenderedMode::Cell,
            virtual_size_override: None,
            bg_colour: Some(TermColour::Black),
            stages: SceneStages::default(),
            behaviors: Vec::new(),
            audio: SceneAudio::default(),
            layers: Vec::new(),
            menu_options: Vec::new(),
            next: None,
        };

        let mut resolver = TargetResolver::default();
        resolver.register_alias("menu-item-0".to_string(), "obj:menu-item-0".to_string());
        let mut object_regions = std::collections::BTreeMap::new();
        object_regions.insert(
            "scene:intro".to_string(),
            Region {
                x: 20,
                y: 10,
                width: 1,
                height: 1,
            },
        );
        object_regions.insert(
            "obj:menu-item-0".to_string(),
            Region {
                x: 30,
                y: 8,
                width: 10,
                height: 3,
            },
        );

        let mut commands = Vec::new();
        behavior.update(
            &object,
            &scene,
            &BehaviorContext {
                stage: SceneStage::OnIdle,
                scene_elapsed_ms: 0,
                stage_elapsed_ms: 0,
                menu_selected_index: 0,
                target_resolver: resolver,
                object_states: Default::default(),
                object_regions,
            },
            &mut commands,
        );

        assert_eq!(
            commands,
            vec![
                BehaviorCommand::SetVisibility {
                    target: "scene:intro".to_string(),
                    visible: true
                },
                BehaviorCommand::SetOffset {
                    target: "scene:intro".to_string(),
                    dx: 8,
                    dy: -1
                }
            ]
        );
    }
}
