use anyhow::{bail, Context, Result};
use serde_yaml::{Mapping, Sequence, Value};
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::{NewEffectArgs, NewLayerArgs, NewModArgs, NewSpriteArgs};
use crate::fs_utils::{ensure_mod_exists, write_file};

pub fn create_mod_scaffold(repo_root: &Path, args: &NewModArgs) -> Result<()> {
    let mod_root = repo_root.join("mods").join(&args.name);
    if mod_root.exists() && !args.force {
        bail!(
            "mod already exists: {} (use --force to overwrite scaffold files)",
            mod_root.display()
        );
    }

    let scene_id = default_scene_id(&args.name, &args.scene);
    let scene_title = human_title(&args.scene);
    let entrypoint = format!("/scenes/{}/scene.yml", args.scene);

    write_file(
        &mod_root.join("mod.yaml"),
        &render_mod_yaml(&args.name, &entrypoint),
        args.force,
    )?;
    write_file(
        &mod_root.join(format!("scenes/{}/scene.yml", args.scene)),
        &render_scene_yaml(&scene_id, &scene_title),
        args.force,
    )?;
    write_file(
        &mod_root.join(format!("scenes/{}/layers/main.yml", args.scene)),
        &render_layer_yaml("main", &scene_title),
        args.force,
    )?;

    Ok(())
}

pub fn create_scene_scaffold(
    repo_root: &Path,
    mod_name: &str,
    scene_dir: &str,
    scene_id: Option<&str>,
    force: bool,
) -> Result<()> {
    let mod_root = repo_root.join("mods").join(mod_name);
    ensure_mod_exists(&mod_root)?;

    let final_scene_id = scene_id
        .map(str::to_string)
        .unwrap_or_else(|| default_scene_id(mod_name, scene_dir));
    let scene_title = human_title(scene_dir);

    write_file(
        &mod_root.join(format!("scenes/{scene_dir}/scene.yml")),
        &render_scene_yaml(&final_scene_id, &scene_title),
        force,
    )?;
    write_file(
        &mod_root.join(format!("scenes/{scene_dir}/layers/main.yml")),
        &render_layer_yaml("main", &scene_title),
        force,
    )?;
    Ok(())
}

