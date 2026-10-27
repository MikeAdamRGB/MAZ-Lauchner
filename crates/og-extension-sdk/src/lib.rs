//! SDK für Erweiterungen des Open Game Launchers (Protokoll Version 1).
//!
//! Eine Erweiterung ist ein eigenes Programm. Der Launcher schreibt pro Zeile eine Anfrage
//! (JSON) auf stdin, die Erweiterung antwortet pro Zeile auf stdout. Siehe docs/EXTENSIONS.md.
//!
//! ```no_run
//! use og_extension_sdk::*;
//! struct Mine;
//! impl Extension for Mine {
//!     fn hello(&mut self, _h: &Hello) -> HelloReply {
//!         HelloReply { name: "Meine".into(), version: "1.0.0".into(), kinds: vec!["library".into()] }
//!     }
//!     fn games(&mut self) -> Result<Vec<GameInfo>, ExtError> { Ok(vec![]) }
//! }
//! fn main() { run(Mine) }
//! ```

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, Write};

pub const API_VERSION: u32 = 1;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Hello {
    pub api_version: u32,
    pub os: String,
    pub lang: String,
    #[serde(default)]
    pub network: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct HelloReply {
    pub name: String,
    pub version: String,
    pub kinds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Launch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exe: Option<ExeLaunch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub appx: Option<AppxLaunch>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExeLaunch {
    pub path: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppxLaunch {
    pub family: String,
    pub app: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct GameInfo {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_dir: Option<String>,
    pub launch: Launch,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hero: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Metadata {
    pub description: String,
    pub genres: Vec<String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub developer: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub release: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct GameRef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub install_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Action {
    pub id: String,
    /// Text oder "@schlüssel" aus lang/<sprache>.txt
    pub label: String,
}

/// Erlaubte Ergebnisse einer Aktion
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActionResult {
    OpenFolder { path: String },
    OpenUrl { url: String },
    Message { text: String },
}

#[derive(Debug, Clone)]
pub struct ExtError {
    pub code: String,
    pub message: String,
}

impl ExtError {
    pub fn new(code: &str, message: &str) -> Self {
        ExtError { code: code.into(), message: message.into() }
    }
    pub fn unsupported() -> Self {
        ExtError::new("unsupported", "not supported")
    }
}

/// Das implementiert eine Erweiterung. Nicht benötigte Aufrufe haben Standard-Antworten.
pub trait Extension {
    fn hello(&mut self, hello: &Hello) -> HelloReply;
    fn games(&mut self) -> Result<Vec<GameInfo>, ExtError> {
        Ok(Vec::new())
    }
    fn owned(&mut self) -> Result<Vec<GameInfo>, ExtError> {
        Err(ExtError::unsupported())
    }
    fn metadata(&mut self, _name: &str, _id: Option<&str>) -> Result<Option<Metadata>, ExtError> {
        Ok(None)
    }
    fn actions(&mut self, _game: &GameRef) -> Result<Vec<Action>, ExtError> {
        Ok(Vec::new())
    }
    fn run_action(&mut self, _action: &str, _game: &GameRef) -> Result<ActionResult, ExtError> {
        Err(ExtError::unsupported())
    }
}

fn handle<E: Extension>(ext: &mut E, req: &Value) -> Result<Value, ExtError> {
    let call = req.get("call").and_then(|c| c.as_str()).unwrap_or("");
    let game = || serde_json::from_value::<GameRef>(req.get("game").cloned().unwrap_or(Value::Null)).unwrap_or_default();
    match call {
        "hello" => {
            let h: Hello = serde_json::from_value(req.clone()).unwrap_or_default();
            Ok(serde_json::to_value(ext.hello(&h)).unwrap_or(Value::Null))
        }
        "games" => Ok(json!({ "games": ext.games()? })),
        "owned" => Ok(json!({ "games": ext.owned()? })),
        "metadata" => {
            let name = req.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let id = req.get("id").and_then(|n| n.as_str());
            Ok(ext.metadata(name, id)?.map(|m| serde_json::to_value(m).unwrap_or(Value::Null)).unwrap_or(json!({})))
        }
        "actions" => Ok(json!({ "actions": ext.actions(&game())? })),
        "run_action" => {
            let action = req.get("action").and_then(|n| n.as_str()).unwrap_or("").to_string();
            Ok(serde_json::to_value(ext.run_action(&action, &game())?).unwrap_or(Value::Null))
        }
        _ => Err(ExtError::new("unknown_call", call)),
    }
}

/// Hauptschleife: Anfragen lesen, beantworten. Endet, sobald der Launcher stdin schließt.
pub fn run<E: Extension>(mut ext: E) {
    let stdin = std::io::stdin();
    let mut out = std::io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let Ok(req) = serde_json::from_str::<Value>(&line) else { continue };
        let id = req.get("id").cloned().unwrap_or(Value::Null);
        let reply = match handle(&mut ext, &req) {
            Ok(result) => json!({ "id": id, "result": result }),
            Err(e) => json!({ "id": id, "error": { "code": e.code, "message": e.message } }),
        };
        if writeln!(out, "{reply}").and_then(|_| out.flush()).is_err() {
            break;
        }
    }
}
