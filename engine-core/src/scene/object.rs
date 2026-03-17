use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize)]
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
pub struct LogicSpec {
    #[serde(default)]
    pub kind: LogicKind,
    #[serde(default)]
    pub behavior: Option<String>,
    #[serde(default)]
    pub params: BTreeMap<String, serde_yaml::Value>,
}

#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LogicKind {
    #[default]
    Native,
    Graph,
    Script,
}

