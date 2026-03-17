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
        let mut id_to_edges: BTreeMap<String, Vec<String>> = BTreeMap::new();
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
            let mut edges = Vec::new();
            if let Some(next) = sf.scene.next.clone() {
                edges.push(next);
            }
            for option in &sf.scene.menu_options {
                edges.push(option.next.clone());
            }
            id_to_edges.insert(sf.scene.id.clone(), edges);
            path_to_id.insert(normalize_scene_path(&sf.path), sf.scene.id.clone());
        }

        let entry_path = normalize_scene_path(ctx.entrypoint());
        let entry_id = path_to_id.get(&entry_path).cloned().ok_or_else(|| {
            EngineError::StartupCheckFailed {
                check: self.name().to_string(),
                details: format!(
                    "entrypoint `{}` not found among discovered scenes",
                    ctx.entrypoint()
                ),
            }
        })?;
        let mut reachable = BTreeSet::new();
        let mut stack = vec![entry_id.clone()];

        while let Some(current_id) = stack.pop() {
            if !reachable.insert(current_id.clone()) {
                continue;
            }

            let Some(edges) = id_to_edges.get(&current_id) else {
                continue;
            };
            for target in edges {
                let resolved_target = resolve_scene_ref(target, &path_to_id);
                if !id_to_edges.contains_key(&resolved_target) {
                    return Err(EngineError::StartupCheckFailed {
                        check: self.name().to_string(),
                        details: format!(
                            "scene `{}` points to missing target scene `{}`",
                            current_id, target
                        ),
                    });
                }
                stack.push(resolved_target);
            }
        }

        if has_cycle(&id_to_edges, &entry_id) {
            report.add_warning(self.name(), "scene graph cycle detected".to_string());
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

fn resolve_scene_ref(scene_ref: &str, path_to_id: &BTreeMap<String, String>) -> String {
    if scene_ref.starts_with('/') {
        let normalized = normalize_scene_path(scene_ref);
        return path_to_id
            .get(&normalized)
            .cloned()
            .unwrap_or_else(|| scene_ref.to_string());
    }
    scene_ref.to_string()
}

fn has_cycle(graph: &BTreeMap<String, Vec<String>>, entry_id: &str) -> bool {
    fn visit(
        node: &str,
        graph: &BTreeMap<String, Vec<String>>,
        visiting: &mut BTreeSet<String>,
        visited: &mut BTreeSet<String>,
    ) -> bool {
        if visiting.contains(node) {
            return true;
        }
        if visited.contains(node) {
            return false;
        }
        visiting.insert(node.to_string());
        for next in graph.get(node).into_iter().flatten() {
            if visit(next, graph, visiting, visited) {
                return true;
            }
        }
        visiting.remove(node);
        visited.insert(node.to_string());
        false
    }

    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    visit(entry_id, graph, &mut visiting, &mut visited)
}

#[cfg(test)]
mod tests {
    use super::SceneGraphCheck;
    use crate::pipelines::startup::{
        StartupCheck, StartupContext, StartupIssueLevel, StartupReport,
    };
    use serde_yaml::Value;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn accepts_explicit_path_refs_in_scene_graph() {
        let temp = tempdir().expect("temp dir");
        let mod_dir = temp.path().join("mod");
        fs::create_dir_all(mod_dir.join("scenes/intro")).expect("create intro dir");
        fs::create_dir_all(mod_dir.join("scenes/next")).expect("create next dir");
        fs::write(
            mod_dir.join("scenes/intro/scene.yml"),
            r#"
id: intro
title: Intro
next: /scenes/next/scene.yml
layers: []
"#,
        )
        .expect("write intro");
        fs::write(
            mod_dir.join("scenes/next/scene.yml"),
            r#"
id: next-scene
title: Next
next: null
layers: []
"#,
        )
        .expect("write next");

        let manifest: Value = serde_yaml::from_str(
            "name: Test\nversion: 0.1.0\nentrypoint: /scenes/intro/scene.yml\n",
        )
        .expect("manifest");
        let ctx = StartupContext::new(&mod_dir, &manifest, "/scenes/intro/scene.yml");
        let mut report = StartupReport::default();

        SceneGraphCheck
            .run(&ctx, &mut report)
            .expect("scene graph ok");

        assert!(report
            .issues()
            .iter()
            .any(|issue| issue.level == StartupIssueLevel::Info
                && issue.message.contains("scene graph verified")));
    }
}
