//! Einstellungen → Erweiterungen, Rechte-Dialog bei der Installation, Aktions-Knöpfe.

use super::components::*;
use super::icons::Icon;
use super::settings::{PageTitle, Section};
use super::state::*;
use crate::extensions::{self as ext, Staged, Status};
use crate::i18n::{t, tf};
use crate::model::Game;
use crate::sys;
use dioxus::prelude::*;

fn status_text(s: &Status) -> (String, &'static str) {
    match s {
        Status::Active => (t("ext.st.active"), "text-success"),
        Status::Off => (t("ext.st.off"), "text-muted-foreground"),
        Status::NotAllowed => (t("ext.st.not_allowed"), "text-muted-foreground"),
        Status::NeedsApproval => (t("ext.st.needs_approval"), "text-warning"),
        Status::HashChanged => (t("ext.st.hash_changed"), "text-warning"),
        Status::Invalid(e) => (tf("ext.st.invalid", &[("error", e)]), "text-destructive"),
        Status::Failed(e) => (tf("ext.st.failed", &[("error", e)]), "text-destructive"),
    }
}

/// Neu einlesen (im Hintergrund), danach Bibliothek neu laden
fn reload(app: App) {
    rescan(app);
}

fn pick_and_stage(app: App, folder: bool) {
    let picked = if folder {
        rfd::FileDialog::new().set_title(t("ext.install")).pick_folder()
    } else {
        rfd::FileDialog::new().set_title(t("ext.install")).add_filter("MAZ Extension", &["mazext", "zip"]).pick_file()
    };
    let Some(path) = picked else { return };
    spawn(async move {
        match tokio::task::spawn_blocking(move || ext::stage(&path)).await.unwrap_or_else(|_| Err(t("common.err_unknown"))) {
            Ok(staged) => app.open(Modal::ExtInstall(staged)),
            Err(e) => app.show_problem(crate::problem::Problem::extension_failed(&e), None),
        }
    });
}

#[component]
pub fn ExtensionsTab() -> Element {
    let app = use_app();
    let allowed = app.settings.read().extensions_allowed;
    // Liste aktualisiert sich nach jedem Neu-Laden der Bibliothek
    let _ = app.games.read();
    let list = ext::summaries();
    rsx! {
        PageTitle { title: t("set.tab.extensions"), sub: t("ext.sub") }

        Section { title: t("ext.allow"), description: t("ext.allow_text"),
            SwitchRow {
                title: t("ext.allow_switch"),
                checked: allowed,
                onchange: move |v: bool| { app.update(|s| s.extensions_allowed = v); reload(app); },
            }
            div { class: "mt-3 flex items-start gap-2 rounded-xl border border-warning/30 bg-warning/10 px-4 py-3 text-xs leading-relaxed",
                Icon { name: "shield", class: "size-4 mt-0.5 shrink-0 text-warning" }
                span { {t("ext.security_note")} }
            }
        }

        if allowed {
            Section { title: t("ext.installed"),
                div { class: "flex flex-wrap gap-2 mb-4",
                    Button { variant: ButtonVariant::Glow, onclick: move |_| pick_and_stage(app, false),
                        Icon { name: "package", class: "size-4" } {t("ext.install_file")}
                    }
                    Button { variant: ButtonVariant::Outline, onclick: move |_| pick_and_stage(app, true),
                        Icon { name: "folder", class: "size-4" } {t("ext.install_folder")}
                    }
                    Button { variant: ButtonVariant::Ghost, onclick: move |_| { let _ = std::fs::create_dir_all(ext::root()); sys::open_folder(&ext::root().to_string_lossy()); },
                        Icon { name: "folder", class: "size-4" } {t("ext.open_root")}
                    }
                }
                if list.is_empty() {
                    div { class: "rounded-xl border border-dashed border-border p-6 text-center text-sm text-muted-foreground", {t("ext.none")} }
                }
                div { class: "grid gap-3",
                    {list.into_iter().map(|e| {
                        let (st, cls) = status_text(&e.status);
                        let key = e.id.clone();
                        let (id1, id2, id3) = (e.id.clone(), e.id.clone(), e.id.clone());
                        let dir = e.dir.clone();
                        let enabled = matches!(e.status, Status::Active | Status::Failed(_));
                        let can_toggle = matches!(e.status, Status::Active | Status::Off | Status::Failed(_));
                        let needs_review = matches!(e.status, Status::NeedsApproval | Status::HashChanged);
                        let name = e.name.clone();
                        let meta = [e.version.clone(), e.author.clone()].into_iter().filter(|x| !x.is_empty()).collect::<Vec<_>>().join(" · ");
                        let desc = e.description.clone();
                        rsx! {
                            div { key: "{key}", class: "rounded-xl border border-border p-4",
                                div { class: "flex items-start gap-3",
                                    div { class: "size-10 shrink-0 rounded-xl bg-primary/15 text-primary grid place-items-center", Icon { name: "plug", class: "size-5" } }
                                    div { class: "min-w-0 flex-1",
                                        div { class: "font-semibold", "{name}" }
                                        if !meta.is_empty() { div { class: "text-xs text-muted-foreground", "{meta}" } }
                                        if !desc.is_empty() { p { class: "text-sm text-muted-foreground mt-1", "{desc}" } }
                                        div { class: "text-xs mt-2 {cls}", "{st}" }
                                    }
                                    if can_toggle {
                                        Switch { checked: enabled, label: name.clone(),
                                            onchange: move |v: bool| {
                                                let id = id1.clone();
                                                app.update(|s| { if let Some(x) = s.extensions.get_mut(&id) { x.enabled = v; } });
                                                reload(app);
                                            },
                                        }
                                    }
                                }
                                div { class: "flex flex-wrap gap-2 mt-3",
                                    Button { variant: if needs_review { ButtonVariant::Glow } else { ButtonVariant::Outline }, size: ButtonSize::Sm,
                                        onclick: move |_| {
                                            let id = id2.clone();
                                            spawn(async move {
                                                match tokio::task::spawn_blocking(move || ext::review(&id)).await.unwrap_or_else(|_| Err(t("common.err_unknown"))) {
                                                    Ok(s) => app.open(Modal::ExtInstall(s)),
                                                    Err(e) => app.show_problem(crate::problem::Problem::extension_failed(&e), None),
                                                }
                                            });
                                        },
                                        Icon { name: "shield", class: "size-3.5" }
                                        {t(if needs_review { "ext.review" } else { "ext.show_rights" })}
                                    }
                                    Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, onclick: move |_| sys::open_folder(&dir),
                                        Icon { name: "folder", class: "size-3.5" } {t("ext.open_folder")}
                                    }
                                    Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, class: "text-destructive hover:text-destructive",
                                        onclick: move |_| {
                                            let id = id3.clone();
                                            if confirm_remove(&id) {
                                                match ext::remove(&id) {
                                                    Ok(()) => {
                                                        let i2 = id.clone();
                                                        app.update(|s| { s.extensions.remove(&i2); });
                                                        app.toast(&t("ext.removed"), "trash");
                                                        reload(app);
                                                    }
                                                    Err(e) => app.show_problem(crate::problem::Problem::extension_failed(&e), None),
                                                }
                                            }
                                        },
                                        Icon { name: "trash", class: "size-3.5" } {t("act.remove")}
                                    }
                                }
                            }
                        }
                    })}
                }
                p { class: "text-xs text-muted-foreground mt-4 leading-relaxed", {t("ext.dev_hint")} }
            }
        }
    }
}

