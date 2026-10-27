//! Spiel-Details: als Fenster (MAZ/Epic) oder direkt in der Seite (Steam-Stil).

use super::art::{gradient_for, ArtImage, CoverFallback, GameIcon};
use super::components::*;
use super::icons::Icon;
use super::library::SourceBadge;
use super::state::*;
use crate::model::{Game, LaunchTarget, Source};
use crate::i18n::{t, tf};
use crate::sys;
use dioxus::prelude::*;

struct InfoRow {
    label: String,
    value: String,
}

fn info_rows(app: App, g: &Game) -> Vec<InfoRow> {
    let s = app.settings.read();
    let mut rows = vec![InfoRow { label: t("info.platform"), value: g.source.label() }];
    rows.push(InfoRow {
        label: t("info.status"),
        value: t(if g.installed { "filter.installed" } else { "game.not_installed" }),
    });
    let pt = s.playtime(g);
    if pt > 0 {
        rows.push(InfoRow { label: t("info.playtime"), value: sys::format_playtime(pt) });
    }
    rows.push(InfoRow { label: t("filter.recent"), value: sys::format_ago(s.last_played(&g.id)) });
    if let Some(n) = s.launch_count.get(&g.id) {
        rows.push(InfoRow { label: t("info.launches"), value: format!("{n}×") });
    }
    if let Some(b) = g.size_bytes {
        rows.push(InfoRow { label: t("info.size"), value: sys::format_bytes(b) });
    }
    if let Some(d) = &g.install_dir {
        rows.push(InfoRow { label: t("info.folder"), value: d.clone() });
    }
    let fs = s.first_seen(&g.id);
    drop(s);
    if let Some(i) = app.meta_cache.read().get(&g.id) {
        if !i.developer.is_empty() {
            rows.push(InfoRow { label: t("info.developer"), value: i.developer.clone() });
        }
        if !i.release.is_empty() {
            rows.push(InfoRow { label: t("info.release"), value: i.release.clone() });
        }
    }
    if fs > 0 {
        rows.push(InfoRow { label: t("info.added"), value: sys::format_ago(fs) });
    }
    let s = app.settings.read();
    if let Some(c) = s.custom(&g.id) {
        if !c.args.trim().is_empty() {
            rows.push(InfoRow { label: t("info.args"), value: c.args.clone() });
        }
        rows.push(InfoRow {
            label: t("info.admin"),
            value: t(if c.admin { "common.yes" } else { "common.no" }),
        });
    }
    rows
}

/// Suche nach dem Spiel auf HowLongToBeat (wird im Browser geöffnet)
pub fn hltb_url(name: &str) -> String {
    // Zeichen wie ™ ® weglassen, damit die Suche etwas findet
    let clean: String = name.chars().filter(|c| c.is_alphanumeric() || c.is_whitespace() || matches!(c, '-' | ':' | '\'' | '&')).collect();
    format!("https://howlongtobeat.com/?q={}", sys::url_encode(clean.trim()))
}

