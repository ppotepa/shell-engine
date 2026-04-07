use crate::animations::AnimationDispatcher;
use crate::authoring::metadata::{FieldMetadata, Requirement, TargetKind, ValueKind, ValueSource};
use crate::scene::{SceneInput, LAYER_FIELDS, OBJECT_FIELDS, SCENE_FIELDS, SPRITE_FIELDS};

/// Parameter shape for one built-in input profile.
#[derive(Debug, Clone, Copy)]
pub struct InputProfileShape {
    /// Profile name as used in YAML (e.g. "obj-viewer").
    pub name: &'static str,
    /// Metadata for every field the profile accepts.
    pub fields: &'static [FieldMetadata],
}

const LIT_ONLY: &[ValueSource] = &[ValueSource::Literal];

const OBJ_VIEWER_FIELDS: &[FieldMetadata] = &[FieldMetadata {
    target: TargetKind::InputProfile,
    name: "sprite_id",
    value_kind: ValueKind::Text,
    requirement: Requirement::Required,
    description: "Target OBJ sprite id for 3-D viewer controls.",
    default_text: None,
    default_number: None,
    enum_options: None,
    min: None,
    max: None,
    step: None,
    unit: None,
    sources: LIT_ONLY,
}];

const TERMINAL_SIZE_TESTER_FIELDS: &[FieldMetadata] = &[FieldMetadata {
    target: TargetKind::InputProfile,
    name: "presets",
    value_kind: ValueKind::SelectList,
    requirement: Requirement::Optional,
    description: "Optional terminal size presets in WIDTHxHEIGHT format (e.g. 120x36).",
    default_text: None,
    default_number: None,
    enum_options: None,
    min: None,
    max: None,
    step: None,
    unit: None,
    sources: LIT_ONLY,
}];

const TERMINAL_SHELL_FIELDS: &[FieldMetadata] = &[
    FieldMetadata {
        target: TargetKind::InputProfile,
        name: "prompt_sprite_id",
        value_kind: ValueKind::Text,
        requirement: Requirement::Required,
        description: "Text sprite id used for the editable command prompt.",
        default_text: None,
        default_number: None,
        enum_options: None,
        min: None,
        max: None,
        step: None,
        unit: None,
        sources: LIT_ONLY,
    },
    FieldMetadata {
        target: TargetKind::InputProfile,
        name: "output_sprite_id",
        value_kind: ValueKind::Text,
        requirement: Requirement::Required,
        description: "Text sprite id used for the command transcript.",
        default_text: None,
        default_number: None,
        enum_options: None,
        min: None,
        max: None,
        step: None,
        unit: None,
        sources: LIT_ONLY,
    },
    FieldMetadata {
        target: TargetKind::InputProfile,
        name: "prompt_prefix",
        value_kind: ValueKind::Text,
        requirement: Requirement::Optional,
        description: "Prompt prefix rendered before the typed command.",
        default_text: Some("> "),
        default_number: None,
        enum_options: None,
        min: None,
        max: None,
        step: None,
        unit: None,
        sources: LIT_ONLY,
    },
    FieldMetadata {
        target: TargetKind::InputProfile,
        name: "max_lines",
        value_kind: ValueKind::Integer,
        requirement: Requirement::Optional,
        description: "Maximum output transcript line count retained on screen.",
        default_text: None,
        default_number: Some(120.0),
        enum_options: None,
        min: Some(1.0),
        max: Some(1000.0),
        step: Some(1.0),
        unit: Some("lines"),
        sources: LIT_ONLY,
    },
    FieldMetadata {
        target: TargetKind::InputProfile,
        name: "commands",
        value_kind: ValueKind::SelectList,
        requirement: Requirement::Optional,
        description: "Optional command table with scripted outputs.",
        default_text: None,
        default_number: None,
        enum_options: None,
        min: None,
        max: None,
        step: None,
        unit: None,
        sources: LIT_ONLY,
    },
];

