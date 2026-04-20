//! Validates the scene graph — unique scene IDs, reachability from the entrypoint, and no dangling transitions.

use std::collections::{BTreeMap, BTreeSet};

use engine_core::scene::Scene;
use engine_error::EngineError;

use super::super::check::StartupCheck;
use super::super::context::StartupContext;
use super::super::report::StartupReport;

/// Startup check that validates scene IDs are unique, all transitions resolve, and the graph is connected.
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
            let current_mod_name = ctx.manifest().get("name").and_then(|value| value.as_str());
            edges.extend(collect_scripted_scene_jumps(&sf.scene, current_mod_name));
            id_to_edges.insert(sf.scene.id.clone(), edges);
            path_to_id.insert(normalize_scene_path(&sf.path), sf.scene.id.clone());
        }
        let resolved_graph = resolve_graph_edges(&id_to_edges, &path_to_id);

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

            let Some(edges) = resolved_graph.get(&current_id) else {
                continue;
            };
            for target in edges {
                if !resolved_graph.contains_key(target) {
                    return Err(EngineError::StartupCheckFailed {
                        check: self.name().to_string(),
                        details: format!(
                            "scene `{}` points to missing target scene `{}`",
                            current_id, target
                        ),
                    });
                }
                stack.push(target.clone());
            }
        }

        if has_cycle(&resolved_graph, &entry_id) {
            report.add_info(
                self.name(),
                "scene graph contains reachable cycles".to_string(),
            );
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

fn resolve_graph_edges(
    graph: &BTreeMap<String, Vec<String>>,
    path_to_id: &BTreeMap<String, String>,
) -> BTreeMap<String, Vec<String>> {
    graph
        .iter()
        .map(|(scene_id, edges)| {
            let resolved = edges
                .iter()
                .map(|target| resolve_scene_ref(target, path_to_id))
                .collect();
            (scene_id.clone(), resolved)
        })
        .collect()
}

fn collect_scripted_scene_jumps(scene: &Scene, current_mod_name: Option<&str>) -> Vec<String> {
    let mut jumps = Vec::new();
    for behavior in &scene.behaviors {
        jumps.extend(collect_behavior_jumps(
            behavior.params.script.as_deref(),
            current_mod_name,
        ));
    }
    for layer in &scene.layers {
        for behavior in &layer.behaviors {
            jumps.extend(collect_behavior_jumps(
                behavior.params.script.as_deref(),
                current_mod_name,
            ));
        }
        for sprite in &layer.sprites {
            sprite.walk_recursive(&mut |node| {
                for behavior in node.behaviors() {
                    jumps.extend(collect_behavior_jumps(
                        behavior.params.script.as_deref(),
                        current_mod_name,
                    ));
                }
            });
        }
    }
    jumps.sort();
    jumps.dedup();
    jumps
}

fn collect_behavior_jumps(script: Option<&str>, current_mod_name: Option<&str>) -> Vec<String> {
    let Some(script) = script else {
        return Vec::new();
    };
    let mut jumps = collect_literal_game_jumps(script);
    jumps.extend(collect_literal_game_jump_mods(script, current_mod_name));
    jumps.sort();
    jumps.dedup();
    jumps
}

fn collect_literal_game_jumps(script: &str) -> Vec<String> {
    let mut out = Vec::new();
    let needle = "game.jump";
    let mut offset = 0usize;
    while let Some(pos) = script[offset..].find(needle) {
        let start = offset + pos + needle.len();
        let bytes = script.as_bytes();
        let mut idx = start;
        while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        if idx >= bytes.len() || bytes[idx] != b'(' {
            offset = start;
            continue;
        }
        idx += 1;
        while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        if idx >= bytes.len() || (bytes[idx] != b'"' && bytes[idx] != b'\'') {
            offset = start;
            continue;
        }
        let quote = bytes[idx];
        idx += 1;
        let value_start = idx;
        while idx < bytes.len() {
            if bytes[idx] == quote && bytes[idx.saturating_sub(1)] != b'\\' {
                let target = &script[value_start..idx];
                if !target.trim().is_empty() {
                    out.push(target.trim().to_string());
                }
                idx += 1;
                break;
            }
            idx += 1;
        }
        offset = idx;
    }
    out
}

fn collect_literal_game_jump_mods(script: &str, current_mod_name: Option<&str>) -> Vec<String> {
    let mut out = Vec::new();
    let needle = "game.jump_mod";
    let mut offset = 0usize;
    while let Some(pos) = script[offset..].find(needle) {
        let start = offset + pos + needle.len();
        let bytes = script.as_bytes();
        let mut idx = start;
        while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        if idx >= bytes.len() || bytes[idx] != b'(' {
            offset = start;
            continue;
        }
        idx += 1;
        let Some((mod_ref, next_idx)) = parse_literal_string_arg(script, idx) else {
            offset = start;
            continue;
        };
        idx = next_idx;
        while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
            idx += 1;
        }
        if idx >= bytes.len() || bytes[idx] != b',' {
            offset = start;
            continue;
        }
        idx += 1;
        let Some((scene_ref, next_idx)) = parse_literal_string_arg(script, idx) else {
            offset = start;
            continue;
        };
        if current_mod_name.is_some_and(|name| mod_ref.trim() == name) || mod_ref.trim().is_empty()
        {
            let scene_ref = scene_ref.trim();
            if !scene_ref.is_empty() {
                out.push(scene_ref.to_string());
            }
        }
        offset = next_idx;
    }
    out
}

