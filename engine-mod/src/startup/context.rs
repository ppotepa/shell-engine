//! Startup context — lazily loads and caches all scene files for use by
//! [`StartupCheck`](super::check::StartupCheck) implementations.
//!
//! Engine-specific validation capabilities (font checking, image validation,
//! Rhai script smoke-testing) are injected via callbacks so that this crate
//! does not depend on engine internals.

use std::path::Path;
use std::sync::OnceLock;

use engine_core::scene::Scene;
use engine_error::EngineError;
use serde_yaml::Value;

use crate::StartupOutputSetting;

/// A parsed scene file alongside its path, used during startup validation.
#[derive(Debug, Clone)]
pub struct StartupSceneFile {
    pub path: String,
    pub scene: Scene,
}

/// Callback signatures for engine-specific validation logic.
pub type SceneLoaderFn = dyn Fn(&Path) -> Result<Vec<StartupSceneFile>, EngineError>;
pub type FontAssetCheckerFn = dyn Fn(Option<&Path>, &str) -> bool;
pub type GlyphCoverageCheckerFn = dyn Fn(Option<&Path>, &str, &str) -> Option<Vec<char>>;
pub type ImageAssetCheckerFn = dyn Fn(&Path, &str) -> bool;
pub type RhaiScriptValidatorFn = dyn Fn(&str, Option<&str>, &Scene) -> Result<(), String>;

/// Read-only view of the mod under validation, with lazy-loaded scene cache
/// and injectable validator callbacks.
pub struct StartupContext<'a> {
    mod_source: &'a Path,
    manifest: &'a Value,
    entrypoint: &'a str,
    // Compatibility-era startup backend token used by checks that still branch
    // on legacy output naming.
    selected_output: StartupOutputSetting,
    scene_cache: OnceLock<Vec<StartupSceneFile>>,
    scene_loader: &'a SceneLoaderFn,
    font_asset_checker: Option<&'a FontAssetCheckerFn>,
    glyph_coverage_checker: Option<&'a GlyphCoverageCheckerFn>,
    image_asset_checker: Option<&'a ImageAssetCheckerFn>,
    rhai_script_validator: Option<&'a RhaiScriptValidatorFn>,
}

impl<'a> StartupContext<'a> {
    fn load_scenes_if_needed(&self) -> Result<(), EngineError> {
        if self.scene_cache.get().is_none() {
            let loaded = (self.scene_loader)(self.mod_source)?;
            let _ = self.scene_cache.set(loaded);
        }
        Ok(())
    }

    /// Creates a new context with a scene loader callback.
    pub fn new(
        mod_source: &'a Path,
        manifest: &'a Value,
        entrypoint: &'a str,
        scene_loader: &'a SceneLoaderFn,
    ) -> Self {
        Self {
            mod_source,
            manifest,
            entrypoint,
            selected_output: StartupOutputSetting::compatibility_default(),
            scene_cache: OnceLock::new(),
            scene_loader,
            font_asset_checker: None,
            glyph_coverage_checker: None,
            image_asset_checker: None,
            rhai_script_validator: None,
        }
    }

    /// Records the resolved startup backend token selected by the launcher.
    ///
    /// Transitional note: naming keeps `output` for compatibility with existing
    /// startup checks. Canonical backend selection lives in runtime/CLI wiring.
    pub fn with_selected_output(mut self, selected_output: StartupOutputSetting) -> Self {
        self.selected_output = selected_output.to_compatibility_token();
        self
    }

    /// Registers a callback that checks whether a font's assets exist.
    pub fn with_font_asset_checker(mut self, checker: &'a FontAssetCheckerFn) -> Self {
        self.font_asset_checker = Some(checker);
        self
    }

    /// Registers a callback that returns missing glyphs for a font.
    pub fn with_glyph_coverage_checker(mut self, checker: &'a GlyphCoverageCheckerFn) -> Self {
        self.glyph_coverage_checker = Some(checker);
        self
    }

