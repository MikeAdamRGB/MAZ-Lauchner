//! Roblox: die Roblox-App erkennen und starten, dazu Lieblings-Experiences
//! (einzelne Roblox-Spiele) als eigene Kacheln, die direkt ins Spiel starten.
//!
//! - Windows: Roblox-Player in %LOCALAPPDATA%\Roblox\Versions (oder Bloxstrap)
//! - macOS: /Applications/Roblox.app
//! - Linux: Roblox gibt es offiziell nicht – unterstützt wird "Sober" (Flatpak org.vinegarhq.Sober)
//!
//! Gestartet wird immer über die offizielle Roblox-App bzw. den offiziellen Link
//! `roblox://experiences/start?placeId=…`. Es werden keine Passwörter abgefragt.

use crate::model::{Art, Game, LaunchTarget, RobloxGame, Source, StoreStatus};
use crate::problem::Problem;
use std::path::PathBuf;

pub const APP_ID: &str = "roblox:app";
#[cfg(target_os = "linux")]
pub const SOBER_ID: &str = "org.vinegarhq.Sober";

/// Offizieller Link, der direkt in eine Experience startet
pub fn experience_uri(place_id: u64) -> String {
    format!("roblox://experiences/start?placeId={place_id}")
}

/// Webseite einer Experience
pub fn experience_page(place_id: u64) -> String {
    format!("https://www.roblox.com/games/{place_id}")
}

/// Place-ID aus dem, was der Nutzer eingibt: Link (…/games/920587237/Adopt-Me),
/// "placeId=…" oder nur die Zahl.
pub fn place_id_from(input: &str) -> Option<u64> {
    let s = input.trim();
    let num = |t: &str| -> Option<u64> {
        let digits: String = t.chars().take_while(|c| c.is_ascii_digit()).collect();
        digits.parse::<u64>().ok().filter(|n| *n > 0)
    };
    if let Some(i) = s.find("/games/") {
        return num(&s[i + 7..]);
    }
    if let Some(i) = s.to_ascii_lowercase().find("placeid=") {
        return num(&s[i + 8..]);
    }
    if !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()) {
        return num(s);
    }
    None
}

// ----------------------------------------------------------------------
// Erkennen
// ----------------------------------------------------------------------

/// Neuester Roblox-Player unter Windows
#[cfg(windows)]
fn windows_player() -> Option<PathBuf> {
    let base = crate::sys::env_path("LOCALAPPDATA")?;
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    for root in [base.join("Roblox").join("Versions"), base.join("Bloxstrap").join("Versions")] {
        for e in std::fs::read_dir(&root).into_iter().flatten().flatten() {
            let exe = e.path().join("RobloxPlayerBeta.exe");
            if let Ok(m) = std::fs::metadata(&exe) {
                let t = m.modified().unwrap_or(std::time::UNIX_EPOCH);
                if best.as_ref().map(|(bt, _)| t > *bt).unwrap_or(true) {
                    best = Some((t, exe));
                }
            }
        }
    }
    best.map(|(_, p)| p)
}

/// Roblox aus dem Microsoft Store
#[cfg(windows)]
fn store_app() -> Option<crate::appx::Package> {
    crate::appx::find("ROBLOXCORPORATION.ROBLOX")
}

#[cfg(windows)]
fn bloxstrap() -> Option<PathBuf> {
    let p = crate::sys::env_path("LOCALAPPDATA")?.join("Bloxstrap").join("Bloxstrap.exe");
    p.exists().then_some(p)
}

#[cfg(target_os = "macos")]
fn mac_app() -> Option<PathBuf> {
    let mut c = vec![PathBuf::from("/Applications/Roblox.app")];
    if let Some(h) = crate::sys::env_path("HOME") {
        c.push(h.join("Applications").join("Roblox.app"));
    }
    c.into_iter().find(|p| p.exists())
}

#[cfg(target_os = "linux")]
fn sober_installed() -> bool {
    let mut dirs = vec![PathBuf::from("/var/lib/flatpak/app").join(SOBER_ID)];
    if let Some(h) = crate::sys::env_path("HOME") {
        dirs.push(h.join(".local/share/flatpak/app").join(SOBER_ID));
    }
    dirs.iter().any(|d| d.exists())
}

