//! Import aus Playnite: Tags, Kategorien (→ Sammlungen), Spielzeit, Status, Favoriten,
//! versteckte Spiele und eigene Spiele.
//!
//! Playnite speichert seine Bibliothek als einzelne JSON-Dateien:
//! `library/games/<id>.json`, `library/tags/<id>.json`, `library/categories/<id>.json`,
//! `library/completionstatuses/<id>.json`. Wir lesen nur – an Playnite wird nichts geändert.

use crate::model::{normalize_name, status, Collection, CustomGame, Game, Settings};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const MAX_FILES: usize = 50_000;
const MAX_FILE_SIZE: u64 = 2 * 1024 * 1024;

const STEAM_PLUGIN: &str = "cb91dfc9-b977-43bf-8e70-55f46e410fab";
const EPIC_PLUGIN: &str = "00000002-dbd1-46c6-b5d0-b1ba559d10e4";
const GOG_PLUGIN: &str = "aebe8b7c-6dc3-4a66-af31-e7375c6b5e9e";
const NO_PLUGIN: &str = "00000000-0000-0000-0000-000000000000";

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct PnAction {
    #[serde(rename = "Type")]
    pub kind: serde_json::Value,
    pub path: Option<String>,
    pub arguments: Option<String>,
    pub working_dir: Option<String>,
    pub is_play_action: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct PnGame {
    pub id: String,
    pub name: String,
    pub game_id: Option<String>,
    pub plugin_id: Option<String>,
    /// Sekunden
    pub playtime: u64,
    pub play_count: u64,
    pub last_activity: Option<String>,
    pub tag_ids: Option<Vec<String>>,
    pub category_ids: Option<Vec<String>>,
    pub completion_status_id: Option<String>,
    pub favorite: bool,
    pub hidden: bool,
    pub install_directory: Option<String>,
    pub game_actions: Option<Vec<PnAction>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
struct Named {
    id: String,
    name: String,
}

#[derive(Debug, Clone, Default)]
pub struct Library {
    pub games: Vec<PnGame>,
    pub tags: BTreeMap<String, String>,
    pub categories: BTreeMap<String, String>,
    pub statuses: BTreeMap<String, String>,
}

/// Wo Playnite seine Bibliothek ablegt (installierte Version)
pub fn default_dir() -> Option<PathBuf> {
    let d = crate::sys::env_path("APPDATA")?.join("Playnite");
    d.join("library").is_dir().then_some(d)
}

/// Ordner normalisieren: Playnite-Ordner oder direkt der „library“-Ordner
pub fn library_dir(dir: &Path) -> Option<PathBuf> {
    if dir.join("games").is_dir() {
        return Some(dir.to_path_buf());
    }
    let l = dir.join("library");
    l.join("games").is_dir().then_some(l)
}

fn read_json_files<T: for<'de> Deserialize<'de>>(dir: &Path) -> Vec<T> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).into_iter().flatten().flatten().take(MAX_FILES) {
        let p = e.path();
        if p.extension().map(|x| x != "json").unwrap_or(true) {
            continue;
        }
        if std::fs::metadata(&p).map(|m| m.len() > MAX_FILE_SIZE).unwrap_or(true) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&p) else { continue };
        // Playnite schreibt manchmal ein BOM an den Anfang
        if let Ok(v) = serde_json::from_str::<T>(text.trim_start_matches('\u{feff}')) {
            out.push(v);
        }
    }
    out
}

pub fn load(dir: &Path) -> Result<Library, String> {
    let lib = library_dir(dir).ok_or_else(|| crate::i18n::t("playnite.err_dir"))?;
    let names = |sub: &str| -> BTreeMap<String, String> {
        read_json_files::<Named>(&lib.join(sub)).into_iter().filter(|n| !n.id.is_empty() && !n.name.trim().is_empty()).map(|n| (n.id, n.name.trim().to_string())).collect()
    };
    let games: Vec<PnGame> = read_json_files::<PnGame>(&lib.join("games")).into_iter().filter(|g| !g.name.trim().is_empty()).collect();
    if games.is_empty() {
        return Err(crate::i18n::t("playnite.err_empty"));
    }
    Ok(Library { games, tags: names("tags"), categories: names("categories"), statuses: names("completionstatuses") })
}

