//! Systemfunktionen: Ordner, Registry, URLs öffnen, Zwischenablage …
//! Läuft auf Windows, macOS und Linux. Windows-Spezialitäten (Registry …) liefern
//! auf den anderen Systemen einfach "nichts gefunden".

use std::path::{Path, PathBuf};
use std::process::Command;

pub const APP_NAME: &str = "Open Game Launcher";
/// Kurzname / Logo
pub const SHORT_NAME: &str = "OG";
/// Früherer Name (bis 1.7) – für Umzug der Ordner, Verknüpfungen und alte Installationen
pub const OLD_APP_NAME: &str = "MAZ Launcher";
#[cfg(windows)]
pub const EXE_NAME: &str = "Open-Game-Launcher.exe";
/// Alte EXE in bestehenden Installationen (wird vom Updater an Ort und Stelle ersetzt)
#[cfg(windows)]
pub const OLD_EXE_NAME: &str = "MAZ-Launcher.exe";
// Linux/macOS: Programmname bleibt intern gleich (Paketnamen, Updates)
#[cfg(not(windows))]
pub const EXE_NAME: &str = "maz-launcher";
#[cfg(not(windows))]
pub const OLD_EXE_NAME: &str = "maz-launcher";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GITHUB_URL: &str = "https://github.com/MikeAdamRGB/Open-Game-Launcher";

