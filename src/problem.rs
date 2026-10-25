//! Verständliche Fehlererklärungen.
//!
//! Jeder Fehler wird in ein [`Problem`] übersetzt: Was ist passiert, warum,
//! und Schritt für Schritt, wie man es behebt – in einfacher Sprache und in
//! der gewählten Sprache (Texte in `lang/*.txt`, Schlüssel `p.<id>.*`).
//! Technische Details sind immer Englisch (praktisch für GitHub-Issues).

use crate::i18n::{t, tf, tlf};

#[derive(Clone, Debug, PartialEq)]
pub enum Fix {
    /// Spiel mit Administratorrechten erneut starten
    RunAsAdmin,
    /// "Spiel bearbeiten"-Fenster öffnen (z. B. um die EXE neu auszuwählen)
    EditGame,
    /// Eine Webseite öffnen
    OpenUrl { label: String, url: String },
    /// Einen Ordner im Explorer öffnen
    OpenFolder { label: String, path: String },
    /// Bibliothek neu einlesen
    Rescan,
    /// Einstellungen öffnen
    OpenSettings,
    /// Eigenes Spiel aus dem Launcher entfernen
    RemoveGame,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Problem {
    pub severity: Severity,
    pub title: String,
    pub what_happened: String,
    pub why: String,
    pub steps: Vec<String>,
    pub fixes: Vec<Fix>,
    /// Technische Details für Profis / GitHub-Issues
    pub technical: String,
}

const VCREDIST_X64: &str = "https://aka.ms/vs/17/release/vc_redist.x64.exe";
const VCREDIST_X86: &str = "https://aka.ms/vs/17/release/vc_redist.x86.exe";
const DIRECTX: &str = "https://www.microsoft.com/download/details.aspx?id=35";
const STEAM_DL: &str = "https://store.steampowered.com/about/";
const EPIC_DL: &str = "https://store.epicgames.com/download";
const WEBVIEW2_DL: &str = "https://go.microsoft.com/fwlink/p/?LinkId=2124703";
const STEAM_API_KEY_URL: &str = "https://steamcommunity.com/dev/apikey";
const NVIDIA: &str = "https://www.nvidia.com/drivers/";
const AMD: &str = "https://www.amd.com/support/download/drivers.html";
const INTEL: &str = "https://www.intel.com/content/www/us/en/support/detect.html";

/// Ordner, in dem eine Datei liegt (für "Ordner öffnen").
fn parent_of(path: &str) -> String {
    match path.rfind(['\\', '/']) {
        Some(i) => path[..i].to_string(),
        None => path.to_string(),
    }
}

/// Laufwerksbuchstabe wie "D:" aus einem Pfad.
pub fn drive_of(path: &str) -> Option<String> {
    let b = path.as_bytes();
    if b.len() >= 2 && b[1] == b':' && b[0].is_ascii_alphabetic() {
        Some(format!("{}:", (b[0] as char).to_ascii_uppercase()))
    } else {
        None
    }
}

fn url(label_key: &str, url: &str) -> Fix {
    Fix::OpenUrl { label: t(label_key), url: url.to_string() }
}

fn folder(label_key: &str, path: String) -> Fix {
    Fix::OpenFolder { label: t(label_key), path }
}

/// Baut ein Problem aus den Texten `p.<id>.title/what/why/steps`.
fn build(id: &str, severity: Severity, args: &[(&str, &str)], fixes: Vec<Fix>, technical: String) -> Problem {
    Problem {
        severity,
        title: tf(&format!("p.{id}.title"), args),
        what_happened: tf(&format!("p.{id}.what"), args),
        why: tf(&format!("p.{id}.why"), args),
        steps: tlf(&format!("p.{id}.steps"), args),
        fixes,
        technical,
    }
}

impl Problem {
    // ------------------------------------------------------------------
    // Eigene Spiele: Probleme vor dem Start
    // ------------------------------------------------------------------

    pub fn exe_missing(game: &str, path: &str, drive_exists: bool) -> Problem {
        if !drive_exists {
            let drive = drive_of(path).unwrap_or_else(|| "?:".into());
            return build(
                "drive_missing",
                Severity::Error,
                &[("game", game), ("drive", &drive)],
                vec![Fix::EditGame],
                format!("Path: {path}\nDrive {drive} does not exist."),
            );
        }
        build(
            "exe_missing",
            Severity::Error,
            &[("game", game)],
            vec![Fix::EditGame, folder("fix.folder_last", parent_of(path))],
            format!("File does not exist: {path}"),
        )
    }

