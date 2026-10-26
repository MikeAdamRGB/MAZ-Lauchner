//! Erweiterungen: eigene Programme, die über JSON-Zeilen (stdin/stdout) mit dem Launcher reden.
//! Siehe docs/EXTENSIONS.md.
//!
//! Grundregeln: kein fremder Code im Launcher-Prozess, keine Passwörter/Tokens, keine
//! Spiele-Downloads, kein Online-Katalog, standardmäßig ausgeschaltet. Jede Antwort wird
//! geprüft wie bei den eingebauten Stores. Eine kaputte Erweiterung blockiert nie den Launcher.

#![deny(clippy::all)]

pub mod manifest;
pub mod process;
pub mod zipread;

use crate::model::{Art, Game, LaunchTarget, Settings, Source, StoreStatus};
use crate::stores::{StoreLibrary, StoreScan};
use manifest::{ExtKind, Manifest};
use process::{CallError, ExtProcess, TIMEOUT};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

pub const MAX_GAMES: usize = 5000;

/// Was der Nutzer zu einer Erweiterung entschieden hat (steht in den Einstellungen)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ExtState {
    pub enabled: bool,
    /// SHA-256 über alle Dateien, dem der Nutzer zugestimmt hat (Hash-Pinning)
    pub hash: String,
    /// Bestätigte zusätzliche Link-Arten
    pub approved_schemes: Vec<String>,
    /// Internet erlaubt
    pub network: bool,
    pub installed: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    Active,
    Off,
    /// Globaler Schalter "Erweiterungen erlauben" ist aus
    NotAllowed,
    /// Ordner liegt da, wurde aber nie über den Launcher bestätigt
    NeedsApproval,
    /// Dateien haben sich seit der Zustimmung geändert
    HashChanged,
    Invalid(String),
    Failed(String),
}

pub struct Loaded {
    pub idx: u16,
    pub id: String,
    pub dir: PathBuf,
    pub manifest: Option<Manifest>,
    pub run: Option<PathBuf>,
    pub hash: String,
    status: Mutex<Status>,
    proc: Mutex<Option<ExtProcess>>,
    restarts: AtomicU8,
    approved_schemes: Vec<String>,
    network: bool,
    texts: HashMap<String, String>,
    texts_en: HashMap<String, String>,
}

impl Loaded {
    pub fn status(&self) -> Status {
        self.status.lock().map(|s| s.clone()).unwrap_or(Status::Failed("lock".into()))
    }
    fn set_status(&self, s: Status) {
        if let Ok(mut g) = self.status.lock() {
            *g = s;
        }
    }
    pub fn name(&self) -> String {
        self.manifest.as_ref().map(|m| self.text(&m.name)).unwrap_or_else(|| self.id.clone())
    }
    pub fn has(&self, k: ExtKind) -> bool {
        self.manifest.as_ref().map(|m| m.has(k)).unwrap_or(false)
    }
    /// Text der Erweiterung übersetzen: "@schlüssel" → lang/<sprache>.txt, sonst Englisch, sonst roh
    pub fn text(&self, s: &str) -> String {
        match s.strip_prefix('@') {
            Some(key) => self.texts.get(key).or_else(|| self.texts_en.get(key)).cloned().unwrap_or_else(|| key.to_string()),
            None => s.to_string(),
        }
    }
    fn stop(&self) {
        if let Ok(mut p) = self.proc.lock() {
            if let Some(mut pr) = p.take() {
                pr.stop();
            }
        }
    }
}

// ----------------------------------------------------------------------
// Register aller Erweiterungen
// ----------------------------------------------------------------------

fn registry() -> &'static RwLock<Vec<Arc<Loaded>>> {
    static REG: OnceLock<RwLock<Vec<Arc<Loaded>>>> = OnceLock::new();
    REG.get_or_init(|| RwLock::new(Vec::new()))
}

/// Feste Nummer pro Erweiterungs-ID (bleibt für die ganze Sitzung gleich)
fn index_of(id: &str) -> u16 {
    static IDS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
    let m = IDS.get_or_init(|| Mutex::new(Vec::new()));
    let Ok(mut ids) = m.lock() else { return u16::MAX };
    match ids.iter().position(|x| x == id) {
        Some(i) => i as u16,
        None => {
            ids.push(id.to_string());
            (ids.len() - 1).min(u16::MAX as usize - 1) as u16
        }
    }
}

pub fn all() -> Vec<Arc<Loaded>> {
    registry().read().map(|r| r.clone()).unwrap_or_default()
}

pub fn get(id: &str) -> Option<Arc<Loaded>> {
    all().into_iter().find(|e| e.id == id)
}

