//! Erweiterung „Humble App“: liest die Spieleliste der Humble App
//! (%APPDATA%\Humble App\config.json, Eintrag "game-collection-4").
//! Es werden nur Spielname, Ordner, Startdatei und Bild gelesen – nie Anmeldedaten.
use maz_extension_sdk::*;
use serde_json::Value;
use std::path::{Path, PathBuf};

fn config_path() -> Option<PathBuf> {
    let appdata = std::env::var_os("APPDATA")?;
    Some(PathBuf::from(appdata).join("Humble App").join("config.json"))
}

#[derive(Debug, Clone, PartialEq)]
struct Entry {
    machine: String,
    name: String,
    dir: Option<String>,
    exe: Option<String>,
    image: Option<String>,
    installed: bool,
}

fn s(v: &Value, k: &str) -> Option<String> {
    v.get(k).and_then(|x| x.as_str()).map(|x| x.trim().to_string()).filter(|x| !x.is_empty())
}

fn safe_id(id: &str) -> String {
    id.chars().filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')).take(90).collect()
}

/// Spiele aus config.json
fn parse(json: &str) -> Vec<Entry> {
    let v: Value = serde_json::from_str(json).unwrap_or(Value::Null);
    let list = v.get("game-collection-4").and_then(|l| l.as_array()).cloned().unwrap_or_default();
    let mut out = Vec::new();
    for g in list.iter().take(5000) {
        let Some(name) = s(g, "gameName") else { continue };
        let machine = s(g, "downloadMachineName").or_else(|| s(g, "machineName")).unwrap_or_default();
        let machine = safe_id(&machine);
        if machine.is_empty() {
            continue;
        }
        let status = s(g, "status").unwrap_or_default().to_lowercase();
        let dir = s(g, "filePath");
        let exe = s(g, "executablePath");
        let installed = matches!(status.as_str(), "downloaded" | "installed") && dir.as_deref().map(|d| Path::new(d).is_dir()).unwrap_or(false);
        let image = s(g, "imagePath").or_else(|| s(g, "iconPath")).filter(|u| u.starts_with("https://") && u.len() < 500);
        out.push(Entry { machine, name, dir, exe, image, installed });
    }
    out
}

fn to_game(e: &Entry) -> GameInfo {
    let uri = Launch { uri: Some(format!("humble://launch/{}", e.machine)), ..Default::default() };
    let launch = match (&e.dir, &e.exe) {
        (Some(d), Some(x)) if e.installed => {
            let p = Path::new(d).join(x);
            // Nur Programme innerhalb des Spielordners (prüft auch der Launcher)
            if p.is_file() && p.starts_with(d) {
                Launch { exe: Some(ExeLaunch { path: p.to_string_lossy().to_string(), args: Vec::new() }), ..Default::default() }
            } else {
                uri
            }
        }
        _ => uri,
    };
    GameInfo {
        id: e.machine.clone(),
        name: e.name.clone(),
        install_dir: if e.installed { e.dir.clone() } else { None },
        launch,
        cover: e.image.clone(),
        ..Default::default()
    }
}

struct Humble;

impl Humble {
    fn entries(&self) -> Vec<Entry> {
        config_path().and_then(|p| std::fs::read_to_string(p).ok()).map(|t| parse(&t)).unwrap_or_default()
    }
}

impl Extension for Humble {
    fn hello(&mut self, _h: &Hello) -> HelloReply {
        HelloReply { name: "Humble App".into(), version: "1.0.0".into(), kinds: vec!["library".into(), "action".into()] }
    }
    fn games(&mut self) -> Result<Vec<GameInfo>, ExtError> {
        Ok(self.entries().iter().filter(|e| e.installed).map(to_game).collect())
    }
    fn owned(&mut self) -> Result<Vec<GameInfo>, ExtError> {
        Ok(self.entries().iter().filter(|e| !e.installed).map(to_game).collect())
    }
    fn actions(&mut self, game: &GameRef) -> Result<Vec<Action>, ExtError> {
        let mut a = vec![Action { id: "open-app".into(), label: "@open_app".into() }];
        if game.install_dir.is_some() {
            a.push(Action { id: "open-folder".into(), label: "@open_folder".into() });
        }
        Ok(a)
    }
    fn run_action(&mut self, action: &str, game: &GameRef) -> Result<ActionResult, ExtError> {
        let id = game.id.rsplit(':').next().unwrap_or("");
        match action {
            "open-folder" => Ok(ActionResult::OpenFolder { path: game.install_dir.clone().unwrap_or_default() }),
            "open-app" => Ok(ActionResult::OpenUrl { url: format!("humble://launch/{}", safe_id(id)) }),
            _ => Err(ExtError::unsupported()),
        }
    }
}

fn main() {
    run(Humble)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_config() {
        let dir = std::env::temp_dir();
        let d = dir.to_string_lossy().to_string();
        let json = format!(
            r#"{{"user":{{"token":"GEHEIM"}},"game-collection-4":[
                {{"gameName":"Spiel A","downloadMachineName":"spiel_a","status":"installed","filePath":{d:?},"executablePath":"gibtsnicht.exe","imagePath":"https://hb.imgix.net/a.png"}},
                {{"gameName":"Spiel B","machineName":"spiel_b","status":"available","imagePath":"http://unsicher"}},
                {{"gameName":"Ohne ID"}}]}}"#
        );
        let e = parse(&json);
        assert_eq!(e.len(), 2);
        assert!(e[0].installed);
        assert!(!e[1].installed);
        assert_eq!(e[1].image, None, "nur https-Bilder");
        let g = to_game(&e[0]);
        assert_eq!(g.launch.uri.as_deref(), Some("humble://launch/spiel_a"), "Startdatei fehlt → über die Humble App");
        assert!(!format!("{:?}", e).contains("GEHEIM"));
        assert_eq!(safe_id("a/../b c"), "a..bc");
        assert!(parse("kaputt").is_empty());
    }
}
