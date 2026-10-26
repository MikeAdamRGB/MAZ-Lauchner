//! Die Bibliothek – in drei Ansichten: MAZ (eigene), Steam-Stil und Epic-Stil.

use super::art::{ArtImage, CoverFallback, GameIcon};
use super::components::*;
use super::detail::SteamDetail;
use super::icons::{Icon, MazLogo};
use super::state::*;
use crate::model::{layout, Game, Settings, Source};
use crate::i18n::{t, tf};
use crate::sys;
use dioxus::prelude::*;

/// Gefilterte und sortierte Spieleliste (liest Signale → aktualisiert sich automatisch)
pub fn visible_games(app: App) -> Vec<Game> {
    let s = app.settings.read();
    let filter = *app.filter.read();
    let arg = app.filter_arg.read().clone();
    let genre = app.genre.read().clone();
    let cache = app.meta_cache.read();
    let q = app.search.read().trim().to_lowercase();
    let mut list: Vec<Game> = app
        .games
        .read()
        .iter()
        .filter(|g| {
            let hidden = s.is_hidden(&g.id);
            if filter == Filter::Hidden {
                return hidden;
            }
            !hidden && filter.matches(&arg, g, &s)
        })
        .filter(|g| {
            genre.is_empty() || {
                let own = s.meta.get(&g.id).map(|m| m.genres.clone()).filter(|v| !v.is_empty());
                let auto = || cache.get(&g.id).map(|i| i.genres.clone()).unwrap_or_default();
                own.unwrap_or_else(auto).iter().any(|x| x.eq_ignore_ascii_case(&genre))
            }
        })
        .filter(|g| {
            q.is_empty() || g.name.to_lowercase().contains(&q) || s.tags_of(&g.id).iter().any(|t| t.to_lowercase().contains(&q))
        })
        .cloned()
        .collect();
    let sort = if filter == Filter::Recent { "recent" } else { s.sort.as_str() };
    match sort {
        "recent" => list.sort_by(|a, b| s.last_played(&b.id).cmp(&s.last_played(&a.id)).then(a.sort_key().cmp(&b.sort_key()))),
        "playtime" => list.sort_by(|a, b| s.playtime(b).cmp(&s.playtime(a)).then(a.sort_key().cmp(&b.sort_key()))),
        "installed" => list.sort_by(|a, b| b.installed.cmp(&a.installed).then(a.sort_key().cmp(&b.sort_key()))),
        "added" => list.sort_by(|a, b| s.first_seen(&b.id).cmp(&s.first_seen(&a.id)).then(a.sort_key().cmp(&b.sort_key()))),
        _ => list.sort_by_key(|a| a.sort_key()),
    }
    list
}

pub fn count_for(app: App, f: Filter) -> usize {
    count_with(app, f, "")
}

pub fn count_with(app: App, f: Filter, arg: &str) -> usize {
    let s = app.settings.read();
    app.games
        .read()
        .iter()
        .filter(|g| {
            let hidden = s.is_hidden(&g.id);
            match f {
                Filter::Hidden => hidden,
                _ if hidden => false,
                _ => f.matches(arg, g, &s),
            }
        })
        .count()
}

/// Alle bekannten Genres (eigene + automatisch geladene), sortiert
pub fn all_genres(app: App) -> Vec<String> {
    let s = app.settings.read();
    let cache = app.meta_cache.read();
    let mut v: Vec<String> = Vec::new();
    for g in app.games.read().iter() {
        let own = s.meta.get(&g.id).map(|m| m.genres.clone()).filter(|v| !v.is_empty());
        for genre in own.unwrap_or_else(|| cache.get(&g.id).map(|i| i.genres.clone()).unwrap_or_default()) {
            if !v.iter().any(|x| x.eq_ignore_ascii_case(&genre)) {
                v.push(genre);
            }
        }
    }
    v.sort_by_key(|x| x.to_lowercase());
    v
}

/// Eindeutiger Wert für Auswahllisten (Erweiterungen haben alle denselben key())
pub fn source_value(src: Source) -> String {
    match src {
        Source::Extension(i) => format!("ext-{i}"),
        other => other.key().to_string(),
    }
}

/// Weitere Stores/Emulatoren, in denen es Spiele gibt
pub fn stores_with_games(app: App) -> Vec<Source> {
    let games = app.games.read();
    let mut list: Vec<Source> = Source::STORES
        .iter()
        .copied()
        .chain([Source::Minecraft, Source::Emulator])
        .filter(|src| games.iter().any(|g| g.source == *src))
        .collect();
    // Erweiterungen
    for g in games.iter() {
        if matches!(g.source, Source::Extension(_)) && !list.contains(&g.source) {
            list.push(g.source);
        }
    }
    list
}

/// Überschrift für den aktuellen Filter
pub fn filter_title(app: App) -> String {
    let f = *app.filter.read();
    let arg = app.filter_arg.read().clone();
    match f {
        Filter::Tag => format!("# {arg}"),
        Filter::Collection => app
            .settings
            .read()
            .collections
            .iter()
            .find(|c| c.id == arg)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| f.label()),
        _ => f.label(),
    }
}

fn min_card_width(s: &Settings) -> u32 {
    match s.card_size {
        1 => 132,
        3 => 230,
        _ => 172,
    }
}

#[component]
pub fn LibraryPage() -> Element {
    let app = use_app();
    let lay = app.settings.read().layout.clone();
    if lay == layout::STEAM {
        rsx! { SteamLayout {} }
    } else if lay == layout::EPIC {
        rsx! { EpicLayout {} }
    } else {
        rsx! { MazLayout {} }
    }
}

// ======================================================================
// Gemeinsame Teile
// ======================================================================

#[component]
pub fn SourceBadge(source: Source) -> Element {
    match source {
        Source::Steam => rsx! { Badge { variant: BadgeVariant::Steam, "STEAM" } },
        Source::Epic => rsx! { Badge { variant: BadgeVariant::Epic, "EPIC" } },
        Source::Roblox => rsx! { Badge { variant: BadgeVariant::Roblox, "ROBLOX" } },
        Source::Custom => rsx! { Badge { variant: BadgeVariant::Custom, {t("badge.custom")} } },
        other => rsx! { Badge { variant: BadgeVariant::Store, {other.badge()} } },
    }
}

