//! Einstellungen: Design (inkl. Design-Editor), Sprache, Verbindungen,
//! eigene Spiele, Allgemein, Über.

use super::components::*;
use super::icons::{Icon, MazLogo};
use super::state::*;
use crate::i18n::{self, t, tf};
use crate::model::{default_checklist, layout, Source, StoreStatus};
use crate::theme::{self, CustomTheme, PRESETS};
use crate::{install, sys};
use dioxus::prelude::*;

#[component]
pub fn SettingsPage() -> Element {
    let app = use_app();
    let tab = match *app.page.read() {
        Page::Settings(t) => t,
        _ => SettingsTab::Design,
    };
    let tabs = [
        (SettingsTab::Design, t("set.tab.design"), "palette"),
        (SettingsTab::Language, t("set.tab.language"), "globe"),
        (SettingsTab::Connections, t("set.tab.connections"), "plug"),
        (SettingsTab::Custom, t("set.tab.custom"), "rocket"),
        (SettingsTab::Extensions, t("set.tab.extensions"), "package"),
        (SettingsTab::General, t("set.tab.general"), "settings"),
        (SettingsTab::About, t("set.tab.about"), "info"),
    ];
    rsx! {
        div { class: "flex h-full min-h-0 maz-ambient",
            aside { class: "w-64 shrink-0 border-r border-border/60 bg-sidenav/60 p-4 flex flex-col gap-1",
                Button { variant: ButtonVariant::Ghost, class: "justify-start mb-4 -ml-1", onclick: move |_| app.go(Page::Library),
                    Icon { name: "arrow-left", class: "size-4" }
                    {t("set.back")}
                }
                div { class: "px-3 pb-2 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground", {t("nav.settings")} }
                {tabs.into_iter().map(|(tb, label, icon)| {
                    let active = tb == tab;
                    let cls = if active {
                        "w-full flex items-center gap-3 px-3 h-10 rounded-lg text-sm font-medium bg-sidenav-accent text-foreground cursor-pointer"
                    } else {
                        "w-full flex items-center gap-3 px-3 h-10 rounded-lg text-sm text-sidenav-foreground/80 hover:bg-sidenav-accent/60 hover:text-foreground cursor-pointer"
                    };
                    rsx! {
                        button { key: "{label}", class: "{cls}", onclick: move |_| app.go(Page::Settings(tb)),
                            Icon { name: icon, class: "size-4" }
                            "{label}"
                        }
                    }
                })}
            }
            main { class: "flex-1 min-w-0 overflow-y-auto",
                div { class: "max-w-3xl mx-auto px-10 py-8 animate-fade-in",
                    {match tab {
                        SettingsTab::Design => rsx! { DesignTab {} },
                        SettingsTab::Language => rsx! { LanguageTab {} },
                        SettingsTab::Connections => rsx! { ConnectionsTab {} },
                        SettingsTab::Custom => rsx! { CustomTab {} },
                        SettingsTab::Extensions => rsx! { super::extensions::ExtensionsTab {} },
                        SettingsTab::General => rsx! { GeneralTab {} },
                        SettingsTab::About => rsx! { AboutTab {} },
                    }}
                }
            }
        }
    }
}

#[component]
pub fn Section(#[props(into)] title: String, #[props(into, default)] description: String, children: Element) -> Element {
    rsx! {
        Card { class: "mb-6 bg-card/80 backdrop-blur",
            CardHeader {
                CardTitle { "{title}" }
                if !description.is_empty() {
                    CardDescription { "{description}" }
                }
            }
            CardContent { {children} }
        }
    }
}

#[component]
pub fn PageTitle(#[props(into)] title: String, #[props(into)] sub: String) -> Element {
    rsx! {
        h1 { class: "text-3xl font-bold font-display mb-2", "{title}" }
        p { class: "text-muted-foreground mb-8", "{sub}" }
    }
}

// ----------------------------------------------------------------------
// Sprache
// ----------------------------------------------------------------------

/// Sprachauswahl (auch im Willkommensfenster)
#[component]
pub fn LanguagePicker() -> Element {
    let app = use_app();
    let current = app.settings.read().language.clone();
    let mut options: Vec<(String, String)> = vec![("auto".to_string(), t("lang.auto"))];
    options.extend(i18n::available());
    rsx! {
        div { class: "grid grid-cols-3 gap-2",
            {options.into_iter().map(|(code, name)| {
                let active = current == code || (current.is_empty() && code == "auto");
                let cls = if active {
                    "h-11 px-3 rounded-lg text-sm font-medium bg-primary text-primary-foreground cursor-pointer flex items-center gap-2"
                } else {
                    "h-11 px-3 rounded-lg text-sm font-medium bg-muted text-foreground hover:bg-accent cursor-pointer flex items-center gap-2"
                };
                let c2 = code.clone();
                rsx! {
                    button { key: "{code}", class: "{cls}",
                        onclick: move |_| {
                            let c = c2.clone();
                            i18n::set_language(&c);
                            app.update(|s| s.language = c);
                        },
                        if active {
                            Icon { name: "check", class: "size-4" }
                        }
                        span { class: "truncate", "{name}" }
                    }
                }
            })}
        }
    }
}

/// Kompakte Sprachauswahl (Dropdown), z. B. im Installer
#[component]
pub fn LanguageSelect() -> Element {
    let app = use_app();
    let current = app.settings.read().language.clone();
    let mut options: Vec<(String, String)> = vec![("auto".to_string(), t("lang.auto"))];
    options.extend(i18n::available());
    rsx! {
        div { class: "relative flex items-center gap-2 text-sm",
            Icon { name: "globe", class: "size-4 text-muted-foreground" }
            select {
                class: "h-9 rounded-md border border-input bg-background/60 px-2 text-sm outline-none cursor-pointer",
                onchange: move |e| {
                    let c = e.value();
                    i18n::set_language(&c);
                    app.update(|s| s.language = c);
                },
                {options.into_iter().map(|(code, name)| {
                    let sel = current == code || (current.is_empty() && code == "auto");
                    rsx! { option { key: "{code}", value: "{code}", selected: sel, "{name}" } }
                })}
            }
        }
    }
}

#[component]
fn LanguageTab() -> Element {
    let folder = i18n::user_lang_folder();
    let folder_s = folder.to_string_lossy().to_string();
    let app = use_app();
    rsx! {
        PageTitle { title: t("set.tab.language"), sub: t("lang.sub") }
        Section { title: t("lang.choose"),
            LanguagePicker {}
        }
        Section { title: t("lang.own"), description: t("lang.own_text"),
            div { class: "flex flex-wrap gap-2",
                Button { variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        let dir = i18n::user_lang_folder();
                        let _ = std::fs::create_dir_all(&dir);
                        if let Some(text) = i18n::builtin("en") {
                            let _ = std::fs::write(dir.join("template-en.txt"), text);
                        }
                        app.toast(&t("lang.template_saved"), "check-circle");
                        sys::open_folder(&dir.to_string_lossy());
                    },
                    Icon { name: "download", class: "size-4" }
                    {t("lang.template")}
                }
                Button { variant: ButtonVariant::Ghost,
                    onclick: move |_| {
                        let _ = std::fs::create_dir_all(&folder);
                        sys::open_folder(&folder_s);
                    },
                    Icon { name: "folder", class: "size-4" }
                    {t("lang.folder")}
                }
            }
        }
    }
}

// ----------------------------------------------------------------------
// Design
// ----------------------------------------------------------------------

/// Drei Vorschau-Karten: MAZ / Steam / Epic
#[component]
pub fn LayoutPicker() -> Element {
    let app = use_app();
    let current = app.settings.read().layout.clone();
    let options = [
        (layout::MAZ, t("layout.maz"), t("layout.maz_desc")),
        (layout::STEAM, t("layout.steam"), t("layout.steam_desc")),
        (layout::EPIC, t("layout.epic"), t("layout.epic_desc")),
    ];
    rsx! {
        div { class: "grid grid-cols-3 gap-4",
            {options.into_iter().map(|(id, title, desc)| {
                let active = current == id;
                let ring = if active { "ring-2 ring-primary" } else { "ring-1 ring-border hover:ring-primary/50" };
                rsx! {
                    button {
                        key: "{id}",
                        class: "text-left rounded-xl overflow-hidden bg-muted cursor-pointer transition-all {ring}",
                        onclick: move |_| app.update(|s| s.layout = id.to_string()),
                        LayoutPreview { kind: id }
                        div { class: "p-3",
                            div { class: "flex items-center gap-2 text-sm font-semibold",
                                if active {
                                    Icon { name: "check-circle", class: "size-4 text-primary" }
                                }
                                "{title}"
                            }
                            div { class: "text-xs text-muted-foreground mt-1", "{desc}" }
                        }
                    }
                }
            })}
        }
    }
}