/// Ist Roblox auf diesem Computer startbereit?
pub fn installed() -> bool {
    #[cfg(windows)]
    {
        windows_player().is_some()
            || bloxstrap().is_some()
            || crate::sys::protocol_registered("roblox-player")
            || store_app().is_some()
    }
    #[cfg(target_os = "macos")]
    {
        mac_app().is_some()
    }
    #[cfg(target_os = "linux")]
    {
        sober_installed()
    }
    #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
    {
        false
    }
}

fn location() -> Option<String> {
    #[cfg(windows)]
    {
        windows_player()
            .and_then(|p| p.parent().and_then(|v| v.parent()).map(|v| v.to_string_lossy().to_string()))
            .or_else(|| bloxstrap().map(|p| p.to_string_lossy().to_string()))
            .or_else(|| store_app().map(|p| format!("Microsoft Store: {}", p.full_name)))
    }
    #[cfg(target_os = "macos")]
    {
        mac_app().map(|p| p.to_string_lossy().to_string())
    }
    #[cfg(target_os = "linux")]
    {
        sober_installed().then(|| format!("Flatpak: {SOBER_ID}"))
    }
    #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

/// Roblox-App + Experiences als Bibliothekseinträge
pub fn scan(list: &[RobloxGame]) -> (Vec<Game>, StoreStatus) {
    let ok = installed();
    let mut status = StoreStatus { found: ok, path: location(), ..Default::default() };
    if !ok {
        status.note = Some(crate::i18n::t("roblox.not_found"));
    }
    let mut games = Vec::new();
    if ok || !list.is_empty() {
        games.push(Game {
            id: APP_ID.into(),
            name: "Roblox".into(),
            source: Source::Roblox,
            installed: ok,
            install_dir: None,
            exe: None,
            launch: LaunchTarget::Roblox(None),
            art: Art::default(),
            size_bytes: None,
            playtime_min: None,
            steam_appid: None,
        });
    }
    for r in list {
        let mut art = Art::default();
        if !r.icon.is_empty() {
            art.cover.push(r.icon.clone());
            art.icon.push(r.icon.clone());
        }
        if !r.thumb.is_empty() {
            art.hero.push(r.thumb.clone());
            art.header.push(r.thumb.clone());
            art.cover.push(r.thumb.clone());
        }
        games.push(Game {
            id: format!("roblox:{}", r.place_id),
            name: if r.name.trim().is_empty() { format!("Roblox {}", r.place_id) } else { r.name.clone() },
            source: Source::Roblox,
            installed: ok,
            install_dir: None,
            exe: None,
            launch: LaunchTarget::Roblox(Some(r.place_id)),
            art,
            size_bytes: None,
            playtime_min: None,
            steam_appid: None,
        });
    }
    status.installed = if ok { games.len() } else { 0 };
    status.owned = games.len();
    (games, status)
}

// ----------------------------------------------------------------------
// Starten
// ----------------------------------------------------------------------

/// Roblox-App oder direkt eine Experience starten
pub fn launch(place: Option<u64>) -> Result<(), Problem> {
    if !installed() {
        return Err(Problem::roblox_missing());
    }
    let fail = |e: String| Problem::unknown("Roblox", &e);

    #[cfg(target_os = "linux")]
    {
        // Sober direkt über Flatpak starten (mit Link, falls eine Experience gewählt ist)
        let mut cmd = crate::sys::host_command("flatpak");
        cmd.arg("run").arg(SOBER_ID);
        if let Some(p) = place {
            cmd.arg(experience_uri(p));
        }
        cmd.spawn().map(|_| ()).map_err(|e| fail(e.to_string()))
    }

    #[cfg(not(target_os = "linux"))]
    {
        if let Some(p) = place {
            return crate::sys::open_url(&experience_uri(p)).map_err(|c| fail(format!("open roblox:// failed ({c})")));
        }
        #[cfg(windows)]
        {
            // Die Roblox-App öffnen (nicht die Webseite!):
            // 1. offizieller Link "roblox-player:1+launchmode:app" (normaler Installer und Bloxstrap)
            if crate::sys::protocol_registered("roblox-player") && crate::sys::open_url("roblox-player:1+launchmode:app").is_ok() {
                return Ok(());
            }
            // 2. Roblox aus dem Microsoft Store
            if let Some(p) = store_app() {
                let app = p.app_ids.first().cloned().unwrap_or_else(|| "App".into());
                return crate::appx::launch(&p.family, &app).map_err(fail);
            }
            // 3. Player direkt im App-Modus starten
            if let Some(exe) = windows_player() {
                return std::process::Command::new(&exe)
                    .arg("--app")
                    .current_dir(exe.parent().unwrap_or(&exe))
                    .spawn()
                    .map(|_| ())
                    .map_err(|e| fail(e.to_string()));
            }
            if let Some(b) = bloxstrap() {
                return std::process::Command::new(b).spawn().map(|_| ()).map_err(|e| fail(e.to_string()));
            }
            Err(Problem::roblox_missing())
        }
        #[cfg(target_os = "macos")]
        {
            let app = mac_app().ok_or_else(Problem::roblox_missing)?;
            std::process::Command::new("/usr/bin/open")
                .arg(app)
                .spawn()
                .map(|_| ())
                .map_err(|e| fail(e.to_string()))
        }
        #[cfg(not(any(windows, target_os = "macos")))]
        {
            Err(Problem::roblox_missing())
        }
    }
}

// ----------------------------------------------------------------------
// Name und Bilder einer Experience (öffentliche Roblox-Schnittstellen, ohne Anmeldung)
// ----------------------------------------------------------------------

fn get_json(agent: &ureq::Agent, url: &str) -> Option<serde_json::Value> {
    agent.get(url).call().ok()?.body_mut().read_json().ok()
}

/// Lädt Name, Icon und Vorschaubild. Fehlt etwas, bleibt das Feld leer.
pub fn fetch_info(place_id: u64) -> RobloxGame {
    let mut g = RobloxGame { place_id, added: crate::sys::now_secs(), ..Default::default() };
    let agent = crate::steam::http_agent();
    let Some(universe) = get_json(&agent, &format!("https://apis.roblox.com/universes/v1/places/{place_id}/universe"))
        .and_then(|v| v.get("universeId").and_then(|u| u.as_u64()))
    else {
        return g;
    };
    if let Some(v) = get_json(&agent, &format!("https://games.roblox.com/v1/games?universeIds={universe}")) {
        if let Some(name) = v.pointer("/data/0/name").and_then(|n| n.as_str()) {
            g.name = name.chars().take(80).collect();
        }
    }
    if let Some(v) = get_json(
        &agent,
        &format!("https://thumbnails.roblox.com/v1/games/icons?universeIds={universe}&size=512x512&format=Png&isCircular=false"),
    ) {
        if let Some(u) = v.pointer("/data/0/imageUrl").and_then(|n| n.as_str()).filter(|u| u.starts_with("https://")) {
            g.icon = u.to_string();
        }
    }
    if let Some(v) = get_json(
        &agent,
        &format!("https://thumbnails.roblox.com/v1/games/multiget/thumbnails?universeIds={universe}&countPerUniverse=1&size=768x432&format=Png"),
    ) {
        if let Some(u) = v.pointer("/data/0/thumbnails/0/imageUrl").and_then(|n| n.as_str()).filter(|u| u.starts_with("https://")) {
            g.thumb = u.to_string();
        }
    }
    g
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn place_ids() {
        assert_eq!(place_id_from("https://www.roblox.com/games/920587237/Adopt-Me"), Some(920587237));
        assert_eq!(place_id_from("roblox://experiences/start?placeId=606849621"), Some(606849621));
        assert_eq!(place_id_from(" 2753915549 "), Some(2753915549));
        assert_eq!(place_id_from("hallo"), None);
        assert_eq!(place_id_from("0"), None);
        assert_eq!(experience_uri(1), "roblox://experiences/start?placeId=1");
    }
}
