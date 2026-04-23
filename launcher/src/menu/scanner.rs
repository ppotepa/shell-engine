use crate::workspace::{self, ModEntry};
use anyhow::Result;
use std::path::Path;

pub struct MenuMod {
    pub name: String,
    pub dir: String,
    pub colors: u16,
    pub world_render_size: String,
    pub policy: String,
    pub scenes: Vec<MenuScene>,
}

pub struct MenuScene {
    pub dir_name: String,
    pub id: Option<String>,
    pub title: Option<String>,
    pub path: String,
}

pub fn scan_menu_entries(workspace_root: &Path) -> Result<Vec<MenuMod>> {
    let mods = workspace::scan_mods(workspace_root)?;

    let menu_mods = mods.into_iter().map(convert_to_menu_mod).collect();

    Ok(menu_mods)
}

fn convert_to_menu_mod(entry: ModEntry) -> MenuMod {
    let colors = entry.manifest.display.min_colours;
    let world_render_size = entry.manifest.display.world_render_size.clone();
    let policy = entry.manifest.display.presentation_policy.clone();

    let scenes = entry
        .scenes
        .into_iter()
        .map(|s| MenuScene {
            dir_name: s.dir_name,
            id: s.id,
            title: s.title,
            path: s.path.to_string_lossy().to_string(),
        })
        .collect();

    MenuMod {
        name: entry.manifest.name,
        dir: entry.dir.to_string_lossy().to_string(),
        colors,
        world_render_size,
        policy,
        scenes,
    }
}
