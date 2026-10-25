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
    /// Gibt es diesen Store auf diesem Betriebssystem?
    fn supported(&self) -> bool {
        cfg!(windows)
    }
    /// Link, der das Store-Programm öffnet (z. B. zum Deinstallieren)
    fn client_uri(&self) -> Option<&'static str> {
        None
    }

    fn scan(&self) -> StoreScan {
        let games = self.games();
        let client = self.client_installed();
        let status = StoreStatus {
            found: client || !games.is_empty(),
            installed: games.len(),
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
        .map(|s| (s.source(), s.scan()))
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

impl StoreLibrary for Gog {
    fn supported(&self) -> bool {
        true
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

impl StoreLibrary for Amazon {
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

impl StoreLibrary for Itch {
    fn supported(&self) -> bool {
        true
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
                    LaunchTarget::Appx { family: pkg.family.clone(), app: cfg.exe_id.clone() },
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
    fn icon_paths() {
        assert_eq!(icon_path(r#""C:\G\game.exe",0"#), "C:\\G\\game.exe");
        assert_eq!(icon_path(r#"C:\G\game.exe,0"#), "C:\\G\\game.exe");
    }
}
