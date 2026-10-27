//! Open Game Launcher (OG) – alle deine Steam-, Epic- und eigenen Spiele in einem Launcher.
//! 100 % Rust · Dioxus · Komponenten im Stil von rust-ui.

// Kein schwarzes Konsolenfenster im Release-Build
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod achievements;
mod appinfo;
mod backup;
mod appx;
mod config;
mod epic;
mod emulators;
mod epicacct;
mod extensions;
mod freegames;
mod i18n;
mod icons;
mod install;
mod launch;
mod metadata;
mod library;
mod mccontent;
mod minecraft;
mod model;
mod notes;
mod playnite;
mod playtime;
mod problem;
mod roblox;
mod runners;
mod steam;
mod sgdb;
mod stats;
mod stores;
mod sys;
mod theme;
mod ui;
mod updater;
mod util;
mod vdf;
#[cfg(windows)]
mod winffi;

use dioxus::desktop::tao::window::Icon as WindowIcon;
use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use dioxus::prelude::*;
use std::sync::OnceLock;

/// Wie wurde gestartet? (Installer oder Launcher)
pub static START: OnceLock<install::StartMode> = OnceLock::new();

/// Fertig gebautes Tailwind-CSS – direkt in die EXE eingebettet (keine Extra-Dateien).
const CSS: &str = include_str!("../assets/maz.css");
const ICON_PNG: &[u8] = include_bytes!("../assets/icon-256.png");

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    install_panic_hook();
    i18n::set_language(&config::load().language);

    if args.iter().any(|a| a.eq_ignore_ascii_case("--uninstall")) {
        install::uninstall_interactive();
        return;
    }

    if !sys::webview2_installed() {
        show_startup_problem(&problem::Problem::webview2_missing());
        return;
    }

    updater::cleanup_old();
    install::migrate_names();
    let mode = install::decide(&args);
    let installer = matches!(mode, install::StartMode::Installer { .. });
    let _ = START.set(mode);

    let (w, h) = if installer { (920.0, 640.0) } else { (1320.0, 840.0) };
    let window = WindowBuilder::new()
        .with_title(sys::APP_NAME)
        .with_inner_size(LogicalSize::new(w, h))
        .with_min_inner_size(LogicalSize::new(860.0, 580.0))
        // macOS: echte Ampel-Knöpfe (schließen/minimieren/Vollbild) in einer durchsichtigen
        // Titelleiste – die eigene Leiste darunter lässt links Platz dafür.
        .with_decorations(cfg!(target_os = "macos"))
        .with_resizable(true);
    // Logo für Fenster und Taskleiste direkt am Fenster setzen (zusätzlich zum EXE-Icon),
    // sonst zeigt Windows in der Taskleiste manchmal nur ein leeres Symbol.
    let window = window.with_window_icon(window_icon());
    #[cfg(windows)]
    let window = {
        use dioxus::desktop::tao::platform::windows::WindowBuilderExtWindows;
        window.with_taskbar_icon(window_icon())
    };
    #[cfg(target_os = "macos")]
    let window = {
        use dioxus::desktop::tao::platform::macos::WindowBuilderExtMacOS;
        window.with_titlebar_transparent(true).with_fullsize_content_view(true).with_title_hidden(true)
    };

    let mut cfg = Config::new()
        .with_window(window)
        .with_disable_context_menu(!cfg!(debug_assertions))
        .with_background_color((10, 10, 18, 255))
        .with_custom_head(format!(
            "<meta name=\"color-scheme\" content=\"dark light\"><style>{CSS}</style>"
        ))
        .with_data_directory(sys::local_dir().join("webview"));
    if let Some(icon) = window_icon() {
        cfg = cfg.with_icon(icon);
    }

    LaunchBuilder::desktop().with_cfg(cfg).launch(ui::Root);
    // Falls das Fenster-System zurückkehrt: Erweiterungs-Prozesse sauber beenden
    extensions::shutdown();
}

fn window_icon() -> Option<WindowIcon> {
    let img = image::load_from_memory_with_format(ICON_PNG, image::ImageFormat::Png).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    WindowIcon::from_rgba(rgba.into_raw(), w, h).ok()
}

/// Zeigt ein Problem als einfaches Windows-Fenster (wenn die Oberfläche nicht starten kann).
fn show_startup_problem(p: &problem::Problem) {
    let mut text = format!("{}\n\n", p.what_happened);
    text.push_str(&format!("{}:\n", i18n::t("problem.steps")));
    for (i, s) in p.steps.iter().enumerate() {
        text.push_str(&format!("{}. {}\n", i + 1, s));
    }
    eprintln!("{}\n{}", p.title, text);
    {
        use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
        let res = MessageDialog::new()
            .set_level(MessageLevel::Error)
            .set_title(format!("{} – {}", sys::APP_NAME, p.title))
            .set_description(text)
            .set_buttons(MessageButtons::OkCancel)
            .show();
        if res == MessageDialogResult::Ok {
            for f in &p.fixes {
                if let problem::Fix::OpenUrl { url, .. } = f {
                    let _ = sys::open_url(url);
                    break;
                }
            }
        }
    }
}

/// Falls doch einmal etwas abstürzt: verständliche Meldung + Logdatei statt stillem Verschwinden.
fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let msg = info.to_string();
        let log = sys::local_dir().join("absturz.log");
        let _ = std::fs::create_dir_all(sys::local_dir());
        let _ = std::fs::write(&log, format!("{} {}\n{}\n", sys::APP_NAME, sys::VERSION, msg));
        let p = problem::Problem::app_crashed(
            &log.display().to_string(),
            &msg,
            &format!("{}/issues", sys::GITHUB_URL),
        );
        show_startup_problem(&p);
    }));
}