fn by_idx(idx: u16) -> Option<Arc<Loaded>> {
    all().into_iter().find(|e| e.idx == idx)
}

/// Anzeigename zu Source::Extension(n)
pub fn name_of(idx: u16) -> String {
    by_idx(idx).map(|e| e.name()).unwrap_or_else(|| "Extension".into())
}

/// Ordner, in dem Erweiterungen liegen
pub fn root() -> PathBuf {
    crate::sys::roaming_dir().join("extensions")
}

fn load_texts(dir: &Path, lang: &str) -> HashMap<String, String> {
    let p = dir.join("lang").join(format!("{lang}.txt"));
    match std::fs::metadata(&p) {
        Ok(m) if m.is_file() && m.len() < 256 * 1024 => std::fs::read_to_string(&p)
            .map(|t| {
                crate::i18n::parse(&t)
                    .into_iter()
                    .filter(|(k, v)| k.len() < 80 && v.len() < 500)
                    .collect()
            })
            .unwrap_or_default(),
        _ => HashMap::new(),
    }
}

/// Erweiterungsordner neu einlesen (läuft im Hintergrund-Thread beim Neu-Laden der Bibliothek)
pub fn refresh(settings: &Settings) {
    let old = all();
    let mut list: Vec<Arc<Loaded>> = Vec::new();
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(root())
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .collect();
    dirs.sort();
    for dir in dirs.into_iter().take(50) {
        let id = dir.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        if !manifest::valid_id(&id) {
            continue; // z. B. ".staging-…"
        }
        let state = settings.extensions.get(&id).cloned();
        let (manifest, run, mut status) = match manifest::load_dir(&dir) {
            Ok((m, _)) if m.id != id => (None, None, Status::Invalid(crate::i18n::t("ext.err.id_dir"))),
            Ok((m, run)) => (Some(m), Some(run), Status::Active),
            Err(e) => (None, None, Status::Invalid(e.explain())),
        };
        let hash = if manifest.is_some() { folder_hash(&dir).unwrap_or_default() } else { String::new() };
        if status == Status::Active {
            status = match &state {
                _ if !settings.extensions_allowed => Status::NotAllowed,
                None => Status::NeedsApproval,
                Some(s) if s.hash.is_empty() || s.hash != hash => Status::HashChanged,
                Some(s) if !s.enabled => Status::Off,
                Some(_) => Status::Active,
            };
        }
        // Laufenden Prozess behalten, wenn sich nichts geändert hat
        if let Some(prev) = old.iter().find(|o| o.id == id && o.hash == hash && !hash.is_empty()) {
            let keep_failed = matches!(prev.status(), Status::Failed(_));
            if status == Status::Active && keep_failed {
                list.push(prev.clone());
                continue;
            }
            if status == Status::Active && prev.status() == Status::Active {
                list.push(prev.clone());
                continue;
            }
            prev.stop();
        }
        let lang = crate::i18n::current_code();
        let st = state.unwrap_or_default();
        list.push(Arc::new(Loaded {
            idx: index_of(&id),
            texts: load_texts(&dir, &lang),
            texts_en: load_texts(&dir, "en"),
            id,
            dir,
            manifest,
            run,
            hash,
            status: Mutex::new(status),
            proc: Mutex::new(None),
            restarts: AtomicU8::new(0),
            approved_schemes: st.approved_schemes,
            network: st.network,
        }));
    }
    // Entfernte/geänderte Erweiterungen beenden
    for o in &old {
        if !list.iter().any(|n| Arc::ptr_eq(n, o)) {
            o.stop();
        }
    }
    if let Ok(mut r) = registry().write() {
        *r = list;
    }
}

/// Alle Prozesse beenden (beim Schließen des Launchers)
pub fn shutdown() {
    for e in all() {
        e.stop();
    }
}

// ----------------------------------------------------------------------
// Aufrufe
// ----------------------------------------------------------------------

fn start(e: &Loaded) -> Result<ExtProcess, CallError> {
    let run = e.run.as_ref().ok_or(CallError::Crashed)?;
    let mut p = ExtProcess::spawn(run, &e.dir, e.network)?;
    let hello = json!({
        "api_version": manifest::API_VERSION,
        "os": manifest::os_key(),
        "lang": crate::i18n::current_code(),
        "network": e.network,
    });
    let res = p.call("hello", hello, TIMEOUT)?;
    if !res.is_object() {
        return Err(CallError::BadJson);
    }
    Ok(p)
}

