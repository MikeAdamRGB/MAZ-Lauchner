//! Datenmodell: Spiele, eigene Spiele und Einstellungen.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Source {
    Steam,
    Epic,
    Roblox,
    Gog,
    Ea,
    Ubisoft,
    BattleNet,
    Amazon,
    Itch,
    Xbox,
    Minecraft,
    Emulator,
    Custom,
    /// Spiele aus einer Erweiterung (Nummer → Eintrag im Erweiterungs-Register)
    Extension(u16),
}

impl Source {
    pub fn label(self) -> String {
        match self {
            Source::Steam => "Steam".into(),
            Source::Epic => "Epic Games".into(),
            Source::Roblox => "Roblox".into(),
            Source::Gog => "GOG".into(),
            Source::Ea => "EA app".into(),
            Source::Ubisoft => "Ubisoft Connect".into(),
            Source::BattleNet => "Battle.net".into(),
            Source::Amazon => "Amazon Games".into(),
            Source::Itch => "itch.io".into(),
            Source::Xbox => "Xbox / Microsoft Store".into(),
            Source::Minecraft => "Minecraft".into(),
            Source::Extension(i) => crate::extensions::name_of(i),
            Source::Emulator => crate::i18n::t("source.emulator"),
            Source::Custom => crate::i18n::t("source.custom"),
        }
    }

    /// Kurzer Text für Abzeichen auf den Kacheln
    pub fn badge(self) -> String {
        match self {
            Source::Steam => "STEAM".into(),
            Source::Epic => "EPIC".into(),
            Source::Roblox => "ROBLOX".into(),
            Source::Gog => "GOG".into(),
            Source::Ea => "EA".into(),
            Source::Ubisoft => "UBISOFT".into(),
            Source::BattleNet => "BATTLE.NET".into(),
            Source::Amazon => "AMAZON".into(),
            Source::Itch => "ITCH".into(),
            Source::Xbox => "XBOX".into(),
            Source::Minecraft => "MINECRAFT".into(),
            Source::Extension(i) => crate::extensions::name_of(i).to_uppercase().chars().take(14).collect(),
            Source::Emulator => "EMU".into(),
            Source::Custom => crate::i18n::t("badge.custom"),
        }
    }

    /// Kennung für Einstellungen ("gog", "ea" …)
    pub fn key(self) -> &'static str {
        match self {
            Source::Steam => "steam",
            Source::Epic => "epic",
            Source::Roblox => "roblox",
            Source::Gog => "gog",
            Source::Ea => "ea",
            Source::Ubisoft => "ubisoft",
            Source::BattleNet => "battlenet",
            Source::Amazon => "amazon",
            Source::Itch => "itch",
            Source::Xbox => "xbox",
            Source::Minecraft => "minecraft",
            Source::Extension(_) => "extension",
            Source::Emulator => "emulator",
            Source::Custom => "custom",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Source::Steam => "gamepad",
            Source::Epic => "layers",
            Source::Roblox => "cube",
            Source::Emulator => "monitor",
            Source::Minecraft => "cube",
            Source::Extension(_) => "plug",
            Source::Custom => "rocket",
            _ => "package",
        }
    }

    /// Welche Version bei doppelten Spielen bevorzugt wird (kleiner = besser)
    pub fn rank(self) -> u8 {
        match self {
            Source::Steam => 0,
            Source::Epic => 1,
            Source::Gog => 2,
            Source::Xbox => 3,
            Source::Ea => 4,
            Source::Ubisoft => 5,
            Source::BattleNet => 6,
            Source::Amazon => 7,
            Source::Itch => 8,
            Source::Roblox => 9,
            Source::Minecraft => 9,
            Source::Extension(_) => 10,
            Source::Emulator => 10,
            Source::Custom => 11,
        }
    }

    /// Weitere Stores (über Steam/Epic/Roblox hinaus)
    pub const STORES: [Source; 7] =
        [Source::Gog, Source::Ea, Source::Ubisoft, Source::BattleNet, Source::Amazon, Source::Itch, Source::Xbox];
}

