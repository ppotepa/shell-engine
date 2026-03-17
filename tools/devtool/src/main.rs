use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand};
use engine_authoring::schema::{generate_mod_schema_files, render_schema_file};
use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Parser)]
#[command(name = "devtool")]
#[command(about = "Developer helper for scaffolding mods/scenes/effects and schema sync.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    New {
        #[command(subcommand)]
        kind: NewCommand,
    },
    Schema {
        #[command(subcommand)]
        action: SchemaCommand,
    },
}

#[derive(Debug, Subcommand)]
enum NewCommand {
    /// Create a new mod scaffold under mods/<name>.
    Mod(NewModArgs),
    /// Create a new scene package under mods/<mod>/scenes/<scene>.
    Scene(NewSceneArgs),
    /// Create a new effects partial file under mods/<mod>/scenes/<scene>/effects/<name>.yml.
    Effect(NewEffectArgs),
}

#[derive(Debug, Subcommand)]
enum SchemaCommand {
    /// Regenerate per-mod local schemas.
    Refresh(SchemaTargetArgs),
    /// Check whether per-mod local schemas are up to date.
    Check(SchemaTargetArgs),
}

#[derive(Debug, Args)]
struct NewModArgs {
    /// Mod directory name under ./mods.
    name: String,
    /// Initial scene package directory.
    #[arg(long, default_value = "main")]
    scene: String,
    /// Overwrite existing scaffold files.
    #[arg(long)]
    force: bool,
}

#[derive(Debug, Args)]
struct NewSceneArgs {
    /// Mod directory name under ./mods.
    r#mod: String,
    /// Scene package directory under scenes/.
    scene: String,
    /// Scene id override (default: <mod>.<scene>).
    #[arg(long)]
    id: Option<String>,
    /// Overwrite existing scene scaffold files.
    #[arg(long)]
    force: bool,
}

#[derive(Debug, Args)]
struct NewEffectArgs {
    /// Mod directory name under ./mods.
    r#mod: String,
    /// Scene package directory under scenes/.
    scene: String,
    /// Effect partial file name (without extension).
    name: String,
    /// Built-in effect name used in the generated starter file.
    #[arg(long, default_value = "fade-in")]
    builtin: String,
    /// Duration in ms for the starter effect entry.
    #[arg(long, default_value_t = 400)]
    duration: u32,
    /// Overwrite existing effect file.
    #[arg(long)]
    force: bool,
}

#[derive(Debug, Args)]
struct SchemaTargetArgs {
    /// Single mod name or path.
    #[arg(long, conflicts_with = "all_mods")]
    r#mod: Option<String>,
    /// All mods under ./mods (default when no target is provided).
    #[arg(long)]
    all_mods: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir().context("failed to read cwd")?;
    let repo_root = find_repo_root(&cwd)?;

    match cli.command {
        Command::New { kind } => match kind {
            NewCommand::Mod(args) => {
                create_mod_scaffold(&repo_root, &args)?;
                let mod_root = repo_root.join("mods").join(&args.name);
                sync_fragment_for_mod(&mod_root, false)?;
                println!("created mod scaffold: {}", mod_root.display());
            }
            NewCommand::Scene(args) => {
                create_scene_scaffold(
                    &repo_root,
                    &args.r#mod,
                    &args.scene,
                    args.id.as_deref(),
                    args.force,
                )?;
                let mod_root = repo_root.join("mods").join(&args.r#mod);
                sync_fragment_for_mod(&mod_root, false)?;
                println!(
                    "created scene scaffold: {}/scenes/{}",
                    mod_root.display(),
                    args.scene
                );
            }
            NewCommand::Effect(args) => {
                create_effect_scaffold(&repo_root, &args)?;
                let mod_root = repo_root.join("mods").join(&args.r#mod);
                sync_fragment_for_mod(&mod_root, false)?;
                println!(
                    "created effect scaffold: {}/scenes/{}/effects/{}.yml",
                    mod_root.display(),
                    args.scene,
                    args.name
                );
            }
        },
        Command::Schema { action } => match action {
            SchemaCommand::Refresh(args) => {
                for mod_root in resolve_mod_roots(&repo_root, &args)? {
                    sync_fragment_for_mod(&mod_root, false)?;
                }
            }
            SchemaCommand::Check(args) => {
                for mod_root in resolve_mod_roots(&repo_root, &args)? {
                    sync_fragment_for_mod(&mod_root, true)?;
                }
            }
        },
    }

    Ok(())
}

fn create_mod_scaffold(repo_root: &Path, args: &NewModArgs) -> Result<()> {
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
        &render_layers_yaml(&scene_title),
        args.force,
    )?;

    Ok(())
}

fn create_scene_scaffold(
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
        &render_layers_yaml(&scene_title),
        force,
    )?;
    Ok(())
}

