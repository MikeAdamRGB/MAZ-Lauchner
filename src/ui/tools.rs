//! Werkzeuge: Zufallsspiel, Sicherung/Wiederherstellung, Import aus Playnite.

use super::art::ArtImage;
use super::components::*;
use super::icons::Icon;
use super::state::*;
use crate::i18n::{t, tf};
use crate::model::Settings;
use crate::stats::{pick, random_candidates, RandomFilter};
use crate::{backup, sys};
use dioxus::prelude::*;

// ----------------------------------------------------------------------
// „Was soll ich spielen?“
// ----------------------------------------------------------------------

#[component]
pub fn RandomDialog() -> Element {
    let app = use_app();
    let mut installed_only = use_signal(|| true);
    let mut skip_done = use_signal(|| true);
    let mut unplayed_only = use_signal(|| false);
    let mut store = use_signal(String::new);
    let mut seed = use_signal(|| sys::now_secs() ^ 0x5eed);
    let mut last = use_signal(String::new);

    let filter = RandomFilter { installed_only: installed_only(), store: store(), skip_done: skip_done(), unplayed_only: unplayed_only() };
    let (count, chosen) = {
        let s = app.settings.read();
        let games = app.games.read();
        let list = random_candidates(&games, &s, &filter);
        (list.len(), pick(&list, seed(), &last.peek()).cloned())
    };
    let stores: Vec<(String, String)> = {
        let games = app.games.read();
        let mut v: Vec<(String, String)> = Vec::new();
        for g in games.iter() {
            let k = g.source.key().to_string();
            if !v.iter().any(|(x, _)| *x == k) {
                v.push((k, g.source.label()));
            }
        }
        v.sort_by(|a, b| a.1.cmp(&b.1));
        v
    };
    let chosen_id = chosen.as_ref().map(|c| c.id.clone());
    let mut reroll = move || {
        if let Some(c) = &chosen_id {
            last.set(c.clone());
        }
        seed.set(seed().wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407) ^ sys::now_secs());
    };
    rsx! {
        Dialog { on_close: move |_| app.close_modal(), class: "max-w-2xl",
            div { class: "p-8",
                div { class: "flex items-center gap-3 mb-5",
                    div { class: "size-11 rounded-xl maz-gradient-bg text-glow-fg grid place-items-center", Icon { name: "dice", class: "size-5" } }
                    div {
                        h2 { class: "text-xl font-bold font-display", {t("random.title")} }
                        p { class: "text-sm text-muted-foreground", {tf("random.count", &[("n", &count.to_string())])} }
                    }
                }
                div { class: "grid gap-x-6 sm:grid-cols-2 rounded-xl border border-border px-4",
                    SwitchRow { title: t("random.installed"), checked: installed_only(), onchange: move |v: bool| installed_only.set(v) }
                    SwitchRow { title: t("random.skip_done"), checked: skip_done(), onchange: move |v: bool| skip_done.set(v) }
                    SwitchRow { title: t("random.unplayed"), checked: unplayed_only(), onchange: move |v: bool| unplayed_only.set(v) }
                    div { class: "flex items-center justify-between gap-4 py-3",
                        label { class: "text-sm font-medium", r#for: "rnd-store", {t("random.store")} }
                        select { id: "rnd-store",
                            class: "h-9 rounded-md border border-input bg-background px-3 text-sm outline-none cursor-pointer",
                            onchange: move |e| store.set(e.value()),
                            option { value: "", selected: store().is_empty(), {t("filter.all")} }
                            {stores.into_iter().map(|(k, l)| {
                                let sel = store() == k;
                                rsx! { option { key: "{k}", value: "{k}", selected: sel, "{l}" } }
                            })}
                        }
                    }
                }
                {match chosen.clone() {
                    None => rsx! {
                        div { class: "mt-6 rounded-xl bg-muted p-8 text-center text-sm", {t("random.none")} }
                    },
                    Some(g) => {
                        let (id1, id2) = (g.id.clone(), g.id.clone());
                        let covers = g.art.cover.clone();
                        let name = g.name.clone();
                        let src = g.source.label();
                        rsx! {
                            div { class: "mt-6 flex gap-5 items-center rounded-xl border border-border bg-card p-4 animate-pop-in", key: "{g.id}",
                                div { class: "relative w-28 aspect-[2/3] shrink-0 rounded-lg overflow-hidden bg-muted",
                                    ArtImage { srcs: covers, class: "absolute inset-0 size-full object-cover",
                                        fallback: rsx! { div { class: "absolute inset-0 grid place-items-center", Icon { name: "gamepad", class: "size-8" } } } }
                                }
                                div { class: "min-w-0 flex-1",
                                    div { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground", "{src}" }
                                    div { class: "text-2xl font-bold font-display mt-1 leading-tight", "{name}" }
                                    div { class: "flex flex-wrap gap-2 mt-4",
                                        Button { variant: ButtonVariant::Glow, onclick: move |_| { app.close_modal(); play(app, id1.clone()); },
                                            Icon { name: "play", class: "size-4", filled: true }
                                            {t(if g.installed { "game.play" } else { "game.install" })}
                                        }
                                        Button { variant: ButtonVariant::Outline, onclick: move |_| app.open(Modal::Detail(id2.clone())),
                                            {t("random.details")}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }}
                div { class: "flex justify-end mt-6",
                    Button { variant: ButtonVariant::Secondary, disabled: count == 0, onclick: move |_| reroll(),
                        Icon { name: "dice", class: "size-4" }
                        {t("random.again")}
                    }
                }
            }
        }
    }
}

// ----------------------------------------------------------------------
// Sicherung
// ----------------------------------------------------------------------

/// Abschnitt in Einstellungen → Allgemein
#[component]
pub fn BackupSection() -> Element {
    let app = use_app();
    rsx! {
        super::settings::Section { title: t("backup.title"), description: t("backup.text"),
            div { class: "flex flex-wrap gap-2",
                Button { variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        let now = sys::now_secs();
                        let Some(path) = rfd::FileDialog::new()
                            .set_title(t("backup.save"))
                            .set_file_name(backup::file_name(now))
                            .add_filter("Open Game Launcher", &[backup::EXTENSION])
                            .save_file() else { return };
                        let text = backup::create(&app.settings.peek(), now);
                        match text.and_then(|t| std::fs::write(&path, t).map_err(|e| e.to_string())) {
                            Ok(()) => app.toast(&t("backup.saved"), "check-circle"),
                            Err(e) => app.toast(&tf("backup.err_save", &[("error", &e)]), "alert"),
                        }
                    },
                    Icon { name: "archive", class: "size-4" }
                    {t("backup.save")}
                }
                Button { variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        let Some(path) = rfd::FileDialog::new()
                            .set_title(t("backup.load"))
                            .add_filter("Open Game Launcher", &[backup::EXTENSION, "json"])
                            .pick_file() else { return };
                        let text = match std::fs::metadata(&path) {
                            Ok(m) if m.len() < 50 * 1024 * 1024 => std::fs::read_to_string(&path).unwrap_or_default(),
                            _ => String::new(),
                        };
                        match backup::read(&text) {
                            Ok((s, _)) => app.open(Modal::Restore(Box::new(s))),
                            Err(e) => app.toast(&e, "alert"),
                        }
                    },
                    Icon { name: "upload", class: "size-4" }
                    {t("backup.load")}
                }
                Button { variant: ButtonVariant::Outline, onclick: move |_| app.open(Modal::PlayniteImport),
                    Icon { name: "download", class: "size-4" }
                    {t("playnite.button")}
                }
            }
            p { class: "mt-3 text-xs text-muted-foreground leading-relaxed", {t("backup.secrets")} }
        }
    }
}

#[component]
pub fn RestoreDialog(settings: Box<Settings>) -> Element {
    let app = use_app();
    let rows = backup::summary(&settings);
    let incoming = settings.clone();
    rsx! {
        Dialog { on_close: move |_| app.close_modal(), class: "max-w-md",
            div { class: "p-8",
                div { class: "flex items-center gap-3 mb-4",
                    div { class: "size-10 rounded-xl bg-primary/15 text-primary grid place-items-center", Icon { name: "upload", class: "size-5" } }
                    h2 { class: "text-lg font-bold", {t("backup.restore_title")} }
                }
                p { class: "text-sm leading-relaxed", {t("backup.restore_text")} }
                ul { class: "mt-4 rounded-xl bg-muted p-4 space-y-1.5 text-sm",
                    {rows.into_iter().map(|(label, n)| rsx! {
                        li { key: "{label}", class: "flex justify-between", span { "{label}" } span { class: "font-semibold tabular-nums", "{n}" } }
                    })}
                }
                div { class: "flex justify-end gap-2 mt-6",
                    Button { variant: ButtonVariant::Ghost, onclick: move |_| app.close_modal(), {t("common.cancel")} }
                    Button { variant: ButtonVariant::Glow,
                        onclick: move |_| {
                            // Vorher automatisch den aktuellen Stand sichern
                            let now = sys::now_secs();
                            let dir = sys::roaming_dir().join("backups");
                            if let Ok(text) = backup::create(&app.settings.peek(), now) {
                                let _ = std::fs::create_dir_all(&dir);
                                let _ = std::fs::write(dir.join(format!("vor-wiederherstellung-{now}.{}", backup::EXTENSION)), text);
                            }
                            let restored = backup::restore(&app.settings.peek(), &incoming);
                            app.update(move |s| *s = restored);
                            app.close_modal();
                            app.toast(&t("backup.restored"), "check-circle");
                            rescan(app);
                        },
                        Icon { name: "check", class: "size-4" }
                        {t("backup.restore_ok")}
                    }
                }
            }
        }
    }
}

// ----------------------------------------------------------------------
// Playnite
// ----------------------------------------------------------------------

#[component]
pub fn PlayniteDialog() -> Element {
    let app = use_app();
    let mut dir = use_signal(|| crate::playnite::default_dir().map(|d| d.to_string_lossy().to_string()).unwrap_or_default());
    let mut loaded = use_signal(|| None::<Result<crate::playnite::Library, String>>);
    let mut report = use_signal(|| None::<crate::playnite::Report>);
    let mut busy = use_signal(|| false);

    let mut load = move || {
        let d = dir.peek().clone();
        if d.is_empty() {
            return;
        }
        busy.set(true);
        spawn(async move {
            let res = tokio::task::spawn_blocking(move || crate::playnite::load(std::path::Path::new(&d))).await.unwrap_or_else(|_| Err(t("common.err_unknown")));
            busy.set(false);
            loaded.set(Some(res));
        });
    };
    use_hook(move || load());

    let info = match &*loaded.read() {
        Some(Ok(lib)) => Some(Ok((lib.games.len(), lib.tags.len(), lib.categories.len()))),
        Some(Err(e)) => Some(Err(e.clone())),
        None => None,
    };
    let can_import = matches!(info, Some(Ok(_))) && report.read().is_none();
    let rep = report.read().clone();
    let d_show = dir();
    rsx! {
        Dialog { on_close: move |_| app.close_modal(), class: "max-w-lg",
            div { class: "p-8",
                div { class: "flex items-center gap-3 mb-4",
                    div { class: "size-10 rounded-xl bg-primary/15 text-primary grid place-items-center", Icon { name: "download", class: "size-5" } }
                    h2 { class: "text-lg font-bold", {t("playnite.title")} }
                }
                p { class: "text-sm leading-relaxed", {t("playnite.text")} }
                div { class: "mt-4 flex gap-2 items-center",
                    div { class: "flex-1 min-w-0 rounded-lg bg-muted px-3 py-2 text-xs font-mono truncate", title: "{d_show}",
                        if d_show.is_empty() { {t("playnite.no_dir")} } else { "{d_show}" }
                    }
                    Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm,
                        onclick: move |_| {
                            if let Some(p) = rfd::FileDialog::new().set_title(t("playnite.pick")).pick_folder() {
                                dir.set(p.to_string_lossy().to_string());
                                report.set(None);
                                load();
                            }
                        },
                        Icon { name: "folder", class: "size-3.5" }
                        {t("playnite.pick")}
                    }
                }
                if busy() {
                    div { class: "mt-4 flex items-center gap-2 text-sm", Spinner {} {t("playnite.loading")} }
                }
                {match info {
                    Some(Ok((g, tg, c))) => rsx! {
                        div { class: "mt-4 rounded-xl bg-muted p-4 text-sm",
                            {tf("playnite.found", &[("games", &g.to_string()), ("tags", &tg.to_string()), ("cols", &c.to_string())])}
                        }
                    },
                    Some(Err(e)) => rsx! {
                        div { class: "mt-4 flex items-start gap-2 rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm",
                            Icon { name: "alert", class: "size-4 mt-0.5 shrink-0 text-destructive" }
                            span { "{e}" }
                        }
                    },
                    None => rsx! {},
                }}
                {rep.map(|r| rsx! {
                    div { class: "mt-4 rounded-xl border border-success/40 bg-success/10 p-4 text-sm space-y-1",
                        div { class: "font-semibold flex items-center gap-2", Icon { name: "check-circle", class: "size-4 text-success" } {t("playnite.done")} }
                        div { {tf("playnite.report", &[
                            ("matched", &r.matched.to_string()), ("tags", &r.tags.to_string()), ("cols", &r.collections.to_string()),
                            ("time", &r.playtime.to_string()), ("status", &r.status.to_string()), ("custom", &r.new_custom.to_string()),
                            ("missing", &r.not_found.to_string()),
                        ])} }
                    }
                })}
                p { class: "mt-4 text-xs text-muted-foreground leading-relaxed", {t("playnite.note")} }
                div { class: "flex justify-end gap-2 mt-6",
                    Button { variant: ButtonVariant::Ghost, onclick: move |_| app.close_modal(), {t("common.close")} }
                    Button { variant: ButtonVariant::Glow, disabled: !can_import,
                        onclick: move |_| {
                            let r = {
                                let lib = loaded.read();
                                let Some(Ok(lib)) = &*lib else { return };
                                let games = app.games.peek().clone();
                                let mut out = None;
                                app.update(|s| out = Some(crate::playnite::apply(lib, &games, s, sys::now_secs())));
                                out
                            };
                            report.set(r);
                            rescan(app);
                        },
                        Icon { name: "download", class: "size-4" }
                        {t("playnite.import")}
                    }
                }
            }
        }
    }
}
