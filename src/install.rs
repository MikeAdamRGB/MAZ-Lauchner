//! Eingebauter Installer: Die EXE installiert sich selbst.
//!
//! - kopiert sich nach %LOCALAPPDATA%\Programs\Open Game Launcher (kein Admin nötig)
//! - Desktop-Verknüpfung + Startmenü-Eintrag
//! - Eintrag unter "Apps & Features" inkl. Deinstallation

use crate::problem::Problem;
use crate::sys;
use std::fs;
use std::path::{Path, PathBuf};

const UNINSTALL_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\MAZLauncher";

#[derive(Clone, Debug, PartialEq)]
pub enum StartMode {
    Launcher,
    Installer { existing: Option<String> },
}

pub fn current_exe() -> Option<PathBuf> {
    std::env::current_exe().ok()
}

fn same_dir(a: &Path, b: &Path) -> bool {
    let norm = |p: &Path| {
        p.to_string_lossy()
            .trim_end_matches(['\\', '/'])
            .replace('/', "\\")
            .to_lowercase()
    };
    norm(a) == norm(b)
}

/// Wo ist der Launcher installiert? (aus der Registry)
pub fn installed_location() -> Option<PathBuf> {
    sys::reg_str(sys::Hive::CurrentUser, UNINSTALL_KEY, "InstallLocation")
        .map(PathBuf::from)
        .filter(|p| p.join(sys::EXE_NAME).exists() || p.join(sys::OLD_EXE_NAME).exists())
}

/// Ordnername gehört dem Launcher (neuer oder alter Name)?
fn is_own_dir_name(dir: &Path) -> bool {
    dir.file_name()
        .map(|n| {
            let n = n.to_string_lossy();
            n.eq_ignore_ascii_case(sys::APP_NAME) || n.eq_ignore_ascii_case(sys::OLD_APP_NAME)
        })
        .unwrap_or(false)
}

fn is_own_file_name(name: &str) -> bool {
    let n = name.to_lowercase();
    n.starts_with(&sys::EXE_NAME.to_lowercase()) || n.starts_with(&sys::OLD_EXE_NAME.to_lowercase()) || n == "portable.txt"
}

/// Nach einem Update (und nach der Umbenennung in 1.8): Eintrag in „Apps & Features“,
/// Verknüpfungen und Symbole auffrischen. Die EXE bleibt, wo sie ist.
pub fn migrate_names() {
    #[cfg(windows)]
    {
        let Some(loc) = installed_location() else { return };
        let Some(exe) = current_exe() else { return };
        if !exe.parent().map(|d| same_dir(d, &loc)).unwrap_or(false) {
            return;
        }
        // Einmal pro neuer Version: Eintrag, Verknüpfungen und Symbole auffrischen
        let marker = sys::local_dir().join(format!("shell-{}.done", sys::VERSION));
        if marker.exists() {
            return;
        }
        register_uninstall(&loc.to_string_lossy(), &exe.to_string_lossy());
        for folder in [sys::desktop_dir(), sys::start_menu_dir()] {
            for ext in ["lnk", "url"] {
                let old = folder.join(format!("{}.{ext}", sys::OLD_APP_NAME));
                let new = folder.join(format!("{}.{ext}", sys::APP_NAME));
                if old.exists() && !new.exists() {
                    let _ = fs::rename(&old, &new);
                }
            }
            // Verknüpfung neu schreiben → zeigt wieder auf das Logo in der EXE
            let lnk = folder.join(format!("{}.lnk", sys::APP_NAME));
            if lnk.exists() {
                let _ = create_shortcut(&lnk, &exe);
            }
        }
        crate::winffi::refresh_shell_icons();
        let _ = fs::create_dir_all(sys::local_dir());
        let _ = fs::write(marker, sys::VERSION);
    }
}