/// Authoring sugar: aliases, shorthands, and normalizers.
#[derive(Debug, Clone)]
pub struct SugarCatalog {
    /// Field aliases (shorthand_name, canonical_name).
    pub aliases: Vec<(&'static str, &'static str)>,
    /// Shorthand transformations (name, description, from_syntax, to_structure).
    pub shorthands: Vec<ShorthandSpec>,
    /// Normalizer function names applied during document processing.
    pub normalizers: Vec<&'static str>,
}

/// Describes one authoring shorthand transformation.
#[derive(Debug, Clone)]
pub struct ShorthandSpec {
    /// Shorthand name (e.g., "pause").
    pub name: &'static str,
    /// Human-readable description.
    pub description: &'static str,
    /// Example authored syntax (e.g., "pause: 1ms").
    pub from_syntax: &'static str,
    /// Resulting canonical structure (e.g., "{duration: N, effects: []}").
    pub to_structure: &'static str,
}

/// Returns catalog of all authoring sugar transformations.
pub fn sugar_catalog() -> SugarCatalog {
    SugarCatalog {
        aliases: vec![("bg", "bg_colour"), ("fg", "fg_colour")],
        shorthands: vec![
            ShorthandSpec {
                name: "pause",
                description: "Duration-only step without effects",
                from_syntax: "pause: 1ms",
                to_structure: "{duration: 1, effects: []}",
            },
            ShorthandSpec {
                name: "at",
                description: "Alignment anchor shorthand",
                from_syntax: "at: cc",
                to_structure: "{align_x: center, align_y: center}",
            },
            ShorthandSpec {
                name: "to",
                description: "Menu option scene target",
                from_syntax: "to: main-menu",
                to_structure: "{scene: main-menu, next: main-menu}",
            },
            ShorthandSpec {
                name: "frame-sequence",
                description: "Expand numbered image sequence into timed image sprites",
                from_syntax: "type: frame-sequence",
                to_structure: "Vec<image sprites> with appear_at_ms/disappear_at_ms",
            },
            ShorthandSpec {
                name: "window",
                description:
                    "Expand semantic window sprite into framed grid + title/body/footer text",
                from_syntax: "type: window",
                to_structure: "type=panel with generated title/body/footer children",
            },
            ShorthandSpec {
                name: "terminal-input",
                description:
                    "Expand terminal input widget into window panel with hint + prompt slots",
                from_syntax: "type: terminal-input",
                to_structure: "type=window sugar with generated hint/prompt text children",
            },
            ShorthandSpec {
                name: "scroll-list",
                description: "Expand list sprite into grid rows of text items",
                from_syntax: "type: scroll-list",
                to_structure:
                    "type=grid with one text child per item (+ optional menu-carousel bindings)",
            },
            ShorthandSpec {
                name: "sprite-defaults",
                description: "Sprite field defaults inherited by child sprites",
                from_syntax: "sprite-defaults: { at: cc, font: generic:1 }",
                to_structure: "child sprite map with missing fields filled from parent",
            },
            ShorthandSpec {
                name: "menu-ui",
                description: "Generate animated menu body sprites from menu-options",
                from_syntax: "menu-ui: { layer: menu }",
                to_structure: "grid + menu-carousel items + selected-arrows sprites",
            },
        ],
        normalizers: vec![
            "normalize_stage",              // engine-authoring/src/document/scene.rs:60
            "normalize_layers",             // engine-authoring/src/document/scene.rs:93
            "normalize_sprites",            // engine-authoring/src/document/scene.rs:108
            "normalize_menu_options",       // engine-authoring/src/document/scene.rs:134
            "expand_menu_ui",               // engine-authoring/src/document/scene.rs
            "apply_alias",                  // engine-authoring/src/document/scene.rs:159
            "apply_at_anchor",              // engine-authoring/src/document/scene.rs:170
            "apply_defaults",               // engine-authoring/src/document/scene.rs
            "expand_frame_sequence",        // engine-authoring/src/document/scene.rs
            "expand_window_sprite",         // engine-authoring/src/document/scene.rs
            "expand_terminal_input_sprite", // engine-authoring/src/document/scene.rs
            "expand_scroll_list_sprite",    // engine-authoring/src/document/scene.rs
        ],
    }
}

