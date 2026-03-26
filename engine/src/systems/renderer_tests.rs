#[cfg(test)]
mod tests {
    use crate::assets::AssetRoot;
    use crate::buffer::{Buffer, VirtualBuffer, TRUE_BLACK};
    use crate::runtime_settings::RuntimeSettings;
    use crate::scene_loader::SceneLoader;
    use crate::scene_runtime::SceneRuntime;
    use crate::systems::compositor::compositor_system;
    use crate::world::World;
    use engine_animation::{Animator, SceneStage};
    use engine_render_terminal::renderer::present_virtual_to_output;
    use std::path::PathBuf;

    #[test]
    fn shell_quest_intro_logo_survives_virtual_presentation() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("engine crate should live under repo root")
            .to_path_buf();
        let mod_root = repo_root.join("mods/shell-quest");
        let loader = SceneLoader::new(mod_root.clone()).expect("scene loader");
        let scene = loader
            .load_by_ref("00.intro.logo")
            .expect("load shell-quest intro logo");

        let mut settings = RuntimeSettings::default();
        settings.use_virtual_buffer = true;
        settings.virtual_width = 120;
        settings.virtual_height = 40;

        let mut world = World::new();
        world.register(Buffer::new(120, 40));
        world.register(VirtualBuffer::new(120, 40));
        world.register(settings);
        world.register(AssetRoot::new(mod_root));
        world.register_scoped(SceneRuntime::new(scene));
        world.register_scoped(Animator {
            stage: SceneStage::OnEnter,
            step_idx: 0,
            elapsed_ms: 300,
            stage_elapsed_ms: 300,
            scene_elapsed_ms: 300,
            next_scene_override: None,
            menu_selected_index: 0,
        });

        compositor_system(&mut world);
        present_virtual_to_output(&mut world);

        let buffer = world.get::<Buffer>().expect("output buffer");
        let has_visible_glyph = (0..buffer.height).any(|y| {
            (0..buffer.width).any(|x| {
                let cell = buffer.get(x, y).expect("cell in bounds");
                cell.symbol != ' ' && (cell.fg != TRUE_BLACK || cell.bg != TRUE_BLACK)
            })
        });
        assert!(
            has_visible_glyph,
            "virtual presentation should preserve intro logo glyphs"
        );
    }
}
