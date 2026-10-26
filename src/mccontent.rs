//! Minecraft-Inhalte (Mods, Texturenpakete, Shader, Datenpakete) von Modrinth und CurseForge
//! suchen und installieren – in den `.minecraft`-Ordner des offiziellen Launchers oder in ein
//! NoRisk-Profil.
//!
//! - Modrinth: öffentliche API, kein Schlüssel nötig.
//! - CurseForge: nur mit eigenem, kostenlosem API-Schlüssel des Nutzers. Mods, deren Autor den
//!   Download über andere Programme verbietet (`allowModDistribution = false`), werden nicht
//!   geladen – stattdessen öffnet sich die CurseForge-Seite.
//! - SICHERHEIT: Downloads nur von den offiziellen Servern (Liste unten), Prüfsumme wird immer
//!   kontrolliert, Dateinamen werden gesäubert, nur .jar/.zip, Größe begrenzt.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;

const MODRINTH: &str = "https://api.modrinth.com/v2";
const CURSEFORGE: &str = "https://api.curseforge.com/v1";
const MC_GAME_ID: u32 = 432;
const MAX_FILE: u64 = 300 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Mod,
    ResourcePack,
    Shader,
    DataPack,
}

impl Kind {
    pub const ALL: [Kind; 4] = [Kind::Mod, Kind::ResourcePack, Kind::Shader, Kind::DataPack];
    pub fn key(self) -> &'static str {
        match self {
            Kind::Mod => "mod",
            Kind::ResourcePack => "resourcepack",
            Kind::Shader => "shader",
            Kind::DataPack => "datapack",
        }
    }
    /// Unterordner im Spielordner (Datenpakete: in einer Welt)
    pub fn folder(self) -> &'static str {
        match self {
            Kind::Mod => "mods",
            Kind::ResourcePack => "resourcepacks",
            Kind::Shader => "shaderpacks",
            Kind::DataPack => "datapacks",
        }
    }
    fn curseforge_class(self) -> u32 {
        match self {
            Kind::Mod => 6,
            Kind::ResourcePack => 12,
            Kind::Shader => 6552,
            Kind::DataPack => 6945,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Provider {
    Modrinth,
    CurseForge,
}

/// Wohin installiert wird
#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    pub id: String,
    pub label: String,
    pub dir: PathBuf,
    /// Feste Version/Loader (NoRisk-Profil) – sonst frei wählbar
    pub game_version: Option<String>,
    pub loader: Option<String>,
    /// Offizieller Launcher → Fabric kann eingerichtet werden
    pub official: bool,
}

/// Alle Ziele: offizieller Launcher + NoRisk-Profile
pub fn targets() -> Vec<Target> {
    let mut out = Vec::new();
    if let Some(d) = crate::minecraft::official_game_dir() {
        out.push(Target { id: "official".into(), label: "Minecraft Launcher (.minecraft)".into(), dir: d, game_version: None, loader: None, official: true });
    }
    for p in crate::minecraft::norisk_profiles() {
        if p.dir.as_os_str().is_empty() {
            continue;
        }
        let gv = (!p.game_version.is_empty()).then(|| p.game_version.clone());
        let lo = (!p.loader.is_empty() && p.loader != "vanilla").then(|| p.loader.clone());
        out.push(Target {
            id: format!("norisk:{}", p.id),
            label: format!("NoRisk · {}", p.name),
            dir: p.dir,
            game_version: gv,
            loader: lo,
            official: false,
        });
    }
    out
}

/// Welten im Spielordner (für Datenpakete)
pub fn worlds(game_dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(game_dir.join("saves"))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().join("level.dat").is_file())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| safe_component(n))
        .collect();
    v.sort();
    v
}

/// Ein Suchergebnis
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Project {
    pub provider: Option<Provider>,
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub icon: String,
    pub downloads: u64,
    pub page: String,
    /// CurseForge: Autor erlaubt Download über andere Programme
    pub distribution_allowed: bool,
}

/// Eine konkrete Datei zum Installieren
#[derive(Clone, Debug, PartialEq, Default)]
pub struct FileRef {
    pub url: String,
    pub file_name: String,
    pub size: u64,
    pub sha1: String,
    pub sha512: String,
    /// Benötigte andere Projekte (gleicher Anbieter)
    pub deps: Vec<String>,
}

