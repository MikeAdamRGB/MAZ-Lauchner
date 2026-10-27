//! Spiele zu Steam hinzufügen („Nicht-Steam-Spiel“): schreibt einen Eintrag in Steams
//! `userdata/<Konto>/config/shortcuts.vdf` (binäres VDF-Format).
//!
//! Gestartet wird über den Open Game Launcher (`--play <id>`), damit Sicherheitsfrage,
//! Starthilfen und Spielzeit-Messung genauso funktionieren wie im Launcher.
//! Steam muss dafür geschlossen sein (Steam überschreibt die Datei beim Beenden).
//! Vor jeder Änderung wird eine Sicherung `shortcuts.vdf.og-backup` angelegt.

use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Map(Vec<(String, Node)>),
    Str(String),
    Int(i32),
}

impl Node {
    fn get(&self, key: &str) -> Option<&Node> {
        match self {
            Node::Map(m) => m.iter().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v),
            _ => None,
        }
    }
    fn str(&self, key: &str) -> Option<&str> {
        match self.get(key) {
            Some(Node::Str(s)) => Some(s),
            _ => None,
        }
    }
}

fn read_cstr(data: &[u8], pos: &mut usize) -> Option<String> {
    let start = *pos;
    let end = start + data.get(start..)?.iter().position(|b| *b == 0)?;
    *pos = end + 1;
    Some(String::from_utf8_lossy(&data[start..end]).to_string())
}

fn parse_map(data: &[u8], pos: &mut usize, depth: usize) -> Option<Vec<(String, Node)>> {
    if depth > 16 {
        return None;
    }
    let mut out = Vec::new();
    loop {
        let t = *data.get(*pos)?;
        *pos += 1;
        match t {
            0x08 => return Some(out),
            0x00 => {
                let k = read_cstr(data, pos)?;
                let v = parse_map(data, pos, depth + 1)?;
                out.push((k, Node::Map(v)));
            }
            0x01 => {
                let k = read_cstr(data, pos)?;
                let v = read_cstr(data, pos)?;
                out.push((k, Node::Str(v)));
            }
            0x02 => {
                let k = read_cstr(data, pos)?;
                let b = data.get(*pos..*pos + 4)?;
                *pos += 4;
                out.push((k, Node::Int(i32::from_le_bytes([b[0], b[1], b[2], b[3]]))));
            }
            _ => return None,
        }
    }
}

/// Ganze Datei lesen (leere Datei = keine Verknüpfungen)
pub fn parse(data: &[u8]) -> Option<Node> {
    if data.is_empty() {
        return Some(Node::Map(vec![("shortcuts".into(), Node::Map(Vec::new()))]));
    }
    let mut pos = 0;
    let root = parse_map(data, &mut pos, 0)?;
    Some(Node::Map(root))
}

fn write_map(out: &mut Vec<u8>, m: &[(String, Node)]) {
    for (k, v) in m {
        match v {
            Node::Map(c) => {
                out.push(0x00);
                out.extend_from_slice(k.as_bytes());
                out.push(0);
                write_map(out, c);
            }
            Node::Str(s) => {
                out.push(0x01);
                out.extend_from_slice(k.as_bytes());
                out.push(0);
                // Nullbytes würden die Datei zerstören
                out.extend(s.bytes().filter(|b| *b != 0));
                out.push(0);
            }
            Node::Int(i) => {
                out.push(0x02);
                out.extend_from_slice(k.as_bytes());
                out.push(0);
                out.extend_from_slice(&i.to_le_bytes());
            }
        }
    }
    out.push(0x08);
}

pub fn write(root: &Node) -> Vec<u8> {
    let mut out = Vec::new();
    if let Node::Map(m) = root {
        write_map(&mut out, m);
    }
    out
}

/// CRC-32 (wie Steam es für die Kennung von Nicht-Steam-Spielen nutzt)
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for b in data {
        crc ^= *b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
    }
    !crc
}

