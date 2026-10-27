//! Automatische Spiel-Infos mit lokalem Zwischenspeicher. Zwei legale, öffentliche Quellen:
//!
//! 1. **Steam-Store** (für Steam-Spiele und Spiele, die es auch auf Steam gibt): Beschreibung,
//!    Genres, Entwickler, Erscheinungsdatum, Mindest-/Empfohlene Anforderungen, Steam-Deck-Status.
//! 2. **Wikipedia + Wikidata** (für alles andere, z. B. ROMs, GOG-, Xbox- oder eigene Spiele):
//!    Kurzbeschreibung (CC BY-SA, mit Quellenangabe), Genres, Entwickler, Erscheinungsjahr.
//!
//! Es wird nur gelesen, nie etwas hochgeladen. Antworten sind in der Größe begrenzt.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Version des Cache-Formats – ältere Einträge werden einmal neu geladen
pub const VERSION: u32 = 3;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Info {
    pub description: String,
    pub genres: Vec<String>,
    pub developer: String,
    pub release: String,
    /// Mindestanforderungen für dieses Betriebssystem: (Bezeichnung, Wert)
    pub req_min: Vec<(String, String)>,
    /// Empfohlene Anforderungen
    pub req_rec: Vec<(String, String)>,
    /// Steam Deck: 0 = unbekannt, 1 = nicht unterstützt, 2 = spielbar, 3 = verifiziert
    pub deck: u8,
    /// "steam" oder "wikipedia"
    pub source: String,
    /// Link zur Quelle (Wikipedia-Artikel)
    pub page: String,
    /// Bild aus der Quelle (nur genutzt, wenn das Spiel sonst kein Cover hat)
    pub cover: String,
    /// Trailer (nur Steam-CDN): Adresse, Art ("mp4", "webm", "hls") und Vorschaubild
    pub trailer: String,
    pub trailer_kind: String,
    pub trailer_thumb: String,
    pub version: u32,
    /// Wann geladen (0 = nichts gefunden)
    pub fetched: u64,
}

impl Info {
    pub fn is_current(&self) -> bool {
        self.version >= VERSION
    }
}

fn path() -> std::path::PathBuf {
    crate::sys::local_dir().join("metadata.json")
}

