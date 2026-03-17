//! Authored reusable object documents referenced from scene `objects:` lists.

use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize)]
/// Authored reusable object definition loaded before scene materialization.
///
/// Object documents provide exported defaults, optional logic metadata, and
/// scene content that can expand into one or more layers or sprites.
pub struct ObjectDocument {
    pub name: String,
    #[serde(default)]
    pub exports: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub state: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub logic: Option<LogicSpec>,
}

#[derive(Debug, Clone, Deserialize)]
/// Authored logic metadata attached to an object document.
///
/// Native logic is lowered into layer behaviors during scene compilation, while
/// other kinds preserve the authored boundary for future runtimes.
pub struct LogicSpec {
    #[serde(default, rename = "type", alias = "kind")]
    pub kind: LogicKind,
    #[serde(default)]
    pub behavior: Option<String>,
    #[serde(default)]
    pub params: BTreeMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
/// Declares which runtime should interpret an object's authored logic block.
pub enum LogicKind {
    #[default]
    Native,
    Graph,
    Script,
}
