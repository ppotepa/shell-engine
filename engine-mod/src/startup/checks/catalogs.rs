//! Validates mod catalogs under `/catalogs`.

use engine_behavior::catalog::ModCatalogs;
use engine_error::EngineError;

use super::super::check::StartupCheck;
use super::super::context::StartupContext;
use super::super::report::StartupReport;

/// Startup check for mod catalogs:
/// - `/catalogs/input-profiles.yaml`
/// - `/catalogs/prefabs.yaml`
/// - `/catalogs/weapons.yaml`
/// - `/catalogs/emitters.yaml`
/// - `/catalogs/spawners.yaml`
///
/// Catalogs are optional; if `/catalogs/` doesn't exist, the check passes with an info message.
/// If `/catalogs/` exists but files cannot be parsed, the check fails.
pub struct CatalogsCheck;

impl StartupCheck for CatalogsCheck {
    fn name(&self) -> &'static str {
        "catalogs"
    }

    fn run(&self, ctx: &StartupContext, report: &mut StartupReport) -> Result<(), EngineError> {
        let catalogs_dir = ctx.mod_source().join("catalogs");

        if !catalogs_dir.exists() {
            report.add_info(
                self.name(),
                "catalogs check skipped (no /catalogs directory)",
            );
            return Ok(());
        }

        if !catalogs_dir.is_dir() {
            return Err(EngineError::StartupCheckFailed {
                check: self.name().to_string(),
                details: "catalogs exists but is not a directory".to_string(),
            });
        }

        let catalogs = ModCatalogs::load_from_directory(&catalogs_dir).map_err(|e| {
            EngineError::StartupCheckFailed {
                check: self.name().to_string(),
                details: e,
            }
        })?;

        report.add_info(
            self.name(),
            format!(
                "catalogs loaded successfully ({} profiles, {} prefabs, {} weapons, {} emitters, {} groups, {} waves, {} planet_types, {} bodies)",
                catalogs.input_profiles.len(),
                catalogs.prefabs.len(),
                catalogs.weapons.len(),
                catalogs.emitters.len(),
                catalogs.groups.len(),
                catalogs.waves.len(),
                catalogs.celestial.planet_types.len(),
                catalogs.celestial.bodies.len(),
            ),
        );

        Ok(())
    }
}
