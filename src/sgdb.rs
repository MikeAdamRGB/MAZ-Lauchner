//! SteamGridDB: Cover, Hintergründe, Logos und Icons – mit dem eigenen, kostenlosen Schlüssel
//! des Nutzers (steamgriddb.com → Profil → Einstellungen → API).
//!
//! Gefundene Bilder werden lokal gemerkt (nur die Adressen). Ohne Schlüssel passiert nichts.
//! SICHERHEIT: Es werden nur Bild-Adressen von SteamGridDB-Servern übernommen.

use crate::model::{normalize_name, Game, Source};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::Duration;

const API: &str = "https://www.steamgriddb.com/api/v2";
/// Nicht gefundene Spiele erst nach 14 Tagen erneut versuchen
const RETRY_SECS: u64 = 14 * 86_400;

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Images {
    pub cover: String,
    pub hero: String,
    pub logo: String,
    pub icon: String,
    pub fetched: u64,
}

impl Images {
    pub fn is_empty(&self) -> bool {
        self.cover.is_empty() && self.hero.is_empty() && self.logo.is_empty() && self.icon.is_empty()
    }
}

/// Eine Auswahlmöglichkeit im Bilder-Dialog
#[derive(Clone, Debug, PartialEq)]
pub struct Choice {
    pub url: String,
    pub thumb: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Cover,
    Hero,
    Logo,
    Icon,
}

impl Kind {
    fn path(self) -> &'static str {
        match self {
            Kind::Cover => "grids",
            Kind::Hero => "heroes",
            Kind::Logo => "logos",
            Kind::Icon => "icons",
        }
    }
    fn query(self) -> &'static str {
        match self {
            // Hochformat-Cover wie bei Steam
            Kind::Cover => "?dimensions=600x900,342x482,660x930&types=static&nsfw=false&humor=false",
            _ => "?types=static&nsfw=false&humor=false",
        }
    }
}

fn cache_path() -> std::path::PathBuf {
    crate::sys::local_dir().join("steamgriddb.json")
}

fn cache() -> &'static Mutex<BTreeMap<String, Images>> {
    static C: std::sync::OnceLock<Mutex<BTreeMap<String, Images>>> = std::sync::OnceLock::new();
    C.get_or_init(|| {
        let map = std::fs::read_to_string(cache_path()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
        Mutex::new(map)
    })
}

fn save_cache(map: &BTreeMap<String, Images>) {
    if let Ok(text) = serde_json::to_string(map) {
        let _ = crate::sys::write_private(&cache_path(), text.as_bytes());
    }
}

/// Schlüssel im Zwischenspeicher: Steam-AppID oder normalisierter Name
pub fn cache_key(g: &Game) -> String {
    match g.steam_appid {
        Some(a) => format!("steam:{a}"),
        None => format!("name:{}", normalize_name(&g.name)),
    }
}

/// SICHERHEIT: nur Bilder von SteamGridDB
pub fn allowed_image(u: &str) -> bool {
    let Some(rest) = u.strip_prefix("https://") else { return false };
    let host = rest.split(['/', '?', '#']).next().unwrap_or("").to_ascii_lowercase();
    u.len() < 500 && (host == "cdn2.steamgriddb.com" || host == "cdn.steamgriddb.com" || host == "steamgriddb.com")
}

/// Gemerkte Bilder an ein Spiel hängen (als Ausweich hinter den Original-Bildern)
pub fn apply(g: &mut Game) {
    if matches!(g.source, Source::Roblox) {
        return;
    }
    let Ok(map) = cache().lock() else { return };
    let Some(img) = map.get(&cache_key(g)) else { return };
    let add = |list: &mut Vec<String>, u: &str| {
        if !u.is_empty() && allowed_image(u) && !list.iter().any(|x| x == u) {
            list.push(u.to_string());
        }
    };
    add(&mut g.art.cover, &img.cover);
    add(&mut g.art.hero, &img.hero);
    add(&mut g.art.logo, &img.logo);
    add(&mut g.art.icon, &img.icon);
}

fn agent() -> ureq::Agent {
    use std::sync::OnceLock;
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT
        .get_or_init(|| {
            use ureq::tls::{RootCerts, TlsConfig, TlsProvider};
            let tls = TlsConfig::builder().provider(TlsProvider::NativeTls).root_certs(RootCerts::PlatformVerifier).build();
            ureq::Agent::config_builder()
                .https_only(true)
                .tls_config(tls)
                .timeout_global(Some(Duration::from_secs(15)))
                .http_status_as_error(false)
                .user_agent(concat!("Open-Game-Launcher/", env!("CARGO_PKG_VERSION")))
                .build()
                .into()
        })
        .clone()
}

#[derive(Debug, PartialEq)]
pub enum ApiError {
    /// Schlüssel abgelehnt
    Key,
    Network(String),
}

fn get(path: &str, key: &str) -> Result<serde_json::Value, ApiError> {
    let mut resp = agent()
        .get(&format!("{API}{path}"))
        .header("Authorization", &format!("Bearer {}", key.trim()))
        .call()
        .map_err(|e| ApiError::Network(e.to_string()))?;
    let status = resp.status().as_u16();
    if status == 401 || status == 403 {
        return Err(ApiError::Key);
    }
    let text = resp.body_mut().with_config().limit(2 * 1024 * 1024).read_to_string().map_err(|e| ApiError::Network(e.to_string()))?;
    if status == 404 {
        return Ok(serde_json::json!({ "success": true, "data": [] }));
    }
    if status >= 400 {
        return Err(ApiError::Network(format!("HTTP {status}")));
    }
    serde_json::from_str(&text).map_err(|e| ApiError::Network(e.to_string()))
}

