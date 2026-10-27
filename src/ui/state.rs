//! Globaler Zustand der Oberfläche + alle Aktionen (Spielen, Favorit, Neu laden …).

use crate::model::{normalize_name, Game, LaunchTarget, Settings, Source, StoreStatus};
use crate::problem::{Fix, Problem};
use crate::i18n::{t, tf};
use crate::{config, launch, library, sys};
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Filter {
    All,
    Installed,
    Steam,
    Epic,
    Roblox,
    Custom,
    Favorites,
    Recent,
    Hidden,
    /// Weitere Stores und Emulatoren
    Store(Source),
    /// Eigener Tag (Name steht in `App::filter_arg`)
    Tag,
    /// Eigene Sammlung (ID steht in `App::filter_arg`)
    Collection,
    /// Spielstatus (Kennung steht in `App::filter_arg`)
    Status,
    /// ROM-Plattform (Name steht in `App::filter_arg`)
    Platform,
}

impl Filter {
    pub fn label(self) -> String {
        match self {
            Filter::Store(src) => src.label(),
            _ => t(match self {
                Filter::All => "filter.all",
                Filter::Installed => "filter.installed",
                Filter::Steam => "filter.steam",
                Filter::Epic => "filter.epic",
                Filter::Roblox => "filter.roblox",
                Filter::Custom => "filter.custom",
                Filter::Favorites => "filter.favorites",
                Filter::Recent => "filter.recent",
                Filter::Hidden => "filter.hidden",
                Filter::Tag => "filter.tag",
                Filter::Collection | Filter::Store(_) => "filter.collection",
                Filter::Status => "filter.status",
                Filter::Platform => "filter.platform",
            }),
        }
    }
    pub fn icon(self) -> &'static str {
        match self {
            Filter::All => "library",
            Filter::Installed => "drive",
            Filter::Steam => "gamepad",
            Filter::Epic => "layers",
            Filter::Roblox => "cube",
            Filter::Custom => "rocket",
            Filter::Favorites => "star",
            Filter::Recent => "clock",
            Filter::Hidden => "eye-off",
            Filter::Store(src) => src.icon(),
            Filter::Tag => "tag",
            Filter::Collection => "folder",
            Filter::Status => "flag",
            Filter::Platform => "joystick",
        }
    }
    /// Passt das Spiel zu diesem Filter? (versteckte Spiele prüft der Aufrufer)
    pub fn matches(self, arg: &str, g: &Game, s: &Settings) -> bool {
        match self {
            Filter::All | Filter::Hidden => true,
            Filter::Installed => g.installed,
            Filter::Steam => g.source == Source::Steam,
            Filter::Epic => g.source == Source::Epic,
            Filter::Roblox => g.source == Source::Roblox,
            Filter::Custom => g.source == Source::Custom,
            Filter::Store(src) => g.source == src,
            Filter::Favorites => s.is_favorite(&g.id),
            Filter::Recent => s.last_played(&g.id) > 0,
            Filter::Tag => s.tags.get(&g.id).map(|t| t.iter().any(|x| x.eq_ignore_ascii_case(arg))).unwrap_or(false),
            Filter::Collection => s.in_collection(arg, &g.id),
            Filter::Status => s.status_of(&g.id) == arg,
            Filter::Platform => crate::emulators::platform(g, &s.emulators).as_deref() == Some(arg),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsTab {
    Design,
    Language,
    Connections,
    Custom,
    Extensions,
    General,
    About,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    Library,
    Free,
    /// Minecraft-Inhalte (Mods, Texturen, Shader, Datenpakete)
    Minecraft,
    /// Statistik: Spielzeit, meistgespielt, ungespielt
    Stats,
    /// Vollbild mit großen Kacheln (TV, Sofa)
    BigPicture,
    Settings(SettingsTab),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Modal {
    /// Sicherheitsfrage vor dem Start eines eigenen Spiels
    Confirm(String),
    /// Nutzer hat "Nein" gesagt
    NotStarted(String),
    /// Fehler mit Erklärung (+ optional Spiel-ID für Aktionen)
    Problem(Problem, Option<String>),
    /// Eigenes Spiel hinzufügen (None) oder bearbeiten
    EditGame(Option<String>),
    DeleteGame(String),
    Detail(String),
    Welcome,
    /// Einfacher Hinweis (Titel, Text)
    Info(String, String),
    /// Neue Version verfügbar
    Update,
    /// Spiel-Infos bearbeiten (Name, Bilder, Beschreibung, Genres)
    EditMeta(String),
    /// Ordner nach Spielen durchsuchen
    FolderScan,
    /// Epic-Konto verbinden (Schritt für Schritt)
    EpicConnect,
    /// Erweiterung installieren/bestätigen: Rechte anzeigen
    ExtInstall(crate::extensions::Staged),
    /// Meldung einer Erweiterungs-Aktion (Titel = Erweiterung, Text)
    ExtMessage(String, String),
    /// „Was soll ich spielen?“ – Zufallsspiel
    Random,
    /// Sicherung wiederherstellen: Vorschau + Bestätigung
    Restore(Box<Settings>),
    /// Import aus Playnite
    PlayniteImport,
    /// Bilder von SteamGridDB auswählen (Spiel-ID)
    SgdbPick(String),
    /// Starthilfen eines Spiels (Spiel-ID)
    LaunchOpts(String),
    /// Spiel zu Steam hinzufügen (Spiel-ID)
    AddToSteam(String),
    /// Absturzbericht vom letzten Start (Text)
    CrashReport(String),
    /// Flatpak: alte App (alte App-ID) entfernen?
    OldFlatpak,
    /// Großansicht eines Screenshots (Spiel-ID, Index)
    Screenshot(String, usize),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Toast {
    pub id: u64,
    pub text: String,
    pub icon: &'static str,
}

#[derive(Clone, Copy)]
pub struct App {
    pub settings: Signal<Settings>,
    pub games: Signal<Vec<Game>>,
    pub steam: Signal<StoreStatus>,
    pub epic: Signal<StoreStatus>,
    pub roblox: Signal<StoreStatus>,
    pub stores: Signal<Vec<(Source, StoreStatus)>>,
    /// Automatisch geladene Spiel-Infos (Beschreibung, Genres)
    pub meta_cache: Signal<std::collections::BTreeMap<String, crate::metadata::Info>>,
    pub warnings: Signal<Vec<Problem>>,
    pub scanning: Signal<bool>,
    pub loaded: Signal<bool>,
    pub page: Signal<Page>,
    pub filter: Signal<Filter>,
    /// Tag-Name bzw. Sammlungs-ID für Filter::Tag / Filter::Collection
    pub filter_arg: Signal<String>,
    /// Genre-Filter (leer = alle)
    pub genre: Signal<String>,
    /// Weitere Versionen desselben Spiels (ID → andere Stores)
    pub dupes: Signal<std::collections::BTreeMap<String, Vec<Game>>>,
    /// Spiele, deren Spielzeit gerade gemessen wird
    pub tracking: Signal<Vec<String>>,
    pub search: Signal<String>,
    /// Vorauswahl für Minecraft-Inhalte ("official" oder "norisk:<id>")
    pub mc_target: Signal<String>,
    pub selected: Signal<Option<String>>,
    pub modal: Signal<Option<Modal>>,
    pub toasts: Signal<Vec<Toast>>,
    // Gratis-Spiele
    pub free: Signal<Vec<crate::freegames::FreeGame>>,
    pub free_state: Signal<LoadState>,
    // Updates
    pub update_avail: Signal<Option<crate::updater::Release>>,
    pub update_state: Signal<UpdateState>,
    // Epic-Konto
    pub epic_busy: Signal<bool>,
    pub epic_error: Signal<Option<String>>,
    /// Laden der Epic-Bibliothek: (fertig, gesamt)
    pub epic_progress: Signal<(usize, usize)>,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub enum UpdateState {
    #[default]
    Idle,
    Checking,
    /// Lädt: (geladen, gesamt) in Bytes
    Downloading(u64, u64),
    Installing,
    /// Fertig – mit dem Knopf neu starten (Pfad zur neuen Version)
    Ready(std::path::PathBuf),
    /// Paket installiert – Launcher bitte neu öffnen
    Done,
    /// Datei liegt bereit, Befehl muss selbst ausgeführt werden
    Manual(String, String),
    Failed(String),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum LoadState {
    #[default]
    Idle,
    Loading,
    Done,
    Failed,
}

pub fn use_app() -> App {
    use_context::<App>()
}

impl App {
    /// Einstellungen ändern und sofort speichern.
    pub fn update(&self, f: impl FnOnce(&mut Settings)) {
        let mut s = self.settings;
        f(&mut s.write());
        let snapshot = s.peek().clone();
        if let Err(e) = config::save(&snapshot) {
            self.toast(&tf("toast.save_failed", &[("error", &e.to_string())]), "alert");
        }
    }

    pub fn open(&self, m: Modal) {
        let mut modal = self.modal;
        modal.set(Some(m));
    }

    pub fn close_modal(&self) {
        let mut modal = self.modal;
        modal.set(None);
    }

    pub fn show_problem(&self, p: Problem, game_id: Option<String>) {
        self.open(Modal::Problem(p, game_id));
    }

    pub fn toast(&self, text: &str, icon: &'static str) {
        let id = sys::now_secs().wrapping_mul(1000) + (self.toasts.peek().len() as u64) + rand_small();
        let mut toasts = self.toasts;
        toasts.write().push(Toast { id, text: text.to_string(), icon });
        dioxus::core::spawn_forever(async move {
            tokio::time::sleep(std::time::Duration::from_millis(5000)).await;
            toasts.write().retain(|t| t.id != id);
        });
    }

    pub fn game(&self, id: &str) -> Option<Game> {
        self.games
            .peek()
            .iter()
            .find(|g| g.id == id)
            .cloned()
            .or_else(|| self.dupes.peek().values().flatten().find(|g| g.id == id).cloned())
    }

    /// Filter setzen (Tag/Sammlung mit Namen bzw. ID)
    pub fn set_filter(&self, f: Filter, arg: &str) {
        let mut fl = self.filter;
        let mut a = self.filter_arg;
        a.set(arg.to_string());
        fl.set(f);
        self.go(Page::Library);
    }

    /// Genres eines Spiels: selbst eingetragen oder automatisch geladen
    pub fn genres_of(&self, id: &str) -> Vec<String> {
        let s = self.settings.read();
        if let Some(m) = s.meta.get(id).filter(|m| !m.genres.is_empty()) {
            return m.genres.clone();
        }
        drop(s);
        self.meta_cache.read().get(id).map(|i| i.genres.clone()).unwrap_or_default()
    }

    pub fn go(&self, page: Page) {
        let mut p = self.page;
        p.set(page);
    }
}

fn rand_small() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64 % 997)
        .unwrap_or(0)
}

// ----------------------------------------------------------------------
// Bibliothek laden
// ----------------------------------------------------------------------

pub fn rescan(app: App) {
    if *app.scanning.peek() {
        return;
    }
    let mut scanning = app.scanning;
    scanning.set(true);
    let settings = app.settings.peek().clone();
    if crate::uitest::active() {
        let mut games = app.games;
        games.set(crate::uitest::demo_games());
        let mut loaded = app.loaded;
        loaded.set(true);
        scanning.set(false);
        // Nach kurzer Zeit im Fenster prüfen und das Ergebnis melden
        dioxus::core::spawn_forever(async move {
            tokio::time::sleep(std::time::Duration::from_secs(4)).await;
            let _ = dioxus::desktop::window().webview.evaluate_script(crate::uitest::CHECK_JS);
        });
        return;
    }
    dioxus::core::spawn_forever(async move {
        let s2 = settings.clone();
        let res = tokio::task::spawn_blocking(move || library::scan(&s2)).await;
        let mut games = app.games;
        let mut steam = app.steam;
        let mut epic = app.epic;
        let mut roblox = app.roblox;
        let mut stores = app.stores;
        let mut dupes = app.dupes;
        let mut warnings = app.warnings;
        let mut loaded = app.loaded;
        if let Ok(r) = res {
            games.set(r.games);
            steam.set(r.steam);
            epic.set(r.epic);
            roblox.set(r.roblox);
            stores.set(r.stores);
            dupes.set(r.duplicates);
            warnings.set(r.warnings);
            apply_meta_covers(app);
            // Neu aufgetauchte Spiele merken (für "Zuletzt hinzugefügt")
            let now = sys::now_secs();
            let new_ids: Vec<String> = games
                .peek()
                .iter()
                .filter(|g| !settings.first_seen.contains_key(&g.id))
                .map(|g| g.id.clone())
                .collect();
            if !new_ids.is_empty() {
                app.update(|s| {
                    for id in new_ids {
                        s.first_seen.entry(id).or_insert(now);
                    }
                });
            }
        }
        loaded.set(true);
        scanning.set(false);

        // Phase 2: schöne Cover für Epic- und eigene Spiele aus dem Steam-Store
        if settings.online_art {
            let names: Vec<String> = games
                .peek()
                .iter()
                .filter(|g| !matches!(g.source, Source::Steam | Source::Roblox))
                .map(|g| g.name.clone())
                .collect();
            let known = settings.art_lookup.clone();
            if let Ok(found) = tokio::task::spawn_blocking(move || library::lookup_missing_art(names, &known)).await {
                if !found.is_empty() {
                    app.update(|s| s.art_lookup.extend(found.clone()));
                    let lookup = app.settings.peek().art_lookup.clone();
                    for g in games.write().iter_mut() {
                        if g.source != Source::Steam && found.contains_key(&normalize_name(&g.name)) {
                            library::apply_lookup(g, &lookup);
                        }
                    }
                }
            }
            fill_metadata(app).await;
        }
        // Phase 3: fehlende Bilder von SteamGridDB (nur mit eigenem Schlüssel)
        let key = app.settings.peek().sgdb_key.clone();
        if !key.trim().is_empty() {
            let list = games.peek().clone();
            let res = tokio::task::spawn_blocking(move || crate::sgdb::fill(&list, &key, sys::now_secs(), 80)).await;
            if let Ok(Ok(true)) = res {
                for g in games.write().iter_mut() {
                    crate::sgdb::apply(g);
                }
            }
        }
    });
}

/// Steam-AppID für Spiel-Infos (eigene oder über den Namen gefunden). ROMs nie über Steam.
fn meta_appid(app: App, g: &Game) -> Option<u32> {
    if g.source == Source::Emulator {
        return None;
    }
    g.steam_appid.or_else(|| app.settings.peek().art_lookup.get(&normalize_name(&g.name)).copied().filter(|a| *a != 0))
}

/// Woher die Infos kommen: Steam-Store oder Wikipedia
#[derive(Clone)]
enum MetaJob {
    Steam(u32),
    Wiki(String),
}

fn meta_job(app: App, g: &Game) -> MetaJob {
    if g.source == Source::Minecraft {
        return MetaJob::Wiki("Minecraft".into());
    }
    match meta_appid(app, g) {
        Some(a) => MetaJob::Steam(a),
        None => MetaJob::Wiki(g.name.clone()),
    }
}

/// Erst Steam, wenn dort nichts steht: Wikipedia
fn run_meta_job(job: MetaJob, name: String, lang: String) -> Option<crate::metadata::Info> {
    match job {
        MetaJob::Steam(appid) => {
            let info = crate::metadata::fetch(appid, &lang)?;
            if info.fetched > 0 {
                Some(info)
            } else {
                crate::metadata::fetch_wiki(&name, &lang)
            }
        }
        MetaJob::Wiki(n) => crate::metadata::fetch_wiki(&n, &lang),
    }
}

/// Eingebaute Quellen zuerst, danach Erweiterungen (falls dort nichts gefunden wurde)
fn run_meta_with_ext(job: MetaJob, name: String, id: String, lang: String) -> Option<crate::metadata::Info> {
    let info = run_meta_job(job, name.clone(), lang);
    match info {
        Some(i) if i.fetched > 0 => Some(i),
        other => crate::extensions::metadata(&name, &id).or(other),
    }
}

/// Bild aus den Spiel-Infos übernehmen, wenn das Spiel sonst kein Cover hat (z. B. ROMs)
pub fn apply_meta_covers(app: App) {
    let cache = app.meta_cache.peek().clone();
    let mut games = app.games;
    for g in games.write().iter_mut() {
        if g.art.cover.is_empty() {
            if let Some(i) = cache.get(&g.id).filter(|i| i.cover.starts_with("https://upload.wikimedia.org/")) {
                g.art.cover.push(i.cover.clone());
            }
        }
    }
}

fn store_info(app: App, id: String, info: crate::metadata::Info) {
    let has_cover = !info.cover.is_empty();
    let mut cache = app.meta_cache;
    cache.write().insert(id, info);
    crate::metadata::save_cache(&cache.peek());
    if has_cover {
        apply_meta_covers(app);
    }
}

/// Fehlende Spiel-Infos langsam im Hintergrund laden (höchstens 30 pro Durchgang)
async fn fill_metadata(app: App) {
    let lang = crate::i18n::current_code();
    let todo: Vec<(String, String, MetaJob)> = {
        let cache = app.meta_cache.peek();
        app.games
            .peek()
            .iter()
            .filter(|g| cache.get(&g.id).map(|i| !i.is_current()).unwrap_or(true))
            .map(|g| (g.id.clone(), g.name.clone(), meta_job(app, g)))
            .take(30)
            .collect()
    };
    for (id, name, job) in todo {
        let l = lang.clone();
        // Keine Verbindung → aufhören, beim nächsten Mal weiter
        let gid = id.clone();
        let Ok(Some(info)) = tokio::task::spawn_blocking(move || run_meta_with_ext(job, name, gid, l)).await else { break };
        store_info(app, id, info);
        tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
    }
}

/// Infos für ein Spiel sofort laden (beim Öffnen der Details)
pub fn load_metadata(app: App, id: String) {
    if !app.settings.peek().online_art || app.meta_cache.peek().get(&id).map(|i| i.is_current()).unwrap_or(false) {
        return;
    }
    let Some(g) = app.game(&id) else { return };
    let job = meta_job(app, &g);
    let name = g.name.clone();
    let lang = crate::i18n::current_code();
    dioxus::core::spawn_forever(async move {
        let gid = id.clone();
        if let Ok(Some(info)) = tokio::task::spawn_blocking(move || run_meta_with_ext(job, name, gid, lang)).await {
            store_info(app, id, info);
        }
    });
}

/// Bibliothek nach ein paar Sekunden erneut einlesen (z. B. nach Installation oder Gratis-Spiel)
pub fn rescan_later(app: App, delays_secs: &'static [u64]) {
    dioxus::core::spawn_forever(async move {
        let mut waited = 0;
        for d in delays_secs {
            tokio::time::sleep(std::time::Duration::from_secs(d - waited)).await;
            waited = *d;
            rescan(app);
        }
    });
}

/// Ein eigenes Spiel nach dem Hinzufügen/Bearbeiten neu in die Liste übernehmen.
pub fn refresh_custom(app: App, id: String) {
    let settings = app.settings.peek().clone();
    let Some(c) = settings.custom(&id).cloned() else {
        let mut games = app.games;
        games.write().retain(|g| g.id != id);
        return;
    };
    // Sofort anzeigen (ohne Icon), dann im Hintergrund Icon + Cover holen
    let quick = library::custom_game(&c, &settings.art_lookup, false);
    {
        let mut games = app.games;
        let mut w = games.write();
        match w.iter_mut().find(|g| g.id == id) {
            Some(g) => *g = quick,
            None => w.push(quick),
        }
    }
    let online = settings.online_art;
    dioxus::core::spawn_forever(async move {
        let name = c.name.clone();
        let known = settings.art_lookup.clone();
        let res = tokio::task::spawn_blocking(move || {
            let found = if online {
                library::lookup_missing_art(vec![name], &known)
            } else {
                Default::default()
            };
            let mut lookup = known.clone();
            lookup.extend(found.clone());
            (library::custom_game(&c, &lookup, true), found)
        })
        .await;
        if let Ok((game, found)) = res {
            if !found.is_empty() {
                app.update(|s| s.art_lookup.extend(found));
            }
            let mut games = app.games;
            let mut list = games.write();
            if let Some(g) = list.iter_mut().find(|g| g.id == game.id) {
                *g = game;
            }
            drop(list);
        }
    });
}

// ----------------------------------------------------------------------
// Spielen
// ----------------------------------------------------------------------

fn mark_played(app: App, id: &str) {
    let id = id.to_string();
    app.update(|s| {
        s.last_played.insert(id.clone(), sys::now_secs());
        *s.launch_count.entry(id).or_insert(0) += 1;
    });
}

fn minimize_window() {
    dioxus::desktop::window().window.set_minimized(true);
}

fn restore_window() {
    let w = dioxus::desktop::window();
    w.window.set_minimized(false);
    w.window.set_focus();
}

/// Woran die Spielzeit-Messung das laufende Spiel erkennt
fn matcher_for(game: &Game) -> Option<crate::playtime::Matcher> {
    use crate::playtime::Matcher;
    match (&game.launch, game.source) {
        (LaunchTarget::Roblox(_), _) => Some(Matcher::Name("robloxplayer".into())),
        (LaunchTarget::Exe { path, .. }, Source::Emulator) => Some(Matcher::Exe(path.clone())),
        _ => game.install_dir.clone().filter(|d| d.len() > 3).map(Matcher::Dir),
    }
}

/// Spielzeit im Hintergrund messen und speichern
pub fn track_playtime(app: App, game: &Game) {
    let Some(m) = matcher_for(game) else { return };
    let id = game.id.clone();
    if app.tracking.peek().contains(&id) {
        return;
    }
    let mut tr = app.tracking;
    tr.write().push(id.clone());
    dioxus::core::spawn_forever(async move {
        let minutes = tokio::task::spawn_blocking(move || crate::playtime::track(m)).await.unwrap_or(0);
        let mut tr = app.tracking;
        tr.write().retain(|x| *x != id);
        crate::discord::clear();
        // Starthilfe: Programm nach dem Spiel
        let post = launch_opts(app, &id).post;
        if minutes > 0 && !post.trim().is_empty() {
            if let Err(e) = crate::launchopts::run_helper(&post) {
                app.toast(&e, "alert");
            }
        }
        if minutes > 0 {
            let start = sys::now_secs().saturating_sub(minutes * 60);
            app.update(|s| {
                *s.playtime_local.entry(id.clone()).or_insert(0) += minutes;
                s.add_session(&id, start, minutes);
            });
        }
    });
}

/// Discord-Status setzen (wenn eingeschaltet)
fn discord_playing(app: App, name: &str) {
    let s = app.settings.peek();
    if s.discord_enabled {
        crate::discord::set_playing(&s.discord_app_id, name);
    }
}

/// Starthilfen eines Spiels (Standard, wenn nichts eingestellt ist)
pub fn launch_opts(app: App, id: &str) -> crate::launchopts::LaunchOpts {
    app.settings.peek().launch_opts.get(id).cloned().unwrap_or_default()
}

/// Starthilfe: Programm vor dem Spiel
fn run_pre(app: App, opts: &crate::launchopts::LaunchOpts) {
    if !opts.pre.trim().is_empty() {
        if let Err(e) = crate::launchopts::run_helper(&opts.pre) {
            app.toast(&e, "alert");
        }
    }
}

/// "Spielen" wurde geklickt.
pub fn play(app: App, id: String) {
    let Some(game) = app.game(&id) else { return };
    if game.source == Source::Custom {
        let Some(c) = app.settings.peek().custom(&id).cloned() else { return };
        match launch::precheck_custom(&c) {
            // Vor JEDEM Start eines eigenen Spiels: Sicherheitsfrage
            Ok(()) => app.open(Modal::Confirm(id)),
            Err(p) => app.show_problem(p, Some(id)),
        }
        return;
    }
    // Programme aus Erweiterungen: wie eigene Spiele vorher die Sicherheitsfrage
    if matches!(game.source, Source::Extension(_)) && matches!(game.launch, LaunchTarget::Exe { .. }) {
        app.open(Modal::Confirm(id));
        return;
    }
    start_store_game(app, game);
}

/// Store-/Erweiterungs-Spiel wirklich starten
fn start_store_game(app: App, game: Game) {
    let id = game.id.clone();
    let opts = launch_opts(app, &id);
    // Beim Installieren keine Starthilfen
    if game.installed && !matches!(game.launch, LaunchTarget::SteamInstall(_) | LaunchTarget::EpicInstall(_)) {
        run_pre(app, &opts);
    }
    match launch::launch_store(&game, &opts) {
        Ok(()) => {
            let installing = !game.installed || matches!(game.launch, LaunchTarget::SteamInstall(_) | LaunchTarget::EpicInstall(_));
            let store = game.source.label();
            if installing {
                app.toast(&tf("toast.installing", &[("name", &game.name), ("store", &store)]), "download");
                // Nach der Installation taucht das Spiel automatisch als installiert auf
                rescan_later(app, &[90, 300, 900]);
                return;
            }
            mark_played(app, &id);
            discord_playing(app, &game.name);
            let text = if game.source == Source::Emulator {
                tf("toast.started", &[("name", &game.name)])
            } else {
                tf("toast.started_via", &[("name", &game.name), ("store", &store)])
            };
            app.toast(&text, "rocket");
            if app.settings.peek().minimize_on_launch {
                minimize_window();
            }
            track_playtime(app, &game);
            // Stürzt das Spiel direkt ab? Dann erklären, was los ist.
            if matches!(game.source, Source::Steam | Source::Epic) {
                let g = game.clone();
                let gid = id.clone();
                dioxus::core::spawn_forever(async move {
                    let res = tokio::task::spawn_blocking(move || launch::watch_store(&g)).await;
                    if let Ok(Some(p)) = res {
                        restore_window();
                        app.show_problem(p, Some(gid));
                    }
                });
            }
        }
        Err(p) => app.show_problem(p, Some(id)),
    }
}

/// Nach "Ja, alles deaktiviert" bei einem Erweiterungs-Spiel
pub fn start_confirmed(app: App, id: String) {
    app.close_modal();
    if app.settings.peek().custom(&id).is_some() {
        start_custom(app, id, false);
        return;
    }
    if let Some(g) = app.game(&id) {
        start_store_game(app, g);
    }
}

/// Nach "Ja, alles deaktiviert" – eigenes Spiel wirklich starten.
pub fn start_custom(app: App, id: String, force_admin: bool) {
    app.close_modal();
    let Some(c) = app.settings.peek().custom(&id).cloned() else { return };
    let opts = launch_opts(app, &id);
    run_pre(app, &opts);
    match launch::launch_custom(&c, force_admin, &opts) {
        Ok(child) => {
            mark_played(app, &id);
            discord_playing(app, &c.name);
            if let Some(g) = app.game(&id) {
                track_playtime(app, &g);
            }
            app.toast(&tf("toast.started", &[("name", &c.name)]), "rocket");
            if app.settings.peek().minimize_on_launch {
                minimize_window();
            }
            if let Some(child) = child {
                let name = c.name.clone();
                let exe = c.exe.clone();
                dioxus::core::spawn_forever(async move {
                    let res = tokio::task::spawn_blocking(move || launch::watch(child, name, exe)).await;
                    if let Ok(Some(p)) = res {
                        restore_window();
                        app.show_problem(p, Some(id));
                    }
                });
            }
        }
        Err(p) => app.show_problem(p, Some(id)),
    }
}

/// Knopf in der Fehlererklärung wurde geklickt.
pub fn run_fix(app: App, fix: Fix, game_id: Option<String>) {
    match fix {
        Fix::RunAsAdmin => {
            if let Some(id) = game_id {
                let is_custom = app.game(&id).map(|g| g.source == Source::Custom).unwrap_or(false);
                if is_custom {
                    start_custom(app, id, true);
                } else {
                    app.close_modal();
                    play(app, id);
                }
            }
        }
        Fix::EditGame => {
            if let Some(id) = game_id {
                if app.settings.peek().custom(&id).is_some() {
                    app.open(Modal::EditGame(Some(id)));
                }
            }
        }
        Fix::OpenUrl { url, .. } => {
            let _ = sys::open_url(&url);
        }
        Fix::OpenFolder { path, .. } => sys::open_folder(&path),
        Fix::Rescan => {
            app.close_modal();
            rescan(app);
        }
        Fix::OpenSettings => {
            app.close_modal();
            app.go(Page::Settings(SettingsTab::Connections));
        }
        Fix::RemoveGame => {
            if let Some(id) = game_id {
                if app.settings.peek().custom(&id).is_some() {
                    app.open(Modal::DeleteGame(id));
                }
            }
        }
    }
}

/// Roblox-Experience hinzufügen (Link oder Place-ID). Name und Bilder werden online geladen.
pub fn add_roblox(app: App, input: &str) -> bool {
    let Some(place) = crate::roblox::place_id_from(input) else {
        app.toast(&t("roblox.invalid"), "alert");
        return false;
    };
    if app.settings.peek().roblox_games.iter().any(|g| g.place_id == place) {
        app.toast(&t("roblox.exists"), "info");
        return false;
    }
    app.update(|s| {
        s.roblox_enabled = true;
        s.roblox_games.push(crate::model::RobloxGame { place_id: place, added: sys::now_secs(), ..Default::default() });
    });
    rescan(app);
    dioxus::core::spawn_forever(async move {
        if let Ok(info) = tokio::task::spawn_blocking(move || crate::roblox::fetch_info(place)).await {
            let name = info.name.clone();
            app.update(|s| {
                if let Some(g) = s.roblox_games.iter_mut().find(|g| g.place_id == place) {
                    g.name = info.name;
                    g.icon = info.icon;
                    g.thumb = info.thumb;
                }
            });
            let shown = if name.is_empty() { format!("Roblox {place}") } else { name };
            app.toast(&tf("roblox.added", &[("name", &shown)]), "check-circle");
            rescan(app);
        }
    });
    true
}

pub fn remove_roblox(app: App, id: &str) {
    let Some(place) = id.strip_prefix("roblox:").and_then(|p| p.parse::<u64>().ok()) else { return };
    app.update(|s| s.roblox_games.retain(|g| g.place_id != place));
    app.close_modal();
    rescan(app);
}

/// "Spiel startet nicht?" – Hilfe mit allen typischen Lösungen
pub fn start_help(app: App, id: String) {
    let Some(game) = app.game(&id) else { return };
    let custom = game.source == Source::Custom;
    let dir = game
        .install_dir
        .clone()
        .or_else(|| app.settings.peek().custom(&id).and_then(|c| std::path::Path::new(c.exe.trim().trim_matches('"')).parent().map(|p| p.to_string_lossy().to_string())));
    let p = Problem::start_help(&game.name, &game.source.label(), launch::verify_fix(&game), dir, custom);
    app.show_problem(p, Some(id));
}

/// Sicherheitsfrage vor dem Deinstallieren eines eigenen Spiels
fn confirm_uninstall(game: &str, u: &launch::Uninstaller) -> bool {
    {
        use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
        let text = tf(
            "uninstall.custom_confirm",
            &[("game", game), ("name", if u.display_name.is_empty() { game } else { u.display_name.as_str() }), ("program", &u.program)],
        );
        MessageDialog::new()
            .set_level(MessageLevel::Warning)
            .set_title(t("act.uninstall"))
            .set_description(text)
            .set_buttons(MessageButtons::YesNo)
            .show()
            == MessageDialogResult::Yes
    }
}

/// Spiel deinstallieren
pub fn uninstall(app: App, id: String) {
    let Some(game) = app.game(&id) else { return };
    match &game.launch {
        LaunchTarget::SteamRun(appid) => {
            // Steam fragt selbst noch einmal nach
            if sys::open_url(&format!("steam://uninstall/{appid}")).is_ok() {
                app.close_modal();
                app.toast(&tf("toast.uninstall_store", &[("name", &game.name), ("store", "Steam")]), "trash");
                rescan_later(app, &[20, 60, 180]);
            } else {
                app.show_problem(Problem::steam_missing(), Some(id));
            }
        }
        LaunchTarget::Epic(uri) => {
            // Linux (Heroic): Heroic selbst öffnen, sonst den Epic Games Launcher
            if uri.starts_with("heroic://") {
                let _ = sys::host_command("heroic").spawn();
            } else {
                let _ = sys::open_url("com.epicgames.launcher://");
            }
            app.show_problem(Problem::epic_uninstall(&game.name), Some(id));
            rescan_later(app, &[60, 180, 600]);
        }
        LaunchTarget::Custom => {
            let Some(c) = app.settings.peek().custom(&id).cloned() else { return };
            let dir = std::path::Path::new(c.exe.trim().trim_matches('"')).parent().map(|p| p.to_string_lossy().to_string());
            match launch::find_uninstaller(&c) {
                Some(u) => {
                    // Vor dem Start genau zeigen, WAS deinstalliert wird
                    if !confirm_uninstall(&c.name, &u) {
                        return;
                    }
                    #[cfg(windows)]
                    let res = crate::winffi::shell_execute(Some("open"), &u.program, Some(&u.args), None);
                    #[cfg(not(windows))]
                    let res: Result<(), i32> = Err(0);
                    match res {
                        Ok(()) => {
                            app.close_modal();
                            app.toast(&tf("toast.uninstall_custom", &[("name", &c.name)]), "trash");
                        }
                        Err(_) => app.show_problem(Problem::no_uninstaller(&c.name, dir), Some(id)),
                    }
                }
                None => app.show_problem(Problem::no_uninstaller(&c.name, dir), Some(id)),
            }
        }
        LaunchTarget::Roblox(_) => {}
        _ => uninstall_other(app, &game),
    }
}

/// GOG, EA, Ubisoft, Battle.net, Amazon, itch.io, Xbox, Emulator-ROMs
fn uninstall_other(app: App, game: &Game) {
    let id = game.id.clone();
    // Microsoft Store / Xbox: Windows-Einstellungen → Apps
    if game.source == Source::Xbox {
        let _ = sys::open_url("ms-settings:appsfeatures");
        app.show_problem(Problem::store_uninstall(&game.name, &game.source.label()), Some(id));
        return;
    }
    // ROMs gehören zu keinem Store – nur den Ordner zeigen
    if game.source == Source::Emulator {
        app.show_problem(Problem::store_uninstall(&game.name, &t("source.emulator")), Some(id));
        return;
    }
    let dir = game.install_dir.clone().unwrap_or_default();
    let entry = crate::stores::uninstall_entries().into_iter().find(|e| {
        launch::specific_dir(&dir) && sys::same_path(&e.install_location, &dir)
    });
    match entry {
        Some(e) if !e.uninstall_string.trim().is_empty() => {
            let (program, args) = launch::split_command(&e.uninstall_string);
            let u = launch::Uninstaller { display_name: e.display_name.clone(), program, args };
            if !confirm_uninstall(&game.name, &u) {
                return;
            }
            #[cfg(windows)]
            let res = crate::winffi::shell_execute(Some("open"), &u.program, Some(&u.args), None);
            #[cfg(not(windows))]
            let res: Result<(), i32> = Err(0);
            match res {
                Ok(()) => {
                    app.close_modal();
                    app.toast(&tf("toast.uninstall_custom", &[("name", &game.name)]), "trash");
                    rescan_later(app, &[30, 120, 600]);
                }
                Err(_) => app.show_problem(Problem::store_uninstall(&game.name, &game.source.label()), Some(id)),
            }
        }
        // Kein Eintrag gefunden → Store-Programm öffnen und erklären
        _ => {
            if let Some(lib) = crate::stores::library(game.source) {
                if let Some(uri) = lib.client_uri() {
                    let _ = sys::open_url(uri);
                }
            }
            app.show_problem(Problem::store_uninstall(&game.name, &game.source.label()), Some(id));
        }
    }
}

pub fn toggle_favorite(app: App, id: &str) {
    let id = id.to_string();
    app.update(|s| {
        if let Some(pos) = s.favorites.iter().position(|f| *f == id) {
            s.favorites.remove(pos);
        } else {
            s.favorites.push(id);
        }
    });
}

pub fn toggle_hidden(app: App, id: &str) {
    let id = id.to_string();
    let mut hide = false;
    app.update(|s| {
        if let Some(pos) = s.hidden.iter().position(|f| *f == id) {
            s.hidden.remove(pos);
        } else {
            s.hidden.push(id);
            hide = true;
        }
    });
    app.toast(&t(if hide { "toast.hidden" } else { "toast.unhidden" }), "eye-off");
}

// ----------------------------------------------------------------------
// Tags und Sammlungen
// ----------------------------------------------------------------------

/// Neue Sammlung anlegen, gibt die ID zurück
pub fn create_collection(app: App, name: &str) -> String {
    let id = format!("c{:x}", sys::now_secs().wrapping_mul(1000) + rand_small());
    let name: String = name.trim().chars().take(40).collect();
    let nid = id.clone();
    app.update(|s| s.collections.push(crate::model::Collection { id: nid, name, games: Vec::new() }));
    id
}

pub fn toggle_in_collection(app: App, coll: &str, game: &str) {
    let (coll, game) = (coll.to_string(), game.to_string());
    app.update(|s| {
        if let Some(c) = s.collections.iter_mut().find(|c| c.id == coll) {
            if let Some(pos) = c.games.iter().position(|g| *g == game) {
                c.games.remove(pos);
            } else {
                c.games.push(game);
            }
        }
    });
}

pub fn delete_collection(app: App, coll: &str) {
    let coll = coll.to_string();
    app.update(|s| s.collections.retain(|c| c.id != coll));
    if *app.filter.peek() == Filter::Collection && *app.filter_arg.peek() == coll {
        app.set_filter(Filter::All, "");
    }
}

pub fn rename_collection(app: App, coll: &str, name: &str) {
    let (coll, name) = (coll.to_string(), name.trim().chars().take(40).collect::<String>());
    if name.is_empty() {
        return;
    }
    app.update(|s| {
        if let Some(c) = s.collections.iter_mut().find(|c| c.id == coll) {
            c.name = name;
        }
    });
}

pub fn add_tag(app: App, game: &str, tag: &str) {
    let tag: String = tag.trim().trim_start_matches('#').chars().take(30).collect();
    if tag.is_empty() {
        return;
    }
    // Schreibweise eines vorhandenen Tags übernehmen
    let tag = app.settings.peek().all_tags().into_iter().find(|t| t.eq_ignore_ascii_case(&tag)).unwrap_or(tag);
    let game = game.to_string();
    app.update(|s| {
        let list = s.tags.entry(game).or_default();
        if !list.iter().any(|t| t.eq_ignore_ascii_case(&tag)) {
            list.push(tag);
        }
    });
}

pub fn remove_tag(app: App, game: &str, tag: &str) {
    let (game, tag) = (game.to_string(), tag.to_string());
    app.update(|s| {
        if let Some(list) = s.tags.get_mut(&game) {
            list.retain(|t| *t != tag);
            if list.is_empty() {
                s.tags.remove(&game);
            }
        }
    });
}

/// Eigenes Cover für irgendein Spiel wählen.
pub fn pick_cover(app: App, id: &str) {
    if let Some(path) = pick_image_file() {
        let id = id.to_string();
        app.update(|s| {
            s.cover_overrides.insert(id, path);
        });
        app.toast(&t("toast.cover_changed"), "image");
    }
}

pub fn reset_cover(app: App, id: &str) {
    let id = id.to_string();
    app.update(|s| {
        s.cover_overrides.remove(&id);
        s.logo_overrides.remove(&id);
    });
    rescan(app);
}

// ----------------------------------------------------------------------
// Datei-Dialoge
// ----------------------------------------------------------------------

pub fn pick_game_file(start: Option<&str>) -> Option<String> {
    let mut d = rfd::FileDialog::new().set_title(t("pick.game_title"));
    if cfg!(windows) {
        d = d.add_filter(t("pick.game_filter"), &["exe", "lnk", "bat", "cmd", "url"]);
    } else if cfg!(target_os = "macos") {
        d = d.add_filter(t("pick.game_filter"), &["app", "command", "sh"]);
    } else {
        d = d.add_filter(t("pick.game_filter"), &["sh", "AppImage", "appimage", "desktop", "x86_64", "exe", "py"]);
    }
    d = d.add_filter(t("pick.all_files"), &["*"]);
    if let Some(s) = start.filter(|s| !s.is_empty()) {
        if let Some(parent) = std::path::Path::new(s).parent() {
            d = d.set_directory(parent);
        }
    }
    d.pick_file().map(|p| p.to_string_lossy().to_string())
}

pub fn pick_folder(title: &str) -> Option<String> {
    rfd::FileDialog::new()
        .set_title(title)
        .pick_folder()
        .map(|p| p.to_string_lossy().to_string())
}

pub fn pick_image_file() -> Option<String> {
    rfd::FileDialog::new()
        .set_title(t("pick.cover_title"))
        .add_filter(t("pick.images"), &["jpg", "jpeg", "png", "webp", "gif", "bmp"])
        .pick_file()
        .map(|p| p.to_string_lossy().to_string())
}

/// Bilder eines Spiels inkl. eigenem Cover
pub fn effective_cover(settings: &Settings, game: &Game) -> Vec<String> {
    let mut v = Vec::new();
    if let Some(p) = settings.cover_overrides.get(&game.id) {
        let r = library::image_ref(p);
        if !r.is_empty() {
            v.push(r);
        }
    }
    v.extend(game.art.cover.iter().cloned());
    v
}
