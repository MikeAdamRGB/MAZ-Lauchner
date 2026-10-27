//! Screenshot-Galerie in den Spiel-Details (Steam-Screenshots + eigener Ordner) und Großansicht.

use super::components::*;
use super::icons::Icon;
use super::state::*;
use crate::i18n::{t, tf};
use crate::screenshots::Shot;
use dioxus::prelude::*;

fn shots_for(app: App, id: &str) -> (Option<u32>, Option<String>) {
    let appid = app.game(id).and_then(|g| g.steam_appid);
    let dir = app.settings.peek().shot_dirs.get(id).cloned();
    (appid, dir)
}

#[component]
pub fn Gallery(id: String) -> Element {
    let app = use_app();
    let mut shots = use_signal(Vec::<Shot>::new);
    let mut show_all = use_signal(|| false);
    let own_dir = app.settings.read().shot_dirs.get(&id).cloned();
    let (gid, gid2, gid3) = (id.clone(), id.clone(), id.clone());
    // Neu laden, wenn sich der eigene Ordner ändert
    use_effect(use_reactive((&own_dir,), move |(dir,)| {
        let appid = app.game(&gid).and_then(|g| g.steam_appid);
        spawn(async move {
            let list = tokio::task::spawn_blocking(move || crate::screenshots::collect(appid, dir.as_deref())).await.unwrap_or_default();
            shots.set(list);
        });
    }));
    let list = shots();
    let total = list.len();
    let visible: Vec<(usize, Shot)> = list.into_iter().enumerate().take(if show_all() { usize::MAX } else { 8 }).collect();
    rsx! {
        div { class: "mt-5 rounded-xl border border-border p-4",
            div { class: "flex flex-wrap items-center gap-2",
                Icon { name: "image", class: "size-4 text-primary" }
                h3 { class: "text-sm font-semibold flex-1",
                    {t("shots.title")}
                    if total > 0 { span { class: "ml-2 text-xs text-muted-foreground font-normal", "{total}" } }
                }
                Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm,
                    onclick: move |_| {
                        if let Some(p) = pick_folder(&t("shots.pick")) {
                            let gid = gid2.clone();
                            app.update(move |s| { s.shot_dirs.insert(gid, p); });
                        }
                    },
                    Icon { name: "folder", class: "size-3.5" }
                    {t(if own_dir.is_some() { "shots.change_dir" } else { "shots.add_dir" })}
                }
                if own_dir.is_some() {
                    Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, title: t("shots.remove_dir"),
                        onclick: move |_| { let gid = gid3.clone(); app.update(move |s| { s.shot_dirs.remove(&gid); }); },
                        Icon { name: "x", class: "size-3.5" }
                    }
                }
            }
            if let Some(d) = own_dir.clone() {
                p { class: "mt-1 text-xs text-muted-foreground font-mono truncate", title: "{d}", "{d}" }
            }
            if total == 0 {
                p { class: "mt-3 text-sm text-muted-foreground", {t("shots.none")} }
            } else {
                div { class: "mt-3 grid grid-cols-2 sm:grid-cols-4 gap-2", role: "list",
                    {visible.into_iter().map(|(i, s)| {
                        let src = crate::sys::file_url(&s.path);
                        let label = tf("shots.open", &[("n", &(i + 1).to_string())]);
                        let gid = id.clone();
                        rsx! {
                            button { key: "{src}", role: "listitem",
                                class: "relative aspect-video rounded-lg overflow-hidden bg-muted cursor-pointer hover:ring-2 hover:ring-primary focus-visible:ring-2 focus-visible:ring-primary",
                                "aria-label": "{label}",
                                onclick: move |_| app.open(Modal::Screenshot(gid.clone(), i)),
                                img { src: "{src}", class: "size-full object-cover", loading: "lazy", alt: "" }
                            }
                        }
                    })}
                }
                if total > 8 && !show_all() {
                    Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, class: "mt-2", onclick: move |_| show_all.set(true),
                        {tf("shots.show_all", &[("n", &total.to_string())])}
                    }
                }
            }
        }
    }
}

/// Großansicht mit Weiter/Zurück (Pfeiltasten) und „Im Ordner zeigen“
#[component]
pub fn ScreenshotViewer(id: String, index: usize) -> Element {
    let app = use_app();
    let mut shots = use_signal(Vec::<Shot>::new);
    let mut cur = use_signal(|| index);
    let gid = id.clone();
    use_hook(move || {
        let (appid, dir) = shots_for(app, &gid);
        spawn(async move {
            let list = tokio::task::spawn_blocking(move || crate::screenshots::collect(appid, dir.as_deref())).await.unwrap_or_default();
            shots.set(list);
        });
    });
    let list = shots();
    let n = list.len();
    let i = if n == 0 { 0 } else { cur().min(n - 1) };
    let shot = list.get(i).cloned();
    let pos = format!("{} / {}", i + 1, n);
    let mut step = move |d: isize| {
        if n > 0 {
            let next = (i as isize + d).rem_euclid(n as isize) as usize;
            cur.set(next);
        }
    };
    rsx! {
        Dialog { on_close: move |_| app.close_modal(), class: "max-w-6xl w-[94vw]",
            div { class: "p-4 outline-none", tabindex: "0", autofocus: true,
                onkeydown: move |e| match e.key() {
                    Key::ArrowLeft => step(-1),
                    Key::ArrowRight => step(1),
                    _ => {}
                },
                match shot {
                    None => rsx! { p { class: "p-8 text-center text-muted-foreground", {t("shots.none")} } },
                    Some(s) => {
                        let src = crate::sys::file_url(&s.path);
                        let path = s.path.to_string_lossy().to_string();
                        let path_title = path.clone();
                        let name = s.path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
                        rsx! {
                            div { class: "relative bg-black rounded-lg overflow-hidden grid place-items-center min-h-[50vh]",
                                img { src: "{src}", class: "max-h-[75vh] max-w-full object-contain", alt: "{name}" }
                                if n > 1 {
                                    button { class: "absolute left-3 top-1/2 -translate-y-1/2 size-11 rounded-full bg-black/60 text-white grid place-items-center hover:bg-black/80 cursor-pointer",
                                        "aria-label": t("shots.prev"), onclick: move |_| step(-1), Icon { name: "chevron-left", class: "size-6" } }
                                    button { class: "absolute right-3 top-1/2 -translate-y-1/2 size-11 rounded-full bg-black/60 text-white grid place-items-center hover:bg-black/80 cursor-pointer",
                                        "aria-label": t("shots.next"), onclick: move |_| step(1), Icon { name: "chevron-right", class: "size-6" } }
                                }
                            }
                            div { class: "mt-3 flex flex-wrap items-center gap-3 text-sm",
                                span { class: "flex-1 min-w-0 truncate font-mono text-xs text-muted-foreground", title: "{path_title}", "{name}" }
                                span { class: "text-xs text-muted-foreground", "{pos}" }
                                Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm,
                                    onclick: move |_| crate::sys::reveal_file(&path),
                                    Icon { name: "folder", class: "size-3.5" } {t("shots.reveal")} }
                                Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, onclick: move |_| app.close_modal(), {t("common.close")} }
                            }
                        }
                    }
                }
            }
        }
    }
}
