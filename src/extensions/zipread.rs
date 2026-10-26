//! Kleiner ZIP-Leser für .mazext-Pakete (nur "stored" und "deflate"), ohne neue Abhängigkeit.
//! SICHERHEIT: Schutz gegen Zip-Slip (Pfade mit "..", absolute Pfade, Laufwerke), gegen
//! Zip-Bomben (Größen- und Anzahl-Grenzen) und gegen Symlinks.

use std::io::Read;
use std::path::{Path, PathBuf};

const MAX_FILES: usize = 2000;
const MAX_TOTAL: u64 = 200 * 1024 * 1024;
const MAX_ONE: u64 = 100 * 1024 * 1024;

#[derive(Debug, PartialEq)]
pub enum ZipError {
    Format,
    Unsafe(String),
    TooBig,
    Unsupported,
    Io(String),
}

fn u16le(b: &[u8], i: usize) -> Option<u16> {
    Some(u16::from_le_bytes([*b.get(i)?, *b.get(i + 1)?]))
}
fn u32le(b: &[u8], i: usize) -> Option<u32> {
    Some(u32::from_le_bytes([*b.get(i)?, *b.get(i + 1)?, *b.get(i + 2)?, *b.get(i + 3)?]))
}

/// Ein Eintrag aus dem Inhaltsverzeichnis
#[derive(Debug)]
struct Entry {
    name: String,
    method: u16,
    comp_size: u64,
    size: u64,
    local_offset: u64,
    is_symlink: bool,
}

/// Pfad aus dem ZIP → sicherer relativer Pfad (oder Fehler)
pub fn safe_entry_path(name: &str) -> Result<PathBuf, ZipError> {
    let n = name.replace('\\', "/");
    if n.starts_with('/') || n.contains(':') || n.contains('\0') || n.len() > 300 {
        return Err(ZipError::Unsafe(name.to_string()));
    }
    let mut out = PathBuf::new();
    for part in n.split('/') {
        match part {
            "" | "." => continue,
            ".." => return Err(ZipError::Unsafe(name.to_string())),
            p => out.push(p),
        }
    }
    Ok(out)
}

fn entries(data: &[u8]) -> Result<Vec<Entry>, ZipError> {
    // Ende des Inhaltsverzeichnisses suchen (End of Central Directory, PK\x05\x06)
    let min = data.len().saturating_sub(65_557);
    let eocd = (min..data.len().saturating_sub(21))
        .rev()
        .find(|&i| data[i..].starts_with(&[0x50, 0x4b, 0x05, 0x06]))
        .ok_or(ZipError::Format)?;
    let count = u16le(data, eocd + 10).ok_or(ZipError::Format)? as usize;
    let cd_offset = u32le(data, eocd + 16).ok_or(ZipError::Format)? as usize;
    if count > MAX_FILES {
        return Err(ZipError::TooBig);
    }
    let mut out = Vec::new();
    let mut p = cd_offset;
    for _ in 0..count {
        if u32le(data, p) != Some(0x0201_4b50) {
            return Err(ZipError::Format);
        }
        let made_by = u16le(data, p + 4).ok_or(ZipError::Format)?;
        let flags = u16le(data, p + 8).ok_or(ZipError::Format)?;
        let method = u16le(data, p + 10).ok_or(ZipError::Format)?;
        let comp_size = u32le(data, p + 20).ok_or(ZipError::Format)? as u64;
        let size = u32le(data, p + 24).ok_or(ZipError::Format)? as u64;
        let name_len = u16le(data, p + 28).ok_or(ZipError::Format)? as usize;
        let extra_len = u16le(data, p + 30).ok_or(ZipError::Format)? as usize;
        let comment_len = u16le(data, p + 32).ok_or(ZipError::Format)? as usize;
        let ext_attr = u32le(data, p + 38).ok_or(ZipError::Format)?;
        let local_offset = u32le(data, p + 42).ok_or(ZipError::Format)? as u64;
        let name_bytes = data.get(p + 46..p + 46 + name_len).ok_or(ZipError::Format)?;
        if flags & 0x1 != 0 {
            return Err(ZipError::Unsupported); // verschlüsselt
        }
        // Unix-Symlink (S_IFLNK) in den oberen 16 Bit der externen Attribute
        let unix_mode = if made_by >> 8 == 3 { ext_attr >> 16 } else { 0 };
        out.push(Entry {
            name: String::from_utf8_lossy(name_bytes).to_string(),
            method,
            comp_size,
            size,
            local_offset,
            is_symlink: unix_mode & 0o170000 == 0o120000,
        });
        p += 46 + name_len + extra_len + comment_len;
    }
    Ok(out)
}

