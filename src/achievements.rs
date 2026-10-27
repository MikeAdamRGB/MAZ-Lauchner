//! Erfolge (Achievements).
//!
//! - **Steam:** über die offizielle Web-API mit dem Schlüssel, der in den Einstellungen schon
//!   eingetragen ist (GetSchemaForGame + GetPlayerAchievements). Das Steam-Profil muss
//!   „Spieldetails: öffentlich“ haben.
//! - **GOG:** soweit GOG Galaxy 2.0 die Zahl lokal speichert (freigeschaltet / gesamt).
//! - **Epic:** Epic bietet dafür keine offene Schnittstelle – es gibt einen Knopf, der die
//!   Erfolge im Epic Games Launcher öffnet.

use std::collections::BTreeMap;
use std::sync::Mutex;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Achievement {
    pub name: String,
    pub description: String,
    pub icon: String,
    pub icon_gray: String,
    pub achieved: bool,
    /// Unix-Sekunden (0 = unbekannt)
    pub unlocked_at: u64,
    pub hidden: bool,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Summary {
    pub unlocked: usize,
    pub total: usize,
    pub list: Vec<Achievement>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// Kein Web-API-Schlüssel oder keine Steam-ID
    NoKey,
    /// Profil/Spieldetails nicht öffentlich
    Private,
    /// Spiel hat keine Erfolge
    None,
    Network(String),
}

fn cache() -> &'static Mutex<BTreeMap<u32, (u64, Summary)>> {
    static C: std::sync::OnceLock<Mutex<BTreeMap<u32, (u64, Summary)>>> = std::sync::OnceLock::new();
    C.get_or_init(|| Mutex::new(BTreeMap::new()))
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

fn https_steam(u: &str) -> String {
    if crate::metadata::steam_media_url(u) {
        u.to_string()
    } else {
        String::new()
    }
}

/// Antworten von GetSchemaForGame und GetPlayerAchievements zusammenführen
pub fn parse_steam(schema: &str, player: &str) -> Result<Summary, Error> {
    let p: serde_json::Value = serde_json::from_str(player).map_err(|_| Error::Network("JSON".into()))?;
    let stats = p.get("playerstats").ok_or_else(|| Error::Network("JSON".into()))?;
    if stats.get("success").and_then(|x| x.as_bool()) == Some(false) {
        let msg = stats.get("error").and_then(|x| x.as_str()).unwrap_or("").to_lowercase();
        return Err(if msg.contains("no stats") || msg.contains("no achievements") { Error::None } else { Error::Private });
    }
    let mine: BTreeMap<String, (bool, u64)> = stats
        .get("achievements")
        .and_then(|a| a.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|x| {
                    let n = x.get("apiname")?.as_str()?.to_string();
                    let done = x.get("achieved").and_then(|v| v.as_u64()).unwrap_or(0) == 1;
                    let t = x.get("unlocktime").and_then(|v| v.as_u64()).unwrap_or(0);
                    Some((n, (done, t)))
                })
                .collect()
        })
        .unwrap_or_default();
    let s: serde_json::Value = serde_json::from_str(schema).unwrap_or(serde_json::Value::Null);
    let defs = s.pointer("/game/availableGameStats/achievements").and_then(|a| a.as_array()).cloned().unwrap_or_default();
    let mut list: Vec<Achievement> = defs
        .iter()
        .take(2000)
        .filter_map(|d| {
            let api = d.get("name")?.as_str()?;
            let (achieved, unlocked_at) = mine.get(api).copied().unwrap_or((false, 0));
            let str_of = |k: &str| d.get(k).and_then(|v| v.as_str()).unwrap_or("").chars().take(300).collect::<String>();
            Some(Achievement {
                name: { let n = str_of("displayName"); if n.is_empty() { api.to_string() } else { n } },
                description: str_of("description"),
                icon: https_steam(&str_of("icon")),
                icon_gray: https_steam(&str_of("icongray")),
                achieved,
                unlocked_at,
                hidden: d.get("hidden").and_then(|v| v.as_u64()).unwrap_or(0) == 1,
            })
        })
        .collect();
    // Ohne Schema (selten): nur die eigenen Einträge zählen
    if list.is_empty() && !mine.is_empty() {
        list = mine.iter().map(|(n, (a, t))| Achievement { name: n.clone(), achieved: *a, unlocked_at: *t, ..Default::default() }).collect();
    }
    if list.is_empty() {
        return Err(Error::None);
    }
    // Freigeschaltete zuerst (neueste oben), dann die übrigen
    list.sort_by(|a, b| b.achieved.cmp(&a.achieved).then(b.unlocked_at.cmp(&a.unlocked_at)));
    let unlocked = list.iter().filter(|a| a.achieved).count();
    Ok(Summary { unlocked, total: list.len(), list })
}