fn agent() -> ureq::Agent {
    use std::sync::OnceLock;
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT
        .get_or_init(|| {
            use ureq::tls::{RootCerts, TlsConfig, TlsProvider};
            let tls = TlsConfig::builder().provider(TlsProvider::NativeTls).root_certs(RootCerts::PlatformVerifier).build();
            ureq::Agent::config_builder()
                .tls_config(tls)
                .timeout_global(Some(Duration::from_secs(60)))
                .http_status_as_error(false)
                // Modrinth wünscht einen eindeutigen User-Agent mit Kontakt
                .user_agent(concat!("MikeAdamRGB/MAZ-Lauchner/", env!("CARGO_PKG_VERSION"), " (https://github.com/MikeAdamRGB/MAZ-Lauchner)"))
                .build()
                .into()
        })
        .clone()
}

fn get_json(url: &str, cf_key: Option<&str>) -> Result<serde_json::Value, String> {
    let mut req = agent().get(url).header("Accept", "application/json");
    if let Some(k) = cf_key {
        req = req.header("x-api-key", k);
    }
    let mut resp = req.call().map_err(|e| format!("{}: {e}", crate::i18n::t("common.err_network")))?;
    let status = resp.status().as_u16();
    let text = resp.body_mut().with_config().limit(8 * 1024 * 1024).read_to_string().map_err(|e| e.to_string())?;
    if status == 401 || status == 403 {
        return Err(crate::i18n::t("mc.err_key"));
    }
    if status == 429 {
        return Err(crate::i18n::t("mc.err_rate"));
    }
    if status >= 400 {
        return Err(format!("HTTP {status}"));
    }
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

fn s(v: &serde_json::Value, k: &str) -> String {
    v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string()
}

fn clip(t: &str, n: usize) -> String {
    t.chars().take(n).collect()
}

fn https_img(u: &str) -> String {
    if u.starts_with("https://") && u.len() < 600 { u.to_string() } else { String::new() }
}

/// Loader-Name → CurseForge-Nummer
fn cf_loader(l: &str) -> Option<u32> {
    match l {
        "forge" => Some(1),
        "fabric" => Some(4),
        "quilt" => Some(5),
        "neoforge" => Some(6),
        _ => None,
    }
}

/// Erlaubte Minecraft-Version ("1.21.4", "24w14a")
pub fn valid_version(v: &str) -> bool {
    !v.is_empty() && v.len() < 24 && v.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

pub fn valid_loader(l: &str) -> bool {
    matches!(l, "fabric" | "forge" | "neoforge" | "quilt")
}

// ----------------------------------------------------------------------
// Suche
// ----------------------------------------------------------------------

pub struct Query<'a> {
    pub text: &'a str,
    pub kind: Kind,
    pub game_version: &'a str,
    pub loader: &'a str,
    pub offset: usize,
}

/// Modrinth-Suchadresse
pub fn modrinth_search_url(q: &Query) -> String {
    let mut facets: Vec<String> = Vec::new();
    match q.kind {
        Kind::DataPack => facets.push("[\"categories:datapack\"]".into()),
        k => facets.push(format!("[\"project_type:{}\"]", k.key())),
    }
    if valid_version(q.game_version) {
        facets.push(format!("[\"versions:{}\"]", q.game_version));
    }
    if q.kind == Kind::Mod && valid_loader(q.loader) {
        facets.push(format!("[\"categories:{}\"]", q.loader));
    }
    let facets = format!("[{}]", facets.join(","));
    format!(
        "{MODRINTH}/search?query={}&limit=24&offset={}&index={}&facets={}",
        crate::sys::url_encode(q.text.trim()),
        q.offset,
        if q.text.trim().is_empty() { "downloads" } else { "relevance" },
        crate::sys::url_encode(&facets)
    )
}

pub fn parse_modrinth_search(v: &serde_json::Value) -> Vec<Project> {
    v.get("hits")
        .and_then(|h| h.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|h| {
                    let id = s(h, "project_id");
                    let slug = s(h, "slug");
                    if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric()) {
                        return None;
                    }
                    let kind = s(h, "project_type");
                    let safe_slug: String = slug.chars().filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_')).collect();
                    Some(Project {
                        provider: Some(Provider::Modrinth),
                        page: format!("https://modrinth.com/{}/{}", if kind.is_empty() { "mod".into() } else { clip(&kind, 20) }, if safe_slug.is_empty() { id.clone() } else { safe_slug.clone() }),
                        id,
                        slug: safe_slug,
                        title: clip(&s(h, "title"), 100),
                        description: clip(&s(h, "description"), 300),
                        author: clip(&s(h, "author"), 60),
                        icon: https_img(&s(h, "icon_url")),
                        downloads: h.get("downloads").and_then(|d| d.as_u64()).unwrap_or(0),
                        distribution_allowed: true,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn curseforge_search_url(q: &Query) -> String {
    let mut url = format!(
        "{CURSEFORGE}/mods/search?gameId={MC_GAME_ID}&classId={}&searchFilter={}&sortField={}&sortOrder=desc&pageSize=24&index={}",
        q.kind.curseforge_class(),
        crate::sys::url_encode(q.text.trim()),
        if q.text.trim().is_empty() { 6 } else { 2 },
        q.offset
    );
    if valid_version(q.game_version) {
        url.push_str(&format!("&gameVersion={}", q.game_version));
    }
    if q.kind == Kind::Mod {
        if let Some(l) = cf_loader(q.loader) {
            url.push_str(&format!("&modLoaderType={l}"));
        }
    }
    url
}

pub fn parse_curseforge_search(v: &serde_json::Value) -> Vec<Project> {
    v.get("data")
        .and_then(|h| h.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|h| {
                    let id = h.get("id")?.as_u64()?.to_string();
                    let page = h.get("links").map(|l| s(l, "websiteUrl")).unwrap_or_default();
                    Some(Project {
                        provider: Some(Provider::CurseForge),
                        slug: clip(&s(h, "slug"), 100),
                        title: clip(&s(h, "name"), 100),
                        description: clip(&s(h, "summary"), 300),
                        author: h
                            .get("authors")
                            .and_then(|a| a.as_array())
                            .and_then(|a| a.first())
                            .map(|a| clip(&s(a, "name"), 60))
                            .unwrap_or_default(),
                        icon: h.get("logo").map(|l| https_img(&s(l, "thumbnailUrl"))).unwrap_or_default(),
                        downloads: h.get("downloadCount").and_then(|d| d.as_f64()).unwrap_or(0.0) as u64,
                        page: if page.starts_with("https://www.curseforge.com/") { page } else { format!("https://www.curseforge.com/minecraft/search?search={id}") },
                        // null = unbekannt → wie erlaubt behandeln; false = verboten
                        distribution_allowed: h.get("allowModDistribution").and_then(|x| x.as_bool()).unwrap_or(true),
                        id,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn search(provider: Provider, q: &Query, cf_key: &str) -> Result<Vec<Project>, String> {
    match provider {
        Provider::Modrinth => Ok(parse_modrinth_search(&get_json(&modrinth_search_url(q), None)?)),
        Provider::CurseForge => {
            if cf_key.trim().is_empty() {
                return Err(crate::i18n::t("mc.err_no_key"));
            }
            Ok(parse_curseforge_search(&get_json(&curseforge_search_url(q), Some(cf_key.trim()))?))
        }
    }
}

/// Minecraft-Versionen (nur Vollversionen) von Modrinth
pub fn game_versions() -> Vec<String> {
    let Ok(v) = get_json(&format!("{MODRINTH}/tag/game_version"), None) else { return Vec::new() };
    v.as_array()
        .map(|a| {
            a.iter()
                .filter(|x| s(x, "version_type") == "release")
                .map(|x| s(x, "version"))
                .filter(|x| valid_version(x))
                .take(60)
                .collect()
        })
        .unwrap_or_default()
}

// ----------------------------------------------------------------------
// Passende Datei finden
// ----------------------------------------------------------------------

fn modrinth_loaders(kind: Kind, loader: &str) -> Vec<String> {
    match kind {
        Kind::Mod if valid_loader(loader) => vec![loader.to_string()],
        Kind::Mod => Vec::new(),
        Kind::ResourcePack => vec!["minecraft".into()],
        Kind::Shader => vec!["iris".into(), "optifine".into(), "canvas".into(), "vanilla".into()],
        Kind::DataPack => vec!["datapack".into()],
    }
}

pub fn parse_modrinth_versions(v: &serde_json::Value) -> Option<FileRef> {
    let ver = v.as_array()?.first()?;
    let files = ver.get("files")?.as_array()?;
    let f = files.iter().find(|f| f.get("primary").and_then(|p| p.as_bool()) == Some(true)).or_else(|| files.first())?;
    let hashes = f.get("hashes");
    Some(FileRef {
        url: s(f, "url"),
        file_name: s(f, "filename"),
        size: f.get("size").and_then(|x| x.as_u64()).unwrap_or(0),
        sha1: hashes.map(|h| s(h, "sha1")).unwrap_or_default(),
        sha512: hashes.map(|h| s(h, "sha512")).unwrap_or_default(),
        deps: ver
            .get("dependencies")
            .and_then(|d| d.as_array())
            .map(|d| {
                d.iter()
                    .filter(|x| s(x, "dependency_type") == "required")
                    .map(|x| s(x, "project_id"))
                    .filter(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_alphanumeric()))
                    .collect()
            })
            .unwrap_or_default(),
    })
}

pub fn parse_curseforge_files(v: &serde_json::Value) -> Option<FileRef> {
    let f = v.get("data")?.as_array()?.first()?;
    let sha1 = f
        .get("hashes")
        .and_then(|h| h.as_array())
        .and_then(|a| a.iter().find(|h| h.get("algo").and_then(|x| x.as_u64()) == Some(1)))
        .map(|h| s(h, "value"))
        .unwrap_or_default();
    Some(FileRef {
        url: s(f, "downloadUrl"),
        file_name: s(f, "fileName"),
        size: f.get("fileLength").and_then(|x| x.as_u64()).unwrap_or(0),
        sha1,
        sha512: String::new(),
        deps: f
            .get("dependencies")
            .and_then(|d| d.as_array())
            .map(|d| {
                d.iter()
                    .filter(|x| x.get("relationType").and_then(|r| r.as_u64()) == Some(3))
                    .filter_map(|x| x.get("modId").and_then(|m| m.as_u64()).map(|m| m.to_string()))
                    .collect()
            })
            .unwrap_or_default(),
    })
}

pub fn find_file(provider: Provider, project: &str, kind: Kind, game_version: &str, loader: &str, cf_key: &str) -> Result<FileRef, String> {
    let none = || crate::i18n::t("mc.err_no_file");
    match provider {
        Provider::Modrinth => {
            if !project.chars().all(|c| c.is_ascii_alphanumeric()) {
                return Err(none());
            }
            let mut url = format!("{MODRINTH}/project/{project}/version?");
            let loaders = modrinth_loaders(kind, loader);
            if !loaders.is_empty() {
                let list = format!("[{}]", loaders.iter().map(|l| format!("\"{l}\"")).collect::<Vec<_>>().join(","));
                url.push_str(&format!("loaders={}&", crate::sys::url_encode(&list)));
            }
            if valid_version(game_version) {
                url.push_str(&format!("game_versions={}", crate::sys::url_encode(&format!("[\"{game_version}\"]"))));
            }
            parse_modrinth_versions(&get_json(&url, None)?).ok_or_else(none)
        }
        Provider::CurseForge => {
            if !project.chars().all(|c| c.is_ascii_digit()) {
                return Err(none());
            }
            let mut url = format!("{CURSEFORGE}/mods/{project}/files?pageSize=10");
            if valid_version(game_version) {
                url.push_str(&format!("&gameVersion={game_version}"));
            }
            if kind == Kind::Mod {
                if let Some(l) = cf_loader(loader) {
                    url.push_str(&format!("&modLoaderType={l}"));
                }
            }
            parse_curseforge_files(&get_json(&url, Some(cf_key.trim()))?).ok_or_else(none)
        }
    }
}

// ----------------------------------------------------------------------
// Herunterladen und installieren
// ----------------------------------------------------------------------

/// SICHERHEIT: nur offizielle Download-Server
pub fn allowed_download(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("https://") else { return false };
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    if host.contains('@') || host.contains(':') {
        return false;
    }
    matches!(host, "cdn.modrinth.com" | "edge.forgecdn.net" | "mediafilez.forgecdn.net" | "media.forgecdn.net")
}

/// Einzelner Ordner-/Dateiname ohne Pfadteile
pub fn safe_component(n: &str) -> bool {
    !n.is_empty() && n.len() < 200 && n != "." && n != ".." && !n.contains(['/', '\\', ':', '\0']) && !n.starts_with('.')
}

/// Dateiname säubern; nur .jar und .zip
pub fn clean_file_name(name: &str) -> Option<String> {
    let base = name.rsplit(['/', '\\']).next().unwrap_or("");
    let clean: String = base.chars().filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+' | ' ' | '(' | ')' | '[' | ']')).collect();
    let clean = clean.trim().to_string();
    let lower = clean.to_lowercase();
    (safe_component(&clean) && (lower.ends_with(".jar") || lower.ends_with(".zip"))).then_some(clean)
}

/// Prüfsumme kontrollieren (SHA-512 bevorzugt, sonst SHA-1). Ohne Prüfsumme kein Install.
pub fn verify(data: &[u8], f: &FileRef) -> bool {
    use crate::util::{hex, sha1, sha512};
    if f.sha512.len() == 128 {
        return hex(&sha512(data)).eq_ignore_ascii_case(&f.sha512);
    }
    if f.sha1.len() == 40 {
        return hex(&sha1(data)).eq_ignore_ascii_case(&f.sha1);
    }
    false
}

fn download(f: &FileRef) -> Result<Vec<u8>, String> {
    if !allowed_download(&f.url) {
        return Err(crate::i18n::t("mc.err_source"));
    }
    if f.size > MAX_FILE {
        return Err(crate::i18n::t("mc.err_size"));
    }
    let mut resp = agent().get(&f.url).call().map_err(|e| format!("{}: {e}", crate::i18n::t("common.err_network")))?;
    if resp.status().as_u16() >= 400 {
        return Err(format!("HTTP {}", resp.status().as_u16()));
    }
    let data = resp.body_mut().with_config().limit(MAX_FILE).read_to_vec().map_err(|e| e.to_string())?;
    if !verify(&data, f) {
        return Err(crate::i18n::t("mc.err_hash"));
    }
    Ok(data)
}

/// Zielordner für eine Art (Datenpakete brauchen eine Welt)
pub fn dest_dir(game_dir: &Path, kind: Kind, world: &str) -> Option<PathBuf> {
    if kind == Kind::DataPack {
        if !safe_component(world) {
            return None;
        }
        return Some(game_dir.join("saves").join(world).join("datapacks"));
    }
    Some(game_dir.join(kind.folder()))
}

/// Datei sicher schreiben (erst .part, dann umbenennen)
fn write_into(dir: &Path, name: &str, data: &[u8]) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let dest = dir.join(name);
    let tmp = dir.join(format!("{name}.part"));
    std::fs::write(&tmp, data).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &dest).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        e.to_string()
    })?;
    Ok(dest)
}

/// Ergebnis einer Installation
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Installed {
    pub files: Vec<String>,
    pub skipped_deps: Vec<String>,
}

pub struct InstallRequest<'a> {
    pub provider: Provider,
    pub project: &'a str,
    pub kind: Kind,
    pub game_version: &'a str,
    pub loader: &'a str,
    pub game_dir: &'a Path,
    pub world: &'a str,
    pub cf_key: &'a str,
}

/// Projekt + benötigte Abhängigkeiten installieren (max. 20 Dateien)
pub fn install(r: &InstallRequest) -> Result<Installed, String> {
    let dir = dest_dir(r.game_dir, r.kind, r.world).ok_or_else(|| crate::i18n::t("mc.err_world"))?;
    let mut done = Installed::default();
    let mut queue: Vec<(String, Kind)> = vec![(r.project.to_string(), r.kind)];
    let mut seen: Vec<String> = Vec::new();
    while let Some((project, kind)) = queue.pop() {
        if seen.contains(&project) || seen.len() >= 20 {
            continue;
        }
        seen.push(project.clone());
        let is_main = project == r.project;
        let f = match find_file(r.provider, &project, kind, r.game_version, r.loader, r.cf_key) {
            Ok(f) => f,
            Err(e) if is_main => return Err(e),
            Err(_) => {
                done.skipped_deps.push(project);
                continue;
            }
        };
        // CurseForge: Autor hat den Download über andere Programme verboten → keine Adresse
        if f.url.is_empty() {
            if is_main {
                return Err(crate::i18n::t("mc.err_distribution"));
            }
            done.skipped_deps.push(project);
            continue;
        }
        let Some(name) = clean_file_name(&f.file_name) else {
            if is_main {
                return Err(crate::i18n::t("mc.err_no_file"));
            }
            continue;
        };
        let target = if kind == r.kind { dir.clone() } else { r.game_dir.join(kind.folder()) };
        // Schon da (gleiche Datei)? Dann nicht erneut laden
        if !target.join(&name).is_file() {
            let data = match download(&f) {
                Ok(d) => d,
                Err(e) if is_main => return Err(e),
                Err(_) => {
                    done.skipped_deps.push(project);
                    continue;
                }
            };
            write_into(&target, &name, &data)?;
        }
        done.files.push(name);
        // Abhängigkeiten sind immer Mods
        for d in f.deps {
            queue.push((d, Kind::Mod));
        }
    }
    Ok(done)
}

// ----------------------------------------------------------------------
// Fabric im offiziellen Minecraft Launcher einrichten (wie der offizielle Fabric-Installer)
// ----------------------------------------------------------------------

pub fn fabric_profile_name(game_version: &str) -> String {
    format!("fabric-loader-{game_version}")
}

/// Ist für diese Version schon ein Fabric-Profil eingerichtet?
pub fn fabric_installed(game_dir: &Path, game_version: &str) -> bool {
    std::fs::read_dir(game_dir.join("versions"))
        .into_iter()
        .flatten()
        .flatten()
        .any(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            n.starts_with("fabric-loader-") && n.ends_with(&format!("-{game_version}"))
        })
}

/// Profil im launcher_profiles.json ergänzen (alles andere bleibt unverändert)
pub fn add_launcher_profile(json: &str, version_id: &str, name: &str, now_iso: &str) -> Option<String> {
    let mut v: serde_json::Value = serde_json::from_str(json).ok()?;
    let profiles = v.as_object_mut()?.entry("profiles").or_insert_with(|| serde_json::json!({}));
    let map = profiles.as_object_mut()?;
    map.insert(
        name.to_string(),
        serde_json::json!({
            "name": name,
            "type": "custom",
            "created": now_iso,
            "lastUsed": now_iso,
            "icon": "Furnace",
            "lastVersionId": version_id,
        }),
    );
    serde_json::to_string_pretty(&v).ok()
}

pub fn install_fabric(game_dir: &Path, game_version: &str) -> Result<String, String> {
    if !valid_version(game_version) {
        return Err(crate::i18n::t("mc.err_version"));
    }
    let loaders = get_json(&format!("https://meta.fabricmc.net/v2/versions/loader/{game_version}"), None)?;
    let loader = loaders
        .as_array()
        .and_then(|a| a.iter().find(|l| l.get("loader").and_then(|x| x.get("stable")).and_then(|s| s.as_bool()) == Some(true)).or_else(|| a.first()))
        .and_then(|l| l.get("loader"))
        .map(|l| s(l, "version"))
        .filter(|v| valid_version(v))
        .ok_or_else(|| crate::i18n::t("mc.err_version"))?;
    let profile = get_json(&format!("https://meta.fabricmc.net/v2/versions/loader/{game_version}/{loader}/profile/json"), None)?;
    let id = s(&profile, "id");
    if !safe_component(&id) || !id.starts_with("fabric-loader-") {
        return Err(crate::i18n::t("mc.err_version"));
    }
    let vdir = game_dir.join("versions").join(&id);
    let text = serde_json::to_string_pretty(&profile).map_err(|e| e.to_string())?;
    write_into(&vdir, &format!("{id}.json"), text.as_bytes())?;
    // Leere .jar wie beim offiziellen Installer
    if !vdir.join(format!("{id}.jar")).is_file() {
        write_into(&vdir, &format!("{id}.jar"), &[])?;
    }
    let now = iso_now();
    let name = fabric_profile_name(game_version);
    for file in ["launcher_profiles.json", "launcher_profiles_microsoft_store.json"] {
        let p = game_dir.join(file);
        let Ok(json) = std::fs::read_to_string(&p) else { continue };
        let Some(new) = add_launcher_profile(&json, &id, &name, &now) else { continue };
        // Sicherung, dann ersetzen
        let _ = std::fs::copy(&p, game_dir.join(format!("{file}.maz-backup")));
        write_into(game_dir, file, new.as_bytes())?;
    }
    Ok(id)
}

fn iso_now() -> String {
    let (y, mo, d, h, mi) = crate::util::civil(crate::sys::now_secs() as i64);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:00.000Z")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_and_parsing() {
        let q = Query { text: "sodium", kind: Kind::Mod, game_version: "1.21.4", loader: "fabric", offset: 0 };
        let u = modrinth_search_url(&q);
        assert!(u.starts_with("https://api.modrinth.com/v2/search?query=sodium"));
        assert!(u.contains("versions%3A1.21.4"));
        let bad = Query { text: "x", kind: Kind::Mod, game_version: "1.21\"]],[\"x", loader: "evil", offset: 0 };
        let u2 = modrinth_search_url(&bad);
        assert!(!u2.contains("evil") && !u2.contains("versions"));
        let cf = curseforge_search_url(&q);
        assert!(cf.contains("classId=6") && cf.contains("modLoaderType=4"));

        let hits = serde_json::json!({"hits":[{"project_id":"AANobbMI","slug":"sodium","title":"Sodium","project_type":"mod","downloads":5,"icon_url":"https://cdn.modrinth.com/i.png"},{"project_id":"../x","title":"Böse"}]});
        let p = parse_modrinth_search(&hits);
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].page, "https://modrinth.com/mod/sodium");

        let vers = serde_json::json!([{"files":[{"url":"https://cdn.modrinth.com/data/a/b.jar","filename":"sodium.jar","primary":true,"size":10,"hashes":{"sha1":"a","sha512":"b"}}],"dependencies":[{"project_id":"P7dR8mSH","dependency_type":"required"},{"project_id":"x","dependency_type":"optional"}]}]);
        let f = parse_modrinth_versions(&vers).unwrap();
        assert_eq!(f.file_name, "sodium.jar");
        assert_eq!(f.deps, vec!["P7dR8mSH".to_string()]);

        let cfj = serde_json::json!({"data":[{"id":1,"name":"JEI","allowModDistribution":false,"links":{"websiteUrl":"https://www.curseforge.com/minecraft/mc-mods/jei"}}]});
        let c = parse_curseforge_search(&cfj);
        assert!(!c[0].distribution_allowed);
    }

    #[test]
    fn safety() {
        assert!(allowed_download("https://cdn.modrinth.com/data/x/y.jar"));
        assert!(allowed_download("https://edge.forgecdn.net/files/1/2/a.jar"));
        assert!(!allowed_download("https://evil.com/cdn.modrinth.com/a.jar"));
        assert!(!allowed_download("https://cdn.modrinth.com.evil.com/a.jar"));
        assert!(!allowed_download("http://cdn.modrinth.com/a.jar"));
        assert!(!allowed_download("https://user@cdn.modrinth.com/a.jar"));
        assert_eq!(clean_file_name("../../evil.jar").as_deref(), Some("evil.jar"));
        assert_eq!(clean_file_name("C:\\x\\mod.zip").as_deref(), Some("mod.zip"));
        assert_eq!(clean_file_name("run.exe"), None);
        assert_eq!(clean_file_name(".jar"), None);
        assert!(dest_dir(Path::new("/g"), Kind::DataPack, "../x").is_none());
        assert_eq!(dest_dir(Path::new("/g"), Kind::DataPack, "Welt").unwrap(), PathBuf::from("/g/saves/Welt/datapacks"));
        let f = FileRef { sha1: crate::util::hex(&crate::util::sha1(b"abc")), ..Default::default() };
        assert!(verify(b"abc", &f));
        assert!(!verify(b"abd", &f));
        assert!(!verify(b"abc", &FileRef::default()));
    }

    #[test]
    fn launcher_profiles() {
        let j = r#"{"profiles":{"a":{"name":"Latest"}},"settings":{"x":1}}"#;
        let out = add_launcher_profile(j, "fabric-loader-0.16.9-1.21.4", "fabric-loader-1.21.4", "2026-01-01T00:00:00.000Z").unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["profiles"]["a"]["name"], "Latest");
        assert_eq!(v["profiles"]["fabric-loader-1.21.4"]["lastVersionId"], "fabric-loader-0.16.9-1.21.4");
        assert_eq!(v["settings"]["x"], 1);
        assert!(add_launcher_profile("kaputt", "a", "b", "c").is_none());
    }
}
