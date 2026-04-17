use engine_core::render_types::{LodHint, LodLevel, LodPolicy, ScreenSpaceMetrics};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static LOD_HISTORY: RefCell<HashMap<String, (u8, f32)>> = RefCell::new(HashMap::new());
}

/// Select a LOD level from engine contracts.
///
/// This is intentionally conservative for now:
/// - `Disabled` => level 0
/// - `Fixed` => authored level
/// - `ScreenSpace` => bounded heuristic based on projected radius
pub fn select_lod_level(hint: Option<&LodHint>, metrics: ScreenSpaceMetrics) -> LodLevel {
    let Some(hint) = hint else {
        return LodLevel(0);
    };
    match hint.policy {
        LodPolicy::Disabled => LodLevel(0),
        LodPolicy::Fixed { level } => level,
        LodPolicy::ScreenSpace {
            min_level,
            max_level,
            hysteresis_px: _,
        } => {
            // Minimal first-step heuristic; full hysteresis/stateful policy is future work.
            let radius = metrics.projected_radius_px.max(0.0);
            let span = max_level.0.saturating_sub(min_level.0);
            if span == 0 {
                return min_level;
            }
            let mut raw_level = if radius >= 260.0 {
                min_level.0
            } else if radius >= 150.0 {
                min_level.0.saturating_add((span / 3).max(1))
            } else if radius >= 80.0 {
                min_level.0.saturating_add(((span * 2) / 3).max(1))
            } else {
                max_level.0
            };
            if hint.bias != 0 {
                let delta = hint.bias.unsigned_abs();
                raw_level = if hint.bias > 0 {
                    raw_level.saturating_add(delta)
                } else {
                    raw_level.saturating_sub(delta)
                };
            }
            LodLevel(raw_level.clamp(min_level.0, max_level.0))
        }
    }
}

/// Select a stable LOD level with basic per-node hysteresis.
pub fn select_lod_level_stable(
    node_id: &str,
    hint: Option<&LodHint>,
    metrics: ScreenSpaceMetrics,
) -> LodLevel {
    let suggested = select_lod_level(hint, metrics);
    let hysteresis_px = lod_hysteresis_px(hint);
    if node_id.is_empty() || hysteresis_px <= 0.0 {
        return suggested;
    }
    LOD_HISTORY.with(|cell| {
        let mut map = cell.borrow_mut();
        if let Some((previous_level, previous_radius)) = map.get(node_id).copied() {
            let drift = (metrics.projected_radius_px - previous_radius).abs();
            if previous_level != suggested.0 && drift < hysteresis_px {
                return LodLevel(previous_level);
            }
        }
        map.insert(
            node_id.to_string(),
            (suggested.0, metrics.projected_radius_px.max(0.0)),
        );
        suggested
    })
}

#[inline]
pub fn lod_hysteresis_px(hint: Option<&LodHint>) -> f32 {
    match hint.map(|value| value.policy) {
        Some(LodPolicy::ScreenSpace { hysteresis_px, .. }) => hysteresis_px.max(0.0),
        _ => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::{lod_hysteresis_px, select_lod_level, select_lod_level_stable};
    use engine_core::render_types::{LodHint, LodLevel, LodPolicy, ScreenSpaceMetrics};

    #[test]
    fn defaults_to_level_zero_without_hint() {
        let level = select_lod_level(
            None,
            ScreenSpaceMetrics {
                projected_radius_px: 100.0,
                viewport_area_px: 640 * 480,
            },
        );
        assert_eq!(level, LodLevel(0));
    }

    #[test]
    fn fixed_policy_wins() {
        let level = select_lod_level(
            Some(&LodHint {
                policy: LodPolicy::Fixed { level: LodLevel(3) },
                bias: 0,
            }),
            ScreenSpaceMetrics {
                projected_radius_px: 999.0,
                viewport_area_px: 1920 * 1080,
            },
        );
        assert_eq!(level, LodLevel(3));
    }

    #[test]
    fn positive_bias_prefers_lower_detail() {
        let level = select_lod_level(
            Some(&LodHint {
                policy: LodPolicy::ScreenSpace {
                    min_level: LodLevel(0),
                    max_level: LodLevel(4),
                    hysteresis_px: 0.0,
                },
                bias: 1,
            }),
            ScreenSpaceMetrics {
                projected_radius_px: 300.0,
                viewport_area_px: 1280 * 720,
            },
        );
        assert_eq!(level, LodLevel(1));
    }

    #[test]
    fn hysteresis_is_derived_from_screenspace_policy() {
        assert_eq!(lod_hysteresis_px(None), 0.0);
        assert_eq!(
            lod_hysteresis_px(Some(&LodHint {
                policy: LodPolicy::ScreenSpace {
                    min_level: LodLevel(0),
                    max_level: LodLevel(3),
                    hysteresis_px: 12.0,
                },
                bias: 0,
            })),
            12.0
        );
    }

    #[test]
    fn stable_selector_keeps_previous_level_inside_hysteresis_window() {
        let hint = LodHint {
            policy: LodPolicy::ScreenSpace {
                min_level: LodLevel(0),
                max_level: LodLevel(4),
                hysteresis_px: 24.0,
            },
            bias: 0,
        };
        let first = select_lod_level_stable(
            "test-node",
            Some(&hint),
            ScreenSpaceMetrics {
                projected_radius_px: 260.0,
                viewport_area_px: 1280 * 720,
            },
        );
        assert_eq!(first, LodLevel(0));
        let second = select_lod_level_stable(
            "test-node",
            Some(&hint),
            ScreenSpaceMetrics {
                projected_radius_px: 245.0,
                viewport_area_px: 1280 * 720,
            },
        );
        assert_eq!(second, LodLevel(0));
    }
}
