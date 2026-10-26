//! Minecraft: offizieller Minecraft Launcher und NoRisk Client.
//!
//! - **Minecraft Launcher**: Microsoft Store/Xbox-App, alter `MinecraftLauncher.exe`, macOS-App,
//!   Linux (`minecraft-launcher` oder Flatpak `com.mojang.Minecraft`).
//! - **NoRisk Client** (<https://norisk.gg>): Jedes NoRisk-Profil wird eine eigene Kachel.
//!   Gestartet wird über den offiziellen Befehl des NoRisk Launchers `launch --profile <ID>`.
//!   Die Profile werden nur aus einer *Kopie* seiner Datenbank gelesen. Es wird kein Code aus
//!   dem NoRisk-Projekt verwendet; das Microsoft-Konto bleibt im jeweiligen Launcher.

use crate::model::{Art, Game, LaunchTarget, Source, StoreStatus};
use crate::sys;
use std::path::{Path, PathBuf};

pub const STORE_PACKAGE: &str = "Microsoft.4297127D64EC6";
pub const NORISK_DOWNLOAD: &str = "https://norisk.gg/";
pub const MC_DOWNLOAD: &str = "https://www.minecraft.net/download";
const NORISK_FLATPAK: &str = "gg.norisk.NoRiskClientLauncherV3";
const MC_FLATPAK: &str = "com.mojang.Minecraft";

/// Ein Profil aus dem NoRisk Launcher
#[derive(Clone, Debug, PartialEq, Default)]
pub struct NoRiskProfile {
    pub id: String,
    pub name: String,
    pub game_version: String,
    pub loader: String,
    /// Spielordner des Profils (für Mods, Shader …)
    pub dir: PathBuf,
    pub last_played: u64,
}

/// Wie ein Launcher gestartet wird: Programm + feste Anfangs-Argumente
#[derive(Clone, Debug, PartialEq)]
pub struct Starter {
    pub program: String,
    pub prefix_args: Vec<String>,
}

fn flatpak_installed(app: &str) -> bool {
    let h = sys::home_dir();
    [PathBuf::from("/var/lib/flatpak/app").join(app), h.join(".local/share/flatpak/app").join(app)]
        .iter()
        .any(|p| p.is_dir())
}

fn first_existing(paths: Vec<PathBuf>) -> Option<PathBuf> {
    paths.into_iter().find(|p| p.is_file())
}

// ----------------------------------------------------------------------
// NoRisk Launcher
// ----------------------------------------------------------------------

/// Programm des NoRisk Launchers
pub fn norisk_starter() -> Option<Starter> {
    let exe = |p: PathBuf| Starter { program: p.to_string_lossy().to_string(), prefix_args: Vec::new() };
    if cfg!(windows) {
        // Eintrag unter "Apps" (DisplayIcon zeigt auf die EXE) – sonst übliche Ordner
        for e in crate::stores::uninstall_entries() {
            if e.display_name.to_lowercase().contains("norisk") {
                let icon = crate::stores::icon_path(&e.display_icon);
                if icon.to_lowercase().ends_with(".exe") && Path::new(&icon).is_file() {
                    return Some(exe(PathBuf::from(icon)));
                }
                if !e.install_location.is_empty() {
                    if let Some(p) = find_exe_in(Path::new(&e.install_location)) {
                        return Some(exe(p));
                    }
                }
            }
        }
        let mut dirs = Vec::new();
        if let Some(l) = sys::env_path("LOCALAPPDATA") {
            dirs.push(l.join("NoRisk Launcher"));
            dirs.push(l.join("Programs").join("NoRisk Launcher"));
        }
        if let Some(p) = sys::env_path("ProgramFiles") {
            dirs.push(p.join("NoRisk Launcher"));
        }
        return dirs.iter().find_map(|d| find_exe_in(d)).map(exe);
    }
    if cfg!(target_os = "macos") {
        for app in [PathBuf::from("/Applications/NoRisk Launcher.app"), sys::home_dir().join("Applications/NoRisk Launcher.app")] {
            let macos = app.join("Contents").join("MacOS");
            let found = std::fs::read_dir(&macos).into_iter().flatten().flatten().map(|e| e.path()).find(|p| p.is_file());
            if let Some(p) = found {
                return Some(exe(p));
            }
        }
        return None;
    }
    if let Some(p) = first_existing(vec![
        PathBuf::from("/usr/bin/noriskclient-launcher-v3"),
        PathBuf::from("/usr/local/bin/noriskclient-launcher-v3"),
        PathBuf::from("/opt/noriskclient-launcher-v3/noriskclient-launcher-v3"),
    ]) {
        return Some(exe(p));
    }
    if flatpak_installed(NORISK_FLATPAK) {
        return Some(Starter { program: sys::unix_tool("flatpak").to_string_lossy().to_string(), prefix_args: vec!["run".into(), NORISK_FLATPAK.into()] });
    }
    None
}

