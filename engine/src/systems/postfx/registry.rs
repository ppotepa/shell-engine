use crate::scene::Effect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PostFxBuiltin {
    Underlay,
    Distort,
    ScanGlitch,
    Ruby,
}

#[derive(Debug, Clone)]
pub(super) enum CompiledPostFx {
    Builtin { kind: PostFxBuiltin, effect: Effect },
    Generic(Effect),
}

pub(super) fn compile_passes(passes: &[Effect]) -> Vec<CompiledPostFx> {
    passes
        .iter()
        .cloned()
        .map(|pass| {
            if let Some(kind) = resolve_builtin(&pass) {
                CompiledPostFx::Builtin { kind, effect: pass }
            } else {
                CompiledPostFx::Generic(pass)
            }
        })
        .collect()
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
    use crate::scene::{Effect, EffectParams, EffectTargetKind};

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
    fn explicit_postfx_names_compile_to_builtins() {
        let input = vec![
            effect("crt-underlay"),
            effect("crt-distort"),
            effect("crt-scan-glitch"),
            effect("crt-ruby"),
        ];
        let compiled = compile_passes(&input);
        assert!(matches!(
            compiled[0],
            CompiledPostFx::Builtin {
                kind: PostFxBuiltin::Underlay,
                ..
            }
        ));
        assert!(matches!(
            compiled[1],
            CompiledPostFx::Builtin {
                kind: PostFxBuiltin::Distort,
                ..
            }
        ));
        assert!(matches!(
            compiled[2],
            CompiledPostFx::Builtin {
                kind: PostFxBuiltin::ScanGlitch,
                ..
            }
        ));
        assert!(matches!(
            compiled[3],
            CompiledPostFx::Builtin {
                kind: PostFxBuiltin::Ruby,
                ..
            }
        ));
    }

    #[test]
    fn terminal_crt_coverage_alias_is_supported() {
        let mut e = effect("terminal-crt");
        e.params.coverage = Some("scan-glitch".to_string());
        let compiled = compile_passes(&[e]);
        assert!(matches!(
            compiled[0],
            CompiledPostFx::Builtin {
                kind: PostFxBuiltin::ScanGlitch,
                ..
            }
        ));
    }
}
