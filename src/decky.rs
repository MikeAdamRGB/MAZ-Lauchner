//! Steam Deck: Spieleliste für das Decky-Plugin („Open Game Launcher“ im Schnellzugriffsmenü).
//!
//! Der Launcher schreibt eine kleine Datei `decky-games.json` in seinen Einstellungsordner.
//! Das Plugin liest sie (nur lesen) und startet ein Spiel über eine Steam-Verknüpfung mit
//! `--play <id>`. Es stehen nur Name, ID, Spielzeit und Favorit drin – keine Pfade zu
//! Spielen, keine Schlüssel, keine Konten.

use crate::model::{Game, Settings};
use serde_json::json;

pub const FILE_NAME: &str = "decky-games.json";
const MAX_GAMES: usize = 400;

/// Inhalt der Datei
pub fn export_json(games: &[Game], s: &Settings) -> Option<String> {
    let (exe, start_dir, prefix) = crate::steamshortcut::self_command()?;
    let mut list: Vec<&Game> = games.iter().filter(|g| !s.is_hidden(&g.id) && crate::steamshortcut::valid_play_id(&g.id)).collect();
    // Zuletzt gespielt zuerst, dann Favoriten, dann nach Name
    list.sort_by(|a, b| {
        s.last_played(&b.id)
            .cmp(&s.last_played(&a.id))
            .then(s.is_favorite(&b.id).cmp(&s.is_favorite(&a.id)))
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    let items: Vec<serde_json::Value> = list
        .into_iter()
        .take(MAX_GAMES)
        .map(|g| {
            let name: String = g.name.chars().filter(|c| !c.is_control()).take(120).collect();
            json!({
                "id": g.id,
                "name": name,
                "installed": g.installed,
                "favorite": s.is_favorite(&g.id),
                "last_played": s.last_played(&g.id),
                "minutes": s.playtime(g),
                "source": g.source.label(),
            })
        })
        .collect();
    let v = json!({
        "format": "og-decky",
        "version": 1,
        "launcher": { "exe": exe, "start_dir": start_dir, "prefix": prefix, "app_version": crate::sys::VERSION },
        "games": items,
    });
    serde_json::to_string(&v).ok()
}

/// Datei schreiben (nur unter Linux, nur wenn eingeschaltet). Ausgeschaltet: Datei entfernen.
pub fn write(games: &[Game], s: &Settings) {
    if !cfg!(target_os = "linux") || crate::uitest::active() {
        return;
    }
    let path = crate::sys::roaming_dir().join(FILE_NAME);
    if !s.decky_export {
        let _ = std::fs::remove_file(&path);
        return;
    }
    let Some(text) = export_json(games, s) else { return };
    if std::fs::read_to_string(&path).map(|old| old == text).unwrap_or(false) {
        return;
    }
    let tmp = path.with_extension("json.tmp");
    if crate::sys::write_private(&tmp, text.as_bytes()).is_ok() {
        let _ = std::fs::rename(&tmp, &path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Art, LaunchTarget, Source};

    #[test]
    fn export_has_no_paths_or_secrets() {
        let g = Game {
            id: "steam:440".into(),
            name: "Team\u{7}Fortress 2".into(),
            source: Source::Steam,
            installed: true,
            install_dir: Some("/geheim/pfad".into()),
            exe: None,
            launch: LaunchTarget::Uri("steam://run/440".into()),
            art: Art::default(),
            size_bytes: None,
            playtime_min: Some(90),
            steam_appid: Some(440),
        };
        let mut bad = g.clone();
        bad.id = "x;rm -rf".into();
        let s = Settings { steam_api_key: "SECRET".into(), ..Default::default() };
        let text = export_json(&[g, bad], &s).unwrap();
        assert!(!text.contains("SECRET"));
        assert!(!text.contains("/geheim/pfad"));
        assert!(!text.contains("rm -rf"));
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["format"], "og-decky");
        assert_eq!(v["games"][0]["name"], "TeamFortress 2");
        assert_eq!(v["games"].as_array().unwrap().len(), 1);
    }
}