/// Read-only entrypoint for authored field catalogs consumed by tooling.
#[derive(Debug, Clone, Copy)]
pub struct StaticAuthoringCatalog {
    pub scene_fields: &'static [FieldMetadata],
    pub layer_fields: &'static [FieldMetadata],
    pub sprite_fields: &'static [FieldMetadata],
    pub object_fields: &'static [FieldMetadata],
    pub effect_names: &'static [&'static str],
}

pub fn static_catalog() -> StaticAuthoringCatalog {
    StaticAuthoringCatalog {
        scene_fields: SCENE_FIELDS,
        layer_fields: LAYER_FIELDS,
        sprite_fields: SPRITE_FIELDS,
        object_fields: OBJECT_FIELDS,
        effect_names: &[],
    }
}

const BEHAVIOR_STAGE_OPTIONS: &[&str] = &[
    "on-enter", "enter", "on-idle", "idle", "on-leave", "leave", "done",
];

/// Returns (behavior_name, fields) tuples for all built-in behaviors.
pub fn behavior_catalog() -> Vec<(&'static str, Vec<FieldMetadata>)> {
    use crate::authoring::metadata::{Requirement, TargetKind, ValueKind, ValueSource};

    vec![
        (
            "blink",
            vec![
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "target",
                    value_kind: ValueKind::Text,
                    requirement: Requirement::Optional,
                    description: "Sprite ID to blink",
                    default_text: None,
                    default_number: None,
                    enum_options: None,
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "visible_ms",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "Milliseconds visible per cycle",
                    default_text: None,
                    default_number: Some(250.0),
                    enum_options: None,
                    min: Some(0.0),
                    max: None,
                    step: Some(10.0),
                    unit: Some("ms"),
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "hidden_ms",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "Milliseconds hidden per cycle",
                    default_text: None,
                    default_number: Some(250.0),
                    enum_options: None,
                    min: Some(0.0),
                    max: None,
                    step: Some(10.0),
                    unit: Some("ms"),
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "phase_ms",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "Time offset for the blink cycle",
                    default_text: None,
                    default_number: Some(0.0),
                    enum_options: None,
                    min: Some(0.0),
                    max: None,
                    step: Some(10.0),
                    unit: Some("ms"),
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "stages",
                    value_kind: ValueKind::SelectList,
                    requirement: Requirement::Optional,
                    description: "Scene stages when behavior is active",
                    default_text: None,
                    default_number: None,
                    enum_options: Some(BEHAVIOR_STAGE_OPTIONS),
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
            ],
        ),
        (
            "bob",
            vec![
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "target",
                    value_kind: ValueKind::Text,
                    requirement: Requirement::Optional,
                    description: "Sprite ID to bob",
                    default_text: None,
                    default_number: None,
                    enum_options: None,
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "amplitude_x",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "Horizontal oscillation amplitude",
                    default_text: None,
                    default_number: Some(0.0),
                    enum_options: None,
                    min: None,
                    max: None,
                    step: Some(1.0),
                    unit: Some("cells"),
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "amplitude_y",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "Vertical oscillation amplitude",
                    default_text: None,
                    default_number: Some(1.0),
                    enum_options: None,
                    min: None,
                    max: None,
                    step: Some(1.0),
                    unit: Some("cells"),
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "period_ms",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "Full oscillation cycle duration",
                    default_text: None,
                    default_number: Some(2000.0),
                    enum_options: None,
                    min: Some(1.0),
                    max: None,
                    step: Some(10.0),
                    unit: Some("ms"),
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "phase_ms",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "Time offset for wave phase",
                    default_text: None,
                    default_number: Some(0.0),
                    enum_options: None,
                    min: Some(0.0),
                    max: None,
                    step: Some(10.0),
                    unit: Some("ms"),
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "stages",
                    value_kind: ValueKind::SelectList,
                    requirement: Requirement::Optional,
                    description: "Scene stages when behavior is active",
                    default_text: None,
                    default_number: None,
                    enum_options: Some(BEHAVIOR_STAGE_OPTIONS),
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
            ],
        ),
        (
            "follow",
            vec![
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "target",
                    value_kind: ValueKind::Text,
                    requirement: Requirement::Required,
                    description: "Sprite ID to follow",
                    default_text: None,
                    default_number: None,
                    enum_options: None,
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "amplitude_x",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "Horizontal offset from the followed sprite",
                    default_text: None,
                    default_number: Some(0.0),
                    enum_options: None,
                    min: None,
                    max: None,
                    step: Some(1.0),
                    unit: Some("cells"),
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "amplitude_y",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "Vertical offset from the followed sprite",
                    default_text: None,
                    default_number: Some(0.0),
                    enum_options: None,
                    min: None,
                    max: None,
                    step: Some(1.0),
                    unit: Some("cells"),
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "stages",
                    value_kind: ValueKind::SelectList,
                    requirement: Requirement::Optional,
                    description: "Scene stages when behavior is active",
                    default_text: None,
                    default_number: None,
                    enum_options: Some(BEHAVIOR_STAGE_OPTIONS),
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
            ],
        ),
        (
            "menu-carousel",
            vec![
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "target",
                    value_kind: ValueKind::Text,
                    requirement: Requirement::Required,
                    description: "Container or sprite ID used as the center anchor",
                    default_text: None,
                    default_number: None,
                    enum_options: None,
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "index",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Required,
                    description: "Menu option index represented by this sprite",
                    default_text: None,
                    default_number: None,
                    enum_options: None,
                    min: Some(0.0),
                    max: None,
                    step: Some(1.0),
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "count",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "Total option count override (defaults to scene menu options)",
                    default_text: None,
                    default_number: None,
                    enum_options: None,
                    min: Some(1.0),
                    max: None,
                    step: Some(1.0),
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "window",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "Visible option window size around selected item",
                    default_text: None,
                    default_number: Some(5.0),
                    enum_options: None,
                    min: Some(1.0),
                    max: None,
                    step: Some(1.0),
                    unit: Some("items"),
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "step_y",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "Vertical spacing between neighboring options",
                    default_text: None,
                    default_number: Some(2.0),
                    enum_options: None,
                    min: Some(1.0),
                    max: None,
                    step: Some(1.0),
                    unit: Some("cells"),
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "endless",
                    value_kind: ValueKind::Boolean,
                    requirement: Requirement::Optional,
                    description: "Wrap list when navigating past first/last option",
                    default_text: Some("true"),
                    default_number: None,
                    enum_options: Some(&["true", "false"]),
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
            ],
        ),
        (
            "menu-selected",
            vec![
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "index",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Required,
                    description: "Menu option index this behavior tracks",
                    default_text: None,
                    default_number: None,
                    enum_options: None,
                    min: Some(0.0),
                    max: None,
                    step: Some(1.0),
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "stages",
                    value_kind: ValueKind::SelectList,
                    requirement: Requirement::Optional,
                    description: "Scene stages when behavior is active",
                    default_text: None,
                    default_number: None,
                    enum_options: Some(BEHAVIOR_STAGE_OPTIONS),
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
            ],
        ),
        (
            "menu-carousel-object",
            vec![
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "target",
                    value_kind: ValueKind::Text,
                    requirement: Requirement::Required,
                    description: "Container ID used as the center anchor",
                    default_text: None,
                    default_number: None,
                    enum_options: None,
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "item_prefix",
                    value_kind: ValueKind::Text,
                    requirement: Requirement::Optional,
                    description: "Prefix for menu item aliases (supports {} placeholder)",
                    default_text: Some("menu-item-"),
                    default_number: None,
                    enum_options: None,
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "count",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "Total option count override (defaults to scene menu options)",
                    default_text: None,
                    default_number: None,
                    enum_options: None,
                    min: Some(1.0),
                    max: None,
                    step: Some(1.0),
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "window",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "Visible option window size around selected item",
                    default_text: None,
                    default_number: Some(5.0),
                    enum_options: None,
                    min: Some(1.0),
                    max: None,
                    step: Some(1.0),
                    unit: Some("items"),
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "step_y",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "Minimum vertical spacing between neighboring options",
                    default_text: None,
                    default_number: Some(2.0),
                    enum_options: None,
                    min: Some(1.0),
                    max: None,
                    step: Some(1.0),
                    unit: Some("cells"),
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "endless",
                    value_kind: ValueKind::Boolean,
                    requirement: Requirement::Optional,
                    description: "Wrap list when navigating past first/last option",
                    default_text: Some("true"),
                    default_number: None,
                    enum_options: Some(&["true", "false"]),
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
            ],
        ),
        (
            "rhai-script",
            vec![
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "src",
                    value_kind: ValueKind::Text,
                    requirement: Requirement::Optional,
                    description: "Script source path (for diagnostics/introspection).",
                    default_text: None,
                    default_number: None,
                    enum_options: None,
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "script",
                    value_kind: ValueKind::Text,
                    requirement: Requirement::Required,
                    description: "Embedded Rhai script body.",
                    default_text: None,
                    default_number: None,
                    enum_options: None,
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "target",
                    value_kind: ValueKind::Text,
                    requirement: Requirement::Optional,
                    description: "Primary target alias (script-defined semantics).",
                    default_text: None,
                    default_number: None,
                    enum_options: None,
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "item_prefix",
                    value_kind: ValueKind::Text,
                    requirement: Requirement::Optional,
                    description: "Optional item alias prefix used by scripts.",
                    default_text: Some("menu-item-"),
                    default_number: None,
                    enum_options: None,
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "count",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "Optional item count hint for scripts.",
                    default_text: None,
                    default_number: None,
                    enum_options: None,
                    min: Some(1.0),
                    max: None,
                    step: Some(1.0),
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "window",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "Optional visible window hint for scripts.",
                    default_text: None,
                    default_number: Some(5.0),
                    enum_options: None,
                    min: Some(1.0),
                    max: None,
                    step: Some(1.0),
                    unit: Some("items"),
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "step_y",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "Optional spacing hint for scripts.",
                    default_text: None,
                    default_number: Some(2.0),
                    enum_options: None,
                    min: Some(1.0),
                    max: None,
                    step: Some(1.0),
                    unit: Some("cells"),
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "endless",
                    value_kind: ValueKind::Boolean,
                    requirement: Requirement::Optional,
                    description: "Optional endless-wrap hint for scripts.",
                    default_text: Some("true"),
                    default_number: None,
                    enum_options: Some(&["true", "false"]),
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
            ],
        ),
        (
            "selected-arrows",
            vec![
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "target",
                    value_kind: ValueKind::Text,
                    requirement: Requirement::Required,
                    description: "Sprite ID that the arrow should flank",
                    default_text: None,
                    default_number: None,
                    enum_options: None,
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "index",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Required,
                    description: "Menu option index this arrow tracks",
                    default_text: None,
                    default_number: None,
                    enum_options: None,
                    min: Some(0.0),
                    max: None,
                    step: Some(1.0),
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "side",
                    value_kind: ValueKind::Select,
                    requirement: Requirement::Optional,
                    description: "Which side the arrow appears on",
                    default_text: Some("left"),
                    default_number: None,
                    enum_options: Some(&["left", "right"]),
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "padding",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "Cell padding from target sprite edge",
                    default_text: None,
                    default_number: Some(1.0),
                    enum_options: None,
                    min: None,
                    max: None,
                    step: Some(1.0),
                    unit: Some("cells"),
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "amplitude_x",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "Horizontal sway amplitude of the arrow",
                    default_text: None,
                    default_number: Some(1.0),
                    enum_options: None,
                    min: None,
                    max: None,
                    step: Some(1.0),
                    unit: Some("cells"),
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "period_ms",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "Full sway cycle duration",
                    default_text: None,
                    default_number: Some(900.0),
                    enum_options: None,
                    min: Some(1.0),
                    max: None,
                    step: Some(10.0),
                    unit: Some("ms"),
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "phase_ms",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "Time offset for sway phase",
                    default_text: None,
                    default_number: Some(0.0),
                    enum_options: None,
                    min: Some(0.0),
                    max: None,
                    step: Some(10.0),
                    unit: Some("ms"),
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "autoscale_height",
                    value_kind: ValueKind::Boolean,
                    requirement: Requirement::Optional,
                    description: "Expand horizontal anchor distance by target item height",
                    default_text: None,
                    default_number: None,
                    enum_options: None,
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "stages",
                    value_kind: ValueKind::SelectList,
                    requirement: Requirement::Optional,
                    description: "Scene stages when behavior is active",
                    default_text: None,
                    default_number: None,
                    enum_options: Some(BEHAVIOR_STAGE_OPTIONS),
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
            ],
        ),
        (
            "stage-visibility",
            vec![
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "target",
                    value_kind: ValueKind::Text,
                    requirement: Requirement::Optional,
                    description: "Sprite ID to control visibility",
                    default_text: None,
                    default_number: None,
                    enum_options: None,
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "stages",
                    value_kind: ValueKind::SelectList,
                    requirement: Requirement::Required,
                    description: "Scene stages when sprite is visible",
                    default_text: None,
                    default_number: None,
                    enum_options: Some(BEHAVIOR_STAGE_OPTIONS),
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
            ],
        ),
        (
            "timed-visibility",
            vec![
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "target",
                    value_kind: ValueKind::Text,
                    requirement: Requirement::Optional,
                    description: "Sprite ID to control visibility",
                    default_text: None,
                    default_number: None,
                    enum_options: None,
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "time_scope",
                    value_kind: ValueKind::Select,
                    requirement: Requirement::Optional,
                    description: "Whether times are scene-relative or stage-relative",
                    default_text: Some("scene"),
                    default_number: None,
                    enum_options: Some(&["scene", "stage"]),
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "start_ms",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "When sprite becomes visible",
                    default_text: None,
                    default_number: Some(0.0),
                    enum_options: None,
                    min: Some(0.0),
                    max: None,
                    step: Some(10.0),
                    unit: Some("ms"),
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "end_ms",
                    value_kind: ValueKind::Integer,
                    requirement: Requirement::Optional,
                    description: "When sprite becomes hidden",
                    default_text: None,
                    default_number: None,
                    enum_options: None,
                    min: Some(0.0),
                    max: None,
                    step: Some(10.0),
                    unit: Some("ms"),
                    sources: &[ValueSource::Literal],
                },
                FieldMetadata {
                    target: TargetKind::Effect,
                    name: "stages",
                    value_kind: ValueKind::SelectList,
                    requirement: Requirement::Optional,
                    description: "Scene stages when behavior is active",
                    default_text: None,
                    default_number: None,
                    enum_options: Some(BEHAVIOR_STAGE_OPTIONS),
                    min: None,
                    max: None,
                    step: None,
                    unit: None,
                    sources: &[ValueSource::Literal],
                },
            ],
        ),
    ]
}