fn parse_literal_string_arg(script: &str, start: usize) -> Option<(String, usize)> {
    let bytes = script.as_bytes();
    let mut idx = start;
    while idx < bytes.len() && bytes[idx].is_ascii_whitespace() {
        idx += 1;
    }
    if idx >= bytes.len() || (bytes[idx] != b'"' && bytes[idx] != b'\'') {
        return None;
    }
    let quote = bytes[idx];
    idx += 1;
    let value_start = idx;
    while idx < bytes.len() {
        if bytes[idx] == quote && bytes[idx.saturating_sub(1)] != b'\\' {
            return Some((script[value_start..idx].to_string(), idx + 1));
        }
        idx += 1;
    }
    None
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
    use crate::startup::{StartupCheck, StartupContext, StartupIssueLevel, StartupReport};
    use engine_core::scene::Scene;
    use engine_error::EngineError;
    use serde_yaml::Value;
    use std::fs;
    use tempfile::tempdir;

    fn scene_loader(
        mod_source: &std::path::Path,
    ) -> Result<Vec<crate::startup::StartupSceneFile>, EngineError> {
        let scenes_dir = mod_source.join("scenes");
        let mut scenes = Vec::new();
        if !scenes_dir.is_dir() {
            return Ok(scenes);
        }
        load_scenes_recursive(mod_source, &scenes_dir, &mut scenes)?;
        Ok(scenes)
    }

    fn load_scenes_recursive(
        mod_root: &std::path::Path,
        dir: &std::path::Path,
        scenes: &mut Vec<crate::startup::StartupSceneFile>,
    ) -> Result<(), EngineError> {
        for entry in fs::read_dir(dir).map_err(|e| EngineError::ManifestRead {
            path: dir.to_path_buf(),
            source: e,
        })? {
            let entry = entry.map_err(|e| EngineError::ManifestRead {
                path: dir.to_path_buf(),
                source: e,
            })?;
            let path = entry.path();
            if path.is_dir() {
                load_scenes_recursive(mod_root, &path, scenes)?;
            } else if path.extension().is_some_and(|ext| ext == "yml") {
                let content = fs::read_to_string(&path).map_err(|e| EngineError::ManifestRead {
                    path: path.clone(),
                    source: e,
                })?;
                let scene =
                    serde_yaml::from_str(&content).map_err(|e| EngineError::InvalidModYaml {
                        path: path.clone(),
                        source: e,
                    })?;
                // Make path relative to mod root, prefixed with /
                let rel = path
                    .strip_prefix(mod_root)
                    .unwrap_or(&path)
                    .display()
                    .to_string();
                let rel = format!("/{}", rel.replace('\\', "/"));
                scenes.push(crate::startup::StartupSceneFile { path: rel, scene });
            }
        }
        Ok(())
    }

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
        let ctx = StartupContext::new(
            &mod_dir,
            &manifest,
            "/scenes/intro/scene.yml",
            &scene_loader,
        );
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

    #[test]
    fn accepts_scripted_game_jump_transitions() {
        let scene_loader = |_mod_source: &std::path::Path| -> Result<
            Vec<crate::startup::StartupSceneFile>,
            EngineError,
        > {
            let intro: Scene = serde_yaml::from_str(
                r#"
id: intro
title: Intro
layers: []
behaviors:
  - name: rhai-script
    params:
      src: /scenes/intro/main.rhai
      script: |
        if input.just_pressed("F10") {
            game.jump("flight");
        }
"#,
            )
            .expect("intro scene");
            let flight: Scene = serde_yaml::from_str(
                r#"
id: flight
title: Flight
layers: []
"#,
            )
            .expect("flight scene");
            Ok(vec![
                crate::startup::StartupSceneFile {
                    path: "/scenes/intro/scene.yml".to_string(),
                    scene: intro,
                },
                crate::startup::StartupSceneFile {
                    path: "/scenes/flight/scene.yml".to_string(),
                    scene: flight,
                },
            ])
        };

        let mod_dir = tempdir().expect("temp dir");
        let manifest: Value = serde_yaml::from_str(
            "name: Test\nversion: 0.1.0\nentrypoint: /scenes/intro/scene.yml\n",
        )
        .expect("manifest");
        let ctx = StartupContext::new(
            mod_dir.path(),
            &manifest,
            "/scenes/intro/scene.yml",
            &scene_loader,
        );
        let mut report = StartupReport::default();

        SceneGraphCheck
            .run(&ctx, &mut report)
            .expect("scene graph ok");

        assert!(!report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Warning
                && issue.message.contains("unreachable scenes")
        }));
    }

    #[test]
    fn accepts_scripted_same_mod_jump_mod_transitions() {
        let scene_loader = |_mod_source: &std::path::Path| -> Result<
            Vec<crate::startup::StartupSceneFile>,
            EngineError,
        > {
            let intro: Scene = serde_yaml::from_str(
                r#"
id: intro
title: Intro
layers: []
behaviors:
  - name: rhai-script
    params:
      src: /scenes/intro/main.rhai
      script: |
        if input.just_pressed("F10") {
            game.jump_mod("Test", "flight");
        }
"#,
            )
            .expect("intro scene");
            let flight: Scene = serde_yaml::from_str(
                r#"
id: flight
title: Flight
layers: []
"#,
            )
            .expect("flight scene");
            Ok(vec![
                crate::startup::StartupSceneFile {
                    path: "/scenes/intro/scene.yml".to_string(),
                    scene: intro,
                },
                crate::startup::StartupSceneFile {
                    path: "/scenes/flight/scene.yml".to_string(),
                    scene: flight,
                },
            ])
        };

        let mod_dir = tempdir().expect("temp dir");
        let manifest: Value = serde_yaml::from_str(
            "name: Test\nversion: 0.1.0\nentrypoint: /scenes/intro/scene.yml\n",
        )
        .expect("manifest");
        let ctx = StartupContext::new(
            mod_dir.path(),
            &manifest,
            "/scenes/intro/scene.yml",
            &scene_loader,
        );
        let mut report = StartupReport::default();

        SceneGraphCheck
            .run(&ctx, &mut report)
            .expect("scene graph ok");

        assert!(!report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Warning
                && issue.message.contains("unreachable scenes")
        }));
    }

    #[test]
    fn ignores_scripted_external_jump_mod_targets_for_local_graph() {
        let scene_loader = |_mod_source: &std::path::Path| -> Result<
            Vec<crate::startup::StartupSceneFile>,
            EngineError,
        > {
            let intro: Scene = serde_yaml::from_str(
                r#"
id: intro
title: Intro
layers: []
behaviors:
  - name: rhai-script
    params:
      src: /scenes/intro/main.rhai
      script: |
        if input.just_pressed("F10") {
            game.jump_mod("OtherMod", "external-flight");
        }
"#,
            )
            .expect("intro scene");
            Ok(vec![crate::startup::StartupSceneFile {
                path: "/scenes/intro/scene.yml".to_string(),
                scene: intro,
            }])
        };

        let mod_dir = tempdir().expect("temp dir");
        let manifest: Value = serde_yaml::from_str(
            "name: Test\nversion: 0.1.0\nentrypoint: /scenes/intro/scene.yml\n",
        )
        .expect("manifest");
        let ctx = StartupContext::new(
            mod_dir.path(),
            &manifest,
            "/scenes/intro/scene.yml",
            &scene_loader,
        );
        let mut report = StartupReport::default();

        SceneGraphCheck
            .run(&ctx, &mut report)
            .expect("scene graph ok");

        assert!(!report.issues().iter().any(|issue| {
            issue.level == StartupIssueLevel::Warning
                && issue.message.contains("missing target scene")
        }));
    }
}
