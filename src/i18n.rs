//! Mehrsprachigkeit.
//!
//! Sprachdateien liegen in `lang/<code>.txt` (eine Zeile = `schlüssel = text`)
//! und werden in die EXE eingebettet. Zusätzlich lädt der Launcher eigene
//! Sprachdateien aus `%APPDATA%\MAZ Launcher\lang\` – so kann jeder neue
//! Sprachen hinzufügen, ohne neu zu kompilieren.
//!
//! Platzhalter: `{name}` usw. – werden mit [`tf`] ersetzt.
//! Zeilenumbruch im Text: `\n`. Kommentare beginnen mit `#`.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// (Code, Name in der eigenen Sprache, Inhalt)
const BUILTIN: &[(&str, &str, &str)] = &[
    ("de", "Deutsch", include_str!("../lang/de.txt")),
    ("en", "English", include_str!("../lang/en.txt")),
    ("es", "Español", include_str!("../lang/es.txt")),
    ("fr", "Français", include_str!("../lang/fr.txt")),
    ("it", "Italiano", include_str!("../lang/it.txt")),
    ("pt", "Português", include_str!("../lang/pt.txt")),
    ("pl", "Polski", include_str!("../lang/pl.txt")),
    ("tr", "Türkçe", include_str!("../lang/tr.txt")),
    ("ru", "Русский", include_str!("../lang/ru.txt")),
];

struct Lang {
    code: String,
    map: HashMap<String, String>,
    /// Englisch als Ersatz für fehlende Schlüssel
    fallback: HashMap<String, String>,
}

static CURRENT: RwLock<Option<Arc<Lang>>> = RwLock::new(None);

pub fn parse(text: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in text.trim_start_matches('\u{feff}').lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let v = v.trim().replace("\\n", "\n");
            map.insert(k.trim().to_string(), v);
        }
    }
    map
}

fn user_lang_dir() -> std::path::PathBuf {
    crate::sys::roaming_dir().join("lang")
}

/// Eigene Sprachdateien des Nutzers: (Code, Name, Inhalt)
fn user_languages() -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(user_lang_dir()) else { return out };
    for e in rd.flatten() {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()) != Some("txt") {
            continue;
        }
        let Some(code) = p.file_stem().and_then(|s| s.to_str()).map(|s| s.to_lowercase()) else { continue };
        let Ok(text) = std::fs::read_to_string(&p) else { continue };
        let name = parse(&text).get("lang.name").cloned().unwrap_or_else(|| code.clone());
        out.push((code, name, text));
    }
    out
}

/// Alle verfügbaren Sprachen: (Code, Name)
pub fn available() -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = BUILTIN.iter().map(|(c, n, _)| (c.to_string(), n.to_string())).collect();
    for (code, name, _) in user_languages() {
        if !v.iter().any(|(c, _)| *c == code) {
            v.push((code, name));
        }
    }
    v
}

/// Eingebaute Sprachdatei (z. B. als Vorlage für eigene Übersetzungen)
pub fn builtin(code: &str) -> Option<&'static str> {
    builtin_text(code)
}

fn builtin_text(code: &str) -> Option<&'static str> {
    BUILTIN.iter().find(|(c, _, _)| *c == code).map(|(_, _, t)| *t)
}

/// Sprache der Windows-Oberfläche erkennen (z. B. "de", "en").
pub fn system_language() -> String {
    #[cfg(windows)]
    {
        #[link(name = "kernel32")]
        extern "system" {
            fn GetUserDefaultUILanguage() -> u16;
        }
        let id = unsafe { GetUserDefaultUILanguage() } & 0x3ff;
        let code = match id {
            0x07 => "de",
            0x09 => "en",
            0x0a => "es",
            0x0c => "fr",
            0x10 => "it",
            0x16 => "pt",
            0x15 => "pl",
            0x1f => "tr",
            0x19 => "ru",
            _ => "en",
        };
        code.to_string()
    }
    #[cfg(not(windows))]
    {
        #[cfg_attr(not(target_os = "macos"), allow(unused_mut))]
        let mut candidates: Vec<String> = ["LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE"]
            .iter()
            .filter_map(|v| std::env::var(v).ok())
            .collect();
        // macOS: Programme aus dem Finder bekommen kein LANG → Systemeinstellung lesen
        #[cfg(target_os = "macos")]
        if let Ok(out) = std::process::Command::new("/usr/bin/defaults").args(["read", "-g", "AppleLanguages"]).output() {
            let text = String::from_utf8_lossy(&out.stdout).to_string();
            if let Some(first) = text.split('"').nth(1).or_else(|| text.lines().nth(1).map(|l| l.trim().trim_end_matches(','))) {
                candidates.insert(0, first.to_string());
            }
        }
        candidates
            .into_iter()
            .map(|l| l.chars().take(2).collect::<String>().to_lowercase())
            .find(|l| builtin_text(l).is_some())
            .unwrap_or_else(|| "en".into())
    }
}