#[derive(Clone, Debug, PartialEq)]
pub enum LaunchTarget {
    /// steam://rungameid/<appid>
    SteamRun(u32),
    /// Nicht installiert → steam://install/<appid>
    SteamInstall(u32),
    /// com.epicgames.launcher://apps/…?action=launch
    Epic(String),
    /// Nicht installiert → com.epicgames.launcher://apps/…?action=install
    EpicInstall(String),
    /// Roblox: None = die Roblox-App, Some(placeId) = direkt in eine Experience
    Roblox(Option<u64>),
    /// Über einen Link des Store-Programms starten (uplay://, amazon-games:// …)
    Uri(String),
    /// Programm direkt starten (z. B. GOG ohne Galaxy, itch.io, Battle.net-Client mit Parametern)
    Exe { path: String, args: Vec<String>, dir: String },
    /// Microsoft-Store-/Xbox-App
    Appx { family: String, app: String },

    /// Eigenes Spiel (Details stehen in den Einstellungen)
    Custom,
}

/// Bild-Kandidaten: Die Oberfläche probiert sie der Reihe nach durch.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Art {
    /// Hochformat-Cover (600×900)
    pub cover: Vec<String>,
    /// Großes Hintergrundbild (Steam "library_hero")
    pub hero: Vec<String>,
    /// Transparentes Spiel-Logo
    pub logo: Vec<String>,
    /// Kleines quadratisches Icon (Original-Spiel-Icon)
    pub icon: Vec<String>,
    /// Querformat-Banner (460×215)
    pub header: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Game {
    pub id: String,
    pub name: String,
    pub source: Source,
    pub installed: bool,
    pub install_dir: Option<String>,
    pub exe: Option<String>,
    pub launch: LaunchTarget,
    pub art: Art,
    pub size_bytes: Option<u64>,
    pub playtime_min: Option<u64>,
    pub steam_appid: Option<u32>,
}

impl Game {
    pub fn sort_key(&self) -> String {
        let n = self.name.to_lowercase();
        n.strip_prefix("the ").map(|s| s.to_string()).unwrap_or(n)
    }
}

/// Ein Spiel, das du selbst hinzugefügt hast.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CustomGame {
    pub id: String,
    pub name: String,
    /// Pfad zur EXE (oder .lnk/.bat/.url)
    pub exe: String,
    /// Startparameter, z. B. "-windowed"
    pub args: String,
    /// Leer = Ordner der EXE
    pub workdir: String,
    /// Eigenes Cover-Bild (leer = automatisch)
    pub cover: String,
    pub admin: bool,
    /// Zusätzliche Punkte für die Sicherheitsfrage (eine Zeile = ein Punkt)
    pub checklist: String,
    pub added: u64,
    /// Linux: Wine/Proton für dieses Spiel (Pfad, leer = System-Wine)
    pub runner: String,
}

/// Eine Roblox-Experience, die der Nutzer als eigene Kachel hinzugefügt hat
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RobloxGame {
    pub place_id: u64,
    pub name: String,
    /// Quadratisches Spiel-Icon (https)
    pub icon: String,
    /// Breites Vorschaubild (https)
    pub thumb: String,
    pub added: u64,
}

/// Eigene Sammlung (z. B. "Mit Freunden", "Durchgespielt")
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub games: Vec<String>,
}

/// Vom Nutzer bearbeitete Spiel-Infos (leer = automatisch)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct GameMeta {
    pub name: String,
    pub hero: String,
    pub description: String,
    pub genres: Vec<String>,
}

/// Emulator-Profil, z. B. RetroArch mit Kern, Dolphin, PCSX2, PPSSPP
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Emulator {
    pub id: String,
    pub name: String,
    /// Programm des Emulators
    pub exe: String,
    /// Startparameter; {rom} wird durch die ROM-Datei ersetzt
    pub args: String,
    /// Dateiendungen der ROMs, z. B. ["iso", "rvz"]
    pub extensions: Vec<String>,
    /// Ordner, die nach ROMs durchsucht werden
    pub rom_dirs: Vec<String>,
}

