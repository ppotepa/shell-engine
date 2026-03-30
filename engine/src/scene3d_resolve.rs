//! Engine-facing compatibility wrapper around shared 3D scene reference resolution.
//!
//! The engine keeps its own resolver trait so existing integrations can implement
//! it locally, while the shared reference-resolution logic itself lives in
//! `engine-3d`.

use crate::scene3d_format::Scene3DDefinition;

/// Engine-local resolver trait kept for compatibility with existing integrations.
pub trait Scene3DAssetResolver {
    fn resolve_and_load_asset(
        &self,
        asset_path: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}

struct ResolverAdapter<'a, R>(&'a R);

impl<R: Scene3DAssetResolver> engine_3d::scene3d_resolve::Scene3DAssetResolver
    for ResolverAdapter<'_, R>
{
    fn resolve_and_load_asset(
        &self,
        asset_path: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.0.resolve_and_load_asset(asset_path)
    }
}

pub fn resolve_scene3d_refs<R: Scene3DAssetResolver>(
    def: &mut Scene3DDefinition,
    src_path: &str,
    resolver: &R,
) {
    engine_3d::scene3d_resolve::resolve_scene3d_refs(def, src_path, &ResolverAdapter(resolver));
}
