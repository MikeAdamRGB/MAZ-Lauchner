//! Bibliothek verwalten: Spiel-Infos bearbeiten, Ordner nach Spielen durchsuchen,
//! Release-Notizen schön anzeigen.

use super::components::*;
use super::dialogs::Field;
use super::icons::Icon;
use super::state::*;
use crate::i18n::{t, tf};
use crate::model::{CustomGame, GameMeta};
use crate::notes::{Block, Span};
use crate::{config, sys};
use dioxus::prelude::*;

// ----------------------------------------------------------------------
// Release-Notizen
// ----------------------------------------------------------------------

fn spans(list: Vec<Span>) -> Element {
    rsx! {
        {list.into_iter().map(|s| match s {
            Span::Text(t) => rsx! { "{t}" },
            Span::Bold(t) => rsx! { strong { class: "font-semibold text-foreground", "{t}" } },
            Span::Code(t) => rsx! { code { class: "rounded bg-background/70 px-1 py-0.5 text-[12px] font-mono", "{t}" } },
        })}
    }
}

/// Changelog als übersichtliche Abschnitte (Neu / Verbessert / Behoben)
#[component]
pub fn MarkdownNotes(text: String) -> Element {
    // Download-Tabelle usw. nur auf GitHub zeigen, Versions-Überschrift steht schon im Fenster
    let body = text.split("<!-- footer -->").next().unwrap_or("");
    let blocks: Vec<Block> = crate::notes::parse(body)
        .into_iter()
        .filter(|b| !matches!(b, Block::Heading(l, t) if *l <= 2 && t.starts_with("MAZ Launcher")))
        .collect();
    rsx! {
        div { class: "space-y-1.5 text-sm leading-relaxed text-foreground/85",
            {blocks.into_iter().enumerate().map(|(i, b)| match b {
                Block::Heading(level, title) => {
                    let kind = crate::notes::section_kind(&title);
                    let (icon, color) = match kind {
                        "new" => ("sparkles", "text-success"),
                        "improved" => ("wrench", "text-primary"),
                        "fixed" => ("check-circle", "text-warning"),
                        "security" => ("shield", "text-destructive"),
                        _ => ("", "text-foreground"),
                    };
                    let size = if level <= 2 { "text-base font-bold" } else { "text-[13px] font-semibold uppercase tracking-wider" };
                    rsx! {
                        div { key: "{i}", class: "flex items-center gap-2 pt-3 first:pt-0 {size} {color}",
                            if !icon.is_empty() {
                                Icon { name: icon, class: "size-4" }
                            }
                            "{title}"
                        }
                    }
                }
                Block::Bullet(s) => rsx! {
                    div { key: "{i}", class: "flex gap-2 pl-1",
                        span { class: "mt-2 size-1.5 shrink-0 rounded-full bg-muted-foreground/60" }
                        div { class: "min-w-0", {spans(s)} }
                    }
                },
                Block::Para(s) => rsx! { p { key: "{i}", class: "text-muted-foreground", {spans(s)} } },
            })}
        }
    }
}

// ----------------------------------------------------------------------
// Spiel-Infos bearbeiten
// ----------------------------------------------------------------------

