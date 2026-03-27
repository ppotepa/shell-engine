//! Checks that the current terminal satisfies any minimum capability requirements declared in `mod.yaml`.
//!
//! # Unit model
//!
//! - `terminal.min_width` / `terminal.min_height` are in **terminal cell** units
//!   (columns × rows), matching what `crossterm::terminal::size()` returns.
//! - `terminal.render_size` is the authored virtual canvas in **virtual pixels**.
//!   With halfblock rendering one terminal row maps to two virtual pixel rows.
//! - When `presentation_policy` is `stretch` or `fit`, terminal size requirements
//!   are skipped because the render canvas is projected onto whatever output size
//!   the terminal provides. Only `min_colours` is enforced unconditionally.
//! - When `presentation_policy` is `strict`, the render canvas must fit 1:1 in the
//!   terminal. The check accounts for halfblock (height requirement = render_height / 2).

use crate::terminal_caps::{TerminalCaps, TerminalRequirements, TerminalViolation};
use crate::StartupOutputSetting;
use engine_error::EngineError;
use engine_runtime::{PresentationPolicy, RuntimeSettings};

use super::super::check::StartupCheck;
use super::super::context::StartupContext;
use super::super::report::StartupReport;

/// Startup check that fails when the current terminal does not meet the mod's declared requirements.
pub struct TerminalRequirementsCheck;

