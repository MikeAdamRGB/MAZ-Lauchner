//! Weitere Stores: GOG, EA app, Ubisoft Connect, Battle.net, Amazon Games, itch.io, Xbox/Microsoft Store.
//!
//! Gemeinsame Schnittstelle: [`StoreLibrary`] – jeder Store findet seine *installierten* Spiele
//! lokal (Registry, Manifeste, Konfigurationsdateien) und sagt, wie sie gestartet werden.
//! Gestartet wird immer über den offiziellen Client bzw. dessen Link. Es werden keine
//! Passwörter abgefragt und nichts aus inoffiziellen Quellen geladen.

use crate::model::{Art, Game, LaunchTarget, Settings, Source, StoreStatus};
use crate::sys;
use std::path::{Path, PathBuf};

pub struct StoreScan {
    pub games: Vec<Game>,
    pub status: StoreStatus,
}

/// Gemeinsame Schnittstelle aller Store-Bibliotheken
pub trait StoreLibrary {
    fn source(&self) -> Source;
    /// Ist der Client (z. B. GOG Galaxy) installiert?
    fn client_installed(&self) -> bool;
    /// Download-Seite des Clients (für die Fehlererklärung)
    fn download_url(&self) -> &'static str;
    /// Installierte Spiele finden
    fn games(&self) -> Vec<Game>;
    /// Gekaufte, aber nicht installierte Spiele (soweit der Store sie lokal speichert)
    fn owned(&self) -> Vec<Game> {
        Vec::new()
    }
    /// Kann dieser Store auch nicht installierte Spiele zeigen?
    fn has_library(&self) -> bool {
        false
    }
    /// Gibt es diesen Store auf diesem Betriebssystem?
    fn supported(&self) -> bool {
        cfg!(windows)
    }
    /// Link, der das Store-Programm öffnet (z. B. zum Deinstallieren)
    fn client_uri(&self) -> Option<&'static str> {
        None
    }

    fn scan(&self, with_owned: bool) -> StoreScan {
        let mut games = self.games();
        let installed = games.len();
        if with_owned {
            for g in self.owned() {
                let key = crate::model::normalize_name(&g.name);
                if !games.iter().any(|o| o.id == g.id || crate::model::normalize_name(&o.name) == key) {
                    games.push(g);
                }
            }
        }
        let client = self.client_installed();
        let status = StoreStatus {
            found: client || !games.is_empty(),
            installed,
            owned: games.len(),
            note: (!client && games.is_empty()).then(|| crate::i18n::tf("store.client_missing", &[("store", &self.source().label())])),
            ..Default::default()
        };
        StoreScan { games, status }
    }
}

/// Alle zusätzlichen Stores
pub fn all() -> Vec<Box<dyn StoreLibrary + Send>> {
    vec![
        Box::new(Gog),
        Box::new(Ea),
        Box::new(Ubisoft),
        Box::new(BattleNet),
        Box::new(Amazon),
        Box::new(Itch),
        Box::new(Xbox),
    ]
}

pub fn library(src: Source) -> Option<Box<dyn StoreLibrary + Send>> {
    all().into_iter().find(|s| s.source() == src)
}

/// Alle eingeschalteten Stores einlesen
pub fn scan_all(settings: &Settings) -> Vec<(Source, StoreScan)> {
    all()
        .into_iter()
        .filter(|s| s.supported() && settings.store_on(s.source()))
        .map(|s| (s.source(), s.scan(settings.stores_show_uninstalled)))
        .collect()
}

fn game(id: String, name: String, source: Source, dir: Option<String>, launch: LaunchTarget) -> Game {
    Game {
        id,
        name,
        source,
        installed: true,
        install_dir: dir.filter(|d| !d.is_empty()),
        exe: None,
        launch,
        art: Art::default(),
        size_bytes: None,
        playtime_min: None,
        steam_appid: None,
    }
}

/// Nicht installiertes Spiel (Klick = im Store-Programm installieren)
fn owned_game(id: String, name: String, source: Source, launch: LaunchTarget) -> Game {
    let mut g = game(id, name, source, None, launch);
    g.installed = false;
    g
}

/// SQLite-Datenbank eines Stores nur lesend öffnen. "immutable" = keine Sperren, der Store
/// kann gleichzeitig weiterlaufen; wir verändern nie etwas.
pub fn open_db(path: &Path) -> Option<rusqlite::Connection> {
    use rusqlite::OpenFlags;
    if !path.is_file() {
        return None;
    }
    // Pfad als file:-URI (Sonderzeichen kodieren, Windows-Laufwerk mit führendem /)
    let mut p = path.to_string_lossy().replace('\\', "/");
    if !p.starts_with('/') {
        p.insert(0, '/');
    }
    let enc: String = p
        .chars()
        .map(|c| match c {
            '%' => "%25".to_string(),
            '?' => "%3f".to_string(),
            '#' => "%23".to_string(),
            ' ' => "%20".to_string(),
            c => c.to_string(),
        })
        .collect();
    let uri = format!("file://{enc}?immutable=1&mode=ro");
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let conn = rusqlite::Connection::open_with_flags(uri, flags).ok()?;
    // Schutz gegen riesige oder kaputte Dateien
    let _ = conn.busy_timeout(std::time::Duration::from_millis(500));
    // SICHERHEIT: Fremde Datenbank – keine Funktionen aus Views/Triggern ausführen lassen
    conn.execute_batch("PRAGMA trusted_schema = OFF;").ok()?;
    Some(conn)
}

/// Eigene Kopie einer Datenbank öffnen (darf geschrieben werden, damit ein WAL-Journal
/// übernommen wird). Nur für Kopien in unserem eigenen Ordner verwenden!
pub fn open_db_wal(copy: &Path) -> Option<rusqlite::Connection> {
    use rusqlite::OpenFlags;
    let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let conn = rusqlite::Connection::open_with_flags(copy, flags).ok()?;
    conn.execute_batch("PRAGMA trusted_schema = OFF;").ok()?;
    Some(conn)
}