#[component]
fn Toolbar(#[props(into)] title: String, count: usize) -> Element {
    let app = use_app();
    let search = app.search.read().clone();
    let sort = app.settings.read().sort.clone();
    let size = app.settings.read().card_size.to_string();
    let scanning = *app.scanning.read();
    rsx! {
        div { class: "flex flex-wrap items-center gap-3 mb-6",
            div { class: "flex items-baseline gap-3 mr-auto",
                h1 { class: "text-2xl font-bold tracking-tight font-display", "{title}" }
                span { class: "text-sm text-muted-foreground", {tf("lib.count", &[("n", &count.to_string())])} }
            }
            div { class: "relative w-64",
                span { class: "absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none",
                    Icon { name: "search", class: "size-4" }
                }
                Input {
                    class: "pl-9 h-9",
                    placeholder: t("lib.search"),
                    value: search,
                    oninput: move |v: String| {
                        let mut s = app.search;
                        s.set(v);
                    },
                }
            }
            GenreSelect {}
            SortSelect { value: sort }
            Segmented {
                options: vec![("1".to_string(), "S".to_string()), ("2".to_string(), "M".to_string()), ("3".to_string(), "L".to_string())],
                value: size,
                onchange: move |v: String| {
                    let n = v.parse::<u8>().unwrap_or(2);
                    app.update(|s| s.card_size = n);
                },
            }
            Button {
                variant: ButtonVariant::Outline,
                size: ButtonSize::Icon,
                title: t("lib.reload_tip"),
                disabled: scanning,
                onclick: move |_| rescan(app),
                if scanning {
                    Spinner {}
                } else {
                    Icon { name: "refresh", class: "size-4" }
                }
            }
        }
    }
}

#[component]
fn SortSelect(value: String) -> Element {
    let app = use_app();
    let sort_title = t("sort.title");
    rsx! {
        select {
            class: "h-9 rounded-md border border-input bg-background/60 px-3 text-sm outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 cursor-pointer",
            value: "{value}",
            title: "{sort_title}",
            onchange: move |e| {
                let v = e.value();
                app.update(|s| s.sort = v);
            },
            option { value: "name", selected: value == "name", {t("sort.name")} }
            option { value: "recent", selected: value == "recent", {t("sort.recent")} }
            option { value: "playtime", selected: value == "playtime", {t("sort.playtime")} }
            option { value: "installed", selected: value == "installed", {t("sort.installed")} }
            option { value: "added", selected: value == "added", {t("sort.added")} }
        }
    }
}

/// Genre-Filter (nur sichtbar, wenn Genres bekannt sind)
#[component]
fn GenreSelect() -> Element {
    let app = use_app();
    let genres = all_genres(app);
    let current = app.genre.read().clone();
    if genres.is_empty() {
        return rsx! {};
    }
    rsx! {
        select {
            class: "h-9 max-w-44 rounded-md border border-input bg-background/60 px-3 text-sm outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40 cursor-pointer",
            title: t("genre.title"),
            onchange: move |e| {
                let mut g = app.genre;
                g.set(e.value());
            },
            option { value: "", selected: current.is_empty(), {t("genre.all")} }
            {genres.into_iter().map(|g| {
                let sel = g.eq_ignore_ascii_case(&current);
                rsx! { option { key: "{g}", value: "{g}", selected: sel, "{g}" } }
            })}
        }
    }
}

/// Hinweis: Epic-Konto verbinden, damit alle Epic-Spiele erscheinen
#[component]
fn EpicHint() -> Element {
    let app = use_app();
    let s = app.settings.read();
    let show = !cfg!(target_os = "linux")
        && s.epic_enabled
        && s.epic_auth.is_none()
        && !s.epic_hint_dismissed
        && (app.epic.read().found || app.games.read().iter().any(|g| g.source == Source::Epic));
    drop(s);
    if !show {
        return rsx! {};
    }
    rsx! {
        div { class: "mb-5 flex items-center gap-4 rounded-xl border border-primary/30 bg-primary/10 px-4 py-3 text-sm animate-slide-up",
            div { class: "size-9 shrink-0 rounded-lg bg-[#2a2a2a] text-white grid place-items-center",
                Icon { name: "layers", class: "size-5" }
            }
            div { class: "flex-1 min-w-0",
                div { class: "font-semibold", {t("epic.hint_title")} }
                div { class: "text-muted-foreground text-xs mt-0.5", {t("epic.hint_text")} }
            }
            Button { variant: ButtonVariant::Glow, size: ButtonSize::Sm, onclick: move |_| app.open(Modal::EpicConnect),
                {t("epic.hint_button")}
            }
            button {
                class: "p-1 text-muted-foreground hover:text-foreground cursor-pointer",
                title: t("lib.dismiss"),
                onclick: move |_| app.update(|s| s.epic_hint_dismissed = true),
                Icon { name: "x", class: "size-4" }
            }
        }
    }
}

#[component]
fn WarningBanner() -> Element {
    let app = use_app();
    let warnings = app.warnings.read().clone();
    rsx! {
        EpicHint {}
        {warnings.into_iter().enumerate().map(|(i, w)| {
            let title = w.title.clone();
            let what = w.what_happened.clone();
            let w2 = w.clone();
            rsx! {
                div { key: "{i}", class: "mb-5 flex items-center gap-3 rounded-xl border border-warning/30 bg-warning/10 px-4 py-3 text-sm animate-slide-up",
                    Icon { name: "info", class: "size-5 text-warning" }
                    div { class: "flex-1 min-w-0",
                        div { class: "font-semibold", "{title}" }
                        div { class: "text-muted-foreground text-xs mt-0.5 truncate", "{what}" }
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        onclick: move |_| app.show_problem(w2.clone(), None),
                        {t("lib.how_fix")}
                    }
                    button {
                        class: "p-1 text-muted-foreground hover:text-foreground cursor-pointer",
                        title: t("lib.dismiss"),
                        onclick: move |_| {
                            let mut ws = app.warnings;
                            ws.write().retain(|x| x.title != w.title);
                        },
                        Icon { name: "x", class: "size-4" }
                    }
                }
            }
        })}
    }
}

