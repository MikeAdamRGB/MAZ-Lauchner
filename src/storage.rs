//! Speicherplatz: Wie viel Platz belegen die installierten Spiele – pro Laufwerk?
//!
//! Nur lesen. Ordnergrößen werden ohne Symlinks gezählt (sonst würde z. B. ein Link auf das
//! ganze Laufwerk mitgezählt) und mit einer Obergrenze an Dateien, damit nichts hängt.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

const MAX_FILES: usize = 3_000_000;

/// Größe eines Ordners in Bytes (ohne Symlinks, mit Grenze)
pub fn dir_size(dir: &Path) -> u64 {
    let mut total = 0u64;
    let mut files = 0usize;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let Ok(m) = std::fs::symlink_metadata(e.path()) else { continue };
            if m.file_type().is_symlink() {
                continue;
            }
            if m.is_dir() {
                stack.push(e.path());
            } else {
                total = total.saturating_add(m.len());
                files += 1;
                if files >= MAX_FILES {
                    return total;
                }
            }
        }
    }
    total
}

fn size_cache() -> &'static Mutex<HashMap<PathBuf, (u64, u64)>> {
    static C: OnceLock<Mutex<HashMap<PathBuf, (u64, u64)>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Ordnergröße mit Zwischenspeicher (10 Minuten)
pub fn dir_size_cached(dir: &Path) -> u64 {
    let now = crate::sys::now_secs();
    if let Ok(c) = size_cache().lock() {
        if let Some((t, s)) = c.get(dir) {
            if now.saturating_sub(*t) < 600 {
                return *s;
            }
        }
    }
    let s = dir_size(dir);
    if let Ok(mut c) = size_cache().lock() {
        c.insert(dir.to_path_buf(), (now, s));
    }
    s
}

pub fn forget_cache() {
    if let Ok(mut c) = size_cache().lock() {
        c.clear();
    }
}

/// Linux: Einhängepunkte aus /proc/self/mounts (Leerzeichen sind dort als \040 kodiert)
#[cfg(target_os = "linux")]
fn mount_points() -> Vec<String> {
    let text = std::fs::read_to_string("/proc/self/mounts").unwrap_or_default();
    parse_mounts(&text)
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn parse_mounts(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|l| l.split_whitespace().nth(1))
        .map(|m| m.replace("\\040", " ").replace("\\011", "\t").replace("\\134", "\\"))
        .filter(|m| m.starts_with('/'))
        .collect()
}

/// Längster passender Einhängepunkt
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn best_mount<'a>(path: &str, mounts: &'a [String]) -> Option<&'a str> {
    mounts
        .iter()
        .filter(|m| m.as_str() == "/" || path == m.as_str() || path.starts_with(&format!("{}/", m.trim_end_matches('/'))))
        .max_by_key(|m| m.len())
        .map(|m| m.as_str())
}

/// Laufwerk (Wurzel) eines Pfads: Windows „C:\“, sonst der Einhängepunkt
pub fn drive_of(path: &str) -> String {
    #[cfg(windows)]
    {
        let b = path.as_bytes();
        if b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':' {
            return format!("{}:\\", (b[0] as char).to_ascii_uppercase());
        }
        "?".into()
    }
    #[cfg(target_os = "linux")]
    {
        let mounts = mount_points();
        best_mount(path, &mounts).unwrap_or("/").to_string()
    }
    #[cfg(target_os = "macos")]
    {
        mac_mount(path).unwrap_or_else(|| "/".into())
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        let _ = path;
        "/".into()
    }
}

#[cfg(target_os = "macos")]
fn mac_mount(path: &str) -> Option<String> {
    use std::ffi::{CStr, CString};
    let c = CString::new(path).ok()?;
    let mut st: libc::statfs = unsafe { std::mem::zeroed() };
    // SAFETY: c ist nullterminiert, st ist eine gültige, beschreibbare Struktur.
    if unsafe { libc::statfs(c.as_ptr(), &mut st) } != 0 {
        return None;
    }
    // SAFETY: f_mntonname ist ein nullterminiertes Feld fester Länge.
    let name = unsafe { CStr::from_ptr(st.f_mntonname.as_ptr()) };
    Some(name.to_string_lossy().to_string())
}