impl StartupCheck for TerminalRequirementsCheck {
    fn name(&self) -> &'static str {
        "terminal-requirements"
    }

    fn run(&self, ctx: &StartupContext, report: &mut StartupReport) -> Result<(), EngineError> {
        if ctx.selected_output() == StartupOutputSetting::Sdl2 {
            report.add_info(
                self.name(),
                "skipped terminal capability validation for selected sdl2 backend",
            );
            return Ok(());
        }

        let Some(req) = TerminalRequirements::from_manifest(ctx.manifest()) else {
            report.add_info(self.name(), "no terminal requirements declared");
            return Ok(());
        };

        let runtime = RuntimeSettings::from_manifest(ctx.manifest());
        let caps = TerminalCaps::detect()?;

        // min_colours is always enforced regardless of presentation policy.
        let mut violations = Vec::new();
        if let Some(min) = req.min_colours {
            if caps.colours < min {
                violations.push(TerminalViolation {
                    requirement: "min_colours".into(),
                    required: format!("{}", min),
                    detected: format!("{}", caps.colours),
                });
            }
        }

        // For stretch/fit policies, terminal size is not a hard gate — the render
        // canvas is projected onto whatever terminal cells are available.
        if runtime.presentation_policy == PresentationPolicy::Strict {
            append_cell_size_violations(&req, &caps, &mut violations);
            append_virtual_buffer_violations(&runtime, &caps, &mut violations);
        } else {
            report.add_info(
                self.name(),
                &format!(
                    "terminal size requirements skipped ({:?} presentation scales to output)",
                    runtime.presentation_policy,
                ),
            );
        }

        if violations.is_empty() {
            report.add_info(self.name(), "terminal requirements satisfied");
            return Ok(());
        }

        let details = violations
            .iter()
            .map(|v| {
                format!(
                    "{}: requires {}, detected {}",
                    v.requirement, v.required, v.detected
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        Err(EngineError::TerminalRequirementsNotMet(details))
    }
}

/// Enforce explicit `min_width`/`min_height` cell requirements (strict only).
fn append_cell_size_violations(
    req: &TerminalRequirements,
    caps: &TerminalCaps,
    violations: &mut Vec<TerminalViolation>,
) {
    if let Some(min) = req.min_width {
        if caps.width < min {
            violations.push(TerminalViolation {
                requirement: "min_width".into(),
                required: format!("{}", min),
                detected: format!("{}", caps.width),
            });
        }
    }
    if let Some(min) = req.min_height {
        if caps.height < min {
            violations.push(TerminalViolation {
                requirement: "min_height".into(),
                required: format!("{}", min),
                detected: format!("{}", caps.height),
            });
        }
    }
}

/// For strict fixed render-size mode, ensure the terminal can display the render
/// canvas at 1:1 scale. Height is divided by 2 to account for halfblock rendering.
fn append_virtual_buffer_violations(
    runtime: &RuntimeSettings,
    caps: &TerminalCaps,
    violations: &mut Vec<TerminalViolation>,
) {
    if runtime.render_size_matches_output() {
        return;
    }
    let Some((render_w, render_h)) = runtime.fixed_render_size() else {
        return;
    };
    // Halfblock: 1 terminal row = 2 virtual pixel rows.
    let required_cols = render_w;
    let required_rows = (render_h + 1) / 2; // ceil division
    if caps.width < required_cols || caps.height < required_rows {
        violations.push(TerminalViolation {
            requirement: "render_size(strict+halfblock)".to_string(),
            required: format!("{}x{} cells (render {}x{} px)", required_cols, required_rows, render_w, render_h),
            detected: format!("{}x{}", caps.width, caps.height),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::TerminalRequirementsCheck;
    use crate::startup::{StartupCheck, StartupContext, StartupReport};
    use crate::StartupOutputSetting;
    use std::path::Path;

    fn scene_loader(_mod_source: &std::path::Path) -> Result<Vec<crate::startup::StartupSceneFile>, engine_error::EngineError> {
        Ok(Vec::new())
    }

    #[test]
    fn skips_terminal_validation_for_sdl_output() {
        let manifest = serde_yaml::from_str::<serde_yaml::Value>(
            "name: Test\nversion: 0.1.0\nentrypoint: /scenes/main.yml\nterminal:\n  min_width: 320\n  min_height: 240\n",
        )
        .expect("manifest");
        let ctx = StartupContext::new(Path::new("."), &manifest, "/scenes/main.yml", &scene_loader)
            .with_selected_output(StartupOutputSetting::Sdl2);
        let mut report = StartupReport::default();

        TerminalRequirementsCheck
            .run(&ctx, &mut report)
            .expect("sdl output should skip terminal validation");

        assert!(report
            .issues()
            .iter()
            .any(|issue| issue.message.contains("skipped terminal capability validation")));
    }

    #[test]
    fn skips_terminal_size_checks_for_stretch_policy() {
        // Even with enormous min_width/min_height, stretch policy skips size enforcement.
        let manifest = serde_yaml::from_str::<serde_yaml::Value>(
            "name: Test\nversion: 0.1.0\nentrypoint: /scenes/main.yml\nterminal:\n  min_width: 9999\n  min_height: 9999\n  presentation_policy: stretch\n",
        )
        .expect("manifest");
        let ctx = StartupContext::new(Path::new("."), &manifest, "/scenes/main.yml", &scene_loader)
            .with_selected_output(StartupOutputSetting::Terminal);
        let mut report = StartupReport::default();

        // This should NOT fail even though no real terminal has 9999 cells,
        // because stretch policy skips size checks.
        let result = TerminalRequirementsCheck.run(&ctx, &mut report);
        assert!(result.is_ok(), "stretch policy should skip terminal size checks: {:?}", result);
    }

    #[test]
    fn skips_terminal_size_checks_for_fit_policy() {
        let manifest = serde_yaml::from_str::<serde_yaml::Value>(
            "name: Test\nversion: 0.1.0\nentrypoint: /scenes/main.yml\nterminal:\n  min_width: 9999\n  min_height: 9999\n  presentation_policy: fit\n",
        )
        .expect("manifest");
        let ctx = StartupContext::new(Path::new("."), &manifest, "/scenes/main.yml", &scene_loader)
            .with_selected_output(StartupOutputSetting::Terminal);
        let mut report = StartupReport::default();

        let result = TerminalRequirementsCheck.run(&ctx, &mut report);
        assert!(result.is_ok(), "fit policy should skip terminal size checks: {:?}", result);
    }
}
