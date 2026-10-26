//! manifest.json einer Erweiterung lesen und streng prüfen.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

/// Diese API-Version spricht der Launcher
pub const API_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtKind {
    Library,
    Metadata,
    Action,
}

/// "run": entweder ein Pfad für alle Systeme oder je System einer
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Run {
    All(String),
    PerOs(BTreeMap<String, String>),
}

#[derive(Clone, Debug, PartialEq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Permissions {
    #[serde(default)]
    pub read_paths: Vec<String>,
    #[serde(default)]
    pub network: bool,
    /// Zusätzliche Link-Arten (z. B. "mystore"), die der Nutzer bestätigen muss
    #[serde(default)]
    pub uri_schemes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub homepage: String,
    pub run: Run,
    pub api_version: u32,
    pub kinds: Vec<ExtKind>,
    #[serde(default)]
    pub permissions: Permissions,
    /// Sprachen, die unter lang/ mitgeliefert werden (z. B. ["de", "en"])
    #[serde(default)]
    pub languages: Vec<String>,
    /// Vorbereitet für spätere Signaturen (minisign/ed25519) – wird in Version 1 nicht erzwungen
    #[serde(default)]
    pub signature: String,
}

/// Fehler beim Prüfen – Schlüssel für den erklärenden Text
#[derive(Clone, Debug, PartialEq)]
pub enum ManifestError {
    Missing,
    TooBig,
    Json(String),
    BadId,
    BadVersion,
    BadText(&'static str),
    NoKinds,
    NoRun,
    BadRun(String),
    BadScheme(String),
    BadPath(String),
    Api(u32),
}

impl ManifestError {
    /// Verständliche Erklärung (Sprache des Launchers)
    pub fn explain(&self) -> String {
        use crate::i18n::{t, tf};
        match self {
            ManifestError::Missing => t("ext.err.missing"),
            ManifestError::TooBig => t("ext.err.too_big"),
            ManifestError::Json(e) => tf("ext.err.json", &[("error", e)]),
            ManifestError::BadId => t("ext.err.id"),
            ManifestError::BadVersion => t("ext.err.version"),
            ManifestError::BadText(f) => tf("ext.err.text", &[("field", f)]),
            ManifestError::NoKinds => t("ext.err.kinds"),
            ManifestError::NoRun => t("ext.err.no_run"),
            ManifestError::BadRun(p) => tf("ext.err.run", &[("path", p)]),
            ManifestError::BadScheme(s) => tf("ext.err.scheme", &[("scheme", s)]),
            ManifestError::BadPath(p) => tf("ext.err.read_path", &[("path", p)]),
            ManifestError::Api(v) => tf("ext.err.api", &[("version", &v.to_string()), ("supported", &API_VERSION.to_string())]),
        }
    }
}

pub fn valid_id(id: &str) -> bool {
    (2..=40).contains(&id.len())
        && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !id.starts_with('-')
        && !id.ends_with('-')
}

/// SemVer "1.2.3" (optional "-beta.1")
pub fn valid_semver(v: &str) -> bool {
    let (core, pre) = v.split_once('-').unwrap_or((v, ""));
    let parts: Vec<&str> = core.split('.').collect();
    parts.len() == 3
        && parts.iter().all(|p| !p.is_empty() && p.len() <= 6 && p.chars().all(|c| c.is_ascii_digit()))
        && pre.len() <= 20
        && pre.chars().all(|c| c.is_ascii_alphanumeric() || c == '.')
        && v.len() <= 32
}

pub fn os_key() -> &'static str {
    if cfg!(windows) {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    }
}

/// Relativer Pfad ohne "..", ohne Laufwerk, ohne absolute Teile
pub fn safe_relative(p: &str) -> Option<PathBuf> {
    if p.is_empty() || p.len() > 200 || p.contains('\0') || p.contains(':') || p.starts_with('/') || p.starts_with('\\') {
        return None;
    }
    let path = PathBuf::from(p.replace('\\', "/"));
    if path.components().all(|c| matches!(c, Component::Normal(_))) {
        Some(path)
    } else {
        None
    }
}

