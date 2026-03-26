use engine_core::scene::Effect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PostFxBuiltin {
    Underlay,
    Distort,
    ScanGlitch,
    Ruby,
    BurnIn,
}

#[derive(Debug, Clone)]
pub(super) enum CompiledPostFx {
    Builtin { kind: PostFxBuiltin, effect: Effect },
    /// Multiple CRT sub-effects coalesced into a single pixel loop.
    CrtComposite { sub_passes: Vec<(PostFxBuiltin, Effect)> },
    Generic(Effect),
}

/// Returns `true` for builtin kinds that can be merged into a CRT composite.
fn is_coalescable(kind: PostFxBuiltin) -> bool {
    matches!(
        kind,
        PostFxBuiltin::Underlay
            | PostFxBuiltin::Distort
            | PostFxBuiltin::ScanGlitch
            | PostFxBuiltin::Ruby
    )
}

fn flush_crt_group(
    group: &mut Vec<(PostFxBuiltin, Effect)>,
    result: &mut Vec<CompiledPostFx>,
) {
    if group.is_empty() {
        return;
    }
    if group.len() == 1 {
        let (kind, effect) = group.remove(0);
        result.push(CompiledPostFx::Builtin { kind, effect });
    } else {
        result.push(CompiledPostFx::CrtComposite {
            sub_passes: std::mem::take(group),
        });
    }
}

pub(super) fn compile_passes(passes: &[Effect]) -> Vec<CompiledPostFx> {
    let mut result = Vec::new();
    let mut crt_group: Vec<(PostFxBuiltin, Effect)> = Vec::new();

    for pass in passes {
        if let Some(kind) = resolve_builtin(pass) {
            if is_coalescable(kind) {
                crt_group.push((kind, pass.clone()));
                continue;
            }
            // Non-coalescable builtin (BurnIn) — flush pending group first.
            flush_crt_group(&mut crt_group, &mut result);
            result.push(CompiledPostFx::Builtin {
                kind,
                effect: pass.clone(),
            });
        } else {
            flush_crt_group(&mut crt_group, &mut result);
            result.push(CompiledPostFx::Generic(pass.clone()));
        }
    }
    flush_crt_group(&mut crt_group, &mut result);
    result
}

fn resolve_builtin(pass: &Effect) -> Option<PostFxBuiltin> {
    let name = pass.name.to_ascii_lowercase();
    match name.as_str() {
        // Explicit registered postfx names (preferred style).
        "crt-underlay" => Some(PostFxBuiltin::Underlay),
        "crt-distort" | "crt-curve" | "tube-distort" => Some(PostFxBuiltin::Distort),
        "crt-scan-glitch" | "scan-glitch" | "scanline-glitch" | "global-scan" => {
            Some(PostFxBuiltin::ScanGlitch)
        }
        "crt-ruby" | "ruby-crt" | "ruby-overlay" => Some(PostFxBuiltin::Ruby),
        "crt-burn-in" | "crt-persistence" | "phosphor-burn" => Some(PostFxBuiltin::BurnIn),

        // Backward-compatible terminal-crt mode switching via params.coverage.
        "terminal-crt" => {
            let mode = pass
                .params
                .coverage
                .as_deref()
                .unwrap_or("underlay")
                .to_ascii_lowercase();
            match mode.as_str() {
                "scan-glitch" | "scanline-glitch" | "global-scan" => {
                    Some(PostFxBuiltin::ScanGlitch)
                }
                "crt-distort" | "crt-curve" | "tube-distort" => Some(PostFxBuiltin::Distort),
                "ruby-crt" | "ruby-overlay" | "crt-ruby" => Some(PostFxBuiltin::Ruby),
                _ => Some(PostFxBuiltin::Underlay),
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{compile_passes, CompiledPostFx, PostFxBuiltin};
    use engine_core::scene::{Effect, EffectParams, EffectTargetKind};

    fn effect(name: &str) -> Effect {
        Effect {
            name: name.to_string(),
            duration: 0,
            looping: false,
            target_kind: EffectTargetKind::Any,
            params: EffectParams::default(),
        }
    }

    #[test]
    fn consecutive_crt_effects_coalesce_into_composite() {
        let input = vec![
            effect("crt-underlay"),
            effect("crt-distort"),
            effect("crt-scan-glitch"),
            effect("crt-ruby"),
        ];
        let compiled = compile_passes(&input);
        assert_eq!(compiled.len(), 1, "4 consecutive CRT effects → 1 composite");
        assert!(matches!(compiled[0], CompiledPostFx::CrtComposite { .. }));
        if let CompiledPostFx::CrtComposite { sub_passes } = &compiled[0] {
            assert_eq!(sub_passes.len(), 4);
            assert_eq!(sub_passes[0].0, PostFxBuiltin::Underlay);
            assert_eq!(sub_passes[1].0, PostFxBuiltin::Distort);
            assert_eq!(sub_passes[2].0, PostFxBuiltin::ScanGlitch);
            assert_eq!(sub_passes[3].0, PostFxBuiltin::Ruby);
        }
    }

    #[test]
    fn burn_in_breaks_coalescence() {
        let input = vec![
            effect("crt-burn-in"),
            effect("crt-underlay"),
            effect("crt-distort"),
            effect("crt-ruby"),
        ];
        let compiled = compile_passes(&input);
        assert_eq!(compiled.len(), 2, "BurnIn + 3 CRT → BurnIn + composite");
        assert!(matches!(
            compiled[0],
            CompiledPostFx::Builtin {
                kind: PostFxBuiltin::BurnIn,
                ..
            }
        ));
        assert!(matches!(compiled[1], CompiledPostFx::CrtComposite { .. }));
    }

    #[test]
    fn single_crt_effect_stays_standalone() {
        let input = vec![effect("crt-distort")];
        let compiled = compile_passes(&input);
        assert_eq!(compiled.len(), 1);
        assert!(matches!(
            compiled[0],
            CompiledPostFx::Builtin {
                kind: PostFxBuiltin::Distort,
                ..
            }
        ));
    }

    #[test]
    fn terminal_crt_coverage_alias_is_supported() {
        let mut e = effect("terminal-crt");
        e.params.coverage = Some("scan-glitch".to_string());
        let compiled = compile_passes(&[e]);
        assert_eq!(compiled.len(), 1);
        assert!(matches!(
            compiled[0],
            CompiledPostFx::Builtin {
                kind: PostFxBuiltin::ScanGlitch,
                ..
            }
        ));
    }

    #[test]
    fn generic_effect_breaks_coalescence() {
        let input = vec![
            effect("crt-underlay"),
            effect("some-custom-effect"),
            effect("crt-distort"),
        ];
        let compiled = compile_passes(&input);
        assert_eq!(compiled.len(), 3, "underlay + generic + distort stay separate");
        assert!(matches!(
            compiled[0],
            CompiledPostFx::Builtin {
                kind: PostFxBuiltin::Underlay,
                ..
            }
        ));
        assert!(matches!(compiled[1], CompiledPostFx::Generic(_)));
        assert!(matches!(
            compiled[2],
            CompiledPostFx::Builtin {
                kind: PostFxBuiltin::Distort,
                ..
            }
        ));
    }
}
