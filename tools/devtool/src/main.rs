mod cli;
mod edit;
mod fs_utils;
mod scaffold;
mod schema;

use anyhow::{Context, Result};
use clap::Parser;

use cli::{Cli, Command, CreateCommand, EditCommand, SchemaCommand};
use edit::edit_sprite;
use fs_utils::{find_repo_root, resolve_mod_roots};
use scaffold::{
    create_effect_scaffold, create_layer_scaffold, create_mod_scaffold, create_scene_scaffold,
    create_sprite_scaffold,
};
use schema::sync_fragment_for_mod;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir().context("failed to read cwd")?;
    let repo_root = find_repo_root(&cwd)?;

    match cli.command {
        Command::Create { kind } => match kind {
            CreateCommand::Mod(args) => {
                create_mod_scaffold(&repo_root, &args)?;
                let mod_root = repo_root.join("mods").join(&args.name);
                sync_fragment_for_mod(&mod_root, false)?;
                println!("created mod scaffold: {}", mod_root.display());
            }
            CreateCommand::Scene(args) => {
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
            CreateCommand::Layer(args) => {
                create_layer_scaffold(&repo_root, &args)?;
                let mod_root = repo_root.join("mods").join(&args.r#mod);
                sync_fragment_for_mod(&mod_root, false)?;
                println!(
                    "created layer scaffold: {}/scenes/{}/layers/{}.yml",
                    mod_root.display(),
                    args.scene,
                    args.name
                );
            }
            CreateCommand::Sprite(args) => {
                let created = create_sprite_scaffold(&repo_root, &args)?;
                let mod_root = repo_root.join("mods").join(&args.r#mod);
                sync_fragment_for_mod(&mod_root, false)?;
                println!(
                    "created sprite {} in {} using {} ({})",
                    created.sprite_id,
                    created.layer_path.display(),
                    created.asset_ref,
                    created.asset_path.display()
                );
            }
            CreateCommand::Effect(args) => {
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
        Command::Edit { kind } => match kind {
            EditCommand::Sprite(args) => {
                let edited = edit_sprite(&repo_root, &args)?;
                let mod_root = repo_root.join("mods").join(&args.r#mod);
                sync_fragment_for_mod(&mod_root, false)?;
                println!(
                    "edited sprite {} in {} ({})",
                    edited.sprite_id,
                    edited.layer_path.display(),
                    edited.updated_fields.join(", ")
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

#[cfg(test)]
mod tests {
    use super::{
        create_effect_scaffold, create_layer_scaffold, create_scene_scaffold,
        create_sprite_scaffold, edit_sprite,
    };
    use crate::cli::{EditSpriteArgs, NewEffectArgs, NewLayerArgs, NewSpriteArgs};
    use crate::fs_utils::write_file;
    use crate::scaffold::default_scene_id;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn scene_id_defaults_are_stable() {
        assert_eq!(
            default_scene_id("shell-engine", "intro-logo"),
            "shell-engine.intro-logo"
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
    fn create_layer_requires_scene_package() {
        let repo = fake_repo_root();
        let mod_root = repo.join("mods/demo");
        fs::create_dir_all(&mod_root).expect("mod dir");
        fs::write(mod_root.join("mod.yaml"), "name: demo\n").expect("mod.yaml");

        let err = create_layer_scaffold(
            &repo,
            &NewLayerArgs {
                r#mod: "demo".to_string(),
                scene: "missing".to_string(),
                name: "overlay".to_string(),
                force: false,
            },
        )
        .expect_err("expected error");
        assert!(err.to_string().contains("scene package not found"));
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
    fn create_sprite_copies_asset_and_updates_layer() {
        let repo = fake_repo_root();
        let mod_root = repo.join("mods/demo");
        fs::create_dir_all(&mod_root).expect("mod dir");
        fs::write(mod_root.join("mod.yaml"), "name: demo\n").expect("mod.yaml");
        create_scene_scaffold(&repo, "demo", "intro", Some("demo.intro"), false).expect("scene");

        let source = repo.join("sample-logo.png");
        fs::write(&source, b"png").expect("source image");

        let created = create_sprite_scaffold(
            &repo,
            &NewSpriteArgs {
                source: source.display().to_string(),
                r#mod: "demo".to_string(),
                scene: "intro".to_string(),
                layer: "main".to_string(),
                id: None,
                asset_name: None,
                at: "cc".to_string(),
                width: 20,
                height: Some(10),
                force: false,
            },
        )
        .expect("sprite");

        assert!(created.asset_path.exists());
        let layer =
            fs::read_to_string(mod_root.join("scenes/intro/layers/main.yml")).expect("layer");
        assert!(layer.contains("type: image"));
        assert!(layer.contains("source: /assets/images/sample-logo.png"));
        assert!(layer.contains("id: sample-logo"));
    }

    #[test]
    fn create_sprite_rejects_duplicate_id_without_force() {
        let repo = fake_repo_root();
        let mod_root = repo.join("mods/demo");
        fs::create_dir_all(&mod_root).expect("mod dir");
        fs::write(mod_root.join("mod.yaml"), "name: demo\n").expect("mod.yaml");
        create_scene_scaffold(&repo, "demo", "intro", Some("demo.intro"), false).expect("scene");

        let source = repo.join("sample-logo.png");
        fs::write(&source, b"png").expect("source image");

        let args = NewSpriteArgs {
            source: source.display().to_string(),
            r#mod: "demo".to_string(),
            scene: "intro".to_string(),
            layer: "main".to_string(),
            id: Some("logo".to_string()),
            asset_name: None,
            at: "cc".to_string(),
            width: 20,
            height: None,
            force: false,
        };

        create_sprite_scaffold(&repo, &args).expect("first sprite");
        let err = create_sprite_scaffold(&repo, &args).expect_err("duplicate should fail");
        assert!(err.to_string().contains("sprite id already exists"));
    }

    #[test]
    fn edit_sprite_updates_common_fields() {
        let repo = fake_repo_root();
        let mod_root = repo.join("mods/demo");
        fs::create_dir_all(&mod_root).expect("mod dir");
        fs::write(mod_root.join("mod.yaml"), "name: demo\n").expect("mod.yaml");
        create_scene_scaffold(&repo, "demo", "intro", Some("demo.intro"), false).expect("scene");

        let source = repo.join("sample-logo.png");
        fs::write(&source, b"png").expect("source image");
        create_sprite_scaffold(
            &repo,
            &NewSpriteArgs {
                source: source.display().to_string(),
                r#mod: "demo".to_string(),
                scene: "intro".to_string(),
                layer: "main".to_string(),
                id: Some("logo".to_string()),
                asset_name: None,
                at: "cc".to_string(),
                width: 20,
                height: Some(10),
                force: false,
            },
        )
        .expect("sprite");

        let edited = edit_sprite(
            &repo,
            &EditSpriteArgs {
                r#mod: "demo".to_string(),
                scene: "intro".to_string(),
                layer: "main".to_string(),
                id: "logo".to_string(),
                at: Some("lt".to_string()),
                x: Some("-2".to_string()),
                y: Some("oscillate(-1,1,1000ms)".to_string()),
                width: Some(30),
                height: None,
                clear_height: true,
            },
        )
        .expect("edit sprite");

        assert_eq!(
            edited.updated_fields,
            vec!["at", "x", "y", "width", "height"]
        );
        let layer =
            fs::read_to_string(mod_root.join("scenes/intro/layers/main.yml")).expect("layer");
        assert!(layer.contains("at: lt"));
        assert!(layer.contains("x: -2"));
        assert!(layer.contains("y: oscillate(-1,1,1000ms)"));
        assert!(layer.contains("width: 30"));
        assert!(!layer.contains("height: 10"));
    }

    #[test]
    fn edit_sprite_requires_a_change_flag() {
        let repo = fake_repo_root();
        let mod_root = repo.join("mods/demo");
        fs::create_dir_all(&mod_root).expect("mod dir");
        fs::write(mod_root.join("mod.yaml"), "name: demo\n").expect("mod.yaml");
        create_scene_scaffold(&repo, "demo", "intro", Some("demo.intro"), false).expect("scene");

        let err = edit_sprite(
            &repo,
            &EditSpriteArgs {
                r#mod: "demo".to_string(),
                scene: "intro".to_string(),
                layer: "main".to_string(),
                id: "logo".to_string(),
                at: None,
                x: None,
                y: None,
                width: None,
                height: None,
                clear_height: false,
            },
        )
        .expect_err("expected missing edit flags error");
        assert!(err.to_string().contains("no sprite changes requested"));
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
        use std::time::{SystemTime, UNIX_EPOCH};

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