pub fn env_path(var: &str) -> Option<PathBuf> {
    std::env::var_os(var)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

fn home() -> PathBuf {
    env_path("HOME").or_else(|| env_path("USERPROFILE")).unwrap_or_else(|| PathBuf::from("."))
}

/// Steam Deck (oder anderes Gerät) im Gaming-Modus von Steam (gamescope-Sitzung)?
pub fn gaming_mode() -> bool {
    if !cfg!(target_os = "linux") {
        return false;
    }
    let var = |k: &str| std::env::var(k).unwrap_or_default();
    var("SteamGamepadUI") == "1" || var("XDG_CURRENT_DESKTOP").eq_ignore_ascii_case("gamescope") || !var("GAMESCOPE_WAYLAND_DISPLAY").is_empty()
}

/// Ordnernamen (neu, alt) je System
#[cfg(any(windows, target_os = "macos"))]
const DIR_NAMES: (&str, &str) = (APP_NAME, OLD_APP_NAME);
#[cfg(not(any(windows, target_os = "macos")))]
const DIR_NAMES: (&str, &str) = ("open-game-launcher", "maz-launcher");

/// Ordner unter `base` mit neuem Namen. Gibt es nur den alten Ordner (bis 1.7 „MAZ Launcher“),
/// wird er einmalig umbenannt – Einstellungen, Spiele und Erweiterungen ziehen so mit.
/// Klappt das Umbenennen nicht (z. B. Datei gerade geöffnet), wird einfach der alte Ordner benutzt.
pub fn migrated_dir(base: &Path, new: &str, old: &str) -> PathBuf {
    let n = base.join(new);
    if n.exists() {
        return n;
    }
    let o = base.join(old);
    let is_real_dir = std::fs::symlink_metadata(&o).map(|m| m.is_dir()).unwrap_or(false);
    if is_real_dir && std::fs::rename(&o, &n).is_err() {
        return o;
    }
    n
}

fn roaming_base() -> PathBuf {
    #[cfg(windows)]
    {
        env_path("APPDATA").unwrap_or_else(home)
    }
    #[cfg(target_os = "macos")]
    {
        home().join("Library").join("Application Support")
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        env_path("XDG_CONFIG_HOME").unwrap_or_else(|| home().join(".config"))
    }
}

fn local_base() -> PathBuf {
    #[cfg(windows)]
    {
        env_path("LOCALAPPDATA").unwrap_or_else(home)
    }
    #[cfg(target_os = "macos")]
    {
        home().join("Library").join("Caches")
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        env_path("XDG_CACHE_HOME").unwrap_or_else(|| home().join(".cache"))
    }
}

/// Einstellungen & eigene Spiele
/// - Windows: %APPDATA%\Open Game Launcher
/// - macOS: ~/Library/Application Support/Open Game Launcher
/// - Linux: ~/.config/open-game-launcher (bzw. $XDG_CONFIG_HOME)
pub fn roaming_dir() -> PathBuf {
    static DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    DIR.get_or_init(|| migrated_dir(&roaming_base(), DIR_NAMES.0, DIR_NAMES.1)).clone()
}

/// Cache (Icons, WebView-Daten)
/// - Windows: %LOCALAPPDATA%\Open Game Launcher
/// - macOS: ~/Library/Caches/Open Game Launcher
/// - Linux: ~/.cache/open-game-launcher (bzw. $XDG_CACHE_HOME)
pub fn local_dir() -> PathBuf {
    static DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    DIR.get_or_init(|| migrated_dir(&local_base(), DIR_NAMES.0, DIR_NAMES.1)).clone()
}

/// Pfad aus einer Konfigurationsdatei in die Schreibweise des Systems bringen
/// (Windows: "\\", sonst unverändert).
pub fn native_path(s: &str) -> String {
    if cfg!(windows) {
        s.replace('/', "\\")
    } else {
        s.to_string()
    }
}

/// Heimatordner des Benutzers
pub fn home_dir() -> PathBuf {
    home()
}

/// Programm zum Öffnen von Links/Ordnern auf macOS bzw. Linux
#[cfg(not(windows))]
fn opener() -> Command {
    #[cfg(target_os = "macos")]
    {
        Command::new("/usr/bin/open")
    }
    #[cfg(not(target_os = "macos"))]
    {
        Command::new(unix_tool("xdg-open"))
    }
}

pub fn icon_cache_dir() -> PathBuf {
    local_dir().join("icons")
}

/// Standard-Installationsordner (ohne Admin-Rechte beschreibbar)
pub fn default_install_dir() -> PathBuf {
    env_path("LOCALAPPDATA")
        .map(|p| p.join("Programs").join(APP_NAME))
        .unwrap_or_else(|| PathBuf::from("C:\\").join(APP_NAME))
}

pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// macOS/Linux: Systemprogramm mit festem Pfad (/usr/bin, /bin) statt Suche im PATH –
/// so kann kein gleichnamiges Programm aus einem fremden Ordner untergeschoben werden.
pub fn unix_tool(name: &str) -> PathBuf {
    for dir in ["/usr/bin", "/bin", "/usr/local/bin", "/run/current-system/sw/bin"] {
        let p = Path::new(dir).join(name);
        if p.is_file() {
            return p;
        }
    }
    PathBuf::from(name)
}

/// Datei schreiben, die nur der eigene Benutzer lesen darf (macOS/Linux: 0600, Ordner 0700).
/// Unter Windows liegt sie im Benutzerprofil (nur für den Benutzer lesbar), Schlüssel sind dort
/// zusätzlich mit DPAPI verschlüsselt.
pub fn write_private(path: &Path, data: &[u8]) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
            let _ = std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700));
        }
        let _ = std::fs::remove_file(path);
        let mut f = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(path)?;
        f.write_all(data)?;
        f.sync_all()
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, data)
    }
}

/// Dateiname ohne verbotene Zeichen (für "Speichern unter")
pub fn safe_file_name(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '-' || c == '_' { c } else { '_' })
        .take(60)
        .collect();
    let s = s.trim().to_string();
    if s.is_empty() { "datei".into() } else { s }
}

/// Zeigen zwei Pfade auf denselben Ordner? (Groß/klein und \\ vs. / egal)
pub fn same_path(a: &str, b: &str) -> bool {
    let n = |p: &str| p.trim().trim_matches('"').replace('\\', "/").trim_end_matches('/').to_lowercase();
    let (a, b) = (n(a), n(b));
    !a.is_empty() && a == b
}

