//! Root runtime-domain scripting contracts.

pub mod api;

pub use api::{
    register_runtime_core_api, ObjectRegistryCoreApi, RuntimeCoreApi, RuntimeSceneCoreApi,
    RuntimeServicesCoreApi, RuntimeStoresCoreApi, RuntimeWorldCoreApi, ScriptRuntimeApi,
};