pub fn load_cache() -> BTreeMap<String, Info> {
    std::fs::read_to_string(path()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

pub fn save_cache(cache: &BTreeMap<String, Info>) {
    let p = path();
    if let Some(d) = p.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    if let Ok(t) = serde_json::to_string(cache) {
        let tmp = p.with_extension("json.tmp");
        if std::fs::write(&tmp, t).is_ok() {
            let _ = std::fs::rename(tmp, p);
        }
    }
}

/// Einfache HTML-Reste entfernen ("&amp;", "<br>")
pub fn plain(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Steam-Anforderungen (HTML-Liste) → [(„Betriebssystem“, „Windows 10“), …]
pub fn parse_requirements(html: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for item in html.split("<li>").skip(1) {
        let item = item.split("</li>").next().unwrap_or(item);
        let (label, value) = match (item.find("<strong>"), item.find("</strong>")) {
            (Some(a), Some(b)) if a < b => (plain(&item[a + 8..b]), plain(&item[b + 9..])),
            _ => (String::new(), plain(item)),
        };
        let label = label.trim_end_matches(':').trim().to_string();
        if !value.is_empty() && value.len() < 400 {
            out.push((label.chars().take(60).collect(), value));
        }
        if out.len() >= 15 {
            break;
        }
    }
    out
}

fn os_req_key() -> &'static str {
    if cfg!(target_os = "macos") {
        "mac_requirements"
    } else if cfg!(target_os = "linux") {
        "linux_requirements"
    } else {
        "pc_requirements"
    }
}

/// Antwort von store.steampowered.com/api/appdetails auswerten
pub fn parse_appdetails(appid: u32, json: &str) -> Option<Info> {
    parse_appdetails_for(appid, json, os_req_key())
}

pub fn parse_appdetails_for(appid: u32, json: &str, req_key: &str) -> Option<Info> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let d = v.get(appid.to_string())?;
    if d.get("success").and_then(|s| s.as_bool()) != Some(true) {
        return None;
    }
    let data = d.get("data")?;
    let s = |k: &str| data.get(k).and_then(|x| x.as_str()).map(plain).unwrap_or_default();
    // Linux/Mac ohne eigene Angaben (oft leer) → Windows-Angaben nehmen (Proton/Wine)
    let reqs = data
        .get(req_key)
        .filter(|r| r.get("minimum").is_some())
        .or_else(|| data.get("pc_requirements"));
    let req = |k: &str| reqs.and_then(|r| r.get(k)).and_then(|x| x.as_str()).map(parse_requirements).unwrap_or_default();
    let trailer = parse_trailer(data);
    Some(Info {
        description: s("short_description"),
        genres: data
            .get("genres")
            .and_then(|g| g.as_array())
            .map(|a| a.iter().filter_map(|g| g.get("description").and_then(|x| x.as_str()).map(|x| x.to_string())).take(8).collect())
            .unwrap_or_default(),
        developer: data
            .get("developers")
            .and_then(|d| d.as_array())
            .and_then(|a| a.first())
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
        release: data.get("release_date").and_then(|r| r.get("date")).and_then(|x| x.as_str()).unwrap_or("").to_string(),
        req_min: req("minimum"),
        req_rec: req("recommended"),
        trailer: trailer.0,
        trailer_kind: trailer.1,
        trailer_thumb: trailer.2,
        source: "steam".into(),
        version: VERSION,
        fetched: crate::sys::now_secs(),
        ..Default::default()
    })
}

/// SICHERHEIT: Videos und Bilder nur von Steams eigenen Servern
pub fn steam_media_url(u: &str) -> bool {
    let Some(rest) = u.strip_prefix("https://") else { return false };
    let host = rest.split(['/', '?', '#']).next().unwrap_or("").to_ascii_lowercase();
    u.len() < 600 && (host.ends_with(".steamstatic.com") || host.ends_with(".steampowered.com") || host == "steamcdn-a.akamaihd.net")
}

/// Ersten Trailer aus „movies“: (Adresse, Art, Vorschaubild). MP4/WebM bevorzugt,
/// sonst HLS (spielt nur macOS direkt ab).
pub fn parse_trailer(data: &serde_json::Value) -> (String, String, String) {
    let Some(movies) = data.get("movies").and_then(|m| m.as_array()) else { return Default::default() };
    // Hervorgehobener Trailer zuerst
    let mut list: Vec<&serde_json::Value> = movies.iter().collect();
    list.sort_by_key(|m| !m.get("highlight").and_then(|h| h.as_bool()).unwrap_or(false));
    for m in list {
        let thumb = m.get("thumbnail").and_then(|x| x.as_str()).filter(|u| steam_media_url(u)).unwrap_or("").to_string();
        let pick = |key: &str| -> Option<String> {
            let v = m.get(key)?;
            ["480", "max"].iter().find_map(|q| v.get(*q).and_then(|x| x.as_str())).filter(|u| steam_media_url(u)).map(|u| u.to_string())
        };
        if let Some(u) = pick("mp4") {
            return (u, "mp4".into(), thumb);
        }
        if let Some(u) = pick("webm") {
            return (u, "webm".into(), thumb);
        }
        if let Some(u) = m.get("hls_h264").and_then(|x| x.as_str()).filter(|u| steam_media_url(u)) {
            return (u.to_string(), "hls".into(), thumb);
        }
    }
    Default::default()
}

/// Steam-Deck-Bewertung aus ajaxgetdeckappcompatibilityreport
pub fn parse_deck(json: &str) -> u8 {
    serde_json::from_str::<serde_json::Value>(json)
        .ok()
        .and_then(|v| v.get("results")?.get("resolved_category")?.as_u64())
        .map(|c| c.min(3) as u8)
        .unwrap_or(0)
}

fn steam_lang(lang: &str) -> &'static str {
    match lang {
        "de" => "german",
        "fr" => "french",
        "es" => "spanish",
        "it" => "italian",
        "pt" => "brazilian",
        "pl" => "polish",
        "tr" => "turkish",
        "ru" => "russian",
        _ => "english",
    }
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
                .timeout_global(Some(std::time::Duration::from_secs(12)))
                // Wikipedia verlangt einen aussagekräftigen User-Agent mit Kontakt
                .user_agent(concat!("Open-Game-Launcher/", env!("CARGO_PKG_VERSION"), " (https://github.com/MikeAdamRGB/Open-Game-Launcher)"))
                .build()
                .into()
        })
        .clone()
}