/// Erlaubte Platzhalter in read_paths
fn expand_placeholder(p: &str) -> Option<PathBuf> {
    let home = crate::sys::home_dir();
    if let Some(rest) = p.strip_prefix("~/") {
        return safe_relative(rest).map(|r| home.join(r));
    }
    for var in ["APPDATA", "LOCALAPPDATA", "PROGRAMDATA", "XDG_DATA_HOME", "XDG_CONFIG_HOME"] {
        let token = format!("%{var}%");
        if let Some(rest) = p.strip_prefix(&token) {
            let base = crate::sys::env_path(var).or_else(|| match var {
                "XDG_DATA_HOME" => Some(home.join(".local/share")),
                "XDG_CONFIG_HOME" => Some(home.join(".config")),
                _ => None,
            })?;
            let rest = rest.trim_start_matches(['/', '\\']);
            return if rest.is_empty() { Some(base) } else { safe_relative(rest).map(|r| base.join(r)) };
        }
    }
    None
}

impl Manifest {
    /// Programm für dieses Betriebssystem (relativ zum Erweiterungsordner)
    pub fn run_for_os(&self) -> Option<String> {
        match &self.run {
            Run::All(p) => Some(p.clone()),
            Run::PerOs(m) => m.get(os_key()).cloned(),
        }
    }

    /// Gelesene Ordner mit eingesetzten Platzhaltern
    pub fn read_paths(&self) -> Vec<PathBuf> {
        self.permissions.read_paths.iter().filter_map(|p| expand_placeholder(p)).collect()
    }

    pub fn has(&self, k: ExtKind) -> bool {
        self.kinds.contains(&k)
    }
}

/// Manifest-Text prüfen (ohne Dateisystem)
pub fn parse(text: &str) -> Result<Manifest, ManifestError> {
    if text.len() > 64 * 1024 {
        return Err(ManifestError::TooBig);
    }
    let m: Manifest = serde_json::from_str(text).map_err(|e| ManifestError::Json(e.to_string().chars().take(200).collect()))?;
    if !valid_id(&m.id) {
        return Err(ManifestError::BadId);
    }
    if !valid_semver(&m.version) {
        return Err(ManifestError::BadVersion);
    }
    for (field, value, max) in [("name", &m.name, 60), ("author", &m.author, 60), ("description", &m.description, 500), ("homepage", &m.homepage, 200)] {
        let required = field == "name" || field == "author";
        if (required && value.trim().is_empty()) || value.chars().count() > max || value.chars().any(|c| c.is_control()) {
            return Err(ManifestError::BadText(field));
        }
    }
    if !m.homepage.is_empty() && !m.homepage.starts_with("https://") {
        return Err(ManifestError::BadText("homepage"));
    }
    if m.api_version != API_VERSION {
        return Err(ManifestError::Api(m.api_version));
    }
    if m.kinds.is_empty() || m.kinds.len() > 3 {
        return Err(ManifestError::NoKinds);
    }
    match &m.run {
        Run::All(p) => {
            safe_relative(p).ok_or_else(|| ManifestError::BadRun(p.clone()))?;
        }
        Run::PerOs(map) => {
            for (os, p) in map {
                if !matches!(os.as_str(), "windows" | "macos" | "linux") {
                    return Err(ManifestError::BadRun(os.clone()));
                }
                safe_relative(p).ok_or_else(|| ManifestError::BadRun(p.clone()))?;
            }
        }
    }
    for s in &m.permissions.uri_schemes {
        let ok = (2..=30).contains(&s.len())
            && s.chars().next().map(|c| c.is_ascii_lowercase()).unwrap_or(false)
            && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '+' | '-' | '.'))
            // gefährliche Schemata nie
            && !matches!(s.as_str(), "file" | "javascript" | "data" | "vbscript" | "ms-msdt" | "search-ms" | "ms-officecmd" | "shell");
        if !ok {
            return Err(ManifestError::BadScheme(s.clone()));
        }
    }
    if m.permissions.read_paths.len() > 20 {
        return Err(ManifestError::BadPath("…".into()));
    }
    for p in &m.permissions.read_paths {
        if expand_placeholder(p).is_none() {
            return Err(ManifestError::BadPath(p.clone()));
        }
    }
    if m.languages.len() > 20 || m.languages.iter().any(|l| l.len() != 2 || !l.chars().all(|c| c.is_ascii_lowercase())) {
        return Err(ManifestError::BadText("languages"));
    }
    Ok(m)
}

