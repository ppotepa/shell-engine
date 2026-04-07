use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PlatformEnv {
    pub is_windows: bool,
    pub sdl2_lib_dir: Option<PathBuf>,
    pub sdl2_include_dir: Option<PathBuf>,
    pub rustflags: Option<String>,
}

pub fn detect_platform_env() -> PlatformEnv {
    let is_windows = cfg!(target_os = "windows");
    
    let mut env = PlatformEnv {
        is_windows,
        sdl2_lib_dir: None,
        sdl2_include_dir: None,
        rustflags: None,
    };
    
    if is_windows {
        reload_user_env_vars();
    }
    
    env.sdl2_lib_dir = env::var("SDL2_LIB_DIR").ok().map(PathBuf::from);
    env.sdl2_include_dir = env::var("SDL2_INCLUDE_DIR").ok().map(PathBuf::from);
    env.rustflags = env::var("RUSTFLAGS").ok();
    
    env
}

#[cfg(target_os = "windows")]
fn reload_user_env_vars() {
    let var_names = ["RUSTFLAGS", "SDL2_LIB_DIR", "SDL2_INCLUDE_DIR"];
    
    for var_name in &var_names {
        if let Ok(value) = read_registry_string("HKEY_CURRENT_USER\\Environment", var_name) {
            if env::var(var_name).is_err() {
                env::set_var(var_name, value);
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn read_registry_string(key_path: &str, value_name: &str) -> Result<String> {
    use winreg::enums::*;
    
    let (root, subkey) = if let Some(rest) = key_path.strip_prefix("HKEY_CURRENT_USER\\") {
        (winreg::RegKey::predef(HKEY_CURRENT_USER), rest)
    } else if let Some(rest) = key_path.strip_prefix("HKEY_LOCAL_MACHINE\\") {
        (winreg::RegKey::predef(HKEY_LOCAL_MACHINE), rest)
    } else {
        anyhow::bail!("unsupported registry root")
    };
    
    let key = root.open_subkey(subkey).context("failed to open registry key")?;
    let value: String = key.get_value(value_name).context("failed to read registry value")?;
    Ok(value)
}

#[cfg(not(target_os = "windows"))]
fn reload_user_env_vars() {
}

#[allow(dead_code)]
pub fn check_sdl2_available(env: &PlatformEnv) -> bool {
    if env.is_windows {
        env.sdl2_lib_dir.as_ref().map_or(false, |p| p.exists())
    } else {
        which::which("sdl2-config").is_ok()
    }
}