/// Freier und gesamter Platz (Bytes)
pub fn disk_space(root: &str) -> Option<(u64, u64)> {
    #[cfg(windows)]
    {
        crate::winffi::disk_space(root)
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let c = std::ffi::CString::new(root).ok()?;
        let mut st: libc::statvfs = unsafe { std::mem::zeroed() };
        // SAFETY: c ist nullterminiert, st ist eine gültige, beschreibbare Struktur.
        if unsafe { libc::statvfs(c.as_ptr(), &mut st) } != 0 {
            return None;
        }
        let frsize = st.f_frsize as u64;
        let total = (st.f_blocks as u64).saturating_mul(frsize);
        (total > 0).then_some(((st.f_bavail as u64).saturating_mul(frsize), total))
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        let _ = root;
        None
    }
}

/// Ein Spiel in der Übersicht
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub bytes: u64,
}

/// Ein Laufwerk mit seinen Spielen
#[derive(Clone, Debug, PartialEq)]
pub struct Drive {
    pub root: String,
    pub free: Option<u64>,
    pub total: Option<u64>,
    pub games: Vec<Entry>,
}

impl Drive {
    pub fn used_by_games(&self) -> u64 {
        self.games.iter().map(|g| g.bytes).sum()
    }
}

/// Übersicht berechnen (langsam bei großen Ordnern → im Hintergrund aufrufen)
pub fn overview(games: &[crate::model::Game]) -> Vec<Drive> {
    let mut map: Vec<Drive> = Vec::new();
    for g in games.iter().filter(|g| g.installed) {
        let Some(dir) = g.install_dir.as_deref().map(str::trim).filter(|d| !d.is_empty()) else { continue };
        let p = Path::new(dir);
        if !p.is_absolute() || !p.is_dir() {
            continue;
        }
        let bytes = match g.size_bytes {
            Some(b) if b > 0 => b,
            _ => dir_size_cached(p),
        };
        let root = drive_of(dir);
        let entry = Entry { id: g.id.clone(), name: g.name.clone(), bytes };
        match map.iter_mut().find(|d| d.root == root) {
            Some(d) => d.games.push(entry),
            None => {
                let space = disk_space(&root);
                map.push(Drive { root, free: space.map(|s| s.0), total: space.map(|s| s.1), games: vec![entry] });
            }
        }
    }
    for d in &mut map {
        d.games.sort_by_key(|g| std::cmp::Reverse(g.bytes));
    }
    map.sort_by_key(|d| std::cmp::Reverse(d.used_by_games()));
    map
}

/// Lesbare Größe (1 KB = 1024 Bytes)
pub fn fmt_bytes(b: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = b as f64;
    let mut u = 0;
    while v >= 1024.0 && u < UNITS.len() - 1 {
        v /= 1024.0;
        u += 1;
    }
    if u <= 1 {
        format!("{} {}", v.round() as u64, UNITS[u])
    } else {
        format!("{v:.1} {}", UNITS[u])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_and_mounts() {
        let root = std::env::temp_dir().join(format!("og-storage-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("a/b")).unwrap();
        std::fs::write(root.join("a/x.bin"), vec![0u8; 1000]).unwrap();
        std::fs::write(root.join("a/b/y.bin"), vec![0u8; 24]).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink("/", root.join("a/rootlink")).unwrap();
        assert_eq!(dir_size(&root.join("a")), 1024);
        let _ = std::fs::remove_dir_all(&root);

        let m = parse_mounts("/dev/sda1 / ext4 rw 0 0\n/dev/sdb1 /mnt/Games\\040SSD ext4 rw 0 0\n/dev/sdc1 /mnt/Games ext4 rw 0 0\nproc /proc proc rw 0 0");
        assert_eq!(best_mount("/mnt/Games SSD/steam", &m), Some("/mnt/Games SSD"));
        assert_eq!(best_mount("/mnt/Games/x", &m), Some("/mnt/Games"));
        assert_eq!(best_mount("/mnt/GamesX/x", &m), Some("/"));
        assert_eq!(fmt_bytes(1023), "1023 B");
        assert_eq!(fmt_bytes(1536 * 1024 * 1024), "1.5 GB");
    }
}