/// Gibt es diese Spalte in der Tabelle?
fn has_column(conn: &rusqlite::Connection, table: &str, col: &str) -> bool {
    conn.prepare(&format!("PRAGMA table_info({table})"))
        .and_then(|mut st| {
            let names: Vec<String> = st.query_map([], |r| r.get::<_, String>(1))?.flatten().collect();
            Ok(names.iter().any(|n| n.eq_ignore_ascii_case(col)))
        })
        .unwrap_or(false)
}

/// GOG Galaxy: alle gekauften GOG-Spiele aus galaxy-2.0.db
pub fn galaxy_owned(conn: &rusqlite::Connection) -> Vec<(String, String, Option<String>)> {
    let sql = "SELECT lr.releaseKey, gp.value FROM LibraryReleases lr \
               JOIN GamePieces gp ON gp.releaseKey = lr.releaseKey \
               JOIN GamePieceTypes t ON t.id = gp.gamePieceTypeId \
               WHERE lr.releaseKey LIKE 'gog\\_%' ESCAPE '\\' AND t.type IN ('originalTitle', 'title', 'originalImages') \
               LIMIT 20000";
    let Ok(mut st) = conn.prepare(sql) else { return Vec::new() };
    let rows: Vec<(String, String)> = st
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map(|it| it.flatten().collect())
        .unwrap_or_default();
    let mut out: Vec<(String, String, Option<String>)> = Vec::new();
    for (key, value) in rows {
        let Some(id) = key.strip_prefix("gog_") else { continue };
        if !id.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let v: serde_json::Value = serde_json::from_str(&value).unwrap_or(serde_json::Value::Null);
        let entry = match out.iter_mut().position(|e| e.0 == id) {
            Some(i) => i,
            None => {
                out.push((id.to_string(), String::new(), None));
                out.len() - 1
            }
        };
        if let Some(t) = v.get("title").and_then(|t| t.as_str()) {
            if out[entry].1.is_empty() {
                out[entry].1 = t.trim().to_string();
            }
        }
        if let Some(c) = v.get("verticalCover").and_then(|c| c.as_str()).filter(|u| u.starts_with("https://")) {
            out[entry].2 = Some(c.to_string());
        }
    }
    out.retain(|e| !e.1.is_empty());
    out
}

/// Amazon Games: GameProductInfo.sqlite (Tabelle DbSet)
pub fn amazon_owned(conn: &rusqlite::Connection) -> Vec<(String, String, Option<String>)> {
    if !has_column(conn, "DbSet", "ProductIdStr") || !has_column(conn, "DbSet", "ProductTitle") {
        return Vec::new();
    }
    let icon = if has_column(conn, "DbSet", "ProductIconUrl") { "ProductIconUrl" } else { "''" };
    let sql = format!("SELECT ProductIdStr, ProductTitle, {icon} FROM DbSet LIMIT 20000");
    let Ok(mut st) = conn.prepare(&sql) else { return Vec::new() };
    st.query_map([], |r| {
        Ok((
            r.get::<_, Option<String>>(0)?.unwrap_or_default(),
            r.get::<_, Option<String>>(1)?.unwrap_or_default(),
            r.get::<_, Option<String>>(2)?.filter(|u| u.starts_with("https://")),
        ))
    })
    .map(|it| it.flatten().filter(|(id, t, _)| !id.is_empty() && !t.trim().is_empty()).collect())
    .unwrap_or_default()
}

/// itch.io: butler.db – Spiele, für die es einen Kauf-/Download-Schlüssel gibt
pub fn itch_owned(conn: &rusqlite::Connection) -> Vec<(String, String, Option<String>, String)> {
    let sql = "SELECT g.id, g.title, COALESCE(g.still_cover_url, g.cover_url, ''), COALESCE(g.url, '') \
               FROM games g WHERE g.id IN (SELECT game_id FROM download_keys) LIMIT 20000";
    let Ok(mut st) = conn.prepare(sql) else { return Vec::new() };
    st.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?.to_string(),
            r.get::<_, Option<String>>(1)?.unwrap_or_default(),
            r.get::<_, Option<String>>(2)?.filter(|u| u.starts_with("https://")),
            r.get::<_, Option<String>>(3)?.unwrap_or_default(),
        ))
    })
    .map(|it| it.flatten().filter(|(_, t, _, _)| !t.trim().is_empty()).collect())
    .unwrap_or_default()
}

// ----------------------------------------------------------------------
// Windows: Einträge unter "Apps & Features"
// ----------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UninstallEntry {
    pub key: String,
    pub display_name: String,
    pub install_location: String,
    pub uninstall_string: String,
    pub display_icon: String,
    pub publisher: String,
}