/// Entscheidet beim Start: Installer zeigen oder direkt den Launcher?
pub fn decide(args: &[String]) -> StartMode {
    let has = |a: &str| args.iter().any(|x| x.eq_ignore_ascii_case(a));
    let exe = current_exe();
    let exe_dir = exe.as_ref().and_then(|e| e.parent().map(|p| p.to_path_buf()));
    if has("--installer") {
        return StartMode::Installer {
            existing: installed_location().map(|p| p.to_string_lossy().to_string()),
        };
    }
    if has("--portable") || crate::uitest::active() {
        return StartMode::Launcher;
    }
    let Some(dir) = exe_dir else { return StartMode::Launcher };
    if dir.join("portable.txt").exists() {
        return StartMode::Launcher;
    }
    if !cfg!(windows) {
        return StartMode::Launcher;
    }
    let existing = installed_location();
    if let Some(loc) = &existing {
        if same_dir(&dir, loc) {
            return StartMode::Launcher;
        }
    }
    if same_dir(&dir, &sys::default_install_dir()) {
        return StartMode::Launcher;
    }
    StartMode::Installer {
        existing: existing.map(|p| p.to_string_lossy().to_string()),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InstallOptions {
    pub dir: String,
    pub desktop: bool,
    pub start_menu: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InstallReport {
    pub exe: String,
    pub dir: String,
    pub warnings: Vec<Problem>,
}

pub fn install(opts: &InstallOptions) -> Result<InstallReport, Problem> {
    let mut dir = PathBuf::from(opts.dir.trim().trim_matches('"'));
    if dir.as_os_str().is_empty() {
        return Err(Problem::install_failed("", "No folder selected."));
    }
    // Eigener Unterordner, wenn ein fremder, nicht leerer Ordner gewählt wurde (z. B. "D:\Spiele"),
    // damit Programmdateien nicht zwischen anderen Dateien landen.
    let own = is_own_dir_name(&dir);
    let has_other_files = fs::read_dir(&dir)
        .map(|rd| rd.flatten().any(|e| !is_own_file_name(&e.file_name().to_string_lossy())))
        .unwrap_or(false);
    if !own && has_other_files {
        dir = dir.join(sys::APP_NAME);
    }
    let dir_s = dir.to_string_lossy().to_string();
    fs::create_dir_all(&dir).map_err(|e| Problem::install_failed(&dir_s, &e.to_string()))?;
    let target = dir.join(sys::EXE_NAME);
    let src = current_exe().ok_or_else(|| Problem::install_failed(&dir_s, "Own EXE not found."))?;
    let same = src
        .canonicalize()
        .ok()
        .zip(target.canonicalize().ok())
        .map(|(a, b)| a == b)
        .unwrap_or(false);
    if !same {
        // Laufende alte Version? Dann erst umbenennen (Windows erlaubt das auch bei laufenden EXEs).
        if target.exists() && fs::remove_file(&target).is_err() {
            let old = dir.join(format!("{}.alt-{}", sys::EXE_NAME, sys::now_secs()));
            let _ = fs::rename(&target, old);
        }
        fs::copy(&src, &target).map_err(|e| Problem::install_failed(&dir_s, &e.to_string()))?;
        // Alte EXE (bis 1.7 „MAZ-Launcher.exe“) aufräumen
        if sys::OLD_EXE_NAME != sys::EXE_NAME {
            let old_exe = dir.join(sys::OLD_EXE_NAME);
            if old_exe.exists() && fs::remove_file(&old_exe).is_err() {
                let _ = fs::rename(&old_exe, dir.join(format!("{}.alt-{}", sys::OLD_EXE_NAME, sys::now_secs())));
            }
        }
    }
    let exe_s = target.to_string_lossy().to_string();
    let mut warnings = Vec::new();

    register_uninstall(&dir_s, &exe_s);

    if opts.desktop {
        let _ = fs::remove_file(sys::desktop_dir().join(format!("{}.lnk", sys::OLD_APP_NAME)));
        let lnk = sys::desktop_dir().join(format!("{}.lnk", sys::APP_NAME));
        if let Err(e) = create_shortcut(&lnk, &target) {
            warnings.push(Problem::shortcut_failed(&exe_s, &e));
        }
    }
    if opts.start_menu {
        let folder = sys::start_menu_dir();
        let _ = fs::create_dir_all(&folder);
        let _ = fs::remove_file(folder.join(format!("{}.lnk", sys::OLD_APP_NAME)));
        let lnk = folder.join(format!("{}.lnk", sys::APP_NAME));
        if let Err(e) = create_shortcut(&lnk, &target) {
            warnings.push(Problem::shortcut_failed(&exe_s, &e));
        }
    }
    Ok(InstallReport { exe: exe_s, dir: dir_s, warnings })
}

fn register_uninstall(dir: &str, exe: &str) {
    #[cfg(windows)]
    {
        use winreg::enums::HKEY_CURRENT_USER;
        let hkcu = winreg::RegKey::predef(HKEY_CURRENT_USER);
        if let Ok((key, _)) = hkcu.create_subkey(UNINSTALL_KEY) {
            let size_kb = fs::metadata(exe).map(|m| (m.len() / 1024) as u32).unwrap_or(0);
            let _ = key.set_value("DisplayName", &sys::APP_NAME.to_string());
            let _ = key.set_value("DisplayVersion", &sys::VERSION.to_string());
            let _ = key.set_value("Publisher", &"MikeAdamRGB".to_string());
            let _ = key.set_value("InstallLocation", &dir.to_string());
            let _ = key.set_value("DisplayIcon", &format!("\"{exe}\",0"));
            let _ = key.set_value("UninstallString", &format!("\"{exe}\" --uninstall"));
            let _ = key.set_value("URLInfoAbout", &sys::GITHUB_URL.to_string());
            let _ = key.set_value("NoModify", &1u32);
            let _ = key.set_value("NoRepair", &1u32);
            let _ = key.set_value("EstimatedSize", &size_kb);
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (dir, exe);
    }
}

/// Erstellt eine Windows-Verknüpfung (.lnk). Fallback: .url-Verknüpfung.
pub fn create_shortcut(lnk: &Path, target: &Path) -> Result<(), String> {
    // SICHERHEIT: Pfade nie in den Befehl einbauen (Anführungszeichen-Tricks), sondern als
    // Umgebungsvariablen übergeben. PowerShell mit absolutem Pfad starten.
    let workdir = target.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
    let script = "$s=(New-Object -ComObject WScript.Shell).CreateShortcut($env:MAZ_LNK);$s.TargetPath=$env:MAZ_TARGET;$s.WorkingDirectory=$env:MAZ_WORKDIR;$s.IconLocation=($env:MAZ_TARGET + ',0');$s.Description=$env:MAZ_DESC;$s.Save()";
    let ps = sys::quiet_command(sys::system_program("powershell.exe"))
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script])
        .env("MAZ_LNK", lnk.as_os_str())
        .env("MAZ_TARGET", target.as_os_str())
        .env("MAZ_WORKDIR", &workdir)
        .env("MAZ_DESC", sys::APP_NAME)
        .status();
    if matches!(ps, Ok(s) if s.success()) && lnk.exists() {
        return Ok(());
    }
    // Fallback ohne PowerShell: Internet-Verknüpfung auf die Datei
    let url_path = lnk.with_extension("url");
    let t = target.to_string_lossy();
    let content = format!(
        "[InternetShortcut]\r\nURL=file:///{}\r\nIconIndex=0\r\nIconFile={}\r\n",
        t.replace('\\', "/"),
        t
    );
    fs::write(&url_path, content).map_err(|e| {
        format!(
            "PowerShell shortcut failed ({}) and fallback shortcut too: {e}",
            match &ps {
                Ok(s) => format!("Exit-Code {:?}", s.code()),
                Err(err) => err.to_string(),
            }
        )
    })
}

/// Deinstallation (über "Apps & Features" oder Einstellungen).
pub fn uninstall_interactive() {
    #[cfg(windows)]
    {
        use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
        let ok = MessageDialog::new()
            .set_level(MessageLevel::Warning)
            .set_title(crate::i18n::t("uninstall.title"))
            .set_description(crate::i18n::t("uninstall.confirm"))
            .set_buttons(MessageButtons::YesNo)
            .show();
        if ok != MessageDialogResult::Yes {
            return;
        }
        let wipe = MessageDialog::new()
            .set_level(MessageLevel::Info)
            .set_title(crate::i18n::t("uninstall.wipe_title"))
            .set_description(crate::i18n::t("uninstall.wipe"))
            .set_buttons(MessageButtons::YesNo)
            .show();

        for name in [sys::APP_NAME, sys::OLD_APP_NAME] {
            for p in [
                sys::desktop_dir().join(format!("{name}.lnk")),
                sys::desktop_dir().join(format!("{name}.url")),
                sys::start_menu_dir().join(format!("{name}.lnk")),
                sys::start_menu_dir().join(format!("{name}.url")),
            ] {
                let _ = fs::remove_file(p);
            }
        }
        let install_dir = installed_location()
            .or_else(|| current_exe().and_then(|e| e.parent().map(|p| p.to_path_buf())));
        {
            use winreg::enums::HKEY_CURRENT_USER;
            let _ = winreg::RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(UNINSTALL_KEY);
        }
        let _ = fs::remove_dir_all(sys::local_dir());
        if wipe == MessageDialogResult::Yes {
            let _ = fs::remove_dir_all(sys::roaming_dir());
        }
        if let Some(dir) = install_dir {
            // SICHERHEIT: Niemals den ganzen Ordner rekursiv löschen – der Launcher kann in
            // "Downloads" oder "D:\Spiele" liegen! Nur die eigenen Dateien entfernen und den
            // Ordner danach nur löschen, wenn er leer ist (rmdir ohne /s schlägt sonst fehl).
            // Die laufende EXE kann sich nicht selbst löschen → kurz warten.
            let d = dir.to_string_lossy().to_string();
            // SICHERHEIT: Pfade mit Zeichen, die cmd.exe auswertet, nie in den Befehl einbauen
            let safe = !d.contains(['"', '%', '^', '!', '\n', '\r']);
            // Den (leeren) Ordner nur entfernen, wenn er wirklich wie der Launcher heißt –
            // nie z. B. einen leeren Downloads- oder Desktop-Ordner.
            let own_dir = is_own_dir_name(&dir);
            let rmdir = if own_dir { format!(" & rmdir \"{d}\"") } else { String::new() };
            let cmd = format!(
                "ping 127.0.0.1 -n 3 > nul & del /f /q \"{d}\\{exe}\" \"{d}\\{exe}.alt-*\" \"{d}\\{old}\" \"{d}\\{old}.alt-*\" \"{d}\\portable.txt\"{rmdir}",
                exe = sys::EXE_NAME,
                old = sys::OLD_EXE_NAME
            );
            use std::os::windows::process::CommandExt;
            if safe {
                let _ = sys::quiet_command(sys::system_program("cmd.exe")).raw_arg(format!("/C {cmd}")).current_dir(sys::system_program("cmd.exe").parent().unwrap_or(Path::new("C:\\"))).spawn();
            }
        }
        MessageDialog::new()
            .set_level(MessageLevel::Info)
            .set_title(crate::i18n::t("uninstall.done_title"))
            .set_description(crate::i18n::t("uninstall.done"))
            .set_buttons(MessageButtons::Ok)
            .show();
    }
}
