//! Direkte Windows-Aufrufe (ohne große Bindings-Crates → kleinere EXE).
//!
//! - `shell_execute`: Programme/URLs/Protokolle öffnen, auch "Als Administrator"
//! - `extract_icon_rgba`: das Original-Icon aus einer EXE holen (256×256)
//! - `set_clipboard`: Text in die Zwischenablage kopieren

#![allow(non_snake_case, dead_code, clippy::upper_case_acronyms)]

use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};

type HANDLE = *mut c_void;
type BOOL = i32;

pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

#[repr(C)]
struct IconInfo {
    f_icon: BOOL,
    x_hotspot: u32,
    y_hotspot: u32,
    hbm_mask: HANDLE,
    hbm_color: HANDLE,
}

#[repr(C)]
struct Bitmap {
    bm_type: i32,
    bm_width: i32,
    bm_height: i32,
    bm_width_bytes: i32,
    bm_planes: u16,
    bm_bits_pixel: u16,
    bm_bits: *mut c_void,
}

#[repr(C)]
struct BitmapInfoHeader {
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bit_count: u16,
    compression: u32,
    size_image: u32,
    x_ppm: i32,
    y_ppm: i32,
    clr_used: u32,
    clr_important: u32,
}

#[repr(C)]
struct BitmapInfo {
    header: BitmapInfoHeader,
    colors: [u32; 4],
}

#[link(name = "shell32")]
extern "system" {
    fn ShellExecuteW(
        hwnd: HANDLE,
        op: *const u16,
        file: *const u16,
        params: *const u16,
        dir: *const u16,
        show: i32,
    ) -> HANDLE;
}

#[link(name = "user32")]
extern "system" {
    fn PrivateExtractIconsW(
        file: *const u16,
        index: i32,
        cx: i32,
        cy: i32,
        phicon: *mut HANDLE,
        piconid: *mut u32,
        count: u32,
        flags: u32,
    ) -> u32;
    fn GetIconInfo(hicon: HANDLE, info: *mut IconInfo) -> BOOL;
    fn DestroyIcon(hicon: HANDLE) -> BOOL;
    fn GetDC(hwnd: HANDLE) -> HANDLE;
    fn ReleaseDC(hwnd: HANDLE, hdc: HANDLE) -> i32;
    fn OpenClipboard(hwnd: HANDLE) -> BOOL;
    fn EmptyClipboard() -> BOOL;
    fn SetClipboardData(format: u32, mem: HANDLE) -> HANDLE;
    fn GetClipboardData(format: u32) -> HANDLE;
    fn CloseClipboard() -> BOOL;
}

#[link(name = "gdi32")]
extern "system" {
    fn GetDIBits(
        hdc: HANDLE,
        hbm: HANDLE,
        start: u32,
        lines: u32,
        bits: *mut c_void,
        bmi: *mut BitmapInfo,
        usage: u32,
    ) -> i32;
    fn GetObjectW(h: HANDLE, size: i32, out: *mut c_void) -> i32;
    fn DeleteObject(h: HANDLE) -> BOOL;
}

#[link(name = "kernel32")]
extern "system" {
    fn GlobalAlloc(flags: u32, bytes: usize) -> HANDLE;
    fn GlobalLock(mem: HANDLE) -> *mut c_void;
    fn GlobalUnlock(mem: HANDLE) -> BOOL;
    fn GlobalFree(mem: HANDLE) -> HANDLE;
    fn GlobalSize(mem: HANDLE) -> usize;
}

pub const SW_SHOWNORMAL: i32 = 1;

/// Öffnet eine Datei, URL oder ein Protokoll (steam://…) über Windows.
/// `verb` = None → Standardaktion, Some("runas") → als Administrator.
/// Fehler: der ShellExecute-Code (≤ 32).
pub fn shell_execute(
    verb: Option<&str>,
    file: &str,
    params: Option<&str>,
    dir: Option<&str>,
) -> Result<(), i32> {
    let verb_w = verb.map(wide);
    let file_w = wide(file);
    let params_w = params.filter(|p| !p.is_empty()).map(wide);
    let dir_w = dir.filter(|d| !d.is_empty()).map(wide);
    let r = unsafe {
        ShellExecuteW(
            null_mut(),
            verb_w.as_ref().map_or(null(), |v| v.as_ptr()),
            file_w.as_ptr(),
            params_w.as_ref().map_or(null(), |v| v.as_ptr()),
            dir_w.as_ref().map_or(null(), |v| v.as_ptr()),
            SW_SHOWNORMAL,
        )
    } as isize;
    if r > 32 {
        Ok(())
    } else {
        Err(r as i32)
    }
}