    pub fn workdir_missing(game: &str, dir: &str) -> Problem {
        build("workdir_missing", Severity::Error, &[("game", game)], vec![Fix::EditGame], format!("Working directory does not exist: {dir}"))
    }

    // ------------------------------------------------------------------
    // Eigene Spiele: Fehler beim Starten (Windows-Fehlercodes)
    // ------------------------------------------------------------------

    /// Übersetzt einen Windows-Fehler beim Starten eines Programms.
    pub fn from_os_error(game: &str, path: &str, code: Option<i32>, message: &str) -> Problem {
        let tech = format!(
            "Path: {path}\nWindows error code: {}\nMessage: {message}",
            code.map(|c| c.to_string()).unwrap_or_else(|| "unknown".into())
        );
        let g = [("game", game)];
        let gf = || folder("fix.folder_game", parent_of(path));
        match code {
            Some(2) | Some(3) => {
                let mut p = Problem::exe_missing(game, path, true);
                p.technical = tech;
                p
            }
            Some(740) => build("admin_required", Severity::Warning, &g, vec![Fix::RunAsAdmin, Fix::EditGame], tech),
            Some(5) => build("access_denied", Severity::Error, &g, vec![Fix::RunAsAdmin, gf()], tech),
            Some(193) | Some(216) | Some(129) => build("not_an_app", Severity::Error, &g, vec![Fix::EditGame, gf()], tech),
            Some(225) => build("virus", Severity::Error, &g, vec![url("fix.defender", "windowsdefender://threat")], tech),
            Some(1260) | Some(4551) | Some(4556) => {
                build("blocked", Severity::Error, &g, vec![url("fix.defender", "windowsdefender://appbrowser")], tech)
            }
            Some(32) | Some(33) => build("in_use", Severity::Warning, &g, vec![], tech),
            Some(1155) => build("no_assoc", Severity::Error, &g, vec![Fix::EditGame], tech),
            _ => Problem::unknown(game, &tech),
        }
    }

    /// Übersetzt den Exit-Code eines Spiels, das direkt nach dem Start abgestürzt ist.
    pub fn from_exit_code(game: &str, path: &str, code: u32) -> Option<Problem> {
        let tech = format!("Path: {path}\nExit code: {code} (0x{code:08X})");
        let g = [("game", game)];
        let p = match code {
            0 => return None,
            0xC000_0135 | 0xC06D_007E => build(
                "dll_missing",
                Severity::Error,
                &g,
                vec![
                    url("fix.vc64", VCREDIST_X64),
                    url("fix.vc86", VCREDIST_X86),
                    url("fix.directx", DIRECTX),
                    folder("fix.folder_game", parent_of(path)),
                ],
                tech,
            ),
            0xC000_007B => build(
                "bad_image",
                Severity::Error,
                &g,
                vec![url("fix.vc64", VCREDIST_X64), url("fix.vc86", VCREDIST_X86), url("fix.directx", DIRECTX)],
                tech,
            ),
            0xC000_0142 => build(
                "dll_init",
                Severity::Error,
                &g,
                vec![url("fix.nvidia", NVIDIA), url("fix.amd", AMD), url("fix.intel", INTEL), url("fix.vc64", VCREDIST_X64)],
                tech,
            ),
            0xC000_0005 | 0xC000_0409 | 0xC000_001D | 0xC000_0374 | 0xC000_00FD => build(
                "crash",
                Severity::Error,
                &g,
                vec![Fix::RunAsAdmin, url("fix.nvidia", NVIDIA), url("fix.amd", AMD), url("fix.intel", INTEL)],
                tech,
            ),
            _ => build("quit_early", Severity::Warning, &g, vec![Fix::RunAsAdmin, Fix::EditGame, url("fix.vc64", VCREDIST_X64)], tech),
        };
        Some(p)
    }

    // ------------------------------------------------------------------
    // Steam- und Epic-Spiele: Absturz direkt nach dem Start, Hilfe, Deinstallieren
    // ------------------------------------------------------------------

