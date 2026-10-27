//! Oberfläche (Dioxus + Komponenten im rust-ui-Stil).

pub mod art;
pub mod bigpicture;
pub mod components;
pub mod detail;
pub mod dialogs;
pub mod extensions;
pub mod free;
pub mod icons;
pub mod installer;
pub mod library;
pub mod manage;
pub mod media;
pub mod minecraft;
pub mod net;
pub mod opts;
pub mod settings;
pub mod state;
pub mod stats;
pub mod tools;
pub mod tray;

use crate::install::StartMode;
use crate::model::{layout, StoreStatus};
use crate::i18n::{t, tf};
use crate::{config, sys, theme};
use dioxus::desktop::wry::http::Response;
use dioxus::desktop::{use_asset_handler, window, LogicalSize};
use dioxus::prelude::*;
use icons::Icon;
use state::*;

/// Wurzel-Komponente: Theme, Titelleiste, Installer oder Launcher, Fenster, Toasts.
#[component]
pub fn Root() -> Element {
    let start = crate::START.get().cloned().unwrap_or(StartMode::Launcher);
    let mut mode = use_signal(|| start);

    let app = use_context_provider(|| App {
        settings: Signal::new({
            let mut s = config::load();
            // Oberflächen-Test: immer gleiche Ausgangslage
            if crate::uitest::active() {
                s = crate::model::Settings { welcome_done: true, animations: false, theme: "oled".into(), update_check: false, online_art: false, tray_icon: false, ..Default::default() };
            }
            s
        }),
        games: Signal::new(Vec::new()),
        steam: Signal::new(StoreStatus::default()),
        epic: Signal::new(StoreStatus::default()),
        roblox: Signal::new(StoreStatus::default()),
        stores: Signal::new(Vec::new()),
        meta_cache: Signal::new(crate::metadata::load_cache()),
        warnings: Signal::new(Vec::new()),
        scanning: Signal::new(false),
        loaded: Signal::new(false),
        page: Signal::new(Page::Library),
        filter: Signal::new(Filter::All),
        filter_arg: Signal::new(String::new()),
        genre: Signal::new(String::new()),
        dupes: Signal::new(Default::default()),
        tracking: Signal::new(Vec::new()),
        search: Signal::new(String::new()),
        mc_target: Signal::new("official".to_string()),
        selected: Signal::new(None),
        modal: Signal::new(None),
        toasts: Signal::new(Vec::new()),
        free: Signal::new(Vec::new()),
        free_state: Signal::new(LoadState::Idle),
        update_avail: Signal::new(None),
        update_state: Signal::new(UpdateState::Idle),
        epic_busy: Signal::new(false),
        epic_error: Signal::new(None),
        epic_progress: Signal::new((0, 0)),
    });

    // Lokale Bilder (Steam-Cache, Icons, eigene Cover) für das Fenster bereitstellen
    use_asset_handler("maz", move |req, responder| {
        let path = req.uri().path().to_string();
        if path.ends_with("/uitest") && crate::uitest::active() {
            crate::uitest::write_result(req.uri().query().unwrap_or(""));
            responder.respond(Response::builder().status(204).body(Vec::new()).unwrap_or_else(|_| Response::new(Vec::new())));
            return;
        }
        std::thread::spawn(move || {
            let found = sys::decode_file_url(&path).and_then(|p| std::fs::read(&p).ok().map(|b| (p, b)));
            let resp = match found {
                Some((p, bytes)) => Response::builder()
                    .status(200)
                    .header("Content-Type", sys::mime_for(&p))
                    .header("Cache-Control", "max-age=604800")
                    .body(bytes),
                None => Response::builder().status(404).body(Vec::new()),
            };
            responder.respond(resp.unwrap_or_else(|_| Response::new(Vec::new())));
        });
    });

    let s = app.settings.read();
    let is_launcher = matches!(*mode.read(), StartMode::Launcher);
    let authentic = if is_launcher && s.authentic_colors && *app.page.read() == Page::Library {
        if s.layout == layout::STEAM {
            "authentic-steam"
        } else if s.layout == layout::EPIC {
            "authentic-epic"
        } else {
            ""
        }
    } else {
        ""
    };
    // Farben: eigenes Design oder fertiges Design (bei Original-Farben übernimmt das CSS)
    let custom = s.custom_themes.iter().find(|c| c.id == s.theme).cloned();
    let data_theme = if custom.is_some() { "custom".to_string() } else { s.theme.clone() };
    let mut style = String::new();
    if authentic.is_empty() {
        if let Some(c) = &custom {
            style.push_str(&c.css_vars());
            style.push_str(&c.extra_style());
        } else if let Some(p) = theme::preset(&s.theme) {
            style.push_str(&p.css_vars());
        }
        if !s.accent.is_empty() && custom.is_none() {
            let a = &s.accent;
            let a2 = theme::preset(&s.theme).map(|p| p.accent2).unwrap_or_else(|| a.clone());
            style.push_str(&format!(
                "--primary:{a};--ring:{a};--glow:{a};--primary-foreground:{};--glow-fg:{};",
                theme::contrast_text(a),
                theme::gradient_text(a, &a2)
            ));
        }
    }
    let anim = if s.animations { "" } else { "no-anim" };
    let wallpaper = if s.wallpaper.is_empty() || !is_launcher {
        None
    } else {
        let url = sys::file_url(std::path::Path::new(&s.wallpaper));
        let opacity = (100 - s.wallpaper_dim.min(90) as u32) as f32 / 100.0;
        let blur = if s.wallpaper_blur { "filter: blur(14px); transform: scale(1.06);" } else { "" };
        Some((url, format!("opacity: {opacity}; {blur}")))
    };
    let lang = if s.language.is_empty() { "auto".to_string() } else { s.language.clone() };
    drop(s);
    // Schriftgröße (Barrierefreiheit): alle Größen hängen an der Grundschrift (rem)
    use_effect(move || {
        let fs = app.settings.read().font_scale.clamp(80, 175);
        let js = format!("document.documentElement.style.fontSize='{fs}%';");
        let _ = dioxus::desktop::window().webview.evaluate_script(&js);
    });

    rsx! {
        div {
            "data-theme": "{data_theme}",
            class: "relative h-screen w-screen flex flex-col bg-background text-foreground font-sans antialiased overflow-hidden {authentic} {anim}",
            style: "{style}",
            {wallpaper.map(|(url, st)| rsx! {
                div { class: "absolute inset-0 pointer-events-none overflow-hidden z-0",
                    img { src: "{url}", class: "size-full object-cover", style: "{st}", "draggable": "false" }
                }
            })}
            // Bei Sprachwechsel alles neu aufbauen (Schlüssel = Sprache)
            {std::iter::once(rsx! {
                div { key: "{lang}", class: "relative z-[1] flex-1 min-h-0 flex flex-col",
                    if *app.page.read() != Page::BigPicture {
                        TitleBar { nav: matches!(mode(), StartMode::Launcher) }
                    }
                    div { class: "flex-1 min-h-0 relative",
                        {match mode() {
                            StartMode::Launcher => rsx! { LauncherShell {} },
                            StartMode::Installer { existing } => rsx! {
                                installer::InstallerView {
                                    existing,
                                    on_portable: move |_| {
                                        let w = window();
                                        w.window.set_inner_size(LogicalSize::new(1320.0, 840.0));
                                        w.window.set_min_inner_size(Some(LogicalSize::new(980.0, 620.0)));
                                        mode.set(StartMode::Launcher);
                                    },
                                }
                            },
                        }}
                    }
                    dialogs::ModalHost {}
                    Toasts {}
                }
            })}
        }
    }
}

