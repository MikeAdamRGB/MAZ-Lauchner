//! Erweiterung „Legacy Games“:
//! - installierte Spiele aus der Windows-Programmliste (Herausgeber „Legacy Games“)
//! - gekaufte Spiele aus dem Legacy Games Launcher, soweit er sie lokal speichert
//!   (%APPDATA%\legacy-games-launcher\*.json). Anmeldedaten werden nie gelesen oder weitergegeben.
use maz_extension_sdk::*;
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Default)]
struct Installed {
    name: String,
    dir: String,
    exe: String,
}

/// Ausgabe von `reg query … /s` in Einträge zerlegen
fn parse_reg(text: &str) -> Vec<Vec<(String, String)>> {
    let mut out = Vec::new();
    let mut cur: Vec<(String, String)> = Vec::new();
    for line in text.lines() {
        if line.starts_with("HKEY_") {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            continue;
        }
        let l = line.trim();
        for ty in ["REG_SZ", "REG_EXPAND_SZ"] {
            if let Some(i) = l.find(&format!("    {ty}    ")) {
                let name = l[..i].trim().to_string();
                let val = l[i + ty.len() + 8..].trim().to_string();
                cur.push((name, val));
            }
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn get<'a>(e: &'a [(String, String)], k: &str) -> Option<&'a str> {
    e.iter().find(|(n, _)| n.eq_ignore_ascii_case(k)).map(|(_, v)| v.as_str()).filter(|v| !v.is_empty())
}

fn main_exe(dir: &Path, icon: Option<&str>) -> Option<PathBuf> {
    if let Some(i) = icon {
        let p = PathBuf::from(i.split(',').next().unwrap_or("").trim().trim_matches('"'));
        if p.extension().map(|x| x.eq_ignore_ascii_case("exe")).unwrap_or(false) && p.starts_with(dir) && p.is_file() {
            return Some(p);
        }
    }
    let mut exes: Vec<PathBuf> = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x.eq_ignore_ascii_case("exe")).unwrap_or(false))
        .filter(|p| {
            let n = p.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
            !n.starts_with("unins") && !n.contains("setup") && !n.contains("crash")
        })
        .collect();
    exes.sort_by_key(|p| std::fs::metadata(p).map(|m| std::cmp::Reverse(m.len())).unwrap_or(std::cmp::Reverse(0)));
    exes.into_iter().next()
}

fn installed_from(entries: &[Vec<(String, String)>]) -> Vec<Installed> {
    let mut out: Vec<Installed> = Vec::new();
    for e in entries {
        let publisher = get(e, "Publisher").unwrap_or("");
        if !publisher.to_lowercase().contains("legacy games") {
            continue;
        }
        let (Some(name), Some(dir)) = (get(e, "DisplayName"), get(e, "InstallLocation")) else { continue };
        let dir = dir.trim_matches('"').trim_end_matches(['\\', '/']);
        let Some(exe) = main_exe(Path::new(dir), get(e, "DisplayIcon")) else { continue };
        if !out.iter().any(|o| o.name == name) {
            out.push(Installed { name: name.to_string(), dir: dir.to_string(), exe: exe.to_string_lossy().to_string() });
        }
    }
    out
}

#[cfg(windows)]
fn reg_text() -> String {
    let mut all = String::new();
    for key in [
        r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall",
        r"HKLM\Software\Microsoft\Windows\CurrentVersion\Uninstall",
        r"HKLM\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
    ] {
        let reg = std::env::var_os("SystemRoot").map(|r| PathBuf::from(r).join("System32").join("reg.exe")).unwrap_or_else(|| "reg.exe".into());
        if let Ok(o) = std::process::Command::new(reg).args(["query", key, "/s"]).output() {
            all.push_str(&String::from_utf8_lossy(&o.stdout));
        }
    }
    all
}

#[cfg(not(windows))]
fn reg_text() -> String {
    String::new()
}

/// Spielnamen aus den JSON-Dateien des Launchers (Objekte mit "game_name")
fn owned_names(json: &str) -> Vec<(String, Option<String>)> {
    fn walk(v: &Value, out: &mut Vec<(String, Option<String>)>, depth: usize) {
        if depth > 12 || out.len() > 3000 {
            return;
        }
        match v {
            Value::Object(m) => {
                if let Some(n) = m.get("game_name").and_then(|x| x.as_str()) {
                    let cover = ["game_coverart", "game_cover", "cover", "image"]
                        .iter()
                        .find_map(|k| m.get(*k).and_then(|x| x.as_str()))
                        .filter(|u| u.starts_with("https://") && u.len() < 500)
                        .map(|u| u.to_string());
                    let n = n.trim();
                    if !n.is_empty() && !out.iter().any(|(x, _)| x == n) {
                        out.push((n.to_string(), cover));
                    }
                }
                for (k, x) in m {
                    // Nichts aus Konto-/Anmeldebereichen lesen
                    let k = k.to_lowercase();
                    if k.contains("token") || k.contains("password") || k.contains("auth") || k == "user" {
                        continue;
                    }
                    walk(x, out, depth + 1);
                }
            }
            Value::Array(a) => a.iter().for_each(|x| walk(x, out, depth + 1)),
            _ => {}
        }
    }
    let v: Value = serde_json::from_str(json).unwrap_or(Value::Null);
    let mut out = Vec::new();
    walk(&v, &mut out, 0);
    out
}

