//! Absturzberichte – nur auf Wunsch. Der Bericht wird lokal erzeugt; ein Klick öffnet ein
//! vorausgefülltes GitHub-Issue im Browser. Dort siehst du alles, bevor du es abschickst.
//! Automatisch wird nie etwas gesendet.

use crate::sys;

/// Datei, in die der Absturz geschrieben wird
pub fn log_path() -> std::path::PathBuf {
    sys::local_dir().join("absturz.log")
}

/// Systeminfos für den Bericht (keine persönlichen Daten)
pub fn system_info() -> String {
    format!("{} {} · {} {} · {}", sys::APP_NAME, sys::VERSION, std::env::consts::OS, std::env::consts::ARCH, crate::updater::InstallKind::detect().label())
}

/// Persönliches aus Pfaden entfernen (Benutzername im Home-Ordner)
pub fn redact(text: &str) -> String {
    let mut out = text.to_string();
    let home = sys::home_dir().to_string_lossy().to_string();
    if home.len() > 3 {
        out = out.replace(&home, "~");
    }
    for var in ["USERNAME", "USER"] {
        if let Ok(u) = std::env::var(var) {
            if u.len() >= 3 {
                out = out.replace(&u, "<user>");
            }
        }
    }
    out
}

/// Vorausgefülltes GitHub-Issue (Titel + Text, gekürzt, damit die Adresse nicht zu lang wird)
pub fn issue_url(title: &str, report: &str) -> String {
    let body = format!("**System:** {}\n\n```\n{}\n```\n\n**Was hast du gerade gemacht?**\n\n", system_info(), redact(report));
    let body: String = body.chars().take(5000).collect();
    format!("{}/issues/new?title={}&body={}", sys::GITHUB_URL, sys::url_encode(title), sys::url_encode(&body))
}

/// Absturz vom letzten Start (wenn noch nicht angesehen)
pub fn pending() -> Option<String> {
    let p = log_path();
    let text = std::fs::read_to_string(&p).ok()?;
    let text: String = text.chars().take(20_000).collect();
    (!text.trim().is_empty()).then_some(text)
}

/// Bericht verwerfen (wurde angesehen)
pub fn dismiss() {
    let p = log_path();
    let _ = std::fs::rename(&p, p.with_extension("log.seen"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_is_encoded_and_short() {
        let u = issue_url("Absturz: test", &"x".repeat(20_000));
        assert!(u.starts_with(&format!("{}/issues/new?title=", sys::GITHUB_URL)));
        assert!(!u.contains(' ') && !u.contains('\n'));
        assert!(u.len() < 16_000);
        let home = sys::home_dir().to_string_lossy().to_string();
        if home.len() > 3 {
            assert!(!redact(&format!("{home}/datei")).contains(&home));
        }
    }
}