/// EXE des NoRisk Launchers in einem Ordner (Name je nach Version unterschiedlich)
fn find_exe_in(dir: &Path) -> Option<PathBuf> {
    let candidates = ["NoRisk Launcher.exe", "noriskclient-launcher-v3.exe", "NoRiskClientLauncher.exe"];
    for c in candidates {
        let p = dir.join(c);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// Datenordner des NoRisk Launchers (dort liegen `meta/app.db` und `profiles/`)
pub fn norisk_data_dirs() -> Vec<PathBuf> {
    let h = sys::home_dir();
    let v = if cfg!(windows) {
        sys::env_path("APPDATA").map(|a| vec![a.join("norisk").join("NoRiskClientV3").join("data")]).unwrap_or_default()
    } else if cfg!(target_os = "macos") {
        vec![h.join("Library/Application Support/gg.norisk.NoRiskClientV3")]
    } else {
        vec![
            sys::env_path("XDG_DATA_HOME").unwrap_or_else(|| h.join(".local/share")).join("noriskclientv3"),
            h.join(".var/app").join(NORISK_FLATPAK).join("data/noriskclientv3"),
        ]
    };
    v.into_iter().filter(|p| p.is_dir()).collect()
}

fn norisk_db(data: &Path) -> PathBuf {
    // Windows: <…>\NoRiskClientV3\meta, sonst <data>/meta
    if cfg!(windows) {
        data.parent().unwrap_or(data).join("meta").join("app.db")
    } else {
        data.join("meta").join("app.db")
    }
}

/// Profile aus der NoRisk-Datenbank lesen. Die Datenbank läuft im WAL-Modus; damit auch
/// gerade erst angelegte Profile erscheinen, wird eine Kopie (inkl. -wal) gelesen.
pub fn norisk_profiles() -> Vec<NoRiskProfile> {
    let mut out = Vec::new();
    for data in norisk_data_dirs() {
        let db = norisk_db(&data);
        if !db.is_file() {
            continue;
        }
        let tmp = sys::local_dir().join("norisk-copy");
        let _ = std::fs::remove_dir_all(&tmp);
        if std::fs::create_dir_all(&tmp).is_err() {
            continue;
        }
        let copy = tmp.join("app.db");
        // Größenlimit: Profile-Datenbanken sind klein
        if std::fs::metadata(&db).map(|m| m.len() > 256 * 1024 * 1024).unwrap_or(true) || std::fs::copy(&db, &copy).is_err() {
            continue;
        }
        let wal = db.with_file_name("app.db-wal");
        if wal.is_file() {
            let _ = std::fs::copy(&wal, tmp.join("app.db-wal"));
        }
        if let Some(conn) = crate::stores::open_db_wal(&copy) {
            for p in read_profiles(&conn, &data.join("profiles")) {
                if !out.iter().any(|o: &NoRiskProfile| o.id == p.id) {
                    out.push(p);
                }
            }
        }
        let _ = std::fs::remove_dir_all(&tmp);
    }
    out
}

/// Profile aus einer geöffneten NoRisk-Datenbank
pub fn read_profiles(conn: &rusqlite::Connection, profiles_dir: &Path) -> Vec<NoRiskProfile> {
    let sql = "SELECT id, name, path, game_version, loader, COALESCE(last_played, 0) FROM profiles LIMIT 2000";
    let Ok(mut st) = conn.prepare(sql) else { return Vec::new() };
    let rows: Vec<(String, String, String, String, String, i64)> = st
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)))
        .map(|it| it.flatten().collect())
        .unwrap_or_default();
    rows.into_iter()
        .filter(|(id, name, ..)| valid_profile_id(id) && !name.trim().is_empty())
        .map(|(id, name, path, game_version, loader, last)| NoRiskProfile {
            id,
            name: name.trim().chars().take(80).collect(),
            dir: profile_dir(profiles_dir, &path),
            game_version: game_version.chars().filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-')).take(20).collect(),
            loader: loader.to_lowercase().chars().filter(|c| c.is_ascii_lowercase()).take(12).collect(),
            last_played: last.max(0) as u64,
        })
        .collect()
}

