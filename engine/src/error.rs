use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("mod source path does not exist: {0}")]
    SourceNotFound(PathBuf),
    #[error("unsupported mod source, expected directory or .zip file: {0}")]
    UnsupportedSource(PathBuf),
    #[error("failed to read mod manifest from {path}: {source}")]
    ManifestRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("zip archive error for {path}: {source}")]
    ZipArchive {
        path: PathBuf,
        #[source]
        source: zip::result::ZipError,
    },
    #[error("missing required mod entrypoint file mod.yaml in source: {0}")]
    MissingModEntrypoint(PathBuf),
    #[error("missing required field `{field}` in mod.yaml for source: {path}")]
    MissingManifestField { path: PathBuf, field: String },
    #[error("invalid field `{field}` in mod.yaml for source {path}, expected {expected}")]
    InvalidManifestFieldType {
        path: PathBuf,
        field: String,
        expected: String,
    },
    #[error("entrypoint scene `{entrypoint}` not found in mod source: {mod_source}")]
    MissingSceneEntrypoint {
        mod_source: PathBuf,
        entrypoint: String,
    },
    #[error("invalid mod.yaml content in source {path}: {source}")]
    InvalidModYaml {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },
    #[error("terminal does not meet mod requirements: {0}")]
    TerminalRequirementsNotMet(String),
    #[error("startup check `{check}` failed: {details}")]
    StartupCheckFailed { check: String, details: String },
    #[error("render error: {0}")]
    Render(#[from] std::io::Error),
}