/// Spielstatus (vom Nutzer gesetzt)
pub mod status {
    pub const NOT_STARTED: &str = "not_started";
    pub const PLAYING: &str = "playing";
    pub const FINISHED: &str = "finished";
    pub const DROPPED: &str = "dropped";
    pub const ALL: [&str; 4] = [NOT_STARTED, PLAYING, FINISHED, DROPPED];

    pub fn valid(s: &str) -> bool {
        ALL.contains(&s)
    }

    pub fn icon(s: &str) -> &'static str {
        match s {
            PLAYING => "play",
            FINISHED => "check-circle",
            DROPPED => "x-circle",
            _ => "flag",
        }
    }
}

/// Eine gemessene Spielsitzung (für die Statistik)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Session {
    pub id: String,
    /// Start (Unix-Sekunden)
    pub start: u64,
    pub minutes: u64,
}

/// Höchstens so viele Sitzungen merken (älteste fallen weg)
pub const MAX_SESSIONS: usize = 5000;

pub mod layout {
    pub const MAZ: &str = "maz";
    pub const STEAM: &str = "steam";
    pub const EPIC: &str = "epic";
}

/// Standard-Punkte der Sicherheitsfrage (in der gewählten Sprache)
pub fn default_checklist() -> Vec<String> {
    crate::i18n::tl("checklist.default")
}