/// Returns (animation_name, fields) tuples for all built-in animations.
pub fn animation_catalog() -> Vec<(&'static str, Vec<FieldMetadata>)> {
    AnimationDispatcher::builtin_names()
        .into_iter()
        .map(|name| (name, AnimationDispatcher::metadata(name)))
        .collect()
}

/// Returns names of all built-in input profiles.
pub fn input_profile_catalog() -> Vec<&'static str> {
    SceneInput::builtin_profiles()
}

/// Returns full parameter-shape metadata for all built-in input profiles.
pub fn input_profile_shapes() -> Vec<InputProfileShape> {
    vec![
        InputProfileShape {
            name: "obj-viewer",
            fields: OBJ_VIEWER_FIELDS,
        },
        InputProfileShape {
            name: "terminal-size-tester",
            fields: TERMINAL_SIZE_TESTER_FIELDS,
        },
        InputProfileShape {
            name: "terminal-shell",
            fields: TERMINAL_SHELL_FIELDS,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        animation_catalog, behavior_catalog, input_profile_catalog,
        input_profile_shapes, static_catalog, sugar_catalog,
    };
    use crate::authoring::metadata::{Requirement, ValueKind};

    #[test]
    fn static_catalog_exposes_scene_and_sprite_fields() {
        let c = static_catalog();
        assert!(c.scene_fields.iter().any(|f| f.name == "id"));
        assert!(c.sprite_fields.iter().any(|f| f.name == "type"));
    }

    #[test]
    fn animation_catalog_complete() {
        let catalog = animation_catalog();
        assert!(!catalog.is_empty());
        assert!(catalog.iter().any(|(name, _)| *name == "float"));
        for (name, fields) in &catalog {
            assert!(!fields.is_empty(), "Animation {} has no fields", name);
        }
    }

    #[test]
    fn input_profile_catalog_has_builtin_profiles() {
        let profiles = input_profile_catalog();
        assert!(profiles.contains(&"obj-viewer"));
        assert!(profiles.contains(&"terminal-size-tester"));
        assert!(profiles.contains(&"terminal-shell"));
    }

    #[test]
    fn input_profile_shapes_have_correct_fields() {
        let shapes = input_profile_shapes();
        let obj_viewer = shapes
            .iter()
            .find(|s| s.name == "obj-viewer")
            .expect("obj-viewer");
        assert!(obj_viewer.fields.iter().any(|f| f.name == "sprite_id"));
        assert_eq!(
            obj_viewer
                .fields
                .iter()
                .find(|f| f.name == "sprite_id")
                .unwrap()
                .requirement,
            Requirement::Required
        );

        let tst = shapes
            .iter()
            .find(|s| s.name == "terminal-size-tester")
            .expect("terminal-size-tester");
        let presets = tst
            .fields
            .iter()
            .find(|f| f.name == "presets")
            .expect("presets");
        assert_eq!(presets.value_kind, ValueKind::SelectList);
        assert_eq!(presets.requirement, Requirement::Optional);

        let terminal_shell = shapes
            .iter()
            .find(|s| s.name == "terminal-shell")
            .expect("terminal-shell");
        assert!(terminal_shell
            .fields
            .iter()
            .any(|f| f.name == "prompt_sprite_id"));
        assert!(terminal_shell
            .fields
            .iter()
            .any(|f| f.name == "output_sprite_id"));
    }

    #[test]
    fn behavior_catalog_has_all_behaviors() {
        let catalog = behavior_catalog();
        assert!(!catalog.is_empty());

        let names: Vec<&str> = catalog.iter().map(|(name, _)| *name).collect();
        assert!(names.contains(&"blink"));
        assert!(names.contains(&"bob"));
        assert!(names.contains(&"follow"));
        assert!(names.contains(&"menu-carousel"));
        assert!(names.contains(&"menu-carousel-object"));
        assert!(names.contains(&"rhai-script"));
        assert!(names.contains(&"menu-selected"));
        assert!(names.contains(&"selected-arrows"));
        assert!(names.contains(&"stage-visibility"));
        assert!(names.contains(&"timed-visibility"));

        for (name, fields) in &catalog {
            assert!(!fields.is_empty(), "Behavior {} has no fields", name);
        }
    }

    #[test]
    fn behavior_catalog_matches_runtime_parameter_shapes() {
        let catalog = behavior_catalog();

        let blink = catalog
            .iter()
            .find(|(name, _)| *name == "blink")
            .expect("blink metadata");
        assert!(blink.1.iter().any(|field| field.name == "phase_ms"));
        assert_eq!(
            blink
                .1
                .iter()
                .find(|field| field.name == "visible_ms")
                .and_then(|field| field.default_number),
            Some(250.0)
        );
        assert_eq!(
            blink
                .1
                .iter()
                .find(|field| field.name == "hidden_ms")
                .and_then(|field| field.default_number),
            Some(250.0)
        );

        let follow = catalog
            .iter()
            .find(|(name, _)| *name == "follow")
            .expect("follow metadata");
        assert!(follow.1.iter().any(|field| field.name == "amplitude_x"));
        assert!(follow.1.iter().any(|field| field.name == "amplitude_y"));
        assert!(follow
            .1
            .iter()
            .find(|field| field.name == "target")
            .is_some_and(|field| matches!(field.requirement, Requirement::Required)));

        let selected_arrows = catalog
            .iter()
            .find(|(name, _)| *name == "selected-arrows")
            .expect("selected-arrows metadata");
        for field_name in [
            "target",
            "index",
            "side",
            "padding",
            "amplitude_x",
            "period_ms",
            "phase_ms",
            "autoscale_height",
        ] {
            assert!(
                selected_arrows
                    .1
                    .iter()
                    .any(|field| field.name == field_name),
                "selected-arrows metadata missing {field_name}"
            );
        }

        let menu_carousel = catalog
            .iter()
            .find(|(name, _)| *name == "menu-carousel")
            .expect("menu-carousel metadata");
        for field_name in ["target", "index", "count", "window", "step_y", "endless"] {
            assert!(
                menu_carousel.1.iter().any(|field| field.name == field_name),
                "menu-carousel metadata missing {field_name}"
            );
        }

        let menu_carousel_object = catalog
            .iter()
            .find(|(name, _)| *name == "menu-carousel-object")
            .expect("menu-carousel-object metadata");
        for field_name in [
            "target",
            "item_prefix",
            "count",
            "window",
            "step_y",
            "endless",
        ] {
            assert!(
                menu_carousel_object
                    .1
                    .iter()
                    .any(|field| field.name == field_name),
                "menu-carousel-object metadata missing {field_name}"
            );
        }

        let rhai_script = catalog
            .iter()
            .find(|(name, _)| *name == "rhai-script")
            .expect("rhai-script metadata");
        for field_name in [
            "src",
            "script",
            "target",
            "item_prefix",
            "count",
            "window",
            "step_y",
            "endless",
        ] {
            assert!(
                rhai_script.1.iter().any(|field| field.name == field_name),
                "rhai-script metadata missing {field_name}"
            );
        }

        for (_, fields) in &catalog {
            let stages = fields.iter().find(|field| field.name == "stages");
            if let Some(stages) = stages {
                assert_eq!(stages.value_kind, ValueKind::SelectList);
                assert_eq!(
                    stages.enum_options,
                    Some(super::BEHAVIOR_STAGE_OPTIONS),
                    "behavior stages should use runtime stage spellings"
                );
            }
        }
    }

    #[test]
    fn sugar_catalog_has_all_aliases_and_shorthands() {
        let catalog = sugar_catalog();

        // Check aliases
        assert!(catalog
            .aliases
            .iter()
            .any(|(from, to)| *from == "bg" && *to == "bg_colour"));
        assert!(catalog
            .aliases
            .iter()
            .any(|(from, to)| *from == "fg" && *to == "fg_colour"));

        // Check shorthands
        assert!(catalog.shorthands.iter().any(|s| s.name == "pause"));
        assert!(catalog.shorthands.iter().any(|s| s.name == "at"));
        assert!(catalog.shorthands.iter().any(|s| s.name == "to"));

        // Check normalizers
        assert!(catalog.normalizers.contains(&"normalize_stage"));
        assert!(catalog.normalizers.contains(&"apply_alias"));
        assert!(catalog.normalizers.contains(&"apply_at_anchor"));
    }
}
