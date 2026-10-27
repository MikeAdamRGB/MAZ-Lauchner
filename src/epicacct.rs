//! Epic-Games-Konto verbinden: damit wirklich ALLE Spiele der Epic-Bibliothek
//! angezeigt werden (auch nie installierte). Ablauf wie bei Heroic/Legendary:
//! Anmeldung im Browser → Epic zeigt einen Code → der Launcher tauscht ihn gegen
//! ein Token und lädt damit die Bibliothek. Die Liste wird lokal zwischengespeichert,
//! damit sie auch ohne gültige Anmeldung erhalten bleibt.

use crate::model::Art;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

/// Fortschritt beim Laden (für die Anzeige „12 von 230“)
#[derive(Default)]
pub struct Progress {
    pub done: AtomicUsize,
    pub total: AtomicUsize,
}

const CLIENT_ID: &str = "34a02cf8f4414e29b15921876da36f9a";
/// Base64 von "client_id:client_secret" des Epic Games Launchers
const BASIC: &str = "MzRhMDJjZjhmNDQxNGUyOWIxNTkyMTg3NmRhMzZmOWE6ZGFhZmJjY2M3Mzc3NDUwMzlkZmZlNTNkOTRmYzc2Y2Y=";
const OAUTH: &str = "https://account-public-service-prod03.ol.epicgames.com/account/api/oauth/token";
const LIBRARY: &str = "https://library-service.live.use1a.on.epicgames.com/library/api/public/items";
const CATALOG: &str = "https://catalog-public-service-prod06.ol.epicgames.com/catalog/api/shared/namespace";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EpicAuth {
    #[serde(with = "crate::util::secret")]
    pub access_token: String,
    #[serde(with = "crate::util::secret")]
    pub refresh_token: String,
    pub expires_at: u64,
    pub refresh_expires_at: u64,
    pub account_id: String,
    pub display_name: String,
}

/// Ein Spiel aus der Online-Bibliothek (wird in einer Datei zwischengespeichert)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct OwnedEpic {
    pub app_name: String,
    pub namespace: String,
    pub item_id: String,
    pub title: String,
    pub cover: Vec<String>,
    pub hero: Vec<String>,
    pub logo: Vec<String>,
}

impl OwnedEpic {
    pub fn art(&self) -> Art {
        Art {
            cover: self.cover.clone(),
            hero: self.hero.clone(),
            logo: self.logo.clone(),
            icon: self.cover.iter().map(|u| format!("{u}?w=64&h=64&resize=1")).collect(),
            header: self.hero.clone(),
        }
    }
}

/// Seite, auf der man sich anmeldet. Danach zeigt Epic eine Textseite mit "authorizationCode".
pub fn login_url() -> String {
    let redirect = format!("https://www.epicgames.com/id/api/redirect?clientId={CLIENT_ID}&responseType=code");
    format!("https://www.epicgames.com/id/login?redirectUrl={}", crate::sys::url_encode(&redirect))
}

/// Holt den Code aus dem, was der Nutzer eingefügt hat: die ganze Seite (JSON),
/// eine URL mit ?code=… oder nur der Code selbst (32 Hex-Zeichen).
pub fn extract_code(text: &str) -> Option<String> {
    let text = text.trim();
    if let Some(i) = text.find("\"authorizationCode\"") {
        let rest = &text[i + "\"authorizationCode\"".len()..];
        let rest = rest.trim_start().strip_prefix(':')?.trim_start();
        if rest.starts_with("null") {
            return None;
        }
        let rest = rest.strip_prefix('"')?;
        let end = rest.find('"')?;
        let code = &rest[..end];
        return (code.len() >= 16).then(|| code.to_string());
    }
    let query = text.split_once('?').map(|(_, q)| q).unwrap_or(text);
    if let Some(code) = crate::util::query_param(query, "code") {
        if code.len() >= 16 && code.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Some(code);
        }
    }
    let t = text.trim_matches('"');
    if t.len() == 32 && t.chars().all(|c| c.is_ascii_hexdigit()) {
        return Some(t.to_string());
    }
    None
}