pub fn uninstall_entries() -> Vec<UninstallEntry> {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        let mut out = Vec::new();
        for (hive, path) in [
            (HKEY_LOCAL_MACHINE, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
            (HKEY_LOCAL_MACHINE, "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
            (HKEY_CURRENT_USER, "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
        ] {
            let Ok(root) = RegKey::predef(hive).open_subkey(path) else { continue };
            for key in root.enum_keys().flatten() {
                let Ok(k) = root.open_subkey(&key) else { continue };
                let get = |n: &str| k.get_value::<String, _>(n).unwrap_or_default().trim().to_string();
                let e = UninstallEntry {
                    display_name: get("DisplayName"),
                    install_location: get("InstallLocation").trim_matches('"').trim_end_matches('\\').to_string(),
                    uninstall_string: get("UninstallString"),
                    display_icon: get("DisplayIcon"),
                    publisher: get("Publisher"),
                    key,
                };
                if !e.display_name.is_empty() {
                    out.push(e);
                }
            }
        }
        out
    }
    #[cfg(not(windows))]
    {
        Vec::new()
    }
}

/// Pfad aus "DisplayIcon" ("C:\x\game.exe",0 → C:\x\game.exe)
pub fn icon_path(display_icon: &str) -> String {
    let s = display_icon.trim();
    let s = s.strip_prefix('"').map(|r| r.split('"').next().unwrap_or(r)).unwrap_or(s);
    s.split(',').next().unwrap_or(s).trim().to_string()
}

#[cfg(windows)]
fn reg(hive: sys::Hive, path: &str, name: &str) -> Option<String> {
    sys::reg_str(hive, path, name)
}

#[cfg(windows)]
fn subkeys(path: &str) -> Vec<String> {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    winreg::RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(path)
        .map(|k| k.enum_keys().flatten().collect())
        .unwrap_or_default()
}

// ----------------------------------------------------------------------
// GOG (Windows: Registry + GOG Galaxy; Linux: Heroic)
// ----------------------------------------------------------------------

pub struct Gog;

#[cfg(windows)]
fn galaxy_exe() -> Option<PathBuf> {
    let dir = reg(sys::Hive::LocalMachine, "SOFTWARE\\WOW6432Node\\GOG.com\\GalaxyClient\\paths", "client")?;
    let p = Path::new(&dir).join("GalaxyClient.exe");
    p.exists().then_some(p)
}

/// Heroic: alle gekauften GOG-Spiele aus der Bibliothek (nicht installiert)
pub fn parse_heroic_gog_library(library_json: &str) -> Vec<Game> {
    let library: serde_json::Value = serde_json::from_str(library_json).unwrap_or(serde_json::Value::Null);
    let mut out = Vec::new();
    for g in library.get("games").and_then(|g| g.as_array()).into_iter().flatten() {
        let s = |k: &str| g.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let (app, title) = (s("app_name"), s("title"));
        if app.is_empty() || title.is_empty() || !app.chars().all(|c| c.is_ascii_alphanumeric()) {
            continue;
        }
        let mut game = owned_game(
            format!("gog:{app}"),
            title,
            Source::Gog,
            LaunchTarget::Uri(format!("heroic://launch?appName={}&runner=gog", sys::url_encode(&app))),
        );
        let cover = s("art_square");
        if cover.starts_with("https://") {
            game.art.cover.push(cover);
        }
        out.push(game);
    }
    out
}

/// Heroic (Linux): gog_store/installed.json + gog_store/library.json
pub fn parse_heroic_gog(installed_json: &str, library_json: &str) -> Vec<Game> {
    let installed: serde_json::Value = serde_json::from_str(installed_json).unwrap_or(serde_json::Value::Null);
    let library: serde_json::Value = serde_json::from_str(library_json).unwrap_or(serde_json::Value::Null);
    let lib: Vec<serde_json::Value> = library.get("games").and_then(|g| g.as_array()).cloned().unwrap_or_default();
    let mut out = Vec::new();
    for e in installed.get("installed").and_then(|i| i.as_array()).into_iter().flatten() {
        let app = e.get("appName").and_then(|a| a.as_str()).unwrap_or("");
        if app.is_empty() {
            continue;
        }
        let info = lib.iter().find(|g| g.get("app_name").and_then(|a| a.as_str()) == Some(app));
        let get = |k: &str| info.and_then(|i| i.get(k)).and_then(|v| v.as_str()).unwrap_or("").to_string();
        let title = get("title");
        let name = if title.is_empty() { app.to_string() } else { title };
        let mut g = game(
            format!("gog:{app}"),
            name,
            Source::Gog,
            e.get("install_path").and_then(|p| p.as_str()).map(|s| s.to_string()),
            LaunchTarget::Uri(format!("heroic://launch?appName={}&runner=gog", sys::url_encode(app))),
        );
        for (k, slot) in [("art_square", 0), ("art_cover", 1)] {
            let u = get(k);
            if u.starts_with("https://") {
                if slot == 0 {
                    g.art.cover.push(u.clone());
                    g.art.icon.push(u);
                } else {
                    g.art.hero.push(u.clone());
                    g.art.header.push(u);
                }
            }
        }
        out.push(g);
    }
    out
}

#[cfg(windows)]
fn galaxy_db() -> Option<PathBuf> {
    sys::env_path("ProgramData").map(|p| p.join("GOG.com").join("Galaxy").join("storage").join("galaxy-2.0.db"))
}

impl StoreLibrary for Gog {
    fn supported(&self) -> bool {
        true
    }
    fn has_library(&self) -> bool {
        true
    }
    fn owned(&self) -> Vec<Game> {
        #[cfg(windows)]
        {
            let Some(conn) = galaxy_db().and_then(|p| open_db(&p)) else { return Vec::new() };
            galaxy_owned(&conn)
                .into_iter()
                .map(|(id, title, cover)| {
                    let mut g = owned_game(format!("gog:{id}"), title, Source::Gog, LaunchTarget::Uri(format!("goggalaxy://openGameView/{id}")));
                    g.art.cover.extend(cover);
                    g
                })
                .collect()
        }
        #[cfg(not(windows))]
        {
            let mut out: Vec<Game> = Vec::new();
            for d in crate::epic::heroic_dirs() {
                let library = std::fs::read_to_string(d.join("store_cache/gog_library.json"))
                    .or_else(|_| std::fs::read_to_string(d.join("gog_store/library.json")))
                    .unwrap_or_default();
                for g in parse_heroic_gog_library(&library) {
                    if !out.iter().any(|o| o.id == g.id) {
                        out.push(g);
                    }
                }
            }
            out
        }
    }
    fn client_uri(&self) -> Option<&'static str> {
        Some("goggalaxy://")
    }
    fn source(&self) -> Source {
        Source::Gog
    }
    fn client_installed(&self) -> bool {
        #[cfg(windows)]
        {
            galaxy_exe().is_some()
        }
        #[cfg(not(windows))]
        {
            !crate::epic::heroic_dirs().is_empty()
        }
    }
    fn download_url(&self) -> &'static str {
        if cfg!(target_os = "linux") {
            "https://flathub.org/apps/com.heroicgameslauncher.hgl"
        } else {
            "https://www.gog.com/galaxy"
        }
    }
    fn games(&self) -> Vec<Game> {
        #[cfg(windows)]
        {
            let galaxy = galaxy_exe();
            let mut out = Vec::new();
            let base = "SOFTWARE\\WOW6432Node\\GOG.com\\Games";
            for id in subkeys(base) {
                let path = format!("{base}\\{id}");
                let r = |n: &str| reg(sys::Hive::LocalMachine, &path, n).unwrap_or_default();
                let name = r("gameName");
                let dir = r("path");
                let exe = r("exe");
                if name.is_empty() || dir.is_empty() || !Path::new(&dir).exists() {
                    continue;
                }
                // Mit Galaxy über Galaxy starten (Cloud-Speicherstände, Overlay), sonst direkt
                let launch = match &galaxy {
                    Some(g) => LaunchTarget::Exe {
                        path: g.to_string_lossy().to_string(),
                        args: vec!["/command=runGame".into(), format!("/gameId={id}"), format!("/path={dir}")],
                        dir: String::new(),
                    },
                    None => LaunchTarget::Exe {
                        path: exe.clone(),
                        args: r("launchParam").split_whitespace().map(|s| s.to_string()).collect(),
                        dir: r("workingDir"),
                    },
                };
                let mut g = game(format!("gog:{id}"), name, Source::Gog, Some(dir), launch);
                g.exe = (!exe.is_empty()).then_some(exe);
                out.push(g);
            }
            out
        }
        #[cfg(not(windows))]
        {
            let mut out = Vec::new();
            for d in crate::epic::heroic_dirs() {
                let installed = std::fs::read_to_string(d.join("gog_store/installed.json")).unwrap_or_default();
                let library = std::fs::read_to_string(d.join("store_cache/gog_library.json"))
                    .or_else(|_| std::fs::read_to_string(d.join("gog_store/library.json")))
                    .unwrap_or_default();
                for g in parse_heroic_gog(&installed, &library) {
                    if !out.iter().any(|o: &Game| o.id == g.id) {
                        out.push(g);
                    }
                }
            }
            out
        }
    }
}

