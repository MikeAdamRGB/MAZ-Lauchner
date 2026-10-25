//! Spielzeit messen: Nach dem Start wird geschaut, wie lange ein Programm aus dem
//! Spielordner läuft. Alles bleibt lokal auf dem PC.

use std::time::{Duration, Instant};

/// Woran wir den laufenden Spielprozess erkennen
#[derive(Clone, Debug, PartialEq)]
pub enum Matcher {
    /// Irgendein Programm aus diesem Ordner (Steam, Epic, GOG …)
    Dir(String),
    /// Genau dieses Programm (z. B. Emulator)
    Exe(String),
    /// Programmname enthält diesen Text (z. B. "robloxplayer")
    Name(String),
}

fn norm(p: &str) -> String {
    p.replace('\\', "/").trim_end_matches('/').to_lowercase()
}

impl Matcher {
    pub fn matches(&self, path: &str) -> bool {
        let p = norm(path);
        match self {
            Matcher::Dir(d) => {
                let d = norm(d);
                d.len() > 3 && p.starts_with(&format!("{d}/"))
            }
            Matcher::Exe(e) => p == norm(e),
            Matcher::Name(n) => p.rsplit('/').next().map(|f| f.contains(&n.to_lowercase())).unwrap_or(false),
        }
    }
}

/// Pfade aller laufenden Programme
pub fn process_paths() -> Vec<String> {
    #[cfg(windows)]
    {
        crate::winffi::process_ids()
            .into_iter()
            .filter_map(|(pid, _)| crate::winffi::Proc::open(pid).and_then(|p| p.path()))
            .collect()
    }
    #[cfg(target_os = "linux")]
    {
        let mut out = Vec::new();
        for e in std::fs::read_dir("/proc").into_iter().flatten().flatten() {
            let name = e.file_name();
            if !name.to_string_lossy().chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            if let Ok(exe) = std::fs::read_link(e.path().join("exe")) {
                out.push(exe.to_string_lossy().to_string());
            }
            // Wine/Proton-Spiele: Windows-Pfad steht in der Kommandozeile
            if let Ok(cmd) = std::fs::read(e.path().join("cmdline")) {
                if let Some(first) = cmd.split(|b| *b == 0).next() {
                    let s = String::from_utf8_lossy(first).to_string();
                    if !s.is_empty() {
                        out.push(s);
                    }
                }
            }
        }
        out
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("/bin/ps")
            .args(["-axo", "comm="])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect())
            .unwrap_or_default()
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        Vec::new()
    }
}

fn running(m: &Matcher) -> bool {
    process_paths().iter().any(|p| m.matches(p))
}

/// Läuft blockierend, bis das Spiel beendet ist. Gibt die gespielten Minuten zurück
/// (0 = Spiel nie gefunden, z. B. weil der Store erst ein Update lädt).
pub fn track(m: Matcher) -> u64 {
    let start = Instant::now();
    // Bis zu 5 Minuten warten, bis das Spiel wirklich läuft (Store-Start, Updates)
    let mut found_at = None;
    while start.elapsed() < Duration::from_secs(300) {
        if running(&m) {
            found_at = Some(Instant::now());
            break;
        }
        std::thread::sleep(Duration::from_secs(5));
    }
    let Some(since) = found_at else { return 0 };
    let mut last_seen = Instant::now();
    loop {
        std::thread::sleep(Duration::from_secs(15));
        if running(&m) {
            last_seen = Instant::now();
        } else if last_seen.elapsed() >= Duration::from_secs(40) {
            break;
        }
        // Sicherheitsgrenze: 24 Stunden
        if since.elapsed() > Duration::from_secs(86_400) {
            break;
        }
    }
    let secs = last_seen.duration_since(since).as_secs();
    if secs < 30 {
        0
    } else {
        secs.div_ceil(60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching() {
        let d = Matcher::Dir("C:\\Games\\Witcher 3\\".into());
        assert!(d.matches("c:\\games\\witcher 3\\bin\\x64\\witcher3.exe"));
        assert!(!d.matches("C:\\Games\\Witcher 3 Mods\\tool.exe"));
        assert!(!Matcher::Dir("C:\\".into()).matches("C:\\Windows\\explorer.exe"));
        assert!(Matcher::Exe("/usr/bin/dolphin-emu".into()).matches("/usr/bin/dolphin-emu"));
        assert!(Matcher::Name("RobloxPlayer".into()).matches("C:\\x\\RobloxPlayerBeta.exe"));
    }
}
