//! Steam-Anbindung: findet Steam, liest alle Bibliotheksordner und installierten
//! Spiele aus und holt (optional) die komplette Spieleliste über die Steam-Web-API.

use crate::model::{normalize_name, Art, Game, LaunchTarget, Source, StoreStatus};
use crate::problem::Problem;
use crate::sys;
use crate::vdf;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const CDN: &str = "https://cdn.cloudflare.steamstatic.com/steam/apps";

/// AppIDs, die keine Spiele sind (Redistributables, Proton, Laufzeiten …)
const NOT_GAMES: &[u32] = &[228980, 1070560, 1391110, 1628350, 1493710, 1826330, 1161040, 1580130, 961940, 1887720, 2180100, 250820, 365670, 223850];

pub fn find_steam_root() -> Option<PathBuf> {
    let p = steam_roots().into_iter().find(|p| p.join("steamapps").exists())?;
    // Linux: ~/.steam/steam ist meist nur ein Verweis → echten Pfad nehmen (keine doppelten Ordner)
    if cfg!(windows) {
        Some(p)
    } else {
        Some(p.canonicalize().unwrap_or(p))
    }
}

/// Mögliche Steam-Ordner je nach System
fn steam_roots() -> Vec<PathBuf> {
    let mut out = Vec::new();
    #[cfg(windows)]
    {
        for c in [
            sys::reg_str(sys::Hive::CurrentUser, "Software\\Valve\\Steam", "SteamPath"),
            sys::reg_str(sys::Hive::LocalMachine, "SOFTWARE\\WOW6432Node\\Valve\\Steam", "InstallPath"),
            sys::reg_str(sys::Hive::LocalMachine, "SOFTWARE\\Valve\\Steam", "InstallPath"),
        ]
        .into_iter()
        .flatten()
        {
            out.push(PathBuf::from(sys::native_path(&c)));
        }
        for base in ["ProgramFiles(x86)", "ProgramFiles"] {
            if let Some(b) = sys::env_path(base) {
                out.push(b.join("Steam"));
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        out.push(sys::home_dir().join("Library").join("Application Support").join("Steam"));
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let h = sys::home_dir();
        // normales Steam, Steam aus Flatpak und Snap
        out.push(h.join(".steam").join("steam"));
        out.push(h.join(".local").join("share").join("Steam"));
        out.push(h.join(".steam").join("root"));
        out.push(h.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"));
        out.push(h.join("snap/steam/common/.local/share/Steam"));
    }
    out
}

/// Alle Steam-Bibliotheksordner (C:\…\Steam, D:\SteamLibrary, …)
pub fn library_folders(root: &Path) -> Vec<PathBuf> {
    let mut out = vec![root.to_path_buf()];
    for file in [root.join("steamapps").join("libraryfolders.vdf"), root.join("config").join("libraryfolders.vdf")] {
        let Ok(text) = fs::read_to_string(&file) else { continue };
        let v = vdf::parse(&text);
        let Some(lf) = v.get("libraryfolders") else { continue };
        for (key, entry) in lf.entries() {
            let path = match entry {
                vdf::Vdf::Obj(_) => entry.str("path").map(|s| s.to_string()),
                vdf::Vdf::Str(s) if key.chars().all(|c| c.is_ascii_digit()) => Some(s.clone()),
                _ => None,
            };
            if let Some(p) = path {
                let mut p = PathBuf::from(sys::native_path(&p));
                if !cfg!(windows) {
                    p = p.canonicalize().unwrap_or(p);
                }
                if !out.iter().any(|o| same_path(o, &p)) {
                    out.push(p);
                }
            }
        }
    }
    out
}

fn same_path(a: &Path, b: &Path) -> bool {
    a.to_string_lossy()
        .trim_end_matches(['\\', '/'])
        .eq_ignore_ascii_case(b.to_string_lossy().trim_end_matches(['\\', '/']))
}

/// Zuletzt angemeldetes Steam-Konto: (SteamID64, Anzeigename)
pub fn logged_in_user(root: &Path) -> Option<(String, String)> {
    let text = fs::read_to_string(root.join("config").join("loginusers.vdf")).ok()?;
    let v = vdf::parse(&text);
    let users = v.get("users")?;
    let mut first = None;
    for (id, u) in users.entries() {
        let name = u
            .str("PersonaName")
            .or_else(|| u.str("AccountName"))
            .unwrap_or("Steam")
            .to_string();
        if u.str("MostRecent") == Some("1") || u.str("mostrecent") == Some("1") {
            return Some((id.clone(), name));
        }
        if first.is_none() {
            first = Some((id.clone(), name));
        }
    }
    first
}

/// Bilder aus Steams lokalem Cache + CDN als Ersatz
pub fn art_for(root: Option<&Path>, appid: u32) -> Art {
    let mut art = Art::default();
    if let Some(root) = root {
        let cache = root.join("appcache").join("librarycache");
        let folder = cache.join(appid.to_string());
        // Neues Format (seit 2024): librarycache/<appid>/library_600x900.jpg …
        let pick = |names: &[&str]| -> Option<String> {
            for n in names {
                let p = folder.join(n);
                if p.exists() {
                    return Some(sys::file_url(&p));
                }
                let old = cache.join(format!("{appid}_{n}"));
                if old.exists() {
                    return Some(sys::file_url(&old));
                }
            }
            None
        };
        if let Some(u) = pick(&["library_600x900_2x.jpg", "library_600x900.jpg", "library_capsule.jpg"]) {
            art.cover.push(u);
        }
        if let Some(u) = pick(&["library_hero.jpg", "library_hero_2x.jpg"]) {
            art.hero.push(u);
        }
        if let Some(u) = pick(&["logo.png", "logo_2x.png"]) {
            art.logo.push(u);
        }
        if let Some(u) = pick(&["header.jpg", "header_2x.jpg"]) {
            art.header.push(u);
        }
        // Icon: alt "<appid>_icon.jpg", neu "<appid>/<sha1>.jpg"
        let old_icon = cache.join(format!("{appid}_icon.jpg"));
        if old_icon.exists() {
            art.icon.push(sys::file_url(&old_icon));
        } else if let Ok(rd) = fs::read_dir(&folder) {
            for e in rd.flatten() {
                let p = e.path();
                let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                if stem.len() == 40 && stem.chars().all(|c| c.is_ascii_hexdigit()) {
                    art.icon.push(sys::file_url(&p));
                    break;
                }
            }
        }
    }
    art.cover.push(format!("{CDN}/{appid}/library_600x900_2x.jpg"));
    art.cover.push(format!("{CDN}/{appid}/library_600x900.jpg"));
    art.hero.push(format!("{CDN}/{appid}/library_hero.jpg"));
    art.logo.push(format!("{CDN}/{appid}/logo.png"));
    art.header.push(format!("{CDN}/{appid}/header.jpg"));
    art
}

/// Fallback-Art (nur CDN), z. B. für Epic-/eigene Spiele, die es auch auf Steam gibt
pub fn cdn_art(appid: u32) -> Art {
    art_for(None, appid)
}

pub struct SteamScan {
    pub games: Vec<Game>,
    pub status: StoreStatus,
    pub warning: Option<Problem>,
}

pub fn scan(api_key: &str, steam_id_setting: &str, show_uninstalled: bool) -> SteamScan {
    let mut status = StoreStatus::default();
    let Some(root) = find_steam_root() else {
        status.note = Some(crate::i18n::t("store.steam_not_found"));
        return SteamScan { games: Vec::new(), status, warning: None };
    };
    status.found = true;
    status.path = Some(root.to_string_lossy().to_string());

    let user = logged_in_user(&root);
    if let Some((_, name)) = &user {
        status.account = Some(name.clone());
    }

    let mut games: Vec<Game> = Vec::new();
    for lib in library_folders(&root) {
        let apps = lib.join("steamapps");
        let Ok(rd) = fs::read_dir(&apps) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            let fname = p.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
            if !(fname.starts_with("appmanifest_") && fname.ends_with(".acf")) {
                continue;
            }
            let Ok(text) = fs::read_to_string(&p) else { continue };
            let v = vdf::parse(&text);
            let Some(st) = v.get("AppState") else { continue };
            let Some(appid) = st.str("appid").and_then(|s| s.parse::<u32>().ok()) else { continue };
            if NOT_GAMES.contains(&appid) || games.iter().any(|g| g.steam_appid == Some(appid)) {
                continue;
            }
            let name = st.str("name").unwrap_or("?").to_string();
            let lower = name.to_lowercase();
            if lower.contains("redistributable")
                || lower.starts_with("proton")
                || lower.starts_with("steam linux runtime")
                || lower.contains("steamworks common")
                || lower.contains("dedicated server")
                || lower == "steamvr"
            {
                continue;
            }
            let install_dir = st
                .str("installdir")
                .map(|d| apps.join("common").join(d).to_string_lossy().to_string());
            let size = st.str("SizeOnDisk").and_then(|s| s.parse::<u64>().ok());
            games.push(Game {
                id: format!("steam:{appid}"),
                name,
                source: Source::Steam,
                installed: true,
                install_dir,
                exe: None,
                launch: LaunchTarget::SteamRun(appid),
                art: art_for(Some(&root), appid),
                size_bytes: size.filter(|s| *s > 0),
                playtime_min: None,
                steam_appid: Some(appid),
            });
        }
    }
    status.installed = games.len();

    // Optional: alle gekauften Spiele (auch nicht installierte) über die Web-API
    let mut warning = None;
    let key = api_key.trim();
    let sid = if !steam_id_setting.trim().is_empty() {
        Some(steam_id_setting.trim().to_string())
    } else {
        user.as_ref().map(|(id, _)| id.clone())
    };
    if !key.is_empty() {
        if let Some(sid) = sid {
            match owned_games(key, &sid) {
                Ok(owned) => {
                    status.owned = owned.len();
                    let by_id: HashMap<u32, usize> = games
                        .iter()
                        .enumerate()
                        .filter_map(|(i, g)| g.steam_appid.map(|a| (a, i)))
                        .collect();
                    for o in owned {
                        if let Some(&i) = by_id.get(&o.appid) {
                            games[i].playtime_min = Some(o.playtime);
                            if let Some(icon) = &o.icon_url {
                                games[i].art.icon.push(icon.clone());
                            }
                        } else if show_uninstalled && !NOT_GAMES.contains(&o.appid) {
                            let mut art = art_for(Some(&root), o.appid);
                            if let Some(icon) = &o.icon_url {
                                art.icon.push(icon.clone());
                            }
                            games.push(Game {
                                id: format!("steam:{}", o.appid),
                                name: o.name,
                                source: Source::Steam,
                                installed: false,
                                install_dir: None,
                                exe: None,
                                launch: LaunchTarget::SteamInstall(o.appid),
                                art,
                                size_bytes: None,
                                playtime_min: Some(o.playtime),
                                steam_appid: Some(o.appid),
                            });
                        }
                    }
                }
                Err(p) => warning = Some(p),
            }
        }
    }

    // Immer zusätzlich: Bibliothek aus Steams eigenen Dateien auf dem PC. Die Web-API kennt nur
    // selbst gekaufte Spiele – Spiele aus der Steam-Familie (Familienbibliothek) stehen nur
    // lokal (Bild-Cache der Bibliothek, gespielte Spiele). Ohne Schlüssel ist das die einzige Quelle.
    {
        let api_owned = status.owned;
        let local = local_library(&root, user.as_ref().map(|(id, _)| id.as_str()));
        let by_id: HashMap<u32, usize> = games
            .iter()
            .enumerate()
            .filter_map(|(i, g)| g.steam_appid.map(|a| (a, i)))
            .collect();
        let mut owned = if api_owned > 0 { api_owned } else { games.len() };
        for (appid, name, playtime) in local {
            if let Some(&i) = by_id.get(&appid) {
                if games[i].playtime_min.is_none() && playtime > 0 {
                    games[i].playtime_min = Some(playtime);
                }
            } else if show_uninstalled && !NOT_GAMES.contains(&appid) {
                owned += 1;
                games.push(Game {
                    id: format!("steam:{appid}"),
                    name,
                    source: Source::Steam,
                    installed: false,
                    install_dir: None,
                    exe: None,
                    launch: LaunchTarget::SteamInstall(appid),
                    art: art_for(Some(&root), appid),
                    size_bytes: None,
                    playtime_min: (playtime > 0).then_some(playtime),
                    steam_appid: Some(appid),
                });
            }
        }
        status.owned = owned;
    }

    SteamScan { games, status, warning }
}

/// Spiele der Steam-Bibliothek aus lokalen Dateien: (AppID, Name, Spielzeit in Minuten).
/// Quellen: Bild-Cache der Bibliothek + localconfig.vdf, Namen aus appinfo.vdf.
fn local_library(root: &Path, steam_id64: Option<&str>) -> Vec<(u32, String, u64)> {
    let mut ids: HashMap<u32, u64> = HashMap::new();

    // 1) Steam legt für jedes Spiel der Bibliothek Bilder im librarycache ab
    if let Ok(rd) = fs::read_dir(root.join("appcache").join("librarycache")) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            let id_part = name.split(['_', '.']).next().unwrap_or("");
            if let Ok(id) = id_part.parse::<u32>() {
                ids.entry(id).or_insert(0);
            }
        }
    }

    // 2) Alle jemals gestarteten Spiele + Spielzeit
    if let Some(account) = steam_id64.and_then(|s| s.parse::<u64>().ok()).map(|v| v & 0xFFFF_FFFF) {
        let cfg = root.join("userdata").join(account.to_string()).join("config").join("localconfig.vdf");
        if let Ok(text) = fs::read_to_string(cfg) {
            let v = vdf::parse(&text);
            let apps = v
                .get("UserLocalConfigStore")
                .and_then(|x| x.get("Software"))
                .and_then(|x| x.get("Valve"))
                .and_then(|x| x.get("Steam"))
                .and_then(|x| x.get("apps"));
            if let Some(apps) = apps {
                for (k, app) in apps.entries() {
                    if let Ok(id) = k.parse::<u32>() {
                        let pt = app.str("Playtime").and_then(|p| p.parse::<u64>().ok()).unwrap_or(0);
                        let e = ids.entry(id).or_insert(0);
                        *e = (*e).max(pt);
                    }
                }
            }
        }
    }
    if ids.is_empty() {
        return Vec::new();
    }

    // 3) Namen und Typ aus appinfo.vdf – nur echte Spiele behalten
    let Ok(data) = fs::read(root.join("appcache").join("appinfo.vdf")) else {
        return Vec::new();
    };
    let meta = crate::appinfo::parse(&data);
    let mut out: Vec<(u32, String, u64)> = ids
        .into_iter()
        .filter_map(|(id, pt)| {
            let m = meta.get(&id)?;
            (m.kind == "game").then(|| (id, m.name.clone(), pt))
        })
        .collect();
    out.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));
    out
}

