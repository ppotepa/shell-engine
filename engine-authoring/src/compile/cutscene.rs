//! Cutscene manifest expansion into scene layer sprites with compile-time filters.

use serde::de::Error as _;
use serde::Deserialize;
use serde_yaml::{Mapping, Number, Value};
use std::collections::BTreeMap;

/// Mutable per-frame compile payload passed through cutscene compile filters.
#[derive(Debug, Clone)]
pub struct CutsceneCompileFrame {
    pub index: usize,
    pub source: String,
    pub delay_ms: u64,
    pub at: Option<String>,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub z_index: Option<i32>,
    pub width: Option<u16>,
    pub height: Option<u16>,
}

/// Extension point for compile-time cutscene frame transforms.
pub trait CutsceneCompileFilter {
    fn name(&self) -> &'static str;
    fn apply(
        &self,
        frame: &mut CutsceneCompileFrame,
        params: &Mapping,
    ) -> Result<(), serde_yaml::Error>;
}

/// Named cutscene filter registry used by the authored scene compiler.
#[derive(Default)]
pub struct CutsceneFilterRegistry {
    filters: BTreeMap<&'static str, Box<dyn CutsceneCompileFilter + Send + Sync>>,
}

impl CutsceneFilterRegistry {
    pub fn with_builtin_filters() -> Self {
        let mut registry = Self::default();
        registry.register(DurationScaleFilter);
        registry
    }

    pub fn register<F>(&mut self, filter: F)
    where
        F: CutsceneCompileFilter + Send + Sync + 'static,
    {
        self.filters.insert(filter.name(), Box::new(filter));
    }

    fn apply_specs(
        &self,
        frame: &mut CutsceneCompileFrame,
        specs: &[CutsceneFilterSpec],
    ) -> Result<(), serde_yaml::Error> {
        for spec in specs {
            let Some(filter) = self.filters.get(spec.name.as_str()) else {
                return Err(yaml_error(format!(
                    "unknown cutscene filter '{}' for frame {}",
                    spec.name, frame.index
                )));
            };
            filter.apply(frame, &spec.params)?;
        }
        Ok(())
    }
}

/// Built-in filter: multiply frame delay by `params.factor`.
pub struct DurationScaleFilter;

impl CutsceneCompileFilter for DurationScaleFilter {
    fn name(&self) -> &'static str {
        "duration-scale"
    }

    fn apply(
        &self,
        frame: &mut CutsceneCompileFrame,
        params: &Mapping,
    ) -> Result<(), serde_yaml::Error> {
        let Some(raw_factor) = params
            .get(Value::String("factor".to_string()))
            .and_then(value_as_f64)
        else {
            return Err(yaml_error(
                "duration-scale filter requires numeric params.factor",
            ));
        };
        if !raw_factor.is_finite() || raw_factor <= 0.0 {
            return Err(yaml_error(
                "duration-scale filter requires params.factor > 0",
            ));
        }
        let scaled = ((frame.delay_ms as f64) * raw_factor).round();
        frame.delay_ms = scaled.max(1.0) as u64;
        Ok(())
    }
}