/// Aufruf an eine Erweiterung. Bei Absturz/Zeitüberschreitung einmal neu starten,
/// danach wird die Erweiterung deaktiviert (bis zum nächsten Start des Launchers).
pub fn call(e: &Loaded, name: &str, params: Value) -> Result<Value, CallError> {
    if e.status() != Status::Active {
        return Err(CallError::Crashed);
    }
    let mut guard = e.proc.lock().map_err(|_| CallError::Crashed)?;
    for _attempt in 0..2 {
        if guard.as_mut().map(|p| !p.alive()).unwrap_or(true) {
            *guard = None;
            match start(e) {
                Ok(p) => *guard = Some(p),
                Err(err) => {
                    if fail_once(e, &err) {
                        continue;
                    }
                    return Err(err);
                }
            }
        }
        let Some(p) = guard.as_mut() else { continue };
        match p.call(name, params.clone(), TIMEOUT) {
            Ok(v) => return Ok(v),
            Err(err @ CallError::Remote(..)) => return Err(err),
            Err(err) => {
                if let Some(mut dead) = guard.take() {
                    dead.stop();
                }
                if fail_once(e, &err) {
                    continue;
                }
                return Err(err);
            }
        }
    }
    Err(CallError::Crashed)
}

/// true = noch ein Versuch erlaubt
fn fail_once(e: &Loaded, err: &CallError) -> bool {
    let n = e.restarts.fetch_add(1, Ordering::SeqCst) + 1;
    if n <= 1 {
        return true;
    }
    e.set_status(Status::Failed(err.explain()));
    false
}

// ----------------------------------------------------------------------
// Antworten prüfen
// ----------------------------------------------------------------------

fn text(v: &Value, k: &str, max: usize) -> String {
    v.get(k)
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .chars()
        .filter(|c| !c.is_control())
        .take(max)
        .collect::<String>()
        .trim()
        .to_string()
}

fn safe_game_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 100 && id.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// Ist der Pfad (aufgelöst) innerhalb eines der Ordner?
pub fn inside_any(path: &Path, dirs: &[PathBuf]) -> bool {
    let Ok(p) = std::fs::canonicalize(path) else { return false };
    dirs.iter().filter_map(|d| std::fs::canonicalize(d).ok()).any(|d| (p.starts_with(&d) && p != d) || (p == d && path.is_dir()))
}

/// Darf diese Erweiterung den Link öffnen?
pub fn url_ok(e_schemes: &[String], url: &str) -> bool {
    if url.len() > 2000 || url.chars().any(|c| c.is_control() || c == '"') {
        return false;
    }
    if crate::sys::is_safe_url(url) {
        return true;
    }
    let scheme = url.split(':').next().unwrap_or("").to_ascii_lowercase();
    e_schemes.iter().any(|s| *s == scheme)
}

/// Von irgendeiner aktiven Erweiterung bestätigte Link-Art (für das Starten von Spielen)
pub fn scheme_approved(url: &str) -> bool {
    let scheme = url.split(':').next().unwrap_or("").to_ascii_lowercase();
    !scheme.is_empty() && all().iter().any(|e| e.status() == Status::Active && e.approved_schemes.iter().any(|s| *s == scheme))
}

/// Bild: https (PNG/JPEG/WebP) oder Datei im Erweiterungsordner (max. 5 MB)
fn image(e: &Loaded, v: &Value, k: &str) -> Option<String> {
    let s = v.get(k)?.as_str()?;
    if s.len() > 800 {
        return None;
    }
    let lower = s.to_ascii_lowercase();
    let path_part = lower.split(['?', '#']).next().unwrap_or("");
    let img_ext = [".png", ".jpg", ".jpeg", ".webp"].iter().any(|x| path_part.ends_with(x));
    if !img_ext {
        return None;
    }
    if s.starts_with("https://") {
        return Some(s.to_string());
    }
    let p = manifest::resolve_inside(&e.dir, s)?;
    let small = std::fs::metadata(&p).map(|m| m.len() <= 5 * 1024 * 1024).unwrap_or(false);
    small.then(|| crate::sys::file_url(&p))
}