/// Holt das Icon einer EXE/DLL als RGBA-Bild (Breite, Höhe, Pixel).
pub fn extract_icon_rgba(path: &str, size: i32) -> Option<(u32, u32, Vec<u8>)> {
    let wpath = wide(path);
    let mut hicon: HANDLE = null_mut();
    let mut id: u32 = 0;
    unsafe {
        let n = PrivateExtractIconsW(wpath.as_ptr(), 0, size, size, &mut hicon, &mut id, 1, 0);
        if n == 0 || n == u32::MAX || hicon.is_null() {
            return None;
        }
        let result = icon_to_rgba(hicon);
        DestroyIcon(hicon);
        result
    }
}

unsafe fn read_bits(hbm: HANDLE, w: i32, h: i32) -> Option<Vec<u8>> {
    let mut bmi: BitmapInfo = zeroed();
    bmi.header.size = size_of::<BitmapInfoHeader>() as u32;
    bmi.header.width = w;
    bmi.header.height = -h; // oben → unten
    bmi.header.planes = 1;
    bmi.header.bit_count = 32;
    bmi.header.compression = 0; // BI_RGB
    let mut buf = vec![0u8; (w as usize) * (h as usize) * 4];
    let dc = GetDC(null_mut());
    let lines = GetDIBits(dc, hbm, 0, h as u32, buf.as_mut_ptr() as *mut c_void, &mut bmi, 0);
    ReleaseDC(null_mut(), dc);
    if lines == 0 {
        None
    } else {
        Some(buf)
    }
}

unsafe fn icon_to_rgba(hicon: HANDLE) -> Option<(u32, u32, Vec<u8>)> {
    let mut info: IconInfo = zeroed();
    if GetIconInfo(hicon, &mut info) == 0 {
        return None;
    }
    let result = (|| {
        if info.hbm_color.is_null() {
            return None;
        }
        let mut bm: Bitmap = zeroed();
        if GetObjectW(
            info.hbm_color,
            size_of::<Bitmap>() as i32,
            &mut bm as *mut Bitmap as *mut c_void,
        ) == 0
        {
            return None;
        }
        let (w, h) = (bm.bm_width, bm.bm_height);
        if w <= 0 || h <= 0 || w > 1024 || h > 1024 {
            return None;
        }
        let px = read_bits(info.hbm_color, w, h)?;
        let count = (w as usize) * (h as usize);
        let mut rgba = vec![0u8; count * 4];
        let mut has_alpha = false;
        for i in 0..count {
            rgba[i * 4] = px[i * 4 + 2];
            rgba[i * 4 + 1] = px[i * 4 + 1];
            rgba[i * 4 + 2] = px[i * 4];
            rgba[i * 4 + 3] = px[i * 4 + 3];
            if px[i * 4 + 3] != 0 {
                has_alpha = true;
            }
        }
        if !has_alpha {
            // Alte Icons ohne Alphakanal: Transparenz aus der Maske nehmen
            let mask = if info.hbm_mask.is_null() {
                None
            } else {
                read_bits(info.hbm_mask, w, h)
            };
            for i in 0..count {
                let transparent = matches!(&mask, Some(m) if m[i * 4] != 0);
                rgba[i * 4 + 3] = if transparent { 0 } else { 255 };
            }
        }
        // Komplett leeres Icon? Dann lieber keins.
        if rgba.chunks(4).all(|p| p[3] == 0) {
            return None;
        }
        Some((w as u32, h as u32, rgba))
    })();
    if !info.hbm_color.is_null() {
        DeleteObject(info.hbm_color);
    }
    if !info.hbm_mask.is_null() {
        DeleteObject(info.hbm_mask);
    }
    let _ = (info.f_icon, info.x_hotspot, info.y_hotspot);
    result
}

