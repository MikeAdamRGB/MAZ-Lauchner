//! Spiele starten – und bei Problemen genau erklären, was los ist.

use crate::model::{CustomGame, Game, LaunchTarget};
use crate::problem::Problem;
use crate::{epic, sys};
use std::path::Path;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

/// Steam- oder Epic-Spiel über den jeweiligen Launcher starten.
pub fn launch_store(game: &Game) -> Result<(), Problem> {
    match &game.launch {
        LaunchTarget::SteamRun(id) | LaunchTarget::SteamInstall(id) => {
            if !sys::protocol_registered("steam") {
                return Err(Problem::steam_missing());
            }
            let uri = if matches!(game.launch, LaunchTarget::SteamRun(_)) {
                format!("steam://rungameid/{id}")
            } else {
                format!("steam://install/{id}")
            };
            sys::open_url(&uri).map_err(|code| Problem::launcher_uri_failed(&game.name, "Steam", code, &uri))
        }
        LaunchTarget::Epic(uri) | LaunchTarget::EpicInstall(uri) => {
            if !epic::launcher_installed() {
                return Err(if uri.starts_with("heroic://") { Problem::heroic_missing() } else { Problem::epic_missing() });
            }
            sys::open_url(uri).map_err(|code| Problem::launcher_uri_failed(&game.name, "Epic Games", code, uri))
        }
        LaunchTarget::Roblox(place) => crate::roblox::launch(*place),
        LaunchTarget::Uri(uri) => {
            let scheme = uri.split(':').next().unwrap_or("");
            if !scheme.starts_with("http") && !sys::protocol_registered(scheme) {
                return Err(client_missing(game));
            }
            sys::open_url(uri).map_err(|code| Problem::launcher_uri_failed(&game.name, &game.source.label(), code, uri))
        }
        LaunchTarget::Exe { path, args, dir } => {
            if path.is_empty() || !Path::new(path).exists() {
                return Err(Problem::exe_missing(&game.name, path, sys::drive_exists(path)));
            }
            let mut cmd = sys::host_command(path);
            cmd.args(args);
            let wd = if dir.is_empty() { Path::new(path).parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default() } else { dir.clone() };
            if !wd.is_empty() && Path::new(&wd).is_dir() {
                cmd.current_dir(&wd);
            }
            cmd.spawn().map(|_| ()).map_err(|e| Problem::from_os_error(&game.name, path, e.raw_os_error(), &e.to_string()))
        }
        LaunchTarget::Appx { family, app } => crate::appx::launch(family, app).map_err(|e| Problem::unknown(&game.name, &e)),
        LaunchTarget::Custom => Err(Problem::unknown(&game.name, "Internal error: custom game without data")),
    }
}

/// Store-Programm fehlt → freundlich erklären, wo es das gibt
fn client_missing(game: &Game) -> Problem {
    let url = crate::stores::library(game.source).map(|s| s.download_url()).unwrap_or(sys::GITHUB_URL);
    Problem::client_missing(&game.source.label(), url)
}

/// Prüft ein eigenes Spiel, bevor die Sicherheitsfrage gestellt wird.
pub fn precheck_custom(c: &CustomGame) -> Result<(), Problem> {
    let exe = c.exe.trim().trim_matches('"');
    if exe.is_empty() {
        return Err(Problem::exe_missing(&c.name, "(-)", true));
    }
    if !Path::new(exe).exists() {
        return Err(Problem::exe_missing(&c.name, exe, sys::drive_exists(exe)));
    }
    let wd = c.workdir.trim().trim_matches('"');
    if !wd.is_empty() && !Path::new(wd).is_dir() {
        return Err(Problem::workdir_missing(&c.name, wd));
    }
    Ok(())
}

