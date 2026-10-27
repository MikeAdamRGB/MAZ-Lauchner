//! Updates: prüft auf GitHub, ob es eine neue Version gibt, und installiert sie
//! direkt aus der App – passend zur Installationsart:
//!
//! - Windows-EXE, AppImage: neue Datei laden, prüfen (SHA256), alte ersetzen, neu starten
//! - macOS-App: neue .app laden, prüfen, austauschen, neu starten
//! - .deb / .rpm / Pacman / Snap / Flatpak: Paket laden, prüfen und mit dem Paketmanager
//!   installieren (fragt nach dem Passwort). Geht das nicht, wird der Befehl angezeigt.
//!
//! Neue Versionen werden weiterhin ganz normal auf GitHub veröffentlicht.

use crate::sys;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

const API: &str = "https://api.github.com/repos/MikeAdamRGB/Open-Game-Launcher/releases/latest";
/// Liste der letzten Releases (inkl. Test-Versionen)
const API_ALL: &str = "https://api.github.com/repos/MikeAdamRGB/Open-Game-Launcher/releases?per_page=15";
const DOWNLOAD_PREFIX: &str = "https://github.com/MikeAdamRGB/Open-Game-Launcher/releases/download/";
/// Früherer Name des Repositorys (bis 1.8) – GitHub leitet weiter
const OLD_DOWNLOAD_PREFIX: &str = "https://github.com/MikeAdamRGB/MAZ-Lauchner/releases/download/";

/// SICHERHEIT: nur Dateien aus genau diesem Repository
fn allowed_url(url: &str) -> bool {
    url.starts_with(DOWNLOAD_PREFIX) || url.starts_with(OLD_DOWNLOAD_PREFIX)
}
const MAX_SIZE: u64 = 400 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq)]
pub struct Asset {
    pub name: String,
    pub url: String,
    pub size: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Release {
    pub version: String,
    pub notes: String,
    pub page: String,
    pub assets: Vec<Asset>,
    /// Test-Version (auf GitHub als „Pre-release“ markiert)
    pub prerelease: bool,
}

/// Wie ist der Launcher installiert?
#[derive(Clone, Debug, PartialEq)]
pub enum InstallKind {
    WindowsExe(PathBuf),
    AppImage(PathBuf),
    MacApp(PathBuf),
    Deb,
    Rpm,
    Pacman,
    Flatpak,
    Snap,
    /// Unbekannt (z. B. selbst gebaut) → nur Hinweis + GitHub
    Manual,
}

impl InstallKind {
    pub fn detect() -> InstallKind {
        let exe = std::env::current_exe().ok();
        if cfg!(windows) {
            return exe.map(InstallKind::WindowsExe).unwrap_or(InstallKind::Manual);
        }
        if cfg!(target_os = "macos") {
            // …/MAZ Launcher.app/Contents/MacOS/maz-launcher
            if let Some(app) = exe.as_ref().and_then(|e| e.ancestors().find(|a| a.extension().map(|x| x == "app").unwrap_or(false))) {
                return InstallKind::MacApp(app.to_path_buf());
            }
            return InstallKind::Manual;
        }
        if let Some(p) = sys::env_path("APPIMAGE") {
            return InstallKind::AppImage(p);
        }
        if std::env::var_os("FLATPAK_ID").is_some() {
            return InstallKind::Flatpak;
        }
        if std::env::var_os("SNAP").is_some() {
            return InstallKind::Snap;
        }
        if Path::new("/var/lib/dpkg/info/maz-launcher.list").exists() {
            return InstallKind::Deb;
        }
        if std::fs::read_dir("/var/lib/pacman/local")
            .map(|rd| rd.flatten().any(|e| e.file_name().to_string_lossy().starts_with("maz-launcher-")))
            .unwrap_or(false)
        {
            return InstallKind::Pacman;
        }
        if std::process::Command::new(sys::unix_tool("rpm")).args(["-q", "maz-launcher"]).output().map(|o| o.status.success()).unwrap_or(false) {
            return InstallKind::Rpm;
        }
        InstallKind::Manual
    }