/// Text in die Windows-Zwischenablage kopieren.
pub fn set_clipboard(text: &str) -> bool {
    let w: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        if OpenClipboard(null_mut()) == 0 {
            return false;
        }
        EmptyClipboard();
        let bytes = w.len() * 2;
        let mem = GlobalAlloc(0x0002, bytes); // GMEM_MOVEABLE
        if mem.is_null() {
            CloseClipboard();
            return false;
        }
        let p = GlobalLock(mem) as *mut u16;
        if p.is_null() {
            GlobalFree(mem);
            CloseClipboard();
            return false;
        }
        std::ptr::copy_nonoverlapping(w.as_ptr(), p, w.len());
        GlobalUnlock(mem);
        let ok = !SetClipboardData(13, mem).is_null(); // CF_UNICODETEXT
        if !ok {
            GlobalFree(mem);
        }
        CloseClipboard();
        ok
    }
}

// ----------------------------------------------------------------------
// Laufende Prozesse (für "spielt gerade")
// ----------------------------------------------------------------------

#[repr(C)]
struct ProcessEntry32W {
    size: u32,
    usage: u32,
    process_id: u32,
    default_heap_id: usize,
    module_id: u32,
    threads: u32,
    parent_process_id: u32,
    pri_class_base: i32,
    flags: u32,
    exe_file: [u16; 260],
}

#[link(name = "kernel32")]
extern "system" {
    fn CreateToolhelp32Snapshot(flags: u32, pid: u32) -> HANDLE;
    fn Process32FirstW(snap: HANDLE, entry: *mut ProcessEntry32W) -> BOOL;
    fn Process32NextW(snap: HANDLE, entry: *mut ProcessEntry32W) -> BOOL;
    fn CloseHandle(h: HANDLE) -> BOOL;
}

/// Dateinamen aller laufenden Prozesse (klein geschrieben, z. B. "witcher3.exe")
pub fn process_names() -> Vec<String> {
    let mut out = Vec::new();
    unsafe {
        let snap = CreateToolhelp32Snapshot(0x0000_0002, 0); // TH32CS_SNAPPROCESS
        if snap.is_null() || snap as isize == -1 {
            return out;
        }
        let mut e: ProcessEntry32W = zeroed();
        e.size = size_of::<ProcessEntry32W>() as u32;
        if Process32FirstW(snap, &mut e) != 0 {
            loop {
                let len = e.exe_file.iter().position(|c| *c == 0).unwrap_or(e.exe_file.len());
                out.push(String::from_utf16_lossy(&e.exe_file[..len]).to_lowercase());
                if Process32NextW(snap, &mut e) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);
    }
    out
}

// ----------------------------------------------------------------------
// Zeitzone (für Uhrzeiten im Chat)
// ----------------------------------------------------------------------

#[repr(C)]
struct TimeZoneInformation {
    bias: i32,
    standard_name: [u16; 32],
    standard_date: [u16; 8],
    standard_bias: i32,
    daylight_name: [u16; 32],
    daylight_date: [u16; 8],
    daylight_bias: i32,
}

#[link(name = "kernel32")]
extern "system" {
    fn GetTimeZoneInformation(info: *mut TimeZoneInformation) -> u32;
}

/// Abstand der lokalen Zeit zu UTC in Sekunden (z. B. +3600 im Winter in Deutschland)
pub fn utc_offset_secs() -> i64 {
    unsafe {
        let mut tz: TimeZoneInformation = zeroed();
        let id = GetTimeZoneInformation(&mut tz);
        let mut bias = tz.bias;
        if id == 2 {
            bias += tz.daylight_bias;
        } else if id == 1 {
            bias += tz.standard_bias;
        }
        -(bias as i64) * 60
    }
}

// ----------------------------------------------------------------------
// Zwischenablage lesen (für "Epic-Konto verbinden")
// ----------------------------------------------------------------------

/// Text aus der Windows-Zwischenablage (höchstens 64 KB)
pub fn get_clipboard() -> Option<String> {
    unsafe {
        if OpenClipboard(null_mut()) == 0 {
            return None;
        }
        let mut out = None;
        let mem = GetClipboardData(13); // CF_UNICODETEXT
        if !mem.is_null() {
            let p = GlobalLock(mem) as *const u16;
            if !p.is_null() {
                // Nie über das Ende des Speicherblocks hinaus lesen
                let max = (GlobalSize(mem) / 2).min(65_536);
                let mut len = 0usize;
                while len < max && *p.add(len) != 0 {
                    len += 1;
                }
                out = Some(String::from_utf16_lossy(std::slice::from_raw_parts(p, len)));
                GlobalUnlock(mem);
            }
        }
        CloseClipboard();
        out
    }
}

// ----------------------------------------------------------------------
// Prozesse beobachten (Absturz-Erkennung bei Steam- und Epic-Spielen)
// ----------------------------------------------------------------------

#[link(name = "kernel32")]
extern "system" {
    fn OpenProcess(access: u32, inherit: BOOL, pid: u32) -> HANDLE;
    fn QueryFullProcessImageNameW(h: HANDLE, flags: u32, name: *mut u16, size: *mut u32) -> BOOL;
    fn GetExitCodeProcess(h: HANDLE, code: *mut u32) -> BOOL;
}

const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
const SYNCHRONIZE: u32 = 0x0010_0000;

/// Geöffneter Prozess – wird beim Verwerfen automatisch geschlossen.
pub struct Proc(HANDLE);

unsafe impl Send for Proc {}

impl Drop for Proc {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

impl Proc {
    pub fn open(pid: u32) -> Option<Proc> {
        let h = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE, 0, pid) };
        (!h.is_null()).then(|| Proc(h))
    }

    /// Vollständiger Pfad der EXE
    pub fn path(&self) -> Option<String> {
        let mut buf = [0u16; 1024];
        let mut size = buf.len() as u32;
        let ok = unsafe { QueryFullProcessImageNameW(self.0, 0, buf.as_mut_ptr(), &mut size) };
        (ok != 0).then(|| String::from_utf16_lossy(&buf[..size as usize]))
    }

    /// None = läuft noch, Some(code) = beendet
    pub fn exit_code(&self) -> Option<u32> {
        let mut code = 0u32;
        let ok = unsafe { GetExitCodeProcess(self.0, &mut code) };
        if ok == 0 || code == 259 {
            None
        } else {
            Some(code)
        }
    }
}