/// Eigenes Spiel starten. Gibt bei normalen EXEs den Prozess zurück, damit wir
/// einen Absturz direkt nach dem Start erkennen und erklären können.
pub fn launch_custom(c: &CustomGame, force_admin: bool) -> Result<Option<Child>, Problem> {
    precheck_custom(c)?;
    let exe = c.exe.trim().trim_matches('"').to_string();
    let path = Path::new(&exe);
    let wd = c.workdir.trim().trim_matches('"');
    let workdir = if wd.is_empty() {
        path.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()
    } else {
        wd.to_string()
    };
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    // Unter macOS/Linux startet immer shell_start (dort gibt es keine Windows-Fehlercodes)
    let is_exe = cfg!(windows) && (ext == "exe" || ext == "com");
    let as_admin = c.admin || force_admin;

    if as_admin || !is_exe {
        return shell_start(c, &exe, &workdir, as_admin).map(|_| None);
    }

    let mut cmd = Command::new(path);
    if !workdir.is_empty() {
        cmd.current_dir(&workdir);
    }
    let args = c.args.trim();
    if !args.is_empty() {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.raw_arg(args);
        }
        #[cfg(not(windows))]
        {
            cmd.args(args.split_whitespace());
        }
    }
    match cmd.spawn() {
        Ok(child) => Ok(Some(child)),
        Err(e) => Err(Problem::from_os_error(&c.name, &exe, e.raw_os_error(), &e.to_string())),
    }
}

fn shell_start(c: &CustomGame, exe: &str, workdir: &str, as_admin: bool) -> Result<(), Problem> {
    #[cfg(windows)]
    {
        let verb = if as_admin { "runas" } else { "open" };
        crate::winffi::shell_execute(Some(verb), exe, Some(c.args.trim()), Some(workdir))
            .map_err(|code| Problem::from_shell_code(&c.name, exe, code, as_admin))
    }
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = as_admin;
        let p = Path::new(exe);
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
        let executable = p.is_file() && p.metadata().map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false);
        let mut cmd = if ext == "exe" && !cfg!(target_os = "macos") {
            // Windows-Programm unter Linux: über Wine starten (falls installiert)
            let mut cmd = sys::host_command("wine");
            cmd.arg(p);
            if !c.args.trim().is_empty() {
                cmd.args(c.args.split_whitespace());
            }
            cmd
        } else if executable && ext != "desktop" {
            let mut cmd = sys::host_command(p);
            let args = c.args.trim();
            if !args.is_empty() {
                cmd.args(args.split_whitespace());
            }
            cmd
        } else if cfg!(target_os = "macos") {
            // .app-Bundles und andere Dateien über "open"
            let mut cmd = Command::new("/usr/bin/open");
            cmd.arg(p);
            if !c.args.trim().is_empty() {
                cmd.arg("--args").args(c.args.split_whitespace());
            }
            cmd
        } else {
            // .desktop-Dateien und alles andere: mit dem Standardprogramm öffnen
            let mut cmd = sys::host_command(if ext == "desktop" { "gio" } else { "xdg-open" });
            if ext == "desktop" {
                cmd.arg("launch");
            }
            cmd.arg(p);
            cmd
        };
        if !workdir.is_empty() && Path::new(workdir).is_dir() {
            cmd.current_dir(workdir);
        }
        cmd.spawn()
            .map(|_| ())
            .map_err(|e| Problem::from_os_error(&c.name, exe, e.raw_os_error(), &e.to_string()))
    }
}

/// Beobachtet ein gestartetes Spiel ein paar Sekunden lang.
/// Stürzt es sofort ab, gibt es eine verständliche Erklärung zurück.
pub fn watch(mut child: Child, name: String, exe: String) -> Option<Problem> {
    let start = Instant::now();
    let limit = Duration::from_secs(10);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let code = status.code().map(|c| c as u32).unwrap_or(0);
                return Problem::from_exit_code(&name, &exe, code);
            }
            Ok(None) => {
                if start.elapsed() >= limit {
                    return None;
                }
                std::thread::sleep(Duration::from_millis(250));
            }
            Err(_) => return None,
        }
    }
}

/// "Spieldateien überprüfen" für Steam- und Epic-Spiele
pub fn verify_fix(game: &Game) -> Option<crate::problem::Fix> {
    use crate::problem::Fix;
    match &game.launch {
        LaunchTarget::SteamRun(id) => Some(Fix::OpenUrl {
            label: crate::i18n::t("fix.verify"),
            url: format!("steam://validate/{id}"),
        }),
        LaunchTarget::Epic(uri) if uri.starts_with("com.epicgames.launcher://") => Some(Fix::OpenUrl {
            label: crate::i18n::t("fix.verify"),
            url: uri.replace("action=launch", "action=verify"),
        }),
        _ => None,
    }
}