pub fn create_layer_scaffold(repo_root: &Path, args: &NewLayerArgs) -> Result<()> {
    let mod_root = repo_root.join("mods").join(&args.r#mod);
    ensure_mod_exists(&mod_root)?;
    let scene_dir = mod_root.join("scenes").join(&args.scene);
    if !scene_dir.exists() {
        bail!(
            "scene package not found: {} (create it first with `devtool create scene`)",
            scene_dir.display()
        );
    }

    let title = human_title(&args.name);
    write_file(
        &scene_dir.join(format!("layers/{}.yml", args.name)),
        &render_layer_yaml(&args.name, &title),
        args.force,
    )?;
    Ok(())
}

#[derive(Debug)]
pub struct CreatedSprite {
    pub layer_path: PathBuf,
    pub asset_path: PathBuf,
    pub sprite_id: String,
    pub asset_ref: String,
}

pub fn create_sprite_scaffold(repo_root: &Path, args: &NewSpriteArgs) -> Result<CreatedSprite> {
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
        bail!(
            "layer file not found: {} (create it first with `devtool create layer`)",
            layer_path.display()
        );
    }

    let source_path = PathBuf::from(&args.source);
    let source_name = args
        .asset_name
        .clone()
        .or_else(|| {
            source_path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .ok_or_else(|| {
            anyhow::anyhow!("source path has no file name: {}", source_path.display())
        })?;
    let asset_path = mod_root.join("assets/images").join(&source_name);
    let asset_ref = format!("/assets/images/{source_name}");
    copy_asset_if_needed(&source_path, &asset_path, args.force)?;

    let sprite_id = args.id.clone().unwrap_or_else(|| slugify(&source_name));
    append_or_replace_sprite(&layer_path, &args.layer, &sprite_id, &asset_ref, args)?;

    Ok(CreatedSprite {
        layer_path,
        asset_path,
        sprite_id,
        asset_ref,
    })
}

pub fn create_effect_scaffold(repo_root: &Path, args: &NewEffectArgs) -> Result<()> {
    let mod_root = repo_root.join("mods").join(&args.r#mod);
    ensure_mod_exists(&mod_root)?;
    let scene_dir = mod_root.join("scenes").join(&args.scene);
    if !scene_dir.exists() {
        bail!(
            "scene package not found: {} (create it first with `devtool create scene`)",
            scene_dir.display()
        );
    }

    write_file(
        &scene_dir.join(format!("effects/{}.yml", args.name)),
        &render_effect_yaml(&args.builtin, args.duration),
        args.force,
    )?;
    Ok(())
}

pub fn default_scene_id(mod_name: &str, scene_dir: &str) -> String {
    let mod_norm = mod_name.replace('_', "-");
    let scene_norm = scene_dir
        .trim_matches('/')
        .replace('\\', "/")
        .replace('/', ".")
        .replace('_', "-");
    format!("{mod_norm}.{scene_norm}")
}

pub fn human_title(raw: &str) -> String {
    raw.split(['/', '-', '_', '.'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            let mut out = String::new();
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
            out
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_mod_yaml(name: &str, entrypoint: &str) -> String {
    format!(
        "# yaml-language-server: $schema=./schemas/mod.yaml\nname: {name}\nversion: 0.1.0\nentrypoint: {entrypoint}\nterminal:\n  min_colours: 256\n  use_virtual_buffer: true\n  virtual_size: max-available\n  virtual_policy: fit\n"
    )
}

fn render_scene_yaml(scene_id: &str, title: &str) -> String {
    format!(
        "# yaml-language-server: $schema=../../schemas/scenes.yaml\nid: {scene_id}\ntitle: {title}\nbg: black\nstages:\n  on_enter:\n    steps:\n      - pause: 300ms\n  on_idle:\n    trigger: any-key\n    steps:\n      - pause: 300ms\n  on_leave:\n    steps:\n      - effects:\n          - name: fade-out\n            duration: 220\nnext: null\n"
    )
}

fn render_layer_yaml(layer_name: &str, title: &str) -> String {
    format!(
        "# yaml-language-server: $schema=../../../schemas/layers.yaml\n- name: {layer_name}\n  z_index: 0\n  visible: true\n  sprites:\n    - id: title\n      type: text\n      at: cc\n      content: \"{title}\"\n      fg: white\n"
    )
}

fn render_effect_yaml(builtin: &str, duration: u32) -> String {
    format!(
        "# yaml-language-server: $schema=../../../schemas/effects.yaml\n- name: {builtin}\n  duration: {duration}\n  params:\n    easing: linear\n"
    )
}

fn copy_asset_if_needed(source_path: &Path, asset_path: &Path, force: bool) -> Result<()> {
    if !source_path.exists() {
        bail!("source image not found: {}", source_path.display());
    }

    if let Some(parent) = asset_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let same_file = if asset_path.exists() {
        let src = fs::canonicalize(source_path)
            .with_context(|| format!("failed to resolve {}", source_path.display()))?;
        let dst = fs::canonicalize(asset_path)
            .with_context(|| format!("failed to resolve {}", asset_path.display()))?;
        src == dst
    } else {
        false
    };

    if same_file {
        return Ok(());
    }

    if asset_path.exists() && files_are_equal(source_path, asset_path)? {
        return Ok(());
    }

    if asset_path.exists() && !force {
        bail!(
            "asset already exists: {} (use --force to overwrite)",
            asset_path.display()
        );
    }

    fs::copy(source_path, asset_path).with_context(|| {
        format!(
            "failed to copy source image from {} to {}",
            source_path.display(),
            asset_path.display()
        )
    })?;

    Ok(())
}

fn files_are_equal(source_path: &Path, asset_path: &Path) -> Result<bool> {
    let source = fs::read(source_path)
        .with_context(|| format!("failed to read {}", source_path.display()))?;
    let asset =
        fs::read(asset_path).with_context(|| format!("failed to read {}", asset_path.display()))?;
    Ok(source == asset)
}

fn append_or_replace_sprite(
    layer_path: &Path,
    layer_name: &str,
    sprite_id: &str,
    asset_ref: &str,
    args: &NewSpriteArgs,
) -> Result<()> {
    let (header, mut doc) = load_yaml_document_with_header(layer_path)?;

    let layers = doc.as_sequence_mut().ok_or_else(|| {
        anyhow::anyhow!(
            "expected top-level YAML sequence in {}",
            layer_path.display()
        )
    })?;

    let layer = find_target_layer(layers, layer_name).ok_or_else(|| {
        anyhow::anyhow!(
            "layer entry not found in {} for {}",
            layer_path.display(),
            layer_name
        )
    })?;
    let layer_map = layer
        .as_mapping_mut()
        .ok_or_else(|| anyhow::anyhow!("invalid layer mapping in {}", layer_path.display()))?;

    let sprites_value = layer_map
        .entry(Value::String("sprites".into()))
        .or_insert_with(|| Value::Sequence(Sequence::new()));
    let sprites = sprites_value
        .as_sequence_mut()
        .ok_or_else(|| anyhow::anyhow!("`sprites` is not a list in {}", layer_path.display()))?;

    let new_sprite = build_image_sprite(sprite_id, asset_ref, args);
    if let Some(existing_index) = find_sprite_index(sprites, sprite_id) {
        if !args.force {
            bail!(
                "sprite id already exists in {}: {} (use --force to replace)",
                layer_path.display(),
                sprite_id
            );
        }
        sprites[existing_index] = new_sprite;
    } else {
        sprites.push(new_sprite);
    }

    write_yaml_document_with_header(layer_path, header.as_deref(), &doc)?;

    Ok(())
}

fn split_yaml_header(content: &str) -> (Option<&str>, &str) {
    let mut header_end = 0usize;
    for line in content.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            header_end += line.len() + 1;
            continue;
        }
        break;
    }
    if header_end == 0 {
        return (None, content);
    }
    let header = content[..header_end].trim_end_matches('\n');
    let body = &content[header_end..];
    (Some(header), body)
}

pub(crate) fn find_target_layer<'a>(
    layers: &'a mut Sequence,
    layer_name: &str,
) -> Option<&'a mut Value> {
    if let Some(index) = layers.iter().position(|layer| {
        layer
            .as_mapping()
            .and_then(|map| map.get(Value::String("name".into())))
            .and_then(Value::as_str)
            == Some(layer_name)
    }) {
        return layers.get_mut(index);
    }
    if layers.len() == 1 {
        return layers.get_mut(0);
    }
    None
}

pub(crate) fn find_sprite_index(sprites: &[Value], sprite_id: &str) -> Option<usize> {
    sprites.iter().position(|sprite| {
        sprite
            .as_mapping()
            .and_then(|map| map.get(Value::String("id".into())))
            .and_then(Value::as_str)
            == Some(sprite_id)
    })
}

fn build_image_sprite(sprite_id: &str, asset_ref: &str, args: &NewSpriteArgs) -> Value {
    let mut map = Mapping::new();
    map.insert(
        Value::String("id".into()),
        Value::String(sprite_id.to_string()),
    );
    map.insert(Value::String("type".into()), Value::String("image".into()));
    map.insert(
        Value::String("source".into()),
        Value::String(asset_ref.to_string()),
    );
    map.insert(
        Value::String("at".into()),
        Value::String(args.at.to_string()),
    );
    map.insert(
        Value::String("width".into()),
        Value::Number(args.width.into()),
    );
    if let Some(height) = args.height {
        map.insert(Value::String("height".into()), Value::Number(height.into()));
    }
    Value::Mapping(map)
}

fn slugify(name: &str) -> String {
    let stem = Path::new(name)
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| name.to_string());
    let mut out = String::new();
    let mut last_dash = false;
    for ch in stem.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "sprite".to_string()
    } else {
        trimmed.to_string()
    }
}

pub(crate) fn load_yaml_document_with_header(path: &Path) -> Result<(Option<String>, Value)> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let (header, yaml_body) = split_yaml_header(&content);
    let doc: Value = serde_yaml::from_str(yaml_body)
        .with_context(|| format!("invalid YAML in {}", path.display()))?;
    Ok((header.map(ToString::to_string), doc))
}

pub(crate) fn write_yaml_document_with_header(
    path: &Path,
    header: Option<&str>,
    doc: &Value,
) -> Result<()> {
    let mut yaml = serde_yaml::to_string(doc)
        .with_context(|| format!("failed to serialize {}", path.display()))?;
    if let Some(stripped) = yaml.strip_prefix("---\n") {
        yaml = stripped.to_string();
    }
    let final_content = match header {
        Some(header) if !header.is_empty() => format!("{header}\n{yaml}"),
        _ => yaml,
    };
    fs::write(path, final_content)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}
