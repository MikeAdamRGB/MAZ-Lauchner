//! Starthilfen pro Spiel:
//! - Programm vor bzw. nach dem Spiel ausführen (alle Systeme)
//! - Linux: GameMode und MangoHud als Schalter
//! - Linux mit Wine/Proton: DXVK (DirectX 9–11 über Vulkan), VKD3D-Proton (DirectX 12),
//!   DXVK-Anzeige und NVIDIA-DLSS (NVAPI)
//!
//! SICHERHEIT: Programme vor/nach dem Spiel werden ohne Shell gestartet (keine Umleitungen,
//! keine verketteten Befehle) und nur, wenn die Datei existiert.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LaunchOpts {
    /// Vor dem Start: Programm + Parameter, z. B. "C:\Tools\fan.exe --game"
    pub pre: String,
    /// Nach dem Spiel (sobald das Spiel beendet ist)
    pub post: String,
    pub gamemode: bool,
    pub mangohud: bool,
    /// DXVK an (Standard) – aus = WineD3D (OpenGL)
    pub dxvk: bool,
    /// VKD3D-Proton für DirectX 12 an (Standard)
    pub vkd3d: bool,
    /// DXVK-Anzeige (FPS) im Spiel
    pub dxvk_hud: bool,
    /// NVIDIA DLSS / NVAPI einschalten
    pub nvapi: bool,
}

impl Default for LaunchOpts {
    fn default() -> Self {
        LaunchOpts { pre: String::new(), post: String::new(), gamemode: false, mangohud: false, dxvk: true, vkd3d: true, dxvk_hud: false, nvapi: false }
    }
}

impl LaunchOpts {
    pub fn is_default(&self) -> bool {
        *self == LaunchOpts::default()
    }

    /// Umgebungsvariablen für Wine bzw. Proton
    pub fn wine_env(&self, proton: bool) -> Vec<(String, String)> {
        let mut env = Vec::new();
        let mut overrides: Vec<&str> = Vec::new();
        if !self.dxvk {
            if proton {
                env.push(("PROTON_USE_WINED3D".into(), "1".into()));
            } else {
                overrides.push("d3d9,d3d10core,d3d11,dxgi=b");
            }
        }
        if !self.vkd3d {
            if proton {
                env.push(("PROTON_NO_D3D12".into(), "1".into()));
            } else {
                overrides.push("d3d12,d3d12core=b");
            }
        }
        if !overrides.is_empty() {
            env.push(("WINEDLLOVERRIDES".into(), overrides.join(";")));
        }
        if self.dxvk_hud {
            env.push(("DXVK_HUD".into(), "fps,gpuload".into()));
        }
        if self.nvapi {
            env.push(("PROTON_ENABLE_NVAPI".into(), "1".into()));
            env.push(("DXVK_ENABLE_NVAPI".into(), "1".into()));
        }
        env
    }

    /// Programme, die vor das Spiel gesetzt werden (nur Linux, nur wenn installiert)
    pub fn wrappers(&self) -> Vec<String> {
        let mut v = Vec::new();
        if cfg!(target_os = "linux") {
            if self.gamemode && tool_available("gamemoderun") {
                v.push("gamemoderun".to_string());
            }
            if self.mangohud && tool_available("mangohud") {
                v.push("mangohud".to_string());
            }
        }
        v
    }
}

/// Ist ein Linux-Werkzeug installiert? (auch für das Flatpak: auf dem Host nachsehen)
pub fn tool_available(name: &str) -> bool {
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return false;
    }
    if std::env::var_os("FLATPAK_ID").is_some() {
        return crate::sys::host_command("which").arg(name).output().map(|o| o.status.success()).unwrap_or(false);
    }
    let path = std::env::var_os("PATH").unwrap_or_default();
    std::env::split_paths(&path)
        .chain(["/usr/bin", "/usr/local/bin", "/run/current-system/sw/bin"].iter().map(std::path::PathBuf::from))
        .any(|d| d.join(name).is_file())
}

/// Befehl zerlegen: Programm (auch in Anführungszeichen) + Parameter
pub fn split(cmd: &str) -> Option<(String, Vec<String>)> {
    let args = crate::emulators::build_args(cmd.trim(), "");
    let mut it = args.into_iter().filter(|a| !a.is_empty());
    let prog = it.next()?;
    Some((prog, it.collect()))
}

/// Programm vor/nach dem Spiel starten (wartet nicht)
pub fn run_helper(cmd: &str) -> Result<(), String> {
    let Some((prog, args)) = split(cmd) else { return Ok(()) };
    if !Path::new(&prog).is_file() {
        return Err(crate::i18n::tf("opts.err_missing", &[("path", &prog)]));
    }
    let mut c = crate::sys::host_command(&prog);
    c.args(&args);
    if let Some(dir) = Path::new(&prog).parent() {
        if std::env::var_os("FLATPAK_ID").is_none() {
            c.current_dir(dir);
        }
    }
    c.spawn().map(|_| ()).map_err(|e| e.to_string())
}

/// Kommando mit Vorsatz-Programmen (GameMode/MangoHud) und Umgebung bauen
pub fn command(opts: &LaunchOpts, program: &str, env: &[(String, String)], dir: Option<&Path>) -> std::process::Command {
    let wrappers = opts.wrappers();
    match wrappers.split_first() {
        None => crate::sys::host_command_env(program, env, dir),
        Some((first, rest)) => {
            let mut c = crate::sys::host_command_env(first, env, dir);
            c.args(rest);
            c.arg(program);
            c
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_and_split() {
        let d = LaunchOpts::default();
        assert!(d.is_default());
        assert!(d.wine_env(true).is_empty());
        let o = LaunchOpts { dxvk: false, vkd3d: false, nvapi: true, dxvk_hud: true, ..Default::default() };
        let p = o.wine_env(true);
        assert!(p.contains(&("PROTON_USE_WINED3D".into(), "1".into())));
        assert!(p.contains(&("PROTON_NO_D3D12".into(), "1".into())));
        let w = o.wine_env(false);
        assert!(w.iter().any(|(k, v)| k == "WINEDLLOVERRIDES" && v.contains("d3d11") && v.contains("d3d12")));
        assert_eq!(split("\"C:\\Program Files\\Tool\\a.exe\" --x 1"), Some(("C:\\Program Files\\Tool\\a.exe".into(), vec!["--x".into(), "1".into()])));
        assert_eq!(split("   "), None);
        assert!(run_helper("/gibt/es/nicht --x").is_err());
        assert!(!tool_available("../evil"));
        // Alte Einstellungen ohne Starthilfen laden weiter
        let s: crate::model::Settings = serde_json::from_str("{}").unwrap();
        assert!(s.launch_opts.is_empty());
    }
}