/// GET mit Größenlimit. `None` = keine Verbindung / Fehler
fn get_text(url: &str, limit: u64) -> Option<String> {
    let mut resp = agent().get(url).call().ok()?;
    resp.body_mut().with_config().limit(limit).read_to_string().ok()
}

/// Infos zu einer Steam-AppID laden (in der Sprache des Launchers).
/// `None` = keine Verbindung (später erneut versuchen), `fetched == 0` = nichts gefunden.
pub fn fetch(appid: u32, lang: &str) -> Option<Info> {
    let l = steam_lang(lang);
    let body = get_text(&format!("https://store.steampowered.com/api/appdetails?appids={appid}&l={l}"), 4 * 1024 * 1024)?;
    let mut info = parse_appdetails(appid, &body).unwrap_or(Info { version: VERSION, ..Default::default() });
    if info.fetched > 0 {
        if let Some(d) = get_text(&format!("https://store.steampowered.com/saleaction/ajaxgetdeckappcompatibilityreport?nAppID={appid}&l={l}"), 256 * 1024) {
            info.deck = parse_deck(&d);
        }
    }
    Some(info)
}

// ----------------------------------------------------------------------
// Zweite Quelle: Wikipedia + Wikidata
// ----------------------------------------------------------------------

/// Wörter, an denen man in der Kurzbeschreibung erkennt, dass es um ein Spiel geht
const GAME_WORDS: &[&str] = &[
    "video game", "videogame", "computer game", "videospiel", "computerspiel", "jeu vidéo", "videojuego", "videogioco",
    "jogo eletrônico", "jogo de vídeo", "gra komputerowa", "gra wideo", "video oyunu", "bilgisayar oyunu", "компьютерная игра",
    "видеоигра",
];

