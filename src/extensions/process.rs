//! Erweiterungs-Prozess starten und mit ihm über JSON-Zeilen (stdin/stdout) reden.
//!
//! - Ein Prozess pro Erweiterung, wird wiederverwendet.
//! - Antworten liest ein eigener Thread; der Aufrufer wartet höchstens 10 Sekunden.
//! - Antwort größer als 1 MB, kaputtes JSON oder Zeitüberschreitung → Prozess wird beendet.
//! - Minimale Umgebung, Arbeitsordner = Erweiterungsordner.
//! - Windows: Prozess hängt an einem Job-Objekt → stirbt garantiert mit dem Launcher.
//!   Linux: PR_SET_PDEATHSIG. macOS: stdin wird beim Ende geschlossen (Protokoll-Regel:
//!   bei EOF auf stdin sofort beenden).

use serde_json::Value;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::time::Duration;

pub const TIMEOUT: Duration = Duration::from_secs(10);
pub const MAX_LINE: usize = 1024 * 1024;

#[derive(Clone, Debug, PartialEq)]
pub enum CallError {
    /// Prozess ließ sich nicht starten
    Spawn(String),
    Timeout,
    TooBig,
    BadJson,
    /// Prozess hat sich beendet
    Crashed,
    /// Fehlerantwort der Erweiterung (code, message)
    Remote(String, String),
}

impl CallError {
    pub fn explain(&self) -> String {
        use crate::i18n::{t, tf};
        match self {
            CallError::Spawn(e) => tf("ext.call.spawn", &[("error", e)]),
            CallError::Timeout => t("ext.call.timeout"),
            CallError::TooBig => t("ext.call.too_big"),
            CallError::BadJson => t("ext.call.bad_json"),
            CallError::Crashed => t("ext.call.crashed"),
            CallError::Remote(c, m) => tf("ext.call.remote", &[("code", c), ("message", m)]),
        }
    }
}

/// Eine Zeile lesen, höchstens `max` Bytes (sonst Fehler statt unbegrenzt Speicher)
fn read_line_limited<R: BufRead>(r: &mut R, max: usize) -> Result<Option<Vec<u8>>, CallError> {
    let mut out = Vec::new();
    loop {
        let buf = match r.fill_buf() {
            Ok(b) => b,
            Err(_) => return Err(CallError::Crashed),
        };
        if buf.is_empty() {
            return if out.is_empty() { Ok(None) } else { Ok(Some(out)) };
        }
        if let Some(i) = buf.iter().position(|&b| b == b'\n') {
            out.extend_from_slice(&buf[..i]);
            r.consume(i + 1);
            if out.len() > max {
                return Err(CallError::TooBig);
            }
            return Ok(Some(out));
        }
        let n = buf.len();
        out.extend_from_slice(buf);
        r.consume(n);
        if out.len() > max {
            return Err(CallError::TooBig);
        }
    }
}

enum Line {
    Data(Vec<u8>),
    TooBig,
    Eof,
}

pub struct ExtProcess {
    child: Child,
    stdin: Option<ChildStdin>,
    lines: Receiver<Line>,
    next_id: u64,
    #[cfg(windows)]
    _job: Option<crate::winffi::Job>,
}

/// Umgebung auf das Nötigste beschränken
fn minimal_env(cmd: &mut Command, ext_dir: &Path, network: bool) {
    cmd.env_clear();
    let keep: &[&str] = if cfg!(windows) {
        &["SystemRoot", "windir", "SystemDrive", "PATH", "PATHEXT", "ComSpec", "TEMP", "TMP", "NUMBER_OF_PROCESSORS", "PROCESSOR_ARCHITECTURE"]
    } else {
        &["PATH", "HOME", "LANG", "LC_ALL", "TMPDIR", "XDG_RUNTIME_DIR"]
    };
    for k in keep {
        if let Some(v) = std::env::var_os(k) {
            cmd.env(k, v);
        }
    }
    cmd.env("MAZ_EXTENSION_DIR", ext_dir);
    cmd.env("MAZ_API_VERSION", super::manifest::API_VERSION.to_string());
    cmd.env("MAZ_NETWORK_ALLOWED", if network { "1" } else { "0" });
}