#[component]
fn LayoutPreview(kind: &'static str) -> Element {
    match kind {
        "steam" => rsx! {
            div { class: "h-28 flex bg-[#1b2838]",
                div { class: "w-1/3 bg-[#171d25] p-1.5 space-y-1",
                    for i in 0..7 { div { key: "{i}", class: "h-1.5 rounded-sm bg-white/15" } }
                }
                div { class: "flex-1 p-2",
                    div { class: "h-10 rounded-sm bg-linear-to-r from-[#2a475e] to-[#66c0f4]/40" }
                    div { class: "mt-2 h-3 w-12 rounded-sm bg-[#75b022]" }
                    div { class: "mt-2 grid grid-cols-4 gap-1",
                        for i in 0..4 { div { key: "{i}", class: "aspect-[2/3] rounded-[2px] bg-white/10" } }
                    }
                }
            }
        },
        "epic" => rsx! {
            div { class: "h-28 flex bg-[#101014]",
                div { class: "w-1/4 bg-[#18181c] p-1.5 space-y-1.5",
                    for i in 0..3 { div { key: "{i}", class: "h-2 rounded bg-white/15" } }
                }
                div { class: "flex-1 p-2",
                    div { class: "h-2.5 w-16 rounded bg-white/70 mb-2" }
                    div { class: "flex gap-1 mb-2",
                        for i in 0..4 { div { key: "{i}", class: "h-1.5 w-6 rounded-full bg-white/20" } }
                    }
                    div { class: "grid grid-cols-4 gap-1.5",
                        for i in 0..4 { div { key: "{i}", class: "aspect-[3/4] rounded bg-white/15" } }
                    }
                }
            }
        },
        _ => rsx! {
            div { class: "h-28 flex bg-background",
                div { class: "w-1/4 bg-sidenav p-1.5 space-y-1",
                    div { class: "h-2 rounded maz-gradient-bg mb-2" }
                    for i in 0..5 { div { key: "{i}", class: "h-1.5 rounded bg-foreground/15" } }
                }
                div { class: "flex-1 p-2",
                    div { class: "h-9 rounded-md maz-gradient-bg opacity-80" }
                    div { class: "mt-2 grid grid-cols-5 gap-1",
                        for i in 0..5 { div { key: "{i}", class: "aspect-[2/3] rounded bg-foreground/15" } }
                    }
                }
            }
        },
    }
}