/// Frühere deutsche Standardliste – wird beim Laden durch die übersetzte ersetzt
pub const OLD_GERMAN_CHECKLIST: &[&str] = &[
    "Antivirus / Echtzeitschutz (falls das Spiel das braucht)",
    "Overlays (Discord, Steam, GeForce Experience, Xbox Game Bar)",
    "Aufnahme- und Streaming-Programme (OBS, Medal, ShadowPlay)",
    "VPN und Proxy",
    "Unnötige Programme im Hintergrund (Browser, Downloads)",
];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Sprache ("auto" = Windows-Sprache)
    pub language: String,

    // Design
    pub theme: String,
    pub custom_themes: Vec<crate::theme::CustomTheme>,
    /// Hintergrundbild (leer = keins)
    pub wallpaper: String,
    /// Wie stark das Hintergrundbild abgedunkelt wird (0–90 %)
    pub wallpaper_dim: u8,
    pub wallpaper_blur: bool,
    pub layout: String,
    pub authentic_colors: bool,
    /// Eigene Akzentfarbe (#rrggbb), leer = Farbe des Themes
    pub accent: String,
    /// 1 = klein, 2 = mittel, 3 = groß
    pub card_size: u8,
    pub animations: bool,
    pub show_names: bool,

    // Verbindungen
    pub steam_enabled: bool,
    pub epic_enabled: bool,
    #[serde(with = "crate::util::secret")]
    pub steam_api_key: String,
    pub steam_id: String,
    pub steam_show_uninstalled: bool,
    pub epic_show_uninstalled: bool,
    /// Verbundenes Epic-Konto (für die komplette Epic-Bibliothek)
    pub epic_auth: Option<crate::epicacct::EpicAuth>,
    /// Wann die Epic-Bibliothek zuletzt online geladen wurde
    pub epic_synced: u64,
    /// Hinweis „Epic-Konto verbinden“ in der Bibliothek weggeklickt
    pub epic_hint_dismissed: bool,
    /// Roblox anzeigen + eigene Roblox-Experiences
    pub roblox_enabled: bool,
    pub roblox_games: Vec<RobloxGame>,
    /// Automatisch nach Updates suchen
    pub update_check: bool,
    /// Auch Test-Versionen (Pre-releases) als Update anbieten
    pub update_test: bool,
    /// Diese Version nicht mehr anbieten
    pub update_skip: String,
    pub online_art: bool,
    /// Eigener CurseForge-API-Schlüssel (für Minecraft-Inhalte von CurseForge)
    #[serde(with = "crate::util::secret")]
    pub curseforge_key: String,

    // Weitere Stores: Kennung ("gog", "ea" …) → angezeigt? (fehlt = ja)
    pub stores_off: Vec<String>,
    /// Doppelte Spiele (gleiches Spiel in mehreren Stores) zusammenfassen
    pub merge_duplicates: bool,
    /// Auch gekaufte, nicht installierte Spiele der weiteren Stores zeigen
    pub stores_show_uninstalled: bool,
    /// Erweiterungen erlauben (Standard: aus)
    pub extensions_allowed: bool,
    /// Zustimmung/Status je Erweiterung (ID → Zustand)
    pub extensions: BTreeMap<String, crate::extensions::ExtState>,
    /// Bedienung mit Controller (große Fokus-Rahmen, Steuerkreuz/Stick)
    pub controller_mode: bool,

    // Bibliothek verwalten
    /// Im MAZ Launcher gemessene Spielzeit in Minuten
    pub playtime_local: BTreeMap<String, u64>,
    /// Wann ein Spiel zum ersten Mal in der Bibliothek auftauchte
    pub first_seen: BTreeMap<String, u64>,
    /// Eigene Tags je Spiel
    pub tags: BTreeMap<String, Vec<String>>,
    pub collections: Vec<Collection>,
    /// Selbst bearbeitete Spiel-Infos
    pub meta: BTreeMap<String, GameMeta>,
    /// Spielstatus je Spiel (siehe `status`)
    pub status: BTreeMap<String, String>,
    /// Gemessene Spielsitzungen (für Statistik pro Woche/Monat)
    pub sessions: Vec<Session>,

    // Emulatoren
    pub emulators: Vec<Emulator>,


    // Eigene Spiele
    /// Leer = Standardliste in der gewählten Sprache
    pub checklist: Vec<String>,
    pub custom_games: Vec<CustomGame>,

    // Verhalten
    pub minimize_on_launch: bool,
    pub sort: String,

    // Gemerkte Daten
    pub favorites: Vec<String>,
    pub hidden: Vec<String>,
    pub last_played: BTreeMap<String, u64>,
    pub launch_count: BTreeMap<String, u32>,
    pub cover_overrides: BTreeMap<String, String>,
    /// Name (normalisiert) → Steam-AppID für Cover-Bilder (0 = nichts gefunden)
    pub art_lookup: BTreeMap<String, u32>,
    pub welcome_done: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            language: "auto".into(),
            theme: "maz".into(),
            custom_themes: Vec::new(),
            wallpaper: String::new(),
            wallpaper_dim: 60,
            wallpaper_blur: false,
            layout: layout::MAZ.into(),
            authentic_colors: true,
            accent: String::new(),
            card_size: 2,
            animations: true,
            show_names: true,
            steam_enabled: true,
            epic_enabled: true,
            steam_api_key: String::new(),
            steam_id: String::new(),
            steam_show_uninstalled: true,
            epic_show_uninstalled: true,
            epic_auth: None,
            epic_synced: 0,
            epic_hint_dismissed: false,
            roblox_enabled: true,
            roblox_games: Vec::new(),
            update_check: true,
            update_test: false,
            update_skip: String::new(),
            stores_off: Vec::new(),
            merge_duplicates: true,
            stores_show_uninstalled: true,
            controller_mode: false,
            extensions_allowed: false,
            extensions: BTreeMap::new(),
            playtime_local: BTreeMap::new(),
            first_seen: BTreeMap::new(),
            tags: BTreeMap::new(),
            collections: Vec::new(),
            meta: BTreeMap::new(),
            status: BTreeMap::new(),
            sessions: Vec::new(),
            emulators: Vec::new(),
            online_art: true,
            curseforge_key: String::new(),
            checklist: Vec::new(),
            custom_games: Vec::new(),
            minimize_on_launch: true,
            sort: "name".into(),
            favorites: Vec::new(),
            hidden: Vec::new(),
            last_played: BTreeMap::new(),
            launch_count: BTreeMap::new(),
            cover_overrides: BTreeMap::new(),
            art_lookup: BTreeMap::new(),
            welcome_done: false,
        }
    }
}