/// Ein Spiel aus der Antwort einer Erweiterung prüfen und umwandeln
pub fn parse_game(e: &Loaded, v: &Value, installed_default: bool) -> Option<Game> {
    let gid = text(v, "id", 100);
    let name = text(v, "name", 120);
    if !safe_game_id(&gid) || name.is_empty() {
        return None;
    }
    let install_dir = v
        .get("install_dir")
        .and_then(|d| d.as_str())
        .filter(|d| d.len() < 500)
        .map(PathBuf::from)
        .filter(|d| d.is_absolute() && d.is_dir());
    let launch = v.get("launch")?;
    let target = if let Some(uri) = launch.get("uri").and_then(|u| u.as_str()) {
        if !url_ok(&e.approved_schemes, uri) {
            return None;
        }
        LaunchTarget::Uri(uri.to_string())
    } else if let Some(x) = launch.get("exe") {
        // Programm muss im Spielordner liegen
        let dir = install_dir.clone()?;
        let path = PathBuf::from(x.get("path")?.as_str()?);
        if !path.is_file() || !inside_any(&path, std::slice::from_ref(&dir)) {
            return None;
        }
        let args: Vec<String> = x
            .get("args")
            .and_then(|a| a.as_array())
            .map(|a| a.iter().filter_map(|s| s.as_str()).filter(|s| s.len() < 300 && !s.contains('\0')).take(20).map(String::from).collect())
            .unwrap_or_default();
        LaunchTarget::Exe { path: path.to_string_lossy().to_string(), args, dir: dir.to_string_lossy().to_string() }
    } else if let Some(a) = launch.get("appx") {
        if !cfg!(windows) {
            return None;
        }
        let simple = |s: &str| !s.is_empty() && s.len() < 130 && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'));
        let (family, app) = (text(a, "family", 130), text(a, "app", 130));
        if !simple(&family) || !simple(&app) {
            return None;
        }
        LaunchTarget::Appx { family, app }
    } else {
        return None;
    };
    let mut art = Art::default();
    art.cover.extend(image(e, v, "cover"));
    art.icon.extend(image(e, v, "icon"));
    art.hero.extend(image(e, v, "hero"));
    Some(Game {
        id: format!("ext:{}:{gid}", e.id),
        name,
        source: Source::Extension(e.idx),
        installed: v.get("installed").and_then(|b| b.as_bool()).unwrap_or(installed_default),
        install_dir: install_dir.map(|d| d.to_string_lossy().to_string()),
        exe: None,
        launch: target,
        art,
        size_bytes: v.get("size").and_then(|s| s.as_u64()),
        playtime_min: None,
        steam_appid: None,
    })
}

fn games_from(e: &Loaded, call_name: &str, installed: bool) -> Vec<Game> {
    let Ok(res) = call(e, call_name, json!({})) else { return Vec::new() };
    let mut out: Vec<Game> = Vec::new();
    for g in res.get("games").and_then(|g| g.as_array()).into_iter().flatten().take(MAX_GAMES) {
        if let Some(game) = parse_game(e, g, installed) {
            if !out.iter().any(|o| o.id == game.id) {
                out.push(game);
            }
        }
    }
    out
}

// ----------------------------------------------------------------------
// Anbindung an die Bibliothek
// ----------------------------------------------------------------------

pub struct ExtensionLibrary(pub Arc<Loaded>);

impl StoreLibrary for ExtensionLibrary {
    fn source(&self) -> Source {
        Source::Extension(self.0.idx)
    }
    fn client_installed(&self) -> bool {
        true
    }
    fn download_url(&self) -> &'static str {
        crate::sys::GITHUB_URL
    }
    fn games(&self) -> Vec<Game> {
        games_from(&self.0, "games", true)
    }
    fn owned(&self) -> Vec<Game> {
        games_from(&self.0, "owned", false)
    }
    fn has_library(&self) -> bool {
        true
    }
    fn supported(&self) -> bool {
        true
    }
    fn scan(&self, with_owned: bool) -> StoreScan {
        let mut games = self.games();
        let installed = games.len();
        if with_owned {
            for g in self.owned() {
                if !games.iter().any(|o| o.id == g.id) && games.len() < MAX_GAMES {
                    games.push(g);
                }
            }
        }
        let note = match self.0.status() {
            Status::Failed(e) => Some(e),
            _ => None,
        };
        let status = StoreStatus { found: true, installed, owned: games.len(), note, ..Default::default() };
        StoreScan { games, status }
    }
}

/// Aktive Bibliotheks-Erweiterungen
pub fn libraries() -> Vec<ExtensionLibrary> {
    all().into_iter().filter(|e| e.status() == Status::Active && e.has(ExtKind::Library)).map(ExtensionLibrary).collect()
}

// ----------------------------------------------------------------------
// Metadaten
// ----------------------------------------------------------------------