// ----------------------------------------------------------------------
// EA app (Windows)
// ----------------------------------------------------------------------

pub struct Ea;

/// Erste contentID aus "__Installer\installerdata.xml"
pub fn ea_content_id(xml: &str) -> Option<String> {
    let start = xml.find("<contentID>")? + "<contentID>".len();
    let end = xml[start..].find("</contentID>")? + start;
    let id = xml[start..end].trim();
    (!id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | ':'))).then(|| id.to_string())
}

impl StoreLibrary for Ea {
    fn client_uri(&self) -> Option<&'static str> {
        Some("origin2://")
    }
    fn source(&self) -> Source {
        Source::Ea
    }
    fn client_installed(&self) -> bool {
        sys::protocol_registered("origin2") || sys::protocol_registered("link2ea")
    }
    fn download_url(&self) -> &'static str {
        "https://www.ea.com/ea-app"
    }
    fn games(&self) -> Vec<Game> {
        let mut out = Vec::new();
        if !cfg!(windows) {
            return out;
        }
        for e in uninstall_entries() {
            if e.install_location.is_empty() {
                continue;
            }
            let xml_path = Path::new(&e.install_location).join("__Installer").join("installerdata.xml");
            let Ok(xml) = std::fs::read_to_string(&xml_path) else { continue };
            let Some(cid) = ea_content_id(&xml) else { continue };
            let id = format!("ea:{cid}");
            if out.iter().any(|g: &Game| g.id == id) {
                continue;
            }
            let mut g = game(
                id,
                e.display_name.clone(),
                Source::Ea,
                Some(e.install_location.clone()),
                LaunchTarget::Uri(format!("origin2://game/launch?offerIds={}&autoDownload=1", sys::url_encode(&cid))),
            );
            let icon = icon_path(&e.display_icon);
            g.exe = icon.to_lowercase().ends_with(".exe").then_some(icon);
            out.push(g);
        }
        out
    }
}

// ----------------------------------------------------------------------
// Ubisoft Connect (Windows)
// ----------------------------------------------------------------------

pub struct Ubisoft;

impl StoreLibrary for Ubisoft {
    fn client_uri(&self) -> Option<&'static str> {
        Some("uplay://")
    }
    fn source(&self) -> Source {
        Source::Ubisoft
    }
    fn client_installed(&self) -> bool {
        sys::protocol_registered("uplay")
    }
    fn download_url(&self) -> &'static str {
        "https://ubisoftconnect.com/"
    }
    fn games(&self) -> Vec<Game> {
        #[cfg(windows)]
        {
            let entries = uninstall_entries();
            let base = "SOFTWARE\\WOW6432Node\\Ubisoft\\Launcher\\Installs";
            let mut out = Vec::new();
            for id in subkeys(base) {
                let dir = reg(sys::Hive::LocalMachine, &format!("{base}\\{id}"), "InstallDir").unwrap_or_default();
                let dir = sys::native_path(dir.trim_end_matches(['/', '\\']));
                if dir.is_empty() || !Path::new(&dir).exists() {
                    continue;
                }
                let name = entries
                    .iter()
                    .find(|e| e.key.eq_ignore_ascii_case(&format!("Uplay Install {id}")))
                    .map(|e| e.display_name.clone())
                    .unwrap_or_else(|| Path::new(&dir).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default());
                if name.is_empty() {
                    continue;
                }
                out.push(game(format!("ubisoft:{id}"), name, Source::Ubisoft, Some(dir), LaunchTarget::Uri(format!("uplay://launch/{id}/0"))));
            }
            out
        }
        #[cfg(not(windows))]
        {
            Vec::new()
        }
    }
}