/// Steam-Kennung einer Verknüpfung (für Bilder im Ordner `grid`)
pub fn shortcut_appid(exe: &str, name: &str) -> u32 {
    crc32(format!("{exe}{name}").as_bytes()) | 0x8000_0000
}

#[derive(Clone, Debug, PartialEq)]
pub struct Shortcut {
    pub name: String,
    /// Programm (wird in Anführungszeichen gesetzt)
    pub exe: String,
    pub start_dir: String,
    pub icon: String,
    pub launch_options: String,
}

/// Eintrag hinzufügen. Gibt false zurück, wenn es ihn (gleiches Programm + Parameter) schon gibt.
pub fn add(root: &mut Node, sc: &Shortcut) -> bool {
    let Node::Map(top) = root else { return false };
    if !top.iter().any(|(k, _)| k.eq_ignore_ascii_case("shortcuts")) {
        top.push(("shortcuts".into(), Node::Map(Vec::new())));
    }
    let Some((_, Node::Map(list))) = top.iter_mut().find(|(k, _)| k.eq_ignore_ascii_case("shortcuts")) else { return false };
    let exe_q = format!("\"{}\"", sc.exe);
    let exists = list.iter().any(|(_, e)| {
        e.str("Exe").map(|x| x.trim_matches('"') == sc.exe).unwrap_or(false) && e.str("LaunchOptions").unwrap_or("") == sc.launch_options
    });
    if exists {
        return false;
    }
    let next = list.iter().filter_map(|(k, _)| k.parse::<u32>().ok()).max().map(|n| n + 1).unwrap_or(0);
    let appid = shortcut_appid(&exe_q, &sc.name) as i32;
    let s = |v: &str| Node::Str(v.to_string());
    let entry = vec![
        ("appid".to_string(), Node::Int(appid)),
        ("AppName".into(), s(&sc.name)),
        ("Exe".into(), s(&exe_q)),
        ("StartDir".into(), s(&format!("\"{}\"", sc.start_dir))),
        ("icon".into(), s(&sc.icon)),
        ("ShortcutPath".into(), s("")),
        ("LaunchOptions".into(), s(&sc.launch_options)),
        ("IsHidden".into(), Node::Int(0)),
        ("AllowDesktopConfig".into(), Node::Int(1)),
        ("AllowOverlay".into(), Node::Int(1)),
        ("OpenVR".into(), Node::Int(0)),
        ("Devkit".into(), Node::Int(0)),
        ("DevkitGameID".into(), s("")),
        ("DevkitOverrideAppID".into(), Node::Int(0)),
        ("LastPlayTime".into(), Node::Int(0)),
        ("FlatpakAppID".into(), s("")),
        ("tags".into(), Node::Map(Vec::new())),
    ];
    list.push((next.to_string(), Node::Map(entry)));
    true
}

/// Datei `shortcuts.vdf` des angemeldeten Steam-Kontos
pub fn shortcuts_file() -> Option<PathBuf> {
    let root = crate::steam::find_steam_root()?;
    let (sid, _) = crate::steam::logged_in_user(&root)?;
    let account = sid.parse::<u64>().ok()? & 0xFFFF_FFFF;
    let dir = root.join("userdata").join(account.to_string()).join("config");
    dir.is_dir().then(|| dir.join("shortcuts.vdf"))
}

/// Läuft Steam gerade?
pub fn steam_running() -> bool {
    crate::playtime::process_paths().iter().any(|p| {
        let f = p.replace('\\', "/").rsplit('/').next().unwrap_or("").to_lowercase();
        f == "steam.exe" || f == "steam" || f == "steamwebhelper" || f == "steamwebhelper.exe"
    })
}

#[derive(Clone, Debug, PartialEq)]
pub enum AddResult {
    Added,
    AlreadyThere,
}