/// Prüft, ob ein Laufwerk wie "D:" existiert.
pub fn drive_exists(path: &str) -> bool {
    match crate::problem::drive_of(path) {
        Some(d) => Path::new(&format!("{d}\\")).exists(),
        None => true,
    }
}

/// Absoluter Pfad zu einem Windows-Programm. SICHERHEIT: Nie nur "cmd.exe" o. Ä. starten –
/// Windows sucht sonst zuerst im Ordner der EXE (z. B. "Downloads"), wo eine untergeschobene
/// Datei gleichen Namens liegen könnte.
pub fn system_program(name: &str) -> PathBuf {
    #[cfg(windows)]
    {
        let sys32 = crate::winffi::system_dir().unwrap_or_else(|| PathBuf::from("C:\\Windows\\System32"));
        let win = crate::winffi::windows_dir().unwrap_or_else(|| PathBuf::from("C:\\Windows"));
        match name {
            "explorer.exe" => win.join(name),
            "powershell.exe" => sys32.join("WindowsPowerShell").join("v1.0").join(name),
            _ => sys32.join(name),
        }
    }
    #[cfg(not(windows))]
    {
        PathBuf::from(name)
    }
}

/// Programm auf dem Computer starten. Läuft der Launcher als Flatpak, geht das nur über
/// "flatpak-spawn --host" (sonst würde er nur in seiner Sandbox suchen).
pub fn host_command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    if std::env::var_os("FLATPAK_ID").is_some() {
        let mut c = Command::new("flatpak-spawn");
        c.arg("--host").arg(program);
        c
    } else {
        Command::new(program)
    }
}

/// Wie `host_command`, aber mit Umgebungsvariablen und Arbeitsordner (auch im Flatpak)
pub fn host_command_env(program: impl AsRef<std::ffi::OsStr>, envs: &[(String, String)], dir: Option<&Path>) -> Command {
    if std::env::var_os("FLATPAK_ID").is_some() {
        let mut c = Command::new("flatpak-spawn");
        c.arg("--host");
        for (k, v) in envs {
            c.arg(format!("--env={k}={v}"));
        }
        if let Some(d) = dir {
            c.arg(format!("--directory={}", d.display()));
        }
        c.arg(program);
        c
    } else {
        let mut c = Command::new(program);
        c.envs(envs.iter().map(|(k, v)| (k.as_str(), v.as_str())));
        if let Some(d) = dir {
            c.current_dir(d);
        }
        c
    }
}

/// Kommando ohne aufblitzendes Konsolenfenster.
pub fn quiet_command(program: impl AsRef<std::ffi::OsStr>) -> Command {
    #[allow(unused_mut)]
    let mut c = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        c.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    c
}

/// Ordner im Explorer öffnen.
pub fn open_folder(path: &str) {
    // SICHERHEIT: Nur echte Ordner öffnen – "open" auf eine Datei würde sie ausführen.
    if !Path::new(path).is_dir() {
        return;
    }
    #[cfg(windows)]
    {
        let _ = crate::winffi::shell_execute(Some("open"), path, None, None)
            .or_else(|_| Command::new(system_program("explorer.exe")).arg(path).spawn().map(|_| ()).map_err(|_| 0));
    }
    #[cfg(not(windows))]
    {
        let _ = opener().arg(path).spawn();
    }
}

/// Datei im Explorer markieren.
pub fn reveal_file(path: &str) {
    #[cfg(windows)]
    {
        let _ = Command::new(system_program("explorer.exe"))
            .arg(format!("/select,{path}"))
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("/usr/bin/open").arg("-R").arg(path).spawn();
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        if let Some(p) = Path::new(path).parent() {
            open_folder(&p.to_string_lossy());
        }
    }
}