/// Farbvorschau eines Designs (Kachel)
#[component]
fn ThemeTile(theme: CustomTheme, #[props(into)] name: String, active: bool, onselect: EventHandler<()>, #[props(optional)] onedit: Option<EventHandler<()>>) -> Element {
    let ring = if active { "ring-2 ring-primary" } else { "ring-1 ring-border hover:ring-primary/50" };
    let bg = format!(
        "background: linear-gradient(135deg, {} 0%, {} 55%, {} 150%);",
        theme.bg, theme.card, theme.accent
    );
    let side = format!("background: {};", theme.sidebar);
    let acc = format!("background: linear-gradient(90deg, {}, {});", theme.accent, theme.accent2);
    let txt = format!("background: {};", theme.text);
    let mtx = format!("background: {};", theme.muted);
    let has_edit = onedit.is_some();
    rsx! {
        div { class: "group relative rounded-xl overflow-hidden cursor-pointer text-left transition-all {ring}",
            onclick: move |_| onselect.call(()),
            div { class: "h-16 relative flex", style: "{bg}",
                div { class: "w-1/4 h-full opacity-90", style: "{side}" }
                div { class: "flex-1 p-2 flex flex-col gap-1.5",
                    div { class: "h-1.5 w-3/4 rounded-full", style: "{txt}" }
                    div { class: "h-1.5 w-1/2 rounded-full opacity-70", style: "{mtx}" }
                    div { class: "mt-auto h-3 w-10 rounded-full", style: "{acc}" }
                }
                if active {
                    div { class: "absolute top-2 right-2 size-5 rounded-full bg-white text-black grid place-items-center shadow",
                        Icon { name: "check", class: "size-3" }
                    }
                }
            }
            div { class: "px-3 py-2 text-xs font-medium bg-muted flex items-center gap-1",
                span { class: "truncate flex-1", "{name}" }
                if has_edit {
                    button { class: "p-1 -m-1 rounded text-muted-foreground hover:text-foreground hover:bg-accent cursor-pointer",
                        title: t("theme.edit"),
                        onclick: move |e| {
                            e.stop_propagation();
                            if let Some(h) = &onedit {
                                h.call(());
                            }
                        },
                        Icon { name: "pencil", class: "size-3.5" }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ColorField {
    Bg,
    Card,
    Sidebar,
    Text,
    Muted,
    Accent,
    Accent2,
    Border,
}

fn set_field(th: &mut CustomTheme, f: ColorField, v: String) {
    match f {
        ColorField::Bg => th.bg = v,
        ColorField::Card => th.card = v,
        ColorField::Sidebar => th.sidebar = v,
        ColorField::Text => th.text = v,
        ColorField::Muted => th.muted = v,
        ColorField::Accent => th.accent = v,
        ColorField::Accent2 => th.accent2 = v,
        ColorField::Border => th.border = v,
    }
}

fn update_theme(app: App, id: &str, f: impl FnOnce(&mut CustomTheme)) {
    let id = id.to_string();
    app.update(|s| {
        if let Some(th) = s.custom_themes.iter_mut().find(|c| c.id == id) {
            f(th);
        }
    });
}

/// Aktuelle Farben (fertiges oder eigenes Design) als Startwert für ein neues Design
fn current_colors(app: App) -> CustomTheme {
    let s = app.settings.peek();
    s.custom_themes
        .iter()
        .find(|c| c.id == s.theme)
        .cloned()
        .or_else(|| theme::preset(&s.theme))
        .unwrap_or_default()
}

fn new_custom_id() -> String {
    let n = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("custom:{n:x}")
}

#[component]
fn ColorRow(#[props(into)] label: String, value: String, onchange: EventHandler<String>) -> Element {
    let hex = value.clone();
    rsx! {
        div { class: "flex items-center gap-3 py-2",
            input {
                r#type: "color",
                class: "h-9 w-12 shrink-0 rounded-md border border-input bg-transparent cursor-pointer",
                value: "{value}",
                oninput: move |e| onchange.call(e.value()),
            }
            span { class: "flex-1 text-sm", "{label}" }
            Input {
                class: "w-28 h-9 font-mono text-xs",
                value: hex,
                oninput: move |v: String| {
                    if theme::parse_hex(&v).is_some() {
                        onchange.call(v);
                    }
                },
            }
        }
    }
}

/// Design aus einer .maztheme.json-Datei laden
fn import_theme_file(app: App) {
    let Some(path) = rfd::FileDialog::new().set_title(t("theme.import_file")).add_filter("OG Theme", &["json", "ogtheme", "maztheme"]).pick_file() else {
        return;
    };
    // Design-Dateien sind klein – alles über 256 KB ist sicher keine
    let text = std::fs::metadata(&path)
        .ok()
        .filter(|m| m.len() < 256 * 1024)
        .and_then(|_| std::fs::read_to_string(&path).ok());
    match text.and_then(|t| CustomTheme::from_file(&t)) {
        Some(mut c) => {
            c.id = new_custom_id();
            if c.name.is_empty() {
                c.name = t("theme.imported");
            }
            let nid = c.id.clone();
            let lay = c.layout.clone();
            app.update(|s| {
                s.custom_themes.push(c);
                s.theme = nid;
                if !lay.is_empty() {
                    s.layout = lay;
                }
            });
            app.toast(&t("theme.import_ok"), "check-circle");
        }
        None => app.toast(&t("theme.import_bad"), "alert"),
    }
}

#[component]
fn ThemeEditor(id: String, on_done: EventHandler<()>) -> Element {
    let app = use_app();
    let Some(th) = app.settings.read().custom_themes.iter().find(|c| c.id == id).cloned() else {
        return rsx! {};
    };
    let fields = [
        (ColorField::Bg, t("theme.c.bg"), th.bg.clone()),
        (ColorField::Card, t("theme.c.card"), th.card.clone()),
        (ColorField::Sidebar, t("theme.c.sidebar"), th.sidebar.clone()),
        (ColorField::Text, t("theme.c.text"), th.text.clone()),
        (ColorField::Muted, t("theme.c.muted"), th.muted.clone()),
        (ColorField::Accent, t("theme.c.accent"), th.accent.clone()),
        (ColorField::Accent2, t("theme.c.accent2"), th.accent2.clone()),
        (ColorField::Border, t("theme.c.border"), th.border.clone()),
    ];
    let radius = match th.radius {
        0..=3 => "0",
        4..=9 => "6",
        10..=15 => "12",
        _ => "20",
    }
    .to_string();
    let (id_n, id_r, id_dup, id_del, id_code) = (id.clone(), id.clone(), id.clone(), id.clone(), id.clone());
    let (id_f, id_d, id_l, id_file) = (id.clone(), id.clone(), id.clone(), id.clone());
    let name = th.name.clone();
    let font = if th.font.is_empty() { "standard".to_string() } else { th.font.clone() };
    let density = th.density.clamp(1, 3).to_string();
    let th_layout = th.layout.clone();
    rsx! {
        Card { class: "mb-6 border-primary/50 ring-1 ring-primary/30 animate-slide-up",
            CardHeader {
                div { class: "flex items-center gap-2",
                    Icon { name: "palette", class: "size-5 text-primary" }
                    CardTitle { {t("theme.editor")} }
                }
                CardDescription { {t("theme.editor_text")} }
            }
            CardContent {
                div { class: "grid gap-2 mb-4",
                    Label { {t("theme.name")} }
                    Input { value: name, oninput: move |v: String| update_theme(app, &id_n, |th| th.name = v) }
                }
                div { class: "grid grid-cols-2 gap-x-6 divide-border",
                    {fields.into_iter().map(|(f, label, value)| {
                        let tid = id.clone();
                        let key = label.clone();
                        rsx! {
                            ColorRow { key: "{key}", label, value,
                                onchange: move |v: String| update_theme(app, &tid, |th| set_field(th, f, v)) }
                        }
                    })}
                }
                div { class: "flex items-center justify-between mt-4 py-2",
                    span { class: "text-sm", {t("theme.radius")} }
                    Segmented {
                        options: vec![
                            ("0".to_string(), t("theme.r0")),
                            ("6".to_string(), t("theme.r1")),
                            ("12".to_string(), t("theme.r2")),
                            ("20".to_string(), t("theme.r3")),
                        ],
                        value: radius,
                        onchange: move |v: String| {
                            let r = v.parse::<u8>().unwrap_or(12);
                            update_theme(app, &id_r, |th| th.radius = r);
                        },
                    }
                }
                div { class: "flex items-center justify-between py-2",
                    span { class: "text-sm", {t("theme.font")} }
                    select {
                        class: "h-9 rounded-md border border-input bg-background/60 px-3 text-sm outline-none cursor-pointer",
                        onchange: move |e| {
                            let v = e.value();
                            update_theme(app, &id_f, |th| th.font = if v == "standard" { String::new() } else { v });
                        },
                        {theme::FONTS.iter().map(|(k, _)| {
                            let k = k.to_string();
                            let label = t(&format!("theme.font.{k}"));
                            rsx! { option { key: "{k}", value: "{k}", selected: font == k, "{label}" } }
                        })}
                    }
                }
                div { class: "flex items-center justify-between py-2",
                    span { class: "text-sm", {t("theme.density")} }
                    Segmented {
                        options: vec![
                            ("1".to_string(), t("theme.d1")),
                            ("2".to_string(), t("theme.d2")),
                            ("3".to_string(), t("theme.d3")),
                        ],
                        value: density,
                        onchange: move |v: String| {
                            let d = v.parse::<u8>().unwrap_or(2);
                            update_theme(app, &id_d, |th| th.density = d);
                        },
                    }
                }
                div { class: "flex items-center justify-between py-2",
                    span { class: "text-sm", {t("theme.with_layout")} }
                    Segmented {
                        options: vec![
                            ("".to_string(), t("theme.layout_keep")),
                            ("maz".to_string(), "OG".to_string()),
                            ("steam".to_string(), "Steam".to_string()),
                            ("epic".to_string(), "Epic".to_string()),
                        ],
                        value: th_layout,
                        onchange: move |v: String| {
                            update_theme(app, &id_l, |th| th.layout = v.clone());
                            if !v.is_empty() {
                                app.update(|s| s.layout = v);
                            }
                        },
                    }
                }
                div { class: "flex flex-wrap gap-2 mt-5",
                    Button { variant: ButtonVariant::Glow, onclick: move |_| on_done.call(()),
                        Icon { name: "check", class: "size-4" }
                        {t("theme.done")}
                    }
                    Button { variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            let code = app.settings.peek().custom_themes.iter().find(|c| c.id == id_code).map(|c| c.to_code());
                            if let Some(code) = code {
                                if sys::copy_to_clipboard(&code) {
                                    app.toast(&t("theme.copied"), "copy");
                                }
                            }
                        },
                        Icon { name: "copy", class: "size-4" }
                        {t("theme.share")}
                    }
                    Button { variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            let th = app.settings.peek().custom_themes.iter().find(|c| c.id == id_file).cloned();
                            if let Some(th) = th {
                                let file = format!("{}.ogtheme.json", sys::safe_file_name(if th.name.is_empty() { "design" } else { &th.name }));
                                if let Some(path) = rfd::FileDialog::new().set_title(t("theme.export")).set_file_name(&file).add_filter("OG Theme", &["json"]).save_file() {
                                    match std::fs::write(&path, th.to_file()) {
                                        Ok(()) => app.toast(&t("theme.exported"), "check-circle"),
                                        Err(e) => app.toast(&tf("toast.save_failed", &[("error", &e.to_string())]), "alert"),
                                    }
                                }
                            }
                        },
                        Icon { name: "download", class: "size-4" }
                        {t("theme.export")}
                    }
                    Button { variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            let src = app.settings.peek().custom_themes.iter().find(|c| c.id == id_dup).cloned();
                            if let Some(mut c) = src {
                                c.id = new_custom_id();
                                c.name = tf("theme.copy_of", &[("name", &c.name)]);
                                let nid = c.id.clone();
                                app.update(|s| {
                                    s.custom_themes.push(c);
                                    s.theme = nid;
                                });
                            }
                        },
                        Icon { name: "layers", class: "size-4" }
                        {t("theme.duplicate")}
                    }
                    Button { variant: ButtonVariant::Ghost, class: "text-destructive",
                        onclick: move |_| {
                            let did = id_del.clone();
                            app.update(|s| {
                                s.custom_themes.retain(|c| c.id != did);
                                if s.theme == did {
                                    s.theme = "maz".into();
                                }
                            });
                            on_done.call(());
                        },
                        Icon { name: "trash", class: "size-4" }
                        {t("theme.delete")}
                    }
                }
            }
        }
    }
}

#[component]
fn DesignTab() -> Element {
    let app = use_app();
    let s = app.settings.read().clone();
    let mut editing = use_signal(|| None::<String>);
    let mut import_code = use_signal(String::new);
    let accent_value = if s.accent.is_empty() { "#8b5cf6".to_string() } else { s.accent.clone() };
    let has_accent = !s.accent.is_empty();
    let size = s.card_size.to_string();
    let wallpaper = s.wallpaper.clone();
    let has_wallpaper = !wallpaper.is_empty();
    let dim = s.wallpaper_dim;
    let presets: Vec<CustomTheme> = PRESETS.iter().filter_map(|(id, _)| theme::preset(id)).collect();
    let customs = s.custom_themes.clone();
    let editing_id = editing();
    let current = s.theme.clone();
    rsx! {
        PageTitle { title: t("set.tab.design"), sub: t("design.sub") }

        Section { title: t("design.layout"), description: t("design.layout_text"),
            LayoutPicker {}
            div { class: "mt-4",
                SwitchRow {
                    title: t("design.authentic"),
                    description: t("design.authentic_text"),
                    checked: s.authentic_colors,
                    onchange: move |v: bool| app.update(|s| s.authentic_colors = v),
                }
            }
        }

        {editing_id.map(|id| rsx! {
            ThemeEditor { id, on_done: move |_| editing.set(None) }
        })}

        Section { title: t("design.my_themes"), description: t("design.my_themes_text"),
            div { class: "grid grid-cols-4 gap-3",
                {customs.into_iter().map(|c| {
                    let active = current == c.id;
                    let (id1, id2) = (c.id.clone(), c.id.clone());
                    let name = if c.name.is_empty() { t("theme.unnamed") } else { c.name.clone() };
                    let key = c.id.clone();
                    rsx! {
                        ThemeTile { key: "{key}", theme: c, name, active,
                            onselect: move |_| { let i = id1.clone(); app.update(|s| s.theme = i); },
                            onedit: move |_| {
                                let i = id2.clone();
                                app.update(|s| s.theme = i.clone());
                                editing.set(Some(i));
                            },
                        }
                    }
                })}
                button {
                    class: "rounded-xl border-2 border-dashed border-border hover:border-primary hover:text-primary text-muted-foreground min-h-[100px] flex flex-col items-center justify-center gap-2 text-xs font-medium cursor-pointer transition-colors",
                    onclick: move |_| {
                        let mut c = current_colors(app);
                        c.id = new_custom_id();
                        let n = app.settings.peek().custom_themes.len() + 1;
                        c.name = tf("theme.new_name", &[("n", &n.to_string())]);
                        let nid = c.id.clone();
                        app.update(|s| {
                            s.custom_themes.push(c);
                            s.theme = nid.clone();
                        });
                        editing.set(Some(nid));
                    },
                    Icon { name: "plus", class: "size-6" }
                    {t("theme.new")}
                }
            }
            div { class: "flex gap-2 mt-5",
                Input { class: "font-mono text-xs", placeholder: t("theme.import_ph"), value: import_code(), oninput: move |v: String| import_code.set(v) }
                Button { variant: ButtonVariant::Secondary, class: "h-10",
                    onclick: move |_| {
                        let code = import_code.peek().clone();
                        match CustomTheme::from_code(&code) {
                            Some(mut c) => {
                                c.id = new_custom_id();
                                if c.name.is_empty() {
                                    c.name = t("theme.imported");
                                }
                                let nid = c.id.clone();
                                app.update(|s| {
                                    s.custom_themes.push(c);
                                    s.theme = nid;
                                });
                                import_code.set(String::new());
                                app.toast(&t("theme.import_ok"), "check-circle");
                            }
                            None => app.toast(&t("theme.import_bad"), "alert"),
                        }
                    },
                    Icon { name: "download", class: "size-4" }
                    {t("theme.import")}
                }
                Button { variant: ButtonVariant::Outline, class: "h-10",
                    onclick: move |_| import_theme_file(app),
                    Icon { name: "folder", class: "size-4" }
                    {t("theme.import_file")}
                }
            }
        }

        Section { title: t("design.presets"), description: t("design.presets_text"),
            div { class: "grid grid-cols-4 gap-3",
                {presets.into_iter().map(|p| {
                    let active = current == p.id;
                    let id = p.id.clone();
                    let name = t(&format!("theme.{}", p.id));
                    let key = p.id.clone();
                    rsx! {
                        ThemeTile { key: "{key}", theme: p, name, active,
                            onselect: move |_| { let i = id.clone(); app.update(|s| s.theme = i); },
                        }
                    }
                })}
            }
            div { class: "mt-5 flex items-center justify-between gap-4 rounded-xl border border-border px-4 py-3",
                div {
                    div { class: "text-sm font-medium", {t("design.accent")} }
                    div { class: "text-xs text-muted-foreground mt-1", {t("design.accent_text")} }
                }
                div { class: "flex items-center gap-2",
                    input {
                        r#type: "color",
                        class: "h-9 w-14 rounded-md border border-input bg-transparent cursor-pointer",
                        value: "{accent_value}",
                        oninput: move |e| { let v = e.value(); app.update(|s| s.accent = v); },
                    }
                    if has_accent {
                        Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, onclick: move |_| app.update(|s| s.accent.clear()), {t("common.reset")} }
                    }
                }
            }
        }

        Section { title: t("design.wallpaper"), description: t("design.wallpaper_text"),
            div { class: "flex flex-wrap items-center gap-2",
                Button { variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        if let Some(p) = pick_image_file() {
                            app.update(|s| s.wallpaper = p);
                        }
                    },
                    Icon { name: "image", class: "size-4" }
                    {t("design.wallpaper_pick")}
                }
                if has_wallpaper {
                    Button { variant: ButtonVariant::Ghost, onclick: move |_| app.update(|s| s.wallpaper.clear()),
                        Icon { name: "x", class: "size-4" }
                        {t("design.wallpaper_remove")}
                    }
                    span { class: "text-xs text-muted-foreground font-mono truncate max-w-xs", title: "{wallpaper}", "{wallpaper}" }
                }
            }
            if has_wallpaper {
                div { class: "flex items-center gap-4 mt-4",
                    span { class: "text-sm w-40", {t("design.wallpaper_dim")} }
                    input {
                        r#type: "range",
                        class: "flex-1 accent-[var(--primary)] cursor-pointer",
                        min: "0",
                        max: "90",
                        step: "5",
                        value: "{dim}",
                        oninput: move |e| {
                            let v = e.value().parse::<u8>().unwrap_or(60).min(90);
                            app.update(|s| s.wallpaper_dim = v);
                        },
                    }
                    span { class: "text-sm tabular-nums w-10 text-right", "{dim} %" }
                }
                SwitchRow {
                    title: t("design.wallpaper_blur"),
                    checked: s.wallpaper_blur,
                    onchange: move |v: bool| app.update(|s| s.wallpaper_blur = v),
                }
            }
        }

        Section { title: t("design.display"),
            div { class: "flex items-center justify-between py-3",
                div {
                    div { class: "text-sm font-medium", {t("design.card_size")} }
                    div { class: "text-xs text-muted-foreground mt-1", {t("design.card_size_text")} }
                }
                Segmented {
                    options: vec![("1".to_string(), t("design.small")), ("2".to_string(), t("design.medium")), ("3".to_string(), t("design.large"))],
                    value: size,
                    onchange: move |v: String| { let n = v.parse::<u8>().unwrap_or(2); app.update(|s| s.card_size = n); },
                }
            }
            Separator {}
            SwitchRow {
                title: t("design.names"),
                checked: s.show_names,
                onchange: move |v: bool| app.update(|s| s.show_names = v),
            }
            Separator {}
            SwitchRow {
                title: t("design.logos"),
                description: t("design.logos_text"),
                checked: s.show_logos,
                onchange: move |v: bool| app.update(|s| s.show_logos = v),
            }
            Separator {}
            SwitchRow {
                title: t("design.animations"),
                description: t("design.animations_text"),
                checked: s.animations,
                onchange: move |v: bool| app.update(|s| s.animations = v),
            }
        }
        Section { title: t("a11y.title"), description: t("a11y.text"),
            div { class: "flex flex-wrap items-center justify-between gap-4 py-3",
                div {
                    div { class: "text-sm font-medium", {t("a11y.font")} }
                    div { class: "text-xs text-muted-foreground mt-1", {t("a11y.font_text")} }
                }
                Segmented {
                    options: vec![("100".to_string(), "100 %".to_string()), ("115".to_string(), "115 %".to_string()), ("130".to_string(), "130 %".to_string()), ("150".to_string(), "150 %".to_string())],
                    value: s.font_scale.to_string(),
                    onchange: move |v: String| { let n = v.parse::<u16>().unwrap_or(100); app.update(|s| s.font_scale = n); },
                }
            }
            Separator {}
            div { class: "py-3 text-sm",
                div { class: "font-medium mb-2", {t("a11y.keys")} }
                ul { class: "space-y-1 text-foreground/85",
                    {crate::i18n::tl("a11y.keys_list").into_iter().enumerate().map(|(i, k)| rsx! { li { key: "{i}", "{k}" } })}
                }
            }
        }
    }
}

// ----------------------------------------------------------------------
// Verbindungen
// ----------------------------------------------------------------------

/// Roblox: App + eigene Experiences
#[component]
fn RobloxSection() -> Element {
    let app = use_app();
    let mut input = use_signal(String::new);
    let status = app.roblox.read().clone();
    let s = app.settings.read();
    let enabled = s.roblox_enabled;
    let list = s.roblox_games.clone();
    drop(s);
    let mut add = move || {
        let v = input.peek().clone();
        if add_roblox(app, &v) {
            input.set(String::new());
        }
    };
    let dl = if cfg!(target_os = "linux") {
        ("https://flathub.org/apps/org.vinegarhq.Sober", t("fix.sober_dl"))
    } else {
        ("https://www.roblox.com/download", t("fix.roblox_dl"))
    };
    rsx! {
        Section { title: "Roblox", description: t("roblox.sub"),
            StatusLine { status: status.clone(), store: "Roblox" }
            if !status.found {
                div { class: "mt-3",
                    Button { variant: ButtonVariant::Outline, onclick: move |_| { let _ = sys::open_url(dl.0); },
                        Icon { name: "download", class: "size-4" }
                        "{dl.1}"
                    }
                }
            }
            div { class: "mt-2",
                SwitchRow {
                    title: t("roblox.show"),
                    checked: enabled,
                    onchange: move |v: bool| { app.update(|s| s.roblox_enabled = v); rescan(app); },
                }
            }
            div { class: "mt-4",
                div { class: "text-sm font-medium mb-1", {t("roblox.add_title")} }
                p { class: "text-xs text-muted-foreground mb-2", {t("roblox.add_text")} }
                div { class: "flex gap-2",
                    Input {
                        class: "font-mono text-xs",
                        placeholder: "https://www.roblox.com/games/920587237/…",
                        value: input(),
                        oninput: move |v: String| input.set(v),
                        onenter: move |_| add(),
                    }
                    Button { variant: ButtonVariant::Glow, class: "h-10", onclick: move |_| add(),
                        Icon { name: "plus", class: "size-4" }
                        {t("edit.add")}
                    }
                }
            }
            if !list.is_empty() {
                div { class: "mt-4 grid gap-2",
                    {list.into_iter().map(|g| {
                        let id = format!("roblox:{}", g.place_id);
                        let name = if g.name.is_empty() { format!("Roblox {}", g.place_id) } else { g.name.clone() };
                        let icon = g.icon.clone();
                        let pid = g.place_id;
                        rsx! {
                            div { key: "{pid}", class: "flex items-center gap-3 rounded-lg border border-border bg-muted/40 px-3 py-2",
                                if !icon.is_empty() {
                                    img { src: "{icon}", class: "size-9 rounded-md object-cover", "draggable": "false" }
                                } else {
                                    div { class: "size-9 rounded-md bg-muted grid place-items-center", Icon { name: "cube", class: "size-4" } }
                                }
                                div { class: "min-w-0 flex-1",
                                    div { class: "text-sm font-medium truncate", "{name}" }
                                    div { class: "text-xs text-muted-foreground font-mono", "{pid}" }
                                }
                                Button { variant: ButtonVariant::Ghost, size: ButtonSize::Icon, class: "size-9", title: t("act.remove"),
                                    onclick: move |_| remove_roblox(app, &id),
                                    Icon { name: "trash", class: "size-4" }
                                }
                            }
                        }
                    })}
                }
            }
        }
    }
}

/// Epic-Konto verbinden → komplette Epic-Bibliothek
/// Tastenkürzel passend zum System (Strg / ⌘)
fn kbd_mod() -> &'static str {
    if cfg!(target_os = "macos") { "⌘" } else { "Strg" }
}