/// SICHERHEIT: Die ID wird als Argument an den NoRisk Launcher übergeben → nur UUIDs
pub fn valid_profile_id(id: &str) -> bool {
    id.len() == 36 && id.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

/// Spielordner eines Profils – nur innerhalb des Profile-Ordners (kein "..")
pub fn profile_dir(profiles_dir: &Path, path: &str) -> PathBuf {
    let clean: String = path.trim().replace('\\', "/");
    let ok = !clean.is_empty() && !clean.starts_with('/') && !clean.contains(':') && !clean.split('/').any(|p| p == ".." || p.is_empty());
    if ok {
        profiles_dir.join(clean)
    } else {
        PathBuf::new()
    }
}

// ----------------------------------------------------------------------
// Offizieller Minecraft Launcher
// ----------------------------------------------------------------------

/// Wie der offizielle Minecraft Launcher gestartet wird
pub fn official_launch() -> Option<(LaunchTarget, String)> {
    if cfg!(windows) {
        if let Some(pkg) = crate::appx::find(STORE_PACKAGE) {
            let app = pkg.app_id("Minecraft")?;
            return Some((LaunchTarget::Appx { family: pkg.family.clone(), app }, format!("Microsoft Store: {}", pkg.full_name)));
        }
        let mut c = Vec::new();
        for var in ["ProgramFiles(x86)", "ProgramFiles"] {
            if let Some(p) = sys::env_path(var) {
                c.push(p.join("Minecraft Launcher").join("MinecraftLauncher.exe"));
            }
        }
        if let Some(a) = sys::env_path("APPDATA") {
            c.push(a.join(".minecraft").join("launcher").join("MinecraftLauncher.exe"));
        }
        return first_existing(c).map(|p| {
            let s = p.to_string_lossy().to_string();
            (LaunchTarget::Exe { path: s.clone(), args: Vec::new(), dir: String::new() }, s)
        });
    }
    if cfg!(target_os = "macos") {
        let app = ["/Applications/Minecraft.app".to_string(), sys::home_dir().join("Applications/Minecraft.app").to_string_lossy().to_string()]
            .into_iter()
            .find(|p| Path::new(p).is_dir())?;
        return Some((LaunchTarget::Exe { path: "/usr/bin/open".into(), args: vec!["-a".into(), app.clone()], dir: String::new() }, app));
    }
    if let Some(p) = first_existing(vec![
        PathBuf::from("/usr/bin/minecraft-launcher"),
        PathBuf::from("/opt/minecraft-launcher/minecraft-launcher"),
        PathBuf::from("/usr/local/bin/minecraft-launcher"),
    ]) {
        let s = p.to_string_lossy().to_string();
        return Some((LaunchTarget::Exe { path: s.clone(), args: Vec::new(), dir: String::new() }, s));
    }
    if flatpak_installed(MC_FLATPAK) {
        let fp = sys::unix_tool("flatpak").to_string_lossy().to_string();
        return Some((LaunchTarget::Exe { path: fp, args: vec!["run".into(), MC_FLATPAK.into()], dir: String::new() }, format!("Flatpak: {MC_FLATPAK}")));
    }
    None
}

/// `.minecraft`-Ordner des offiziellen Launchers
pub fn official_game_dir() -> Option<PathBuf> {
    let h = sys::home_dir();
    let candidates = if cfg!(windows) {
        sys::env_path("APPDATA").map(|a| vec![a.join(".minecraft")]).unwrap_or_default()
    } else if cfg!(target_os = "macos") {
        vec![h.join("Library/Application Support/minecraft")]
    } else {
        vec![h.join(".minecraft"), h.join(".var/app").join(MC_FLATPAK).join(".minecraft")]
    };
    candidates.into_iter().find(|p| p.is_dir())
}

// ----------------------------------------------------------------------
// Bibliothek
// ----------------------------------------------------------------------

fn mc_game(id: String, name: String, launch: LaunchTarget, dir: Option<String>) -> Game {
    Game {
        id,
        name,
        source: Source::Minecraft,
        installed: true,
        install_dir: dir,
        exe: None,
        launch,
        art: Art::default(),
        size_bytes: None,
        playtime_min: None,
        steam_appid: None,
    }
}

/// Loader schön schreiben ("fabric" → "Fabric")
pub fn loader_label(l: &str) -> String {
    match l {
        "neoforge" => "NeoForge".into(),
        "vanilla" | "" => "Vanilla".into(),
        other => {
            let mut c = other.chars();
            c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
        }
    }
}

/// Minecraft-Kacheln: offizieller Launcher + ein Eintrag pro NoRisk-Profil
pub fn scan() -> (Vec<Game>, StoreStatus) {
    let mut games = Vec::new();
    let mut status = StoreStatus::default();
    if let Some((launch, path)) = official_launch() {
        games.push(mc_game("minecraft:launcher".into(), "Minecraft".into(), launch, official_game_dir().map(|d| d.to_string_lossy().to_string())));
        status.path = Some(path);
    }
    if let Some(starter) = norisk_starter() {
        for p in norisk_profiles() {
            let mut args = starter.prefix_args.clone();
            args.extend(["launch".to_string(), "--profile".to_string(), p.id.clone()]);
            let detail = [p.game_version.clone(), loader_label(&p.loader)].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" ");
            let name = if detail.is_empty() { format!("NoRisk · {}", p.name) } else { format!("NoRisk · {} ({detail})", p.name) };
            let dir = (!p.dir.as_os_str().is_empty()).then(|| p.dir.to_string_lossy().to_string());
            games.push(mc_game(
                format!("norisk:{}", p.id),
                name,
                LaunchTarget::Exe { path: starter.program.clone(), args, dir: String::new() },
                dir,
            ));
        }
        if status.path.is_none() {
            status.path = Some(starter.program.clone());
        }
    }
    status.found = !games.is_empty();
    status.installed = games.len();
    status.owned = games.len();
    (games, status)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles() {
        let c = rusqlite::Connection::open_in_memory().unwrap();
        c.execute_batch(
            "CREATE TABLE profiles(id TEXT, name TEXT, path TEXT, game_version TEXT, loader TEXT, last_played INTEGER);
             INSERT INTO profiles VALUES ('0b5c1a8e-4f1e-4c55-9d6b-2b9f0c3a1e22','Survival','survival','1.21.4','fabric',5),
               ('kaputt; rm -rf','Böse','x','1.0','vanilla',NULL),
               ('1b5c1a8e-4f1e-4c55-9d6b-2b9f0c3a1e22','Ausbruch','../../etc','1.20.1','forge',NULL);",
        )
        .unwrap();
        let list = read_profiles(&c, Path::new("/p"));
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "Survival");
        assert_eq!(list[0].dir, PathBuf::from("/p/survival"));
        assert_eq!(list[1].dir, PathBuf::new());
        assert!(!valid_profile_id("--profile"));
        assert_eq!(loader_label("neoforge"), "NeoForge");
        assert_eq!(loader_label("fabric"), "Fabric");
    }
}