#[component]
fn EmptyState() -> Element {
    let app = use_app();
    let filter = *app.filter.read();
    let searching = !app.search.read().trim().is_empty();
    let loaded = *app.loaded.read();
    if !loaded {
        return rsx! {
            div { class: "grid gap-5", style: "grid-template-columns: repeat(auto-fill, minmax(172px, 1fr));",
                for i in 0..12 {
                    div { key: "{i}", class: "aspect-[2/3] rounded-xl skeleton" }
                }
            }
        };
    }
    let (title, text) = if searching {
        (t("empty.search.title"), t("empty.search.text"))
    } else {
        match filter {
            Filter::Favorites => (t("empty.fav.title"), t("empty.fav.text")),
            Filter::Recent => (t("empty.recent.title"), t("empty.recent.text")),
            Filter::Custom => (t("empty.custom.title"), t("empty.custom.text")),
            Filter::Hidden => (t("empty.hidden.title"), t("empty.hidden.text")),
            Filter::Steam => (t("empty.steam.title"), t("empty.steam.text")),
            Filter::Epic => (t("empty.epic.title"), t("empty.epic.text")),
            Filter::Roblox => (t("empty.roblox.title"), t("empty.roblox.text")),
            Filter::Tag | Filter::Collection => (t("empty.group.title"), t("empty.group.text")),
            Filter::Store(Source::Emulator) => (t("empty.emu.title"), t("empty.emu.text")),
            Filter::Store(src) => (tf("empty.store.title", &[("store", &src.label())]), tf("store.client_missing", &[("store", &src.label())])),
            _ => (t("empty.all.title"), t("empty.all.text")),
        }
    };
    rsx! {
        div { class: "flex flex-col items-center justify-center text-center py-24 animate-fade-in",
            div { class: "size-20 rounded-3xl bg-muted grid place-items-center mb-5 text-muted-foreground",
                Icon { name: "gamepad", class: "size-10" }
            }
            h2 { class: "text-xl font-bold", "{title}" }
            p { class: "text-muted-foreground mt-2 max-w-md text-sm leading-relaxed", "{text}" }
            div { class: "flex gap-3 mt-6",
                Button { variant: ButtonVariant::Glow, onclick: move |_| app.open(Modal::EditGame(None)),
                    Icon { name: "plus", class: "size-4" }
                    {t("lib.add_custom")}
                }
                Button { variant: ButtonVariant::Outline, onclick: move |_| app.open(Modal::FolderScan),
                    Icon { name: "folder", class: "size-4" }
                    {t("scan.button")}
                }
                Button { variant: ButtonVariant::Outline, onclick: move |_| rescan(app),
                    Icon { name: "refresh", class: "size-4" }
                    {t("lib.reload")}
                }
            }
        }
    }
}

/// Karte im MAZ-Stil
#[component]
pub fn GameCard(game: Game, favorite: bool, show_name: bool, #[props(into)] covers: Vec<String>) -> Element {
    let app = use_app();
    let more = app.dupes.read().get(&game.id).map(|v| v.len()).unwrap_or(0);
    let id = game.id.clone();
    let id_play = game.id.clone();
    let name = game.name.clone();
    let installed = game.installed;
    let img_cls = if installed {
        "absolute inset-0 size-full object-cover transition-transform duration-500 group-hover:scale-105"
    } else {
        "absolute inset-0 size-full object-cover grayscale opacity-60 transition-transform duration-500 group-hover:scale-105"
    };
    let play_icon = if installed { "play" } else { "download" };
    let play_title = t(if installed { "game.play" } else { "game.install" });
    let g2 = game.clone();
    let g3 = game.clone();
    rsx! {
        div {
            class: "group relative cursor-pointer animate-pop-in",
            onclick: move |_| app.open(Modal::Detail(id.clone())),
            div { class: "relative aspect-[2/3] rounded-xl overflow-hidden bg-muted ring-1 ring-white/5 shadow-lg shadow-black/30 transition-all duration-300 group-hover:-translate-y-1.5 group-hover:ring-2 group-hover:ring-primary group-hover:shadow-2xl card-shine",
                ArtImage {
                    srcs: covers.clone(),
                    class: img_cls,
                    alt: name.clone(),
                    fallback: rsx! { CoverFallback { game: g2.clone(), show_name: !show_name } },
                }
                div { class: "absolute inset-x-0 bottom-0 h-2/5 bg-linear-to-t from-black/80 to-transparent opacity-70 group-hover:opacity-100 transition-opacity" }
                div { class: "absolute top-2 left-2 flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity",
                    SourceBadge { source: game.source }
                    if more > 0 {
                        span { class: "rounded-md bg-black/70 px-1.5 py-0.5 text-[10px] font-semibold text-white", title: t("dupes.title"), "+{more}" }
                    }
                }
                if favorite {
                    div { class: "absolute top-2 right-2 text-yellow-400 drop-shadow",
                        Icon { name: "star", class: "size-5", filled: true }
                    }
                }
                if !installed {
                    div { class: "absolute top-2 right-2 rounded-md bg-black/70 px-1.5 py-0.5 text-[10px] font-semibold text-white/80", {t("badge.not_installed")} }
                }
                div { class: "absolute inset-0 grid place-items-center opacity-0 group-hover:opacity-100 transition-opacity duration-200",
                    button {
                        class: "size-14 rounded-full maz-gradient-bg text-white grid place-items-center shadow-2xl shadow-black/50 scale-75 group-hover:scale-100 transition-transform duration-300 cursor-pointer hover:brightness-110",
                        title: "{play_title}",
                        onclick: move |e| {
                            e.stop_propagation();
                            play(app, id_play.clone());
                        },
                        Icon { name: play_icon, class: "size-6 translate-x-0.5", filled: true }
                    }
                }
                div { class: "absolute left-2 bottom-2 opacity-0 group-hover:opacity-100 transition-opacity",
                    GameIcon { game: g3.clone(), class: "size-7 rounded-md ring-1 ring-white/20 shadow" }
                }
            }
            if show_name {
                div { class: "mt-2.5 px-0.5 text-[13px] font-medium leading-snug line-clamp-2 text-foreground/90 group-hover:text-foreground", "{name}" }
            }
        }
    }
}