/// Schlüssel prüfen (sucht kurz nach einem bekannten Spiel)
pub fn check_key(key: &str) -> Result<(), ApiError> {
    get("/search/autocomplete/portal", key).map(|_| ())
}

/// SteamGridDB-Spiel-ID für ein Spiel finden
pub fn find_game(g: &Game, key: &str) -> Result<Option<u64>, ApiError> {
    if let Some(a) = g.steam_appid {
        let v = get(&format!("/games/steam/{a}"), key)?;
        if let Some(id) = v.pointer("/data/id").and_then(|x| x.as_u64()) {
            return Ok(Some(id));
        }
    }
    let name = crate::metadata::search_name(&g.name);
    if name.trim().len() < 2 {
        return Ok(None);
    }
    let v = get(&format!("/search/autocomplete/{}", crate::sys::url_encode(name.trim())), key)?;
    Ok(parse_search(&v, &name))
}

/// Bestes Suchergebnis: gleicher Name bevorzugt, sonst das erste
pub fn parse_search(v: &serde_json::Value, name: &str) -> Option<u64> {
    let list = v.get("data")?.as_array()?;
    let want = normalize_name(name);
    list.iter()
        .find(|e| e.get("name").and_then(|n| n.as_str()).map(|n| normalize_name(n) == want).unwrap_or(false))
        .or_else(|| list.first())
        .and_then(|e| e.get("id").and_then(|x| x.as_u64()))
}

/// Bilder einer Art: (Bild, Vorschaubild)
pub fn parse_images(v: &serde_json::Value) -> Vec<Choice> {
    v.get("data")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|e| {
                    let url = e.get("url")?.as_str()?.to_string();
                    let thumb = e.get("thumb").and_then(|t| t.as_str()).unwrap_or(&url).to_string();
                    (allowed_image(&url) && allowed_image(&thumb)).then_some(Choice { url, thumb })
                })
                .take(30)
                .collect()
        })
        .unwrap_or_default()
}

pub fn images(game_id: u64, kind: Kind, key: &str) -> Result<Vec<Choice>, ApiError> {
    let v = get(&format!("/{}/game/{game_id}{}", kind.path(), kind.query()), key)?;
    Ok(parse_images(&v))
}

/// Auswahl für den Bilder-Dialog
pub fn choices(g: &Game, kind: Kind, key: &str) -> Result<Vec<Choice>, ApiError> {
    match find_game(g, key)? {
        Some(id) => images(id, kind, key),
        None => Ok(Vec::new()),
    }
}

/// Fehlende Bilder für mehrere Spiele laden (im Hintergrund). Gibt zurück, ob sich etwas geändert hat.
pub fn fill(games: &[Game], key: &str, now: u64, limit: usize) -> Result<bool, ApiError> {
    let todo: Vec<&Game> = {
        let Ok(map) = cache().lock() else { return Ok(false) };
        games
            .iter()
            .filter(|g| !matches!(g.source, Source::Roblox))
            .filter(|g| g.art.cover.is_empty() || g.art.logo.is_empty() || g.art.hero.is_empty())
            .filter(|g| match map.get(&cache_key(g)) {
                None => true,
                Some(i) => i.is_empty() && now.saturating_sub(i.fetched) > RETRY_SECS,
            })
            .take(limit)
            .collect()
    };
    let mut changed = false;
    for g in todo {
        let mut img = Images { fetched: now, ..Default::default() };
        if let Some(id) = find_game(g, key)? {
            let first = |k: Kind| images(id, k, key).ok().and_then(|v| v.into_iter().next()).map(|c| c.url).unwrap_or_default();
            if g.art.cover.is_empty() {
                img.cover = first(Kind::Cover);
            }
            if g.art.hero.is_empty() {
                img.hero = first(Kind::Hero);
            }
            if g.art.logo.is_empty() {
                img.logo = first(Kind::Logo);
            }
            if g.art.icon.is_empty() {
                img.icon = first(Kind::Icon);
            }
        }
        changed |= !img.is_empty();
        if let Ok(mut map) = cache().lock() {
            map.insert(cache_key(g), img);
            save_cache(&map);
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_and_safety() {
        let v: serde_json::Value = serde_json::from_str(r#"{"success":true,"data":[{"id":1,"name":"Portal 2"},{"id":2,"name":"Portal"}]}"#).unwrap();
        assert_eq!(parse_search(&v, "Portal"), Some(2));
        assert_eq!(parse_search(&v, "Etwas anderes"), Some(1));
        assert_eq!(parse_search(&serde_json::json!({"data":[]}), "x"), None);
        let imgs: serde_json::Value = serde_json::from_str(r#"{"data":[
            {"url":"https://cdn2.steamgriddb.com/grid/a.png","thumb":"https://cdn2.steamgriddb.com/thumb/a.png"},
            {"url":"https://evil.example/b.png","thumb":"https://cdn2.steamgriddb.com/thumb/b.png"},
            {"url":"http://cdn2.steamgriddb.com/grid/c.png"}]}"#).unwrap();
        let c = parse_images(&imgs);
        assert_eq!(c.len(), 1);
        assert!(c[0].url.ends_with("a.png"));
        assert!(!allowed_image("https://cdn2.steamgriddb.com.evil.example/x.png"));
    }
}
