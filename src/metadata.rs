//! Automatische Spiel-Infos (Beschreibung, Genres) aus dem öffentlichen Steam-Store,
//! mit lokalem Zwischenspeicher. Für Spiele aus anderen Stores wird die passende
//! Steam-Seite über den Namen gesucht (gleiche Zuordnung wie bei den Covern).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Info {
    pub description: String,
    pub genres: Vec<String>,
    pub developer: String,
    pub release: String,
    /// Wann geladen (0 = nichts gefunden)
    pub fetched: u64,
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
        let _ = std::fs::write(p, t);
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
    out.replace("&amp;", "&").replace("&quot;", "\"").replace("&#39;", "'").replace("&lt;", "<").replace("&gt;", ">").replace("&nbsp;", " ").trim().to_string()
}

/// Antwort von store.steampowered.com/api/appdetails auswerten
pub fn parse_appdetails(appid: u32, json: &str) -> Option<Info> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let d = v.get(appid.to_string())?;
    if d.get("success").and_then(|s| s.as_bool()) != Some(true) {
        return None;
    }
    let data = d.get("data")?;
    let s = |k: &str| data.get(k).and_then(|x| x.as_str()).map(plain).unwrap_or_default();
    Some(Info {
        description: s("short_description"),
        genres: data
            .get("genres")
            .and_then(|g| g.as_array())
            .map(|a| a.iter().filter_map(|g| g.get("description").and_then(|x| x.as_str()).map(|x| x.to_string())).collect())
            .unwrap_or_default(),
        developer: data
            .get("developers")
            .and_then(|d| d.as_array())
            .and_then(|a| a.first())
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
        release: data.get("release_date").and_then(|r| r.get("date")).and_then(|x| x.as_str()).unwrap_or("").to_string(),
        fetched: crate::sys::now_secs(),
    })
}

/// Infos zu einer Steam-AppID laden (in der Sprache des Launchers).
/// `None` = keine Verbindung (später erneut versuchen), `fetched == 0` = nichts gefunden.
pub fn fetch(appid: u32, lang: &str) -> Option<Info> {
    let l = match lang {
        "de" => "german",
        "fr" => "french",
        "es" => "spanish",
        "it" => "italian",
        "pt" => "brazilian",
        "pl" => "polish",
        "tr" => "turkish",
        "ru" => "russian",
        _ => "english",
    };
    let url = format!("https://store.steampowered.com/api/appdetails?appids={appid}&l={l}");
    let agent = crate::steam::http_agent();
    let mut resp = agent.get(&url).call().ok()?;
    // Antwort begrenzen (normal sind es wenige KB)
    let body = resp.body_mut().with_config().limit(2 * 1024 * 1024).read_to_string().ok()?;
    Some(parse_appdetails(appid, &body).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appdetails() {
        let j = r#"{"570":{"success":true,"data":{"short_description":"Every day, millions &amp; more <b>play</b>.","genres":[{"id":"1","description":"Action"},{"id":"2","description":"Strategy"}],"developers":["Valve"],"release_date":{"date":"9 Jul, 2013"}}}}"#;
        let i = parse_appdetails(570, j).unwrap();
        assert_eq!(i.description, "Every day, millions & more play.");
        assert_eq!(i.genres, vec!["Action", "Strategy"]);
        assert_eq!(i.developer, "Valve");
        assert!(parse_appdetails(570, r#"{"570":{"success":false}}"#).is_none());
    }
}