#[component]
fn Kbd(#[props(into)] k: String) -> Element {
    rsx! { kbd { class: "inline-flex items-center h-6 px-2 rounded-md border border-border bg-background text-xs font-semibold font-mono shadow-sm", "{k}" } }
}

/// Ein Schritt im Epic-Ablauf
#[component]
fn EpicStep(n: u8, state: u8, #[props(into)] title: String, children: Element) -> Element {
    // state: 0 = noch nicht, 1 = jetzt, 2 = erledigt
    let (dot, txt) = match state {
        2 => ("bg-success text-white", "text-muted-foreground"),
        1 => ("maz-gradient-bg text-glow-fg ring-4 ring-primary/20", "text-foreground"),
        _ => ("bg-muted text-muted-foreground", "text-muted-foreground"),
    };
    rsx! {
        div { class: "flex gap-3",
            div { class: "size-7 shrink-0 rounded-full grid place-items-center text-xs font-bold {dot}",
                if state == 2 { Icon { name: "check", class: "size-4" } } else { "{n}" }
            }
            div { class: "min-w-0 flex-1 pb-4",
                div { class: "text-sm font-semibold pt-1 {txt}", "{title}" }
                if state == 1 {
                    div { class: "mt-2", {children} }
                }
            }
        }
    }
}

