//! Flatpak: Wechsel der App-ID (bis 2.0 `io.github.mikeadamrgb.MAZLauncher`, seit 2.1
//! `io.github.mikeadamrgb.OpenGameLauncher`).
//!
//! Jede Flatpak-App hat ihren eigenen Datenordner (`~/.var/app/<ID>/`). Beim ersten Start mit
//! der neuen ID werden Einstellungen und Cache der alten App **kopiert** (nur lesen, die alte
//! App bleibt unverändert). Danach bietet der Launcher an, die alte App zu entfernen.
//!
//! SICHERHEIT: Symlinks werden nie verfolgt, es gibt eine Größen- und Anzahlgrenze, und
//! kopiert wird nur innerhalb der zwei festen Ordner.

use std::path::{Path, PathBuf};

pub const OLD_ID: &str = "io.github.mikeadamrgb.MAZLauncher";
pub const NEW_ID: &str = "io.github.mikeadamrgb.OpenGameLauncher";

const MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_FILES: usize = 200_000;

/// Läuft gerade die Flatpak-App mit der neuen ID?
pub fn is_new_flatpak() -> bool {
    std::env::var("FLATPAK_ID").map(|v| v == NEW_ID).unwrap_or(false)
}

struct Budget {
    bytes: u64,
    files: usize,
}

/// Ordner rekursiv kopieren – ohne Symlinks, mit Grenzen. `skip`: Namen auf oberster Ebene, die
/// nicht kopiert werden (z. B. WebView-Cache).
fn copy_tree(from: &Path, to: &Path, skip: &[&str], budget: &mut Budget, top: bool) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for e in std::fs::read_dir(from)? {
        let e = e?;
        let name = e.file_name();
        if top && skip.iter().any(|s| name.to_string_lossy() == *s) {
            continue;
        }
        let meta = std::fs::symlink_metadata(e.path())?;
        let target = to.join(&name);
        if meta.file_type().is_symlink() {
            continue;
        } else if meta.is_dir() {
            copy_tree(&e.path(), &target, skip, budget, false)?;
        } else if meta.is_file() {
            budget.files += 1;
            budget.bytes = budget.bytes.saturating_add(meta.len());
            if budget.files > MAX_FILES || budget.bytes > MAX_BYTES {
                return Err(std::io::Error::other("zu groß"));
            }
            std::fs::copy(e.path(), &target)?;
        }
    }
    Ok(())
}

/// Ordner `<base>/open-game-launcher` aus dem alten App-Ordner übernehmen, falls es ihn noch nicht gibt.
/// Gibt zurück, ob kopiert wurde.
pub fn migrate_one(old_base: &Path, new_base: &Path, skip: &[&str]) -> bool {
    let target = new_base.join("open-game-launcher");
    if target.exists() {
        return false;
    }
    for name in ["open-game-launcher", "maz-launcher"] {
        let src = old_base.join(name);
        let is_dir = std::fs::symlink_metadata(&src).map(|m| m.is_dir()).unwrap_or(false);
        if !is_dir {
            continue;
        }
        // Erst in einen Zwischenordner, dann umbenennen – so gibt es nie einen halben Ordner
        let tmp = new_base.join(".open-game-launcher.migrating");
        let _ = std::fs::remove_dir_all(&tmp);
        let mut budget = Budget { bytes: 0, files: 0 };
        let ok = copy_tree(&src, &tmp, skip, &mut budget, true).is_ok() && std::fs::rename(&tmp, &target).is_ok();
        if !ok {
            let _ = std::fs::remove_dir_all(&tmp);
        }
        return ok;
    }
    false
}

fn old_app_dir() -> Option<PathBuf> {
    Some(crate::sys::env_path("HOME")?.join(".var").join("app").join(OLD_ID))
}

/// Beim Start (vor dem Laden der Einstellungen) aufrufen
pub fn migrate() {
    if !is_new_flatpak() {
        return;
    }
    let Some(old) = old_app_dir() else { return };
    if let Some(cfg) = crate::sys::env_path("XDG_CONFIG_HOME") {
        migrate_one(&old.join("config"), &cfg, &[]);
    }
    if let Some(cache) = crate::sys::env_path("XDG_CACHE_HOME") {
        // WebView-Daten werden neu angelegt
        migrate_one(&old.join("cache"), &cache, &["webview"]);
    }
}

/// Ist die alte Flatpak-App noch installiert?
pub fn old_installed() -> bool {
    if !is_new_flatpak() {
        return false;
    }
    crate::sys::host_command("flatpak")
        .args(["info", OLD_ID])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Alte Flatpak-App entfernen (zuerst Benutzer-, dann System-Installation)
pub fn uninstall_old() -> bool {
    for scope in ["--user", "--system"] {
        let ok = crate::sys::host_command("flatpak")
            .args(["uninstall", scope, "-y", "--noninteractive", OLD_ID])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if ok {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copies_once_without_symlinks() {
        let root = std::env::temp_dir().join(format!("og-fpmig-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let old = root.join("old");
        let new = root.join("new");
        std::fs::create_dir_all(old.join("maz-launcher/sub")).unwrap();
        std::fs::create_dir_all(old.join("maz-launcher/webview")).unwrap();
        std::fs::create_dir_all(&new).unwrap();
        std::fs::write(old.join("maz-launcher/config.json"), "{}").unwrap();
        std::fs::write(old.join("maz-launcher/sub/a.txt"), "a").unwrap();
        std::fs::write(old.join("maz-launcher/webview/big"), "x").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink("/etc/passwd", old.join("maz-launcher/link")).unwrap();

        assert!(migrate_one(&old, &new, &["webview"]));
        let t = new.join("open-game-launcher");
        assert_eq!(std::fs::read_to_string(t.join("config.json")).unwrap(), "{}");
        assert!(t.join("sub/a.txt").exists());
        assert!(!t.join("webview").exists());
        assert!(!t.join("link").exists());
        // Zweites Mal: nichts überschreiben
        std::fs::write(t.join("config.json"), "{\"neu\":1}").unwrap();
        assert!(!migrate_one(&old, &new, &[]));
        assert_eq!(std::fs::read_to_string(t.join("config.json")).unwrap(), "{\"neu\":1}");
        let _ = std::fs::remove_dir_all(&root);
        assert!(!is_new_flatpak() || std::env::var("FLATPAK_ID").is_ok());
    }
}