#[component]
pub fn EditMetaDialog(id: String) -> Element {
    let app = use_app();
    let Some(game) = app.game(&id) else {
        return rsx! {};
    };
    let own = app.settings.peek().meta.get(&id).cloned().unwrap_or_default();
    let auto = app.meta_cache.peek().get(&id).cloned().unwrap_or_default();
    let mut name = use_signal(|| if own.name.is_empty() { game.name.clone() } else { own.name.clone() });
    let mut hero = use_signal(|| own.hero.clone());
    let mut cover = use_signal(|| app.settings.peek().cover_overrides.get(&id).cloned().unwrap_or_default());
    let mut desc = use_signal(|| if own.description.is_empty() { auto.description.clone() } else { own.description.clone() });
    let mut genres = use_signal(|| {
        let g = if own.genres.is_empty() { auto.genres.clone() } else { own.genres.clone() };
        g.join(", ")
    });
    // Für "Zurücksetzen": Originalname ohne eigene Änderung
    let original_name = game.name.clone();
    let (id_save, id_reset) = (id.clone(), id.clone());
    let auto2 = auto.clone();

    let save = move |_: MouseEvent| {
        let n = name.peek().trim().to_string();
        let d = desc.peek().trim().to_string();
        let gs: Vec<String> = genres
            .peek()
            .split([',', ';'])
            .map(|g| g.trim().chars().take(30).collect::<String>())
            .filter(|g| !g.is_empty())
            .take(10)
            .collect();
        let meta = GameMeta {
            // Nur speichern, was wirklich anders ist als automatisch
            name: if n == original_name || n.is_empty() { String::new() } else { n.chars().take(120).collect() },
            hero: hero.peek().trim().trim_matches('"').to_string(),
            description: if d == auto2.description { String::new() } else { d.chars().take(4000).collect() },
            genres: if gs == auto2.genres { Vec::new() } else { gs },
        };
        let c = cover.peek().trim().trim_matches('"').to_string();
        let gid = id_save.clone();
        app.update(|s| {
            if meta == GameMeta::default() {
                s.meta.remove(&gid);
            } else {
                s.meta.insert(gid.clone(), meta);
            }
            if c.is_empty() {
                s.cover_overrides.remove(&gid);
            } else {
                s.cover_overrides.insert(gid.clone(), c);
            }
        });
        app.close_modal();
        app.toast(&t("meta.saved"), "check-circle");
        rescan(app);
    };

    rsx! {
        Dialog { on_close: move |_| app.close_modal(), close_on_backdrop: false, class: "max-w-2xl",
            div { class: "p-8",
                div { class: "flex items-center gap-3 mb-1",
                    div { class: "size-10 rounded-xl maz-gradient-bg grid place-items-center text-white",
                        Icon { name: "pencil", class: "size-5" }
                    }
                    h2 { class: "text-2xl font-bold font-display", {t("meta.title")} }
                }
                p { class: "text-sm text-muted-foreground mb-6", {t("meta.intro")} }
                div { class: "grid gap-5",
                    Field { label: t("edit.name"),
                        Input { value: name(), oninput: move |v: String| name.set(v) }
                    }
                    div { class: "grid grid-cols-2 gap-4",
                        Field { label: t("edit.cover"), hint: t("meta.cover_hint"),
                            div { class: "flex gap-2",
                                Input { class: "font-mono text-xs", placeholder: t("edit.auto_ph"), value: cover(), oninput: move |v: String| cover.set(v) }
                                Button { variant: ButtonVariant::Secondary, size: ButtonSize::Icon, class: "size-10", title: t("edit.pick_image"),
                                    onclick: move |_| { if let Some(p) = pick_image_file() { cover.set(p); } },
                                    Icon { name: "image", class: "size-4" }
                                }
                            }
                        }
                        Field { label: t("meta.hero"), hint: t("meta.hero_hint"),
                            div { class: "flex gap-2",
                                Input { class: "font-mono text-xs", placeholder: t("edit.auto_ph"), value: hero(), oninput: move |v: String| hero.set(v) }
                                Button { variant: ButtonVariant::Secondary, size: ButtonSize::Icon, class: "size-10", title: t("edit.pick_image"),
                                    onclick: move |_| { if let Some(p) = pick_image_file() { hero.set(p); } },
                                    Icon { name: "image", class: "size-4" }
                                }
                            }
                        }
                    }
                    Field { label: t("meta.description"),
                        textarea {
                            class: "min-h-28 w-full rounded-md border border-input bg-background/60 px-3 py-2 text-sm outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40",
                            value: "{desc}",
                            oninput: move |e| desc.set(e.value()),
                        }
                    }
                    Field { label: t("meta.genres"), hint: t("meta.genres_hint"),
                        Input { placeholder: "Action, RPG", value: genres(), oninput: move |v: String| genres.set(v) }
                    }
                }
                div { class: "flex justify-between gap-2 mt-8",
                    Button { variant: ButtonVariant::Ghost,
                        onclick: move |_| {
                            let gid = id_reset.clone();
                            app.update(|s| {
                                s.meta.remove(&gid);
                                s.cover_overrides.remove(&gid);
                            });
                            app.close_modal();
                            rescan(app);
                        },
                        Icon { name: "refresh", class: "size-4" }
                        {t("meta.reset")}
                    }
                    div { class: "flex gap-2",
                        Button { variant: ButtonVariant::Outline, onclick: move |_| app.close_modal(), {t("common.cancel")} }
                        Button { variant: ButtonVariant::Glow, onclick: save,
                            Icon { name: "check", class: "size-4" }
                            {t("common.save")}
                        }
                    }
                }
            }
        }
    }
}

// ----------------------------------------------------------------------
// Ordner nach Spielen durchsuchen
// ----------------------------------------------------------------------

#[derive(Clone, PartialEq)]
struct Found {
    name: String,
    exe: String,
    on: bool,
}