#[component]
fn GameGrid(games: Vec<Game>) -> Element {
    let app = use_app();
    let s = app.settings.read();
    let min = min_card_width(&s);
    let show_names = s.show_names;
    let cards: Vec<(Game, bool, Vec<String>)> = games
        .into_iter()
        .map(|g| {
            let fav = s.is_favorite(&g.id);
            let covers = effective_cover(&s, &g);
            (g, fav, covers)
        })
        .collect();
    drop(s);
    rsx! {
        div { class: "grid gap-x-5 gap-y-6", style: "grid-template-columns: repeat(auto-fill, minmax({min}px, 1fr));",
            {cards.into_iter().map(|(g, fav, covers)| {
                let key = g.id.clone();
                rsx! { GameCard { key: "{key}", game: g, favorite: fav, show_name: show_names, covers } }
            })}
        }
    }
}

// ======================================================================
// MAZ-Ansicht (eigene)
// ======================================================================

#[component]
fn NavItem(filter: Filter, count: usize, #[props(into, default)] arg: String, #[props(into, default)] label: String) -> Element {
    let app = use_app();
    let active = *app.filter.read() == filter && *app.page.read() == Page::Library && (arg.is_empty() || *app.filter_arg.read() == arg);
    let cls = if active {
        "w-full flex items-center gap-3 px-3 h-9 rounded-lg text-sm font-medium bg-sidenav-accent text-foreground cursor-pointer relative"
    } else {
        "w-full flex items-center gap-3 px-3 h-9 rounded-lg text-sm text-sidenav-foreground/80 hover:bg-sidenav-accent/60 hover:text-foreground cursor-pointer relative"
    };
    let label = if label.is_empty() { filter.label() } else { label };
    let icon = filter.icon();
    rsx! {
        button {
            class: "{cls}",
            onclick: move |_| app.set_filter(filter, &arg),
            if active {
                span { class: "absolute left-0 top-2 bottom-2 w-1 rounded-full maz-gradient-bg" }
            }
            Icon { name: icon, class: "size-4" }
            span { class: "flex-1 text-left truncate", "{label}" }
            if count > 0 {
                span { class: "text-[11px] text-muted-foreground tabular-nums", "{count}" }
            }
        }
    }
}

#[component]
pub fn MazSidebar() -> Element {
    let app = use_app();
    let settings_active = matches!(*app.page.read(), Page::Settings(_));
    let set_cls = if settings_active {
        "w-full flex items-center gap-3 px-3 h-9 rounded-lg text-sm font-medium bg-sidenav-accent text-foreground cursor-pointer"
    } else {
        "w-full flex items-center gap-3 px-3 h-9 rounded-lg text-sm text-sidenav-foreground/80 hover:bg-sidenav-accent/60 hover:text-foreground cursor-pointer"
    };
    rsx! {
        aside { class: "w-60 shrink-0 flex flex-col bg-sidenav border-r border-border/60",
            div { class: "flex-1 overflow-y-auto px-3 py-4 space-y-6",
                div { class: "space-y-0.5",
                    div { class: "px-3 pb-2 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground", {t("nav.library")} }
                    NavItem { filter: Filter::All, count: count_for(app, Filter::All) }
                    NavItem { filter: Filter::Installed, count: count_for(app, Filter::Installed) }
                    NavItem { filter: Filter::Favorites, count: count_for(app, Filter::Favorites) }
                    NavItem { filter: Filter::Recent, count: count_for(app, Filter::Recent) }
                }
                div { class: "space-y-0.5",
                    div { class: "px-3 pb-2 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground", {t("nav.platforms")} }
                    NavItem { filter: Filter::Steam, count: count_for(app, Filter::Steam) }
                    NavItem { filter: Filter::Epic, count: count_for(app, Filter::Epic) }
                    // Unter Windows kein eigener Roblox-Eintrag (Roblox-Spiele stehen in „Alle Spiele“)
                    if !cfg!(windows) && count_for(app, Filter::Roblox) > 0 {
                        NavItem { filter: Filter::Roblox, count: count_for(app, Filter::Roblox) }
                    }
                    {stores_with_games(app).into_iter().map(|src| {
                        let key = source_value(src);
                        rsx! { NavItem { key: "{key}", filter: Filter::Store(src), count: count_for(app, Filter::Store(src)) } }
                    })}
                    NavItem { filter: Filter::Custom, count: count_for(app, Filter::Custom) }
                }
                CollectionsNav {}
                TagsNav {}
                if count_for(app, Filter::Hidden) > 0 {
                    div { class: "space-y-0.5",
                        NavItem { filter: Filter::Hidden, count: count_for(app, Filter::Hidden) }
                    }
                }
            }
            div { class: "p-3 space-y-2 border-t border-border/60",
                Button { variant: ButtonVariant::Glow, class: "w-full", onclick: move |_| app.open(Modal::EditGame(None)),
                    Icon { name: "plus", class: "size-4" }
                    {t("lib.add")}
                }
                button { class: "{set_cls}", onclick: move |_| app.go(Page::Settings(SettingsTab::Design)),
                    Icon { name: "settings", class: "size-4" }
                    {t("nav.settings")}
                }
            }
        }
    }
}

/// Eigene Sammlungen in der Seitenleiste (+ neue anlegen)
#[component]
fn CollectionsNav() -> Element {
    let app = use_app();
    let mut adding = use_signal(|| false);
    let mut name = use_signal(String::new);
    let cols = app.settings.read().collections.clone();
    let mut create = move || {
        let n = name.peek().trim().to_string();
        if !n.is_empty() {
            let id = create_collection(app, &n);
            app.set_filter(Filter::Collection, &id);
        }
        name.set(String::new());
        adding.set(false);
    };
    rsx! {
        div { class: "space-y-0.5",
            div { class: "flex items-center px-3 pb-2",
                span { class: "flex-1 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground", {t("nav.collections")} }
                button {
                    class: "p-0.5 rounded text-muted-foreground hover:text-foreground cursor-pointer",
                    title: t("coll.new"),
                    onclick: move |_| adding.set(!adding()),
                    Icon { name: "plus", class: "size-3.5" }
                }
            }
            if adding() {
                div { class: "px-1 pb-1",
                    input {
                        class: "w-full h-8 rounded-md border border-input bg-background/60 px-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring/40",
                        placeholder: t("coll.name_ph"),
                        autofocus: true,
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                        onkeydown: move |e| {
                            if e.key() == Key::Enter {
                                create();
                            } else if e.key() == Key::Escape {
                                adding.set(false);
                            }
                        },
                    }
                }
            }
            if cols.is_empty() && !adding() {
                div { class: "px-3 text-xs text-muted-foreground/70", {t("coll.none")} }
            }
            {cols.into_iter().map(|c| {
                let n = count_with(app, Filter::Collection, &c.id);
                let key = c.id.clone();
                rsx! { NavItem { key: "{key}", filter: Filter::Collection, count: n, arg: c.id.clone(), label: c.name.clone() } }
            })}
        }
    }
}