/// Nur diese Arten von Links dürfen geöffnet werden. Verhindert, dass ein Link aus dem
/// Internet (z. B. von einer Gratis-Spiele-Seite) eine Datei, ein Programm oder einen
/// Netzwerkpfad (\\server\…) startet.
pub fn is_safe_url(url: &str) -> bool {
    let u = url.trim().to_ascii_lowercase();
    if u.contains('\n') || u.contains('\r') || u.contains('"') {
        return false;
    }
    const ALLOWED: [&str; 16] = [
        "https://",
        "http://",
        "steam://",
        "uplay://",
        "origin2://",
        "amazon-games://",
        "goggalaxy://",
        "battlenet://",
        "roblox://",
        "roblox-player:",
        "heroic://",
        "com.epicgames.launcher://",
        "ms-settings:",
        "windowsdefender://",
        "mailto:",
        "ms-windows-store:",
    ];
    if !ALLOWED.iter().any(|p| u.starts_with(p)) {
        return false;
    }
    // SICHERHEIT: Steam-Links mit angehängten Startparametern („steam://run/123//-befehl“)
    // werden abgelehnt – darüber ließen sich in manchen Spielen gefährliche Optionen setzen.
    if let Some(rest) = u.strip_prefix("steam://") {
        // Ausnahme: „steam://openurl/https://…“ öffnet nur eine Seite im Steam-Browser
        let rest = rest.strip_prefix("openurl/https://").unwrap_or(rest);
        if rest.contains("//") || rest.contains("%2f%2f") || rest.contains(' ') || rest.contains("%20") {
            return false;
        }
    }
    true
}

/// URL oder Protokoll-Link öffnen (https://, steam://, …).
pub fn open_url(url: &str) -> Result<(), i32> {
    // Zusätzlich: Link-Arten, die der Nutzer für eine aktive Erweiterung bestätigt hat
    let basic_ok = !url.contains(['\n', '\r', '"']) && url.len() < 4000;
    if !is_safe_url(url) && !(basic_ok && crate::extensions::scheme_approved(url)) {
        return Err(5);
    }
    #[cfg(windows)]
    {
        crate::winffi::shell_execute(Some("open"), url, None, None)
    }
    #[cfg(not(windows))]
    {
        opener().arg(url).spawn().map(|_| ()).map_err(|_| 0)
    }
}

/// Text aus der Zwischenablage lesen
pub fn read_clipboard() -> Option<String> {
    #[cfg(windows)]
    {
        crate::winffi::get_clipboard()
    }
    #[cfg(not(windows))]
    {
        let tools: &[(&str, &[&str])] = if cfg!(target_os = "macos") {
            &[("/usr/bin/pbpaste", &[])]
        } else {
            &[("wl-paste", &["--no-newline"]), ("xclip", &["-selection", "clipboard", "-o"]), ("xsel", &["--clipboard", "--output"])]
        };
        for (cmd, args) in tools {
            if let Ok(out) = Command::new(unix_tool(cmd)).args(*args).output() {
                if out.status.success() {
                    return Some(String::from_utf8_lossy(&out.stdout).chars().take(65_536).collect());
                }
            }
        }
        None
    }
}

pub fn copy_to_clipboard(text: &str) -> bool {
    #[cfg(windows)]
    {
        crate::winffi::set_clipboard(text)
    }
    #[cfg(not(windows))]
    {
        use std::io::Write;
        use std::process::Stdio;
        let tools: &[(&str, &[&str])] = if cfg!(target_os = "macos") {
            &[("/usr/bin/pbcopy", &[])]
        } else {
            &[("wl-copy", &[]), ("xclip", &["-selection", "clipboard"]), ("xsel", &["--clipboard", "--input"])]
        };
        for (cmd, args) in tools {
            if let Ok(mut child) = Command::new(unix_tool(cmd)).args(*args).stdin(Stdio::piped()).spawn() {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(text.as_bytes());
                }
                if child.wait().map(|s| s.success()).unwrap_or(false) {
                    return true;
                }
            }
        }
        false
    }
}

// ----------------------------------------------------------------------
// Registry
// ----------------------------------------------------------------------

