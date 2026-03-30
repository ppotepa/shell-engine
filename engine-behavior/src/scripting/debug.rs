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
}
