//! Sicherung und Wiederherstellung aller OG-Einstellungen als eine Datei (.ogbackup).
//!
//! SICHERHEIT: Schlüssel und Konten (Steam-Web-API-Schlüssel, CurseForge-Schlüssel,
//! Epic-Anmeldung) kommen nie in die Sicherung – die Datei kann man also bedenkenlos
//! auf einen anderen PC mitnehmen. Beim Wiederherstellen bleiben die vorhandenen Schlüssel.

use crate::model::Settings;
use serde::{Deserialize, Serialize};

pub const FORMAT: &str = "og-backup";
pub const EXTENSION: &str = "ogbackup";
const MAX_SIZE: usize = 50 * 1024 * 1024;

#[derive(Serialize, Deserialize)]
struct BackupFile {
    format: String,
    version: u32,
    created: u64,
    app_version: String,
    settings: serde_json::Value,
}

/// Kopie ohne Schlüssel und Konten
fn without_secrets(s: &Settings) -> Settings {
    let mut c = s.clone();
    c.steam_api_key.clear();
    c.curseforge_key.clear();
    c.epic_auth = None;
    c
}

pub fn create(s: &Settings, now: u64) -> Result<String, String> {
    let settings = serde_json::to_value(without_secrets(s)).map_err(|e| e.to_string())?;
    let file = BackupFile { format: FORMAT.into(), version: 1, created: now, app_version: crate::sys::VERSION.into(), settings };
    serde_json::to_string_pretty(&file).map_err(|e| e.to_string())
}

/// Sicherung lesen. Gibt die Einstellungen aus der Datei zurück (ohne Schlüssel).
pub fn read(text: &str) -> Result<(Settings, u64), String> {
    if text.len() > MAX_SIZE {
        return Err(crate::i18n::t("backup.err_format"));
    }
    let f: BackupFile = serde_json::from_str(text).map_err(|_| crate::i18n::t("backup.err_format"))?;
    if f.format != FORMAT || f.version == 0 || f.version > 1 {
        return Err(crate::i18n::t("backup.err_format"));
    }
    let s: Settings = serde_json::from_value(f.settings).map_err(|_| crate::i18n::t("backup.err_format"))?;
    Ok((without_secrets(&s), f.created))
}

/// Wiederherstellen: alles aus der Sicherung, aber die Schlüssel/Konten von diesem PC behalten.
pub fn restore(current: &Settings, from_backup: &Settings) -> Settings {
    let mut s = from_backup.clone();
    s.steam_api_key = current.steam_api_key.clone();
    s.curseforge_key = current.curseforge_key.clone();
    s.epic_auth = current.epic_auth.clone();
    s
}

/// Kurze Übersicht für den Bestätigungsdialog
pub fn summary(s: &Settings) -> Vec<(String, usize)> {
    use crate::i18n::t;
    vec![
        (t("backup.sum.custom"), s.custom_games.len()),
        (t("backup.sum.tags"), s.tags.values().filter(|v| !v.is_empty()).count()),
        (t("backup.sum.collections"), s.collections.len()),
        (t("backup.sum.playtime"), s.playtime_local.len()),
        (t("backup.sum.status"), s.status.len()),
        (t("backup.sum.favorites"), s.favorites.len()),
        (t("backup.sum.themes"), s.custom_themes.len()),
    ]
}

/// Vorschlag für den Dateinamen
pub fn file_name(now: u64) -> String {
    let (y, m, d, _, _) = crate::util::civil(now as i64);
    format!("open-game-launcher-{y:04}-{m:02}-{d:02}.{EXTENSION}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_without_secrets() {
        let mut s = Settings::default();
        s.steam_api_key = "GEHEIM1234567890GEHEIM1234567890".into();
        s.curseforge_key = "$2a$10$geheimgeheimgeheim".into();
        s.favorites.push("steam:10".into());
        s.tags.insert("steam:10".into(), vec!["Koop".into()]);
        s.status.insert("steam:10".into(), "finished".into());
        let text = create(&s, 1_700_000_000).unwrap();
        assert!(!text.contains("GEHEIM") && !text.contains("geheim"), "keine Schlüssel in der Datei");
        let (back, created) = read(&text).unwrap();
        assert_eq!(created, 1_700_000_000);
        assert_eq!(back.favorites, s.favorites);
        assert_eq!(back.status_of("steam:10"), "finished");
        // Schlüssel vom aktuellen PC bleiben
        let mut here = Settings::default();
        here.steam_api_key = "HIER".into();
        assert_eq!(restore(&here, &back).steam_api_key, "HIER");
        assert!(read("{}").is_err());
        assert!(read(r#"{"format":"anders","version":1,"created":0,"app_version":"","settings":{}}"#).is_err());
        assert!(file_name(0).starts_with("open-game-launcher-1970-01-01"));
    }
}