#[derive(Clone, Copy)]
pub enum Hive {
    CurrentUser,
    LocalMachine,
    ClassesRoot,
}

#[cfg(windows)]
fn hive_key(h: Hive) -> winreg::RegKey {
    use winreg::enums::*;
    match h {
        Hive::CurrentUser => winreg::RegKey::predef(HKEY_CURRENT_USER),
        Hive::LocalMachine => winreg::RegKey::predef(HKEY_LOCAL_MACHINE),
        Hive::ClassesRoot => winreg::RegKey::predef(HKEY_CLASSES_ROOT),
    }
}

pub fn reg_str(hive: Hive, path: &str, name: &str) -> Option<String> {
    #[cfg(windows)]
    {
        let key = hive_key(hive).open_subkey(path).ok()?;
        let v: String = key.get_value(name).ok()?;
        let v = v.trim().to_string();
        if v.is_empty() {
            None
        } else {
            Some(v)
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (hive, path, name);
        None
    }
}

pub fn reg_u32(hive: Hive, path: &str, name: &str) -> Option<u32> {
    #[cfg(windows)]
    {
        let key = hive_key(hive).open_subkey(path).ok()?;
        key.get_value::<u32, _>(name).ok()
    }
    #[cfg(not(windows))]
    {
        let _ = (hive, path, name);
        None
    }
}

pub fn reg_key_exists(hive: Hive, path: &str) -> bool {
    #[cfg(windows)]
    {
        hive_key(hive).open_subkey(path).is_ok()
    }
    #[cfg(not(windows))]
    {
        let _ = (hive, path);
        false
    }
}

/// Ist ein Link-Protokoll wie "steam" oder "com.epicgames.launcher" registriert?
pub fn protocol_registered(name: &str) -> bool {
    #[cfg(windows)]
    {
        reg_key_exists(Hive::ClassesRoot, &format!("{name}\\shell\\open\\command"))
            || reg_key_exists(Hive::CurrentUser, &format!("Software\\Classes\\{name}\\shell\\open\\command"))
    }
    #[cfg(target_os = "macos")]
    {
        // macOS fragt beim Öffnen selbst nach – die passenden Apps prüfen die Stores selbst
        let _ = name;
        true
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        // Linux: Gibt es ein Programm für x-scheme-handler/<name>?
        Command::new(unix_tool("xdg-mime"))
            .args(["query", "default", &format!("x-scheme-handler/{name}")])
            .output()
            .map(|o| o.status.success() && !String::from_utf8_lossy(&o.stdout).trim().is_empty())
            .unwrap_or(true)
    }
}

/// Ist die Microsoft-WebView2-Laufzeit installiert? (Wird für das Fenster gebraucht.)
pub fn webview2_installed() -> bool {
    #[cfg(windows)]
    {
        const GUID: &str = "{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";
        let paths = [
            (Hive::LocalMachine, format!("SOFTWARE\\WOW6432Node\\Microsoft\\EdgeUpdate\\Clients\\{GUID}")),
            (Hive::LocalMachine, format!("SOFTWARE\\Microsoft\\EdgeUpdate\\Clients\\{GUID}")),
            (Hive::CurrentUser, format!("Software\\Microsoft\\EdgeUpdate\\Clients\\{GUID}")),
        ];
        for (hive, p) in paths.iter() {
            if let Some(v) = reg_str(*hive, p, "pv") {
                if v != "0.0.0.0" {
                    return true;
                }
            }
        }
        // Fallback: fest installierte Runtime-Ordner
        for base in ["ProgramFiles(x86)", "ProgramFiles"] {
            if let Some(b) = env_path(base) {
                if b.join("Microsoft").join("EdgeWebView").join("Application").exists() {
                    return true;
                }
            }
        }
        false
    }
    #[cfg(not(windows))]
    {
        true
    }
}

/// %VAR% in Registry-Pfaden auflösen.
pub fn expand_env(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(start) = rest.find('%') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        if let Some(end) = after.find('%') {
            let var = &after[..end];
            match std::env::var(var) {
                Ok(v) => out.push_str(&v),
                Err(_) => {
                    out.push('%');
                    out.push_str(var);
                    out.push('%');
                }
            }
            rest = &after[end + 1..];
        } else {
            out.push_str(&rest[start..]);
            rest = "";
        }
    }
    out.push_str(rest);
    out
}