/// Verknüpfung in Steam anlegen (Steam muss geschlossen sein – prüft der Aufrufer)
pub fn add_to_file(file: &Path, sc: &Shortcut) -> Result<AddResult, String> {
    let data = if file.exists() { std::fs::read(file).map_err(|e| e.to_string())? } else { Vec::new() };
    if data.len() > 20 * 1024 * 1024 {
        return Err("shortcuts.vdf too big".into());
    }
    let mut root = parse(&data).ok_or_else(|| crate::i18n::t("steamsc.err_format"))?;
    if !add(&mut root, sc) {
        return Ok(AddResult::AlreadyThere);
    }
    if file.exists() {
        std::fs::copy(file, file.with_extension("vdf.og-backup")).map_err(|e| e.to_string())?;
    }
    let tmp = file.with_extension("vdf.og-tmp");
    std::fs::write(&tmp, write(&root)).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, file).map_err(|e| e.to_string())?;
    Ok(AddResult::Added)
}

/// Wie Steam den Open Game Launcher starten soll (Programm, Startordner, Parameter davor)
pub fn self_command() -> Option<(String, String, String)> {
    if let Ok(id) = std::env::var("FLATPAK_ID") {
        // Nur bekannte IDs (die Startoptionen landen in Steam)
        let id = if id == crate::flatpakid::OLD_ID { crate::flatpakid::OLD_ID } else { crate::flatpakid::NEW_ID };
        return Some(("flatpak".into(), String::new(), format!("run {id}")));
    }
    let exe = crate::sys::env_path("APPIMAGE").or_else(|| std::env::current_exe().ok())?;
    let dir = exe.parent().map(|d| d.to_string_lossy().to_string()).unwrap_or_default();
    Some((exe.to_string_lossy().to_string(), dir, String::new()))
}

/// Verknüpfung für ein Spiel aus dem Launcher (startet über `--play <id>`)
pub fn for_game(id: &str, name: &str, icon: &str) -> Option<Shortcut> {
    let (exe, dir, pre) = self_command()?;
    let args = format!("{pre} --play {id}").trim().to_string();
    Some(Shortcut { name: name.to_string(), exe, start_dir: dir, icon: icon.to_string(), launch_options: args })
}

/// Gültige Spiel-ID für `--play` (keine Leerzeichen, keine Anführungszeichen)
pub fn valid_play_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 200 && id.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, ':' | '-' | '_' | '.' | '/'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_add() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        let mut root = parse(&[]).unwrap();
        let sc = Shortcut { name: "Mein Spiel".into(), exe: "C:\\OG\\og.exe".into(), start_dir: "C:\\OG".into(), icon: String::new(), launch_options: "--play custom:1".into() };
        assert!(add(&mut root, &sc));
        assert!(!add(&mut root, &sc), "nicht doppelt");
        let sc2 = Shortcut { launch_options: "--play custom:2".into(), name: "Zwei".into(), ..sc.clone() };
        assert!(add(&mut root, &sc2));
        let bytes = write(&root);
        let back = parse(&bytes).unwrap();
        assert_eq!(back, root);
        let list = back.get("shortcuts").unwrap();
        assert_eq!(list.get("1").and_then(|e| e.str("AppName")), Some("Zwei"));
        assert_eq!(list.get("0").and_then(|e| e.str("Exe")), Some("\"C:\\OG\\og.exe\""));
        assert!(parse(&[0x00, b'x']).is_none(), "kaputte Datei");
        assert!(valid_play_id("steam:620") && !valid_play_id("a b") && !valid_play_id("\"x"));
        // Datei schreiben + Sicherung
        let dir = std::env::temp_dir().join(format!("og-steamsc-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("shortcuts.vdf");
        std::fs::write(&f, &bytes).unwrap();
        let sc3 = Shortcut { launch_options: "--play x".into(), ..sc };
        assert_eq!(add_to_file(&f, &sc3), Ok(AddResult::Added));
        assert!(dir.join("shortcuts.vdf.og-backup").exists());
        assert_eq!(add_to_file(&f, &sc3), Ok(AddResult::AlreadyThere));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
