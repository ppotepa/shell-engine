use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::marker::PhantomData;

/// High-level plugin layers supported by the engine architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginLayer {
    /// Content-only packages: mods, scenes, prefabs, assets, schemas.
    Content,
    /// Script-side extensions: Rhai modules, script helpers, registries.
    Script,
    /// Native Rust engine extensions: systems, importers, handlers, renderer features.
    NativeEngine,
}

/// Stable manifest-style descriptor for one plugin/extension package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginDescriptor {
    pub id: String,
    pub display_name: String,
    pub layer: PluginLayer,
    pub version: Option<String>,
    pub description: Option<String>,
}

impl PluginDescriptor {
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        layer: PluginLayer,
    ) -> Self {
        Self {
            id: id.into(),
            display_name: display_name.into(),
            layer,
            version: None,
            description: None,
        }
    }
}

/// Typed capability value used instead of ad-hoc bool flags.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CapabilityValue {
    Flag(bool),
    Integer(i64),
    Number(f64),
    Text(String),
    TextList(Vec<String>),
}

/// Named capability bag attached to plugins, services, and extension points.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CapabilitySet {
    values: BTreeMap<String, CapabilityValue>,
}

impl CapabilitySet {
    pub fn insert(&mut self, key: impl Into<String>, value: CapabilityValue) {
        self.values.insert(key.into(), value);
    }

    pub fn set_flag(&mut self, key: impl Into<String>, value: bool) {
        self.insert(key, CapabilityValue::Flag(value));
    }

    pub fn set_integer(&mut self, key: impl Into<String>, value: i64) {
        self.insert(key, CapabilityValue::Integer(value));
    }

    pub fn set_number(&mut self, key: impl Into<String>, value: f64) {
        self.insert(key, CapabilityValue::Number(value));
    }

    pub fn set_text(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.insert(key, CapabilityValue::Text(value.into()));
    }

    pub fn set_text_list(&mut self, key: impl Into<String>, value: Vec<String>) {
        self.insert(key, CapabilityValue::TextList(value));
    }

    pub fn get(&self, key: &str) -> Option<&CapabilityValue> {
        self.values.get(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &CapabilityValue)> {
        self.values.iter().map(|(key, value)| (key.as_str(), value))
    }
}

/// High-level engine extension points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtensionPointKind {
    System,
    AssetImporter,
    CommandHandler,
    RendererFeature,
}

/// Ordering hints for registered runtime systems.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SystemOrdering {
    pub after: Vec<String>,
    pub before: Vec<String>,
}

/// Extension registration descriptor used by native engine plugins.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionDescriptor {
    pub id: String,
    pub kind: ExtensionPointKind,
    pub description: Option<String>,
}

/// Registration descriptor for a system extension.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemExtensionDescriptor {
    pub id: String,
    pub phase: String,
    pub ordering: SystemOrdering,
    pub description: Option<String>,
}

/// Registration descriptor for an asset importer extension.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetImporterDescriptor {
    pub id: String,
    pub asset_kind: String,
    pub file_extensions: Vec<String>,
    pub description: Option<String>,
}

/// Registration descriptor for command-handler extensions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandHandlerDescriptor {
    pub id: String,
    pub command_namespace: String,
    pub description: Option<String>,
}

/// Registration descriptor for renderer features.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RendererFeatureDescriptor {
    pub id: String,
    pub packet_kind: String,
    pub description: Option<String>,
}

/// Opaque typed asset handle used across plugins and services.
#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetHandle<T> {
    id: String,
    #[serde(skip)]
    marker: PhantomData<fn() -> T>,
}

impl<T> AssetHandle<T> {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            marker: PhantomData,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

impl<T> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self::new(self.id.clone())
    }
}

/// Marker type for image/texture assets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageAsset {}

/// Marker type for mesh assets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MeshAsset {}

/// Marker type for audio assets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioAsset {}

/// Marker type for script assets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptAsset {}

/// Marker type for material/profile assets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaterialAsset {}

/// Service categories exposed by the runtime without handing out `World`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceKind {
    AssetStore,
    InputRouter,
    Persistence,
    SceneQueries,
    Diagnostics,
    Clipboard,
    Renderer,
    Custom(String),
}

/// Describes one runtime service interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceDescriptor {
    pub id: String,
    pub kind: ServiceKind,
    pub description: Option<String>,
}

/// Trait implemented by runtime service interfaces.
pub trait EngineService: Send + Sync {
    fn descriptor(&self) -> ServiceDescriptor;

    fn capabilities(&self) -> CapabilitySet {
        CapabilitySet::default()
    }
}

/// Data-first contract for native engine plugin registration.
pub trait NativeEnginePlugin: Send + Sync {
    fn descriptor(&self) -> PluginDescriptor;
    fn capabilities(&self) -> CapabilitySet {
        CapabilitySet::default()
    }
    fn declared_extensions(&self) -> Vec<ExtensionDescriptor>;
}

#[cfg(test)]
mod tests {
    use super::{
        AssetHandle, CapabilitySet, CapabilityValue, ExtensionDescriptor, ExtensionPointKind,
        NativeEnginePlugin, PluginDescriptor, PluginLayer,
    };

    #[derive(Debug)]
    struct FakePlugin;

    impl NativeEnginePlugin for FakePlugin {
        fn descriptor(&self) -> PluginDescriptor {
            PluginDescriptor::new("fake.plugin", "Fake Plugin", PluginLayer::NativeEngine)
        }

        fn declared_extensions(&self) -> Vec<ExtensionDescriptor> {
            vec![ExtensionDescriptor {
                id: "fake.plugin.system".to_string(),
                kind: ExtensionPointKind::System,
                description: Some("test system".to_string()),
            }]
        }
    }

    #[test]
    fn capability_set_round_trips_typed_values() {
        let mut capabilities = CapabilitySet::default();
        capabilities.set_flag("renderer.gpu", true);
        capabilities.set_integer("renderer.msaa", 4);
        capabilities.set_text("renderer.backend", "wgpu");

        assert_eq!(
            capabilities.get("renderer.gpu"),
            Some(&CapabilityValue::Flag(true))
        );
        assert_eq!(
            capabilities.get("renderer.msaa"),
            Some(&CapabilityValue::Integer(4))
        );
        assert_eq!(
            capabilities.get("renderer.backend"),
            Some(&CapabilityValue::Text("wgpu".to_string()))
        );
    }

    #[test]
    fn typed_asset_handle_preserves_string_identity() {
        let handle = AssetHandle::<super::MeshAsset>::new("mesh://terrain/chunk-0");
        assert_eq!(handle.id(), "mesh://terrain/chunk-0");
        assert_eq!(handle.clone(), handle);
    }

    #[test]
    fn native_plugin_descriptor_exposes_layer_and_extensions() {
        let plugin = FakePlugin;
        let descriptor = plugin.descriptor();
        assert_eq!(descriptor.layer, PluginLayer::NativeEngine);
        assert_eq!(plugin.declared_extensions().len(), 1);
    }
}