#[component]
pub fn EpicAccount() -> Element {
    let app = use_app();
    let mut waiting = use_signal(|| false);
    // Seit wann gewartet wird – nach 10 Minuten hört der Launcher auf, die Zwischenablage zu lesen
    let mut since = use_signal(|| 0u64);
    let mut pasted = use_signal(String::new);
    let count = use_memo(move || {
        let _ = app.settings.read().epic_synced;
        crate::epicacct::load_cache().len()
    });
    // Während auf die Anmeldung gewartet wird: Zwischenablage beobachten.
    // Sobald die kopierte Epic-Seite darin ist, geht es automatisch weiter.
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(400)).await;
            if !*waiting.peek() || *app.epic_busy.peek() {
                continue;
            }
            if sys::now_secs().saturating_sub(*since.peek()) > 600 {
                waiting.set(false);
                continue;
            }
            if let Some(text) = sys::read_clipboard() {
                if text.contains("authorizationCode") && crate::epicacct::extract_code(&text).is_some() {
                    waiting.set(false);
                    // Code aus der Zwischenablage entfernen (er ist nur einmal gültig)
                    sys::copy_to_clipboard(" ");
                    super::net::epic_connect(app, text);
                }
            }
        }
    });

    let busy = *app.epic_busy.read();
    let (done, total) = *app.epic_progress.read();
    let err = app.epic_error.read().clone();
    let s = app.settings.read();
    let auth = s.epic_auth.clone();
    let synced = s.epic_synced;
    drop(s);
    let n = count();
    let m = kbd_mod();
    let pct = if total > 0 { (done as f64 / total as f64 * 100.0).min(100.0) } else { 0.0 };
    let step = if busy { 3 } else if waiting() { 2 } else { 1 };
    let st = |k: u8| if k < step { 2 } else if k == step { 1 } else { 0 };

    rsx! {
        div { class: "mt-4 rounded-xl border border-border bg-muted/40 p-5",
            div { class: "flex items-start gap-3",
                div { class: "size-10 shrink-0 rounded-xl bg-primary/15 text-primary grid place-items-center",
                    Icon { name: "key", class: "size-5" }
                }
                div { class: "min-w-0 flex-1",
                    div { class: "font-semibold", {t("epic.title")} }
                    if let Some(a) = auth.clone() {
                        p { class: "text-sm text-success mt-0.5",
                            {tf("epic.connected_as", &[("name", if a.display_name.is_empty() { "Epic" } else { a.display_name.as_str() })])}
                        }
                        p { class: "text-sm text-muted-foreground mt-0.5",
                            {tf("epic.synced", &[("n", &n.to_string()), ("ago", &sys::format_ago(synced))])}
                        }
                    } else {
                        p { class: "text-sm text-muted-foreground mt-0.5", {t("epic.text")} }
                        if n > 0 {
                            p { class: "text-sm text-muted-foreground mt-1", {tf("epic.cached", &[("n", &n.to_string())])} }
                        }
                    }
                }
            }
            if auth.is_some() && !busy {
                div { class: "flex flex-wrap gap-2 mt-4",
                    Button { variant: ButtonVariant::Secondary, onclick: move |_| super::net::sync_epic(app, true),
                        Icon { name: "refresh", class: "size-4" }
                        {t("epic.sync_now")}
                    }
                    Button { variant: ButtonVariant::Ghost, onclick: move |_| super::net::epic_disconnect(app),
                        Icon { name: "log-out", class: "size-4" }
                        {t("epic.disconnect")}
                    }
                }
            } else {
                div { class: "mt-5",
                    EpicStep { n: 1, state: st(1), title: t("epic.s1"),
                        p { class: "text-sm text-muted-foreground leading-relaxed mb-3", {t("epic.s1_text")} }
                        Button { variant: ButtonVariant::Glow,
                            onclick: move |_| {
                                let _ = sys::open_url(&crate::epicacct::login_url());
                                let mut e = app.epic_error;
                                e.set(None);
                                since.set(sys::now_secs());
                                waiting.set(true);
                            },
                            Icon { name: "external", class: "size-4" }
                            {t("epic.connect")}
                        }
                    }
                    EpicStep { n: 2, state: st(2), title: t("epic.s2"),
                        p { class: "text-sm text-muted-foreground leading-relaxed", {t("epic.s2_text")} }
                        div { class: "flex flex-wrap items-center gap-2 my-3 text-sm",
                            Kbd { k: format!("{m} + A") }
                            span { class: "text-muted-foreground", {t("epic.then")} }
                            Kbd { k: format!("{m} + C") }
                        }
                        div { class: "flex items-center gap-2 text-sm text-muted-foreground mb-3",
                            Spinner {}
                            {t("epic.watching")}
                        }
                        Input {
                            class: "font-mono text-xs",
                            placeholder: tf("epic.paste_ph2", &[("key", &format!("{m} + V"))]),
                            value: pasted(),
                            oninput: move |v: String| {
                                pasted.set(v.clone());
                                if crate::epicacct::extract_code(&v).is_some() {
                                    waiting.set(false);
                                    pasted.set(String::new());
                                    super::net::epic_connect(app, v);
                                }
                            },
                        }
                        div { class: "flex flex-wrap gap-2 mt-3",
                            Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm,
                                onclick: move |_| { since.set(sys::now_secs()); let _ = sys::open_url(&crate::epicacct::login_url()); },
                                Icon { name: "refresh", class: "size-3.5" }
                                {t("epic.reopen")}
                            }
                            Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, onclick: move |_| waiting.set(false),
                                {t("common.cancel")}
                            }
                        }
                    }
                    EpicStep { n: 3, state: st(3), title: t("epic.s3"),
                        div { class: "flex justify-between text-sm mb-2",
                            span { class: "text-muted-foreground",
                                {if total > 1 { tf("epic.progress", &[("done", &done.to_string()), ("total", &total.to_string())]) } else { t("epic.signing_in") }}
                            }
                            if total > 1 { span { class: "tabular-nums", {format!("{pct:.0} %")} } }
                        }
                        div { class: "h-2 rounded-full bg-muted overflow-hidden",
                            div { class: "h-full maz-gradient-bg transition-all", style: "width: {pct}%" }
                        }
                    }
                }
            }
            if let Some(e) = err {
                div { class: "mt-3 flex items-start gap-2 rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm",
                    Icon { name: "alert", class: "size-4 mt-0.5 shrink-0 text-destructive" }
                    span { "{e}" }
                }
            }
            p { class: "mt-2 text-xs text-muted-foreground leading-relaxed flex gap-1.5",
                Icon { name: "shield", class: "size-3.5 mt-0.5 shrink-0" }
                {t("epic.privacy")}
            }
        }
    }
}

#[component]
fn StatusLine(status: StoreStatus, #[props(into)] store: String) -> Element {
    let (cls, icon, text) = if status.found {
        ("text-success", "check-circle", tf("conn.connected", &[("store", &store)]))
    } else {
        ("text-muted-foreground", "x-circle", tf("conn.not_found", &[("store", &store)]))
    };
    let mut details: Vec<String> = Vec::new();
    if let Some(a) = &status.account {
        details.push(tf("conn.account", &[("name", a)]));
    }
    if status.found {
        details.push(tf("conn.installed", &[("n", &status.installed.to_string())]));
    }
    if status.owned > 0 {
        details.push(tf("conn.owned", &[("n", &status.owned.to_string())]));
    }
    let detail = details.join(" · ");
    let path = status.path.clone().unwrap_or_default();
    let note = status.note.clone().unwrap_or_default();
    rsx! {
        div { class: "rounded-xl bg-muted p-4",
            div { class: "flex items-center gap-2 font-semibold {cls}",
                Icon { name: icon, class: "size-5" }
                "{text}"
            }
            if !detail.is_empty() {
                div { class: "text-sm mt-1.5", "{detail}" }
            }
            if !path.is_empty() {
                div { class: "text-xs text-muted-foreground mt-1 font-mono truncate select-text", title: "{path}", "{path}" }
            }
            if !note.is_empty() {
                div { class: "text-xs text-muted-foreground mt-2 leading-relaxed", "{note}" }
            }
        }
    }
}