/// Eine gemeinsame Verbindung für alle Anfragen (spart pro Anfrage den TLS-Aufbau)
fn agent() -> ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(build_agent).clone()
}

fn build_agent() -> ureq::Agent {
    use ureq::tls::{RootCerts, TlsConfig, TlsProvider};
    let tls = TlsConfig::builder()
        .provider(TlsProvider::NativeTls)
        .root_certs(RootCerts::PlatformVerifier)
        .build();
    ureq::Agent::config_builder()
        .https_only(true)
        .tls_config(tls)
        .timeout_global(Some(Duration::from_secs(20)))
        .http_status_as_error(false)
        .user_agent("UELauncher/11.0.1-14907503+++Portal+Release-Live Windows/10.0.19041.1.256.64bit")
        .build()
        .into()
}

fn read(resp: Result<ureq::http::Response<ureq::Body>, ureq::Error>) -> Result<(u16, serde_json::Value), String> {
    let mut resp = resp.map_err(|e| format!("{}: {e}", crate::i18n::t("common.err_network")))?;
    let status = resp.status().as_u16();
    // Antworten begrenzen (Katalog-Antworten sind wenige hundert KB)
    let text = resp.body_mut().with_config().limit(8 * 1024 * 1024).read_to_string().unwrap_or_default();
    let v = serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);
    Ok((status, v))
}

fn jstr(v: &serde_json::Value, key: &str) -> String {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("").to_string()
}

fn token(body: String) -> Result<EpicAuth, String> {
    let (status, v) = read(
        agent()
            .post(OAUTH)
            .header("Authorization", &format!("basic {BASIC}"))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send(body.as_str()),
    )?;
    if status >= 400 || v.get("access_token").is_none() {
        let code = jstr(&v, "errorCode");
        let msg = jstr(&v, "errorMessage");
        return Err(if code.contains("invalid_grant") || code.contains("not_found") {
            crate::i18n::t("epic.err_code")
        } else if msg.is_empty() {
            format!("Epic: HTTP {status}")
        } else {
            format!("Epic: {msg}")
        });
    }
    let now = crate::sys::now_secs();
    let secs = |k: &str, d: u64| v.get(k).and_then(|x| x.as_u64()).unwrap_or(d);
    Ok(EpicAuth {
        access_token: jstr(&v, "access_token"),
        refresh_token: jstr(&v, "refresh_token"),
        expires_at: now + secs("expires_in", 7200),
        refresh_expires_at: now + secs("refresh_expires", 28800),
        account_id: jstr(&v, "account_id"),
        display_name: jstr(&v, "displayName"),
    })
}

/// Code aus dem Browser gegen eine Anmeldung tauschen
pub fn exchange(code: &str) -> Result<EpicAuth, String> {
    token(format!("grant_type=authorization_code&code={}&token_type=eg1", crate::sys::url_encode(code)))
}

/// Anmeldung erneuern (falls das Token abgelaufen ist)
pub fn ensure_fresh(auth: &EpicAuth) -> Result<EpicAuth, String> {
    let now = crate::sys::now_secs();
    if !auth.access_token.is_empty() && auth.expires_at > now + 120 {
        return Ok(auth.clone());
    }
    if auth.refresh_token.is_empty() {
        return Err(crate::i18n::t("epic.err_expired"));
    }
    token(format!(
        "grant_type=refresh_token&refresh_token={}&token_type=eg1",
        crate::sys::url_encode(&auth.refresh_token)
    ))
    .map_err(|_| crate::i18n::t("epic.err_expired"))
}

fn get(url: &str, auth: &EpicAuth) -> Result<serde_json::Value, String> {
    let (status, v) = read(agent().get(url).header("Authorization", &format!("bearer {}", auth.access_token)).call())?;
    if status == 401 {
        return Err(crate::i18n::t("epic.err_expired"));
    }
    if status >= 400 {
        return Err(format!("Epic: HTTP {status}"));
    }
    Ok(v)
}