/// Alle laufenden Prozesse als (Prozess-ID, Dateiname klein)
pub fn process_ids() -> Vec<(u32, String)> {
    let mut out = Vec::new();
    unsafe {
        let snap = CreateToolhelp32Snapshot(0x0000_0002, 0);
        if snap.is_null() || snap as isize == -1 {
            return out;
        }
        let mut e: ProcessEntry32W = zeroed();
        e.size = size_of::<ProcessEntry32W>() as u32;
        if Process32FirstW(snap, &mut e) != 0 {
            loop {
                let len = e.exe_file.iter().position(|c| *c == 0).unwrap_or(e.exe_file.len());
                out.push((e.process_id, String::from_utf16_lossy(&e.exe_file[..len]).to_lowercase()));
                if Process32NextW(snap, &mut e) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);
    }
    out
}

// ----------------------------------------------------------------------
// DPAPI: Anmelde-Tokens verschlüsselt speichern (nur dieser Windows-Benutzer kann sie lesen)
// ----------------------------------------------------------------------

#[repr(C)]
struct DataBlob {
    cb: u32,
    pb: *mut u8,
}

#[link(name = "crypt32")]
extern "system" {
    fn CryptProtectData(
        input: *const DataBlob,
        descr: *const u16,
        entropy: *const DataBlob,
        reserved: *mut c_void,
        prompt: *mut c_void,
        flags: u32,
        output: *mut DataBlob,
    ) -> BOOL;
    fn CryptUnprotectData(
        input: *const DataBlob,
        descr: *mut *mut u16,
        entropy: *const DataBlob,
        reserved: *mut c_void,
        prompt: *mut c_void,
        flags: u32,
        output: *mut DataBlob,
    ) -> BOOL;
}

#[link(name = "kernel32")]
extern "system" {
    fn LocalFree(mem: HANDLE) -> HANDLE;
}

const ENTROPY: &[u8] = b"MAZ-Launcher-Token";
const CRYPTPROTECT_UI_FORBIDDEN: u32 = 0x1;

fn dpapi(data: &[u8], protect: bool) -> Option<Vec<u8>> {
    if data.is_empty() || data.len() > 1 << 20 {
        return None;
    }
    let input = DataBlob { cb: data.len() as u32, pb: data.as_ptr() as *mut u8 };
    let entropy = DataBlob { cb: ENTROPY.len() as u32, pb: ENTROPY.as_ptr() as *mut u8 };
    let mut out = DataBlob { cb: 0, pb: null_mut() };
    let ok = unsafe {
        if protect {
            CryptProtectData(&input, null(), &entropy, null_mut(), null_mut(), CRYPTPROTECT_UI_FORBIDDEN, &mut out)
        } else {
            CryptUnprotectData(&input, null_mut(), &entropy, null_mut(), null_mut(), CRYPTPROTECT_UI_FORBIDDEN, &mut out)
        }
    };
    if ok == 0 || out.pb.is_null() {
        return None;
    }
    let v = unsafe { std::slice::from_raw_parts(out.pb, out.cb as usize).to_vec() };
    unsafe {
        LocalFree(out.pb as HANDLE);
    }
    Some(v)
}

pub fn dpapi_protect(data: &[u8]) -> Option<Vec<u8>> {
    dpapi(data, true)
}

pub fn dpapi_unprotect(data: &[u8]) -> Option<Vec<u8>> {
    dpapi(data, false)
}

// ----------------------------------------------------------------------
// Windows-Systemordner (für absolute Pfade zu cmd.exe, PowerShell, explorer.exe)
// ----------------------------------------------------------------------

#[link(name = "kernel32")]
extern "system" {
    fn GetSystemDirectoryW(buf: *mut u16, size: u32) -> u32;
    fn GetWindowsDirectoryW(buf: *mut u16, size: u32) -> u32;
}

fn dir_from(f: unsafe extern "system" fn(*mut u16, u32) -> u32) -> Option<std::path::PathBuf> {
    let mut buf = [0u16; 520];
    let n = unsafe { f(buf.as_mut_ptr(), buf.len() as u32) } as usize;
    (n > 0 && n < buf.len()).then(|| std::path::PathBuf::from(String::from_utf16_lossy(&buf[..n])))
}

/// z. B. C:\Windows\System32
pub fn system_dir() -> Option<std::path::PathBuf> {
    dir_from(GetSystemDirectoryW)
}

/// z. B. C:\Windows
pub fn windows_dir() -> Option<std::path::PathBuf> {
    dir_from(GetWindowsDirectoryW)
}

// ----------------------------------------------------------------------
// Job-Objekt: Kindprozesse (Erweiterungen) sterben garantiert mit dem Launcher
// ----------------------------------------------------------------------

#[repr(C)]
#[derive(Default)]
struct JobBasicLimit {
    per_process_user_time: i64,
    per_job_user_time: i64,
    limit_flags: u32,
    min_working_set: usize,
    max_working_set: usize,
    active_process_limit: u32,
    affinity: usize,
    priority_class: u32,
    scheduling_class: u32,
}

#[repr(C)]
#[derive(Default)]
struct IoCounters {
    v: [u64; 6],
}

#[repr(C)]
#[derive(Default)]
struct JobExtendedLimit {
    basic: JobBasicLimit,
    io: IoCounters,
    process_memory_limit: usize,
    job_memory_limit: usize,
    peak_process_memory: usize,
    peak_job_memory: usize,
}

#[link(name = "kernel32")]
extern "system" {
    fn CreateJobObjectW(attrs: *const c_void, name: *const u16) -> HANDLE;
    fn SetInformationJobObject(job: HANDLE, class: i32, info: *const c_void, len: u32) -> BOOL;
    fn AssignProcessToJobObject(job: HANDLE, process: HANDLE) -> BOOL;
}

pub struct Job(HANDLE);

// SAFETY: Ein Job-Handle darf zwischen Threads weitergegeben werden.
unsafe impl Send for Job {}

impl Job {
    /// Job, der beim Schließen (auch beim Absturz des Launchers) alle Prozesse beendet
    pub fn kill_on_close() -> Option<Job> {
        unsafe {
            let h = CreateJobObjectW(null(), null());
            if h.is_null() {
                return None;
            }
            let mut info = JobExtendedLimit::default();
            info.basic.limit_flags = 0x2000; // JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
            let ok = SetInformationJobObject(h, 9, &info as *const _ as *const c_void, size_of::<JobExtendedLimit>() as u32);
            if ok == 0 {
                CloseHandle(h);
                return None;
            }
            Some(Job(h))
        }
    }

    pub fn assign(&self, child: &std::process::Child) -> bool {
        use std::os::windows::io::AsRawHandle;
        unsafe { AssignProcessToJobObject(self.0, child.as_raw_handle() as HANDLE) != 0 }
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}
