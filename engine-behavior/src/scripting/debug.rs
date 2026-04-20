//! Debug domain API: ScriptDebugApi for debug logging.

use std::sync::{Arc, Mutex};

use rhai::Engine as RhaiEngine;

use crate::{BehaviorCommand, DebugLogSeverity};

#[derive(Clone)]
pub(crate) struct ScriptDebugApi {
    scene_id: String,
    source: Option<String>,
    queue: Arc<Mutex<Vec<BehaviorCommand>>>,
}

impl ScriptDebugApi {
    pub(crate) fn new(
        scene_id: String,
        source: Option<String>,
        queue: Arc<Mutex<Vec<BehaviorCommand>>>,
    ) -> Self {
        Self {
            scene_id,
            source,
            queue,
        }
    }

    fn info(&mut self, message: &str) {
        self.push(DebugLogSeverity::Info, message);
    }

    fn warn(&mut self, message: &str) {
        self.push(DebugLogSeverity::Warn, message);
    }

    fn error(&mut self, message: &str) {
        self.push(DebugLogSeverity::Error, message);
    }

    fn layout_info(&mut self, target: &str, message: &str) {
        self.push_layout(DebugLogSeverity::Info, target, message);
    }

    fn layout_warn(&mut self, target: &str, message: &str) {
        self.push_layout(DebugLogSeverity::Warn, target, message);
    }

    fn layout_error(&mut self, target: &str, message: &str) {
        self.push_layout(DebugLogSeverity::Error, target, message);
    }

    fn push(&mut self, severity: DebugLogSeverity, message: &str) {
        let trimmed = message.trim();
        if trimmed.is_empty() {
            return;
        }
        let Ok(mut queue) = self.queue.lock() else {
            return;
        };
        queue.push(BehaviorCommand::DebugLog {
            scene_id: self.scene_id.clone(),
            source: self.source.clone(),
            severity,
            message: trimmed.to_string(),
        });
    }

    fn push_layout(&mut self, severity: DebugLogSeverity, target: &str, message: &str) {
        let target = target.trim();
        let message = message.trim();
        if target.is_empty() || message.is_empty() {
            return;
        }
        self.push(severity, &format!("[layout:{target}] {message}"));
    }
}

pub(crate) fn register_with_rhai(engine: &mut RhaiEngine) {
    engine.register_type_with_name::<ScriptDebugApi>("DebugApi");

    engine.register_fn("info", |debug: &mut ScriptDebugApi, message: &str| {
        debug.info(message);
    });
    engine.register_fn("warn", |debug: &mut ScriptDebugApi, message: &str| {
        debug.warn(message);
    });
    engine.register_fn("error", |debug: &mut ScriptDebugApi, message: &str| {
        debug.error(message);
    });
    engine.register_fn(
        "layout_info",
        |debug: &mut ScriptDebugApi, target: &str, message: &str| {
            debug.layout_info(target, message);
        },
    );
    engine.register_fn(
        "layout_warn",
        |debug: &mut ScriptDebugApi, target: &str, message: &str| {
            debug.layout_warn(target, message);
        },
    );
    engine.register_fn(
        "layout_error",
        |debug: &mut ScriptDebugApi, target: &str, message: &str| {
            debug.layout_error(target, message);
        },
    );
}

#[cfg(test)]
mod tests {
    use super::ScriptDebugApi;
    use crate::{BehaviorCommand, DebugLogSeverity};
    use std::sync::{Arc, Mutex};

    #[test]
    fn pushes_standard_debug_log_entries() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut api = ScriptDebugApi::new(
            "scene-a".to_string(),
            Some("./scene.rhai".to_string()),
            Arc::clone(&queue),
        );

        api.info("hello");
        api.warn("careful");
        api.error("boom");

        let queue = queue.lock().expect("queue lock");
        assert_eq!(queue.len(), 3);
        assert!(matches!(
            &queue[0],
            BehaviorCommand::DebugLog {
                scene_id,
                source,
                severity: DebugLogSeverity::Info,
                message,
            } if scene_id == "scene-a"
                && source.as_deref() == Some("./scene.rhai")
                && message == "hello"
        ));
        assert!(matches!(
            &queue[1],
            BehaviorCommand::DebugLog {
                severity: DebugLogSeverity::Warn,
                message,
                ..
            } if message == "careful"
        ));
        assert!(matches!(
            &queue[2],
            BehaviorCommand::DebugLog {
                severity: DebugLogSeverity::Error,
                message,
                ..
            } if message == "boom"
        ));
    }

    #[test]
    fn pushes_prefixed_layout_debug_entries() {
        let queue = Arc::new(Mutex::new(Vec::<BehaviorCommand>::new()));
        let mut api = ScriptDebugApi::new("scene-a".to_string(), None, Arc::clone(&queue));

        api.layout_warn("hud-score", "overflow");
        api.layout_error("hud-score", "missing font");

        let queue = queue.lock().expect("queue lock");
        assert_eq!(queue.len(), 2);
        assert!(matches!(
            &queue[0],
            BehaviorCommand::DebugLog {
                severity: DebugLogSeverity::Warn,
                message,
                ..
            } if message == "[layout:hud-score] overflow"
        ));
        assert!(matches!(
            &queue[1],
            BehaviorCommand::DebugLog {
                severity: DebugLogSeverity::Error,
                message,
                ..
            } if message == "[layout:hud-score] missing font"
        ));
    }
}