#[component]
fn ActionList(game: Game, #[props(default = false)] steam_style: bool) -> Element {
    let app = use_app();
    let s = app.settings.read();
    let fav = s.is_favorite(&game.id);
    let hidden = s.is_hidden(&game.id);
    let has_override = s.cover_overrides.contains_key(&game.id) || s.logo_overrides.contains_key(&game.id);
    let is_custom = s.custom(&game.id).is_some();
    let has_sgdb = !s.sgdb_key.trim().is_empty() && game.source != Source::Roblox;
    drop(s);
    let id11 = game.id.clone();
    let (id12, id13) = (game.id.clone(), game.id.clone());
    let btn = if steam_style { "w-full justify-start rounded-sm" } else { "w-full justify-start" };
    let (id1, id2, id3, id4, id5, id6) = (
        game.id.clone(),
        game.id.clone(),
        game.id.clone(),
        game.id.clone(),
        game.id.clone(),
        game.id.clone(),
    );
    let (id7, id8, id9, id10) = (game.id.clone(), game.id.clone(), game.id.clone(), game.id.clone());
    let rb_place = match game.launch {
        LaunchTarget::Roblox(Some(p)) => Some(p),
        _ => None,
    };
    let is_roblox_exp = rb_place.is_some();
    let is_mc = game.source == Source::Minecraft;
    let mc_target = game.id.strip_prefix("norisk:").map(|id| format!("norisk:{id}")).unwrap_or_else(|| "official".into());
    let rb_page = rb_place.map(crate::roblox::experience_page).unwrap_or_default();
    let dir = game.install_dir.clone();
    let appid = game.steam_appid;
    let fav_label = t(if fav { "act.unfav" } else { "act.fav" });
    let star_cls = if fav { "size-4 text-yellow-400" } else { "size-4" };
    let hide_icon = if hidden { "eye" } else { "eye-off" };
    let hide_label = t(if hidden { "act.unhide" } else { "act.hide" });
    let hltb_name = app.settings.read().meta.get(&game.id).map(|m| m.name.clone()).filter(|n| !n.trim().is_empty()).unwrap_or_else(|| game.name.clone());
    rsx! {
        div { class: "flex flex-col gap-2",
            Button { variant: ButtonVariant::Outline, class: btn, onclick: move |_| toggle_favorite(app, &id1),
                Icon { name: "star", class: star_cls, filled: fav }
                "{fav_label}"
            }
            {dir.map(|d| rsx! {
                Button { variant: ButtonVariant::Outline, class: btn, onclick: move |_| sys::open_folder(&d),
                    Icon { name: "folder", class: "size-4" }
                    {t("fix.folder_game")}
                }
            })}
            Button { variant: ButtonVariant::Outline, class: btn, onclick: move |_| pick_cover(app, &id2),
                Icon { name: "image", class: "size-4" }
                {t("act.cover")}
            }
            Button { variant: ButtonVariant::Outline, class: btn, onclick: move |_| app.open(Modal::EditMeta(id10.clone())),
                Icon { name: "pencil", class: "size-4" }
                {t("act.edit_meta")}
            }
            if has_sgdb {
                Button { variant: ButtonVariant::Outline, class: btn, onclick: move |_| app.open(Modal::SgdbPick(id11.clone())),
                    Icon { name: "image", class: "size-4" }
                    {t("sgdb.button")}
                }
            }
            if has_override {
                Button { variant: ButtonVariant::Ghost, class: btn, onclick: move |_| reset_cover(app, &id3),
                    Icon { name: "refresh", class: "size-4" }
                    {t("act.cover_reset")}
                }
            }
            {appid.map(|a| rsx! {
                Button { variant: ButtonVariant::Outline, class: btn, onclick: move |_| { let _ = sys::open_url(&format!("steam://store/{a}")); },
                    Icon { name: "external", class: "size-4" }
                    {t("act.store")}
                }
            })}
            if !matches!(game.source, Source::Roblox | Source::Minecraft) {
                Button { variant: ButtonVariant::Outline, class: btn, title: t("hltb.tip"),
                    onclick: move |_| { let _ = sys::open_url(&hltb_url(&hltb_name)); },
                    Icon { name: "hourglass", class: "size-4" }
                    {t("hltb.button")}
                }
            }
            if is_custom {
                Button { variant: ButtonVariant::Outline, class: btn, onclick: move |_| app.open(Modal::EditGame(Some(id4.clone()))),
                    Icon { name: "pencil", class: "size-4" }
                    {t("fix.edit")}
                }
            }
            if is_mc {
                Button { variant: ButtonVariant::Outline, class: btn,
                    onclick: move |_| {
                        let mut t = app.mc_target;
                        t.set(mc_target.clone());
                        app.close_modal();
                        app.go(Page::Minecraft);
                    },
                    Icon { name: "cube", class: "size-4" }
                    {t("mc.open_content")}
                }
            }
            if is_roblox_exp {
                Button { variant: ButtonVariant::Outline, class: btn, onclick: move |_| { let _ = sys::open_url(&rb_page); },
                    Icon { name: "external", class: "size-4" }
                    {t("roblox.page")}
                }
                Button { variant: ButtonVariant::Ghost, class: "{btn} text-destructive hover:text-destructive", onclick: move |_| remove_roblox(app, &id9),
                    Icon { name: "trash", class: "size-4" }
                    {t("act.remove")}
                }
            }
            if game.installed && !matches!(game.source, Source::Roblox | Source::Minecraft) && (cfg!(windows) || game.source != Source::Custom) {
                Button { variant: ButtonVariant::Outline, class: btn, onclick: move |_| start_help(app, id7.clone()),
                    Icon { name: "wrench", class: "size-4" }
                    {t("act.help")}
                }
                Button { variant: ButtonVariant::Outline, class: "{btn} hover:text-destructive", onclick: move |_| uninstall(app, id8.clone()),
                    Icon { name: "package", class: "size-4" }
                    {t("act.uninstall")}
                }
            }
            if !matches!(game.source, Source::Roblox) {
                Button { variant: ButtonVariant::Outline, class: btn, onclick: move |_| app.open(Modal::LaunchOpts(id12.clone())),
                    Icon { name: "wrench", class: "size-4" }
                    {t("opts.button")}
                }
            }
            if game.source != Source::Steam && !matches!(game.source, Source::Roblox) {
                Button { variant: ButtonVariant::Outline, class: btn, onclick: move |_| app.open(Modal::AddToSteam(id13.clone())),
                    Icon { name: "gamepad", class: "size-4" }
                    {t("steamsc.button")}
                }
            }
            super::extensions::ExtActions { game: game.clone(), class: btn.to_string() }
            Button { variant: ButtonVariant::Ghost, class: btn, onclick: move |_| toggle_hidden(app, &id5),
                Icon { name: hide_icon, class: "size-4" }
                "{hide_label}"
            }
            if is_custom {
                Button { variant: ButtonVariant::Ghost, class: "{btn} text-destructive hover:text-destructive", onclick: move |_| app.open(Modal::DeleteGame(id6.clone())),
                    Icon { name: "trash", class: "size-4" }
                    {t("act.remove")}
                }
            }
        }
    }
}