    /// Ein Steam-/Epic-Spiel hat sich direkt nach dem Start mit Fehlercode beendet.
    /// `verify` = Knopf "Spieldateien überprüfen" für den jeweiligen Store.
    pub fn store_crash(game: &str, store: &str, path: &str, code: u32, verify: Fix) -> Problem {
        let tech = format!("Store: {store}\nPath: {path}\nExit code: {code} (0x{code:08X})");
        match code {
            0xC000_0135 | 0xC06D_007E | 0xC000_007B | 0xC000_0142 => {
                let mut p = Problem::from_exit_code(game, path, code).unwrap_or_else(|| Problem::unknown(game, &tech));
                p.fixes.retain(|f| matches!(f, Fix::OpenUrl { .. } | Fix::OpenFolder { .. }));
                p.fixes.insert(0, verify);
                p.technical = tech;
                p
            }
            _ => build(
                "store_crash",
                Severity::Error,
                &[("game", game), ("store", store)],
                vec![
                    verify,
                    url("fix.vc64", VCREDIST_X64),
                    url("fix.vc86", VCREDIST_X86),
                    url("fix.directx", DIRECTX),
                    url("fix.nvidia", NVIDIA),
                    url("fix.amd", AMD),
                    url("fix.intel", INTEL),
                    folder("fix.folder_game", parent_of(path)),
                ],
                tech,
            ),
        }
    }

    /// "Spiel startet nicht?" – alle typischen Lösungen auf einen Blick.
    pub fn start_help(game: &str, store: &str, verify: Option<Fix>, dir: Option<String>, custom: bool) -> Problem {
        let mut fixes = Vec::new();
        if custom {
            fixes.push(Fix::RunAsAdmin);
            fixes.push(Fix::EditGame);
        }
        if let Some(v) = verify {
            fixes.push(v);
        }
        fixes.extend([
            url("fix.vc64", VCREDIST_X64),
            url("fix.vc86", VCREDIST_X86),
            url("fix.directx", DIRECTX),
            url("fix.nvidia", NVIDIA),
            url("fix.amd", AMD),
            url("fix.intel", INTEL),
        ]);
        if let Some(d) = dir {
            fixes.push(folder("fix.folder_game", d));
        }
        build(
            if custom { "start_help_custom" } else { "start_help" },
            Severity::Info,
            &[("game", game), ("store", store)],
            fixes,
            format!("Store: {store}"),
        )
    }

    /// Roblox ist nicht installiert (Linux: Sober fehlt)
    pub fn roblox_missing() -> Problem {
        let (url_, label) = if cfg!(target_os = "linux") {
            ("https://flathub.org/apps/org.vinegarhq.Sober", "fix.sober_dl")
        } else {
            ("https://www.roblox.com/download", "fix.roblox_dl")
        };
        build(
            if cfg!(target_os = "linux") { "roblox_missing_linux" } else { "roblox_missing" },
            Severity::Error,
            &[],
            vec![url(label, url_), Fix::Rescan],
            "Roblox client not found.".into(),
        )
    }

    /// Epic hat keinen direkten "Deinstallieren"-Link → Anleitung + Launcher öffnen
    pub fn epic_uninstall(game: &str) -> Problem {
        build(
            "epic_uninstall",
            Severity::Info,
            &[("game", game)],
            vec![url("fix.epic_open", "com.epicgames.launcher://")],
            "Epic Games Launcher has no uninstall URI.".into(),
        )
    }

    /// Für ein eigenes Spiel wurde kein Deinstallationsprogramm gefunden
    pub fn no_uninstaller(game: &str, dir: Option<String>) -> Problem {
        let mut fixes = vec![url("fix.apps", "ms-settings:appsfeatures")];
        if let Some(d) = dir {
            fixes.push(folder("fix.folder_game", d));
        }
        fixes.push(Fix::RemoveGame);
        build("no_uninstaller", Severity::Info, &[("game", game)], fixes, "No matching entry in ...\\CurrentVersion\\Uninstall".into())
    }