fn confirm_remove(id: &str) -> bool {
    use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
    MessageDialog::new()
        .set_level(MessageLevel::Warning)
        .set_title(t("act.remove"))
        .set_description(tf("ext.remove_confirm", &[("name", id)]))
        .set_buttons(MessageButtons::YesNo)
        .show()
        == MessageDialogResult::Yes
}

/// Rechte anzeigen und Zustimmung einholen (Installation, Update oder geänderte Dateien)
#[component]
pub fn ExtInstallDialog(staged: Staged) -> Element {
    let app = use_app();
    let m = staged.manifest.clone();
    let kinds: Vec<String> = m
        .kinds
        .iter()
        .map(|k| t(match k {
            crate::extensions::manifest::ExtKind::Library => "ext.kind.library",
            crate::extensions::manifest::ExtKind::Metadata => "ext.kind.metadata",
            crate::extensions::manifest::ExtKind::Action => "ext.kind.action",
        }))
        .collect();
    let paths = m.permissions.read_paths.clone();
    let schemes = m.permissions.uri_schemes.clone();
    let network = m.permissions.network;
    let in_root = staged.dir.starts_with(ext::root()) && !staged.dir.file_name().map(|n| n.to_string_lossy().starts_with(".staging")).unwrap_or(false);
    let (s_ok, s_cancel) = (staged.clone(), staged.clone());
    let title = tf(if staged.update { "ext.dialog.update" } else { "ext.dialog.install" }, &[("name", &m.name)]);
    let hash = staged.hash.clone();
    let run_hash = staged.run_hash.clone();
    let is_update = staged.update;
    rsx! {
        Dialog { on_close: move |_| { if !in_root { ext::discard(&s_cancel) }; app.close_modal(); }, close_on_backdrop: false, class: "max-w-xl",
            div { class: "p-8",
                div { class: "flex items-center gap-3 mb-1",
                    div { class: "size-10 rounded-xl bg-warning/15 text-warning grid place-items-center", Icon { name: "shield", class: "size-5" } }
                    h2 { class: "text-xl font-bold font-display", "{title}" }
                }
                p { class: "text-sm text-muted-foreground mb-5", {t("ext.dialog.text")} }
                div { class: "rounded-xl border border-border divide-y divide-border text-sm",
                    Row { label: t("ext.f.name"), value: m.name.clone() }
                    Row { label: t("ext.f.author"), value: m.author.clone() }
                    Row { label: t("ext.f.version"), value: m.version.clone() }
                    if !m.homepage.is_empty() { Row { label: t("ext.f.homepage"), value: m.homepage.clone() } }
                    Row { label: t("ext.f.kinds"), value: kinds.join(", ") }
                    Row { label: t("ext.f.read_paths"), value: if paths.is_empty() { t("ext.f.none") } else { paths.join("\n") } }
                    Row { label: t("ext.f.network"), value: t(if network { "common.yes" } else { "common.no" }) }
                    Row { label: t("ext.f.schemes"), value: if schemes.is_empty() { t("ext.f.none") } else { schemes.iter().map(|s| format!("{s}://")).collect::<Vec<_>>().join(", ") } }
                    Row { label: t("ext.f.hash"), value: hash.clone(), mono: true }
                    Row { label: t("ext.f.run_hash"), value: run_hash.clone(), mono: true }
                }
                div { class: "mt-4 flex items-start gap-2 rounded-xl border border-warning/30 bg-warning/10 px-4 py-3 text-xs leading-relaxed",
                    Icon { name: "alert", class: "size-4 mt-0.5 shrink-0 text-warning" }
                    span { {t("ext.dialog.warning")} }
                }
                div { class: "flex justify-end gap-2 mt-6",
                    Button { variant: ButtonVariant::Outline,
                        onclick: move |_| { if !in_root { ext::discard(&staged) }; app.close_modal(); },
                        {t("common.cancel")}
                    }
                    Button { variant: ButtonVariant::Glow,
                        onclick: move |_| {
                            let s = s_ok.clone();
                            if in_root {
                                // Bereits im Ordner (manuell kopiert/geändert): nur Zustimmung speichern
                                let st = ext::ExtState {
                                    enabled: true,
                                    hash: s.hash.clone(),
                                    approved_schemes: s.manifest.permissions.uri_schemes.clone(),
                                    network: s.manifest.permissions.network,
                                    installed: sys::now_secs(),
                                };
                                let id = s.manifest.id.clone();
                                app.update(|x| { x.extensions.insert(id, st); });
                                app.close_modal();
                                app.toast(&t("ext.approved"), "check-circle");
                                rescan(app);
                                return;
                            }
                            match ext::commit(&s) {
                                Ok((id, st)) => {
                                    app.update(|x| { x.extensions.insert(id, st); });
                                    app.close_modal();
                                    app.toast(&tf("ext.installed_toast", &[("name", &s.manifest.name)]), "check-circle");
                                    rescan(app);
                                }
                                Err(e) => {
                                    ext::discard(&s);
                                    app.show_problem(crate::problem::Problem::extension_failed(&e), None);
                                }
                            }
                        },
                        Icon { name: "check", class: "size-4" }
                        {t(if is_update { "ext.dialog.agree_update" } else { "ext.dialog.agree" })}
                    }
                }
            }
        }
    }
}

