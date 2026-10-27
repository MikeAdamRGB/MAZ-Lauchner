//! Discord-Status („Spielt gerade …“) – optional und abschaltbar.
//!
//! Redet nur lokal mit der laufenden Discord-App (Windows: Named Pipe, sonst Unix-Socket).
//! Ohne laufendes Discord passiert einfach nichts. Es geht nur der Spielname und die Startzeit
//! an Discord – keine anderen Daten.

use std::io::{Read, Write};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Mutex, OnceLock};

enum Msg {
    Playing { app_id: String, game: String, start: u64 },
    Clear,
}

fn sender() -> &'static Mutex<Sender<Msg>> {
    static TX: OnceLock<Mutex<Sender<Msg>>> = OnceLock::new();
    TX.get_or_init(|| {
        let (tx, rx) = channel::<Msg>();
        let _ = std::thread::Builder::new().name("discord".into()).spawn(move || worker(rx));
        Mutex::new(tx)
    })
}

/// Gültige Discord-Anwendungs-ID (nur Ziffern)
pub fn valid_app_id(id: &str) -> bool {
    (15..=21).contains(&id.len()) && id.chars().all(|c| c.is_ascii_digit())
}

/// „Spielt gerade <Spiel>“ anzeigen
pub fn set_playing(app_id: &str, game: &str) {
    if !valid_app_id(app_id) {
        return;
    }
    let game: String = game.chars().filter(|c| !c.is_control()).take(120).collect();
    if let Ok(tx) = sender().lock() {
        let _ = tx.send(Msg::Playing { app_id: app_id.to_string(), game, start: crate::sys::now_secs() });
    }
}

pub fn clear() {
    if let Ok(tx) = sender().lock() {
        let _ = tx.send(Msg::Clear);
    }
}

trait Pipe: Read + Write + Send {}
impl<T: Read + Write + Send> Pipe for T {}

fn connect() -> Option<Box<dyn Pipe>> {
    #[cfg(windows)]
    {
        for i in 0..10 {
            if let Ok(f) = std::fs::OpenOptions::new().read(true).write(true).open(format!(r"\\.\pipe\discord-ipc-{i}")) {
                return Some(Box::new(f));
            }
        }
        None
    }
    #[cfg(unix)]
    {
        let mut bases: Vec<std::path::PathBuf> = Vec::new();
        for var in ["XDG_RUNTIME_DIR", "TMPDIR", "TMP", "TEMP"] {
            if let Some(p) = crate::sys::env_path(var) {
                bases.push(p);
            }
        }
        bases.push("/tmp".into());
        let subs = ["", "app/com.discordapp.Discord", "app/com.discordapp.DiscordCanary", "snap.discord", ".flatpak/dev.vencord.Vesktop/xdg-run"];
        for b in &bases {
            for s in subs {
                for i in 0..10 {
                    let p = b.join(s).join(format!("discord-ipc-{i}"));
                    // SICHERHEIT: Nur Sockets des eigenen Benutzers (in /tmp könnte ein anderer
                    // Benutzer einen falschen „Discord“ anbieten und Spielnamen mitlesen)
                    use std::os::unix::fs::MetadataExt;
                    // SAFETY: getuid hat keine Vorbedingungen.
                    let me = unsafe { libc::getuid() };
                    if std::fs::symlink_metadata(&p).map(|m| m.uid() != me).unwrap_or(true) {
                        continue;
                    }
                    if let Ok(sock) = std::os::unix::net::UnixStream::connect(&p) {
                        let _ = sock.set_read_timeout(Some(std::time::Duration::from_secs(3)));
                        let _ = sock.set_write_timeout(Some(std::time::Duration::from_secs(3)));
                        return Some(Box::new(sock));
                    }
                }
            }
        }
        None
    }
    #[cfg(not(any(windows, unix)))]
    {
        None
    }
}

/// Ein Paket: Art (u32), Länge (u32), JSON
pub fn frame(op: u32, json: &serde_json::Value) -> Vec<u8> {
    let body = json.to_string();
    let mut out = Vec::with_capacity(body.len() + 8);
    out.extend_from_slice(&op.to_le_bytes());
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(body.as_bytes());
    out
}

fn read_frame(p: &mut dyn Pipe) -> Option<(u32, Vec<u8>)> {
    let mut head = [0u8; 8];
    p.read_exact(&mut head).ok()?;
    let op = u32::from_le_bytes([head[0], head[1], head[2], head[3]]);
    let len = u32::from_le_bytes([head[4], head[5], head[6], head[7]]) as usize;
    if len > 64 * 1024 {
        return None;
    }
    let mut body = vec![0u8; len];
    p.read_exact(&mut body).ok()?;
    Some((op, body))
}

/// Aktivität für SET_ACTIVITY
pub fn activity(game: &str, start: u64, nonce: u64) -> serde_json::Value {
    serde_json::json!({
        "cmd": "SET_ACTIVITY",
        "args": {
            "pid": std::process::id(),
            "activity": {
                "details": game,
                "timestamps": { "start": start },
                "instance": false
            }
        },
        "nonce": nonce.to_string()
    })
}

fn worker(rx: Receiver<Msg>) {
    let mut conn: Option<(String, Box<dyn Pipe>)> = None;
    let mut nonce = 0u64;
    while let Ok(msg) = rx.recv() {
        nonce += 1;
        match msg {
            Msg::Playing { app_id, game, start } => {
                // Neue Verbindung, wenn noch keine da ist oder eine andere Anwendungs-ID gewählt wurde
                if conn.as_ref().map(|(id, _)| *id != app_id).unwrap_or(true) {
                    conn = None;
                    if let Some(mut p) = connect() {
                        let hello = frame(0, &serde_json::json!({ "v": 1, "client_id": app_id }));
                        if p.write_all(&hello).is_ok() && read_frame(p.as_mut()).is_some() {
                            conn = Some((app_id.clone(), p));
                        }
                    }
                }
                if let Some((_, p)) = conn.as_mut() {
                    let ok = p.write_all(&frame(1, &activity(&game, start, nonce))).is_ok() && read_frame(p.as_mut()).is_some();
                    if !ok {
                        conn = None;
                    }
                }
            }
            Msg::Clear => {
                if let Some((_, p)) = conn.as_mut() {
                    let clear = serde_json::json!({ "cmd": "SET_ACTIVITY", "args": { "pid": std::process::id() }, "nonce": nonce.to_string() });
                    if p.write_all(&frame(1, &clear)).is_err() || read_frame(p.as_mut()).is_none() {
                        conn = None;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames() {
        let f = frame(1, &serde_json::json!({"a":1}));
        assert_eq!(&f[0..4], &1u32.to_le_bytes());
        assert_eq!(u32::from_le_bytes([f[4], f[5], f[6], f[7]]) as usize, f.len() - 8);
        let a = activity("Portal 2", 100, 7);
        assert_eq!(a["args"]["activity"]["details"], "Portal 2");
        assert_eq!(a["nonce"], "7");
        assert!(valid_app_id("1234567890123456789"));
        assert!(!valid_app_id("12ab"));
        // Ohne Discord: nichts passiert, kein Absturz
        set_playing("1234567890123456789", "Test");
        clear();
    }
}