    pub fn uac_cancelled(game: &str) -> Problem {
        build(
            "uac_cancelled",
            Severity::Info,
            &[("game", game)],
            vec![Fix::RunAsAdmin, Fix::EditGame],
            "ShellExecute(runas) -> SE_ERR_ACCESSDENIED / ERROR_CANCELLED".into(),
        )
    }

    /// ShellExecute-Fehlercodes (≤ 32) in Windows-Fehlercodes übersetzen.
    pub fn from_shell_code(game: &str, path: &str, code: i32, as_admin: bool) -> Problem {
        let os = match code {
            2 => 2,
            3 => 3,
            5 if as_admin => return Problem::uac_cancelled(game),
            5 => 5,
            11 => 193,
            26 => 32,
            27 | 31 => 1155,
            32 => 126,
            other => other,
        };
        if os == 126 {
            if let Some(p) = Problem::from_exit_code(game, path, 0xC000_0135) {
                return p;
            }
        }
        Problem::from_os_error(game, path, Some(os), &format!("ShellExecute code {code}"))
    }

    // ------------------------------------------------------------------
    // Steam / Epic / Installation
    // ------------------------------------------------------------------

    pub fn steam_missing() -> Problem {
        build(
            "steam_missing",
            Severity::Error,
            &[],
            vec![url("fix.steam_dl", STEAM_DL), Fix::Rescan],
            "No handler registered for steam:// (HKEY_CLASSES_ROOT\\steam missing).".into(),
        )
    }

    pub fn epic_missing() -> Problem {
        build(
            "epic_missing",
            Severity::Error,
            &[],
            vec![url("fix.epic_dl", EPIC_DL), Fix::Rescan],
            "No handler registered for com.epicgames.launcher://.".into(),
        )
    }

    /// Store-Programm (GOG Galaxy, Ubisoft Connect …) fehlt
    pub fn client_missing(store: &str, download: &str) -> Problem {
        build(
            "client_missing",
            Severity::Error,
            &[("store", store)],
            vec![Fix::OpenUrl { label: tf("fix.client_dl", &[("store", store)]), url: download.to_string() }, Fix::Rescan],
            format!("Protocol handler for {store} not registered."),
        )
    }

    /// Spiel aus einem anderen Store deinstallieren: Das macht der Store selbst
    pub fn store_uninstall(game: &str, store: &str) -> Problem {
        let mut fixes = Vec::new();
        if cfg!(windows) {
            fixes.push(url("fix.apps", "ms-settings:appsfeatures"));
        }
        fixes.push(Fix::Rescan);
        build("store_uninstall", Severity::Info, &[("game", game), ("store", store)], fixes, String::new())
    }

    /// Linux: Epic-Spiele laufen über den Heroic Games Launcher
    pub fn heroic_missing() -> Problem {
        build(
            "heroic_missing",
            Severity::Error,
            &[],
            vec![url("fix.heroic_dl", "https://flathub.org/apps/com.heroicgameslauncher.hgl"), Fix::Rescan],
            "Heroic Games Launcher config not found (~/.config/heroic).".into(),
        )
    }

    pub fn launcher_uri_failed(game: &str, store: &str, code: i32, uri: &str) -> Problem {
        build(
            "uri_failed",
            Severity::Error,
            &[("game", game), ("store", store)],
            vec![Fix::Rescan],
            format!("ShellExecute code: {code}\nURI: {uri}"),
        )
    }

    pub fn shortcut_failed(path: &str, message: &str) -> Problem {
        build(
            "shortcut_failed",
            Severity::Warning,
            &[],
            vec![folder("fix.folder_app", parent_of(path))],
            format!("Target: {path}\nError: {message}"),
        )
    }

    pub fn install_failed(dir: &str, message: &str) -> Problem {
        build("install_failed", Severity::Error, &[], vec![], format!("Target folder: {dir}\nError: {message}"))
    }

    pub fn steam_api(status: Option<u16>, message: &str) -> Problem {
        match status {
            Some(401) | Some(403) => build(
                "steam_key",
                Severity::Warning,
                &[],
                vec![url("fix.api_key", STEAM_API_KEY_URL), Fix::OpenSettings],
                format!("HTTP {}: {message}", status.unwrap_or(0)),
            ),
            _ => build("steam_api", Severity::Info, &[], vec![Fix::OpenSettings, Fix::Rescan], format!("HTTP {status:?}: {message}")),
        }
    }