/// Der echte Desktop-Ordner (funktioniert auch, wenn er auf OneDrive umgeleitet ist).
pub fn desktop_dir() -> PathBuf {
    let key = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\User Shell Folders";
    if let Some(p) = reg_str(Hive::CurrentUser, key, "Desktop") {
        let p = PathBuf::from(expand_env(&p));
        if p.exists() {
            return p;
        }
    }
    let key = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Shell Folders";
    if let Some(p) = reg_str(Hive::CurrentUser, key, "Desktop") {
        let p = PathBuf::from(p);
        if p.exists() {
            return p;
        }
    }
    env_path("USERPROFILE")
        .or_else(|| env_path("HOME"))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Desktop")
}

pub fn start_menu_dir() -> PathBuf {
    env_path("APPDATA")
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Microsoft")
        .join("Windows")
        .join("Start Menu")
        .join("Programs")
}

// ----------------------------------------------------------------------
// Dateien für die Oberfläche bereitstellen (/maz/f/<hex>)
// ----------------------------------------------------------------------

/// Macht aus einem lokalen Dateipfad eine URL, die das Fenster laden darf.
pub fn file_url(path: &Path) -> String {
    let s = path.to_string_lossy();
    let mut hex = String::with_capacity(s.len() * 2);
    for b in s.as_bytes() {
        hex.push_str(&format!("{b:02x}"));
    }
    format!("/maz/f/{hex}")
}

pub fn decode_file_url(url_path: &str) -> Option<PathBuf> {
    let hex = url_path.trim_start_matches('/').strip_prefix("maz/f/")?;
    let hex = hex.split(['?', '#']).next()?;
    if hex.len() % 2 != 0 {
        return None;
    }
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        bytes.push(u8::from_str_radix(hex.get(i..i + 2)?, 16).ok()?);
    }
    let text = String::from_utf8(bytes).ok()?;
    // SICHERHEIT: Keine Netzwerk- oder Gerätepfade (\\\\server\\… würde den Windows-Login-Hash senden)
    let b = text.as_bytes();
    let drive_path = b.len() > 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/');
    if !drive_path && !text.starts_with('/') {
        return None;
    }
    if text.starts_with("//") || text.contains("..") {
        return None;
    }
    let path = PathBuf::from(text);
    // SICHERHEIT: Nur Bilder ausliefern – nie Einstellungen, Tokens oder andere Dateien.
    let ext = path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()).unwrap_or_default();
    matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" | "ico" | "avif").then_some(path)
}

#[cfg(test)]
mod safety_tests {
    use super::*;
    #[test]
    fn only_safe_links_and_images() {
        assert!(is_safe_url("https://store.epicgames.com/p/x"));
        assert!(is_safe_url("steam://rungameid/1"));
        assert!(is_safe_url("steam://openurl/https://store.steampowered.com/app/1/"));
        assert!(!is_safe_url("steam://run/440//+exec evil.cfg"));
        assert!(!is_safe_url("steam://run/440/%2F%2F-x"));
        assert!(!is_safe_url("steam://openurl/https://x//y z"));
        assert!(!is_safe_url("C:\\Windows\\System32\\cmd.exe"));
        assert!(!is_safe_url("\\\\evil\\share\\x.exe"));
        assert!(!is_safe_url("file:///C:/x.exe"));
        assert!(!is_safe_url("javascript:alert(1)"));
        assert!(decode_file_url(&file_url(Path::new("C:\\a\\cover.png"))).is_some());
        assert!(decode_file_url(&file_url(Path::new("C:\\a\\config.json"))).is_none());
        assert!(decode_file_url(&file_url(Path::new("\\\\evil\\share\\x.png"))).is_none());
        assert!(decode_file_url(&file_url(Path::new("C:\\a\\..\\b.png"))).is_none());
    }
}