/// Umbenennen / Löschen der gerade gezeigten Sammlung
#[component]
fn CollectionBar() -> Element {
    let app = use_app();
    let id = app.filter_arg.read().clone();
    let current = app.settings.read().collections.iter().find(|c| c.id == id).map(|c| c.name.clone()).unwrap_or_default();
    let renaming = use_signal(|| None::<String>);
    let (id_r, id_d) = (id.clone(), id.clone());
    rsx! {
        div { class: "flex flex-wrap items-center gap-2 -mt-3 mb-6",
            {match renaming() {
                Some(v) => rsx! {
                    input {
                        class: "h-8 w-64 rounded-md border border-input bg-background/60 px-2 text-sm outline-none focus-visible:ring-2 focus-visible:ring-ring/40",
                        autofocus: true,
                        value: "{v}",
                        oninput: move |e| {
                            let mut r = renaming;
                            r.set(Some(e.value()));
                        },
                        onkeydown: move |e| {
                            let mut r = renaming;
                            if e.key() == Key::Enter {
                                if let Some(n) = r.peek().clone() {
                                    rename_collection(app, &id_r, &n);
                                }
                                r.set(None);
                            } else if e.key() == Key::Escape {
                                r.set(None);
                            }
                        },
                    }
                },
                None => rsx! {
                    Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm,
                        onclick: move |_| {
                            let mut r = renaming;
                            r.set(Some(current.clone()));
                        },
                        Icon { name: "pencil", class: "size-3.5" }
                        {t("coll.rename")}
                    }
                },
            }}
            Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, class: "text-destructive hover:text-destructive",
                onclick: move |_| delete_collection(app, &id_d),
                Icon { name: "trash", class: "size-3.5" }
                {t("coll.delete")}
            }
            span { class: "text-xs text-muted-foreground", {t("coll.delete_hint")} }
        }
    }
}

/// Eigene Tags in der Seitenleiste
#[component]
fn TagsNav() -> Element {
    let app = use_app();
    let tags = app.settings.read().all_tags();
    if tags.is_empty() {
        return rsx! {};
    }
    rsx! {
        div { class: "space-y-0.5",
            div { class: "px-3 pb-2 text-[11px] font-semibold uppercase tracking-wider text-muted-foreground", {t("nav.tags")} }
            {tags.into_iter().map(|tag| {
                let n = count_with(app, Filter::Tag, &tag);
                let key = tag.clone();
                rsx! { NavItem { key: "{key}", filter: Filter::Tag, count: n, arg: tag.clone(), label: tag.clone() } }
            })}
        }
    }
}

#[component]
fn ContinueHero() -> Element {
    let app = use_app();
    let s = app.settings.read();
    let last = app
        .games
        .read()
        .iter()
        .filter(|g| g.installed && s.last_played(&g.id) > 0 && !s.is_hidden(&g.id))
        .max_by_key(|g| s.last_played(&g.id))
        .cloned();
    let Some(g) = last else { return rsx! {} };
    let mut hero = g.art.hero.clone();
    hero.extend(effective_cover(&s, &g));
    let ago = sys::format_ago(s.last_played(&g.id));
    drop(s);
    let name = g.name.clone();
    let logos = g.art.logo.clone();
    let id = g.id.clone();
    let id2 = g.id.clone();
    let grad = super::art::gradient_for(&g.name);
    rsx! {
        div { class: "relative h-64 rounded-2xl overflow-hidden mb-8 ring-1 ring-white/5 shadow-2xl shadow-black/40 group animate-slide-up",
            ArtImage {
                srcs: hero,
                class: "absolute inset-0 size-full object-cover transition-transform duration-[2000ms] group-hover:scale-105",
                fallback: rsx! { div { class: "absolute inset-0 {grad}" } },
            }
            div { class: "absolute inset-0 bg-linear-to-r from-black/85 via-black/40 to-transparent" }
            div { class: "absolute inset-0 p-8 flex flex-col justify-end gap-4",
                div { class: "text-xs font-semibold uppercase tracking-[0.2em] text-white/70", {tf("hero.continue", &[("ago", &ago)])} }
                div { class: "max-w-[420px] h-20 flex items-end",
                    ArtImage {
                        srcs: logos,
                        class: "max-h-20 max-w-full object-contain object-left drop-shadow-2xl",
                        fallback: rsx! { h2 { class: "text-4xl font-black text-white drop-shadow-lg leading-tight font-display", "{name}" } },
                    }
                }
                div { class: "flex gap-3",
                    Button { variant: ButtonVariant::Glow, size: ButtonSize::Lg, onclick: move |_| play(app, id.clone()),
                        Icon { name: "play", class: "size-5", filled: true }
                        {t("game.play")}
                    }
                    Button { variant: ButtonVariant::Secondary, size: ButtonSize::Lg, class: "bg-white/10 text-white hover:bg-white/20 backdrop-blur", onclick: move |_| app.open(Modal::Detail(id2.clone())),
                        {t("game.details")}
                    }
                }
            }
        }
    }
}

#[component]
fn MazLayout() -> Element {
    let app = use_app();
    let games = visible_games(app);
    let filter = *app.filter.read();
    let searching = !app.search.read().trim().is_empty();
    let count = games.len();
    let title = filter_title(app);
    rsx! {
        div { class: "flex h-full min-h-0",
            MazSidebar {}
            main { class: "flex-1 min-w-0 overflow-y-auto maz-ambient",
                div { class: "px-8 py-7 max-w-[1800px] mx-auto",
                    WarningBanner {}
                    if filter == Filter::All && !searching {
                        ContinueHero {}
                    }
                    Toolbar { title, count }
                    if filter == Filter::Collection {
                        CollectionBar {}
                    }
                    if games.is_empty() {
                        EmptyState {}
                    } else {
                        GameGrid { games }
                    }
                }
            }
        }
    }
}

