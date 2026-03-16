use std::collections::{BTreeMap, BTreeSet};

use crate::EngineError;

use super::super::check::StartupCheck;
use super::super::context::StartupContext;
use super::super::report::StartupReport;

pub struct SceneGraphCheck;

impl StartupCheck for SceneGraphCheck {
    fn name(&self) -> &'static str {
        "scene-graph"
    }

    fn run(&self, ctx: &StartupContext, report: &mut StartupReport) -> Result<(), EngineError> {
        let all = ctx.all_scenes()?;
        if all.is_empty() {
            return Err(EngineError::StartupCheckFailed {
                check: self.name().to_string(),
                details: "no scene files found under /scenes".to_string(),
            });
        }

        let mut id_to_path = BTreeMap::new();
        let mut id_to_next = BTreeMap::new();
        let mut path_to_id = BTreeMap::new();
        for sf in all {
            if let Some(existing) = id_to_path.insert(sf.scene.id.clone(), sf.path.clone()) {
                return Err(EngineError::StartupCheckFailed {
                    check: self.name().to_string(),
                    details: format!(
                        "duplicate scene id `{}` in `{}` and `{}`",
                        sf.scene.id, existing, sf.path
                    ),
                });
            }
            id_to_next.insert(sf.scene.id.clone(), sf.scene.next.clone());
            path_to_id.insert(normalize_scene_path(&sf.path), sf.scene.id.clone());
        }

        let entry_path = normalize_scene_path(ctx.entrypoint());
        let mut current_id = path_to_id
            .get(&entry_path)
            .cloned()
            .ok_or_else(|| EngineError::StartupCheckFailed {
                check: self.name().to_string(),
                details: format!("entrypoint `{}` not found among discovered scenes", ctx.entrypoint()),
            })?;
        let mut reachable = BTreeSet::new();

        loop {
            if !reachable.insert(current_id.clone()) {
                report.add_warning(
                    self.name(),
                    format!("scene graph cycle detected at `{}`", current_id),
                );
                break;
            }

            let Some(next_id) = id_to_next.get(&current_id).cloned().flatten() else {
                break;
            };
            if !id_to_next.contains_key(&next_id) {
                return Err(EngineError::StartupCheckFailed {
                    check: self.name().to_string(),
                    details: format!(
                        "scene `{}` points to missing next scene `{}`",
                        current_id, next_id
                    ),
                });
            }
            current_id = next_id;
        }

        let mut unreachable = Vec::new();
        for scene_id in id_to_path.keys() {
            if !reachable.contains(scene_id) {
                unreachable.push(scene_id.clone());
            }
        }
        if !unreachable.is_empty() {
            report.add_warning(
                self.name(),
                format!("unreachable scenes: {}", unreachable.join(", ")),
            );
        }

        report.add_info(
            self.name(),
            format!("scene graph verified ({} scenes)", id_to_path.len()),
        );
        Ok(())
    }
}

fn normalize_scene_path(path: &str) -> String {
    if path.starts_with('/') {
        path.replace('\\', "/")
    } else {
        format!("/{}", path.replace('\\', "/"))
    }
}