/// Beschreibung + Genres (selbst eingetragen oder automatisch geladen)
#[component]
fn AboutGame(id: String) -> Element {
    let app = use_app();
    let own = app.settings.read().meta.get(&id).cloned().unwrap_or_default();
    let auto = app.meta_cache.read().get(&id).cloned().unwrap_or_default();
    let desc = if own.description.trim().is_empty() { auto.description.clone() } else { own.description.clone() };
    let genres = app.genres_of(&id);
    let own_desc = !own.description.trim().is_empty();
    let wiki_page = (!own_desc && auto.source == "wikipedia" && auto.page.starts_with("https://")).then(|| auto.page.clone());
    let deck = auto.deck;
    let (req_min, req_rec) = (auto.req_min.clone(), auto.req_rec.clone());
    let trailer = (!auto.trailer.is_empty()).then(|| (auto.trailer.clone(), auto.trailer_kind.clone(), auto.trailer_thumb.clone()));
    if desc.is_empty() && genres.is_empty() && deck == 0 && req_min.is_empty() && trailer.is_none() {
        return rsx! {};
    }
    let store_page = app.game(&id).and_then(|g| g.steam_appid.or_else(|| {
        app.settings.peek().art_lookup.get(&crate::model::normalize_name(&g.name)).copied().filter(|a| *a != 0)
    })).map(|a| format!("https://store.steampowered.com/app/{a}/"));
    rsx! {
        div { class: "mt-5",
            {trailer.map(|(url, kind, thumb)| rsx! { TrailerBox { url, kind, thumb, page: store_page.clone().unwrap_or_default() } })}
            if !desc.is_empty() {
                p { class: "text-sm leading-relaxed text-foreground/85 select-text", "{desc}" }
            }
            {wiki_page.map(|page| rsx! {
                button {
                    class: "mt-1 text-xs text-muted-foreground hover:text-foreground underline underline-offset-2 cursor-pointer",
                    onclick: move |_| { let _ = sys::open_url(&page); },
                    {t("meta.source_wiki")}
                }
            })}
            if deck > 0 {
                DeckBadge { deck }
            }
            if !genres.is_empty() {
                div { class: "flex flex-wrap gap-1.5 mt-3",
                    {genres.into_iter().map(|g| {
                        let g2 = g.clone();
                        rsx! {
                            button {
                                key: "{g}",
                                class: "h-6 px-2.5 rounded-full bg-secondary text-secondary-foreground text-xs hover:bg-accent cursor-pointer",
                                onclick: move |_| {
                                    let mut gs = app.genre;
                                    gs.set(g2.clone());
                                    app.close_modal();
                                    app.set_filter(Filter::All, "");
                                },
                                "{g}"
                            }
                        }
                    })}
                }
            }
            if !req_min.is_empty() || !req_rec.is_empty() {
                SpecsBox { req_min, req_rec }
            }
        }
    }
}