fn create_effect_scaffold(repo_root: &Path, args: &NewEffectArgs) -> Result<()> {
    let mod_root = repo_root.join("mods").join(&args.r#mod);
    ensure_mod_exists(&mod_root)?;
    let scene_dir = mod_root.join("scenes").join(&args.scene);
    if !scene_dir.exists() {
        bail!(
            "scene package not found: {} (create it first with `devtool new scene`)",
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

fn sync_fragment_for_mod(mod_root: &Path, check: bool) -> Result<()> {
    for file in generate_mod_schema_files(mod_root)? {
        let out_path = mod_root.join(&file.file_name);
        if !check {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
        }
        sync_schema_file(&out_path, &file.value, check)?;
    }
    Ok(())
}

fn sync_schema_file(path: &Path, value: &Value, check: bool) -> Result<()> {
    let yaml = render_schema_file(value)?;
    if check {
        let existing = fs::read_to_string(path)
            .with_context(|| format!("failed to read {} in --check mode", path.display()))?;
        if existing != yaml {
            bail!(
                "generated schema is out of date: {} (run `devtool schema refresh`)",
                path.display()
            );
        }
        println!("checked {}", path.display());
        return Ok(());
    }
    fs::write(path, yaml).with_context(|| format!("failed to write {}", path.display()))?;
    println!("generated {}", path.display());
    Ok(())
}

fn resolve_mod_roots(repo_root: &Path, args: &SchemaTargetArgs) -> Result<Vec<PathBuf>> {
    if let Some(mod_arg) = &args.r#mod {
        let mod_root = parse_mod_target(repo_root, mod_arg);
        ensure_mod_exists(&mod_root)?;
        return Ok(vec![mod_root]);
    }

    let _all = args.all_mods || args.r#mod.is_none();
    let mods_dir = repo_root.join("mods");
    let mut out = Vec::new();
    let entries = fs::read_dir(&mods_dir)
        .with_context(|| format!("failed to read {}", mods_dir.display()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("mod.yaml").exists() {
            out.push(path);
        }
    }
    out.sort();
    if out.is_empty() {
        bail!("no mods found under {}", mods_dir.display());
    }
    Ok(out)
}

fn parse_mod_target(repo_root: &Path, mod_arg: &str) -> PathBuf {
    let path = PathBuf::from(mod_arg);
    if path.is_absolute() {
        return path;
    }
    if mod_arg.contains('/') || mod_arg.contains('\\') {
        return repo_root.join(path);
    }
    repo_root.join("mods").join(mod_arg)
}

fn find_repo_root(start: &Path) -> Result<PathBuf> {
    for dir in start.ancestors() {
        if dir.join("Cargo.toml").exists() && dir.join("mods").is_dir() {
            return Ok(dir.to_path_buf());
        }
    }
    bail!(
        "could not find repository root from {} (expected Cargo.toml + mods/)",
        start.display()
    )
}

fn ensure_mod_exists(mod_root: &Path) -> Result<()> {
    if mod_root.join("mod.yaml").exists() {
        return Ok(());
    }
    bail!(
        "mod root not found or missing mod.yaml: {}",
        mod_root.display()
    )
}

fn write_file(path: &Path, content: &str, force: bool) -> Result<()> {
    if path.exists() && !force {
        bail!(
            "file already exists: {} (use --force to overwrite)",
            path.display()
        );
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn default_scene_id(mod_name: &str, scene_dir: &str) -> String {
    let mod_norm = mod_name.replace('_', "-");
    let scene_norm = scene_dir
        .trim_matches('/')
        .replace('\\', "/")
        .replace('/', ".")
        .replace('_', "-");
    format!("{mod_norm}.{scene_norm}")
}

fn human_title(raw: &str) -> String {
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
        "# yaml-language-server: $schema=../../schemas/mod.schema.yaml\nname: {name}\nversion: 0.1.0\nentrypoint: {entrypoint}\nterminal:\n  min_colours: 256\n  use_virtual_buffer: true\n  virtual_size: max-available\n  virtual_policy: fit\n"
    )
}

fn render_scene_yaml(scene_id: &str, title: &str) -> String {
    format!(
        "# yaml-language-server: $schema=../../schemas/scenes.yaml\nid: {scene_id}\ntitle: {title}\nbg: black\nstages:\n  on_enter:\n    steps:\n      - pause: 300ms\n  on_idle:\n    trigger: any-key\n    steps:\n      - pause: 300ms\n  on_leave:\n    steps:\n      - effects:\n          - name: fade-out\n            duration: 220\nnext: null\n"
    )
}

fn render_layers_yaml(title: &str) -> String {
    format!(
        "# yaml-language-server: $schema=../../../schemas/layers.yaml\n- name: main\n  z_index: 0\n  visible: true\n  sprites:\n    - id: title\n      type: text\n      at: cc\n      content: \"{title}\"\n      fg: white\n"
    )
}

fn render_effect_yaml(builtin: &str, duration: u32) -> String {
    format!(
        "# yaml-language-server: $schema=../../../schemas/effects.yaml\n- name: {builtin}\n  duration: {duration}\n  params:\n    easing: linear\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn scene_id_defaults_are_stable() {
        assert_eq!(
            default_scene_id("shell-quest", "intro-logo"),
            "shell-quest.intro-logo"
        );
        assert_eq!(
            default_scene_id("my_mod", "foo/bar_scene"),
            "my-mod.foo.bar-scene"
        );
    }

    #[test]
    fn create_scene_scaffold_writes_expected_headers() {
        let repo = fake_repo_root();
        let mod_root = repo.join("mods/demo");
        fs::create_dir_all(&mod_root).expect("mod dir");
        fs::write(mod_root.join("mod.yaml"), "name: demo\n").expect("mod.yaml");

        create_scene_scaffold(&repo, "demo", "intro", Some("demo.intro"), false).expect("scaffold");

        let scene = fs::read_to_string(mod_root.join("scenes/intro/scene.yml")).expect("scene");
        assert!(scene.starts_with("# yaml-language-server: $schema=../../schemas/scenes.yaml"));
        let layers =
            fs::read_to_string(mod_root.join("scenes/intro/layers/main.yml")).expect("layers");
        assert!(layers.starts_with("# yaml-language-server: $schema=../../../schemas/layers.yaml"));
    }

    #[test]
    fn create_effect_requires_scene_package() {
        let repo = fake_repo_root();
        let mod_root = repo.join("mods/demo");
        fs::create_dir_all(&mod_root).expect("mod dir");
        fs::write(mod_root.join("mod.yaml"), "name: demo\n").expect("mod.yaml");

        let err = create_effect_scaffold(
            &repo,
            &NewEffectArgs {
                r#mod: "demo".to_string(),
                scene: "missing".to_string(),
                name: "flash".to_string(),
                builtin: "fade-in".to_string(),
                duration: 200,
                force: false,
            },
        )
        .expect_err("expected error");
        assert!(err.to_string().contains("scene package not found"));
    }

    #[test]
    fn write_file_respects_force_flag() {
        let root = fake_repo_root();
        let file = root.join("foo.txt");
        write_file(&file, "a\n", false).expect("write");
        let err = write_file(&file, "b\n", false).expect_err("should fail");
        assert!(err.to_string().contains("file already exists"));
        write_file(&file, "b\n", true).expect("overwrite");
        let content = fs::read_to_string(&file).expect("read");
        assert_eq!(content, "b\n");
    }

    fn fake_repo_root() -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("devtool-test-{}-{now}", std::process::id()));
        fs::create_dir_all(root.join("mods")).expect("mods");
        fs::write(root.join("Cargo.toml"), "[workspace]\n").expect("cargo");
        root
    }
}
