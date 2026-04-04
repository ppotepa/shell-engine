//! Validates mod palettes under `/palettes`.
//!
//! Each `*.yml` file is parsed as a `PaletteData`. Missing directory is fine.

use engine_behavior::palette::PaletteStore;
use engine_error::EngineError;

use super::super::check::StartupCheck;
use super::super::context::StartupContext;
use super::super::report::StartupReport;

pub struct PalettesCheck;

impl StartupCheck for PalettesCheck {
    fn name(&self) -> &'static str {
        "palettes"
    }

    fn run(&self, ctx: &StartupContext, report: &mut StartupReport) -> Result<(), EngineError> {
        let palettes_dir = ctx.mod_source().join("palettes");

        if !palettes_dir.exists() {
            report.add_info(self.name(), "palettes check skipped (no /palettes directory)");
            return Ok(());
        }

        if !palettes_dir.is_dir() {
            return Err(EngineError::StartupCheckFailed {
                check: self.name().to_string(),
                details: "palettes exists but is not a directory".to_string(),
            });
        }

        let store = PaletteStore::load_from_directory(&palettes_dir).map_err(|e| {
            EngineError::StartupCheckFailed {
                check: self.name().to_string(),
                details: e,
            }
        })?;

        report.add_info(
            self.name(),
            &format!(
                "palettes loaded successfully ({} palette(s): {})",
                store.len(),
                store.order.join(", ")
            ),
        );

        Ok(())
    }
}