/// Infos aus Erweiterungen (nach den eingebauten Quellen)
pub fn metadata(name: &str, game_id: &str) -> Option<crate::metadata::Info> {
    for e in all().into_iter().filter(|e| e.status() == Status::Active && e.has(ExtKind::Metadata)) {
        let Ok(v) = call(&e, "metadata", json!({ "name": name.chars().take(120).collect::<String>(), "id": game_id })) else { continue };
        let description = text(&v, "description", 4000);
        if description.is_empty() && v.get("genres").is_none() {
            continue;
        }
        return Some(crate::metadata::Info {
            description,
            genres: v
                .get("genres")
                .and_then(|g| g.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str()).map(|x| x.chars().take(30).collect()).take(8).collect())
                .unwrap_or_default(),
            developer: text(&v, "developer", 80),
            release: text(&v, "release", 30),
            cover: image(&e, &v, "cover").filter(|c| c.starts_with("https://")).unwrap_or_default(),
            source: format!("ext:{}", e.id),
            version: crate::metadata::VERSION,
            fetched: crate::sys::now_secs(),
            ..Default::default()
        });
    }
    None
}

// ----------------------------------------------------------------------
// Aktionen (zusätzliche Knöpfe in den Spiel-Details)
// ----------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Action {
    pub ext: String,
    pub id: String,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ActionResult {
    OpenFolder(String),
    OpenUrl(String),
    Message(String),
}

fn game_json(g: &Game) -> Value {
    json!({ "id": g.id, "name": g.name, "source": g.source.key(), "install_dir": g.install_dir })
}

pub fn actions(g: &Game) -> Vec<Action> {
    let mut out = Vec::new();
    for e in all().into_iter().filter(|e| e.status() == Status::Active && e.has(ExtKind::Action)) {
        let Ok(v) = call(&e, "actions", json!({ "game": game_json(g) })) else { continue };
        for a in v.get("actions").and_then(|a| a.as_array()).into_iter().flatten().take(8) {
            let (id, label) = (text(a, "id", 60), text(a, "label", 60));
            if safe_game_id(&id) && !label.is_empty() {
                out.push(Action { ext: e.id.clone(), id, label: e.text(&label) });
            }
        }
    }
    out
}

/// Aktion ausführen. Erlaubt sind NUR: Ordner öffnen (aus read_paths oder dem Spielordner),
/// erlaubten Link öffnen, Meldung anzeigen.
pub fn run_action(ext: &str, action: &str, g: &Game) -> Result<ActionResult, String> {
    let e = get(ext).ok_or_else(|| crate::i18n::t("ext.call.crashed"))?;
    let v = call(&e, "run_action", json!({ "action": action, "game": game_json(g) })).map_err(|x| x.explain())?;
    validate_action(&e, &v, g)
}

fn validate_action(e: &Loaded, v: &Value, g: &Game) -> Result<ActionResult, String> {
    let deny = || crate::i18n::t("ext.action.denied");
    match v.get("type").and_then(|t| t.as_str()) {
        Some("open_folder") => {
            let p = PathBuf::from(text(v, "path", 500));
            let mut allowed = e.manifest.as_ref().map(|m| m.read_paths()).unwrap_or_default();
            if let Some(d) = &g.install_dir {
                allowed.push(PathBuf::from(d));
            }
            if p.is_dir() && inside_any(&p, &allowed) {
                Ok(ActionResult::OpenFolder(p.to_string_lossy().to_string()))
            } else {
                Err(deny())
            }
        }
        Some("open_url") => {
            let u = text(v, "url", 2000);
            if url_ok(&e.approved_schemes, &u) {
                Ok(ActionResult::OpenUrl(u))
            } else {
                Err(deny())
            }
        }
        Some("message") => Ok(ActionResult::Message(e.text(&text(v, "text", 1000)))),
        _ => Err(deny()),
    }
}

// ----------------------------------------------------------------------
// Installation, Hash-Pinning, Entfernen
// ----------------------------------------------------------------------

/// SHA-256 über alle Dateien (Pfad + Inhalt), sortiert. Symlinks sind nicht erlaubt.
pub fn folder_hash(dir: &Path) -> Result<String, String> {
    let mut files: Vec<(String, PathBuf)> = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    let mut total: u64 = 0;
    while let Some(d) = stack.pop() {
        for ent in std::fs::read_dir(&d).map_err(|e| e.to_string())?.flatten() {
            let p = ent.path();
            let meta = std::fs::symlink_metadata(&p).map_err(|e| e.to_string())?;
            if meta.file_type().is_symlink() {
                return Err(crate::i18n::t("ext.err.symlink"));
            }
            if meta.is_dir() {
                stack.push(p);
            } else {
                total += meta.len();
                if total > 200 * 1024 * 1024 || files.len() > 2000 {
                    return Err(crate::i18n::t("ext.err.too_big"));
                }
                let rel = p.strip_prefix(dir).map(|r| r.to_string_lossy().replace('\\', "/")).unwrap_or_default();
                files.push((rel, p));
            }
        }
    }
    files.sort();
    let mut all = Vec::new();
    for (rel, p) in files {
        let data = std::fs::read(&p).map_err(|e| e.to_string())?;
        all.extend_from_slice(rel.as_bytes());
        all.push(0);
        all.extend_from_slice(&crate::util::sha256(&data));
    }
    Ok(crate::util::hex(&crate::util::sha256(&all)))
}