/// Erfolge eines Steam-Spiels laden (für 10 Minuten zwischengespeichert)
pub fn steam(appid: u32, key: &str, steam_id: &str, lang: &str) -> Result<Summary, Error> {
    let key = key.trim();
    let sid = steam_id.trim();
    if key.is_empty() || sid.is_empty() || !sid.chars().all(|c| c.is_ascii_digit()) {
        return Err(Error::NoKey);
    }
    let now = crate::sys::now_secs();
    if let Ok(c) = cache().lock() {
        if let Some((t, s)) = c.get(&appid) {
            if now.saturating_sub(*t) < 600 {
                return Ok(s.clone());
            }
        }
    }
    let agent = crate::steam::http_agent();
    let fetch = |url: String| -> Result<String, Error> {
        let mut r = agent.get(&url).config().http_status_as_error(false).build().call().map_err(|e| Error::Network(e.to_string()))?;
        r.body_mut().with_config().limit(8 * 1024 * 1024).read_to_string().map_err(|e| Error::Network(e.to_string()))
    };
    let k = crate::sys::url_encode(key);
    let player = fetch(format!("https://api.steampowered.com/ISteamUserStats/GetPlayerAchievements/v1/?key={k}&steamid={sid}&appid={appid}"))?;
    let schema = fetch(format!(
        "https://api.steampowered.com/ISteamUserStats/GetSchemaForGame/v2/?key={k}&appid={appid}&l={}",
        steam_lang(lang)
    ))?;
    let s = parse_steam(&schema, &player)?;
    if let Ok(mut c) = cache().lock() {
        c.insert(appid, (now, s.clone()));
    }
    Ok(s)
}

/// GOG Galaxy: freigeschaltet / gesamt, soweit Galaxy das lokal speichert
pub fn parse_galaxy_count(value: &str) -> Option<(usize, usize)> {
    let v: serde_json::Value = serde_json::from_str(value).ok()?;
    let obj = v.as_object()?;
    let mut unlocked = None;
    let mut total = None;
    for (k, val) in obj {
        let Some(n) = val.as_u64() else { continue };
        let k = k.to_lowercase();
        if k.contains("unlock") {
            unlocked = Some(n as usize);
        } else if k.contains("all") || k.contains("total") || k == "count" {
            total = Some(n as usize);
        }
    }
    let (u, t) = (unlocked?, total?);
    (t > 0 && u <= t).then_some((u, t))
}

#[cfg(windows)]
pub fn gog(id: &str) -> Option<(usize, usize)> {
    if !id.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let db = crate::sys::env_path("ProgramData")?.join("GOG.com").join("Galaxy").join("storage").join("galaxy-2.0.db");
    let conn = crate::stores::open_db(&db)?;
    let sql = "SELECT gp.value FROM GamePieces gp JOIN GamePieceTypes t ON t.id = gp.gamePieceTypeId \
               WHERE gp.releaseKey = ?1 AND t.type LIKE '%chievement%' LIMIT 5";
    let mut st = conn.prepare(sql).ok()?;
    let rows: Vec<String> = st.query_map([format!("gog_{id}")], |r| r.get::<_, String>(0)).ok()?.flatten().collect();
    rows.iter().find_map(|v| parse_galaxy_count(v))
}

#[cfg(not(windows))]
pub fn gog(_id: &str) -> Option<(usize, usize)> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steam_parsing() {
        let schema = r#"{"game":{"availableGameStats":{"achievements":[
            {"name":"A1","displayName":"Erster Schritt","description":"Starte","icon":"https://steamcdn-a.akamaihd.net/a.jpg","icongray":"https://evil.example/g.jpg","hidden":0},
            {"name":"A2","displayName":"Geheim","hidden":1}]}}}"#;
        let player = r#"{"playerstats":{"success":true,"achievements":[{"apiname":"A1","achieved":1,"unlocktime":100},{"apiname":"A2","achieved":0,"unlocktime":0}]}}"#;
        let s = parse_steam(schema, player).unwrap();
        assert_eq!((s.unlocked, s.total), (1, 2));
        assert_eq!(s.list[0].name, "Erster Schritt");
        assert!(s.list[0].icon.starts_with("https://"));
        assert!(s.list[0].icon_gray.is_empty(), "fremde Bild-Adressen werden verworfen");
        assert!(s.list[1].hidden);
        let private = r#"{"playerstats":{"error":"Profile is not public","success":false}}"#;
        assert_eq!(parse_steam(schema, private), Err(Error::Private));
        let none = r#"{"playerstats":{"error":"Requested app has no stats","success":false}}"#;
        assert_eq!(parse_steam(schema, none), Err(Error::None));
        assert_eq!(steam(1, "", "1", "de"), Err(Error::NoKey));
    }

    #[test]
    fn galaxy_counts() {
        assert_eq!(parse_galaxy_count(r#"{"unlockedCount":5,"allCount":30}"#), Some((5, 30)));
        assert_eq!(parse_galaxy_count(r#"{"unlocked":2,"total":2}"#), Some((2, 2)));
        assert_eq!(parse_galaxy_count(r#"{"unlocked":5}"#), None);
        assert_eq!(parse_galaxy_count("kaputt"), None);
    }
}