// ======================================================================
// Steam-Stil
// ======================================================================

#[component]
fn SteamLayout() -> Element {
    let app = use_app();
    let games = visible_games(app);
    let selected = app.selected.read().clone();
    let search = app.search.read().clone();
    let filter = *app.filter.read();
    let count = games.len();
    let sel_game = selected.as_ref().and_then(|id| app.games.read().iter().find(|g| &g.id == id).cloned());
    let rows: Vec<Game> = games.clone();
    rsx! {
        div { class: "flex h-full min-h-0 text-[13px]",
            aside { class: "w-72 shrink-0 flex flex-col bg-sidenav border-r border-black/40",
                // Kopf: Startseite + Filter
                div { class: "p-2 flex flex-col gap-2 border-b border-black/30",
                    button {
                        class: "flex items-center gap-2 px-3 h-9 rounded-sm bg-secondary/70 hover:bg-secondary text-foreground font-semibold text-xs tracking-wide uppercase cursor-pointer",
                        onclick: move |_| {
                            let mut s = app.selected;
                            s.set(None);
                            app.go(Page::Library);
                        },
                        Icon { name: "home", class: "size-4" }
                        {t("steam.home")}
                    }
                    div { class: "flex gap-2",
                        select {
                            class: "flex-1 h-8 rounded-sm bg-input/60 border border-black/30 px-2 text-xs outline-none cursor-pointer",
                            onchange: move |e| {
                                let f = match e.value().as_str() {
                                    "installed" => Filter::Installed,
                                    "steam" => Filter::Steam,
                                    "epic" => Filter::Epic,
                                    "roblox" => Filter::Roblox,
                                    "custom" => Filter::Custom,
                                    "fav" => Filter::Favorites,
                                    "recent" => Filter::Recent,
                                    "hidden" => Filter::Hidden,
                                    other => {
                                        if let Some(c) = other.strip_prefix("coll:") {
                                            app.set_filter(Filter::Collection, c);
                                            return;
                                        }
                                        if let Some(tag) = other.strip_prefix("tag:") {
                                            app.set_filter(Filter::Tag, tag);
                                            return;
                                        }
                                        stores_with_games(app).into_iter().find(|s| source_value(*s) == other).map(Filter::Store).unwrap_or(Filter::All)
                                    }
                                };
                                app.set_filter(f, "");
                            },
                            option { value: "all", selected: filter == Filter::All, {t("steam.f_all")} }
                            option { value: "installed", selected: filter == Filter::Installed, {t("filter.installed").to_uppercase()} }
                            option { value: "steam", selected: filter == Filter::Steam, "STEAM" }
                            option { value: "epic", selected: filter == Filter::Epic, "EPIC GAMES" }
                            if !cfg!(windows) {
                                option { value: "roblox", selected: filter == Filter::Roblox, "ROBLOX" }
                            }
                            {stores_with_games(app).into_iter().map(|src| {
                                let k = source_value(src);
                                let label = src.label().to_uppercase();
                                rsx! { option { key: "{k}", value: "{k}", selected: filter == Filter::Store(src), "{label}" } }
                            })}
                            option { value: "custom", selected: filter == Filter::Custom, {t("filter.custom").to_uppercase()} }
                            option { value: "fav", selected: filter == Filter::Favorites, {t("filter.favorites").to_uppercase()} }
                            option { value: "recent", selected: filter == Filter::Recent, {t("filter.recent").to_uppercase()} }
                            option { value: "hidden", selected: filter == Filter::Hidden, {t("filter.hidden").to_uppercase()} }
                            {app.settings.read().collections.iter().map(|c| {
                                let v = format!("coll:{}", c.id);
                                let sel = filter == Filter::Collection && *app.filter_arg.read() == c.id;
                                let label = format!("▸ {}", c.name.to_uppercase());
                                rsx! { option { key: "{v}", value: "{v}", selected: sel, "{label}" } }
                            })}
                            {app.settings.read().all_tags().into_iter().map(|tag| {
                                let v = format!("tag:{tag}");
                                let sel = filter == Filter::Tag && *app.filter_arg.read() == tag;
                                let label = format!("# {}", tag.to_uppercase());
                                rsx! { option { key: "{v}", value: "{v}", selected: sel, "{label}" } }
                            })}
                        }
                    }
                    div { class: "relative",
                        span { class: "absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none",
                            Icon { name: "search", class: "size-3.5" }
                        }
                        Input {
                            class: "pl-8 h-8 rounded-sm text-xs bg-input/40 border-black/30",
                            placeholder: t("steam.search"),
                            value: search,
                            oninput: move |v: String| {
                                let mut s = app.search;
                                s.set(v);
                            },
                        }
                    }
                }
                div { class: "px-3 pt-3 pb-1 text-[11px] uppercase tracking-wider text-muted-foreground", {format!("— {}", tf("lib.count", &[("n", &count.to_string())]))} }
                div { class: "flex-1 overflow-y-auto pb-2",
                    {rows.into_iter().map(|g| {
                        let id = g.id.clone();
                        let id2 = g.id.clone();
                        let active = selected.as_deref() == Some(g.id.as_str());
                        let cls = if active {
                            "w-full flex items-center gap-2 px-3 h-7 text-left bg-[#3d4450] text-white cursor-pointer"
                        } else if g.installed {
                            "w-full flex items-center gap-2 px-3 h-7 text-left text-foreground/90 hover:bg-white/5 cursor-pointer"
                        } else {
                            "w-full flex items-center gap-2 px-3 h-7 text-left text-muted-foreground/70 hover:bg-white/5 cursor-pointer"
                        };
                        let name = g.name.clone();
                        rsx! {
                            button {
                                key: "{id}",
                                class: "{cls}",
                                onclick: move |_| {
                                    let mut s = app.selected;
                                    s.set(Some(id.clone()));
                                },
                                ondoubleclick: move |_| play(app, id2.clone()),
                                GameIcon { game: g.clone(), class: "size-4 rounded-[2px]" }
                                span { class: "truncate", "{name}" }
                            }
                        }
                    })}
                }
                div { class: "p-2 border-t border-black/30 flex gap-2",
                    Button { variant: ButtonVariant::Secondary, size: ButtonSize::Sm, class: "flex-1 rounded-sm", onclick: move |_| app.open(Modal::EditGame(None)),
                        Icon { name: "plus", class: "size-3.5" }
                        {t("lib.add").to_uppercase()}
                    }
                    Button { variant: ButtonVariant::Secondary, size: ButtonSize::IconSm, class: "rounded-sm", title: "Einstellungen", onclick: move |_| app.go(Page::Settings(SettingsTab::Design)),
                        Icon { name: "settings", class: "size-4" }
                    }
                    Button { variant: ButtonVariant::Secondary, size: ButtonSize::IconSm, class: "rounded-sm", title: t("lib.reload"), onclick: move |_| rescan(app),
                        Icon { name: "refresh", class: "size-4" }
                    }
                }
            }
            main { class: "flex-1 min-w-0 overflow-y-auto bg-linear-to-b from-[#2a475e]/40 to-transparent",
                {match sel_game {
                    Some(g) => {
                        let k = g.id.clone();
                        rsx! { {std::iter::once(rsx! { SteamDetail { key: "{k}", game: g } })} }
                    }
                    None => rsx! { SteamHome { games } },
                }}
            }
        }
    }
}