/// Controller-Modus ein-/ausschalten (Skript wird nur einmal geladen)
pub fn apply_controller(on: bool) {
    let js = include_str!("../../assets/controller.js").replace("__ON__", if on { "true" } else { "false" });
    let _ = window().webview.evaluate_script(&js);
}

#[component]
fn LauncherShell() -> Element {
    let app = use_app();
    // Suche leeren, sobald man zu einer anderen Seite oder Ansicht (Store, Sammlung …) wechselt
    use_effect(move || {
        let _ = (app.page.read().clone(), app.filter.read().clone(), app.filter_arg.read().clone());
        let mut s = app.search;
        if !s.peek().is_empty() {
            s.set(String::new());
        }
    });
    use_effect(move || {
        let on = app.settings.read().controller_mode;
        apply_controller(on);
    });
    // Absturz beim letzten Mal? Bericht anbieten (nur auf Wunsch senden)
    use_hook(move || {
        if let Some(text) = crate::crashreport::pending() {
            app.open(Modal::CrashReport(text));
        }
    });
    // `--play <id>`: Spiel starten, sobald die Bibliothek geladen ist
    let mut play_done = use_signal(|| false);
    use_effect(move || {
        if *app.loaded.read() && !*play_done.peek() {
            play_done.set(true);
            if let Some(id) = crate::PLAY_ON_START.get() {
                if app.game(id).is_some() {
                    play(app, id.clone());
                }
            }
        }
    });
    use_effect(move || {
        if !*app.loaded.peek() {
            rescan(app);
        }
        if !app.settings.peek().welcome_done {
            app.open(Modal::Welcome);
        }
        if crate::uitest::active() {
            return;
        }
        // Gratis-Spiele im Hintergrund laden (für die Zahl im Tab)
        net::load_free(app, false);
        net::sync_epic(app, false);
        net::update_loop(app);
    });
    // Steam Deck im Gaming-Modus: direkt Big Picture
    use_hook(move || {
        if crate::sys::gaming_mode() {
            app.go(Page::BigPicture);
        }
    });
    let page = *app.page.read();
    let body = match page {
        Page::Library => rsx! { library::LibraryPage {} },
        Page::Free => rsx! { free::FreePage {} },
        Page::Minecraft => rsx! { minecraft::MinecraftPage {} },
        Page::Stats => rsx! { stats::StatsPage {} },
        Page::BigPicture => rsx! { bigpicture::BigPicture {} },
        Page::Settings(_) => rsx! { settings::SettingsPage {} },
    };
    rsx! {
        tray::TrayHost {}
        {body}
    }
}