// ----------------------------------------------------------------------
// Battle.net (Windows)
// ----------------------------------------------------------------------

pub struct BattleNet;

/// Interne Kennung (aus dem Deinstallations-Eintrag "--uid=…") → Startcode für Battle.net
pub fn battlenet_code(uid: &str) -> Option<&'static str> {
    let uid = uid.to_ascii_lowercase();
    let code = match uid.as_str() {
        "wow" => "WoW",
        "wow_classic" | "wow_classic_era" => "WoWC",
        "overwatch" | "prometheus" => "Pro",
        "hs_beta" | "hearthstone" => "WTCG",
        "diablo3" => "D3",
        "fenris" => "Fen",
        "osi" => "OSI",
        "s2" | "sc2" => "S2",
        "s1" | "starcraft" => "S1",
        "heroes" => "Hero",
        "w3" => "W3",
        "odin" => "ODIN",
        "viper" => "VIPR",
        "zeus" => "ZEUS",
        "lazarus" => "LAZR",
        "fore" => "FORE",
        "auks" => "AUKS",
        "anbs" => "ANBS",
        "rtro" => "RTRO",
        "wlby" => "WLBY",
        "gryphon" => "GRY",
        _ => return None,
    };
    Some(code)
}

/// "--uid=overwatch" aus dem Deinstallations-Befehl
pub fn battlenet_uid(uninstall: &str) -> Option<String> {
    let i = uninstall.find("--uid=")? + 6;
    let rest = &uninstall[i..];
    let uid: String = rest.chars().take_while(|c| !c.is_whitespace() && *c != '"').collect();
    (!uid.is_empty()).then_some(uid)
}

#[cfg(windows)]
fn battlenet_exe() -> Option<PathBuf> {
    let from_reg = reg(sys::Hive::LocalMachine, "SOFTWARE\\WOW6432Node\\Blizzard Entertainment\\Battle.net\\Capabilities", "ApplicationIcon")
        .map(|i| PathBuf::from(icon_path(&i)));
    let fallback = sys::env_path("ProgramFiles(x86)").map(|p| p.join("Battle.net").join("Battle.net.exe"));
    [from_reg, fallback].into_iter().flatten().find(|p| p.exists())
}

impl StoreLibrary for BattleNet {
    fn client_uri(&self) -> Option<&'static str> {
        Some("battlenet://")
    }
    fn source(&self) -> Source {
        Source::BattleNet
    }
    fn client_installed(&self) -> bool {
        #[cfg(windows)]
        {
            battlenet_exe().is_some()
        }
        #[cfg(not(windows))]
        {
            false
        }
    }
    fn download_url(&self) -> &'static str {
        "https://download.battle.net/"
    }
    fn games(&self) -> Vec<Game> {
        #[cfg(windows)]
        {
            let client = battlenet_exe();
            let mut out = Vec::new();
            for e in uninstall_entries() {
                if !e.uninstall_string.contains("Battle.net") || !e.uninstall_string.contains("--uid=") {
                    continue;
                }
                let Some(uid) = battlenet_uid(&e.uninstall_string) else { continue };
                if uid.starts_with("battle.net") || uid.contains("agent") {
                    continue;
                }
                let icon = icon_path(&e.display_icon);
                let launch = match (battlenet_code(&uid), &client) {
                    (Some(code), Some(c)) => LaunchTarget::Exe {
                        path: c.to_string_lossy().to_string(),
                        args: vec![format!("--exec=launch {code}")],
                        dir: String::new(),
                    },
                    _ => LaunchTarget::Exe { path: icon.clone(), args: Vec::new(), dir: String::new() },
                };
                let mut g = game(format!("battlenet:{uid}"), e.display_name.clone(), Source::BattleNet, Some(e.install_location.clone()), launch);
                g.exe = icon.to_lowercase().ends_with(".exe").then_some(icon);
                if !out.iter().any(|o: &Game| o.id == g.id) {
                    out.push(g);
                }
            }
            out
        }
        #[cfg(not(windows))]
        {
            Vec::new()
        }
    }
}

// ----------------------------------------------------------------------
// Amazon Games (Windows)
// ----------------------------------------------------------------------

pub struct Amazon;

/// Spiel-ID aus "…Amazon Game Remover.exe" -m Game -p amzn1.adg.product.…
pub fn amazon_game_id(uninstall: &str) -> Option<String> {
    if !uninstall.contains("Amazon Game Remover") {
        return None;
    }
    let i = uninstall.find(" -p ")? + 4;
    let id: String = uninstall[i..].trim_start().chars().take_while(|c| !c.is_whitespace() && *c != '"').collect();
    (!id.is_empty()).then_some(id)
}

#[cfg(windows)]
fn amazon_db() -> Option<PathBuf> {
    sys::env_path("LOCALAPPDATA").map(|p| p.join("Amazon Games").join("Data").join("Games").join("Sql").join("GameProductInfo.sqlite"))
}

