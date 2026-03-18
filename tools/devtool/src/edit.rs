use anyhow::{bail, Context, Result};
use serde_yaml::Value;
use std::path::{Path, PathBuf};

use crate::cli::EditSpriteArgs;
use crate::fs_utils::ensure_mod_exists;
use crate::scaffold::{
    find_sprite_index, find_target_layer, load_yaml_document_with_header,
    write_yaml_document_with_header,
};

#[derive(Debug)]
pub struct EditedSprite {
    pub layer_path: PathBuf,
    pub sprite_id: String,
    pub updated_fields: Vec<&'static str>,
}

pub fn edit_sprite(repo_root: &Path, args: &EditSpriteArgs) -> Result<EditedSprite> {
    if !has_requested_sprite_edit(args) {
        bail!("no sprite changes requested (use --at, --x, --y, --width, --height, or --clear-height)");
    }

    let mod_root = repo_root.join("mods").join(&args.r#mod);
    ensure_mod_exists(&mod_root)?;

    let scene_dir = mod_root.join("scenes").join(&args.scene);
    if !scene_dir.exists() {
        bail!(
            "scene package not found: {} (create it first with `devtool create scene`)",
            scene_dir.display()
        );
    }

    let layer_path = scene_dir.join(format!("layers/{}.yml", args.layer));
    if !layer_path.exists() {
        bail!("layer file not found: {}", layer_path.display());
    }

    let (header, mut doc) = load_yaml_document_with_header(&layer_path)?;
    let layers = doc.as_sequence_mut().ok_or_else(|| {
        anyhow::anyhow!(
            "expected top-level YAML sequence in {}",
            layer_path.display()
        )
    })?;
    let layer = find_target_layer(layers, &args.layer).ok_or_else(|| {
        anyhow::anyhow!(
            "layer entry not found in {} for {}",
            layer_path.display(),
            args.layer
        )
    })?;
    let layer_map = layer
        .as_mapping_mut()
        .ok_or_else(|| anyhow::anyhow!("invalid layer mapping in {}", layer_path.display()))?;
    let sprites = layer_map
        .get_mut(Value::String("sprites".into()))
        .and_then(Value::as_sequence_mut)
        .ok_or_else(|| anyhow::anyhow!("`sprites` is not a list in {}", layer_path.display()))?;

    let sprite_index = find_sprite_index(sprites, &args.id).ok_or_else(|| {
        anyhow::anyhow!(
            "sprite id not found in {}: {}",
            layer_path.display(),
            args.id
        )
    })?;
    let sprite = sprites[sprite_index]
        .as_mapping_mut()
        .ok_or_else(|| anyhow::anyhow!("invalid sprite mapping in {}", layer_path.display()))?;

    let mut updated_fields = Vec::new();
    if let Some(at) = &args.at {
        sprite.insert(Value::String("at".into()), Value::String(at.clone()));
        updated_fields.push("at");
    }
    if let Some(x) = &args.x {
        sprite.insert(Value::String("x".into()), scalar_value(x));
        updated_fields.push("x");
    }
    if let Some(y) = &args.y {
        sprite.insert(Value::String("y".into()), scalar_value(y));
        updated_fields.push("y");
    }
    if let Some(width) = args.width {
        sprite.insert(Value::String("width".into()), Value::Number(width.into()));
        updated_fields.push("width");
    }
    if let Some(height) = args.height {
        sprite.insert(Value::String("height".into()), Value::Number(height.into()));
        updated_fields.push("height");
    } else if args.clear_height {
        sprite.remove(Value::String("height".into()));
        updated_fields.push("height");
    }

    write_yaml_document_with_header(&layer_path, header.as_deref(), &doc).with_context(|| {
        format!(
            "failed to persist edited sprite in {}",
            layer_path.display()
        )
    })?;

    Ok(EditedSprite {
        layer_path,
        sprite_id: args.id.clone(),
        updated_fields,
    })
}

fn has_requested_sprite_edit(args: &EditSpriteArgs) -> bool {
    args.at.is_some()
        || args.x.is_some()
        || args.y.is_some()
        || args.width.is_some()
        || args.height.is_some()
        || args.clear_height
}

fn scalar_value(raw: &str) -> Value {
    if let Ok(value) = raw.parse::<i64>() {
        return Value::Number(value.into());
    }
    Value::String(raw.to_string())
}
