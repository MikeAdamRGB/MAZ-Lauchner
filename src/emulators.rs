//! Emulatoren: Profile (Programm + Startparameter) und ROM-Scanner.
//! Jede gefundene ROM wird ein eigenes Spiel in der Bibliothek.

use crate::model::{Art, Emulator, Game, LaunchTarget, Source};
use std::path::{Path, PathBuf};

/// Vorlagen für bekannte Emulatoren: (Name, Startparameter, Dateiendungen)
pub const PRESETS: &[(&str, &str, &[&str])] = &[
    ("RetroArch", "-L \"CORE-PFAD\" \"{rom}\"", &["zip", "7z", "nes", "sfc", "smc", "gb", "gbc", "gba", "md", "n64", "z64", "cue", "chd"]),
    ("Dolphin (GameCube/Wii)", "-b -e \"{rom}\"", &["iso", "gcm", "rvz", "wbfs", "ciso", "gcz", "wad"]),
    ("PCSX2 (PS2)", "-batch -- \"{rom}\"", &["iso", "chd", "cso", "bin"]),
    ("DuckStation (PS1)", "-batch \"{rom}\"", &["cue", "chd", "pbp", "m3u", "iso"]),
    ("PPSSPP (PSP)", "\"{rom}\"", &["iso", "cso", "pbp", "chd"]),
    ("mGBA (Game Boy)", "\"{rom}\"", &["gba", "gb", "gbc"]),
    ("Snes9x (SNES)", "\"{rom}\"", &["sfc", "smc"]),
];

/// Startparameter in einzelne Teile zerlegen (Anführungszeichen beachten) und {rom} einsetzen
pub fn build_args(template: &str, rom: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut has = false;
    for c in template.chars() {
        match c {
            '"' => {
                quoted = !quoted;
                has = true;
            }
            c if c.is_whitespace() && !quoted => {
                if has || !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                    has = false;
                }
            }
            c => cur.push(c),
        }
    }
    if has || !cur.is_empty() {
        out.push(cur);
    }
    let mut replaced = false;
    for a in out.iter_mut() {
        if a.contains("{rom}") {
            *a = a.replace("{rom}", rom);
            replaced = true;
        }
    }
    if !replaced {
        out.push(rom.to_string());
    }
    out
}

/// Schöner Name aus dem Dateinamen: "Super Mario World (USA) [!].sfc" → "Super Mario World"
pub fn clean_name(file: &Path) -> String {
    let stem = file.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let mut out = String::new();
    let mut depth = 0;
    for c in stem.chars() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = (depth - 1).max(0),
            _ if depth == 0 => out.push(if c == '_' { ' ' } else { c }),
            _ => {}
        }
    }
    let name = out.split_whitespace().collect::<Vec<_>>().join(" ");
    if name.is_empty() { stem } else { name }
}

/// ROM-Dateien in einem Ordner (inkl. Unterordner, max. 4 Ebenen)
pub fn find_roms(dir: &Path, exts: &[String]) -> Vec<PathBuf> {
    let exts: Vec<String> = exts.iter().map(|e| e.trim().trim_start_matches('.').to_lowercase()).filter(|e| !e.is_empty()).collect();
    let mut out = Vec::new();
    if exts.is_empty() {
        return out;
    }
    let mut stack = vec![(dir.to_path_buf(), 0)];
    while let Some((d, depth)) = stack.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            let Ok(ft) = e.file_type() else { continue };
            if ft.is_dir() {
                if depth < 4 {
                    stack.push((p, depth + 1));
                }
            } else if p.extension().map(|x| exts.contains(&x.to_string_lossy().to_lowercase())).unwrap_or(false) {
                out.push(p);
            }
            if out.len() > 5000 {
                return out;
            }
        }
    }
    // .bin neben .cue nicht doppelt zählen
    let cues: Vec<String> = out
        .iter()
        .filter(|p| p.extension().map(|x| x.eq_ignore_ascii_case("cue")).unwrap_or(false))
        .map(|p| p.with_extension("").to_string_lossy().to_lowercase())
        .collect();
    out.retain(|p| {
        !(p.extension().map(|x| x.eq_ignore_ascii_case("bin")).unwrap_or(false) && cues.contains(&p.with_extension("").to_string_lossy().to_lowercase()))
    });
    out.sort();
    out
}

/// Alle ROMs aller Emulator-Profile als Spiele
pub fn scan(emulators: &[Emulator]) -> Vec<Game> {
    let mut games = Vec::new();
    for emu in emulators {
        if emu.exe.trim().is_empty() {
            continue;
        }
        for dir in &emu.rom_dirs {
            for rom in find_roms(Path::new(dir), &emu.extensions) {
                let rom_s = rom.to_string_lossy().to_string();
                let id = format!("emu:{}:{:x}", emu.id, crate::sys::fnv1a(&rom_s));
                if games.iter().any(|g: &Game| g.id == id) {
                    continue;
                }
                games.push(Game {
                    id,
                    name: clean_name(&rom),
                    source: Source::Emulator,
                    installed: Path::new(&emu.exe).exists(),
                    install_dir: rom.parent().map(|p| p.to_string_lossy().to_string()),
                    exe: Some(emu.exe.clone()),
                    launch: LaunchTarget::Exe {
                        path: emu.exe.clone(),
                        args: build_args(&emu.args, &rom_s),
                        dir: Path::new(&emu.exe).parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
                    },
                    art: Art::default(),
                    size_bytes: std::fs::metadata(&rom).ok().map(|m| m.len()),
                    playtime_min: None,
                    steam_appid: None,
                });
            }
        }
    }
    games
}

/// Ordner nach installierten Spielen durchsuchen (für "Eigenes Spiel hinzufügen"):
/// jeder Unterordner mit einer passenden Start-Datei wird ein Vorschlag (Name, Datei).
pub fn scan_game_folder(dir: &Path) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = e.path();
        if !p.is_dir() {
            continue;
        }
        let name = e.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name.starts_with('$') {
            continue;
        }
        let exe = if cfg!(windows) {
            crate::icons::guess_main_exe(&p, &name)
        } else {
            crate::stores::guess_exe_pub(&p, &name)
        };
        if let Some(exe) = exe {
            out.push((name, exe.to_string_lossy().to_string()));
        }
    }
    out.sort_by_key(|(n, _)| n.to_lowercase());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_and_names() {
        assert_eq!(build_args("-b -e \"{rom}\"", "C:\\Games\\Mario Kart.iso"), vec!["-b", "-e", "C:\\Games\\Mario Kart.iso"]);
        assert_eq!(build_args("", "x.gba"), vec!["x.gba"]);
        assert_eq!(build_args("-L \"C:\\RA\\cores\\snes9x.dll\" \"{rom}\"", "a.sfc"), vec!["-L", "C:\\RA\\cores\\snes9x.dll", "a.sfc"]);
        assert_eq!(clean_name(Path::new("Super Mario World (USA) [!].sfc")), "Super Mario World");
        assert_eq!(clean_name(Path::new("zelda_minish_cap.gba")), "zelda minish cap");
    }
}