    /// Registers a callback that checks whether an image asset exists and is loadable.
    pub fn with_image_asset_checker(mut self, checker: &'a ImageAssetCheckerFn) -> Self {
        self.image_asset_checker = Some(checker);
        self
    }

    /// Registers a callback that smoke-validates a Rhai script.
    pub fn with_rhai_script_validator(mut self, validator: &'a RhaiScriptValidatorFn) -> Self {
        self.rhai_script_validator = Some(validator);
        self
    }

    // --- Accessors ---

    /// Returns the path to the mod source directory or archive.
    pub fn mod_source(&self) -> &Path {
        self.mod_source
    }

    /// Returns the parsed `mod.yaml` manifest value.
    pub fn manifest(&self) -> &Value {
        self.manifest
    }

    /// Returns the entrypoint scene path declared in the manifest.
    pub fn entrypoint(&self) -> &str {
        self.entrypoint
    }

    /// Returns the resolved startup backend token selected by the launcher.
    ///
    /// Transitional note: this remains compatibility-oriented and does not mean
    /// backend choice is manifest-driven.
    pub fn selected_output(&self) -> StartupOutputSetting {
        self.selected_output
    }

    /// Returns (and caches) every parsed scene in the mod, loading them on first call.
    pub fn all_scenes(&self) -> Result<&[StartupSceneFile], EngineError> {
        self.load_scenes_if_needed()?;
        Ok(self.scene_cache.get().map(Vec::as_slice).unwrap_or(&[]))
    }

    // --- Validator delegates ---

    /// Returns `true` when font assets exist for `font_name`.
    /// Always returns `false` when no font asset checker is registered.
    pub fn has_font_assets(&self, font_name: &str) -> bool {
        self.font_asset_checker
            .is_some_and(|f| f(Some(self.mod_source), font_name))
    }

    /// Returns the set of glyphs in `text` that are missing from `font_name`.
    /// Returns `None` when the font itself cannot be found, or when no checker is registered.
    pub fn font_missing_glyphs(&self, font_name: &str, text: &str) -> Option<Vec<char>> {
        self.glyph_coverage_checker
            .and_then(|f| f(Some(self.mod_source), font_name, text))
    }

    /// Returns `true` when the image asset at `source` exists and is loadable.
    /// Always returns `false` when no image checker is registered.
    pub fn has_image_asset(&self, source: &str) -> bool {
        self.image_asset_checker
            .is_some_and(|f| f(self.mod_source, source))
    }

    /// Smoke-validates a Rhai script. Returns `Ok(())` on success or an error description.
    /// Passes through `Ok(())` when no validator is registered (skips validation).
    pub fn validate_rhai_script(
        &self,
        script: &str,
        src: Option<&str>,
        scene: &Scene,
    ) -> Result<(), String> {
        self.rhai_script_validator
            .map_or(Ok(()), |f| f(script, src, scene))
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use serde_yaml::Value;

    use crate::{startup::StartupContext, StartupOutputSetting};

    fn empty_scene_loader(
        _mod_source: &Path,
    ) -> Result<Vec<crate::startup::StartupSceneFile>, engine_error::EngineError> {
        Ok(Vec::new())
    }

    #[test]
    fn new_context_defaults_to_backend_neutral_compatibility_token() {
        let manifest = Value::Mapping(Default::default());
        let ctx = StartupContext::new(
            Path::new("."),
            &manifest,
            "/scenes/main.yml",
            &empty_scene_loader,
        );
        assert_eq!(ctx.selected_output(), StartupOutputSetting::Compatibility);
    }

    #[test]
    fn selected_output_stores_compatibility_token() {
        let manifest = Value::Mapping(Default::default());
        let ctx = StartupContext::new(
            Path::new("."),
            &manifest,
            "/scenes/main.yml",
            &empty_scene_loader,
        )
        .with_selected_output(StartupOutputSetting::Compatibility);
        assert_eq!(ctx.selected_output(), StartupOutputSetting::Compatibility);
    }
}