pub fn file_hash(p: &Path) -> String {
    std::fs::read(p).map(|d| crate::util::hex(&crate::util::sha256(&d))).unwrap_or_default()
}

/// Vorbereitete Installation (noch nicht bestätigt)
#[derive(Clone, Debug, PartialEq)]
pub struct Staged {
    pub dir: PathBuf,
    pub manifest: Manifest,
    pub hash: String,
    pub run_hash: String,
    pub update: bool,
}

fn copy_dir(from: &Path, to: &Path, budget: &mut u64) -> Result<(), String> {
    std::fs::create_dir_all(to).map_err(|e| e.to_string())?;
    for ent in std::fs::read_dir(from).map_err(|e| e.to_string())?.flatten() {
        let p = ent.path();
        let meta = std::fs::symlink_metadata(&p).map_err(|e| e.to_string())?;
        if meta.file_type().is_symlink() {
            return Err(crate::i18n::t("ext.err.symlink"));
        }
        let dest = to.join(ent.file_name());
        if meta.is_dir() {
            copy_dir(&p, &dest, budget)?;
        } else {
            *budget = budget.checked_sub(meta.len()).ok_or_else(|| crate::i18n::t("ext.err.too_big"))?;
            std::fs::copy(&p, &dest).map_err(|e| e.to_string())?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = meta.permissions().mode() & 0o755;
                let _ = std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(mode));
            }
        }
    }
    Ok(())
}

/// Ordner oder .mazext-Datei in einen Zwischenordner holen und prüfen
pub fn stage(source: &Path) -> Result<Staged, String> {
    let root = root();
    let staging = root.join(format!(".staging-{}", crate::sys::now_secs()));
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging).map_err(|e| e.to_string())?;
    let res = (|| {
        if source.is_dir() {
            let mut budget = 200 * 1024 * 1024;
            copy_dir(source, &staging, &mut budget)?;
        } else {
            let meta = std::fs::metadata(source).map_err(|e| e.to_string())?;
            if meta.len() > 200 * 1024 * 1024 {
                return Err(crate::i18n::t("ext.err.too_big"));
            }
            let data = std::fs::read(source).map_err(|e| e.to_string())?;
            let files = zipread::extract(&data, &staging).map_err(|e| match e {
                zipread::ZipError::Unsafe(p) => crate::i18n::tf("ext.err.zip_unsafe", &[("path", &p)]),
                zipread::ZipError::TooBig => crate::i18n::t("ext.err.too_big"),
                _ => crate::i18n::t("ext.err.zip"),
            })?;
            // Ausführbar machen (ZIP speichert die Rechte hier nicht)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(text) = std::fs::read_to_string(staging.join("manifest.json")) {
                    if let Ok(m) = manifest::parse(&text) {
                        if let Some(run) = m.run_for_os().and_then(|r| manifest::safe_relative(&r)) {
                            let _ = std::fs::set_permissions(staging.join(run), std::fs::Permissions::from_mode(0o755));
                        }
                    }
                }
            }
            if files.is_empty() {
                return Err(crate::i18n::t("ext.err.zip"));
            }
        }
        let (m, run) = manifest::load_dir(&staging).map_err(|e| e.explain())?;
        let hash = folder_hash(&staging)?;
        Ok(Staged { update: root.join(&m.id).exists(), run_hash: file_hash(&run), manifest: m, hash, dir: staging.clone() })
    })();
    if res.is_err() {
        let _ = std::fs::remove_dir_all(&staging);
    }
    res
}