#[component]
fn SteamCapsule(game: Game, covers: Vec<String>, #[props(default = false)] big: bool) -> Element {
    let app = use_app();
    let id = game.id.clone();
    let id2 = game.id.clone();
    let name = game.name.clone();
    let img_cls = if game.installed {
        "absolute inset-0 size-full object-cover"
    } else {
        "absolute inset-0 size-full object-cover grayscale opacity-60"
    };
    let g2 = game.clone();
    rsx! {
        div {
            class: "group relative aspect-[2/3] cursor-pointer transition-all duration-200 hover:-translate-y-1 hover:brightness-110 hover:shadow-[0_8px_30px_rgba(102,192,244,0.25)] shadow-lg shadow-black/50",
            title: "{name}",
            onclick: move |_| {
                let mut s = app.selected;
                s.set(Some(id.clone()));
            },
            ondoubleclick: move |_| play(app, id2.clone()),
            ArtImage {
                srcs: covers,
                class: img_cls,
                alt: name.clone(),
                fallback: rsx! { CoverFallback { game: g2.clone(), show_name: true } },
            }
            if big {
                div { class: "absolute inset-x-0 bottom-0 bg-linear-to-t from-black/90 to-transparent p-2 pt-8 opacity-0 group-hover:opacity-100 transition-opacity",
                    div { class: "text-xs text-white font-semibold truncate", "{name}" }
                }
            }
        }
    }
}

#[component]
fn SteamHome(games: Vec<Game>) -> Element {
    let app = use_app();
    let s = app.settings.read();
    let mut recent: Vec<Game> = app
        .games
        .read()
        .iter()
        .filter(|g| s.last_played(&g.id) > 0 && !s.is_hidden(&g.id))
        .cloned()
        .collect();
    recent.sort_by_key(|g| std::cmp::Reverse(s.last_played(&g.id)));
    recent.truncate(8);
    let recent: Vec<(Game, Vec<String>)> = recent.into_iter().map(|g| { let c = effective_cover(&s, &g); (g, c) }).collect();
    let all: Vec<(Game, Vec<String>)> = games.into_iter().map(|g| { let c = effective_cover(&s, &g); (g, c) }).collect();
    let min = min_card_width(&s);
    drop(s);
    let n = all.len();
    rsx! {
        div { class: "px-8 py-6",
            WarningBanner {}
            if !recent.is_empty() {
                div { class: "mb-8",
                    div { class: "text-[13px] text-foreground/80 mb-3 tracking-wide", {t("filter.recent")} }
                    div { class: "grid gap-4", style: "grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));",
                        {recent.into_iter().map(|(g, c)| {
                            let key = g.id.clone();
                            rsx! { SteamCapsule { key: "{key}", game: g, covers: c, big: true } }
                        })}
                    }
                }
            }
            div { class: "flex items-center gap-3 mb-3",
                div { class: "text-[13px] text-foreground/80 tracking-wide", {format!("{} ({n})", t("filter.all"))} }
                div { class: "flex-1 h-px bg-white/10" }
                SortSelect { value: app.settings.read().sort.clone() }
            }
            if n == 0 {
                EmptyState {}
            } else {
                div { class: "grid gap-4", style: "grid-template-columns: repeat(auto-fill, minmax({min}px, 1fr));",
                    {all.into_iter().map(|(g, c)| {
                        let key = g.id.clone();
                        rsx! { SteamCapsule { key: "{key}", game: g, covers: c } }
                    })}
                }
            }
        }
    }
}

// ======================================================================
// Epic-Stil
// ======================================================================

#[component]
fn EpicNav(#[props(into)] label: String, icon: &'static str, active: bool, onclick: EventHandler<()>) -> Element {
    let cls = if active {
        "w-full flex items-center gap-3 px-4 h-11 rounded-lg text-sm font-medium bg-accent text-white cursor-pointer"
    } else {
        "w-full flex items-center gap-3 px-4 h-11 rounded-lg text-sm text-sidenav-foreground hover:bg-accent/60 hover:text-white cursor-pointer"
    };
    rsx! {
        button { class: "{cls}", onclick: move |_| onclick.call(()),
            Icon { name: icon, class: "size-5" }
            "{label}"
        }
    }
}