    /// Kurzer Name für die Anzeige
    pub fn label(&self) -> &'static str {
        match self {
            InstallKind::WindowsExe(_) => "Windows",
            InstallKind::AppImage(_) => "AppImage",
            InstallKind::MacApp(_) => "macOS",
            InstallKind::Deb => ".deb",
            InstallKind::Rpm => ".rpm",
            InstallKind::Pacman => "Pacman",
            InstallKind::Flatpak => "Flatpak",
            InstallKind::Snap => "Snap",
            InstallKind::Manual => "",
        }
    }

    /// Kann die App das Update selbst einspielen?
    pub fn can_update(&self) -> bool {
        !matches!(self, InstallKind::Manual)
    }
}

fn arch() -> (&'static str, &'static str) {
    // (Linux/AppImage-Name, Debian-Name)
    if cfg!(target_arch = "aarch64") {
        ("aarch64", "arm64")
    } else {
        ("x86_64", "amd64")
    }
}

/// Passende Datei aus dem Release für diese Installation
pub fn pick_asset<'a>(kind: &InstallKind, assets: &'a [Asset]) -> Option<&'a Asset> {
    let (a, deb) = arch();
    let find = |f: &dyn Fn(&str) -> bool| assets.iter().find(|x| f(&x.name));
    match kind {
        InstallKind::WindowsExe(_) => {
            // Neuer Name zuerst, alter Name (bis 1.7) als Ausweich
            if cfg!(target_arch = "aarch64") {
                find(&|n| n == "Open-Game-Launcher-arm64.exe").or_else(|| find(&|n| n == "MAZ-Launcher-arm64.exe"))
            } else {
                find(&|n| n == "Open-Game-Launcher.exe").or_else(|| find(&|n| n == "MAZ-Launcher.exe"))
            }
        }
        InstallKind::AppImage(_) => find(&|n| n.ends_with(&format!("-{a}.AppImage"))),
        InstallKind::MacApp(_) => find(&|n| n.ends_with("-macOS.app.zip")),
        InstallKind::Deb => find(&|n| n.ends_with(&format!("_{deb}.deb"))),
        InstallKind::Rpm => find(&|n| n.ends_with(&format!(".{a}.rpm"))),
        InstallKind::Pacman => find(&|n| n.ends_with(&format!("-{a}.pkg.tar.zst"))),
        InstallKind::Flatpak => find(&|n| n.ends_with(&format!("-{a}.flatpak"))),
        InstallKind::Snap => find(&|n| n.ends_with(&format!("_{deb}.snap"))),
        InstallKind::Manual => None,
    }
}

/// "v1.10.2" → (1, 10, 2)
pub fn parse_version(v: &str) -> Option<(u32, u32, u32)> {
    let v = v.trim().trim_start_matches(['v', 'V']);
    let core = v.split(['-', '+']).next()?;
    let mut it = core.split('.').map(|p| p.parse::<u32>().ok());
    Some((it.next()??, it.next().flatten().unwrap_or(0), it.next().flatten().unwrap_or(0)))
}

pub fn is_newer(remote: &str, local: &str) -> bool {
    match (parse_version(remote), parse_version(local)) {
        (Some(r), Some(l)) => r > l,
        _ => false,
    }
}

fn agent(timeout: u64) -> ureq::Agent {
    use ureq::tls::{RootCerts, TlsConfig, TlsProvider};
    let tls = TlsConfig::builder()
        .provider(TlsProvider::NativeTls)
        .root_certs(RootCerts::PlatformVerifier)
        .build();
    ureq::Agent::config_builder()
        .https_only(true)
        .tls_config(tls)
        .timeout_global(Some(Duration::from_secs(timeout)))
        .user_agent(format!("Open-Game-Launcher/{}", sys::VERSION))
        .build()
        .into()
}