#[component]
fn ConnectionsTab() -> Element {
    let app = use_app();
    let s = app.settings.read().clone();
    let steam = app.steam.read().clone();
    let epic = app.epic.read().clone();
    let scanning = *app.scanning.read();
    let steam_found = steam.found;
    let epic_found = epic.found;
    let key_steps = i18n::tl("conn.key_steps");
    rsx! {
        PageTitle { title: t("set.tab.connections"), sub: t("conn.sub") }

        Section { title: "Steam",
            StatusLine { status: steam, store: "Steam" }
            if !steam_found {
                div { class: "mt-3",
                    Button { variant: ButtonVariant::Outline, onclick: move |_| { let _ = sys::open_url("https://store.steampowered.com/about/"); },
                        Icon { name: "download", class: "size-4" }
                        {t("fix.steam_dl")}
                    }
                }
            }
            div { class: "mt-2",
                SwitchRow {
                    title: t("conn.show_steam"),
                    checked: s.steam_enabled,
                    onchange: move |v: bool| { app.update(|s| s.steam_enabled = v); rescan(app); },
                }
            }
            Separator { class: "my-2" }
            div { class: "py-3",
                div { class: "text-sm font-medium", {t("conn.all_title")} }
                p { class: "text-xs text-muted-foreground mt-1 mb-3 leading-relaxed", {t("conn.all_text")} }
                ol { class: "text-xs text-muted-foreground space-y-1 mb-4 list-decimal pl-5 leading-relaxed",
                    {key_steps.into_iter().enumerate().map(|(i, st)| rsx! { li { key: "{i}", "{st}" } })}
                }
                div { class: "grid gap-3",
                    div { class: "flex gap-2",
                        Input {
                            kind: "password",
                            class: "font-mono text-xs",
                            placeholder: t("conn.key_ph"),
                            value: s.steam_api_key.clone(),
                            oninput: move |v: String| app.update(|s| s.steam_api_key = v.trim().to_string()),
                        }
                        Button { variant: ButtonVariant::Secondary, class: "h-10", onclick: move |_| { let _ = sys::open_url("https://steamcommunity.com/dev/apikey"); },
                            Icon { name: "external", class: "size-4" }
                            {t("fix.api_key")}
                        }
                    }
                    Input {
                        class: "font-mono text-xs",
                        placeholder: t("conn.id_ph"),
                        value: s.steam_id.clone(),
                        oninput: move |v: String| app.update(|s| s.steam_id = v.trim().to_string()),
                    }
                }
                div { class: "mt-2",
                    SwitchRow {
                        title: t("conn.uninstalled"),
                        description: t("conn.uninstalled_text"),
                        checked: s.steam_show_uninstalled,
                        onchange: move |v: bool| { app.update(|s| s.steam_show_uninstalled = v); rescan(app); },
                    }
                }
            }
        }

        Section { title: if cfg!(target_os = "linux") { "Epic Games (Heroic)" } else { "Epic Games" },
            StatusLine { status: epic, store: "Epic Games" }
            if !epic_found {
                div { class: "mt-3",
                    Button { variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            let _ = sys::open_url(if cfg!(target_os = "linux") { "https://flathub.org/apps/com.heroicgameslauncher.hgl" } else { "https://store.epicgames.com/download" });
                        },
                        Icon { name: "download", class: "size-4" }
                        {t(if cfg!(target_os = "linux") { "fix.heroic_dl" } else { "fix.epic_dl" })}
                    }
                }
            }
            div { class: "mt-2",
                SwitchRow {
                    title: t("conn.show_epic"),
                    checked: s.epic_enabled,
                    onchange: move |v: bool| { app.update(|s| s.epic_enabled = v); rescan(app); },
                }
                SwitchRow {
                    title: t("conn.uninstalled"),
                    description: t("conn.epic_uninstalled_text"),
                    checked: s.epic_show_uninstalled,
                    onchange: move |v: bool| { app.update(|s| s.epic_show_uninstalled = v); rescan(app); },
                }
            }
            if !cfg!(target_os = "linux") {
                EpicAccount {}
            }
        }

        RobloxSection {}

        MoreStoresSection {}

        super::media::SgdbSection {}

        MinecraftSection {}

        EmulatorSection {}

        Section { title: t("conn.images"),
            SwitchRow {
                title: t("conn.online_art"),
                description: t("conn.online_art_text"),
                checked: s.online_art,
                onchange: move |v: bool| app.update(|s| s.online_art = v),
            }
        }

        div { class: "flex justify-end",
            Button { variant: ButtonVariant::Glow, size: ButtonSize::Lg, disabled: scanning, onclick: move |_| rescan(app),
                if scanning { Spinner {} } else { Icon { name: "refresh", class: "size-4" } }
                {t("conn.reload_now")}
            }
        }
    }
}

/// Minecraft Launcher, NoRisk Client und CurseForge-Schlüssel
#[component]
fn MinecraftSection() -> Element {
    let app = use_app();
    let on = app.settings.read().store_on(Source::Minecraft);
    let has_key = !app.settings.read().curseforge_key.is_empty();
    let games = app.games.read();
    let official = games.iter().any(|g| g.id == "minecraft:launcher");
    let profiles = games.iter().filter(|g| g.id.starts_with("norisk:")).count();
    drop(games);
    let mut show_guide = use_signal(|| false);
    let mut key_input = use_signal(String::new);
    rsx! {
        Section { title: "Minecraft", description: t("mc.conn_text"),
            div { class: "rounded-xl border border-border divide-y divide-border mb-3",
                div { class: "flex items-center gap-4 px-4 py-3",
                    span { class: if official { "size-2.5 rounded-full bg-success" } else { "size-2.5 rounded-full bg-muted-foreground/40" } }
                    div { class: "flex-1 text-sm font-medium", "Minecraft Launcher" }
                    if !official {
                        Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, onclick: move |_| { let _ = sys::open_url(crate::minecraft::MC_DOWNLOAD); },
                            Icon { name: "download", class: "size-3.5" } {t("conn.get_client")}
                        }
                    }
                }
                div { class: "flex items-center gap-4 px-4 py-3",
                    span { class: if profiles > 0 { "size-2.5 rounded-full bg-success" } else { "size-2.5 rounded-full bg-muted-foreground/40" } }
                    div { class: "flex-1 min-w-0",
                        div { class: "text-sm font-medium", "NoRisk Client" }
                        div { class: "text-xs text-muted-foreground mt-0.5",
                            {if profiles > 0 { tf("mc.norisk_profiles", &[("n", &profiles.to_string())]) } else { t("mc.norisk_missing") }}
                        }
                    }
                    if profiles == 0 {
                        Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, onclick: move |_| { let _ = sys::open_url(crate::minecraft::NORISK_DOWNLOAD); },
                            Icon { name: "download", class: "size-3.5" } {t("conn.get_client")}
                        }
                    }
                }
            }
            SwitchRow {
                title: t("mc.show"),
                checked: on,
                onchange: move |v: bool| {
                    app.update(|s| {
                        s.stores_off.retain(|k| k != "minecraft");
                        if !v {
                            s.stores_off.push("minecraft".into());
                        }
                    });
                    rescan(app);
                },
            }
            div { class: "mt-3",
                if has_key {
                    div { class: "flex items-center gap-3 rounded-xl border border-border px-4 py-3 text-sm",
                        Icon { name: "key", class: "size-4 text-success" }
                        span { class: "flex-1", {t("mc.key_set")} }
                        Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, class: "text-destructive",
                            onclick: move |_| app.update(|s| s.curseforge_key.clear()),
                            Icon { name: "trash", class: "size-3.5" } {t("act.remove")}
                        }
                    }
                } else {
                    super::minecraft::CurseForgeKeyBox { show_guide: show_guide(), on_toggle: move |_| show_guide.set(!show_guide()), key_input: key_input(),
                        on_input: move |v: String| key_input.set(v),
                        on_save: move |_| {
                            let cleaned = crate::mccontent::clean_cf_key(&key_input.peek());
                            if let Some(k) = cleaned {
                                app.update(|s| s.curseforge_key = k);
                                key_input.set(String::new());
                                app.toast(&t("mc.key_saved"), "check-circle");
                            } else {
                                app.toast(&t("mc.key_invalid"), "alert");
                            }
                        },
                    }
                }
            }
        }
    }
}

/// GOG, EA, Ubisoft, Battle.net, Amazon, itch.io, Xbox
#[component]
fn MoreStoresSection() -> Element {
    let app = use_app();
    let s = app.settings.read().clone();
    let statuses = app.stores.read().clone();
    let libs: Vec<(Source, &'static str)> = crate::stores::all()
        .into_iter()
        .filter(|l| l.supported())
        .map(|l| (l.source(), l.download_url()))
        .collect();
    rsx! {
        Section { title: t("conn.more"), description: t("conn.more_text"),
            div { class: "rounded-xl border border-border divide-y divide-border",
                {libs.into_iter().map(|(src, dl)| {
                    let on = s.store_on(src);
                    let st = statuses.iter().find(|(x, _)| *x == src).map(|(_, st)| st.clone());
                    let (dot, line) = match (&st, on) {
                        (_, false) => ("bg-muted-foreground/40", t("conn.store_off")),
                        (Some(st), true) if st.found && st.owned > st.installed => (
                            "bg-success",
                            format!("{} · {}", tf("conn.installed", &[("n", &st.installed.to_string())]), tf("conn.owned", &[("n", &st.owned.to_string())])),
                        ),
                        (Some(st), true) if st.found => ("bg-success", tf("conn.installed", &[("n", &st.installed.to_string())])),
                        _ => ("bg-muted-foreground/40", tf("store.client_missing", &[("store", &src.label())])),
                    };
                    let found = st.map(|x| x.found).unwrap_or(false);
                    let key = src.key();
                    let label = src.label();
                    rsx! {
                        div { key: "{key}", class: "flex items-center gap-4 px-4 py-3",
                            span { class: "size-2.5 rounded-full shrink-0 {dot}" }
                            div { class: "min-w-0 flex-1",
                                div { class: "text-sm font-medium", "{label}" }
                                div { class: "text-xs text-muted-foreground mt-0.5 leading-relaxed", "{line}" }
                            }
                            if on && !found {
                                Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, onclick: move |_| { let _ = sys::open_url(dl); },
                                    Icon { name: "download", class: "size-3.5" }
                                    {t("conn.get_client")}
                                }
                            }
                            Switch {
                                checked: on,
                                label: label.clone(),
                                onchange: move |v: bool| {
                                    app.update(|s| {
                                        s.stores_off.retain(|k| k != src.key());
                                        if !v {
                                            s.stores_off.push(src.key().to_string());
                                        }
                                    });
                                    rescan(app);
                                },
                            }
                        }
                    }
                })}
            }
            SwitchRow {
                title: t("conn.uninstalled"),
                description: t("conn.more_uninstalled_text"),
                checked: s.stores_show_uninstalled,
                onchange: move |v: bool| { app.update(|s| s.stores_show_uninstalled = v); rescan(app); },
            }
            p { class: "text-xs text-muted-foreground mt-3 leading-relaxed", {t("conn.more_privacy")} }
        }
    }
}