fn id_for(name: &str) -> String {
    name.chars().filter(|c| c.is_ascii_alphanumeric()).take(80).collect::<String>().to_lowercase()
}

struct Legacy;

impl Extension for Legacy {
    fn hello(&mut self, _h: &Hello) -> HelloReply {
        HelloReply { name: "Legacy Games".into(), version: "1.0.0".into(), kinds: vec!["library".into(), "action".into()] }
    }
    fn games(&mut self) -> Result<Vec<GameInfo>, ExtError> {
        Ok(installed_from(&parse_reg(&reg_text()))
            .into_iter()
            .map(|g| GameInfo {
                id: id_for(&g.name),
                name: g.name,
                install_dir: Some(g.dir),
                launch: Launch { exe: Some(ExeLaunch { path: g.exe, args: Vec::new() }), ..Default::default() },
                ..Default::default()
            })
            .collect())
    }
    fn owned(&mut self) -> Result<Vec<GameInfo>, ExtError> {
        let Some(dir) = std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("legacy-games-launcher")) else { return Ok(Vec::new()) };
        let mut names: Vec<(String, Option<String>)> = Vec::new();
        for e in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let p = e.path();
            if p.extension().map(|x| x == "json").unwrap_or(false) && std::fs::metadata(&p).map(|m| m.len() < 20 * 1024 * 1024).unwrap_or(false) {
                for n in owned_names(&std::fs::read_to_string(&p).unwrap_or_default()) {
                    if !names.iter().any(|(x, _)| *x == n.0) {
                        names.push(n);
                    }
                }
            }
        }
        Ok(names
            .into_iter()
            .map(|(name, cover)| GameInfo {
                id: id_for(&name),
                name,
                launch: Launch { uri: Some("https://legacygames.com/".into()), ..Default::default() },
                cover,
                ..Default::default()
            })
            .collect())
    }
    fn actions(&mut self, game: &GameRef) -> Result<Vec<Action>, ExtError> {
        Ok(if game.install_dir.is_some() { vec![Action { id: "open-folder".into(), label: "@open_folder".into() }] } else { Vec::new() })
    }
    fn run_action(&mut self, action: &str, game: &GameRef) -> Result<ActionResult, ExtError> {
        match action {
            "open-folder" => Ok(ActionResult::OpenFolder { path: game.install_dir.clone().unwrap_or_default() }),
            _ => Err(ExtError::unsupported()),
        }
    }
}

fn main() {
    run(Legacy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_and_json() {
        let dir = std::env::temp_dir().join(format!("legacy-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("Game.exe"), vec![0u8; 100]).unwrap();
        std::fs::write(dir.join("unins000.exe"), vec![0u8; 1000]).unwrap();
        let d = dir.to_string_lossy();
        let text = format!(
            "HKEY_CURRENT_USER\\Software\\X\\Uninstall\\Game1\n    DisplayName    REG_SZ    Mystery Case\n    Publisher    REG_SZ    Legacy Games\n    InstallLocation    REG_SZ    {d}\n\nHKEY_CURRENT_USER\\Software\\X\\Uninstall\\Other\n    DisplayName    REG_SZ    Anderes\n    Publisher    REG_SZ    Jemand\n"
        );
        let entries = parse_reg(&text);
        assert_eq!(entries.len(), 2);
        let inst = installed_from(&entries);
        assert_eq!(inst.len(), 1);
        assert!(inst[0].exe.ends_with("Game.exe"), "Deinstaller wird ignoriert");
        let json = r#"{"user":{"token":"x","game_name":"Geheim"},"siteData":{"catalog":[{"game_name":"Spiel 1","game_coverart":"https://a/b.jpg"},{"game_name":"Spiel 1"},{"game_name":"Spiel 2","game_coverart":"http://x"}]}}"#;
        let names = owned_names(json);
        assert_eq!(names.len(), 2);
        assert_eq!(names[0].1.as_deref(), Some("https://a/b.jpg"));
        assert!(names.iter().all(|(n, _)| n != "Geheim"), "Kontodaten werden nicht gelesen");
        assert_eq!(id_for("Mystery Case: Part 2"), "mysterycasepart2");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
