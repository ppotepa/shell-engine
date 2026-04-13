use serde_yaml::Value;

pub const DEFAULT_TARGET_FPS: u16 = 60;
pub const MAX_TARGET_FPS: u16 = 240;

pub fn target_fps_from_manifest(manifest: &Value) -> u16 {
    manifest
        .get("display")
        .and_then(|block| {
            block
                .get("target_fps")
                .or_else(|| block.get("target-fps"))
                .and_then(Value::as_u64)
        })
        .map(|fps| (fps as u16).clamp(1, MAX_TARGET_FPS))
        .unwrap_or(DEFAULT_TARGET_FPS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_fps_defaults_to_sixty() {
        let yaml = serde_yaml::from_str::<serde_yaml::Value>("name: test\n").unwrap();
        assert_eq!(target_fps_from_manifest(&yaml), 60);
    }

    #[test]
    fn target_fps_reads_display_block() {
        let yaml =
            serde_yaml::from_str::<serde_yaml::Value>("display:\n  target_fps: 30\n").unwrap();
        assert_eq!(target_fps_from_manifest(&yaml), 30);
    }

    #[test]
    fn target_fps_supports_kebab_case_alias() {
        let yaml =
            serde_yaml::from_str::<serde_yaml::Value>("display:\n  target-fps: 30\n").unwrap();
        assert_eq!(target_fps_from_manifest(&yaml), 30);
    }
}