impl StoreLibrary for Amazon {
    fn has_library(&self) -> bool {
        true
    }
    fn owned(&self) -> Vec<Game> {
        #[cfg(windows)]
        {
            let Some(conn) = amazon_db().and_then(|p| open_db(&p)) else { return Vec::new() };
            amazon_owned(&conn)
                .into_iter()
                .filter(|(id, _, _)| id.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_')))
                .map(|(id, title, icon)| {
                    let mut g = owned_game(format!("amazon:{id}"), title, Source::Amazon, LaunchTarget::Uri(format!("amazon-games://play/{}", sys::url_encode(&id))));
                    g.art.icon.extend(icon);
                    g
                })
                .collect()
        }
        #[cfg(not(windows))]
        {
            Vec::new()
        }
    }
    fn client_uri(&self) -> Option<&'static str> {
        Some("amazon-games://")
    }
    fn source(&self) -> Source {
        Source::Amazon
    }
    fn client_installed(&self) -> bool {
        sys::protocol_registered("amazon-games")
    }
    fn download_url(&self) -> &'static str {
        "https://gaming.amazon.com/home"
    }
    fn games(&self) -> Vec<Game> {
        let mut out = Vec::new();
        for e in uninstall_entries() {
            let Some(id) = amazon_game_id(&e.uninstall_string) else { continue };
            out.push(game(
                format!("amazon:{id}"),
                e.display_name.clone(),
                Source::Amazon,
                Some(e.install_location.clone()),
                LaunchTarget::Uri(format!("amazon-games://play/{}", sys::url_encode(&id))),
            ));
        }
        out
    }
}

// ----------------------------------------------------------------------
// itch.io (alle Systeme): apps/<Spiel>/.itch/receipt.json.gz
// ----------------------------------------------------------------------

pub struct Itch;

fn itch_apps_dir() -> PathBuf {
    if cfg!(windows) {
        sys::env_path("APPDATA").unwrap_or_else(sys::home_dir).join("itch").join("apps")
    } else if cfg!(target_os = "macos") {
        sys::home_dir().join("Library/Application Support/itch/apps")
    } else {
        sys::home_dir().join(".config/itch/apps")
    }
}

/// Titel und Cover aus der (entpackten) receipt.json
pub fn parse_itch_receipt(json: &str) -> Option<(String, Option<String>)> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let g = v.get("game")?;
    let title = g.get("title")?.as_str()?.trim().to_string();
    let cover = ["stillCoverUrl", "coverUrl"]
        .iter()
        .find_map(|k| g.get(*k).and_then(|c| c.as_str()).filter(|u| u.starts_with("https://")).map(|u| u.to_string()));
    (!title.is_empty()).then_some((title, cover))
}

/// Startbare Datei in einem Spielordner raten (auch für den Ordner-Scan)
pub fn guess_exe_pub(dir: &Path, name: &str) -> Option<PathBuf> {
    guess_exe(dir, name)
}

fn guess_exe(dir: &Path, name: &str) -> Option<PathBuf> {
    if cfg!(windows) {
        return crate::icons::guess_main_exe(dir, name);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut best: Option<(u64, PathBuf)> = None;
        let mut stack = vec![(dir.to_path_buf(), 0)];
        while let Some((d, depth)) = stack.pop() {
            for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
                let p = e.path();
                let fname = e.file_name().to_string_lossy().to_string();
                if fname.starts_with('.') {
                    continue;
                }
                let Ok(m) = e.metadata() else { continue };
                if m.is_dir() {
                    if depth < 2 {
                        stack.push((p, depth + 1));
                    }
                } else if m.permissions().mode() & 0o111 != 0 && !fname.contains(".so") {
                    let size = m.len();
                    if best.as_ref().map(|(s, _)| size > *s).unwrap_or(true) {
                        best = Some((size, p));
                    }
                }
            }
        }
        best.map(|(_, p)| p)
    }
    #[cfg(not(unix))]
    {
        let _ = (dir, name);
        None
    }
}

/// Nur echte itch.io-Seiten (https://<name>.itch.io/…) öffnen
pub fn is_itch_url(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("https://") else { return false };
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    !host.contains('@') && !host.contains(':') && (host == "itch.io" || host.ends_with(".itch.io")) && host.len() < 80
}

fn itch_db() -> PathBuf {
    itch_apps_dir().parent().map(|p| p.join("db").join("butler.db")).unwrap_or_default()
}

impl StoreLibrary for Itch {
    fn supported(&self) -> bool {
        true
    }
    fn has_library(&self) -> bool {
        true
    }
    fn owned(&self) -> Vec<Game> {
        let Some(conn) = open_db(&itch_db()) else { return Vec::new() };
        itch_owned(&conn)
            .into_iter()
            .map(|(id, title, cover, url)| {
                // Installieren geht über die itch-App; der Link öffnet die offizielle Spielseite
                let page = if is_itch_url(&url) { url } else { "https://itch.io/my-purchases".to_string() };
                let mut g = owned_game(format!("itch-owned:{id}"), title, Source::Itch, LaunchTarget::Uri(page));
                if let Some(c) = cover {
                    g.art.cover.push(c.clone());
                    g.art.header.push(c);
                }
                g
            })
            .collect()
    }
    fn source(&self) -> Source {
        Source::Itch
    }
    fn client_installed(&self) -> bool {
        itch_apps_dir().exists()
    }
    fn download_url(&self) -> &'static str {
        "https://itch.io/app"
    }
    fn games(&self) -> Vec<Game> {
        use std::io::Read;
        let mut out = Vec::new();
        for e in std::fs::read_dir(itch_apps_dir()).into_iter().flatten().flatten() {
            let dir = e.path();
            let Ok(bytes) = std::fs::read(dir.join(".itch").join("receipt.json.gz")) else { continue };
            let mut json = String::new();
            if flate2::read::GzDecoder::new(&bytes[..]).take(4 * 1024 * 1024).read_to_string(&mut json).is_err() {
                continue;
            }
            let Some((title, cover)) = parse_itch_receipt(&json) else { continue };
            let Some(exe) = guess_exe(&dir, &title) else { continue };
            let folder = e.file_name().to_string_lossy().to_string();
            let mut g = game(
                format!("itch:{folder}"),
                title,
                Source::Itch,
                Some(dir.to_string_lossy().to_string()),
                LaunchTarget::Exe {
                    path: exe.to_string_lossy().to_string(),
                    args: Vec::new(),
                    dir: exe.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
                },
            );
            g.exe = Some(exe.to_string_lossy().to_string());
            if let Some(c) = cover {
                g.art.cover.push(c.clone());
                g.art.header.push(c);
            }
            out.push(g);
        }
        out
    }
}