pub struct OwnedGame {
    pub appid: u32,
    pub name: String,
    pub playtime: u64,
    pub icon_url: Option<String>,
}

pub fn http_agent() -> ureq::Agent {
    use ureq::tls::{RootCerts, TlsConfig, TlsProvider};
    let tls = TlsConfig::builder()
        .provider(TlsProvider::NativeTls)
        .root_certs(RootCerts::PlatformVerifier)
        .build();
    let config = ureq::Agent::config_builder()
        .https_only(true)
        .tls_config(tls)
        .timeout_global(Some(std::time::Duration::from_secs(12)))
        .user_agent("Open-Game-Launcher")
        .build();
    config.into()
}

fn owned_games(key: &str, steam_id: &str) -> Result<Vec<OwnedGame>, Problem> {
    let url = format!(
        "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/?key={}&steamid={}&include_appinfo=1&include_played_free_games=1&format=json",
        sys::url_encode(key),
        sys::url_encode(steam_id)
    );
    let agent = http_agent();
    let value: serde_json::Value = match agent.get(&url).call() {
        Ok(mut resp) => resp
            .body_mut()
            .read_json()
            .map_err(|e| Problem::steam_api(None, &e.to_string()))?,
        Err(ureq::Error::StatusCode(code)) => return Err(Problem::steam_api(Some(code), "access denied")),
        Err(e) => return Err(Problem::steam_api(None, &e.to_string())),
    };
    let list = value
        .get("response")
        .and_then(|r| r.get("games"))
        .and_then(|g| g.as_array())
        .cloned()
        .unwrap_or_default();
    if list.is_empty() && value.get("response").and_then(|r| r.get("game_count")).is_none() {
        return Err(Problem::steam_api(
            None,
            "Empty response - profile or game details are probably set to private.",
        ));
    }
    Ok(list
        .iter()
        .filter_map(|g| {
            let appid = g.get("appid")?.as_u64()? as u32;
            let name = g.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
            if name.is_empty() {
                return None;
            }
            let playtime = g.get("playtime_forever").and_then(|p| p.as_u64()).unwrap_or(0);
            let icon_url = g
                .get("img_icon_url")
                .and_then(|i| i.as_str())
                .filter(|s| !s.is_empty())
                .map(|h| format!("https://media.steampowered.com/steamcommunity/public/images/apps/{appid}/{h}.jpg"));
            Some(OwnedGame { appid, name, playtime, icon_url })
        })
        .collect())
}

/// Sucht ein Spiel im Steam-Store (für schöne Cover bei Epic- und eigenen Spielen).
/// Gibt nur bei eindeutigem Namen eine AppID zurück, sonst 0.
pub fn store_lookup(agent: &ureq::Agent, name: &str) -> Option<u32> {
    let url = format!(
        "https://store.steampowered.com/api/storesearch/?term={}&l=german&cc=DE",
        sys::url_encode(name)
    );
    let value: serde_json::Value = agent.get(&url).call().ok()?.body_mut().read_json().ok()?;
    let wanted = normalize_name(name);
    if wanted.len() < 3 {
        return Some(0);
    }
    let items = value.get("items")?.as_array()?;
    for it in items {
        let n = it.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let id = it.get("id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let norm = normalize_name(n);
        if id != 0 && (norm == wanted || norm.trim_end_matches("goty") == wanted) {
            return Some(id);
        }
    }
    Some(0)
}