/// Haupt-Tab in der Titelleiste
#[component]
fn NavTab(#[props(into)] label: String, icon: &'static str, active: bool, badge: usize, #[props(default = false)] alert: bool, onclick: EventHandler<()>) -> Element {
    let cls = if active {
        "relative h-8 px-3 rounded-md flex items-center gap-2 text-[13px] font-medium bg-sidenav-accent text-foreground cursor-pointer"
    } else {
        "relative h-8 px-3 rounded-md flex items-center gap-2 text-[13px] text-sidenav-foreground/80 hover:bg-sidenav-accent/60 hover:text-foreground cursor-pointer"
    };
    let badge_cls = if alert { "bg-primary text-primary-foreground" } else { "bg-success/20 text-success" };
    rsx! {
        button { class: "{cls}", onclick: move |_| onclick.call(()),
            Icon { name: icon, class: "size-4" }
            "{label}"
            if badge > 0 {
                span { class: "min-w-5 h-5 px-1.5 rounded-full text-[11px] font-bold grid place-items-center {badge_cls}", "{badge}" }
            }
            if active {
                span { class: "absolute left-3 right-3 -bottom-[5px] h-0.5 rounded-full maz-gradient-bg" }
            }
        }
    }
}

/// Eigene Titelleiste (ziehen zum Verschieben, Doppelklick = maximieren) + Haupt-Tabs
#[component]
fn TitleBar(#[props(default = false)] nav: bool) -> Element {
    let app = use_app();
    let page = *app.page.read();
    let filter = *app.filter.read();
    let lib_active = page == Page::Library && filter != Filter::Installed;
    let inst_active = page == Page::Library && filter == Filter::Installed;
    let (free_badge, installed) = if nav {
        (free::unclaimed_count(app), library::count_for(app, Filter::Installed))
    } else {
        (0, 0)
    };
    // Minecraft-Tab nur, wenn Minecraft oder NoRisk installiert ist
    let has_mc = nav && app.games.read().iter().any(|g| g.source == crate::model::Source::Minecraft);
    let go_lib = move |f: Filter| {
        let mut fl = app.filter;
        fl.set(f);
        app.go(Page::Library);
    };
    rsx! {
        div {
            class: "h-11 shrink-0 flex items-center bg-sidenav/90 border-b border-border/60 relative z-40",
            onmousedown: move |_| window().drag(),
            ondoubleclick: move |_| window().toggle_maximized(),
            // macOS: Platz für die Ampel-Knöpfe links oben
            if cfg!(target_os = "macos") {
                div { class: "w-[72px] shrink-0" }
            }
            library::TitleBrand {}
            if nav {
                div { class: "flex items-center gap-1 ml-6",
                    onmousedown: move |e| e.stop_propagation(),
                    ondoubleclick: move |e| e.stop_propagation(),
                    NavTab { label: t("nav.library"), icon: "library", active: lib_active, badge: 0, onclick: move |_| go_lib(Filter::All) }
                    NavTab { label: t("filter.installed"), icon: "drive", active: inst_active, badge: installed, onclick: move |_| go_lib(Filter::Installed) }
                    NavTab { label: t("free.title"), icon: "gift", active: page == Page::Free, badge: free_badge, onclick: move |_| app.go(Page::Free) }
                    NavTab { label: t("stats.title"), icon: "chart", active: page == Page::Stats, badge: 0, onclick: move |_| app.go(Page::Stats) }
                    if has_mc {
                        NavTab { label: "Minecraft", icon: "cube", active: page == Page::Minecraft, badge: 0, onclick: move |_| app.go(Page::Minecraft) }
                    }
                }
            }
            div { class: "flex-1" }
            if nav {
                button {
                    class: "mr-1 h-7 px-2.5 rounded-md text-[12px] font-medium text-muted-foreground hover:text-foreground hover:bg-white/10 flex items-center gap-1.5 cursor-pointer",
                    title: t("bp.tip"),
                    "aria-label": t("bp.tip"),
                    onmousedown: move |e| e.stop_propagation(),
                    onclick: move |_| app.go(Page::BigPicture),
                    Icon { name: "monitor", class: "size-4" }
                }
                button {
                    class: "mr-2 h-7 px-2.5 rounded-md text-[12px] font-medium text-muted-foreground hover:text-foreground hover:bg-white/10 flex items-center gap-1.5 cursor-pointer",
                    "aria-label": t("random.tip"),
                    title: t("random.tip"),
                    onmousedown: move |e| e.stop_propagation(),
                    onclick: move |_| app.open(Modal::Random),
                    Icon { name: "dice", class: "size-4" }
                    span { class: "sr-only", {t("random.button")} }
                }
            }
            if let Some(v) = app.update_avail.read().as_ref().map(|r| r.version.clone()) {
                button {
                    class: "mr-2 h-7 px-3 rounded-full text-[12px] font-semibold maz-gradient-bg text-glow-fg flex items-center gap-1.5 shadow cursor-pointer animate-pop-in",
                    onmousedown: move |e| e.stop_propagation(),
                    onclick: move |_| app.open(Modal::Update),
                    Icon { name: "download", class: "size-3.5" }
                    {tf("update.pill", &[("version", &v)])}
                }
            }
            if !cfg!(target_os = "macos") {
            div {
                class: "flex h-full",
                onmousedown: move |e| e.stop_propagation(),
                ondoubleclick: move |e| e.stop_propagation(),
                button {
                    class: "titlebar-btn w-12 h-full grid place-items-center text-muted-foreground hover:bg-white/10 hover:text-foreground cursor-pointer",
                    title: t("win.minimize"),
                    onclick: move |_| window().window.set_minimized(true),
                    Icon { name: "minus", class: "size-4" }
                }
                button {
                    class: "titlebar-btn w-12 h-full grid place-items-center text-muted-foreground hover:bg-white/10 hover:text-foreground cursor-pointer",
                    title: t("win.maximize"),
                    onclick: move |_| window().toggle_maximized(),
                    Icon { name: "square", class: "size-3.5" }
                }
                button {
                    class: "titlebar-btn is-close w-12 h-full grid place-items-center text-muted-foreground hover:text-white cursor-pointer",
                    title: t("common.close"),
                    onclick: move |_| window().close(),
                    Icon { name: "x", class: "size-4" }
                }
            }
            }
        }
    }
}