fn key_images(v: &serde_json::Value, types: &[&str]) -> Vec<String> {
    let Some(list) = v.get("keyImages").and_then(|k| k.as_array()) else { return Vec::new() };
    let mut out = Vec::new();
    for t in types {
        for img in list {
            if img.get("type").and_then(|x| x.as_str()) == Some(*t) {
                if let Some(u) = img.get("url").and_then(|x| x.as_str()).filter(|u| !u.is_empty()) {
                    out.push(u.to_string());
                }
            }
        }
    }
    out
}

/// Bibliothek laden. `known` = bereits bekannte Einträge (Cache), damit nicht jedes Mal
/// hunderte Katalog-Abfragen nötig sind.
pub fn fetch_library(auth: &EpicAuth, known: &[OwnedEpic], lang: &str, progress: &Progress) -> Result<Vec<OwnedEpic>, String> {
    // 1. Alle Einträge der Bibliothek (seitenweise)
    let mut records: Vec<(String, String, String)> = Vec::new();
    let mut cursor = String::new();
    for _ in 0..50 {
        let mut url = format!("{LIBRARY}?includeMetadata=true");
        if !cursor.is_empty() {
            url.push_str(&format!("&cursor={}", crate::sys::url_encode(&cursor)));
        }
        let v = get(&url, auth)?;
        for r in v.get("records").and_then(|r| r.as_array()).into_iter().flatten() {
            let ns = jstr(r, "namespace");
            let item = jstr(r, "catalogItemId");
            let app = jstr(r, "appName");
            // "ue" = Unreal-Marktplatz (keine Spiele)
            if ns == "ue" || ns.is_empty() || item.is_empty() || app.is_empty() {
                continue;
            }
            if jstr(r, "sandboxType") == "PRIVATE" {
                continue;
            }
            if !records.iter().any(|(_, _, a)| *a == app) {
                records.push((ns, item, app));
            }
        }
        cursor = v
            .get("responseMetadata")
            .and_then(|m| m.get("nextCursor"))
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();
        if cursor.is_empty() {
            break;
        }
    }

    // 2. Titel und Bilder aus dem Katalog (bekannte aus dem Cache übernehmen)
    let (locale, country) = match lang {
        "de" => ("de", "DE"),
        "fr" => ("fr", "FR"),
        "es" => ("es-ES", "ES"),
        "it" => ("it", "IT"),
        "pt" => ("pt-BR", "BR"),
        "pl" => ("pl", "PL"),
        "tr" => ("tr", "TR"),
        "ru" => ("ru", "RU"),
        _ => ("en-US", "US"),
    };
    let mut out = Vec::new();
    let mut by_ns: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    let total = records.len();
    for (ns, item, app) in records {
        if let Some(k) = known.iter().find(|k| k.app_name == app && !k.title.is_empty()) {
            out.push(k.clone());
        } else {
            by_ns.entry(ns).or_default().push((item, app));
        }
    }
    progress.total.store(total, Ordering::Relaxed);
    progress.done.store(out.len(), Ordering::Relaxed);

    // Katalog-Abfragen parallel (fast jedes Spiel hat einen eigenen Namespace → sonst
    // hunderte Anfragen nacheinander). 8 gleichzeitig ist schnell und schont Epic.
    let mut jobs: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for (ns, items) in by_ns {
        for chunk in items.chunks(40) {
            jobs.push((ns.clone(), chunk.to_vec()));
        }
    }
    let jobs = Arc::new(Mutex::new(jobs));
    let found: Arc<Mutex<Vec<OwnedEpic>>> = Arc::new(Mutex::new(Vec::new()));
    std::thread::scope(|scope| {
        for _ in 0..8 {
            let (jobs, found) = (jobs.clone(), found.clone());
            scope.spawn(move || loop {
                let next = jobs.lock().ok().and_then(|mut j| j.pop());
                let Some((ns, chunk)) = next else { break };
                let items = catalog_items(auth, &ns, &chunk, locale, country);
                progress.done.fetch_add(chunk.len(), Ordering::Relaxed);
                if let Ok(mut f) = found.lock() {
                    f.extend(items);
                }
            });
        }
    });
    if let Ok(mut f) = found.lock() {
        out.append(&mut f);
    }
    out.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
    Ok(out)
}

