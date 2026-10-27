//! Symbol im Infobereich der Taskleiste mit „Zuletzt gespielt“ zum Schnellstart.
//!
//! Klicks im Menü landen in einer kleinen Warteschlange und werden im Dioxus-Teil verarbeitet.
//! Unter Linux nur, wenn die AppIndicator-Bibliothek vorhanden ist (sonst kein Symbol).

use super::state::*;
use crate::i18n::t;
use dioxus::desktop::trayicon::menu::{Menu, MenuItem, PredefinedMenuItem};
use dioxus::desktop::trayicon::{Icon as TrayImg, TrayIcon, TrayIconBuilder};
use dioxus::desktop::window;
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Mutex;

static QUEUE: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Unter Linux braucht das Symbol libayatana-appindicator (oder libappindicator)
fn tray_supported() -> bool {
    if !cfg!(target_os = "linux") {
        return true;
    }
    let dirs = [
        "/usr/lib",
        "/usr/lib64",
        "/usr/lib/x86_64-linux-gnu",
        "/usr/lib/aarch64-linux-gnu",
        "/lib/x86_64-linux-gnu",
        "/lib/aarch64-linux-gnu",
        "/usr/local/lib",
        "/app/lib",
        "/run/current-system/sw/lib",
    ];
    let names = ["libayatana-appindicator3.so.1", "libappindicator3.so.1"];
    dirs.iter().any(|d| names.iter().any(|n| std::path::Path::new(d).join(n).exists()))
}

fn icon() -> Option<TrayImg> {
    let img = image::load_from_memory_with_format(include_bytes!("../../assets/icon-256.png"), image::ImageFormat::Png).ok()?;
    let small = img.resize(64, 64, image::imageops::FilterType::Lanczos3).to_rgba8();
    let (w, h) = small.dimensions();
    TrayImg::from_rgba(small.into_raw(), w, h).ok()
}

/// Menü: Öffnen · zuletzt gespielte Spiele · Beenden
fn build_menu(recent: &[(String, String)]) -> Menu {
    let menu = Menu::new();
    let _ = menu.append(&MenuItem::with_id("og-open", t("tray.open"), true, None));
    let _ = menu.append(&PredefinedMenuItem::separator());
    if recent.is_empty() {
        let _ = menu.append(&MenuItem::with_id("og-none", t("tray.no_recent"), false, None));
    }
    for (id, name) in recent {
        let label: String = name.chars().take(40).collect();
        let _ = menu.append(&MenuItem::with_id(format!("og-play:{id}"), format!("▶  {label}"), true, None));
    }
    let _ = menu.append(&PredefinedMenuItem::separator());
    let _ = menu.append(&MenuItem::with_id("og-quit", t("tray.quit"), true, None));
    menu
}

fn recent_games(app: App) -> Vec<(String, String)> {
    let s = app.settings.read();
    let mut list: Vec<(u64, String, String)> = app
        .games
        .read()
        .iter()
        .filter(|g| g.installed && s.last_played(&g.id) > 0 && !s.is_hidden(&g.id))
        .map(|g| (s.last_played(&g.id), g.id.clone(), g.name.clone()))
        .collect();
    list.sort_by(|a, b| b.0.cmp(&a.0));
    list.into_iter().take(6).map(|(_, id, n)| (id, n)).collect()
}

#[component]
pub fn TrayHost() -> Element {
    let app = use_app();
    let enabled = app.settings.read().tray_icon && tray_supported();
    let tray: Rc<RefCell<Option<TrayIcon>>> = use_hook(|| Rc::new(RefCell::new(None)));

    // Klicks im Menü einsammeln (läuft außerhalb von Dioxus → nur in die Warteschlange legen)
    dioxus::desktop::use_tray_menu_event_handler(move |ev| {
        if ev.id.0.starts_with("og-") {
            if let Ok(mut q) = QUEUE.lock() {
                q.push(ev.id.0.clone());
            }
        }
    });

    // Warteschlange abarbeiten
    use_hook(move || {
        spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                let items: Vec<String> = QUEUE.lock().map(|mut q| std::mem::take(&mut *q)).unwrap_or_default();
                for id in items {
                    let w = window();
                    match id.as_str() {
                        "og-open" => {
                            w.window.set_visible(true);
                            w.window.set_minimized(false);
                            w.window.set_focus();
                        }
                        "og-quit" => w.close(),
                        other => {
                            if let Some(gid) = other.strip_prefix("og-play:") {
                                play(app, gid.to_string());
                            }
                        }
                    }
                }
            }
        });
    });

    // Symbol anlegen/entfernen und Menü aktuell halten
    let recent = recent_games(app);
    let t2 = tray.clone();
    use_effect(use_reactive((&enabled, &recent), move |(enabled, recent)| {
        let mut slot = t2.borrow_mut();
        if !enabled {
            *slot = None;
            return;
        }
        match slot.as_ref() {
            Some(tr) => tr.set_menu(Some(Box::new(build_menu(&recent)))),
            None => {
                let mut b = TrayIconBuilder::new().with_menu(Box::new(build_menu(&recent))).with_tooltip(crate::sys::APP_NAME);
                if let Some(i) = icon() {
                    b = b.with_icon(i);
                }
                *slot = b.build().ok();
            }
        }
    }));
    rsx! {}
}