pub fn mime_for(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        Some("bmp") => "image/bmp",
        Some("ico") => "image/x-icon",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

/// Einfacher, stabiler Hash (FNV-1a) für Cache-Dateinamen.
pub fn fnv1a(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// Prozentkodierung für URL-Parameter.
pub fn url_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.as_bytes() {
        match *b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(*b as char),
            b' ' => out.push_str("%20"),
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

fn decimal(v: f64) -> String {
    format!("{v:.1}").replace('.', &crate::i18n::t("fmt.decimal"))
}

pub fn format_bytes(b: u64) -> String {
    let gb = b as f64 / 1_073_741_824.0;
    if gb >= 1.0 {
        format!("{} GB", decimal(gb))
    } else {
        format!("{:.0} MB", b as f64 / 1_048_576.0)
    }
}

pub fn format_playtime(min: u64) -> String {
    use crate::i18n::tf;
    if min < 60 {
        tf("fmt.minutes", &[("n", &min.to_string())])
    } else {
        let h = min as f64 / 60.0;
        let n = if h < 10.0 { decimal(h) } else { format!("{h:.0}") };
        tf("fmt.hours", &[("n", &n)])
    }
}

pub fn format_ago(ts: u64) -> String {
    use crate::i18n::{t, tf};
    if ts == 0 {
        return t("fmt.never");
    }
    let diff = now_secs().saturating_sub(ts);
    match diff {
        0..=119 => t("fmt.just_now"),
        120..=3599 => tf("fmt.min_ago", &[("n", &(diff / 60).to_string())]),
        3600..=86_399 => tf("fmt.hours_ago", &[("n", &(diff / 3600).to_string())]),
        86_400..=172_799 => t("fmt.yesterday"),
        _ => tf("fmt.days_ago", &[("n", &(diff / 86_400).to_string())]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_folder_moves_to_new_name() {
        let base = std::env::temp_dir().join(format!("og-migrate-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(base.join("Alt")).unwrap();
        std::fs::write(base.join("Alt").join("config.json"), "{}").unwrap();
        let d = migrated_dir(&base, "Neu", "Alt");
        assert_eq!(d, base.join("Neu"));
        assert!(d.join("config.json").exists());
        assert!(!base.join("Alt").exists());
        // Gibt es beide, bleibt der neue – der alte wird nicht angefasst
        std::fs::create_dir_all(base.join("Alt")).unwrap();
        assert_eq!(migrated_dir(&base, "Neu", "Alt"), base.join("Neu"));
        assert!(base.join("Alt").exists());
        // Weder noch: neuer Name
        assert_eq!(migrated_dir(&base, "X", "Y"), base.join("X"));
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn file_url_roundtrip() {
        let p = PathBuf::from("C:\\Spiele\\Über Spiel\\cover.jpg");
        let u = file_url(&p);
        assert_eq!(decode_file_url(&u), Some(p));
        assert_eq!(decode_file_url("/maz/f/zz"), None);
    }

    #[test]
    fn env_expansion() {
        std::env::set_var("MAZ_TEST_VAR", "X");
        assert_eq!(expand_env("%MAZ_TEST_VAR%\\Desktop"), "X\\Desktop");
        assert_eq!(expand_env("%NOPE_NOT_SET%"), "%NOPE_NOT_SET%");
        assert_eq!(expand_env("50%"), "50%");
    }

    #[test]
    fn formatting() {
        crate::i18n::set_language("de");
        assert_eq!(url_encode("Halo: MCC"), "Halo%3A%20MCC");
        assert_eq!(format_playtime(30), "30 Min.");
        assert_eq!(format_playtime(90), "1,5 Std.");
        assert_eq!(format_bytes(5 * 1_073_741_824), "5,0 GB");
    }
}
