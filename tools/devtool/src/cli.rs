use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "devtool")]
#[command(about = "Developer helper for scaffolding mods/scenes/layers/effects and schema sync.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(visible_alias = "new")]
    Create {
        #[command(subcommand)]
        kind: CreateCommand,
    },
    Edit {
        #[command(subcommand)]
        kind: EditCommand,
    },
    Schema {
        #[command(subcommand)]
        action: SchemaCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum CreateCommand {
    /// Create a new mod scaffold under mods/<name>.
    Mod(NewModArgs),
    /// Create a new scene package under mods/<mod>/scenes/<scene>.
    Scene(NewSceneArgs),
    /// Create a new layer partial file under mods/<mod>/scenes/<scene>/layers/<name>.yml.
    Layer(NewLayerArgs),
    /// Copy an image asset into the mod and append an image sprite to a layer.
    Sprite(NewSpriteArgs),
    /// Create a new effects partial file under mods/<mod>/scenes/<scene>/effects/<name>.yml.
    Effect(NewEffectArgs),
}

#[derive(Debug, Subcommand)]
pub enum SchemaCommand {
    /// Regenerate per-mod local schemas.
    Refresh(SchemaTargetArgs),
    /// Check whether per-mod local schemas are up to date.
    Check(SchemaTargetArgs),
}

#[derive(Debug, Subcommand)]
pub enum EditCommand {
    /// Update common sprite fields in a layer YAML file.
    Sprite(EditSpriteArgs),
}

#[derive(Debug, Args)]
pub struct NewModArgs {
    /// Mod directory name under ./mods.
    pub name: String,
    /// Initial scene package directory.
    #[arg(long, default_value = "main")]
    pub scene: String,
    /// Overwrite existing scaffold files.
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct NewSceneArgs {
    /// Mod directory name under ./mods.
    #[arg(long)]
    pub r#mod: String,
    /// Scene package directory under scenes/.
    pub scene: String,
    /// Scene id override (default: <mod>.<scene>).
    #[arg(long)]
    pub id: Option<String>,
    /// Overwrite existing scene scaffold files.
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct NewLayerArgs {
    /// Mod directory name under ./mods.
    #[arg(long)]
    pub r#mod: String,
    /// Scene package directory under scenes/.
    #[arg(long)]
    pub scene: String,
    /// Layer partial file name (without extension).
    pub name: String,
    /// Overwrite existing layer file.
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct NewSpriteArgs {
    /// Source image path to import.
    pub source: String,
    /// Mod directory name under ./mods.
    #[arg(long)]
    pub r#mod: String,
    /// Scene package directory under scenes/.
    #[arg(long, default_value = "main")]
    pub scene: String,
    /// Layer partial file name (without extension).
    #[arg(long, default_value = "main")]
    pub layer: String,
    /// Sprite id override (default: slugified source file stem).
    #[arg(long)]
    pub id: Option<String>,
    /// Asset file name override under assets/images/ (defaults to source basename).
    #[arg(long)]
    pub asset_name: Option<String>,
    /// Sprite anchor/placement value.
    #[arg(long, default_value = "cc")]
    pub at: String,
    /// Initial sprite width in terminal cells.
    #[arg(long, default_value_t = 24)]
    pub width: u32,
    /// Optional initial sprite height in terminal cells.
    #[arg(long)]
    pub height: Option<u32>,
    /// Overwrite existing asset file or sprite with the same id.
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct NewEffectArgs {
    /// Mod directory name under ./mods.
    #[arg(long)]
    pub r#mod: String,
    /// Scene package directory under scenes/.
    #[arg(long)]
    pub scene: String,
    /// Effect partial file name (without extension).
    pub name: String,
    /// Built-in effect name used in the generated starter file.
    #[arg(long, default_value = "fade-in")]
    pub builtin: String,
    /// Duration in ms for the starter effect entry.
    #[arg(long, default_value_t = 400)]
    pub duration: u32,
    /// Overwrite existing effect file.
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct EditSpriteArgs {
    /// Mod directory name under ./mods.
    #[arg(long)]
    pub r#mod: String,
    /// Scene package directory under scenes/.
    #[arg(long, default_value = "main")]
    pub scene: String,
    /// Layer partial file name (without extension).
    #[arg(long, default_value = "main")]
    pub layer: String,
    /// Existing sprite id to update.
    #[arg(long)]
    pub id: String,
    /// Replace the sprite anchor/placement value.
    #[arg(long)]
    pub at: Option<String>,
    /// Replace horizontal offset (integer or expression string).
    #[arg(long, allow_hyphen_values = true)]
    pub x: Option<String>,
    /// Replace vertical offset (integer or expression string).
    #[arg(long, allow_hyphen_values = true)]
    pub y: Option<String>,
    /// Replace width in terminal cells.
    #[arg(long)]
    pub width: Option<u32>,
    /// Replace height in terminal cells.
    #[arg(long, conflicts_with = "clear_height")]
    pub height: Option<u32>,
    /// Remove the explicit height field.
    #[arg(long)]
    pub clear_height: bool,
}

#[derive(Debug, Args)]
pub struct SchemaTargetArgs {
    /// Single mod name or path.
    #[arg(long, conflicts_with = "all_mods")]
    pub r#mod: Option<String>,
    /// All mods under ./mods (default when no target is provided).
    #[arg(long)]
    pub all_mods: bool,
}