#[component]
fn Row(#[props(into)] label: String, #[props(into)] value: String, #[props(default = false)] mono: bool) -> Element {
    let vcls = if mono { "font-mono text-[11px] break-all" } else { "whitespace-pre-line" };
    rsx! {
        div { class: "flex gap-4 px-4 py-2.5",
            span { class: "w-36 shrink-0 text-muted-foreground", "{label}" }
            span { class: "min-w-0 flex-1 select-text {vcls}", "{value}" }
        }
    }
}

/// Zusätzliche Knöpfe aus Erweiterungen in den Spiel-Details
#[component]
pub fn ExtActions(game: Game, #[props(default)] class: String) -> Element {
    let app = use_app();
    let mut actions = use_signal(Vec::<ext::Action>::new);
    let allowed = app.settings.read().extensions_allowed;
    let g0 = game.clone();
    use_hook(move || {
        if !allowed {
            return;
        }
        spawn(async move {
            let list = tokio::task::spawn_blocking(move || ext::actions(&g0)).await.unwrap_or_default();
            actions.set(list);
        });
    });
    rsx! {
        {actions.read().clone().into_iter().map(|a| {
            let key = format!("{}-{}", a.ext, a.id);
            let g = game.clone();
            let label = a.label.clone();
            let cls = class.clone();
            rsx! {
                Button { key: "{key}", variant: ButtonVariant::Outline, class: cls,
                    onclick: move |_| {
                        let (e, id, g) = (a.ext.clone(), a.id.clone(), g.clone());
                        spawn(async move {
                            let res = tokio::task::spawn_blocking(move || {
                                let r = ext::run_action(&e, &id, &g);
                                (e, r)
                            }).await;
                            match res {
                                Ok((_, Ok(ext::ActionResult::OpenFolder(p)))) => sys::open_folder(&p),
                                Ok((_, Ok(ext::ActionResult::OpenUrl(u)))) => { let _ = sys::open_url(&u); }
                                Ok((e, Ok(ext::ActionResult::Message(m)))) => {
                                    let title = ext::get(&e).map(|x| x.name()).unwrap_or(e);
                                    app.open(Modal::ExtMessage(title, m));
                                }
                                Ok((_, Err(msg))) => app.toast(&msg, "alert"),
                                Err(_) => app.toast(&t("common.err_unknown"), "alert"),
                            }
                        });
                    },
                    Icon { name: "plug", class: "size-4" }
                    "{label}"
                }
            }
        })}
    }
}
