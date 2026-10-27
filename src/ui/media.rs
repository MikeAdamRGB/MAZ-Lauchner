//! Bilder & Medien: SteamGridDB (Einstellungen + Auswahl-Dialog) und Erfolge.

use super::components::*;
use super::icons::Icon;
use super::state::*;
use crate::i18n::{t, tf};
use crate::sgdb::{ApiError, Choice, Kind};
use crate::sys;
use dioxus::prelude::*;

/// Einstellungen → Verbindungen: eigener SteamGridDB-Schlüssel
#[component]
pub fn SgdbSection() -> Element {
    let app = use_app();
    let has_key = !app.settings.read().sgdb_key.trim().is_empty();
    let mut input = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let steps = crate::i18n::tl("sgdb.steps");
    let mut save = move || {
        let k: String = input.peek().chars().filter(|c| !c.is_whitespace()).collect();
        if k.len() < 16 || k.len() > 128 || !k.chars().all(|c| c.is_ascii_alphanumeric()) {
            error.set(Some(t("sgdb.err_format")));
            return;
        }
        busy.set(true);
        error.set(None);
        spawn(async move {
            let k2 = k.clone();
            let res = tokio::task::spawn_blocking(move || crate::sgdb::check_key(&k2)).await.unwrap_or(Err(ApiError::Network(String::new())));
            busy.set(false);
            match res {
                Ok(()) => {
                    app.update(|s| s.sgdb_key = k);
                    input.set(String::new());
                    app.toast(&t("sgdb.saved"), "check-circle");
                    rescan(app);
                }
                Err(ApiError::Key) => error.set(Some(t("sgdb.err_key"))),
                Err(ApiError::Network(e)) => error.set(Some(format!("{}: {e}", t("common.err_network")))),
            }
        });
    };
    rsx! {
        super::settings::Section { title: "SteamGridDB", description: t("sgdb.text"),
            if has_key {
                div { class: "flex flex-wrap items-center gap-3",
                    span { class: "inline-flex items-center gap-1.5 text-sm text-success font-medium",
                        Icon { name: "check-circle", class: "size-4" } {t("sgdb.active")}
                    }
                    Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, class: "text-destructive",
                        onclick: move |_| app.update(|s| s.sgdb_key.clear()),
                        Icon { name: "trash", class: "size-3.5" } {t("sgdb.remove")}
                    }
                }
            } else {
                ol { class: "text-sm space-y-1.5 mb-4 list-decimal pl-5 leading-relaxed",
                    {steps.into_iter().enumerate().map(|(i, st)| rsx! { li { key: "{i}", "{st}" } })}
                }
                div { class: "flex flex-wrap gap-2 mb-3",
                    Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm,
                        onclick: move |_| { let _ = sys::open_url("https://www.steamgriddb.com/profile/preferences/api"); },
                        Icon { name: "external", class: "size-3.5" } {t("sgdb.open")}
                    }
                }
                div { class: "flex gap-2",
                    Input { kind: "password", class: "font-mono text-xs", placeholder: t("sgdb.ph"), value: input(),
                        oninput: move |v: String| input.set(v), onenter: move |_| save() }
                    Button { variant: ButtonVariant::Glow, class: "h-10", disabled: busy(), onclick: move |_| save(),
                        if busy() { Spinner {} } else { Icon { name: "check", class: "size-4" } }
                        {t("sgdb.save")}
                    }
                }
                if let Some(e) = error() {
                    div { class: "mt-3 flex items-start gap-2 rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm", role: "alert",
                        Icon { name: "alert", class: "size-4 mt-0.5 shrink-0 text-destructive" }
                        span { "{e}" }
                    }
                }
            }
            p { class: "mt-3 text-xs text-muted-foreground leading-relaxed", {t("sgdb.privacy")} }
        }
    }
}