/// ZIP-Daten in einen (neuen, leeren) Ordner entpacken
pub fn extract(data: &[u8], dest: &Path) -> Result<Vec<PathBuf>, ZipError> {
    let list = entries(data)?;
    let total: u64 = list.iter().map(|e| e.size).sum();
    if total > MAX_TOTAL {
        return Err(ZipError::TooBig);
    }
    let mut written = Vec::new();
    for e in list {
        if e.is_symlink {
            return Err(ZipError::Unsafe(e.name));
        }
        let rel = safe_entry_path(&e.name)?;
        if rel.as_os_str().is_empty() {
            continue;
        }
        let target = dest.join(&rel);
        if e.name.ends_with('/') {
            std::fs::create_dir_all(&target).map_err(|x| ZipError::Io(x.to_string()))?;
            continue;
        }
        if e.size > MAX_ONE || e.comp_size > MAX_ONE {
            return Err(ZipError::TooBig);
        }
        let lo = e.local_offset as usize;
        if u32le(data, lo) != Some(0x0403_4b50) {
            return Err(ZipError::Format);
        }
        let n = u16le(data, lo + 26).ok_or(ZipError::Format)? as usize;
        let x = u16le(data, lo + 28).ok_or(ZipError::Format)? as usize;
        let start = lo + 30 + n + x;
        let raw = data.get(start..start + e.comp_size as usize).ok_or(ZipError::Format)?;
        let bytes = match e.method {
            0 => raw.to_vec(),
            8 => {
                let mut out = Vec::new();
                flate2::read::DeflateDecoder::new(raw)
                    .take(e.size + 1)
                    .read_to_end(&mut out)
                    .map_err(|_| ZipError::Format)?;
                out
            }
            _ => return Err(ZipError::Unsupported),
        };
        if bytes.len() as u64 != e.size {
            return Err(ZipError::Format);
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|x| ZipError::Io(x.to_string()))?;
        }
        std::fs::write(&target, &bytes).map_err(|x| ZipError::Io(x.to_string()))?;
        written.push(rel);
    }
    Ok(written)
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// Einfaches ZIP (nur "stored") für Tests bauen
    pub fn make_zip(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut central = Vec::new();
        for (name, data) in files {
            let off = out.len() as u32;
            let crc = 0u32; // wird beim Lesen nicht geprüft
            out.extend_from_slice(&[0x50, 0x4b, 0x03, 0x04, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
            out.extend_from_slice(&crc.to_le_bytes());
            out.extend_from_slice(&(data.len() as u32).to_le_bytes());
            out.extend_from_slice(&(data.len() as u32).to_le_bytes());
            out.extend_from_slice(&(name.len() as u16).to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes());
            out.extend_from_slice(name.as_bytes());
            out.extend_from_slice(data);
            central.extend_from_slice(&[0x50, 0x4b, 0x01, 0x02, 20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
            central.extend_from_slice(&crc.to_le_bytes());
            central.extend_from_slice(&(data.len() as u32).to_le_bytes());
            central.extend_from_slice(&(data.len() as u32).to_le_bytes());
            central.extend_from_slice(&(name.len() as u16).to_le_bytes());
            central.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
            central.extend_from_slice(&off.to_le_bytes());
            central.extend_from_slice(name.as_bytes());
        }
        let cd_off = out.len() as u32;
        let cd_len = central.len() as u32;
        out.extend_from_slice(&central);
        out.extend_from_slice(&[0x50, 0x4b, 0x05, 0x06, 0, 0, 0, 0]);
        out.extend_from_slice(&(files.len() as u16).to_le_bytes());
        out.extend_from_slice(&(files.len() as u16).to_le_bytes());
        out.extend_from_slice(&cd_len.to_le_bytes());
        out.extend_from_slice(&cd_off.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out
    }

    #[test]
    fn extract_and_zip_slip() {
        let dir = std::env::temp_dir().join(format!("maz-zip-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let ok = make_zip(&[("manifest.json", b"{}"), ("bin/run", b"hi")]);
        let files = extract(&ok, &dir).unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(std::fs::read(dir.join("bin/run")).unwrap(), b"hi");
        for evil in ["../evil", "/etc/passwd", "C:\\evil", "a/../../evil", "..\\evil"] {
            let z = make_zip(&[(evil, b"x")]);
            assert!(matches!(extract(&z, &dir), Err(ZipError::Unsafe(_))), "{evil}");
        }
        assert!(!dir.parent().unwrap().join("evil").exists());
        assert_eq!(extract(b"kein zip", &dir), Err(ZipError::Format));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