// ----------------------------------------------------------------------
// Xbox / Microsoft Store (Windows): <Laufwerk>:\XboxGames\<Spiel>\Content\MicrosoftGame.config
// ----------------------------------------------------------------------

pub struct Xbox;

#[derive(Debug, PartialEq)]
pub struct XboxConfig {
    pub identity: String,
    pub display_name: String,
    pub exe_id: String,
    pub logo: String,
}

fn xml_attr(tag: &str, attr: &str) -> Option<String> {
    let needle = format!("{attr}=\"");
    let i = tag.find(&needle)? + needle.len();
    let end = tag[i..].find('"')? + i;
    Some(tag[i..end].to_string())
}

fn xml_tag<'a>(xml: &'a str, name: &str) -> Option<&'a str> {
    let i = xml.find(&format!("<{name}"))?;
    let end = xml[i..].find('>')? + i;
    Some(&xml[i..=end])
}

pub fn parse_xbox_config(xml: &str) -> Option<XboxConfig> {
    let identity = xml_attr(xml_tag(xml, "Identity")?, "Name")?;
    let visuals = xml_tag(xml, "ShellVisuals").unwrap_or("");
    let exe_tag = xml_tag(xml, "Executable ").or_else(|| xml_tag(xml, "Executable"))?;
    Some(XboxConfig {
        identity,
        display_name: xml_attr(visuals, "DefaultDisplayName").unwrap_or_default(),
        exe_id: xml_attr(exe_tag, "Id").unwrap_or_else(|| "Game".into()),
        logo: xml_attr(visuals, "Square150x150Logo").or_else(|| xml_attr(visuals, "Square480x480Logo")).unwrap_or_default(),
    })
}

fn xbox_roots() -> Vec<PathBuf> {
    if !cfg!(windows) {
        return Vec::new();
    }
    (b'C'..=b'Z')
        .map(|d| PathBuf::from(format!("{}:\\XboxGames", d as char)))
        .filter(|p| p.is_dir())
        .collect()
}