/// Bilder für ein Spiel auswählen (Cover, Hintergrund, Logo)
#[component]
pub fn SgdbPickDialog(id: String) -> Element {
    let app = use_app();
    let mut kind = use_signal(|| Kind::Cover);
    let mut list = use_signal(|| None::<Result<Vec<Choice>, String>>);
    let gid = id.clone();
    use_effect(move || {
        let k = kind();
        let Some(game) = app.game(&gid) else { return };
        let key = app.settings.peek().sgdb_key.clone();
        list.set(None);
        spawn(async move {
            let res = tokio::task::spawn_blocking(move || crate::sgdb::choices(&game, k, &key)).await;
            list.set(Some(match res {
                Ok(Ok(v)) => Ok(v),
                Ok(Err(ApiError::Key)) => Err(t("sgdb.err_key")),
                Ok(Err(ApiError::Network(e))) => Err(format!("{}: {e}", t("common.err_network"))),
                Err(_) => Err(t("common.err_unknown")),
            }));
        });
    });
    let tabs = vec![("cover".to_string(), t("sgdb.kind.cover")), ("hero".to_string(), t("sgdb.kind.hero")), ("logo".to_string(), t("sgdb.kind.logo"))];
    let cur = match kind() {
        Kind::Cover => "cover",
        Kind::Hero => "hero",
        _ => "logo",
    }
    .to_string();
    let aspect = match kind() {
        Kind::Cover => "aspect-[2/3]",
        Kind::Hero => "aspect-[96/31]",
        _ => "aspect-[16/9]",
    };
    let cols = if kind() == Kind::Hero { "grid-cols-2" } else { "grid-cols-3 sm:grid-cols-4" };
    let name = app.game(&id).map(|g| g.name).unwrap_or_default();
    rsx! {
        Dialog { on_close: move |_| app.close_modal(), class: "max-w-3xl",
            div { class: "p-8",
                h2 { class: "text-xl font-bold font-display pr-8", {tf("sgdb.pick_title", &[("game", &name)])} }
                p { class: "text-sm text-muted-foreground mt-1 mb-4", {t("sgdb.pick_text")} }
                Segmented { options: tabs, value: cur, onchange: move |v: String| kind.set(match v.as_str() { "hero" => Kind::Hero, "logo" => Kind::Logo, _ => Kind::Cover }) }
                div { class: "mt-5 min-h-48",
                    {match list() {
                        None => rsx! { div { class: "py-16 grid place-items-center", Spinner { class: "size-6" } } },
                        Some(Err(e)) => rsx! { div { class: "rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm", role: "alert", "{e}" } },
                        Some(Ok(v)) if v.is_empty() => rsx! { div { class: "py-16 text-center text-sm", {t("sgdb.none")} } },
                        Some(Ok(v)) => rsx! {
                            div { class: "grid gap-3 {cols}",
                                {v.into_iter().enumerate().map(|(i, c)| {
                                    let (gid, url) = (id.clone(), c.url.clone());
                                    let k = kind();
                                    let label = tf("sgdb.choose_n", &[("n", &(i + 1).to_string())]);
                                    rsx! {
                                        button { key: "{c.url}",
                                            class: "group relative {aspect} rounded-lg overflow-hidden border border-border bg-muted hover:ring-2 hover:ring-primary focus-visible:ring-2 focus-visible:ring-primary cursor-pointer",
                                            "aria-label": "{label}",
                                            onclick: move |_| {
                                                let (gid, url) = (gid.clone(), url.clone());
                                                app.update(move |s| match k {
                                                    Kind::Cover => { s.cover_overrides.insert(gid, url); }
                                                    Kind::Hero => { s.meta.entry(gid).or_default().hero = url; }
                                                    _ => { s.logo_overrides.insert(gid, url); }
                                                });
                                                app.toast(&t("sgdb.applied"), "check-circle");
                                                app.close_modal();
                                                rescan(app);
                                            },
                                            img { src: "{c.thumb}", alt: "", loading: "lazy", class: "absolute inset-0 size-full object-contain" }
                                        }
                                    }
                                })}
                            }
                        },
                    }}
                }
                p { class: "mt-4 text-xs text-muted-foreground", {t("sgdb.credit")} }
            }
        }
    }
}