/// Expands `cutscene-ref` scene property into a generated image layer.
pub fn expand_scene_cutscene_ref_with_filters<F>(
    root: &mut Value,
    scene_source_path: &str,
    asset_loader: &mut F,
    filters: &CutsceneFilterRegistry,
) -> Result<(), serde_yaml::Error>
where
    F: FnMut(&str) -> Option<String>,
{
    let Some(scene_map) = root.as_mapping_mut() else {
        return Ok(());
    };

    let cutscene_ref = scene_map
        .get(Value::String("cutscene-ref".to_string()))
        .or_else(|| scene_map.get(Value::String("cutscene_ref".to_string())))
        .and_then(Value::as_str)
        .map(str::trim);

    let Some(cutscene_ref) = cutscene_ref else {
        return Ok(());
    };
    if cutscene_ref.is_empty() {
        return Err(yaml_error("cutscene-ref cannot be empty"));
    }

    let cutscene_path = resolve_cutscene_ref_path(scene_source_path, cutscene_ref);
    let raw = asset_loader(&cutscene_path).ok_or_else(|| {
        yaml_error(format!(
            "cutscene-ref '{}' resolved to '{}', but file was not found",
            cutscene_ref, cutscene_path
        ))
    })?;

    let doc = serde_yaml::from_str::<CutsceneDocument>(&raw).map_err(|err| {
        yaml_error(format!(
            "failed to parse cutscene '{}': {err}",
            cutscene_path
        ))
    })?;
    if doc.frames.is_empty() {
        return Err(yaml_error(format!(
            "cutscene '{}' has no frames",
            cutscene_path
        )));
    }

    let generated_layer = compile_cutscene_layer(&doc, &cutscene_path, filters)?;
    let layers_value = scene_map
        .entry(Value::String("layers".to_string()))
        .or_insert_with(|| Value::Sequence(Vec::new()));
    let Some(layers) = layers_value.as_sequence_mut() else {
        return Err(yaml_error(
            "scene 'layers' must be an array when using cutscene-ref",
        ));
    };
    layers.push(generated_layer);

    scene_map.remove(Value::String("cutscene-ref".to_string()));
    scene_map.remove(Value::String("cutscene_ref".to_string()));
    scene_map
        .entry(Value::String("cutscene".to_string()))
        .or_insert(Value::Bool(true));
    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
struct CutsceneDocument {
    #[serde(default, rename = "layer-name", alias = "layer_name")]
    layer_name: Option<String>,
    #[serde(default, rename = "layer-z-index", alias = "layer_z_index")]
    layer_z_index: Option<i32>,
    #[serde(default, rename = "id-prefix", alias = "id_prefix")]
    id_prefix: Option<String>,
    #[serde(default)]
    defaults: CutsceneSpriteDefaults,
    #[serde(default)]
    filters: Vec<CutsceneFilterSpec>,
    #[serde(default)]
    frames: Vec<CutsceneFrameDocument>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct CutsceneSpriteDefaults {
    #[serde(default)]
    at: Option<String>,
    #[serde(default)]
    x: Option<i32>,
    #[serde(default)]
    y: Option<i32>,
    #[serde(default, rename = "z-index", alias = "z_index")]
    z_index: Option<i32>,
    #[serde(default)]
    width: Option<u16>,
    #[serde(default)]
    height: Option<u16>,
}

#[derive(Debug, Clone, Deserialize)]
struct CutsceneFrameDocument {
    source: String,
    #[serde(default = "default_delay_ms", rename = "delay-ms", alias = "delay_ms")]
    delay_ms: u64,
    #[serde(default)]
    at: Option<String>,
    #[serde(default)]
    x: Option<i32>,
    #[serde(default)]
    y: Option<i32>,
    #[serde(default, rename = "z-index", alias = "z_index")]
    z_index: Option<i32>,
    #[serde(default)]
    width: Option<u16>,
    #[serde(default)]
    height: Option<u16>,
    #[serde(default)]
    filters: Vec<CutsceneFilterSpec>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct CutsceneFilterSpec {
    name: String,
    #[serde(default)]
    params: Mapping,
}

fn default_delay_ms() -> u64 {
    100
}

fn compile_cutscene_layer(
    doc: &CutsceneDocument,
    cutscene_path: &str,
    filters: &CutsceneFilterRegistry,
) -> Result<Value, serde_yaml::Error> {
    let mut elapsed_ms = 0u64;
    let id_prefix = doc
        .id_prefix
        .clone()
        .unwrap_or_else(|| format!("cutscene:{}:frame-", derive_cutscene_name(cutscene_path)));
    let layer_name = doc
        .layer_name
        .clone()
        .unwrap_or_else(|| format!("cutscene:{}", derive_cutscene_name(cutscene_path)));

    let mut sprites = Vec::with_capacity(doc.frames.len());
    for (idx, frame_doc) in doc.frames.iter().enumerate() {
        let mut frame = CutsceneCompileFrame {
            index: idx + 1,
            source: frame_doc.source.clone(),
            delay_ms: frame_doc.delay_ms,
            at: frame_doc.at.clone().or_else(|| doc.defaults.at.clone()),
            x: frame_doc.x.or(doc.defaults.x),
            y: frame_doc.y.or(doc.defaults.y),
            z_index: frame_doc.z_index.or(doc.defaults.z_index),
            width: frame_doc.width.or(doc.defaults.width),
            height: frame_doc.height.or(doc.defaults.height),
        };
        if frame.source.trim().is_empty() {
            return Err(yaml_error(format!(
                "cutscene frame {} has an empty source path",
                frame.index
            )));
        }
        if frame.delay_ms == 0 {
            return Err(yaml_error(format!(
                "cutscene frame {} has delay-ms = 0; use a value > 0",
                frame.index
            )));
        }

        filters.apply_specs(&mut frame, &doc.filters)?;
        filters.apply_specs(&mut frame, &frame_doc.filters)?;
        if frame.delay_ms == 0 {
            return Err(yaml_error(format!(
                "cutscene frame {} resolved to delay-ms = 0 after filters",
                frame.index
            )));
        }

        let appear_at_ms = elapsed_ms;
        let disappear_at_ms = elapsed_ms.saturating_add(frame.delay_ms);
        elapsed_ms = disappear_at_ms;

        sprites.push(Value::Mapping(build_image_sprite_mapping(
            &frame,
            &id_prefix,
            appear_at_ms,
            disappear_at_ms,
        )));
    }

    let mut layer = Mapping::new();
    layer.insert(
        Value::String("name".to_string()),
        Value::String(layer_name.to_string()),
    );
    if let Some(z_index) = doc.layer_z_index {
        layer.insert(
            Value::String("z_index".to_string()),
            Value::Number(Number::from(z_index)),
        );
    }
    layer.insert(
        Value::String("sprites".to_string()),
        Value::Sequence(sprites),
    );
    Ok(Value::Mapping(layer))
}

fn build_image_sprite_mapping(
    frame: &CutsceneCompileFrame,
    id_prefix: &str,
    appear_at_ms: u64,
    disappear_at_ms: u64,
) -> Mapping {
    let mut sprite = Mapping::new();
    sprite.insert(
        Value::String("type".to_string()),
        Value::String("image".to_string()),
    );
    sprite.insert(
        Value::String("id".to_string()),
        Value::String(format!("{id_prefix}{:04}", frame.index)),
    );
    sprite.insert(
        Value::String("source".to_string()),
        Value::String(frame.source.clone()),
    );
    if let Some(at) = frame.at.as_ref() {
        sprite.insert(
            Value::String("at".to_string()),
            Value::String(at.to_string()),
        );
    }
    if let Some(x) = frame.x {
        sprite.insert(
            Value::String("x".to_string()),
            Value::Number(Number::from(x)),
        );
    }
    if let Some(y) = frame.y {
        sprite.insert(
            Value::String("y".to_string()),
            Value::Number(Number::from(y)),
        );
    }
    if let Some(z_index) = frame.z_index {
        sprite.insert(
            Value::String("z_index".to_string()),
            Value::Number(Number::from(z_index)),
        );
    }
    if let Some(width) = frame.width {
        sprite.insert(
            Value::String("width".to_string()),
            Value::Number(Number::from(width)),
        );
    }
    if let Some(height) = frame.height {
        sprite.insert(
            Value::String("height".to_string()),
            Value::Number(Number::from(height)),
        );
    }
    sprite.insert(
        Value::String("appear_at_ms".to_string()),
        Value::Number(Number::from(appear_at_ms)),
    );
    sprite.insert(
        Value::String("disappear_at_ms".to_string()),
        Value::Number(Number::from(disappear_at_ms)),
    );
    sprite
}

fn value_as_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse::<f64>().ok(),
        _ => None,
    }
}

fn resolve_cutscene_ref_path(scene_source_path: &str, reference: &str) -> String {
    if reference.starts_with('/') {
        return normalize_mod_path(reference);
    }
    if reference.starts_with("./") || reference.starts_with("../") {
        let scene_dir = parent_dir(scene_source_path);
        return normalize_mod_path(&format!("{scene_dir}/{reference}"));
    }

    let trimmed = reference.trim_start_matches('/');
    let has_yaml_ext = trimmed.ends_with(".yml") || trimmed.ends_with(".yaml");
    if has_yaml_ext {
        if trimmed.starts_with("cutscenes/") {
            return normalize_mod_path(&format!("/{trimmed}"));
        }
        return normalize_mod_path(&format!("/cutscenes/{trimmed}"));
    }

    normalize_mod_path(&format!("/cutscenes/{trimmed}.yml"))
}

fn derive_cutscene_name(cutscene_path: &str) -> String {
    let normalized = normalize_mod_path(cutscene_path);
    let last = normalized.rsplit('/').next().unwrap_or("cutscene");
    last.trim_end_matches(".yml")
        .trim_end_matches(".yaml")
        .to_string()
}

fn parent_dir(path: &str) -> String {
    let normalized = normalize_mod_path(path);
    match normalized.rsplit_once('/') {
        Some(("", _)) | None => "/".to_string(),
        Some((dir, _)) => dir.to_string(),
    }
}

fn normalize_mod_path(path: &str) -> String {
    let mut parts = Vec::new();
    for part in path.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            let _ = parts.pop();
            continue;
        }
        parts.push(part);
    }
    if parts.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", parts.join("/"))
    }
}

fn yaml_error(message: impl Into<String>) -> serde_yaml::Error {
    serde_yaml::Error::custom(message.into())
}