impl StoreLibrary for Xbox {
    fn source(&self) -> Source {
        Source::Xbox
    }
    fn client_installed(&self) -> bool {
        !xbox_roots().is_empty() || crate::appx::find("Microsoft.GamingApp").is_some()
    }
    fn download_url(&self) -> &'static str {
        "https://www.xbox.com/apps/xbox-app-for-pc"
    }
    fn games(&self) -> Vec<Game> {
        let roots = xbox_roots();
        if roots.is_empty() {
            return Vec::new();
        }
        let packages = crate::appx::packages();
        let mut out = Vec::new();
        for root in roots {
            for e in std::fs::read_dir(&root).into_iter().flatten().flatten() {
                let content = e.path().join("Content");
                let Ok(xml) = std::fs::read_to_string(content.join("MicrosoftGame.config")) else { continue };
                let Some(cfg) = parse_xbox_config(&xml) else { continue };
                // SICHERHEIT: nur einfache Kennungen an explorer.exe weitergeben
                let simple = |v: &str| !v.is_empty() && v.len() < 130 && v.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
                if !simple(&cfg.exe_id) || !simple(&cfg.identity) {
                    continue;
                }
                // Minecraft Launcher hat eine eigene Kachel (Quelle "Minecraft")
                if cfg.identity.eq_ignore_ascii_case(crate::minecraft::STORE_PACKAGE) {
                    continue;
                }
                let Some(pkg) = packages.iter().find(|p| p.name.eq_ignore_ascii_case(&cfg.identity)) else { continue };
                let name = if !cfg.display_name.is_empty() && !cfg.display_name.starts_with("ms-resource") {
                    cfg.display_name.clone()
                } else {
                    e.file_name().to_string_lossy().to_string()
                };
                let mut g = game(
                    format!("xbox:{}", cfg.identity),
                    name,
                    Source::Xbox,
                    Some(content.to_string_lossy().to_string()),
                    LaunchTarget::Appx { family: pkg.family.clone(), app: pkg.app_id(&cfg.exe_id).unwrap_or_else(|| cfg.exe_id.clone()) },
                );
                if !cfg.logo.is_empty() {
                    let logo = content.join(sys::native_path(&cfg.logo));
                    if logo.exists() {
                        g.art.icon.push(sys::file_url(&logo));
                    }
                }
                out.push(g);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ea_installerdata() {
        let xml = r#"<?xml version="1.0"?><DiPManifest><contentIDs><contentID>Origin.OFR.50.0004694</contentID><contentID>x</contentID></contentIDs></DiPManifest>"#;
        assert_eq!(ea_content_id(xml).as_deref(), Some("Origin.OFR.50.0004694"));
        assert_eq!(ea_content_id("<x/>"), None);
        assert_eq!(ea_content_id("<contentID>bad id!</contentID>"), None);
    }

    #[test]
    fn battlenet_codes() {
        let u = r#""C:\ProgramData\Battle.net\Agent\Blizzard Uninstaller.exe" --lang=deDE --uid=overwatch --displayname="Overwatch""#;
        assert_eq!(battlenet_uid(u).as_deref(), Some("overwatch"));
        assert_eq!(battlenet_code("overwatch"), Some("Pro"));
        assert_eq!(battlenet_code("fenris"), Some("Fen"));
        assert_eq!(battlenet_code("unbekannt"), None);
    }

    #[test]
    fn amazon_ids() {
        let u = r#""C:\Users\x\AppData\Local\Amazon Games\App\Amazon Game Remover.exe" -m Game -p amzn1.adg.product.1234-abcd"#;
        assert_eq!(amazon_game_id(u).as_deref(), Some("amzn1.adg.product.1234-abcd"));
        assert_eq!(amazon_game_id("C:\\uninstall.exe"), None);
    }

    #[test]
    fn itch_receipt() {
        let j = r#"{"game":{"id":1,"title":"Celeste Classic","coverUrl":"https://img.itch.zone/a.png"},"files":[]}"#;
        assert_eq!(parse_itch_receipt(j), Some(("Celeste Classic".into(), Some("https://img.itch.zone/a.png".into()))));
        assert_eq!(parse_itch_receipt("{}"), None);
    }

    #[test]
    fn xbox_config() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<Game configVersion="1">
  <Identity Name="Microsoft.624F8B84B80" Publisher="CN=A4954634-DF4B-47C7-AB70-D3215D246AF1" Version="1.0.0.0"/>
  <ExecutableList><Executable Name="Forza.exe" Id="Game" TargetDeviceFamily="PC"/></ExecutableList>
  <ShellVisuals DefaultDisplayName="Forza Horizon 5" Square150x150Logo="GraphicsLogo.png" Square480x480Logo="Big.png"/>
</Game>"#;
        let c = parse_xbox_config(xml).unwrap();
        assert_eq!(c.identity, "Microsoft.624F8B84B80");
        assert_eq!(c.display_name, "Forza Horizon 5");
        assert_eq!(c.exe_id, "Game");
        assert_eq!(c.logo, "GraphicsLogo.png");
    }

    #[test]
    fn heroic_gog() {
        let inst = r#"{"installed":[{"appName":"1207658930","install_path":"/home/u/Games/Heroic/Witcher","platform":"windows"}]}"#;
        let lib = r#"{"games":[{"app_name":"1207658930","title":"The Witcher","art_square":"https://images.gog.com/a.jpg"}]}"#;
        let g = parse_heroic_gog(inst, lib);
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].name, "The Witcher");
        assert!(matches!(&g[0].launch, LaunchTarget::Uri(u) if u.contains("runner=gog")));
        assert_eq!(g[0].art.cover.len(), 1);
    }

    #[test]
    fn owned_from_databases() {
        let c = rusqlite::Connection::open_in_memory().unwrap();
        c.execute_batch(
            "CREATE TABLE LibraryReleases(releaseKey TEXT, userId INT);
             CREATE TABLE GamePieceTypes(id INT, type TEXT);
             CREATE TABLE GamePieces(releaseKey TEXT, gamePieceTypeId INT, value TEXT);
             INSERT INTO GamePieceTypes VALUES (1,'originalTitle'),(2,'originalImages');
             INSERT INTO LibraryReleases VALUES ('gog_1207658930',1),('steam_570',1),('gog_x1',1);
             INSERT INTO GamePieces VALUES ('gog_1207658930',1,'{\"title\":\"The Witcher\"}'),
               ('gog_1207658930',2,'{\"verticalCover\":\"https://images.gog.com/c.jpg\"}'),
               ('steam_570',1,'{\"title\":\"Dota 2\"}'),('gog_x1',1,'{\"title\":\"Kaputt\"}');
             CREATE TABLE DbSet(Id INT, ProductIdStr TEXT, ProductTitle TEXT, ProductIconUrl TEXT);
             INSERT INTO DbSet VALUES (1,'amzn1.adg.product.abc','Spiel A','https://m.media-amazon.com/x.png'),(2,'','Leer',NULL);
             CREATE TABLE games(id INT, title TEXT, url TEXT, cover_url TEXT, still_cover_url TEXT);
             CREATE TABLE download_keys(id INT, game_id INT, owner_id INT);
             INSERT INTO games VALUES (5,'Celeste','https://mattmakesgames.itch.io/celeste','https://img.itch.zone/c.png',NULL),(6,'Nicht gekauft','',NULL,NULL);
             INSERT INTO download_keys VALUES (1,5,9);",
        )
        .unwrap();
        let gog = galaxy_owned(&c);
        assert_eq!(gog, vec![("1207658930".into(), "The Witcher".into(), Some("https://images.gog.com/c.jpg".into()))]);
        let amz = amazon_owned(&c);
        assert_eq!(amz.len(), 1);
        assert_eq!(amz[0].1, "Spiel A");
        let itch = itch_owned(&c);
        assert_eq!(itch.len(), 1);
        assert_eq!(itch[0].1, "Celeste");
        let lib = parse_heroic_gog_library(r#"{"games":[{"app_name":"123","title":"Spiel"},{"app_name":"x;rm","title":"Böse"}]}"#);
        assert_eq!(lib.len(), 1);
        assert!(!lib[0].installed);
    }

    #[test]
    fn itch_urls() {
        assert!(is_itch_url("https://mattmakesgames.itch.io/celeste"));
        assert!(!is_itch_url("https://evil.com/.itch.io/x"));
        assert!(!is_itch_url("https://itch.io.evil.com/x"));
        assert!(!is_itch_url("https://user@evil.itch.io.com/"));
        assert!(!is_itch_url("http://a.itch.io/"));
    }

    #[test]
    fn icon_paths() {
        assert_eq!(icon_path(r#""C:\G\game.exe",0"#), "C:\\G\\game.exe");
        assert_eq!(icon_path(r#"C:\G\game.exe,0"#), "C:\\G\\game.exe");
    }
}