// ----------------------------------------------------------------------
// Erfolge
// ----------------------------------------------------------------------

#[derive(Clone, PartialEq)]
enum AchState {
    Loading,
    Steam(crate::achievements::Summary),
    Count(usize, usize),
    Message(String),
    Hidden,
}

/// Erfolge in den Spiel-Details (Steam vollständig, GOG als Zahl, Epic als Link)
#[component]
pub fn Achievements(id: String) -> Element {
    let app = use_app();
    let mut state = use_signal(|| AchState::Loading);
    let mut show_all = use_signal(|| false);
    let game = app.game(&id);
    let gid = id.clone();
    use_effect(move || {
        let Some(g) = app.game(&gid) else {
            state.set(AchState::Hidden);
            return;
        };
        let s = app.settings.peek();
        let (key, sid) = (s.steam_api_key.clone(), s.steam_id.clone());
        drop(s);
        let lang = crate::i18n::current_code();
        match (g.source, g.steam_appid) {
            (crate::model::Source::Steam, Some(appid)) => {
                spawn(async move {
                    let sid = if sid.trim().is_empty() {
                        tokio::task::spawn_blocking(|| crate::steam::find_steam_root().and_then(|r| crate::steam::logged_in_user(&r)).map(|u| u.0))
                            .await
                            .ok()
                            .flatten()
                            .unwrap_or_default()
                    } else {
                        sid
                    };
                    let res = tokio::task::spawn_blocking(move || crate::achievements::steam(appid, &key, &sid, &lang)).await;
                    state.set(match res {
                        Ok(Ok(sum)) => AchState::Steam(sum),
                        Ok(Err(crate::achievements::Error::NoKey)) => AchState::Message(t("ach.need_key")),
                        Ok(Err(crate::achievements::Error::Private)) => AchState::Message(t("ach.private")),
                        Ok(Err(crate::achievements::Error::None)) => AchState::Hidden,
                        Ok(Err(crate::achievements::Error::Network(_))) | Err(_) => AchState::Hidden,
                    });
                });
            }
            (crate::model::Source::Gog, _) => {
                let gog_id = g.id.trim_start_matches("gog:").to_string();
                spawn(async move {
                    let res = tokio::task::spawn_blocking(move || crate::achievements::gog(&gog_id)).await.ok().flatten();
                    state.set(match res {
                        Some((u, total)) => AchState::Count(u, total),
                        None => AchState::Hidden,
                    });
                });
            }
            _ => state.set(AchState::Hidden),
        }
    });
    let is_epic = game.as_ref().map(|g| g.source == crate::model::Source::Epic).unwrap_or(false);
    let st = state();
    if is_epic {
        return rsx! {
            div { class: "mt-5 rounded-xl border border-border p-4 flex flex-wrap items-center gap-3",
                Icon { name: "star", class: "size-4 text-primary" }
                span { class: "text-sm flex-1", {t("ach.epic")} }
                Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm,
                    onclick: move |_| { let _ = sys::open_url("com.epicgames.launcher://store/library"); },
                    Icon { name: "external", class: "size-3.5" } {t("ach.open_epic")}
                }
            }
        };
    }
    match st {
        AchState::Hidden => rsx! {},
        AchState::Loading => rsx! {
            div { class: "mt-5 h-16 rounded-xl skeleton", "aria-busy": "true" }
        },
        AchState::Message(m) => rsx! {
            div { class: "mt-5 rounded-xl border border-border p-4 text-sm flex items-start gap-2",
                Icon { name: "info", class: "size-4 mt-0.5 text-primary shrink-0" }
                span { "{m}" }
            }
        },
        AchState::Count(u, total) => {
            let pct = u as f64 / total.max(1) as f64 * 100.0;
            rsx! {
                div { class: "mt-5 rounded-xl border border-border p-4",
                    div { class: "flex justify-between text-sm mb-2",
                        span { class: "font-semibold", {t("ach.title")} }
                        span { class: "tabular-nums", "{u} / {total}" }
                    }
                    div { class: "h-2 rounded-full bg-muted overflow-hidden", role: "progressbar", "aria-valuenow": "{u}", "aria-valuemax": "{total}",
                        div { class: "h-full maz-gradient-bg", style: "width: {pct}%;" }
                    }
                }
            }
        }
        AchState::Steam(sum) => {
            let pct = sum.unlocked as f64 / sum.total.max(1) as f64 * 100.0;
            let all = show_all();
            let shown: Vec<crate::achievements::Achievement> = if all { sum.list.clone() } else { sum.list.iter().take(12).cloned().collect() };
            let more = sum.total > 12;
            let (u, total) = (sum.unlocked, sum.total);
            rsx! {
                div { class: "mt-5 rounded-xl border border-border p-4",
                    div { class: "flex justify-between text-sm mb-2",
                        span { class: "font-semibold", {t("ach.title")} }
                        span { class: "tabular-nums", "{u} / {total}" }
                    }
                    div { class: "h-2 rounded-full bg-muted overflow-hidden mb-4", role: "progressbar", "aria-valuenow": "{u}", "aria-valuemax": "{total}",
                        div { class: "h-full maz-gradient-bg", style: "width: {pct}%;" }
                    }
                    ul { class: "grid gap-2 sm:grid-cols-2",
                        {shown.into_iter().enumerate().map(|(i, a)| {
                            let secret = a.hidden && !a.achieved;
                            let name = if secret { t("ach.hidden") } else { a.name.clone() };
                            let desc = if secret { String::new() } else { a.description.clone() };
                            let icon = if a.achieved || a.icon_gray.is_empty() { a.icon.clone() } else { a.icon_gray.clone() };
                            let cls = if a.achieved { "flex gap-3 items-center rounded-lg p-2 bg-success/10" } else { "flex gap-3 items-center rounded-lg p-2 opacity-80" };
                            let status = t(if a.achieved { "ach.unlocked" } else { "ach.locked" });
                            rsx! {
                                li { key: "{i}", class: "{cls}",
                                    if !icon.is_empty() {
                                        img { src: "{icon}", alt: "", class: "size-10 rounded-md shrink-0", loading: "lazy" }
                                    } else {
                                        div { class: "size-10 rounded-md bg-muted grid place-items-center shrink-0", Icon { name: "star", class: "size-4" } }
                                    }
                                    div { class: "min-w-0",
                                        div { class: "text-sm font-medium truncate", "{name}" span { class: "sr-only", " – {status}" } }
                                        if !desc.is_empty() {
                                            div { class: "text-xs text-muted-foreground line-clamp-2", "{desc}" }
                                        }
                                    }
                                }
                            }
                        })}
                    }
                    if more {
                        button { class: "mt-3 text-sm font-medium text-primary hover:underline cursor-pointer", onclick: move |_| show_all.set(!all),
                            {t(if all { "ach.less" } else { "ach.all" })}
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
enum NewsState {
    Closed,
    Loading,
    Ready(Vec<crate::news::NewsItem>),
    Failed,
}

/// Spiel-Details: News und Patch-Notes von Steam (lädt erst auf Klick)
#[component]
pub fn GameNews(appid: u32) -> Element {
    let mut state = use_signal(|| NewsState::Closed);
    let mut only_patch = use_signal(|| false);
    let mut open_item = use_signal(|| None::<usize>);
    let mut load = move || {
        state.set(NewsState::Loading);
        spawn(async move {
            let res = tokio::task::spawn_blocking(move || crate::news::steam(appid)).await;
            state.set(match res {
                Ok(Ok(v)) => NewsState::Ready(v),
                _ => NewsState::Failed,
            });
        });
    };
    let st = state();
    let patch_on = only_patch();
    let all_on = !patch_on;
    rsx! {
        div { class: "mt-5 rounded-xl border border-border p-4",
            div { class: "flex flex-wrap items-center gap-3",
                Icon { name: "news", class: "size-4 text-primary" }
                h3 { class: "text-sm font-semibold flex-1", {t("news.title")} }
                if matches!(st, NewsState::Ready(_)) {
                    div { class: "flex rounded-lg bg-muted p-0.5 text-xs", role: "group",
                        button { class: if !only_patch() { "px-2.5 py-1 rounded-md bg-background font-medium cursor-pointer" } else { "px-2.5 py-1 rounded-md text-muted-foreground cursor-pointer" },
                            "aria-pressed": "{all_on}", onclick: move |_| only_patch.set(false), {t("news.all")} }
                        button { class: if only_patch() { "px-2.5 py-1 rounded-md bg-background font-medium cursor-pointer" } else { "px-2.5 py-1 rounded-md text-muted-foreground cursor-pointer" },
                            "aria-pressed": "{patch_on}", onclick: move |_| only_patch.set(true), {t("news.patches")} }
                    }
                }
                if matches!(st, NewsState::Closed | NewsState::Failed) {
                    Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm, onclick: move |_| load(),
                        Icon { name: "download", class: "size-3.5" } {t("news.load")} }
                }
            }
            match st {
                NewsState::Closed => rsx! {},
                NewsState::Loading => rsx! { p { class: "mt-3 text-sm text-muted-foreground", role: "status", {t("news.loading")} } },
                NewsState::Failed => rsx! { p { class: "mt-3 text-sm text-muted-foreground", role: "alert", {t("news.failed")} } },
                NewsState::Ready(items) => {
                    let list: Vec<(usize, crate::news::NewsItem)> = items.into_iter().enumerate().filter(|(_, n)| !only_patch() || n.patch).collect();
                    rsx! {
                        if list.is_empty() {
                            p { class: "mt-3 text-sm text-muted-foreground", {t("news.none")} }
                        }
                        ul { class: "mt-3 divide-y divide-border",
                            {list.into_iter().map(|(i, n)| {
                                let (y, m, d, _, _) = crate::util::civil(n.date as i64);
                                let date = format!("{d:02}.{m:02}.{y}");
                                let expanded = open_item() == Some(i);
                                let url = n.url.clone();
                                let chevron: &'static str = if expanded { "chevron-up" } else { "chevron-down" };
                                let (title, feed, body, is_patch) = (n.title.clone(), n.feed.clone(), n.text.clone(), n.patch);
                                rsx! {
                                    li { key: "{i}", class: "py-3",
                                        button { class: "w-full text-left flex items-start gap-2 cursor-pointer rounded-md",
                                            "aria-expanded": "{expanded}",
                                            onclick: move |_| open_item.set(if expanded { None } else { Some(i) }),
                                            div { class: "flex-1 min-w-0",
                                                div { class: "text-sm font-medium leading-snug", "{title}" }
                                                div { class: "text-xs text-muted-foreground mt-0.5 flex gap-2",
                                                    span { "{date}" }
                                                    if !feed.is_empty() { span { "· {feed}" } }
                                                    if is_patch { span { class: "text-primary font-medium", "· " {t("news.patch")} } }
                                                }
                                            }
                                            Icon { name: chevron, class: "size-4 text-muted-foreground mt-0.5" }
                                        }
                                        if expanded {
                                            p { class: "mt-2 text-sm leading-relaxed whitespace-pre-line select-text", "{body}" }
                                            if let Some(u) = url {
                                                Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, class: "mt-2",
                                                    onclick: move |_| { let _ = sys::open_url(&u); },
                                                    Icon { name: "external", class: "size-3.5" } {t("news.read_more")} }
                                            }
                                        }
                                    }
                                }
                            })}
                        }
                    }
                }
            }
        }
    }
}

/// News nur für Spiele mit Steam-Nummer
#[component]
pub fn NewsFor(id: String) -> Element {
    let app = use_app();
    match app.game(&id).and_then(|g| g.steam_appid) {
        Some(appid) => rsx! { GameNews { key: "{appid}", appid } },
        None => rsx! {},
    }
}