/// Trailer-Vorschau (Steam). MP4/WebM spielt überall, HLS nur unter macOS –
/// sonst Vorschaubild mit Link zur Store-Seite.
#[component]
fn TrailerBox(url: String, kind: String, thumb: String, page: String) -> Element {
    let playable = matches!(kind.as_str(), "mp4" | "webm") || (kind == "hls" && cfg!(target_os = "macos"));
    if !crate::metadata::steam_media_url(&url) {
        return rsx! {};
    }
    rsx! {
        div { class: "mb-4 rounded-xl overflow-hidden border border-border bg-black aspect-video relative",
            if playable {
                video {
                    class: "size-full object-contain bg-black",
                    src: "{url}",
                    poster: "{thumb}",
                    controls: true,
                    preload: "none",
                    "playsinline": "true",
                    "aria-label": t("trailer.label"),
                }
            } else if !thumb.is_empty() {
                button {
                    class: "group absolute inset-0 cursor-pointer",
                    "aria-label": t("trailer.open"),
                    onclick: move |_| { if !page.is_empty() { let _ = sys::open_url(&page); } },
                    img { src: "{thumb}", alt: "", class: "absolute inset-0 size-full object-cover opacity-80 group-hover:opacity-100 transition-opacity" }
                    span { class: "absolute inset-0 grid place-items-center",
                        span { class: "flex items-center gap-2 rounded-full bg-black/75 text-white px-4 py-2 text-sm font-semibold",
                            Icon { name: "play", class: "size-4", filled: true }
                            {t("trailer.open")}
                        }
                    }
                }
            }
        }
    }
}

/// Steam-Deck-Bewertung
#[component]
fn DeckBadge(deck: u8) -> Element {
    let (cls, key) = match deck {
        3 => ("bg-success/15 text-success", "deck.verified"),
        2 => ("bg-warning/15 text-warning", "deck.playable"),
        _ => ("bg-destructive/15 text-destructive", "deck.unsupported"),
    };
    rsx! {
        div { class: "mt-3 inline-flex items-center gap-2 h-7 px-3 rounded-full text-xs font-semibold {cls}", title: t("deck.title"),
            Icon { name: "gamepad", class: "size-3.5" }
            {format!("Steam Deck: {}", t(key))}
        }
    }
}