/// Programme im Spielordner, die nicht das Spiel selbst sind
fn is_helper(path: &str) -> bool {
    const HELPERS: [&str; 14] = [
        "crashreport", "crashhandler", "crashpad", "unitycrashhandler", "easyanticheat", "battleye", "redist",
        "vc_redist", "vcredist", "dxsetup", "dotnet", "uninst", "setup", "launcherpatcher",
    ];
    let file = path.rsplit(['\\', '/']).next().unwrap_or(path).to_lowercase();
    HELPERS.iter().any(|h| file.contains(h))
}

/// Beobachtet ein Steam-/Epic-Spiel nach dem Start: Startet ein Programm aus dem
/// Spielordner und beendet sich innerhalb von 20 Sekunden mit einem Fehlercode,
/// wird erklärt, was los ist. Läuft das Spiel länger, ist alles gut.
#[cfg(windows)]
pub fn watch_store(game: &Game) -> Option<Problem> {
    use crate::winffi::{process_ids, Proc};
    use std::collections::HashSet;

    let dir = game.install_dir.as_ref()?.trim_end_matches(['\\', '/']).to_lowercase().replace('/', "\\");
    if dir.len() < 4 {
        return None;
    }
    let prefix = format!("{dir}\\");
    let store = game.source.label();
    let verify = verify_fix(game)?;

    // Bereits laufende Prozesse ignorieren
    let mut seen: HashSet<u32> = process_ids().into_iter().map(|(pid, _)| pid).collect();
    let started = Instant::now();
    let mut tracked: Vec<(Proc, String, Instant)> = Vec::new();
    let mut crash: Option<(String, u32)> = None;
    let mut last_activity = Instant::now();
    let mut found_any = false;

    loop {
        std::thread::sleep(Duration::from_millis(500));
        for (pid, _) in process_ids() {
            if !seen.insert(pid) {
                continue;
            }
            let Some(p) = Proc::open(pid) else { continue };
            let Some(path) = p.path() else { continue };
            if !path.to_lowercase().starts_with(&prefix) || is_helper(&path) {
                continue;
            }
            found_any = true;
            last_activity = Instant::now();
            tracked.push((p, path, Instant::now()));
        }

        let mut i = 0;
        while i < tracked.len() {
            let (p, path, since) = &tracked[i];
            match p.exit_code() {
                None => {
                    // Läuft seit 20 Sekunden → Spiel ist erfolgreich gestartet
                    if since.elapsed() >= Duration::from_secs(20) {
                        return None;
                    }
                    i += 1;
                }
                Some(code) => {
                    if code != 0 && since.elapsed() < Duration::from_secs(20) {
                        crash = Some((path.clone(), code));
                    }
                    last_activity = Instant::now();
                    tracked.remove(i);
                }
            }
        }

        if !found_any {
            // Nach 2 Minuten ohne Spielprozess aufgeben (z. B. Update läuft noch)
            if started.elapsed() > Duration::from_secs(120) {
                return None;
            }
        } else if tracked.is_empty() && last_activity.elapsed() > Duration::from_secs(6) {
            // Alles beendet und nichts Neues gestartet
            return crash.map(|(path, code)| Problem::store_crash(&game.name, &store, &path, code, verify));
        }
        if started.elapsed() > Duration::from_secs(300) {
            return None;
        }
    }
}

#[cfg(not(windows))]
pub fn watch_store(_game: &Game) -> Option<Problem> {
    None
}

/// Ein gefundenes Deinstallationsprogramm
pub struct Uninstaller {
    pub display_name: String,
    pub program: String,
    pub args: String,
}

/// Ist das ein Ordner, der zu genau einem Programm gehört? ("C:\\Program Files" ist es nicht.)
fn specific_dir(loc: &str) -> bool {
    let parts: Vec<&str> = loc.split(['\\', '/']).filter(|p| !p.is_empty()).collect();
    if parts.len() < 2 {
        return false;
    }
    let last = parts[parts.len() - 1].to_lowercase();
    const ROOTS: [&str; 12] = [
        "program files", "program files (x86)", "programdata", "windows", "users", "games", "spiele",
        "steamapps", "common", "appdata", "local", "programs",
    ];
    !ROOTS.contains(&last.as_str())
}