/// Emulator-Profile und ROM-Ordner
#[component]
fn EmulatorSection() -> Element {
    let app = use_app();
    let s = app.settings.read().clone();
    let on = s.store_on(Source::Emulator);
    let emus = s.emulators.clone();
    let mut preset = use_signal(|| 0usize);
    let other_idx = crate::emulators::PRESETS.len().to_string();
    rsx! {
        Section { title: t("emu.title"), description: t("emu.text"),
            SwitchRow {
                title: t("emu.show"),
                checked: on,
                onchange: move |v: bool| {
                    app.update(|s| {
                        s.stores_off.retain(|k| k != "emulator");
                        if !v {
                            s.stores_off.push("emulator".into());
                        }
                    });
                    rescan(app);
                },
            }
            div { class: "grid gap-4 mt-2",
                {emus.into_iter().map(|e| {
                    let key = e.id.clone();
                    rsx! { EmulatorCard { key: "{key}", emu: e } }
                })}
            }
            div { class: "flex flex-wrap items-center gap-2 mt-4",
                select {
                    class: "h-10 rounded-md border border-input bg-background/60 px-3 text-sm outline-none cursor-pointer",
                    onchange: move |e| preset.set(e.value().parse().unwrap_or(0)),
                    {crate::emulators::PRESETS.iter().enumerate().map(|(i, (name, _, _))| rsx! {
                        option { key: "{i}", value: "{i}", selected: preset() == i, "{name}" }
                    })}
                    option { value: "{other_idx}", {t("emu.other")} }
                }
                Button { variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        let i = *preset.peek();
                        let mut e = crate::model::Emulator { id: new_custom_id(), ..Default::default() };
                        match crate::emulators::PRESETS.get(i) {
                            Some((name, args, exts)) => {
                                e.name = name.to_string();
                                e.args = args.to_string();
                                e.extensions = exts.iter().map(|x| x.to_string()).collect();
                                e.platform = crate::emulators::preset_platform(name).to_string();
                            }
                            None => {
                                e.name = t("emu.new_name");
                                e.args = "\"{rom}\"".into();
                            }
                        }
                        app.update(|s| s.emulators.push(e));
                    },
                    Icon { name: "plus", class: "size-4" }
                    {t("emu.add")}
                }
            }
            p { class: "text-xs text-muted-foreground mt-3 leading-relaxed", {t("emu.legal")} }
        }
    }
}

fn update_emu(app: App, id: &str, f: impl FnOnce(&mut crate::model::Emulator)) {
    let id = id.to_string();
    app.update(|s| {
        if let Some(e) = s.emulators.iter_mut().find(|e| e.id == id) {
            f(e);
        }
    });
}

#[component]
fn EmulatorCard(emu: crate::model::Emulator) -> Element {
    let app = use_app();
    let (i1, i2, i3, i4, i5, i6, i7) = (emu.id.clone(), emu.id.clone(), emu.id.clone(), emu.id.clone(), emu.id.clone(), emu.id.clone(), emu.id.clone());
    let i_plat = emu.id.clone();
    let exts = emu.extensions.join(", ");
    let emu_name = emu.name.clone();
    let exe_ok = !emu.exe.is_empty() && std::path::Path::new(&emu.exe).exists();
    let count = app.games.read().iter().filter(|g| g.id.starts_with(&format!("emu:{}:", emu.id))).count();
    rsx! {
        div { class: "rounded-xl border border-border p-4 grid gap-3",
            div { class: "flex items-center gap-3",
                Icon { name: "joystick", class: "size-5 text-primary" }
                input {
                    class: "flex-1 bg-transparent text-sm font-semibold outline-none",
                    value: "{emu_name}",
                    oninput: move |e| { let v = e.value(); update_emu(app, &i1, |x| x.name = v); },
                }
                span { class: "text-xs text-muted-foreground", {tf("emu.count", &[("n", &count.to_string())])} }
                Button { variant: ButtonVariant::Ghost, size: ButtonSize::IconSm, class: "text-destructive", title: t("act.remove"),
                    onclick: move |_| { let id = i2.clone(); app.update(|s| s.emulators.retain(|e| e.id != id)); rescan(app); },
                    Icon { name: "trash", class: "size-4" }
                }
            }
            div { class: "flex gap-2",
                Input { class: "font-mono text-xs", placeholder: t("emu.exe_ph"), value: emu.exe.clone(),
                    oninput: move |v: String| update_emu(app, &i3, |x| x.exe = v.trim().trim_matches('"').to_string()) }
                Button { variant: ButtonVariant::Secondary, class: "h-10",
                    onclick: move |_| { if let Some(p) = pick_game_file(None) { update_emu(app, &i4, |x| x.exe = p); } },
                    Icon { name: "folder", class: "size-4" }
                    {t("common.browse")}
                }
            }
            if !emu.exe.is_empty() && !exe_ok {
                div { class: "text-xs text-destructive", {t("emu.exe_missing")} }
            }
            div { class: "grid grid-cols-2 gap-3",
                div { class: "grid gap-1",
                    Label { {t("emu.args")} }
                    Input { class: "font-mono text-xs", value: emu.args.clone(),
                        oninput: move |v: String| update_emu(app, &i5, |x| x.args = v) }
                }
                div { class: "grid gap-1",
                    Label { {t("emu.exts")} }
                    Input { class: "font-mono text-xs", placeholder: "iso, chd, cue", value: exts,
                        oninput: move |v: String| {
                            let list: Vec<String> = v.split([',', ' ', ';']).map(|x| x.trim().trim_start_matches('.').to_lowercase()).filter(|x| !x.is_empty()).collect();
                            update_emu(app, &i6, |x| x.extensions = list);
                        } }
                }
            }
            div { class: "grid gap-1",
                Label { {t("emu.platform")} }
                Input { placeholder: crate::emulators::preset_platform(&emu.name).to_string(), value: emu.platform.clone(),
                    oninput: move |v: String| update_emu(app, &i_plat, |x| x.platform = v.chars().take(40).collect()) }
            }
            div { class: "grid gap-1",
                Label { {t("emu.folders")} }
                {emu.rom_dirs.iter().map(|d| {
                    let (eid, d2) = (emu.id.clone(), d.clone());
                    rsx! {
                        div { key: "{d}", class: "flex items-center gap-2 text-xs font-mono",
                            Icon { name: "folder", class: "size-3.5 text-muted-foreground" }
                            span { class: "flex-1 truncate", title: "{d}", "{d}" }
                            button { class: "p-1 rounded text-muted-foreground hover:text-destructive cursor-pointer", title: t("act.remove"),
                                onclick: move |_| { let d3 = d2.clone(); update_emu(app, &eid, |x| x.rom_dirs.retain(|r| *r != d3)); rescan(app); },
                                Icon { name: "x", class: "size-3.5" }
                            }
                        }
                    }
                })}
                div {
                    Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm,
                        onclick: move |_| {
                            if let Some(p) = pick_folder(&t("emu.folders")) {
                                update_emu(app, &i7, |x| if !x.rom_dirs.contains(&p) { x.rom_dirs.push(p) });
                                rescan(app);
                            }
                        },
                        Icon { name: "plus", class: "size-3.5" }
                        {t("emu.add_folder")}
                    }
                }
            }
        }
    }
}

// ----------------------------------------------------------------------
// Eigene Spiele
// ----------------------------------------------------------------------

