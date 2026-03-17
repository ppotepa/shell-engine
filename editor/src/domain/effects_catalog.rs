//! Builtin effects catalogue sourced from engine-core registry.

use engine_core::effects::EffectDispatcher;

#[derive(Debug, Clone, Copy)]
pub struct EffectDoc {
    pub summary: &'static str,
    pub params: &'static [&'static str],
    pub sample: &'static str,
}

pub fn builtin_effect_names() -> Vec<String> {
    EffectDispatcher::builtin_names()
        .iter()
        .map(|s| (*s).to_string())
        .collect()
}

pub fn effect_doc(name: &str) -> EffectDoc {
    match name {
        "crt-on" => EffectDoc {
            summary: "CRT startup sweep with phosphor-like reveal.",
            params: &["easing"],
            sample: "- name: crt-on\n  duration: 900\n  params:\n    easing: easeOutCubic",
        },
        "power-off" => EffectDoc {
            summary: "Old display shutdown collapse to center line.",
            params: &["easing"],
            sample: "- name: power-off\n  duration: 700\n  params:\n    easing: easeInCubic",
        },
        "shine" => EffectDoc {
            summary: "Moving highlight beam crossing the whole frame.",
            params: &["angle", "width", "falloff", "intensity", "easing"],
            sample: "- name: shine\n  duration: 800\n  params:\n    angle: 18\n    width: 6\n    intensity: 1.0",
        },
        "screen-shake" => EffectDoc {
            summary: "Camera-like shake offsetting rendered output.",
            params: &["amplitude_x", "amplitude_y", "frequency", "easing"],
            sample: "- name: screen-shake\n  duration: 260\n  params:\n    amplitude_x: 1.2\n    amplitude_y: 0.4\n    frequency: 8.0",
        },
        "clear-to-colour" => EffectDoc {
            summary: "Clears selected region to a target terminal colour.",
            params: &["colour", "easing"],
            sample: "- name: clear-to-colour\n  duration: 500\n  params:\n    colour: black",
        },
        "lightning-flash" => EffectDoc {
            summary: "Short global lightning flash with glow peak.",
            params: &["intensity", "easing", "orientation"],
            sample: "- name: lightning-flash\n  duration: 260\n  params:\n    intensity: 1.0",
        },
        "lightning-branch" => EffectDoc {
            summary: "Procedural forked bolt between start and end anchors.",
            params: &["strikes", "thickness", "glow", "start_x", "end_x", "easing"],
            sample: "- name: lightning-branch\n  duration: 720\n  params:\n    strikes: 3\n    glow: true\n    start_x: random\n    end_x: random",
        },
        "tesla-orb" => EffectDoc {
            summary: "Orbital electric arcs around a noisy plasma core.",
            params: &["intensity", "speed", "octave_count", "easing"],
            sample: "- name: tesla-orb\n  duration: 1000\n  loop: true\n  params:\n    speed: 1.0\n    intensity: 0.9",
        },
        "fade-in" => EffectDoc {
            summary: "Alpha-like reveal from dark to full brightness.",
            params: &["easing"],
            sample: "- name: fade-in\n  duration: 500\n  params:\n    easing: linear",
        },
        "fade-out" => EffectDoc {
            summary: "Alpha-like fade from full brightness to dark.",
            params: &["easing"],
            sample: "- name: fade-out\n  duration: 500\n  params:\n    easing: linear",
        },
        "fade-to-black" => EffectDoc {
            summary: "Color-preserving fade that converges to black.",
            params: &["easing"],
            sample: "- name: fade-to-black\n  duration: 650\n  params:\n    easing: easeInOutSine",
        },
        _ => EffectDoc {
            summary: "Builtin effect registered in engine dispatcher.",
            params: &["easing", "intensity", "coverage", "orientation"],
            sample: "- name: EFFECT_NAME\n  duration: 600\n  params:\n    easing: linear",
        },
    }
}