/// Playnite-Status (Name) → unser Status
pub fn map_status(name: &str) -> Option<&'static str> {
    let n = name.trim().to_lowercase();
    let n = n.as_str();
    if ["not played", "plan to play", "nicht gespielt", "geplant", "nicht begonnen"].contains(&n) {
        Some(status::NOT_STARTED)
    } else if ["playing", "played", "on hold", "spiele ich", "gespielt", "pausiert"].contains(&n) {
        Some(status::PLAYING)
    } else if ["beaten", "completed", "durchgespielt", "abgeschlossen", "geschafft"].contains(&n) {
        Some(status::FINISHED)
    } else if ["abandoned", "abgebrochen", "aufgegeben"].contains(&n) {
        Some(status::DROPPED)
    } else {
        None
    }
}

/// Eigene ID passend zur Playnite-Herkunft (Steam/Epic/GOG)
fn store_id(g: &PnGame) -> Option<String> {
    let plugin = g.plugin_id.as_deref()?.to_lowercase();
    let gid = g.game_id.as_deref()?.trim();
    if gid.is_empty() {
        return None;
    }
    match plugin.as_str() {
        STEAM_PLUGIN => Some(format!("steam:{gid}")),
        EPIC_PLUGIN => Some(format!("epic:{gid}")),
        GOG_PLUGIN => Some(format!("gog:{gid}")),
        _ => None,
    }
}