/// Titel und Bilder für einige Einträge eines Namespaces
fn catalog_items(auth: &EpicAuth, ns: &str, chunk: &[(String, String)], locale: &str, country: &str) -> Vec<OwnedEpic> {
    let ids: String = chunk.iter().map(|(i, _)| format!("&id={}", crate::sys::url_encode(i))).collect();
    let url = format!(
        "{CATALOG}/{}/bulk/items?includeDLCDetails=true&includeMainGameDetails=true&country={country}&locale={locale}{ids}",
        crate::sys::url_encode(ns)
    );
    // Einmal neu versuchen (z. B. kurzer Verbindungsfehler)
    let v = match get(&url, auth).or_else(|_| get(&url, auth)) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for (item, app) in chunk {
        let Some(info) = v.get(item.as_str()) else { continue };
        // DLCs, Zusatzinhalte und Programme ohne Spiel weglassen
        if info.get("mainGameItem").map(|m| !m.is_null()).unwrap_or(false) {
            continue;
        }
        let cats: Vec<String> = info
            .get("categories")
            .and_then(|c| c.as_array())
            .map(|a| a.iter().map(|c| jstr(c, "path").to_lowercase()).collect())
            .unwrap_or_default();
        if cats.iter().any(|c| matches!(c.as_str(), "addons" | "engines" | "digitalextras" | "plugins")) {
            continue;
        }
        if !cats.is_empty() && !cats.iter().any(|c| c == "games" || c == "applications") {
            continue;
        }
        let title = jstr(info, "title").trim().to_string();
        if title.is_empty() {
            continue;
        }
        let https = |v: Vec<String>| v.into_iter().filter(|u| u.starts_with("https://")).collect::<Vec<_>>();
        out.push(OwnedEpic {
            app_name: app.clone(),
            namespace: ns.to_string(),
            item_id: item.clone(),
            title,
            cover: https(key_images(info, &["DieselGameBoxTall", "OfferImageTall", "Thumbnail"])),
            hero: https(key_images(info, &["DieselGameBox", "OfferImageWide", "DieselStoreFrontWide"])),
            logo: https(key_images(info, &["DieselGameBoxLogo"])),
        });
    }
    out
}

fn cache_path() -> std::path::PathBuf {
    crate::sys::local_dir().join("epic-library.json")
}

/// Zwischengespeicherte Epic-Bibliothek
pub fn load_cache() -> Vec<OwnedEpic> {
    std::fs::read_to_string(cache_path())
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

pub fn save_cache(list: &[OwnedEpic]) {
    let p = cache_path();
    if let Some(dir) = p.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(text) = serde_json::to_string(list) {
        let _ = crate::sys::write_private(&p, text.as_bytes());
    }
}

pub fn clear_cache() {
    let _ = std::fs::remove_file(cache_path());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_extraction() {
        let page = r#"{"warning":"Do not share this code with any 3rd party service.","redirectUrl":"https://localhost/launcher/authorized?code=0123456789abcdef0123456789abcdef","authorizationCode":"0123456789abcdef0123456789abcdef","exchangeCode":null,"sid":null}"#;
        assert_eq!(extract_code(page).as_deref(), Some("0123456789abcdef0123456789abcdef"));
        assert_eq!(extract_code("  0123456789abcdef0123456789abcdef \n").as_deref(), Some("0123456789abcdef0123456789abcdef"));
        assert_eq!(extract_code(r#"{"authorizationCode":null}"#), None);
        assert_eq!(extract_code("hallo"), None);
        assert!(login_url().starts_with("https://www.epicgames.com/id/login?redirectUrl=https%3A%2F%2F"));
    }
}