impl Settings {
    pub fn is_favorite(&self, id: &str) -> bool {
        self.favorites.iter().any(|f| f == id)
    }
    pub fn is_hidden(&self, id: &str) -> bool {
        self.hidden.iter().any(|f| f == id)
    }
    pub fn custom(&self, id: &str) -> Option<&CustomGame> {
        self.custom_games.iter().find(|c| c.id == id)
    }
    /// Punkte der Sicherheitsfrage (eigene oder Standard)
    pub fn effective_checklist(&self) -> Vec<String> {
        if self.checklist.is_empty() {
            default_checklist()
        } else {
            self.checklist.clone()
        }
    }
    pub fn store_on(&self, s: Source) -> bool {
        !self.stores_off.iter().any(|k| k == s.key())
    }
    pub fn tags_of(&self, id: &str) -> Vec<String> {
        self.tags.get(id).cloned().unwrap_or_default()
    }
    /// Alle vergebenen Tags (sortiert, ohne Doppelte)
    pub fn all_tags(&self) -> Vec<String> {
        let mut v: Vec<String> = self.tags.values().flatten().cloned().collect();
        v.sort_by_key(|t| t.to_lowercase());
        v.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
        v
    }
    pub fn last_played(&self, id: &str) -> u64 {
        self.last_played.get(id).copied().unwrap_or(0)
    }
    /// Spielzeit in Minuten: Store-Angabe (Steam zählt selbst mit) oder im Launcher gemessen
    pub fn playtime(&self, g: &Game) -> u64 {
        let local = self.playtime_local.get(&g.id).copied().unwrap_or(0);
        g.playtime_min.unwrap_or(0).max(local)
    }
    pub fn first_seen(&self, id: &str) -> u64 {
        self.first_seen.get(id).copied().unwrap_or(0)
    }
    /// Spielstatus ("" = nicht gesetzt)
    pub fn status_of(&self, id: &str) -> &str {
        self.status.get(id).map(|s| s.as_str()).unwrap_or("")
    }
    /// Spielsitzung merken; beim ersten Spielen wird „Nicht begonnen“ zu „Spiele ich“
    pub fn add_session(&mut self, id: &str, start: u64, minutes: u64) {
        if minutes == 0 {
            return;
        }
        self.sessions.push(Session { id: id.to_string(), start, minutes });
        if self.sessions.len() > MAX_SESSIONS {
            let extra = self.sessions.len() - MAX_SESSIONS;
            self.sessions.drain(..extra);
        }
        if matches!(self.status_of(id), "" | status::NOT_STARTED) {
            self.status.insert(id.to_string(), status::PLAYING.to_string());
        }
    }
    pub fn in_collection(&self, coll: &str, id: &str) -> bool {
        self.collections.iter().any(|c| c.id == coll && c.games.iter().any(|g| g == id))
    }
}

/// Status einer Plattform-Verbindung (für Einstellungen → Verbindungen).
#[derive(Clone, Debug, PartialEq, Default)]
pub struct StoreStatus {
    pub found: bool,
    pub path: Option<String>,
    pub installed: usize,
    pub owned: usize,
    pub account: Option<String>,
    pub note: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sessions_and_status() {
        let mut s = Settings::default();
        s.add_session("a", 100, 0);
        assert!(s.sessions.is_empty());
        s.add_session("a", 100, 30);
        assert_eq!(s.status_of("a"), status::PLAYING);
        s.status.insert("b".into(), status::FINISHED.into());
        s.add_session("b", 200, 10);
        assert_eq!(s.status_of("b"), status::FINISHED, "durchgespielt bleibt durchgespielt");
        for i in 0..(MAX_SESSIONS + 10) {
            s.add_session("c", i as u64, 1);
        }
        assert_eq!(s.sessions.len(), MAX_SESSIONS);
        // Alte config.json ohne neue Felder lädt weiter
        let old: Settings = serde_json::from_str(r#"{"theme":"oled"}"#).unwrap();
        assert!(old.status.is_empty() && old.sessions.is_empty());
    }
}

/// Normalisierter Name für Vergleiche ("The Witcher® 3: Wild Hunt" → "thewitcher3wildhunt").
pub fn normalize_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}