/// Nach Zustimmung: in den Erweiterungsordner verschieben und Zustimmung (Hash) merken
pub fn commit(staged: &Staged) -> Result<(String, ExtState), String> {
    let id = staged.manifest.id.clone();
    if !manifest::valid_id(&id) {
        return Err(crate::i18n::t("ext.err.id"));
    }
    if let Some(e) = get(&id) {
        e.stop();
    }
    let dest = root().join(&id);
    if dest.exists() {
        std::fs::remove_dir_all(&dest).map_err(|e| e.to_string())?;
    }
    std::fs::rename(&staged.dir, &dest).map_err(|e| e.to_string())?;
    // Nach dem Verschieben erneut prüfen: genau das, was der Nutzer gesehen hat?
    if folder_hash(&dest)? != staged.hash {
        let _ = std::fs::remove_dir_all(&dest);
        return Err(crate::i18n::t("ext.err.changed"));
    }
    Ok((
        id,
        ExtState {
            enabled: true,
            hash: staged.hash.clone(),
            approved_schemes: staged.manifest.permissions.uri_schemes.clone(),
            network: staged.manifest.permissions.network,
            installed: crate::sys::now_secs(),
        },
    ))
}

pub fn discard(staged: &Staged) {
    if staged.dir.starts_with(root()) {
        let _ = std::fs::remove_dir_all(&staged.dir);
    }
}

