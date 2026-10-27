//! Screenshots eines Spiels: aus Steam (F12-Screenshots) und aus einem eigenen Ordner je Spiel.
//! Nur lesen – es wird nichts verändert oder hochgeladen.

use std::path::{Path, PathBuf};

const MAX_SHOTS: usize = 300;
const IMAGE_EXT: [&str; 6] = ["png", "jpg", "jpeg", "webp", "bmp", "gif"];

#[derive(Clone, Debug, PartialEq)]
pub struct Shot {
    pub path: PathBuf,
    pub modified: u64,
    pub from_steam: bool,
}

fn is_image(p: &Path) -> bool {
    p.extension().and_then(|e| e.to_str()).map(|e| IMAGE_EXT.contains(&e.to_ascii_lowercase().as_str())).unwrap_or(false)
}

/// Bilder direkt in einem Ordner (nicht rekursiv, keine Symlinks)
pub fn images_in(dir: &Path, from_steam: bool) -> Vec<Shot> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).into_iter().flatten().flatten().take(5000) {
        let Ok(meta) = std::fs::symlink_metadata(e.path()) else { continue };
        if !meta.is_file() || !is_image(&e.path()) {
            continue;
        }
        let modified = meta.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
        out.push(Shot { path: e.path(), modified, from_steam });
    }
    out
}

/// Steam: `<Steam>/userdata/<Konto>/760/remote/<AppID>/screenshots`
pub fn steam_dirs(steam_root: &Path, appid: u32) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for acc in std::fs::read_dir(steam_root.join("userdata")).into_iter().flatten().flatten() {
        let name = acc.file_name().to_string_lossy().to_string();
        if !name.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let d = acc.path().join("760").join("remote").join(appid.to_string()).join("screenshots");
        if d.is_dir() {
            dirs.push(d);
        }
    }
    dirs
}

/// Alle Screenshots, neueste zuerst
pub fn collect(steam_appid: Option<u32>, own_dir: Option<&str>) -> Vec<Shot> {
    let mut all = Vec::new();
    if let (Some(appid), Some(root)) = (steam_appid, crate::steam::find_steam_root()) {
        for d in steam_dirs(&root, appid) {
            all.extend(images_in(&d, true));
        }
    }
    if let Some(dir) = own_dir.map(str::trim).filter(|d| !d.is_empty()) {
        let p = Path::new(dir);
        if p.is_absolute() && p.is_dir() {
            all.extend(images_in(p, false));
        }
    }
    all.sort_by_key(|s| std::cmp::Reverse(s.modified));
    all.dedup_by(|a, b| a.path == b.path);
    all.truncate(MAX_SHOTS);
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_steam_and_own() {
        let root = std::env::temp_dir().join(format!("og-shots-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let sd = root.join("userdata/123/760/remote/440/screenshots");
        std::fs::create_dir_all(sd.join("thumbnails")).unwrap();
        std::fs::write(sd.join("a.jpg"), "x").unwrap();
        std::fs::write(sd.join("notes.txt"), "x").unwrap();
        std::fs::create_dir_all(root.join("userdata/abc/760/remote/440/screenshots")).unwrap();
        assert_eq!(steam_dirs(&root, 440), vec![sd.clone()]);
        let imgs = images_in(&sd, true);
        assert_eq!(imgs.len(), 1);
        assert!(imgs[0].from_steam);
        let own = root.join("own");
        std::fs::create_dir_all(&own).unwrap();
        std::fs::write(own.join("b.PNG"), "x").unwrap();
        let c = collect(None, Some(own.to_str().unwrap()));
        assert_eq!(c.len(), 1);
        assert!(!c[0].from_steam);
        assert!(collect(None, Some("relativ/pfad")).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }
}
