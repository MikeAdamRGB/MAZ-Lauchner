//! Einstellungen laden und speichern (%APPDATA%\Open Game Launcher\config.json).

use crate::model::Settings;
use crate::sys;
use std::fs;
use std::path::PathBuf;

pub fn config_path() -> PathBuf {
    sys::roaming_dir().join("config.json")
}

pub fn load() -> Settings {
    let path = config_path();
    let Ok(text) = fs::read_to_string(&path) else {
        return Settings::default();
    };
    match parse(&text) {
        Some(s) => s,
        None => {
            // Kaputte Datei sichern statt überschreiben – so geht nichts verloren.
            let backup = path.with_extension(format!("defekt-{}.json", sys::now_secs()));
            let _ = fs::copy(&path, backup);
            Settings::default()
        }
    }
}

/// Einstellungen aus Text lesen – auch alte Versionen: fehlende Felder bekommen
/// Standardwerte, unbekannte (z. B. vom früheren Konto) werden ignoriert.
pub fn parse(text: &str) -> Option<Settings> {
    let mut s = serde_json::from_str::<Settings>(text).ok()?;
    // Alte deutsche Standardliste → übersetzte Standardliste verwenden
    if s.checklist.iter().map(|x| x.as_str()).eq(crate::model::OLD_GERMAN_CHECKLIST.iter().copied()) {
        s.checklist.clear();
    }
    // Unsinnige Werte aus Handarbeit an der Datei abfangen
    if !(1..=3).contains(&s.card_size) {
        s.card_size = 2;
    }
    s.stores_off.sort();
    s.stores_off.dedup();
    s.collections.retain(|c| !c.id.is_empty());
    for list in s.tags.values_mut() {
        list.retain(|t| !t.trim().is_empty());
    }
    s.tags.retain(|_, v| !v.is_empty());
    Some(s)
}

pub fn save(settings: &Settings) -> std::io::Result<()> {
    // Oberflächen-Test: echte Einstellungen nie überschreiben
    if crate::uitest::active() {
        return Ok(());
    }
    let dir = sys::roaming_dir();
    fs::create_dir_all(&dir)?;
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    // Erst in eine Temp-Datei schreiben, dann umbenennen → nie halb geschriebene Datei.
    let tmp = dir.join("config.json.tmp");
    sys::write_private(&tmp, json.as_bytes())?;
    fs::rename(&tmp, config_path())
}

/// Kleine eindeutige ID für eigene Spiele.
pub fn new_id() -> String {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("custom:{:x}", t ^ (std::process::id() as u128) << 64)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Einstellungen aus 1.3/1.4 (mit Konto-Feldern, ohne neue Felder) müssen weiter laden
    #[test]
    fn migrate_old_config() {
        let old = r#"{
            "language": "de",
            "theme": "neon",
            "layout": "steam",
            "card_size": 3,
            "steam_enabled": true,
            "epic_enabled": false,
            "favorites": ["steam:570"],
            "hidden": ["epic:abc"],
            "last_played": {"steam:570": 1700000000},
            "custom_games": [{"id": "custom:1", "name": "Mein Spiel", "exe": "C:\\Spiele\\a.exe", "admin": true}],
            "account": {"user": "x"},
            "friends_enabled": true,
            "supabase_session": "egal"
        }"#;
        let s = parse(old).expect("alte Datei muss laden");
        assert_eq!(s.theme, "neon");
        assert_eq!(s.layout, "steam");
        assert!(!s.epic_enabled);
        assert_eq!(s.favorites, vec!["steam:570"]);
        assert_eq!(s.custom_games[0].name, "Mein Spiel");
        assert!(s.custom_games[0].admin);
        // Neue Felder mit Standardwerten
        assert!(s.merge_duplicates);
        assert!(!s.controller_mode);
        assert!(s.stores_off.is_empty());
        assert!(s.store_on(crate::model::Source::Gog));
        assert!(s.emulators.is_empty() && s.collections.is_empty() && s.tags.is_empty());
        assert!(s.roblox_enabled && s.update_check);
    }

    #[test]
    fn roundtrip_new_fields() {
        let mut s = Settings::default();
        s.stores_off = vec!["ea".into(), "ea".into()];
        s.tags.insert("steam:570".into(), vec!["Koop".into(), " ".into()]);
        s.collections.push(crate::model::Collection { id: "c1".into(), name: "Mit Freunden".into(), games: vec!["steam:570".into()] });
        s.emulators.push(crate::model::Emulator { id: "e1".into(), name: "Dolphin".into(), extensions: vec!["iso".into()], ..Default::default() });
        s.playtime_local.insert("gog:1".into(), 42);
        s.card_size = 9;
        let back = parse(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(back.stores_off, vec!["ea"]);
        assert!(!back.store_on(crate::model::Source::Ea));
        assert_eq!(back.tags.get("steam:570").unwrap(), &vec!["Koop".to_string()]);
        assert!(back.in_collection("c1", "steam:570"));
        assert_eq!(back.emulators[0].extensions, vec!["iso"]);
        assert_eq!(back.playtime_local.get("gog:1"), Some(&42));
        assert_eq!(back.card_size, 2);
        assert!(parse("kaputt").is_none());
    }
}
