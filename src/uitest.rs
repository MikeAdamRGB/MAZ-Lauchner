//! Oberflächen-Test für die CI (`--ui-test <ordner>`): Der Launcher startet mit festen
//! Beispiel-Spielen (ohne Netzwerk, ohne echte Bibliothek), prüft im Fenster ein paar Dinge
//! und schreibt das Ergebnis nach `<ordner>/ui-test.json`. Die CI macht dann ein Bildschirmfoto.

use crate::model::{Art, Game, LaunchTarget, Source};
use std::path::PathBuf;
use std::sync::OnceLock;

pub static DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn active() -> bool {
    DIR.get().is_some()
}

/// Feste Beispiel-Spiele (immer gleich → vergleichbare Bildschirmfotos)
pub fn demo_games() -> Vec<Game> {
    let names = [
        ("Abenteuer im Nebel", Source::Steam, true),
        ("Bergrennen 3", Source::Epic, true),
        ("Codeknacker", Source::Gog, false),
        ("Drachenflug", Source::Custom, true),
        ("Eiswelt", Source::Steam, false),
        ("Funkenschlag", Source::Xbox, true),
        ("Galaxie der Stille", Source::Emulator, true),
        ("Hafenstadt", Source::Itch, false),
    ];
    names
        .iter()
        .enumerate()
        .map(|(i, (n, src, inst))| Game {
            id: format!("uitest:{i}"),
            name: n.to_string(),
            source: *src,
            installed: *inst,
            install_dir: None,
            exe: None,
            launch: LaunchTarget::Uri("https://example.invalid/".into()),
            art: Art::default(),
            size_bytes: None,
            playtime_min: Some(60 * (i as u64 + 1)),
            steam_appid: None,
        })
        .collect()
}

/// Prüfungen im Fenster (JavaScript) – meldet sich über /maz/uitest zurück
pub const CHECK_JS: &str = r#"
(function(){
  var cards = document.querySelectorAll('[role=button][aria-label]').length;
  var h1 = document.querySelector('h1');
  var errors = document.querySelectorAll('[role=alert]').length;
  var nav = document.querySelectorAll('button').length;
  var q = 'cards=' + cards + '&buttons=' + nav + '&alerts=' + errors + '&title=' + encodeURIComponent(h1 ? h1.textContent : '');
  fetch('/maz/uitest?' + q).catch(function(){});
})();
"#;

/// Ergebnis schreiben (aus der Antwort des Fensters)
pub fn write_result(query: &str) {
    let Some(dir) = DIR.get() else { return };
    let mut map = serde_json::Map::new();
    for part in query.split('&') {
        let mut kv = part.splitn(2, '=');
        let (k, v) = (kv.next().unwrap_or(""), crate::util::url_decode(kv.next().unwrap_or("")));
        if !k.is_empty() && k.len() < 20 {
            let val = v.parse::<u64>().map(serde_json::Value::from).unwrap_or_else(|_| serde_json::Value::from(v.chars().take(200).collect::<String>()));
            map.insert(k.to_string(), val);
        }
    }
    map.insert("version".into(), crate::sys::VERSION.into());
    let _ = std::fs::create_dir_all(dir);
    let _ = std::fs::write(dir.join("ui-test.json"), serde_json::Value::Object(map).to_string());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_is_stable() {
        let a = demo_games();
        assert_eq!(a.len(), 8);
        assert_eq!(a, demo_games());
        let dir = std::env::temp_dir().join(format!("og-uitest-{}", std::process::id()));
        let _ = DIR.set(dir.clone());
        write_result("cards=8&buttons=40&alerts=0&title=Alle%20Spiele");
        let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("ui-test.json")).unwrap()).unwrap();
        assert_eq!(v["cards"], 8);
        assert_eq!(v["title"], "Alle Spiele");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