/// Startbare Datei eines eigenen Playnite-Spiels
fn custom_from(g: &PnGame) -> Option<CustomGame> {
    if g.plugin_id.as_deref().map(|p| p != NO_PLUGIN).unwrap_or(false) {
        return None;
    }
    let actions = g.game_actions.as_ref()?;
    let a = actions.iter().find(|a| a.is_play_action && a.kind.as_u64().unwrap_or(0) == 0)?;
    let dir = g.install_directory.clone().unwrap_or_default();
    let fill = |s: &str| s.replace("{InstallDir}", &dir).replace("{InstallDirectory}", &dir);
    let exe = fill(a.path.as_deref()?);
    // Platzhalter, die wir nicht kennen (z. B. {PlayniteDir}), lieber nicht raten
    if exe.trim().is_empty() || exe.contains('{') {
        return None;
    }
    let windows_abs = exe.as_bytes().get(1) == Some(&b':') || exe.starts_with("\\\\");
    let exe_path = if windows_abs || Path::new(&exe).is_absolute() { PathBuf::from(&exe) } else { Path::new(&dir).join(&exe) };
    Some(CustomGame {
        id: String::new(),
        name: g.name.trim().to_string(),
        exe: exe_path.to_string_lossy().to_string(),
        args: a.arguments.clone().map(|x| fill(&x)).unwrap_or_default(),
        workdir: a.working_dir.clone().map(|x| fill(&x)).unwrap_or_default(),
        ..Default::default()
    })
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Report {
    pub matched: usize,
    pub tags: usize,
    pub collections: usize,
    pub playtime: usize,
    pub status: usize,
    pub new_custom: usize,
    pub not_found: usize,
}

/// Import in die Einstellungen übernehmen. Vorhandenes bleibt, es wird nur ergänzt
/// (Spielzeit: der größere Wert gewinnt).
pub fn apply(lib: &Library, games: &[Game], s: &mut Settings, now: u64) -> Report {
    let mut r = Report::default();
    let by_name: BTreeMap<String, String> = games.iter().map(|g| (normalize_name(&g.name), g.id.clone())).collect();
    let ids: std::collections::BTreeSet<&str> = games.iter().map(|g| g.id.as_str()).collect();
    for pg in &lib.games {
        let target = store_id(pg)
            .filter(|id| ids.contains(id.as_str()))
            .or_else(|| by_name.get(&normalize_name(&pg.name)).cloned())
            .or_else(|| s.custom_games.iter().find(|c| normalize_name(&c.name) == normalize_name(&pg.name)).map(|c| c.id.clone()));
        let id = match target {
            Some(id) => id,
            None => match custom_from(pg) {
                // Eigenes Spiel aus Playnite, das es hier noch nicht gibt
                Some(mut c) => {
                    if s.custom_games.iter().any(|x| x.exe.eq_ignore_ascii_case(&c.exe) || normalize_name(&x.name) == normalize_name(&c.name)) {
                        r.not_found += 1;
                        continue;
                    }
                    c.id = format!("custom:pn-{}", pg.id.chars().filter(|ch| ch.is_ascii_alphanumeric()).take(36).collect::<String>());
                    c.added = now;
                    let id = c.id.clone();
                    s.custom_games.push(c);
                    r.new_custom += 1;
                    id
                }
                None => {
                    r.not_found += 1;
                    continue;
                }
            },
        };
        r.matched += 1;
        // Tags
        let tags: Vec<String> = pg.tag_ids.iter().flatten().filter_map(|t| lib.tags.get(t).cloned()).collect();
        if !tags.is_empty() {
            let entry = s.tags.entry(id.clone()).or_default();
            for t in tags {
                if !entry.iter().any(|x| x.eq_ignore_ascii_case(&t)) {
                    entry.push(t);
                    r.tags += 1;
                }
            }
        }
        // Kategorien → Sammlungen
        for cid in pg.category_ids.iter().flatten() {
            let Some(cname) = lib.categories.get(cid) else { continue };
            let i = match s.collections.iter().position(|c| c.name.eq_ignore_ascii_case(cname)) {
                Some(i) => i,
                None => {
                    s.collections.push(Collection { id: format!("pn-{}", cid.chars().filter(|c| c.is_ascii_alphanumeric()).take(36).collect::<String>()), name: cname.clone(), games: Vec::new() });
                    s.collections.len() - 1
                }
            };
            if !s.collections[i].games.contains(&id) {
                s.collections[i].games.push(id.clone());
                r.collections += 1;
            }
        }
        // Spielzeit (Sekunden → Minuten), der größere Wert gewinnt
        let minutes = pg.playtime / 60;
        if minutes > 0 {
            let e = s.playtime_local.entry(id.clone()).or_insert(0);
            if minutes > *e {
                *e = minutes;
                r.playtime += 1;
            }
        }
        if pg.play_count > 0 {
            let e = s.launch_count.entry(id.clone()).or_insert(0);
            *e = (*e).max(pg.play_count.min(u32::MAX as u64) as u32);
        }
        if let Some(ts) = pg.last_activity.as_deref().and_then(crate::util::parse_datetime) {
            let e = s.last_played.entry(id.clone()).or_insert(0);
            *e = (*e).max(ts);
        }
        // Status nur setzen, wenn hier noch keiner gesetzt ist
        if let Some(st) = pg.completion_status_id.as_ref().and_then(|c| lib.statuses.get(c)).and_then(|n| map_status(n)) {
            if s.status_of(&id).is_empty() {
                s.status.insert(id.clone(), st.to_string());
                r.status += 1;
            }
        }
        if pg.favorite && !s.is_favorite(&id) {
            s.favorites.push(id.clone());
        }
        if pg.hidden && !s.is_hidden(&id) {
            s.hidden.push(id.clone());
        }
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Art, LaunchTarget, Source};

    fn game(id: &str, name: &str) -> Game {
        Game {
            id: id.into(),
            name: name.into(),
            source: Source::Steam,
            installed: true,
            install_dir: None,
            exe: None,
            launch: LaunchTarget::Custom,
            art: Art::default(),
            size_bytes: None,
            playtime_min: None,
            steam_appid: None,
        }
    }

    #[test]
    fn import_folder() {
        let dir = std::env::temp_dir().join(format!("og-playnite-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let lib = dir.join("library");
        for sub in ["games", "tags", "categories", "completionstatuses"] {
            std::fs::create_dir_all(lib.join(sub)).unwrap();
        }
        let w = |p: &str, t: &str| std::fs::write(lib.join(p), t).unwrap();
        w("tags/t1.json", r#"{"Id":"t1","Name":"Koop"}"#);
        w("categories/c1.json", r#"{"Id":"c1","Name":"Mit Freunden"}"#);
        w("completionstatuses/s1.json", r#"{"Id":"s1","Name":"Beaten"}"#);
        w("games/g1.json", "\u{feff}{\"Id\":\"g1\",\"Name\":\"Portal 2\",\"PluginId\":\"cb91dfc9-b977-43bf-8e70-55f46e410fab\",\"GameId\":\"620\",\"Playtime\":7200,\"PlayCount\":3,\"TagIds\":[\"t1\"],\"CategoryIds\":[\"c1\"],\"CompletionStatusId\":\"s1\",\"Favorite\":true,\"LastActivity\":\"2024-05-01T10:00:00+02:00\"}");
        w("games/g2.json", r#"{"Id":"g2","Name":"The Witcher® 3","Playtime":60}"#);
        w("games/g3.json", r#"{"Id":"g3","Name":"Mein Spiel","PluginId":"00000000-0000-0000-0000-000000000000","InstallDirectory":"C:\\Spiele\\Mein","GameActions":[{"Type":0,"Path":"{InstallDir}\\start.exe","Arguments":"-x","IsPlayAction":true}]}"#);
        w("games/g4.json", r#"{"Id":"g4","Name":"Unbekannt"}"#);
        w("games/kaputt.json", "kein json");

        let pl = load(&dir).unwrap();
        assert_eq!(pl.games.len(), 4);
        let games = vec![game("steam:620", "Portal 2"), game("gog:1", "The Witcher 3")];
        let mut s = Settings::default();
        let r = apply(&pl, &games, &mut s, 1000);
        assert_eq!(r.matched, 3);
        assert_eq!(r.new_custom, 1);
        assert_eq!(r.not_found, 1);
        assert_eq!(s.tags_of("steam:620"), vec!["Koop".to_string()]);
        assert_eq!(s.collections[0].name, "Mit Freunden");
        assert_eq!(s.playtime_local.get("steam:620"), Some(&120));
        assert_eq!(s.status_of("steam:620"), status::FINISHED);
        assert!(s.is_favorite("steam:620"));
        assert!(s.last_played("steam:620") > 0);
        assert_eq!(s.playtime_local.get("gog:1"), Some(&1), "Name-Abgleich ohne ®");
        let c = &s.custom_games[0];
        assert_eq!(c.exe, "C:\\Spiele\\Mein\\start.exe");
        assert_eq!(c.args, "-x");
        // Zweimal importieren legt nichts doppelt an
        let r2 = apply(&pl, &games, &mut s, 1000);
        assert_eq!(r2.new_custom, 0);
        assert_eq!(r2.not_found, 1);
        assert_eq!(s.collections.len(), 1);
        assert_eq!(s.tags_of("steam:620").len(), 1);
        assert!(load(&std::env::temp_dir().join("og-gibt-es-nicht")).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn status_names() {
        assert_eq!(map_status("Completed"), Some(status::FINISHED));
        assert_eq!(map_status("Plan to Play"), Some(status::NOT_STARTED));
        assert_eq!(map_status("Abandoned"), Some(status::DROPPED));
        assert_eq!(map_status("irgendwas"), None);
    }
}
