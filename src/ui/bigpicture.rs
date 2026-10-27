//! Big-Picture-Modus: Vollbild mit großen Kacheln für TV und Sofa. Mit Controller,
//! Tastatur (Pfeiltasten, Enter, Esc) oder Maus bedienbar.

use super::art::{ArtImage, CoverFallback};
use super::icons::Icon;
use super::state::*;
use crate::i18n::t;
use crate::model::Game;
use dioxus::desktop::window;
use dioxus::prelude::*;

/// Big Picture verlassen: Vollbild aus, zurück zur Bibliothek
pub fn exit(app: App) {
    window().set_fullscreen(false);
    let on = app.settings.peek().controller_mode;
    super::apply_controller(on);
    app.go(Page::Library);
}

#[component]
pub fn BigPicture() -> Element {
    let app = use_app();
    let mut focused = use_signal(|| None::<String>);
    use_hook(move || {
        window().set_fullscreen(true);
        // Steuerung mit Controller/Pfeiltasten immer an
        super::apply_controller(true);
    });

    let rows: Vec<(String, Vec<Game>)> = {
        let s = app.settings.read();
        let games: Vec<Game> = app.games.read().iter().filter(|g| !s.is_hidden(&g.id)).cloned().collect();
        let mut recent: Vec<Game> = games.iter().filter(|g| s.last_played(&g.id) > 0).cloned().collect();
        recent.sort_by_key(|g| std::cmp::Reverse(s.last_played(&g.id)));
        recent.truncate(12);
        let favs: Vec<Game> = games.iter().filter(|g| s.is_favorite(&g.id)).cloned().collect();
        let mut installed: Vec<Game> = games.iter().filter(|g| g.installed).cloned().collect();
        installed.sort_by_key(|g| g.sort_key());
        let mut all = games.clone();
        all.sort_by_key(|g| g.sort_key());
        vec![
            (t("bp.recent"), recent),
            (t("filter.favorites"), favs),
            (t("filter.installed"), installed),
            (t("filter.all"), all),
        ]
        .into_iter()
        .filter(|(_, v)| !v.is_empty())
        .collect()
    };
    let current = focused()
        .and_then(|id| app.game(&id))
        .or_else(|| rows.first().and_then(|(_, v)| v.first().cloned()));
    let hero: Vec<String> = current.as_ref().map(|g| g.art.hero.iter().chain(g.art.header.iter()).cloned().collect()).unwrap_or_default();
    let title = current.as_ref().map(|g| g.name.clone()).unwrap_or_default();
    let logos = if app.settings.read().show_logos { current.as_ref().map(|g| g.art.logo.clone()).unwrap_or_default() } else { Vec::new() };
    let cur_id = current.as_ref().map(|g| g.id.clone()).unwrap_or_default();
    let (cur_id2, installed) = (cur_id.clone(), current.as_ref().map(|g| g.installed).unwrap_or(false));
    let cur_play = cur_id.clone();
    let has_current = !cur_id.is_empty();
    let play_icon: &'static str = if installed { "play" } else { "download" };
    let play_label = t(if installed { "game.play" } else { "game.install" });
    let playtime = current.as_ref().map(|g| crate::stats::fmt_minutes(app.settings.read().playtime(g))).unwrap_or_default();

    rsx! {
        div {
            class: "fixed inset-0 z-30 bg-black text-white flex flex-col overflow-hidden",
            onkeydown: move |e| {
                if e.key() == Key::Escape && app.modal.peek().is_none() {
                    exit(app);
                }
            },
            // Hintergrund: Bild des ausgewählten Spiels
            div { class: "absolute inset-0 pointer-events-none",
                ArtImage { srcs: hero, class: "absolute inset-0 size-full object-cover opacity-50 animate-fade-in", fallback: rsx! {} }
                div { class: "absolute inset-0 bg-linear-to-t from-black via-black/70 to-black/20" }
            }
            header { class: "relative flex items-center gap-4 px-12 pt-8",
                super::icons::MazLogo { class: "size-10 text-sm" }
                span { class: "text-xl font-bold font-display", "Big Picture" }
                div { class: "flex-1" }
                button {
                    class: "h-12 px-6 rounded-full bg-white/10 hover:bg-white/20 focus-visible:ring-4 focus-visible:ring-white text-base font-semibold flex items-center gap-2 cursor-pointer",
                    "data-dialog-close": "1",
                    onclick: move |_| exit(app),
                    Icon { name: "x", class: "size-5" }
                    {t("bp.exit")}
                }
            }
            section { class: "relative px-12 pt-10 pb-6 min-h-[34vh] flex flex-col justify-end", "aria-live": "polite",
                div { class: "h-28 flex items-end",
                    ArtImage { srcs: logos, class: "max-h-28 max-w-[520px] object-contain object-left drop-shadow-2xl", alt: title.clone(),
                        fallback: rsx! { h1 { class: "text-6xl font-black font-display drop-shadow-lg line-clamp-2", "{title}" } } }
                }
                div { class: "mt-4 flex items-center gap-4 text-lg text-white/85",
                    if !playtime.is_empty() {
                        span { class: "flex items-center gap-2", Icon { name: "clock", class: "size-5" } "{playtime}" }
                    }
                }
                div { class: "mt-6 flex gap-4",
                    if has_current {
                        button {
                            class: "h-16 px-10 rounded-full bg-white text-black text-xl font-bold flex items-center gap-3 hover:scale-105 focus-visible:ring-4 focus-visible:ring-primary transition-transform cursor-pointer",
                            onclick: move |_| play(app, cur_play.clone()),
                            Icon { name: play_icon, class: "size-6", filled: true }
                            {play_label}
                        }
                        button {
                            class: "h-16 px-8 rounded-full bg-white/15 text-xl font-semibold hover:bg-white/25 focus-visible:ring-4 focus-visible:ring-white cursor-pointer",
                            onclick: move |_| app.open(Modal::Detail(cur_id2.clone())),
                            {t("random.details")}
                        }
                    }
                }
            }
            main { class: "relative flex-1 overflow-y-auto pb-16",
                {rows.into_iter().map(|(label, games)| rsx! {
                    section { key: "{label}", class: "mt-6",
                        h2 { class: "px-12 mb-3 text-2xl font-bold font-display", "{label}" }
                        div { class: "flex gap-5 overflow-x-auto px-12 py-4 scroll-px-12", role: "list",
                            {games.into_iter().map(|g| {
                                let (id, id2, key, id_play) = (g.id.clone(), g.id.clone(), g.id.clone(), g.id.clone());
                                let covers = effective_cover(&app.settings.read(), &g);
                                let name = g.name.clone();
                                let g2 = g.clone();
                                rsx! {
                                    button {
                                        key: "{key}",
                                        role: "listitem",
                                        class: "relative shrink-0 w-48 xl:w-56 aspect-[2/3] rounded-2xl overflow-hidden bg-white/10 ring-0 outline-none transition-transform duration-200 hover:scale-105 focus:scale-110 focus:ring-4 focus:ring-white focus:z-10 shadow-2xl cursor-pointer",
                                        "aria-label": "{name}",
                                        onfocus: move |_| focused.set(Some(id.clone())),
                                        onmouseenter: move |_| focused.set(Some(id2.clone())),
                                        onclick: move |_| play(app, id_play.clone()),
                                        ArtImage { srcs: covers, class: "absolute inset-0 size-full object-cover", alt: name.clone(),
                                            fallback: rsx! { CoverFallback { game: g2.clone(), show_name: true } } }
                                    }
                                }
                            })}
                        }
                    }
                })}
            }
            footer { class: "relative px-12 py-4 text-sm text-white/80 flex gap-8 border-t border-white/10",
                span { {t("bp.hint_play")} }
                span { {t("bp.hint_move")} }
                span { {t("bp.hint_exit")} }
            }
        }
    }
}