/// Erweiterung löschen (nur innerhalb des Erweiterungsordners)
pub fn remove(id: &str) -> Result<(), String> {
    if !manifest::valid_id(id) {
        return Err(crate::i18n::t("ext.err.id"));
    }
    if let Some(e) = get(id) {
        e.stop();
    }
    let dir = root().join(id);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Aktuellen Stand eines (manuell kopierten oder geänderten) Ordners für die Zustimmung vorbereiten
pub fn review(id: &str) -> Result<Staged, String> {
    let dir = root().join(id);
    let (m, run) = manifest::load_dir(&dir).map_err(|e| e.explain())?;
    Ok(Staged { hash: folder_hash(&dir)?, run_hash: file_hash(&run), manifest: m, dir, update: true })
}

/// Zusammenfassung für die Einstellungen
#[derive(Clone, Debug, PartialEq)]
pub struct Summary {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub status: Status,
    pub dir: String,
}

pub fn summaries() -> Vec<Summary> {
    all()
        .iter()
        .map(|e| Summary {
            id: e.id.clone(),
            name: e.name(),
            version: e.manifest.as_ref().map(|m| m.version.clone()).unwrap_or_default(),
            author: e.manifest.as_ref().map(|m| m.author.clone()).unwrap_or_default(),
            description: e.manifest.as_ref().map(|m| e.text(&m.description)).unwrap_or_default(),
            status: e.status(),
            dir: e.dir.to_string_lossy().to_string(),
        })
        .collect()
}

/// Für Tests: einen Eintrag direkt anlegen
#[cfg(test)]
fn test_loaded(dir: &Path, schemes: Vec<String>) -> Loaded {
    Loaded {
        idx: 0,
        id: "test-ext".into(),
        dir: dir.to_path_buf(),
        manifest: None,
        run: None,
        hash: String::new(),
        status: Mutex::new(Status::Active),
        proc: Mutex::new(None),
        restarts: AtomicU8::new(0),
        approved_schemes: schemes,
        network: false,
        texts: HashMap::from([("hi".to_string(), "Hallo".to_string())]),
        texts_en: HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_validation() {
        let base = std::env::temp_dir().join(format!("maz-extg-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("game")).unwrap();
        std::fs::write(base.join("game").join("run.exe"), b"x").unwrap();
        std::fs::write(base.join("outside.exe"), b"x").unwrap();
        let e = test_loaded(&base, vec!["mystore".into()]);
        let dir = base.join("game").to_string_lossy().to_string();
        let ok = json!({"id":"g1","name":"Spiel","install_dir":dir,"launch":{"exe":{"path":base.join("game/run.exe"),"args":["-x"]}}});
        let g = parse_game(&e, &ok, true).unwrap();
        assert_eq!(g.id, "ext:test-ext:g1");
        let outside = json!({"id":"g2","name":"Böse","install_dir":dir,"launch":{"exe":{"path":base.join("outside.exe")}}});
        assert!(parse_game(&e, &outside, true).is_none(), "exe außerhalb von install_dir");
        let bad_uri = json!({"id":"g3","name":"x","launch":{"uri":"file:///etc/passwd"}});
        assert!(parse_game(&e, &bad_uri, true).is_none(), "verbotenes Schema");
        let own_uri = json!({"id":"g4","name":"x","launch":{"uri":"mystore://play/1"}});
        assert!(parse_game(&e, &own_uri, true).is_some(), "bestätigtes Schema");
        let other_uri = json!({"id":"g5","name":"x","launch":{"uri":"otherstore://play/1"}});
        assert!(parse_game(&e, &other_uri, true).is_none());
        let bad_id = json!({"id":"../x","name":"x","launch":{"uri":"https://a.b"}});
        assert!(parse_game(&e, &bad_id, true).is_none());
        let img = json!({"id":"g6","name":"x","launch":{"uri":"https://a.b"},"cover":"https://x.y/c.exe","icon":"../../etc/x.png"});
        let g6 = parse_game(&e, &img, true).unwrap();
        assert!(g6.art.cover.is_empty() && g6.art.icon.is_empty());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn actions_validated() {
        let base = std::env::temp_dir().join(format!("maz-exta-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("game/saves")).unwrap();
        let e = test_loaded(&base, vec![]);
        let mut g = Game {
            id: "ext:test-ext:g".into(),
            name: "G".into(),
            source: Source::Extension(0),
            installed: true,
            install_dir: Some(base.join("game").to_string_lossy().to_string()),
            exe: None,
            launch: LaunchTarget::Custom,
            art: Art::default(),
            size_bytes: None,
            playtime_min: None,
            steam_appid: None,
        };
        let ok = json!({"type":"open_folder","path":base.join("game/saves")});
        assert!(matches!(validate_action(&e, &ok, &g), Ok(ActionResult::OpenFolder(_))));
        let bad = json!({"type":"open_folder","path":std::env::temp_dir()});
        assert!(validate_action(&e, &bad, &g).is_err());
        assert!(validate_action(&e, &json!({"type":"open_url","url":"https://example.com"}), &g).is_ok());
        assert!(validate_action(&e, &json!({"type":"open_url","url":"file:///etc"}), &g).is_err());
        assert!(validate_action(&e, &json!({"type":"run","cmd":"rm -rf /"}), &g).is_err());
        assert_eq!(validate_action(&e, &json!({"type":"message","text":"@hi"}), &g), Ok(ActionResult::Message("Hallo".into())));
        g.install_dir = None;
        assert!(validate_action(&e, &ok, &g).is_err());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn hash_pinning() {
        let d = std::env::temp_dir().join(format!("maz-exth-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("sub")).unwrap();
        std::fs::write(d.join("a"), b"1").unwrap();
        std::fs::write(d.join("sub/b"), b"2").unwrap();
        let h1 = folder_hash(&d).unwrap();
        assert_eq!(h1, folder_hash(&d).unwrap());
        std::fs::write(d.join("sub/b"), b"3").unwrap();
        assert_ne!(h1, folder_hash(&d).unwrap(), "geänderte Datei → anderer Hash");
        std::fs::write(d.join("sub/b"), b"2").unwrap();
        std::fs::write(d.join("neu"), b"").unwrap();
        assert_ne!(h1, folder_hash(&d).unwrap(), "neue Datei → anderer Hash");
        let _ = std::fs::remove_dir_all(&d);
    }

    /// Integrationstest: Beispiel-Erweiterung bauen, starten und ihre Spiele laden.
    /// Wird übersprungen, wenn Cargo/Internet fehlt.
    #[test]
    fn example_extension_end_to_end() {
        let Ok(cargo) = std::env::var("CARGO") else { return };
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let target = std::env::temp_dir().join("maz-ext-example-target");
        let ok = std::process::Command::new(&cargo)
            .args(["build", "--quiet", "--manifest-path"])
            .arg(root.join("examples/extension-hello/Cargo.toml"))
            .arg("--target-dir")
            .arg(&target)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !ok {
            eprintln!("Beispiel-Erweiterung konnte nicht gebaut werden – Test übersprungen");
            return;
        }
        let exe = target.join("debug").join(if cfg!(windows) { "extension-hello.exe" } else { "extension-hello" });
        let dir = root.join("examples/extension-hello");
        let mut p = ExtProcess::spawn(&exe, &dir, false).expect("start");
        let hello = p.call("hello", json!({"api_version":1,"os":manifest::os_key(),"lang":"de","network":false}), TIMEOUT).expect("hello");
        assert_eq!(hello["name"], "Hello Extension");
        let games = p.call("games", json!({}), TIMEOUT).expect("games");
        let e = test_loaded(&dir, vec![]);
        let list: Vec<Game> = games["games"].as_array().unwrap().iter().filter_map(|g| parse_game(&e, g, true)).collect();
        assert_eq!(list.len(), 2);
        let meta = p.call("metadata", json!({"name":"Hello Game One"}), TIMEOUT).expect("metadata");
        assert!(meta["description"].as_str().unwrap().len() > 5);
        let acts = p.call("actions", json!({"game": game_json(&list[0])}), TIMEOUT).expect("actions");
        assert_eq!(acts["actions"][0]["id"], "open-folder");
        p.stop();
    }
}