/// Mindest- und empfohlene Anforderungen
#[component]
fn SpecsBox(req_min: Vec<(String, String)>, req_rec: Vec<(String, String)>) -> Element {
    let os = if cfg!(target_os = "macos") { "macOS" } else if cfg!(target_os = "linux") { "Linux" } else { "Windows" };
    let col = |title: String, rows: Vec<(String, String)>| {
        let empty = rows.is_empty();
        rsx! {
            div { class: "min-w-0",
                div { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-2", "{title}" }
                if empty {
                    div { class: "text-sm text-muted-foreground", "–" }
                }
                {rows.into_iter().enumerate().map(|(i, (label, value))| rsx! {
                    div { key: "{i}", class: "text-[13px] py-1 border-b border-border/60 last:border-0",
                        if !label.is_empty() {
                            span { class: "text-muted-foreground", "{label}: " }
                        }
                        span { class: "select-text", "{value}" }
                    }
                })}
            }
        }
    };
    rsx! {
        div { class: "mt-5 rounded-xl border border-border p-4",
            div { class: "flex items-center gap-2 text-sm font-medium mb-3",
                Icon { name: "monitor", class: "size-4 text-primary" }
                {tf("specs.title", &[("os", os)])}
            }
            div { class: "grid grid-cols-2 gap-5",
                {col(t("specs.min"), req_min)}
                {col(t("specs.rec"), req_rec)}
            }
        }
    }
}

/// Tags und Sammlungen eines Spiels
#[component]
pub fn OrganizeBox(id: String) -> Element {
    let app = use_app();
    let tags = app.settings.read().tags_of(&id);
    let cols = app.settings.read().collections.clone();
    let mut new_tag = use_signal(String::new);
    let id_add = id.clone();
    let mut add = move || {
        let v = new_tag.peek().clone();
        add_tag(app, &id_add, &v);
        new_tag.set(String::new());
    };
    let cur_status = app.settings.read().status_of(&id).to_string();
    rsx! {
        div { class: "mt-6 grid gap-4",
            div {
                div { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-2", {t("nav.status")} }
                div { class: "flex flex-wrap gap-1.5", role: "radiogroup", "aria-label": t("nav.status"),
                    {crate::model::status::ALL.iter().map(|st| {
                        let st = *st;
                        let on = cur_status == st;
                        let cls = if on {
                            "h-7 px-3 rounded-full text-xs font-semibold bg-primary text-primary-foreground cursor-pointer flex items-center gap-1.5"
                        } else {
                            "h-7 px-3 rounded-full text-xs font-medium border border-border text-foreground/80 hover:text-foreground hover:border-primary cursor-pointer flex items-center gap-1.5"
                        };
                        let gid = id.clone();
                        rsx! {
                            button { key: "{st}", class: "{cls}", role: "radio", "aria-checked": "{on}",
                                onclick: move |_| {
                                    let gid = gid.clone();
                                    // Nochmal klicken = Status entfernen
                                    app.update(move |s| {
                                        if s.status_of(&gid) == st {
                                            s.status.remove(&gid);
                                        } else {
                                            s.status.insert(gid.clone(), st.to_string());
                                        }
                                    });
                                },
                                Icon { name: crate::model::status::icon(st), class: "size-3.5" }
                                {t(&format!("status.{st}"))}
                            }
                        }
                    })}
                }
            }
            div {
                div { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-2", {t("nav.tags")} }
                div { class: "flex flex-wrap items-center gap-1.5",
                    {tags.into_iter().map(|tag| {
                        let (gid, t2) = (id.clone(), tag.clone());
                        rsx! {
                            span { key: "{tag}", class: "inline-flex items-center gap-1 h-7 pl-2.5 pr-1 rounded-full bg-primary/15 text-primary text-xs font-medium",
                                "# {tag}"
                                button { class: "p-0.5 rounded-full hover:bg-primary/20 cursor-pointer", title: t("common.remove"),
                                    onclick: move |_| remove_tag(app, &gid, &t2),
                                    Icon { name: "x", class: "size-3" }
                                }
                            }
                        }
                    })}
                    input {
                        class: "h-7 w-36 rounded-full border border-input bg-background/60 px-3 text-xs outline-none focus-visible:ring-2 focus-visible:ring-ring/40",
                        placeholder: t("tag.add_ph"),
                        value: "{new_tag}",
                        list: "maz-tags",
                        oninput: move |e| new_tag.set(e.value()),
                        onkeydown: move |e| {
                            if e.key() == Key::Enter {
                                add();
                            }
                        },
                    }
                    datalist { id: "maz-tags",
                        {app.settings.read().all_tags().into_iter().map(|t| rsx! { option { key: "{t}", value: "{t}" } })}
                    }
                }
            }
            if !cols.is_empty() {
                div {
                    div { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-2", {t("nav.collections")} }
                    div { class: "flex flex-wrap gap-1.5",
                        {cols.into_iter().map(|c| {
                            let inside = c.games.iter().any(|g| *g == id);
                            let cls = if inside {
                                "h-7 px-3 rounded-full text-xs font-medium bg-primary text-primary-foreground cursor-pointer flex items-center gap-1"
                            } else {
                                "h-7 px-3 rounded-full text-xs font-medium border border-border text-muted-foreground hover:text-foreground hover:border-primary cursor-pointer flex items-center gap-1"
                            };
                            let (cid, gid) = (c.id.clone(), id.clone());
                            let name = c.name.clone();
                            rsx! {
                                button { key: "{cid}", class: "{cls}", onclick: move |_| toggle_in_collection(app, &cid, &gid),
                                    if inside {
                                        Icon { name: "check", class: "size-3" }
                                    }
                                    "{name}"
                                }
                            }
                        })}
                    }
                }
            }
        }
    }
}

/// Dasselbe Spiel gibt es noch in anderen Stores
#[component]
fn OtherStores(id: String) -> Element {
    let app = use_app();
    let others = app.dupes.read().get(&id).cloned().unwrap_or_default();
    if others.is_empty() {
        return rsx! {};
    }
    rsx! {
        div { class: "mt-6 rounded-xl border border-border p-4",
            div { class: "text-sm font-medium mb-1", {t("dupes.title")} }
            div { class: "text-xs text-muted-foreground mb-3", {t("dupes.text")} }
            div { class: "flex flex-col gap-2",
                {others.into_iter().map(|g| {
                    let gid = g.id.clone();
                    let key = g.id.clone();
                    let label = t(if g.installed { "game.play" } else { "game.install" });
                    let icon = if g.installed { "play" } else { "download" };
                    rsx! {
                        div { key: "{key}", class: "flex items-center gap-3",
                            SourceBadge { source: g.source }
                            span { class: "flex-1 text-sm text-muted-foreground",
                                {t(if g.installed { "filter.installed" } else { "game.not_installed" })}
                            }
                            Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm,
                                onclick: move |_| { app.close_modal(); play(app, gid.clone()); },
                                Icon { name: icon, class: "size-3.5" }
                                "{label}"
                            }
                        }
                    }
                })}
            }
        }
    }
}

/// Details als Fenster (MAZ- und Epic-Ansicht)
#[component]
pub fn DetailModal(id: String) -> Element {
    let app = use_app();
    let id_load = id.clone();
    use_hook(move || load_metadata(app, id_load));
    let Some(game) = app.games.read().iter().find(|g| g.id == id).cloned() else {
        return rsx! {};
    };
    let s = app.settings.read();
    let covers = effective_cover(&s, &game);
    drop(s);
    let mut hero = game.art.hero.clone();
    hero.extend(game.art.header.iter().cloned());
    let logos = if app.settings.read().show_logos { game.art.logo.clone() } else { Vec::new() };
    let rows = info_rows(app, &game);
    let name = game.name.clone();
    let grad = gradient_for(&game.name);
    let installed = game.installed;
    let play_label = t(if installed { "game.play" } else { "game.install" });
    let play_icon = if installed { "play" } else { "download" };
    let id_play = game.id.clone();
    let g2 = game.clone();
    let g3 = game.clone();
    let source = game.source;
    let note = match (&game.launch, game.source) {
        (LaunchTarget::SteamInstall(_) | LaunchTarget::EpicInstall(_), _) => Some(t("detail.note_install")),
        _ if !game.installed && game.source != Source::Roblox => Some(tf("detail.note_install_store", &[("store", &game.source.label())])),
        (_, Source::Custom) => Some(t("detail.note_custom")),
        _ => None,
    };
    rsx! {
        Dialog { on_close: move |_| app.close_modal(), class: "max-w-4xl",
            div { class: "relative h-72 overflow-hidden rounded-t-2xl",
                ArtImage {
                    srcs: hero,
                    class: "absolute inset-0 size-full object-cover",
                    fallback: rsx! { div { class: "absolute inset-0 {grad}" } },
                }
                div { class: "absolute inset-0 hero-fade" }
                div { class: "absolute inset-x-0 bottom-0 p-8 flex items-end gap-6",
                    div { class: "relative w-32 aspect-[2/3] rounded-lg overflow-hidden shadow-2xl ring-1 ring-white/10 shrink-0",
                        ArtImage {
                            srcs: covers,
                            class: "absolute inset-0 size-full object-cover",
                            fallback: rsx! { CoverFallback { game: g2.clone(), show_name: false } },
                        }
                    }
                    div { class: "min-w-0 pb-1",
                        div { class: "flex items-center gap-2 mb-3",
                            SourceBadge { source }
                            if !installed {
                                Badge { variant: BadgeVariant::Warning, {t("badge.not_installed")} }
                            }
                        }
                        div { class: "h-20 flex items-end",
                            ArtImage {
                                srcs: logos,
                                class: "max-h-20 max-w-[380px] object-contain object-left drop-shadow-2xl",
                                fallback: rsx! { h2 { class: "text-3xl font-black leading-tight font-display drop-shadow-lg line-clamp-2", "{name}" } },
                            }
                        }
                    }
                }
            }
            div { class: "p-8 pt-6 grid grid-cols-[1fr_260px] gap-8",
                div { class: "min-w-0",
                    div { class: "flex items-center gap-3",
                        Button { variant: ButtonVariant::Glow, size: ButtonSize::Xl, class: "min-w-52", onclick: move |_| { app.close_modal(); play(app, id_play.clone()); },
                            Icon { name: play_icon, class: "size-5", filled: true }
                            "{play_label}"
                        }
                        GameIcon { game: g3.clone(), class: "size-12 rounded-xl ring-1 ring-border" }
                    }
                    {note.map(|n| rsx! {
                        p { class: "mt-4 text-sm text-muted-foreground flex items-start gap-2",
                            Icon { name: "info", class: "size-4 mt-0.5 shrink-0" }
                            "{n}"
                        }
                    })}
                    AboutGame { id: id.clone() }
                    super::media::Achievements { id: id.clone() }
                    super::gallery::Gallery { id: id.clone() }
                    super::media::NewsFor { id: id.clone() }
                    OtherStores { id: id.clone() }
                    OrganizeBox { id: id.clone() }
                    div { class: "mt-6 rounded-xl border border-border divide-y divide-border overflow-hidden",
                        {rows.into_iter().map(|r| {
                            let label = r.label;
                            let value = r.value;
                            rsx! {
                                div { key: "{label}", class: "flex items-center gap-4 px-4 py-2.5 text-sm",
                                    span { class: "w-40 shrink-0 text-muted-foreground", "{label}" }
                                    span { class: "min-w-0 truncate select-text", title: "{value}", "{value}" }
                                }
                            }
                        })}
                    }
                }
                ActionList { game }
            }
        }
    }
}

/// Details direkt in der Seite – im Look der Steam-Bibliothek
#[component]
pub fn SteamDetail(game: Game) -> Element {
    let app = use_app();
    let id_load = game.id.clone();
    use_hook(move || load_metadata(app, id_load));
    let gid = game.id.clone();
    let s = app.settings.read();
    let mut hero = game.art.hero.clone();
    hero.extend(game.art.header.iter().cloned());
    hero.extend(effective_cover(&s, &game));
    let last = sys::format_ago(s.last_played(&game.id));
    drop(s);
    let logos = if app.settings.read().show_logos { game.art.logo.clone() } else { Vec::new() };
    let name = game.name.clone();
    let grad = gradient_for(&game.name);
    let installed = game.installed;
    let id = game.id.clone();
    let play_cls = if installed {
        "steam-play-btn is-green h-12 px-8 rounded-sm text-white font-semibold tracking-wide text-[15px] flex items-center gap-3 shadow-lg hover:brightness-110 cursor-pointer"
    } else {
        "steam-play-btn h-12 px-8 rounded-sm text-white font-semibold tracking-wide text-[15px] flex items-center gap-3 shadow-lg hover:brightness-110 cursor-pointer"
    };
    let label = t(if installed { "game.play" } else { "game.install" }).to_uppercase();
    let icon = if installed { "play" } else { "download" };
    let playtime = Some(app.settings.read().playtime(&game)).filter(|m| *m > 0).map(sys::format_playtime);
    let rows = info_rows(app, &game);
    let source = game.source;
    rsx! {
        div { class: "animate-fade-in",
            div { class: "relative h-[380px] overflow-hidden",
                ArtImage {
                    srcs: hero,
                    class: "absolute inset-0 size-full object-cover",
                    fallback: rsx! { div { class: "absolute inset-0 {grad}" } },
                }
                div { class: "absolute inset-0 steam-hero-fade" }
                div { class: "absolute left-10 bottom-10 max-w-[480px] h-36 flex items-end",
                    ArtImage {
                        srcs: logos,
                        class: "max-h-36 max-w-full object-contain object-left drop-shadow-[0_4px_12px_rgba(0,0,0,0.8)]",
                        fallback: rsx! { h1 { class: "text-5xl font-black text-white drop-shadow-lg font-display", "{name}" } },
                    }
                }
            }
            div { class: "flex items-center gap-8 px-10 py-4 bg-black/25 border-y border-black/30",
                button { class: "{play_cls}", onclick: move |_| play(app, id.clone()),
                    Icon { name: icon, class: "size-5", filled: true }
                    "{label}"
                }
                div { class: "flex flex-col",
                    span { class: "text-[11px] uppercase tracking-wider text-muted-foreground", {t("filter.recent")} }
                    span { class: "text-sm text-foreground", "{last}" }
                }
                {playtime.map(|pt| rsx! {
                    div { class: "flex flex-col",
                        span { class: "text-[11px] uppercase tracking-wider text-muted-foreground", {t("info.playtime")} }
                        span { class: "text-sm text-foreground", "{pt}" }
                    }
                })}
                div { class: "flex-1" }
                SourceBadge { source }
            }
            div { class: "px-10 py-6 grid grid-cols-[1fr_260px] gap-8",
                div { class: "rounded-sm bg-black/20 p-5",
                    AboutGame { id: gid.clone() }
                    super::media::Achievements { id: gid.clone() }
                    super::gallery::Gallery { id: gid.clone() }
                    super::media::NewsFor { id: gid.clone() }
                    OtherStores { id: gid.clone() }
                    OrganizeBox { id: gid.clone() }
                    div { class: "text-[13px] uppercase tracking-wider text-muted-foreground mb-3 mt-6", {t("detail.info")} }
                    {rows.into_iter().map(|r| {
                        let label = r.label;
                        let value = r.value;
                        rsx! {
                            div { key: "{label}", class: "flex gap-4 py-1.5 text-[13px] border-b border-white/5 last:border-0",
                                span { class: "w-40 shrink-0 text-muted-foreground", "{label}" }
                                span { class: "min-w-0 truncate select-text", title: "{value}", "{value}" }
                            }
                        }
                    })}
                }
                ActionList { game, steam_style: true }
            }
        }
    }
}