/// Deinstallationsprogramm eines eigenen Spiels aus der Windows-Registry suchen.
pub fn find_uninstaller(c: &CustomGame) -> Option<Uninstaller> {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        let exe = c.exe.trim().trim_matches('"').to_lowercase().replace('/', "\\");
        let exe_dir = Path::new(&exe).parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
        let name = crate::model::normalize_name(&c.name);
        let roots = [
            (HKEY_LOCAL_MACHINE, "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
            (HKEY_LOCAL_MACHINE, "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
            (HKEY_CURRENT_USER, "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall"),
        ];
        let mut by_name: Option<Uninstaller> = None;
        for (hive, path) in roots {
            let Ok(root) = RegKey::predef(hive).open_subkey(path) else { continue };
            for sub in root.enum_keys().flatten() {
                let Ok(k) = root.open_subkey(&sub) else { continue };
                let get = |n: &str| k.get_value::<String, _>(n).unwrap_or_default();
                let uninstall = get("UninstallString");
                if uninstall.trim().is_empty() {
                    continue;
                }
                let loc = get("InstallLocation").trim().trim_matches('"').trim_end_matches('\\').to_lowercase();
                let icon = get("DisplayIcon").to_lowercase();
                let display_name = get("DisplayName");
                // Nur eindeutige Treffer: genau diese EXE als Icon, oder ein Ordner, der wirklich
                // nur zu diesem Programm gehört (nicht "C:\\Program Files" o. Ä.).
                let matches_dir = specific_dir(&loc) && (exe.starts_with(&format!("{loc}\\")) || exe_dir == loc);
                let matches_icon = !icon.is_empty() && icon.trim_matches('"').split(',').next().map(|i| i.trim_matches('"') == exe).unwrap_or(false);
                if matches_dir || matches_icon {
                    let (program, args) = split_command(&uninstall);
                    return Some(Uninstaller { display_name, program, args });
                }
                let dn = crate::model::normalize_name(&display_name);
                if name.len() >= 4 && dn == name && by_name.is_none() {
                    let (program, args) = split_command(&uninstall);
                    by_name = Some(Uninstaller { display_name, program, args });
                }
            }
        }
        by_name
    }
    #[cfg(not(windows))]
    {
        let _ = c;
        None
    }
}

/// "\"C:\\x\\unins000.exe\" /SILENT" → ("C:\\x\\unins000.exe", "/SILENT")
pub fn split_command(cmd: &str) -> (String, String) {
    let cmd = cmd.trim();
    if let Some(rest) = cmd.strip_prefix('"') {
        if let Some(end) = rest.find('"') {
            return (rest[..end].to_string(), rest[end + 1..].trim().to_string());
        }
    }
    let lower = cmd.to_lowercase();
    if let Some(i) = lower.find(".exe") {
        let end = i + 4;
        return (cmd[..end].to_string(), cmd[end..].trim().to_string());
    }
    match cmd.split_once(' ') {
        Some((a, b)) => (a.to_string(), b.trim().to_string()),
        None => (cmd.to_string(), String::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uninstall_command_split() {
        assert_eq!(split_command("\"C:\\G\\unins000.exe\" /x"), ("C:\\G\\unins000.exe".into(), "/x".into()));
        assert_eq!(split_command("MsiExec.exe /X{ABC}"), ("MsiExec.exe".into(), "/X{ABC}".into()));
        assert_eq!(split_command("C:\\Program Files\\G\\uninstall.exe"), ("C:\\Program Files\\G\\uninstall.exe".into(), "".into()));
        assert!(is_helper("C:\\G\\UnityCrashHandler64.exe"));
        assert!(!is_helper("C:\\G\\Game.exe"));
        assert!(!specific_dir("c:\\program files"));
        assert!(!specific_dir("c:\\"));
        assert!(specific_dir("c:\\program files\\cool game"));
    }
}