    pub fn webview2_missing() -> Problem {
        build(
            "webview2",
            Severity::Error,
            &[],
            vec![url("fix.webview2", WEBVIEW2_DL)],
            "WebView2 runtime not found in registry (EdgeUpdate\\Clients\\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}).".into(),
        )
    }

    pub fn app_crashed(log: &str, message: &str, issues_url: &str) -> Problem {
        build(
            "app_crashed",
            Severity::Error,
            &[("log", log)],
            vec![Fix::OpenUrl { label: t("fix.report"), url: issues_url.to_string() }],
            message.to_string(),
        )
    }

    pub fn unknown(game: &str, technical: &str) -> Problem {
        build("unknown", Severity::Error, &[("game", game)], vec![Fix::RunAsAdmin, Fix::EditGame], technical.to_string())
    }

    /// Kurzer Text für die Zwischenablage / GitHub-Issues.
    pub fn to_clipboard_text(&self) -> String {
        let mut out = format!(
            "MAZ Launcher – {}\n\n{}\n{}\n\n{}\n{}\n\n{}\n",
            self.title,
            t("problem.what"),
            self.what_happened,
            t("problem.why"),
            self.why,
            t("problem.steps")
        );
        for (i, st) in self.steps.iter().enumerate() {
            out.push_str(&format!("{}. {}\n", i + 1, st));
        }
        out.push_str(&format!("\n{}\n{}\n", t("problem.tech_short"), self.technical));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drive_detection() {
        assert_eq!(drive_of("d:\\Games\\x.exe"), Some("D:".into()));
        assert_eq!(drive_of("\\\\server\\share"), None);
    }

    #[test]
    fn problems_are_translated() {
        crate::i18n::set_language("de");
        let p = Problem::exe_missing("Test", "E:\\Spiele\\test.exe", false);
        assert!(p.title.contains("E:"), "{}", p.title);
        assert!(!p.steps.is_empty());
        assert_eq!(Problem::from_os_error("x", "C:\\a.exe", Some(740), "").fixes[0], Fix::RunAsAdmin);
        assert!(Problem::from_exit_code("x", "C:\\a.exe", 0).is_none());
        for code in [0xC000_0135u32, 0xC000_007B, 0xC000_0142, 0xC000_0005, 1] {
            let p = Problem::from_exit_code("x", "C:\\a.exe", code).unwrap();
            assert!(!p.title.starts_with("p."), "Schlüssel fehlt: {}", p.title);
            assert!(!p.steps.is_empty());
        }
        for code in [5, 193, 225, 1260, 32, 1155, 99999] {
            let p = Problem::from_os_error("x", "C:\\a.exe", Some(code), "");
            assert!(!p.title.starts_with("p."), "Schlüssel fehlt: {}", p.title);
        }
        for p in [
            Problem::steam_missing(),
            Problem::epic_missing(),
            Problem::launcher_uri_failed("x", "Steam", 2, "steam://"),
            Problem::shortcut_failed("C:\\a.exe", "x"),
            Problem::install_failed("C:\\", "x"),
            Problem::steam_api(Some(403), ""),
            Problem::steam_api(None, ""),
            Problem::webview2_missing(),
            Problem::uac_cancelled("x"),
            Problem::workdir_missing("x", "C:\\"),
            Problem::app_crashed("log", "msg", "url"),
            Problem::store_crash("x", "Steam", "C:\\a.exe", 1, Fix::Rescan),
            Problem::store_crash("x", "Steam", "C:\\a.exe", 0xC000_0135, Fix::Rescan),
            Problem::start_help("x", "Steam", None, None, false),
            Problem::start_help("x", "x", None, None, true),
            Problem::epic_uninstall("x"),
            Problem::roblox_missing(),
            Problem::heroic_missing(),
            Problem::client_missing("GOG", "https://www.gog.com/galaxy"),
            Problem::no_uninstaller("x", None),
            Problem::store_uninstall("x", "GOG"),
        ] {
            assert!(!p.title.starts_with("p."), "Schlüssel fehlt: {}", p.title);
            assert!(!p.steps.is_empty(), "{}", p.title);
        }
        assert_eq!(parent_of("C:\\a\\b.exe"), "C:\\a");
    }
}
