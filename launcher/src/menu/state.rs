use super::scanner::MenuMod;
use crate::config::LaunchFlags;
use std::collections::HashSet;

pub struct MenuState {
    pub mods: Vec<MenuMod>,
    pub cursor: usize,
    pub expanded: HashSet<usize>,
    pub search: String,
    pub search_mode: bool,
    pub filtered_indices: Vec<(usize, Option<usize>)>,
    pub flags: LaunchFlags,
    /// Scroll offset: first visible row index in filtered_indices
    pub scroll: usize,
}

pub enum MenuAction {
    None,
    Redraw,
    Launch,
    Quit,
    FlagsChanged,
}

pub struct Selection {
    pub mod_name: String,
    pub mod_dir: String,
    pub scene_path: Option<String>,
}

impl MenuState {
    pub fn new(mods: Vec<MenuMod>, flags: LaunchFlags) -> Self {
        let mut state = Self {
            mods,
            cursor: 0,
            expanded: HashSet::new(),
            search: String::new(),
            search_mode: false,
            filtered_indices: Vec::new(),
            flags,
            scroll: 0,
        };
        state.rebuild_filter();
        state
    }

    pub fn rebuild_filter(&mut self) {
        self.filtered_indices.clear();

        if self.search.is_empty() {
            for (mod_idx, m) in self.mods.iter().enumerate() {
                self.filtered_indices.push((mod_idx, None));
                if self.expanded.contains(&mod_idx) {
                    for (scene_idx, _) in m.scenes.iter().enumerate() {
                        self.filtered_indices.push((mod_idx, Some(scene_idx)));
                    }
                }
            }
        } else {
            let query = self.search.to_lowercase();
            for (mod_idx, m) in self.mods.iter().enumerate() {
                let has_match = m.name.to_lowercase().contains(&query)
                    || m.scenes.iter().any(|s| {
                        s.dir_name.to_lowercase().contains(&query)
                            || s.id
                                .as_deref()
                                .unwrap_or("")
                                .to_lowercase()
                                .contains(&query)
                            || s.title
                                .as_deref()
                                .unwrap_or("")
                                .to_lowercase()
                                .contains(&query)
                    });

                if has_match {
                    self.filtered_indices.push((mod_idx, None));
                    self.expanded.insert(mod_idx);

                    for (scene_idx, s) in m.scenes.iter().enumerate() {
                        let scene_matches = s.dir_name.to_lowercase().contains(&query)
                            || s.id
                                .as_deref()
                                .unwrap_or("")
                                .to_lowercase()
                                .contains(&query)
                            || s.title
                                .as_deref()
                                .unwrap_or("")
                                .to_lowercase()
                                .contains(&query);
                        if scene_matches {
                            self.filtered_indices.push((mod_idx, Some(scene_idx)));
                        }
                    }
                }
            }
        }

        self.clamp_cursor();
    }

    fn clamp_cursor(&mut self) {
        let len = self.filtered_indices.len();
        if len == 0 {
            self.cursor = 0;
            self.scroll = 0;
        } else {
            if self.cursor >= len {
                self.cursor = len - 1;
            }
        }
    }

    /// Ensure scroll window includes the cursor row, given viewport height.
    pub fn ensure_visible(&mut self, viewport_height: usize) {
        if viewport_height == 0 {
            return;
        }
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
        } else if self.cursor >= self.scroll + viewport_height {
            self.scroll = self.cursor + 1 - viewport_height;
        }
    }

    pub fn navigate(&mut self, delta: isize) {
        if self.filtered_indices.is_empty() {
            return;
        }
        let len = self.filtered_indices.len() as isize;
        self.cursor = ((self.cursor as isize + delta).rem_euclid(len)) as usize;
    }

    /// Enter on mod row: expand if collapsed, launch if already expanded.
    /// Enter on scene: launch.
    pub fn enter_action(&mut self) -> MenuAction {
        if self.filtered_indices.is_empty() {
            return MenuAction::None;
        }
        let (mod_idx, scene_idx) = self.filtered_indices[self.cursor];
        if scene_idx.is_some() {
            return MenuAction::Launch;
        }
        if self.expanded.contains(&mod_idx) {
            MenuAction::Launch
        } else {
            self.expanded.insert(mod_idx);
            self.rebuild_filter();
            for (i, &(m, s)) in self.filtered_indices.iter().enumerate() {
                if m == mod_idx && s.is_none() {
                    self.cursor = i;
                    break;
                }
            }
            MenuAction::Redraw
        }
    }

    pub fn collapse_current(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        let (mod_idx, _) = self.filtered_indices[self.cursor];
        if self.expanded.contains(&mod_idx) {
            self.expanded.remove(&mod_idx);
            self.rebuild_filter();
            for (i, &(m, s)) in self.filtered_indices.iter().enumerate() {
                if m == mod_idx && s.is_none() {
                    self.cursor = i;
                    break;
                }
            }
        }
    }

    pub fn toggle_flag(&mut self, n: u8) -> MenuAction {
        match n {
            1 => self.flags.skip_splash = !self.flags.skip_splash,
            2 => self.flags.audio = !self.flags.audio,
            3 => self.flags.check_scenes = !self.flags.check_scenes,
            4 => self.flags.release = !self.flags.release,
            5 => self.flags.dev = !self.flags.dev,
            6 => self.flags.all_opt = !self.flags.all_opt,
            7 => self.flags.render_backend.toggle(),
            _ => return MenuAction::None,
        }
        MenuAction::FlagsChanged
    }

    pub fn get_selection(&self) -> Option<Selection> {
        if self.filtered_indices.is_empty() {
            return None;
        }
        let (mod_idx, scene_idx) = self.filtered_indices[self.cursor];
        let m = &self.mods[mod_idx];

        Some(Selection {
            mod_name: m.name.clone(),
            mod_dir: m.dir.clone(),
            scene_path: scene_idx.map(|si| {
                let scene = &m.scenes[si];
                let rel = scene.path.strip_prefix(&m.dir).unwrap_or(&scene.path);
                let rel = rel.trim_start_matches('\\').trim_start_matches('/');
                format!("/{}", rel.replace('\\', "/"))
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{MenuAction, MenuState};
    use crate::config::LaunchFlags;

    #[test]
    fn toggle_backend_flag_switches_between_software_and_hardware() {
        let mut state = MenuState::new(Vec::new(), LaunchFlags::default());
        assert_eq!(state.flags.render_backend.as_cli_value(), "hardware");

        let action = state.toggle_flag(7);
        assert!(matches!(action, MenuAction::FlagsChanged));
        assert_eq!(state.flags.render_backend.as_cli_value(), "software");

        let action = state.toggle_flag(7);
        assert!(matches!(action, MenuAction::FlagsChanged));
        assert_eq!(state.flags.render_backend.as_cli_value(), "hardware");
    }
}