/// Name für die Suche säubern ("Spiel™ – Deluxe Edition" → "Spiel")
pub fn search_name(name: &str) -> String {
    let mut n: String = name.chars().filter(|c| !matches!(c, '™' | '®' | '©')).collect();
    for cut in [" - ", " – ", ": Game of the Year", " Deluxe Edition", " Definitive Edition", " GOTY", " Remastered"] {
        if let Some(i) = n.find(cut) {
            if i > 2 {
                n.truncate(i);
            }
        }
    }
    n.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Ersten passenden Artikeltitel aus der Wikipedia-Suche
pub fn pick_search_result(json: &str, name: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let want = crate::model::normalize_name(name);
    let results = v.get("query")?.get("search")?.as_array()?;
    results
        .iter()
        .filter_map(|r| r.get("title")?.as_str())
        .find(|t| {
            let n = crate::model::normalize_name(t.split(" (").next().unwrap_or(t));
            !want.is_empty() && (n == want || n.starts_with(&want) || want.starts_with(&n) && n.len() > 3)
        })
        .map(|t| t.to_string())
}

/// Kurzfassung eines Artikels → (Text, Bild, Wikidata-ID, Seite). Nur, wenn es wirklich ein Spiel ist.
pub fn parse_summary(json: &str) -> Option<(String, String, String, String)> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    if v.get("type").and_then(|t| t.as_str()) == Some("disambiguation") {
        return None;
    }
    let desc = v.get("description").and_then(|d| d.as_str()).unwrap_or("").to_lowercase();
    let extract = v.get("extract").and_then(|d| d.as_str()).unwrap_or("").trim().to_string();
    let head: String = extract.chars().take(300).collect::<String>().to_lowercase();
    if !GAME_WORDS.iter().any(|w| desc.contains(w) || head.contains(w)) {
        return None;
    }
    let thumb = v
        .get("thumbnail")
        .and_then(|t| t.get("source"))
        .and_then(|s| s.as_str())
        .filter(|u| u.starts_with("https://upload.wikimedia.org/"))
        .unwrap_or("")
        .to_string();
    let qid = v.get("wikibase_item").and_then(|q| q.as_str()).unwrap_or("").to_string();
    let page = v
        .get("content_urls")
        .and_then(|c| c.get("desktop"))
        .and_then(|d| d.get("page"))
        .and_then(|p| p.as_str())
        .filter(|u| u.starts_with("https://") && u.contains(".wikipedia.org/"))
        .unwrap_or("")
        .to_string();
    // Lange Texte kürzen (am Satzende)
    let mut text: String = extract.chars().take(700).collect();
    if extract.chars().count() > 700 {
        if let Some(i) = text.rfind(". ") {
            text.truncate(i + 1);
        }
    }
    Some((text, thumb, qid, page))
}

fn valid_qid(q: &str) -> bool {
    q.len() > 1 && q.len() < 15 && q.starts_with('Q') && q[1..].chars().all(|c| c.is_ascii_digit())
}

/// Aus den Wikidata-Aussagen: Genre-IDs (P136), Entwickler-IDs (P178), Erscheinungsjahr (P577)
pub fn parse_wikidata_claims(json: &str, qid: &str) -> (Vec<String>, Vec<String>, String) {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(json) else { return Default::default() };
    let Some(claims) = v.get("entities").and_then(|e| e.get(qid)).and_then(|e| e.get("claims")) else { return Default::default() };
    let ids = |p: &str| -> Vec<String> {
        claims
            .get(p)
            .and_then(|a| a.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|c| c.get("mainsnak")?.get("datavalue")?.get("value")?.get("id")?.as_str())
                    .filter(|q| valid_qid(q))
                    .map(|q| q.to_string())
                    .take(6)
                    .collect()
            })
            .unwrap_or_default()
    };
    let year = claims
        .get("P577")
        .and_then(|a| a.as_array())
        .and_then(|a| {
            a.iter()
                .filter_map(|c| c.get("mainsnak")?.get("datavalue")?.get("value")?.get("time")?.as_str())
                .map(|t| t.trim_start_matches('+').chars().take(4).collect::<String>())
                .filter(|y| y.len() == 4 && y.chars().all(|c| c.is_ascii_digit()))
                .min()
        })
        .unwrap_or_default();
    (ids("P136"), ids("P178"), year)
}

/// Beschriftungen zu Wikidata-IDs (in der Sprache, sonst Englisch)
pub fn parse_labels(json: &str, lang: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let Ok(v) = serde_json::from_str::<serde_json::Value>(json) else { return out };
    for (id, e) in v.get("entities").and_then(|e| e.as_object()).into_iter().flatten() {
        let labels = e.get("labels");
        let l = labels
            .and_then(|l| l.get(lang))
            .or_else(|| labels.and_then(|l| l.get("en")))
            .and_then(|l| l.get("value"))
            .and_then(|v| v.as_str());
        if let Some(l) = l {
            out.insert(id.clone(), l.chars().take(40).collect());
        }
    }
    out
}