/// Erweiterungsordner laden: Manifest prüfen + "run" muss als Datei IM Ordner liegen
/// (keine Verknüpfung nach außen).
pub fn load_dir(dir: &Path) -> Result<(Manifest, PathBuf), ManifestError> {
    let mpath = dir.join("manifest.json");
    let meta = std::fs::symlink_metadata(&mpath).map_err(|_| ManifestError::Missing)?;
    if !meta.is_file() {
        return Err(ManifestError::Missing);
    }
    if meta.len() > 64 * 1024 {
        return Err(ManifestError::TooBig);
    }
    let text = std::fs::read_to_string(&mpath).map_err(|_| ManifestError::Missing)?;
    let m = parse(&text)?;
    let run_rel = m.run_for_os().ok_or(ManifestError::NoRun)?;
    let run = resolve_inside(dir, &run_rel).ok_or_else(|| ManifestError::BadRun(run_rel.clone()))?;
    Ok((m, run))
}

/// Pfad im Ordner auflösen; Symlinks, die nach außen zeigen, werden abgelehnt
pub fn resolve_inside(dir: &Path, rel: &str) -> Option<PathBuf> {
    let rel = safe_relative(rel)?;
    let base = std::fs::canonicalize(dir).ok()?;
    let full = std::fs::canonicalize(base.join(&rel)).ok()?;
    (full.starts_with(&base) && full.is_file()).then_some(full)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good() -> String {
        r#"{"id":"hello-world","name":"Hello","version":"1.0.0","author":"MAZ","run":{"windows":"hello.exe","linux":"hello","macos":"hello"},
            "api_version":1,"kinds":["library","action"],"permissions":{"read_paths":["~/Games"],"network":false,"uri_schemes":["hello"]}}"#
            .to_string()
    }

    #[test]
    fn valid_manifest() {
        let m = parse(&good()).unwrap();
        assert_eq!(m.id, "hello-world");
        assert!(m.has(ExtKind::Library) && !m.has(ExtKind::Metadata));
        assert!(m.run_for_os().is_some());
        assert_eq!(m.read_paths().len(), 1);
    }

    #[test]
    fn invalid_manifests() {
        let with = |from: &str, to: &str| parse(&good().replace(from, to));
        assert_eq!(with("hello-world", "Hello World").unwrap_err(), ManifestError::BadId);
        assert_eq!(with("\"1.0.0\"", "\"1.0\"").unwrap_err(), ManifestError::BadVersion);
        assert_eq!(with("\"api_version\":1", "\"api_version\":2").unwrap_err(), ManifestError::Api(2));
        assert!(matches!(with("\"author\":\"MAZ\"", "\"author\":\"MAZ\",\"evil\":1").unwrap_err(), ManifestError::Json(_)));
        assert!(matches!(with("\"hello.exe\"", "\"../hello.exe\"").unwrap_err(), ManifestError::BadRun(_)));
        assert!(matches!(with("\"hello.exe\"", "\"C:\\\\x.exe\"").unwrap_err(), ManifestError::BadRun(_)));
        assert!(matches!(with("\"linux\":\"hello\"", "\"linux\":\"/bin/sh\"").unwrap_err(), ManifestError::BadRun(_)));
        assert!(matches!(with("[\"hello\"]", "[\"javascript\"]").unwrap_err(), ManifestError::BadScheme(_)));
        assert!(matches!(with("~/Games", "/etc").unwrap_err(), ManifestError::BadPath(_)));
        assert!(matches!(with("~/Games", "~/../../etc").unwrap_err(), ManifestError::BadPath(_)));
        assert_eq!(with("[\"library\",\"action\"]", "[]").unwrap_err(), ManifestError::NoKinds);
        assert!(parse("kein json").is_err());
    }

    #[test]
    fn run_must_stay_inside() {
        let dir = std::env::temp_dir().join(format!("maz-ext-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("ext")).unwrap();
        std::fs::write(dir.join("outside"), b"x").unwrap();
        std::fs::write(dir.join("ext").join("prog"), b"x").unwrap();
        assert!(resolve_inside(&dir.join("ext"), "prog").is_some());
        assert!(resolve_inside(&dir.join("ext"), "../outside").is_none());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(dir.join("outside"), dir.join("ext").join("link")).unwrap();
            assert!(resolve_inside(&dir.join("ext"), "link").is_none());
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