/// Sprache setzen ("auto" = Windows-Sprache).
pub fn set_language(code: &str) {
    let code = if code.is_empty() || code == "auto" { system_language() } else { code.to_lowercase() };
    let mut map = HashMap::new();
    if let Some(t) = builtin_text(&code) {
        map = parse(t);
    }
    // Eigene Datei ergänzt/überschreibt die eingebaute
    if let Some((_, _, text)) = user_languages().into_iter().find(|(c, _, _)| *c == code) {
        map.extend(parse(&text));
    }
    let mut fallback = parse(builtin_text("en").unwrap_or(""));
    if map.is_empty() {
        map = parse(builtin_text("de").unwrap_or(""));
    }
    // Deutsch als letzte Rettung, falls auch Englisch etwas fehlt
    for (k, v) in parse(builtin_text("de").unwrap_or("")) {
        fallback.entry(k).or_insert(v);
    }
    let lang = Lang { code, map, fallback };
    if let Ok(mut w) = CURRENT.write() {
        *w = Some(Arc::new(lang));
    }
}

fn current() -> Arc<Lang> {
    if let Ok(r) = CURRENT.read() {
        if let Some(l) = r.as_ref() {
            return l.clone();
        }
    }
    set_language("auto");
    CURRENT.read().ok().and_then(|r| r.clone()).unwrap_or_else(|| {
        Arc::new(Lang { code: "de".into(), map: HashMap::new(), fallback: HashMap::new() })
    })
}

pub fn current_code() -> String {
    current().code.clone()
}

/// Übersetzung holen. Fehlt der Schlüssel, wird Englisch/Deutsch oder der Schlüssel selbst gezeigt.
pub fn t(key: &str) -> String {
    let l = current();
    l.map
        .get(key)
        .or_else(|| l.fallback.get(key))
        .cloned()
        .unwrap_or_else(|| key.to_string())
}

/// Übersetzung mit Platzhaltern: `tf("toast.started", &[("name", "Doom")])`
pub fn tf(key: &str, args: &[(&str, &str)]) -> String {
    let mut s = t(key);
    for (k, v) in args {
        s = s.replace(&format!("{{{k}}}"), v);
    }
    s
}

/// Liste aus einem mehrzeiligen Eintrag (eine Zeile = ein Punkt)
pub fn tl(key: &str) -> Vec<String> {
    t(key).lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

/// Wie [`tl`], mit Platzhaltern
pub fn tlf(key: &str, args: &[(&str, &str)]) -> Vec<String> {
    tf(key, args).lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

pub fn user_lang_folder() -> std::path::PathBuf {
    user_lang_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lines() {
        let m = parse("# Kommentar\na = Hallo {name}\nb=Zeile1\\nZeile2\n\nkaputt");
        assert_eq!(m.get("a").unwrap(), "Hallo {name}");
        assert_eq!(m.get("b").unwrap(), "Zeile1\nZeile2");
        assert!(m.get("kaputt").is_none());
    }

    #[test]
    fn all_languages_have_all_german_keys() {
        let de = parse(builtin_text("de").unwrap());
        for (code, _, text) in BUILTIN {
            let m = parse(text);
            let missing: Vec<_> = de.keys().filter(|k| !m.contains_key(*k)).collect();
            assert!(missing.is_empty(), "Sprache {code}: fehlende Schlüssel {missing:?}");
            // Platzhalter müssen gleich sein
            for (k, v) in &de {
                for part in v.split('{').skip(1) {
                    if let Some(name) = part.split('}').next() {
                        assert!(m[k].contains(&format!("{{{name}}}")), "Sprache {code}: Platzhalter {{{name}}} fehlt in {k}");
                    }
                }
            }
        }
    }
}