#[component]
fn Toasts() -> Element {
    let app = use_app();
    let toasts = app.toasts.read().clone();
    rsx! {
        div { class: "fixed bottom-5 right-5 z-[60] flex flex-col gap-2 pointer-events-none",
            {toasts.into_iter().map(|t| {
                let id = t.id;
                let text = t.text.clone();
                let icon = t.icon;
                rsx! {
                    div { key: "{id}", class: "pointer-events-auto relative overflow-hidden flex items-center gap-3 rounded-xl border border-border bg-popover/95 backdrop-blur pl-4 pr-2 py-3 text-sm shadow-2xl shadow-black/40 animate-slide-up max-w-sm",
                        div { class: "size-8 rounded-lg maz-gradient-bg text-glow-fg grid place-items-center shrink-0",
                            Icon { name: icon, class: "size-4" }
                        }
                        span { class: "flex-1", "{text}" }
                        button {
                            class: "p-1.5 rounded-md text-muted-foreground hover:text-foreground hover:bg-accent cursor-pointer",
                            onclick: move |_| {
                                let mut ts = app.toasts;
                                ts.write().retain(|x| x.id != id);
                            },
                            Icon { name: "x", class: "size-3.5" }
                        }
                        // Fortschrittsbalken: verschwindet in 5 Sekunden
                        div { class: "absolute left-0 bottom-0 h-0.5 maz-gradient-bg toast-progress" }
                    }
                }
            })}
        }
    }
}