#[component]
pub fn FolderScanDialog() -> Element {
    let app = use_app();
    let mut folder = use_signal(String::new);
    let mut found = use_signal(Vec::<Found>::new);
    let mut busy = use_signal(|| false);
    let mut searched = use_signal(|| false);

    let mut start = move |dir: String| {
        folder.set(dir.clone());
        busy.set(true);
        searched.set(false);
        // Schon vorhandene Spiele nicht noch einmal anbieten
        let known: Vec<String> = app.settings.peek().custom_games.iter().map(|c| c.exe.to_lowercase()).collect();
        spawn(async move {
            let res = tokio::task::spawn_blocking(move || crate::emulators::scan_game_folder(std::path::Path::new(&dir))).await.unwrap_or_default();
            let list = res
                .into_iter()
                .filter(|(_, exe)| !known.contains(&exe.to_lowercase()))
                .map(|(name, exe)| Found { name, exe, on: true })
                .collect();
            found.set(list);
            busy.set(false);
            searched.set(true);
        });
    };

    let add = move |_: MouseEvent| {
        let picked: Vec<Found> = found.peek().iter().filter(|f| f.on).cloned().collect();
        if picked.is_empty() {
            return;
        }
        let n = picked.len();
        let now = sys::now_secs();
        app.update(|s| {
            for f in picked {
                s.custom_games.push(CustomGame { id: config::new_id(), name: f.name, exe: f.exe, added: now, ..Default::default() });
            }
        });
        app.close_modal();
        app.toast(&tf("scan.added", &[("n", &n.to_string())]), "check-circle");
        rescan(app);
    };

    let list = found();
    let count_on = list.iter().filter(|f| f.on).count();
    let dir = folder();
    rsx! {
        Dialog { on_close: move |_| app.close_modal(), class: "max-w-2xl",
            div { class: "p-8",
                div { class: "flex items-center gap-3 mb-1",
                    div { class: "size-10 rounded-xl maz-gradient-bg grid place-items-center text-white",
                        Icon { name: "folder-search", class: "size-5" }
                    }
                    h2 { class: "text-2xl font-bold font-display", {t("scan.title")} }
                }
                p { class: "text-sm text-muted-foreground mb-6", {t("scan.intro")} }
                div { class: "flex gap-2",
                    Input { class: "font-mono text-xs", placeholder: t("scan.folder_ph"), value: dir.clone(), oninput: move |v: String| folder.set(v) }
                    Button { variant: ButtonVariant::Secondary, class: "h-10",
                        onclick: move |_| { if let Some(p) = pick_folder(&t("scan.title")) { start(p); } },
                        Icon { name: "folder", class: "size-4" }
                        {t("common.browse")}
                    }
                    Button { variant: ButtonVariant::Outline, class: "h-10", disabled: dir.trim().is_empty() || busy(),
                        onclick: move |_| { let d = folder.peek().trim().trim_matches('"').to_string(); start(d); },
                        Icon { name: "search", class: "size-4" }
                        {t("scan.search")}
                    }
                }
                if busy() {
                    div { class: "flex items-center gap-3 py-10 justify-center text-sm text-muted-foreground",
                        Spinner {}
                        {t("scan.searching")}
                    }
                } else if searched() && list.is_empty() {
                    div { class: "py-10 text-center text-sm text-muted-foreground", {t("scan.none")} }
                } else if !list.is_empty() {
                    div { class: "mt-5 max-h-80 overflow-y-auto rounded-xl border border-border divide-y divide-border",
                        {list.into_iter().enumerate().map(|(i, f)| {
                            let exe = f.exe.clone();
                            let fname = f.name.clone();
                            rsx! {
                                label { key: "{exe}", class: "flex items-center gap-3 px-4 py-2.5 cursor-pointer hover:bg-accent/40",
                                    input {
                                        r#type: "checkbox",
                                        class: "size-4 accent-[var(--primary)] cursor-pointer",
                                        checked: f.on,
                                        onchange: move |e| {
                                            let v = e.checked();
                                            let mut fw = found;
                                            if let Some(x) = fw.write().get_mut(i) {
                                                x.on = v;
                                            }
                                        },
                                    }
                                    div { class: "min-w-0 flex-1",
                                        input {
                                            class: "w-full bg-transparent text-sm font-medium outline-none",
                                            value: "{fname}",
                                            onclick: move |e| e.stop_propagation(),
                                            oninput: move |e| {
                                                let mut fw = found;
                                                if let Some(x) = fw.write().get_mut(i) {
                                                    x.name = e.value();
                                                }
                                            },
                                        }
                                        div { class: "text-xs text-muted-foreground font-mono truncate", title: "{exe}", "{exe}" }
                                    }
                                }
                            }
                        })}
                    }
                    p { class: "mt-3 text-xs text-muted-foreground", {t("scan.hint")} }
                }
                div { class: "flex justify-end gap-2 mt-6",
                    Button { variant: ButtonVariant::Outline, onclick: move |_| app.close_modal(), {t("common.cancel")} }
                    Button { variant: ButtonVariant::Glow, disabled: count_on == 0, onclick: add,
                        Icon { name: "plus", class: "size-4" }
                        {tf("scan.add", &[("n", &count_on.to_string())])}
                    }
                }
            }
        }
    }
}
