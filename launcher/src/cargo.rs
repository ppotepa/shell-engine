use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub struct CargoCommand {
    profile: Option<String>,
    features: Vec<String>,
    no_default_features: bool,
    package: String,
    app_args: Vec<String>,
}

impl CargoCommand {
    pub fn new(package: impl Into<String>) -> Self {
        Self {
            profile: None,
            features: Vec::new(),
            no_default_features: false,
            package: package.into(),
            app_args: Vec::new(),
        }
    }

    pub fn profile(mut self, profile: impl Into<String>) -> Self {
        self.profile = Some(profile.into());
        self
    }

    pub fn feature(mut self, feature: impl Into<String>) -> Self {
        self.features.push(feature.into());
        self
    }

    pub fn no_default_features(mut self) -> Self {
        self.no_default_features = true;
        self
    }

    pub fn app_arg(mut self, arg: impl Into<String>) -> Self {
        self.app_args.push(arg.into());
        self
    }

    pub fn app_args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.app_args.extend(args.into_iter().map(|s| s.into()));
        self
    }

    pub fn build_args(&self) -> Vec<String> {
        let mut args = vec!["run".to_string(), "-p".to_string(), self.package.clone()];

        if let Some(ref profile) = self.profile {
            if profile == "release" {
                args.push("--release".to_string());
            } else if profile != "dev" {
                args.push("--profile".to_string());
                args.push(profile.clone());
            }
        }

        if self.no_default_features {
            args.push("--no-default-features".to_string());
        }

        if !self.features.is_empty() {
            args.push("--features".to_string());
            args.push(self.features.join(","));
        }

        if !self.app_args.is_empty() {
            args.push("--".to_string());
            args.extend(self.app_args.clone());
        }

        args
    }

    pub fn exec(self, workspace_root: &Path) -> Result<std::process::ExitStatus> {
        let args = self.build_args();

        let mut cmd = Command::new("cargo");
        cmd.args(&args).current_dir(workspace_root);
        cmd.status().context("failed to execute cargo")
    }

    #[allow(dead_code)]
    pub fn spawn(self, workspace_root: &Path) -> Result<std::process::Child> {
        let args = self.build_args();

        let mut cmd = Command::new("cargo");
        cmd.args(&args).current_dir(workspace_root);
        cmd.spawn().context("failed to spawn cargo")
    }
}

#[cfg(test)]
mod tests {
    use super::CargoCommand;

    #[test]
    fn build_args_include_no_default_features_for_hardware_path() {
        let args = CargoCommand::new("app")
            .no_default_features()
            .feature("app/hardware-backend")
            .build_args();

        assert!(args.iter().any(|arg| arg == "--no-default-features"));
        assert!(args
            .windows(2)
            .any(|pair| { pair[0] == "--features" && pair[1].contains("app/hardware-backend") }));
    }

    #[test]
    fn build_args_keep_default_features_for_generic_feature_path() {
        let args = CargoCommand::new("app")
            .feature("app/telemetry")
            .build_args();

        assert!(!args.iter().any(|arg| arg == "--no-default-features"));
        assert!(args
            .windows(2)
            .any(|pair| { pair[0] == "--features" && pair[1].contains("app/telemetry") }));
    }

    #[test]
    fn build_args_preserve_generic_features_without_backend_side_effects() {
        let args = CargoCommand::new("app")
            .feature("app/telemetry")
            .build_args();

        assert!(!args.iter().any(|arg| arg == "--no-default-features"));
        assert!(args
            .windows(2)
            .any(|pair| { pair[0] == "--features" && pair[1].contains("app/telemetry") }));
    }
}