fn get_json(url: &str) -> Result<serde_json::Value, String> {
    let mut resp = agent(15)
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| format!("{}: {e}", crate::i18n::t("common.err_network")))?;
    resp.body_mut().read_json().map_err(|e| e.to_string())
}

/// Ein Release aus der GitHub-Antwort lesen (None = Entwurf oder ungültige Version)
pub fn parse_release(v: &serde_json::Value) -> Option<Release> {
    if v.get("draft").and_then(|d| d.as_bool()).unwrap_or(false) {
        return None;
    }
    let tag = v.get("tag_name").and_then(|t| t.as_str()).unwrap_or("").to_string();
    parse_version(&tag)?;
    let assets = v
        .get("assets")
        .and_then(|a| a.as_array())
        .map(|list| {
            list.iter()
                .filter_map(|a| {
                    let url = a.get("browser_download_url")?.as_str()?.to_string();
                    // SICHERHEIT: nur Dateien aus genau diesem Repository
                    if !allowed_url(&url) {
                        return None;
                    }
                    Some(Asset {
                        name: a.get("name")?.as_str()?.to_string(),
                        url,
                        size: a.get("size").and_then(|s| s.as_u64()).unwrap_or(0),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    Some(Release {
        version: tag.trim_start_matches(['v', 'V']).to_string(),
        notes: v.get("body").and_then(|b| b.as_str()).unwrap_or("").chars().take(6000).collect(),
        page: v
            .get("html_url")
            .and_then(|b| b.as_str())
            .filter(|u| u.starts_with(&format!("{}/releases/", sys::GITHUB_URL)))
            .unwrap_or(sys::GITHUB_URL)
            .to_string(),
        assets,
        prerelease: v.get("prerelease").and_then(|p| p.as_bool()).unwrap_or(false),
    })
}

/// Höchste Version aus einer Release-Liste. Bei gleicher Nummer gewinnt die fertige Version.
pub fn newest(list: &serde_json::Value) -> Option<Release> {
    list.as_array()?
        .iter()
        .filter_map(parse_release)
        .max_by_key(|r| (parse_version(&r.version), !r.prerelease))
}

/// Neuestes Release von GitHub lesen. `test` = auch Test-Versionen berücksichtigen.
pub fn latest(test: bool) -> Result<Release, String> {
    let r = if test { newest(&get_json(API_ALL)?) } else { parse_release(&get_json(API)?) };
    r.ok_or_else(|| "GitHub: no valid release".into())
}

/// Gibt es eine neuere Version? (None = aktuell)
pub fn check(test: bool) -> Result<Option<Release>, String> {
    let r = latest(test)?;
    Ok(is_newer(&r.version, sys::VERSION).then_some(r))
}

fn download(asset: &Asset, progress: &Arc<AtomicU64>) -> Result<Vec<u8>, String> {
    if !allowed_url(&asset.url) {
        return Err("Invalid download URL".into());
    }
    let mut resp = agent(900).get(&asset.url).call().map_err(|e| format!("{}: {e}", crate::i18n::t("common.err_network")))?;
    let mut reader = resp.body_mut().with_config().limit(MAX_SIZE).reader();
    let mut data = Vec::with_capacity(asset.size.min(MAX_SIZE) as usize);
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = std::io::Read::read(&mut reader, &mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        data.extend_from_slice(&buf[..n]);
        progress.store(data.len() as u64, Ordering::Relaxed);
    }
    Ok(data)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Prüfsumme aus SHA256SUMS.txt ("<hash>  <datei>")
pub fn expected_hash(sums: &str, name: &str) -> Option<String> {
    sums.lines().find_map(|l| {
        let mut parts = l.split_whitespace();
        let h = parts.next()?;
        let n = parts.next()?.trim_start_matches('*');
        (n == name && h.len() == 64).then(|| h.to_lowercase())
    })
}

/// Ergebnis eines Updates
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    /// Neue Version liegt bereit → App neu starten (Pfad zum Starten)
    Restart(PathBuf),
    /// Paket wurde installiert → Neustart nötig
    Installed,
    /// Konnte nicht automatisch installieren → Datei liegt hier, Befehl zum Installieren
    Manual { file: PathBuf, command: String },
}

/// Lädt die passende Datei, prüft sie und installiert sie.
pub fn install(release: &Release, kind: &InstallKind, progress: Arc<AtomicU64>) -> Result<Outcome, String> {
    let asset = pick_asset(kind, &release.assets).ok_or_else(|| crate::i18n::t("update.err_no_file"))?;
    let sums_asset = release.assets.iter().find(|a| a.name == "SHA256SUMS.txt").ok_or_else(|| crate::i18n::t("update.err_no_sums"))?;
    let sums = String::from_utf8_lossy(&download(sums_asset, &Arc::new(AtomicU64::new(0)))?).to_string();
    let want = expected_hash(&sums, &asset.name).ok_or_else(|| crate::i18n::t("update.err_no_sums"))?;
    let data = download(asset, &progress)?;
    let got = hex(&crate::util::sha256(&data));
    if got != want {
        return Err(crate::i18n::t("update.err_hash"));
    }
    match kind {
        InstallKind::WindowsExe(exe) | InstallKind::AppImage(exe) => replace_file(exe, &data).map(|_| Outcome::Restart(exe.clone())),
        InstallKind::MacApp(app) => replace_mac_app(app, &data).map(|_| Outcome::Restart(app.clone())),
        InstallKind::Manual => Err(crate::i18n::t("update.err_no_file")),
        _ => install_package(kind, &asset.name, &data),
    }
}

/// Laufende Datei ersetzen: Windows und Linux erlauben das Umbenennen einer laufenden Datei.
fn replace_file(target: &Path, data: &[u8]) -> Result<(), String> {
    let dir = target.parent().ok_or("No folder")?;
    let name = target.file_name().ok_or("No name")?.to_string_lossy().to_string();
    let new = dir.join(format!("{name}.new"));
    let old = dir.join(format!("{name}.alt-{}", sys::now_secs()));
    std::fs::write(&new, data).map_err(|e| format!("{}: {e}", crate::i18n::t("update.err_write")))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&new, std::fs::Permissions::from_mode(0o755));
    }
    std::fs::rename(target, &old).map_err(|e| {
        let _ = std::fs::remove_file(&new);
        format!("{}: {e}", crate::i18n::t("update.err_write"))
    })?;
    if let Err(e) = std::fs::rename(&new, target) {
        let _ = std::fs::rename(&old, target);
        return Err(format!("{}: {e}", crate::i18n::t("update.err_write")));
    }
    // Unter Linux kann die alte Datei sofort weg (das laufende Programm behält sie im Speicher)
    if !cfg!(windows) {
        let _ = std::fs::remove_file(&old);
    }
    Ok(())
}

/// macOS: .app aus dem Zip entpacken und austauschen
fn replace_mac_app(app: &Path, data: &[u8]) -> Result<(), String> {
    let tmp = std::env::temp_dir().join(format!("maz-update-{}", sys::now_secs()));
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let zip = tmp.join("update.zip");
    std::fs::write(&zip, data).map_err(|e| e.to_string())?;
    let ok = std::process::Command::new("/usr/bin/ditto")
        .args(["-x", "-k"])
        .arg(&zip)
        .arg(&tmp)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !ok {
        return Err(crate::i18n::t("update.err_write"));
    }
    let new_app = std::fs::read_dir(&tmp)
        .map_err(|e| e.to_string())?
        .flatten()
        .map(|e| e.path())
        .find(|p| p.extension().map(|x| x == "app").unwrap_or(false))
        .ok_or_else(|| crate::i18n::t("update.err_write"))?;
    let old = app.with_extension(format!("app.alt-{}", sys::now_secs()));
    std::fs::rename(app, &old).map_err(|e| format!("{}: {e}", crate::i18n::t("update.err_write")))?;
    if let Err(e) = std::fs::rename(&new_app, app).or_else(|_| {
        std::process::Command::new("/usr/bin/ditto").arg(&new_app).arg(app).status().map(|_| ())
    }) {
        let _ = std::fs::rename(&old, app);
        return Err(format!("{}: {e}", crate::i18n::t("update.err_write")));
    }
    let _ = std::fs::remove_dir_all(&old);
    let _ = std::fs::remove_dir_all(&tmp);
    Ok(())
}

/// Linux-Pakete: Datei in "Downloads" speichern und mit dem Paketmanager installieren
fn install_package(kind: &InstallKind, name: &str, data: &[u8]) -> Result<Outcome, String> {
    // SICHERHEIT: in einen privaten Ordner (nur der eigene Benutzer, 0700) statt "Downloads",
    // damit niemand die geprüfte Datei austauschen kann, bevor sie installiert wird.
    let dir = sys::local_dir().join("update");
    // Nur einfache Dateinamen aus dem Release übernehmen
    let safe_name: String = name.chars().filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_')).collect();
    if safe_name.is_empty() || safe_name.starts_with('.') {
        return Err(crate::i18n::t("update.err_no_file"));
    }
    let file = dir.join(safe_name);
    sys::write_private(&file, data).map_err(|e| format!("{}: {e}", crate::i18n::t("update.err_write")))?;
    let f = file.to_string_lossy().to_string();
    let (prog, args): (&str, Vec<String>) = match kind {
        InstallKind::Deb => ("pkexec", vec!["apt-get".into(), "install".into(), "-y".into(), f.clone()]),
        InstallKind::Rpm => {
            if Path::new("/usr/bin/zypper").exists() {
                ("pkexec", vec!["zypper".into(), "--non-interactive".into(), "install".into(), "--allow-unsigned-rpm".into(), f.clone()])
            } else {
                ("pkexec", vec!["dnf".into(), "install".into(), "-y".into(), f.clone()])
            }
        }
        InstallKind::Pacman => ("pkexec", vec!["pacman".into(), "-U".into(), "--noconfirm".into(), f.clone()]),
        InstallKind::Snap => ("pkexec", vec!["snap".into(), "install".into(), "--dangerous".into(), "--classic".into(), f.clone()]),
        InstallKind::Flatpak => ("flatpak", vec!["install".into(), "--user".into(), "-y".into(), "--noninteractive".into(), "--reinstall".into(), f.clone()]),
        _ => return Err(crate::i18n::t("update.err_no_file")),
    };
    let command = if prog == "pkexec" { format!("sudo {}", args.join(" ")) } else { format!("{prog} {}", args.join(" ")) };
    let ok = sys::host_command(prog).args(&args).status().map(|s| s.success()).unwrap_or(false);
    if ok {
        Ok(Outcome::Installed)
    } else {
        Ok(Outcome::Manual { file, command })
    }
}

/// Nach dem Update neu starten
pub fn restart(target: &Path) {
    if cfg!(target_os = "macos") {
        let _ = std::process::Command::new("/usr/bin/open").arg("-n").arg(target).spawn();
    } else {
        let _ = std::process::Command::new(target).spawn();
    }
    std::process::exit(0);
}

/// Alte Dateien von früheren Updates aufräumen (Windows: *.alt-*)
pub fn cleanup_old() {
    let Some(exe) = std::env::current_exe().ok() else { return };
    let Some(dir) = exe.parent() else { return };
    let Some(name) = exe.file_name().map(|n| n.to_string_lossy().to_string()) else { return };
    for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let n = e.file_name().to_string_lossy().to_string();
        if n.starts_with(&format!("{name}.alt-")) || n == format!("{name}.new") {
            let _ = std::fs::remove_file(e.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions() {
        assert!(is_newer("v1.4.0", "1.3.2"));
        assert!(is_newer("1.10.0", "1.9.9"));
        assert!(!is_newer("1.3.2", "1.3.2"));
        assert!(!is_newer("v1.2.0", "1.3.2"));
        assert!(!is_newer("kaputt", "1.0.0"));
        assert_eq!(parse_version("v2"), Some((2, 0, 0)));
    }

    #[test]
    fn assets_and_hashes() {
        let a = |n: &str| Asset { name: n.into(), url: format!("{DOWNLOAD_PREFIX}v1/{n}"), size: 1 };
        let list = vec![
            a("MAZ-Launcher.exe"),
            a("MAZ-Launcher-arm64.exe"),
            a("MAZ-Launcher-1.4.0-x86_64.AppImage"),
            a("MAZ-Launcher-1.4.0-aarch64.AppImage"),
            a("maz-launcher_1.4.0_amd64.deb"),
            a("maz-launcher-1.4.0-1.x86_64.rpm"),
            a("maz-launcher-1.4.0-1-x86_64.pkg.tar.zst"),
            a("MAZ-Launcher-1.4.0-macOS.app.zip"),
            a("SHA256SUMS.txt"),
        ];
        if cfg!(target_arch = "x86_64") {
            assert_eq!(pick_asset(&InstallKind::Deb, &list).unwrap().name, "maz-launcher_1.4.0_amd64.deb");
            assert_eq!(pick_asset(&InstallKind::AppImage(PathBuf::new()), &list).unwrap().name, "MAZ-Launcher-1.4.0-x86_64.AppImage");
            assert_eq!(pick_asset(&InstallKind::Rpm, &list).unwrap().name, "maz-launcher-1.4.0-1.x86_64.rpm");
            assert_eq!(pick_asset(&InstallKind::Pacman, &list).unwrap().name, "maz-launcher-1.4.0-1-x86_64.pkg.tar.zst");
            assert_eq!(pick_asset(&InstallKind::WindowsExe(PathBuf::new()), &list).unwrap().name, "MAZ-Launcher.exe");
        }
        assert_eq!(pick_asset(&InstallKind::MacApp(PathBuf::new()), &list).unwrap().name, "MAZ-Launcher-1.4.0-macOS.app.zip");
        assert!(pick_asset(&InstallKind::Manual, &list).is_none());
        assert!(allowed_url("https://github.com/MikeAdamRGB/Open-Game-Launcher/releases/download/v1/x.exe"));
        assert!(allowed_url("https://github.com/MikeAdamRGB/MAZ-Lauchner/releases/download/v1/x.exe"));
        assert!(!allowed_url("https://github.com/someone/else/releases/download/v1/x.exe"));
        let sums = "ab12  other.exe\n0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef  MAZ-Launcher.exe\n";
        assert_eq!(expected_hash(sums, "MAZ-Launcher.exe").unwrap().len(), 64);
        // Neuer Name hat Vorrang vor dem alten
        if cfg!(target_arch = "x86_64") {
            let mut both = list.clone();
            both.push(a("Open-Game-Launcher.exe"));
            assert_eq!(pick_asset(&InstallKind::WindowsExe(PathBuf::new()), &both).unwrap().name, "Open-Game-Launcher.exe");
        }
        assert!(expected_hash(sums, "other.exe").is_none());
    }

    #[test]
    fn test_versions() {
        let rel = |tag: &str, pre: bool, draft: bool| {
            serde_json::json!({"tag_name": tag, "prerelease": pre, "draft": draft, "body": "", "assets": []})
        };
        let list = serde_json::json!([rel("v1.8.0", false, true), rel("v1.7.0", true, false), rel("v1.6.0", false, false)]);
        let r = newest(&list).unwrap();
        assert_eq!(r.version, "1.7.0");
        assert!(r.prerelease);
        // Fertige Version mit gleicher Nummer hat Vorrang
        let list = serde_json::json!([rel("v1.7.0", true, false), rel("v1.7.0", false, false)]);
        assert!(!newest(&list).unwrap().prerelease);
        assert!(parse_release(&rel("v2.0.0", false, true)).is_none());
        assert!(parse_release(&rel("kaputt", false, false)).is_none());
        assert!(newest(&serde_json::json!({})).is_none());
    }
}
