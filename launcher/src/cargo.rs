use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// On Windows, copy SDL2.dll from SDL2_LIB_DIR into the cargo target dir so the
/// executable can find it at runtime without needing SDL2_LIB_DIR in PATH.
pub fn ensure_sdl2_dll(workspace_root: &Path, profile: Option<&str>) {
    #[cfg(target_os = "windows")]
    {
        let lib_dir = match std::env::var("SDL2_LIB_DIR") {
            Ok(d) => std::path::PathBuf::from(d),
            Err(_) => return,
        };
        let src = lib_dir.join("SDL2.dll");
        if !src.exists() {
            return;
        }

        let target_subdir = match profile {
            Some("release") => "release",
            _ => "debug",
        };
        let dst_dir = workspace_root.join("target").join(target_subdir);
        let dst = dst_dir.join("SDL2.dll");

        if dst.exists() {
            return; // already there
        }

        if let Err(e) = std::fs::copy(&src, &dst) {
            eprintln!("\x1b[33m[se] warning: could not copy SDL2.dll to {}: {}\x1b[0m", dst.display(), e);
            eprintln!("\x1b[33m[se] hint: add {} to your PATH or copy SDL2.dll manually\x1b[0m", lib_dir.display());
        } else {
            println!("\x1b[2m[se] copied SDL2.dll → {}\x1b[0m", dst.display());
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = workspace_root;
        let _ = profile;
    }
}

pub struct CargoCommand {
    profile: Option<String>,
    features: Vec<String>,
    package: String,
    app_args: Vec<String>,
}

impl CargoCommand {
    pub fn new(package: impl Into<String>) -> Self {
        Self {
            profile: None,
            features: Vec::new(),
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
        if self.features.iter().any(|f| f == "sdl2") {
            ensure_sdl2_dll(workspace_root, self.profile.as_deref());
        }
        let args = self.build_args();
        
        let mut cmd = Command::new("cargo");
        cmd.args(&args).current_dir(workspace_root);
        inject_sdl2_rustflags(&mut cmd, &self.features);
        cmd.status().context("failed to execute cargo")
    }
    
    #[allow(dead_code)]
    pub fn spawn(self, workspace_root: &Path) -> Result<std::process::Child> {
        if self.features.iter().any(|f| f == "sdl2") {
            ensure_sdl2_dll(workspace_root, self.profile.as_deref());
        }
        let args = self.build_args();
        
        let mut cmd = Command::new("cargo");
        cmd.args(&args).current_dir(workspace_root);
        inject_sdl2_rustflags(&mut cmd, &self.features);
        cmd.spawn().context("failed to spawn cargo")
    }
}

/// When building with the sdl2 feature on Windows, ensure RUSTFLAGS includes
/// `-L native=<SDL2_LIB_DIR>` so the linker can find SDL2.lib.
fn inject_sdl2_rustflags(cmd: &mut Command, features: &[String]) {
    #[cfg(target_os = "windows")]
    if features.iter().any(|f| f == "sdl2") {
        if let Ok(lib_dir) = std::env::var("SDL2_LIB_DIR") {
            let needed = format!("-L native={}", lib_dir);
            let current = std::env::var("RUSTFLAGS").unwrap_or_default();
            if !current.contains(&needed) {
                let new_flags = if current.is_empty() {
                    needed
                } else {
                    format!("{} {}", current, needed)
                };
                cmd.env("RUSTFLAGS", &new_flags);
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = cmd;
        let _ = features;
    }
}