fn first_upper(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// Spiel-Infos aus Wikipedia/Wikidata. `None` = keine Verbindung, `fetched == 0` = nichts gefunden
pub fn fetch_wiki(name: &str, lang: &str) -> Option<Info> {
    let name = search_name(name);
    if name.chars().count() < 2 {
        return Some(Info { version: VERSION, ..Default::default() });
    }
    let langs: Vec<&str> = if lang == "en" { vec!["en"] } else { vec![lang, "en"] };
    let mut summary = None;
    let mut reached = false;
    for l in &langs {
        let l = if l.chars().all(|c| c.is_ascii_lowercase()) && l.len() == 2 { *l } else { "en" };
        let q = crate::sys::url_encode(&format!("{name} video game"));
        let Some(search) = get_text(&format!("https://{l}.wikipedia.org/w/api.php?action=query&list=search&srsearch={q}&srlimit=5&format=json&utf8=1"), 512 * 1024) else {
            continue;
        };
        reached = true;
        let Some(title) = pick_search_result(&search, &name) else { continue };
        let t = crate::sys::url_encode(&title.replace(' ', "_"));
        if let Some(s) = get_text(&format!("https://{l}.wikipedia.org/api/rest_v1/page/summary/{t}"), 512 * 1024).as_deref().and_then(parse_summary) {
            summary = Some((l.to_string(), s));
            break;
        }
    }
    if !reached {
        return None;
    }
    let Some((wl, (text, thumb, qid, page))) = summary else {
        return Some(Info { version: VERSION, ..Default::default() });
    };
    let mut info = Info {
        description: text,
        cover: thumb,
        page,
        source: "wikipedia".into(),
        version: VERSION,
        fetched: crate::sys::now_secs(),
        ..Default::default()
    };
    if valid_qid(&qid) {
        if let Some(j) = get_text(&format!("https://www.wikidata.org/wiki/Special:EntityData/{qid}.json"), 8 * 1024 * 1024) {
            let (genres, devs, year) = parse_wikidata_claims(&j, &qid);
            info.release = year;
            let ids: Vec<String> = genres.iter().chain(devs.iter()).cloned().collect();
            if !ids.is_empty() {
                let url = format!(
                    "https://www.wikidata.org/w/api.php?action=wbgetentities&ids={}&props=labels&languages={}|en&format=json",
                    ids.join("|"),
                    wl
                );
                if let Some(lj) = get_text(&url, 1024 * 1024) {
                    let labels = parse_labels(&lj, &wl);
                    info.genres = genres.iter().filter_map(|g| labels.get(g)).map(|g| first_upper(g)).collect();
                    info.developer = devs.iter().filter_map(|d| labels.get(d)).cloned().collect::<Vec<_>>().join(", ");
                }
            }
        }
    }
    Some(info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appdetails() {
        let j = r#"{"570":{"success":true,"data":{"short_description":"Every day, millions &amp; more <b>play</b>.","genres":[{"id":"1","description":"Action"},{"id":"2","description":"Strategy"}],"developers":["Valve"],"release_date":{"date":"9 Jul, 2013"},
            "pc_requirements":{"minimum":"<strong>Minimum:</strong><br><ul class=\"bb_ul\"><li><strong>OS:</strong> Windows 10<br></li><li><strong>Memory:</strong> 4 GB RAM<br></li><li>Broadband</li></ul>","recommended":"<ul><li><strong>Graphics:</strong> GTX 1060</li></ul>"},
            "linux_requirements":[]}}}"#;
        let i = parse_appdetails_for(570, j, "linux_requirements").unwrap();
        assert_eq!(i.description, "Every day, millions & more play.");
        assert_eq!(i.genres, vec!["Action", "Strategy"]);
        assert_eq!(i.developer, "Valve");
        assert_eq!(i.req_min, vec![("OS".into(), "Windows 10".into()), ("Memory".into(), "4 GB RAM".into()), ("".into(), "Broadband".into())]);
        assert_eq!(i.req_rec, vec![("Graphics".into(), "GTX 1060".into())]);
        assert!(i.is_current());
        assert!(parse_appdetails(570, r#"{"570":{"success":false}}"#).is_none());
        let movies: serde_json::Value = serde_json::from_str(r#"{"movies":[
            {"thumbnail":"https://shared.akamai.steamstatic.com/a.jpg","hls_h264":"https://video.akamai.steamstatic.com/a.m3u8"},
            {"highlight":true,"thumbnail":"https://shared.akamai.steamstatic.com/b.jpg","mp4":{"480":"https://video.akamai.steamstatic.com/b.mp4"}},
            {"mp4":{"480":"https://evil.example/c.mp4"}}]}"#).unwrap();
        let (u, k, th) = parse_trailer(&movies);
        assert_eq!((u.as_str(), k.as_str()), ("https://video.akamai.steamstatic.com/b.mp4", "mp4"));
        assert!(th.ends_with("b.jpg"));
        assert!(!steam_media_url("https://evil.example/x.mp4"));
        assert!(!steam_media_url("https://steamstatic.com.evil.example/x.mp4"));
        assert!(!steam_media_url("http://video.akamai.steamstatic.com/x.mp4"));
        assert_eq!(parse_deck(r#"{"success":1,"results":{"appid":570,"resolved_category":3}}"#), 3);
        assert_eq!(parse_deck("kaputt"), 0);
    }

    #[test]
    fn wikipedia() {
        assert_eq!(search_name("The Witcher® 3: Wild Hunt – Game of the Year Edition"), "The Witcher 3: Wild Hunt");
        let s = r#"{"query":{"search":[{"title":"Mario (Figur)"},{"title":"Super Mario World"},{"title":"Super Mario World 2"}]}}"#;
        assert_eq!(pick_search_result(s, "Super Mario World").as_deref(), Some("Super Mario World"));
        assert_eq!(pick_search_result(s, "Zelda"), None);
        let sum = r#"{"type":"standard","description":"1990 video game","extract":"Super Mario World is a platform game.","thumbnail":{"source":"https://upload.wikimedia.org/x.png"},"wikibase_item":"Q1137577","content_urls":{"desktop":{"page":"https://en.wikipedia.org/wiki/Super_Mario_World"}}}"#;
        let (t, img, q, page) = parse_summary(sum).unwrap();
        assert_eq!(t, "Super Mario World is a platform game.");
        assert_eq!(img, "https://upload.wikimedia.org/x.png");
        assert_eq!(q, "Q1137577");
        assert!(page.ends_with("Super_Mario_World"));
        assert!(parse_summary(r#"{"type":"standard","description":"Stadt in Bayern","extract":"Eine Stadt."}"#).is_none());
        // Bild von fremder Seite wird ignoriert
        let evil = r#"{"description":"video game","extract":"x","thumbnail":{"source":"https://evil.example/a.png"}}"#;
        assert_eq!(parse_summary(evil).unwrap().1, "");
        let wd = r#"{"entities":{"Q1":{"claims":{"P136":[{"mainsnak":{"datavalue":{"value":{"id":"Q2"}}}}],"P178":[{"mainsnak":{"datavalue":{"value":{"id":"Q3"}}}}],"P577":[{"mainsnak":{"datavalue":{"value":{"time":"+1991-11-21T00:00:00Z"}}}},{"mainsnak":{"datavalue":{"value":{"time":"+1990-11-21T00:00:00Z"}}}}]}}}}"#;
        assert_eq!(parse_wikidata_claims(wd, "Q1"), (vec!["Q2".to_string()], vec!["Q3".to_string()], "1990".to_string()));
        let labels = parse_labels(r#"{"entities":{"Q2":{"labels":{"en":{"value":"platform game"},"de":{"value":"Jump ’n’ Run"}}}}}"#, "de");
        assert_eq!(labels.get("Q2").map(|s| s.as_str()), Some("Jump ’n’ Run"));
    }
}