#[component]
fn CustomTab() -> Element {
    let app = use_app();
    let s = app.settings.read().clone();
    let items = s.effective_checklist();
    let games = s.custom_games.clone();
    rsx! {
        PageTitle { title: t("set.tab.custom"), sub: t("custom.sub") }

        Section { title: t("custom.check_title"), description: t("custom.check_text"),
            div { class: "grid gap-2",
                {items.iter().cloned().enumerate().map(|(i, it)| rsx! {
                    div { key: "{i}", class: "flex gap-2",
                        Input {
                            value: it,
                            oninput: move |v: String| app.update(|s| {
                                if s.checklist.is_empty() {
                                    s.checklist = default_checklist();
                                }
                                if let Some(x) = s.checklist.get_mut(i) {
                                    *x = v;
                                }
                            }),
                        }
                        Button { variant: ButtonVariant::Ghost, size: ButtonSize::Icon, class: "size-10", title: t("custom.delete_item"),
                            onclick: move |_| app.update(|s| {
                                if s.checklist.is_empty() {
                                    s.checklist = default_checklist();
                                }
                                if i < s.checklist.len() {
                                    s.checklist.remove(i);
                                }
                                if s.checklist.is_empty() {
                                    // Leere Liste würde wieder die Standardliste bedeuten → Platzhalter behalten
                                    s.checklist.push(String::new());
                                }
                            }),
                            Icon { name: "trash", class: "size-4" }
                        }
                    }
                })}
            }
            div { class: "flex gap-2 mt-4",
                Button { variant: ButtonVariant::Outline,
                    onclick: move |_| app.update(|s| {
                        if s.checklist.is_empty() {
                            s.checklist = default_checklist();
                        }
                        s.checklist.push(String::new());
                    }),
                    Icon { name: "plus", class: "size-4" }
                    {t("custom.add_item")}
                }
                Button { variant: ButtonVariant::Ghost, onclick: move |_| app.update(|s| s.checklist.clear()),
                    Icon { name: "refresh", class: "size-4" }
                    {t("custom.reset")}
                }
            }
        }

        Section { title: t("custom.games"), description: t("custom.games_text"),
            if games.is_empty() {
                div { class: "text-sm text-muted-foreground py-2", {t("custom.none")} }
            }
            div { class: "divide-y divide-border",
                {games.into_iter().map(|g| {
                    let (id1, id2) = (g.id.clone(), g.id.clone());
                    let name = g.name.clone();
                    let exe = g.exe.clone();
                    let key = g.id.clone();
                    rsx! {
                        div { key: "{key}", class: "flex items-center gap-3 py-3",
                            div { class: "min-w-0 flex-1",
                                div { class: "text-sm font-medium truncate", "{name}" }
                                div { class: "text-xs text-muted-foreground font-mono truncate", title: "{exe}", "{exe}" }
                            }
                            Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm, onclick: move |_| app.open(Modal::EditGame(Some(id1.clone()))),
                                Icon { name: "pencil", class: "size-3.5" }
                                {t("common.edit")}
                            }
                            Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, class: "text-destructive", onclick: move |_| app.open(Modal::DeleteGame(id2.clone())),
                                Icon { name: "trash", class: "size-3.5" }
                            }
                        }
                    }
                })}
            }
            div { class: "mt-4 flex flex-wrap gap-2",
                Button { variant: ButtonVariant::Glow, onclick: move |_| app.open(Modal::EditGame(None)),
                    Icon { name: "plus", class: "size-4" }
                    {t("lib.add")}
                }
                Button { variant: ButtonVariant::Outline, onclick: move |_| app.open(Modal::FolderScan),
                    Icon { name: "folder-search", class: "size-4" }
                    {t("scan.button")}
                }
            }
        }
    }
}

// ----------------------------------------------------------------------
// Allgemein
// ----------------------------------------------------------------------

#[component]
fn GeneralTab() -> Element {
    let app = use_app();
    let s = app.settings.read().clone();
    let exe = install::current_exe().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
    let exe_dir = install::current_exe()
        .and_then(|p| p.parent().map(|d| d.to_string_lossy().to_string()))
        .unwrap_or_default();
    let installed = install::installed_location().is_some();
    let exe2 = exe.clone();
    rsx! {
        PageTitle { title: t("set.tab.general"), sub: t("general.sub") }

        Section { title: t("general.behavior"),
            SwitchRow {
                title: t("general.minimize"),
                description: t("general.minimize_text"),
                checked: s.minimize_on_launch,
                onchange: move |v: bool| app.update(|s| s.minimize_on_launch = v),
            }
            SwitchRow {
                title: t("general.merge"),
                description: t("general.merge_text"),
                checked: s.merge_duplicates,
                onchange: move |v: bool| { app.update(|s| s.merge_duplicates = v); rescan(app); },
            }
            SwitchRow {
                title: t("general.controller"),
                description: t("general.controller_text"),
                checked: s.controller_mode,
                onchange: move |v: bool| app.update(|s| s.controller_mode = v),
            }
        }

        UpdateSection {}

        super::tools::BackupSection {}

        super::opts::IntegrationSection {}

        Section { title: t("general.install"),
            div { class: "rounded-xl bg-muted p-4 mb-4",
                div { class: "text-xs text-muted-foreground", {t(if installed { "general.installed_in" } else { "general.portable" })} }
                div { class: "text-sm font-mono mt-1 truncate select-text", title: "{exe}", "{exe}" }
            }
            div { class: "flex flex-wrap gap-2",
                Button { variant: ButtonVariant::Outline, onclick: move |_| sys::open_folder(&exe_dir),
                    Icon { name: "folder", class: "size-4" }
                    {t("fix.folder_app")}
                }
                if cfg!(windows) {
                Button { variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        let lnk = sys::desktop_dir().join(format!("{}.lnk", sys::APP_NAME));
                        match install::create_shortcut(&lnk, std::path::Path::new(&exe2)) {
                            Ok(()) => app.toast(&t("toast.shortcut"), "check-circle"),
                            Err(e) => app.show_problem(crate::problem::Problem::shortcut_failed(&exe2, &e), None),
                        }
                    },
                    Icon { name: "monitor", class: "size-4" }
                    {t("general.shortcut")}
                }
                }
                if installed {
                    Button { variant: ButtonVariant::Ghost, class: "text-destructive",
                        onclick: move |_| {
                            install::uninstall_interactive();
                            if install::installed_location().is_none() {
                                std::process::exit(0);
                            }
                        },
                        Icon { name: "trash", class: "size-4" }
                        {t("general.uninstall")}
                    }
                }
            }
        }

        Section { title: t("general.data"),
            div { class: "flex flex-wrap gap-2",
                Button { variant: ButtonVariant::Outline, onclick: move |_| sys::open_folder(&sys::roaming_dir().to_string_lossy()),
                    Icon { name: "folder", class: "size-4" }
                    {t("general.settings_folder")}
                }
                Button { variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        let _ = std::fs::remove_dir_all(sys::icon_cache_dir());
                        app.update(|s| s.art_lookup.clear());
                        app.toast(&t("toast.cache_cleared"), "refresh");
                        rescan(app);
                    },
                    Icon { name: "image", class: "size-4" }
                    {t("general.reload_images")}
                }
            }
        }
    }
}

// ----------------------------------------------------------------------
// Über
// ----------------------------------------------------------------------

#[component]
fn AboutTab() -> Element {
    let version = tf("about.version", &[("v", sys::VERSION)]);
    rsx! {
        div { class: "flex flex-col items-center text-center pt-6",
            MazLogo { class: "size-24 text-2xl rounded-3xl" }
            h1 { class: "text-4xl font-black font-display mt-6", span { class: "maz-gradient-text", {crate::sys::APP_NAME} } }
            p { class: "text-muted-foreground mt-2", "{version}" }
            p { class: "text-sm text-muted-foreground mt-6 max-w-md leading-relaxed", {t("about.text")} }
            div { class: "flex gap-3 mt-8",
                Button { variant: ButtonVariant::Glow, onclick: move |_| { let _ = sys::open_url(sys::GITHUB_URL); },
                    Icon { name: "code", class: "size-4" }
                    {t("about.github")}
                }
                Button { variant: ButtonVariant::Outline, onclick: move |_| { let _ = sys::open_url(&format!("{}/issues", sys::GITHUB_URL)); },
                    Icon { name: "alert", class: "size-4" }
                    {t("fix.report")}
                }
            }
            p { class: "text-xs text-muted-foreground mt-12 max-w-md leading-relaxed", {t("about.legal")} }
        }
    }
}


/// Updates: aktuelle Version, automatisch prüfen, jetzt prüfen
#[component]
fn UpdateSection() -> Element {
    let app = use_app();
    let auto = app.settings.read().update_check;
    let test = app.settings.read().update_test;
    let checking = matches!(*app.update_state.read(), UpdateState::Checking);
    let avail = app.update_avail.read().as_ref().map(|r| r.version.clone());
    let failed = match &*app.update_state.read() {
        UpdateState::Failed(e) => Some(e.clone()),
        _ => None,
    };
    let kind = crate::updater::InstallKind::detect();
    let kind_label = kind.label();
    rsx! {
        Section { title: t("update.section"), description: tf("update.current", &[("version", sys::VERSION)]),
            SwitchRow {
                title: t("update.auto"),
                description: t("update.auto_text"),
                checked: auto,
                onchange: move |v: bool| app.update(|s| s.update_check = v),
            }
            SwitchRow {
                title: t("update.test"),
                description: t("update.test_text"),
                checked: test,
                onchange: move |v: bool| {
                    app.update(|s| s.update_test = v);
                    // Neu prüfen, damit die passende Version angeboten wird
                    let mut upd = app.update_avail;
                    upd.set(None);
                    super::net::check_update(app, false);
                },
            }
            if !kind_label.is_empty() {
                p { class: "text-xs text-muted-foreground mt-1", {tf("update.how", &[("kind", kind_label)])} }
            }
            div { class: "flex flex-wrap gap-2 mt-4",
                Button { variant: ButtonVariant::Secondary, disabled: checking, onclick: move |_| super::net::check_update(app, true),
                    if checking { Spinner {} } else { Icon { name: "refresh", class: "size-4" } }
                    {t("update.check")}
                }
                if let Some(v) = avail {
                    Button { variant: ButtonVariant::Glow, onclick: move |_| app.open(Modal::Update),
                        Icon { name: "download", class: "size-4" }
                        {tf("update.pill", &[("version", &v)])}
                    }
                }
                Button { variant: ButtonVariant::Ghost, onclick: move |_| { let _ = sys::open_url(&format!("{}/releases", sys::GITHUB_URL)); },
                    Icon { name: "external", class: "size-4" }
                    {t("update.github")}
                }
            }
            if let Some(e) = failed {
                div { class: "mt-3 flex items-start gap-2 rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm",
                    Icon { name: "alert", class: "size-4 mt-0.5 shrink-0 text-destructive" }
                    span { "{e}" }
                }
            }
        }
    }
}