#[component]
fn EpicLayout() -> Element {
    let app = use_app();
    let games = visible_games(app);
    let filter = *app.filter.read();
    let page = *app.page.read();
    let search = app.search.read().clone();
    let sort = app.settings.read().sort.clone();
    let mut chips = vec![Filter::All, Filter::Installed, Filter::Favorites, Filter::Steam, Filter::Epic];
    if !cfg!(windows) && count_for(app, Filter::Roblox) > 0 {
        chips.push(Filter::Roblox);
    }
    chips.extend(stores_with_games(app).into_iter().map(Filter::Store));
    chips.extend([Filter::Custom, Filter::Recent]);
    let coll_chips: Vec<(String, String)> = app.settings.read().collections.iter().map(|c| (c.id.clone(), c.name.clone())).collect();
    let arg_now = app.filter_arg.read().clone();
    let s = app.settings.read();
    let min = min_card_width(&s) + 20;
    let cards: Vec<(Game, Vec<String>, bool)> = games
        .into_iter()
        .map(|g| {
            let c = effective_cover(&s, &g);
            let f = s.is_favorite(&g.id);
            (g, c, f)
        })
        .collect();
    drop(s);
    let count = cards.len();
    rsx! {
        div { class: "flex h-full min-h-0",
            aside { class: "w-60 shrink-0 bg-sidenav flex flex-col p-4 gap-1",
                EpicNav { label: t("nav.library"), icon: "grid", active: page == Page::Library && filter != Filter::Custom,
                    onclick: move |_| { let mut f = app.filter; f.set(Filter::All); app.go(Page::Library); } }
                EpicNav { label: t("filter.custom"), icon: "rocket", active: page == Page::Library && filter == Filter::Custom,
                    onclick: move |_| { let mut f = app.filter; f.set(Filter::Custom); app.go(Page::Library); } }
                EpicNav { label: t("nav.settings"), icon: "settings", active: matches!(page, Page::Settings(_)),
                    onclick: move |_| app.go(Page::Settings(SettingsTab::Design)) }
                div { class: "flex-1" }
                Button { variant: ButtonVariant::Default, size: ButtonSize::Lg, class: "w-full", onclick: move |_| app.open(Modal::EditGame(None)),
                    Icon { name: "plus", class: "size-4" }
                    {t("lib.add")}
                }
            }
            main { class: "flex-1 min-w-0 overflow-y-auto",
                div { class: "px-10 py-8 max-w-[1800px] mx-auto",
                    WarningBanner {}
                    div { class: "flex items-center justify-between mb-6",
                        h1 { class: "text-4xl font-bold tracking-tight font-display", {t("nav.library")} }
                        Button { variant: ButtonVariant::Ghost, size: ButtonSize::Icon, title: t("lib.reload"), onclick: move |_| rescan(app),
                            Icon { name: "refresh", class: "size-5" }
                        }
                    }
                    div { class: "flex flex-wrap items-center gap-2 mb-8",
                        {chips.into_iter().map(|c| {
                            let active = c == filter;
                            let cls = if active {
                                "h-9 px-4 rounded-full text-sm font-medium bg-white text-black cursor-pointer"
                            } else {
                                "h-9 px-4 rounded-full text-sm font-medium bg-secondary text-secondary-foreground hover:bg-accent cursor-pointer"
                            };
                            let label = c.label();
                            rsx! {
                                button { key: "{label}", class: "{cls}", onclick: move |_| app.set_filter(c, ""), "{label}" }
                            }
                        })}
                        {coll_chips.into_iter().map(|(cid, name)| {
                            let active = filter == Filter::Collection && arg_now == cid;
                            let cls = if active {
                                "h-9 px-4 rounded-full text-sm font-medium bg-white text-black cursor-pointer"
                            } else {
                                "h-9 px-4 rounded-full text-sm font-medium bg-secondary text-secondary-foreground hover:bg-accent cursor-pointer"
                            };
                            let key = format!("coll-{cid}");
                            rsx! {
                                button { key: "{key}", class: "{cls}", onclick: move |_| app.set_filter(Filter::Collection, &cid), "{name}" }
                            }
                        })}
                        div { class: "flex-1" }
                        span { class: "text-sm text-muted-foreground mr-2", {tf("epic.titles", &[("n", &count.to_string())])} }
                        SortSelect { value: sort }
                        div { class: "relative w-60",
                            span { class: "absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none",
                                Icon { name: "search", class: "size-4" }
                            }
                            Input {
                                class: "pl-9 h-9 rounded-full bg-secondary border-transparent",
                                placeholder: t("epic.search"),
                                value: search,
                                oninput: move |v: String| { let mut s = app.search; s.set(v); },
                            }
                        }
                    }
                    if count == 0 {
                        EmptyState {}
                    } else {
                        div { class: "grid gap-x-6 gap-y-8", style: "grid-template-columns: repeat(auto-fill, minmax({min}px, 1fr));",
                            {cards.into_iter().map(|(g, c, fav)| {
                                let key = g.id.clone();
                                rsx! { EpicCard { key: "{key}", game: g, covers: c, favorite: fav } }
                            })}
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn EpicCard(game: Game, covers: Vec<String>, favorite: bool) -> Element {
    let app = use_app();
    let id = game.id.clone();
    let id2 = game.id.clone();
    let name = game.name.clone();
    let status = if !game.installed {
        t("game.not_installed")
    } else {
        game.source.label()
    };
    let action = t(if game.installed { "epic.start" } else { "game.install" });
    let g2 = game.clone();
    rsx! {
        div { class: "group cursor-pointer animate-pop-in", onclick: move |_| app.open(Modal::Detail(id.clone())),
            div { class: "relative aspect-[3/4] rounded-lg overflow-hidden bg-card transition-all duration-200 group-hover:brightness-110",
                ArtImage {
                    srcs: covers,
                    class: "absolute inset-0 size-full object-cover",
                    alt: name.clone(),
                    fallback: rsx! { CoverFallback { game: g2.clone(), show_name: false } },
                }
                div { class: "absolute inset-0 bg-white/0 group-hover:bg-white/10 transition-colors" }
                if favorite {
                    div { class: "absolute top-2 right-2 text-yellow-400 drop-shadow",
                        Icon { name: "star", class: "size-4", filled: true }
                    }
                }
                div { class: "absolute inset-x-3 bottom-3 opacity-0 translate-y-2 group-hover:opacity-100 group-hover:translate-y-0 transition-all duration-200",
                    button {
                        class: "w-full h-10 rounded-md bg-primary text-primary-foreground text-sm font-semibold hover:brightness-110 cursor-pointer shadow-lg",
                        onclick: move |e| {
                            e.stop_propagation();
                            play(app, id2.clone());
                        },
                        "{action}"
                    }
                }
            }
            div { class: "mt-3 font-semibold text-[15px] leading-tight line-clamp-2", "{name}" }
            div { class: "mt-1 text-xs text-muted-foreground", "{status}" }
        }
    }
}

/// Kleines Logo + Name oben links in der Titelleiste
#[component]
pub fn TitleBrand() -> Element {
    rsx! {
        div { class: "flex items-center gap-2.5 pl-3 pointer-events-none",
            MazLogo { class: "size-6 text-[8px] rounded-md" }
            span { class: "text-[13px] font-semibold tracking-tight", "MAZ Launcher" }
        }
    }
}