impl ExtProcess {
    pub fn spawn(program: &Path, ext_dir: &Path, network: bool) -> Result<ExtProcess, CallError> {
        let mut cmd = Command::new(program);
        cmd.current_dir(ext_dir).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null());
        minimal_env(&mut cmd, ext_dir, network);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x0800_0000); // kein Konsolenfenster
        }
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::process::CommandExt;
            // SAFETY: prctl ist async-signal-safe; setzt nur ein Prozess-Attribut im Kind.
            unsafe {
                cmd.pre_exec(|| {
                    libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL);
                    Ok(())
                });
            }
        }
        let mut child = cmd.spawn().map_err(|e| CallError::Spawn(e.to_string()))?;
        #[cfg(windows)]
        let job = crate::winffi::Job::kill_on_close().and_then(|j| j.assign(&child).then_some(j));
        let stdout = child.stdout.take().ok_or(CallError::Crashed)?;
        let stdin = child.stdin.take();
        let (tx, rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("maz-ext-reader".into())
            .spawn(move || {
                let mut r = BufReader::new(stdout.take(u64::MAX));
                loop {
                    match read_line_limited(&mut r, MAX_LINE) {
                        Ok(Some(l)) => {
                            if tx.send(Line::Data(l)).is_err() {
                                break;
                            }
                        }
                        Ok(None) | Err(CallError::Crashed) => {
                            let _ = tx.send(Line::Eof);
                            break;
                        }
                        Err(_) => {
                            let _ = tx.send(Line::TooBig);
                            break;
                        }
                    }
                }
            })
            .map_err(|e| CallError::Spawn(e.to_string()))?;
        Ok(ExtProcess {
            child,
            stdin,
            lines: rx,
            next_id: 1,
            #[cfg(windows)]
            _job: job,
        })
    }

    /// Eine Anfrage senden und auf die passende Antwort warten
    pub fn call(&mut self, call: &str, params: Value, timeout: Duration) -> Result<Value, CallError> {
        let id = self.next_id;
        self.next_id += 1;
        let mut req = serde_json::json!({ "id": id, "call": call });
        if let (Some(obj), Value::Object(p)) = (req.as_object_mut(), params) {
            for (k, v) in p {
                if k != "id" && k != "call" {
                    obj.insert(k, v);
                }
            }
        }
        let line = format!("{req}\n");
        let stdin = self.stdin.as_mut().ok_or(CallError::Crashed)?;
        stdin.write_all(line.as_bytes()).map_err(|_| CallError::Crashed)?;
        stdin.flush().map_err(|_| CallError::Crashed)?;
        let deadline = std::time::Instant::now() + timeout;
        loop {
            let left = deadline.saturating_duration_since(std::time::Instant::now());
            match self.lines.recv_timeout(left) {
                Ok(Line::Data(bytes)) => {
                    let v: Value = serde_json::from_slice(&bytes).map_err(|_| CallError::BadJson)?;
                    // Antworten auf ältere (abgelaufene) Anfragen überspringen
                    if v.get("id").and_then(|x| x.as_u64()) != Some(id) {
                        continue;
                    }
                    if let Some(e) = v.get("error") {
                        let s = |k: &str| e.get(k).and_then(|x| x.as_str()).unwrap_or("").chars().take(200).collect::<String>();
                        return Err(CallError::Remote(s("code"), s("message")));
                    }
                    return Ok(v.get("result").cloned().unwrap_or(Value::Null));
                }
                Ok(Line::TooBig) => return Err(CallError::TooBig),
                Ok(Line::Eof) | Err(RecvTimeoutError::Disconnected) => return Err(CallError::Crashed),
                Err(RecvTimeoutError::Timeout) => return Err(CallError::Timeout),
            }
        }
    }

    pub fn alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Sauber beenden: stdin schließen, kurz warten, dann hart beenden
    pub fn stop(&mut self) {
        self.stdin = None;
        for _ in 0..10 {
            if !self.alive() {
                return;
            }
            std::thread::sleep(Duration::from_millis(30));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for ExtProcess {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Programm einer Erweiterung (für Tests auch direkt nutzbar)
pub fn program_path(dir: &Path, run: &str) -> Option<PathBuf> {
    super::manifest::resolve_inside(dir, run)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limited_lines() {
        let mut r = std::io::Cursor::new(b"abc\ndef".to_vec());
        assert_eq!(read_line_limited(&mut r, 10).unwrap(), Some(b"abc".to_vec()));
        assert_eq!(read_line_limited(&mut r, 10).unwrap(), Some(b"def".to_vec()));
        assert_eq!(read_line_limited(&mut r, 10).unwrap(), None);
        let mut big = std::io::Cursor::new(vec![b'x'; 100]);
        assert_eq!(read_line_limited(&mut big, 10), Err(CallError::TooBig));
    }

    /// Protokoll mit kleinen Shell-Skripten testen (Zeitüberschreitung, kaputtes JSON, Absturz)
    #[cfg(unix)]
    #[test]
    fn protocol_failures() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("maz-proc-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let script = |name: &str, body: &str| {
            let p = dir.join(name);
            std::fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
            std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
            p
        };
        let ok = script("ok", r#"while read l; do echo '{"id":1,"result":{"name":"x"}}'; done"#);
        let mut p = ExtProcess::spawn(&ok, &dir, false).unwrap();
        assert_eq!(p.call("hello", serde_json::json!({}), TIMEOUT).unwrap()["name"], "x");

        let slow = script("slow", "read l; sleep 5");
        let mut p = ExtProcess::spawn(&slow, &dir, false).unwrap();
        assert_eq!(p.call("hello", serde_json::json!({}), Duration::from_millis(300)), Err(CallError::Timeout));

        let bad = script("bad", "read l; echo 'kein json'; sleep 1");
        let mut p = ExtProcess::spawn(&bad, &dir, false).unwrap();
        assert_eq!(p.call("hello", serde_json::json!({}), TIMEOUT), Err(CallError::BadJson));

        let crash = script("crash", "exit 3");
        let mut p = ExtProcess::spawn(&crash, &dir, false).unwrap();
        assert_eq!(p.call("hello", serde_json::json!({}), TIMEOUT), Err(CallError::Crashed));

        let err = script("err", r#"read l; echo '{"id":1,"error":{"code":"nope","message":"geht nicht"}}'; sleep 1"#);
        let mut p = ExtProcess::spawn(&err, &dir, false).unwrap();
        assert_eq!(p.call("hello", serde_json::json!({}), TIMEOUT), Err(CallError::Remote("nope".into(), "geht nicht".into())));

        let huge = script("huge", "read l; head -c 2000000 /dev/zero | tr '\\0' 'a'; echo");
        let mut p = ExtProcess::spawn(&huge, &dir, false).unwrap();
        assert_eq!(p.call("hello", serde_json::json!({}), TIMEOUT), Err(CallError::TooBig));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
