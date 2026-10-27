//! Alle Fenster: Sicherheitsfrage, Fehlererklärung, Spiel hinzufügen/bearbeiten …

use super::components::*;
use super::detail::DetailModal;
use super::settings::{LanguagePicker, LayoutPicker};
use super::icons::{Icon, MazLogo};
use super::state::*;
use crate::model::CustomGame;
use crate::problem::{Fix, Problem, Severity};
use crate::i18n::{t, tf};
use crate::{config, sys};
use dioxus::prelude::*;
use std::path::Path;

#[component]
pub fn ModalHost() -> Element {
    let app = use_app();
    let modal = app.modal.read().clone();
    match modal {
        None => rsx! {},
        Some(Modal::Confirm(id)) => rsx! { ConfirmDialog { id } },
        Some(Modal::NotStarted(id)) => rsx! { NotStartedDialog { id } },
        Some(Modal::Problem(p, gid)) => rsx! { ProblemDialog { problem: p, game_id: gid } },
        Some(Modal::EditGame(id)) => rsx! { EditGameDialog { id } },
        Some(Modal::DeleteGame(id)) => rsx! { DeleteDialog { id } },
        Some(Modal::Detail(id)) => rsx! { DetailModal { id } },
        Some(Modal::Welcome) => rsx! { WelcomeDialog {} },
        Some(Modal::Update) => rsx! { UpdateDialog {} },
        Some(Modal::EditMeta(id)) => rsx! { super::manage::EditMetaDialog { id } },
        Some(Modal::FolderScan) => rsx! { super::manage::FolderScanDialog {} },
        Some(Modal::ExtInstall(staged)) => rsx! { super::extensions::ExtInstallDialog { staged } },
        Some(Modal::ExtMessage(title, text)) => rsx! {
            Dialog { on_close: move |_| app.close_modal(), class: "max-w-md",
                div { class: "p-8",
                    div { class: "flex items-center gap-3 mb-4",
                        div { class: "size-10 rounded-xl bg-primary/15 text-primary grid place-items-center", Icon { name: "plug", class: "size-5" } }
                        h2 { class: "text-lg font-bold", "{title}" }
                    }
                    p { class: "text-sm leading-relaxed whitespace-pre-line select-text", "{text}" }
                    div { class: "flex justify-end mt-6",
                        Button { variant: ButtonVariant::Glow, onclick: move |_| app.close_modal(), "OK" }
                    }
                }
            }
        },
        Some(Modal::Random) => rsx! { super::tools::RandomDialog {} },
        Some(Modal::Restore(settings)) => rsx! { super::tools::RestoreDialog { settings } },
        Some(Modal::PlayniteImport) => rsx! { super::tools::PlayniteDialog {} },
        Some(Modal::SgdbPick(id)) => rsx! { super::media::SgdbPickDialog { id } },
        Some(Modal::LaunchOpts(id)) => rsx! { super::opts::LaunchOptsDialog { id } },
        Some(Modal::AddToSteam(id)) => rsx! { super::opts::AddToSteamDialog { id } },
        Some(Modal::CrashReport(text)) => rsx! { super::opts::CrashReportDialog { text } },
        Some(Modal::EpicConnect) => rsx! {
            Dialog { on_close: move |_| app.close_modal(), class: "max-w-xl",
                div { class: "p-6 pt-8",
                    super::settings::EpicAccount {}
                    if app.settings.read().epic_auth.is_some() && !*app.epic_busy.read() {
                        div { class: "flex justify-end mt-4",
                            Button { variant: ButtonVariant::Glow, onclick: move |_| app.close_modal(),
                                Icon { name: "check", class: "size-4" }
                                {t("theme.done")}
                            }
                        }
                    }
                }
            }
        },
        Some(Modal::Info(title, text)) => rsx! {
            Dialog { on_close: move |_| app.close_modal(), class: "max-w-md",
                div { class: "p-8 text-center",
                    div { class: "mx-auto size-14 rounded-2xl maz-gradient-bg grid place-items-center text-glow-fg mb-5",
                        Icon { name: "mail", class: "size-7" }
                    }
                    h2 { class: "text-xl font-bold", "{title}" }
                    p { class: "text-sm text-muted-foreground mt-3 leading-relaxed whitespace-pre-line", "{text}" }
                    Button { variant: ButtonVariant::Glow, class: "mt-6", onclick: move |_| app.close_modal(), "OK" }
                }
            }
        },
    }
}

/// "Neue Version verfügbar" – mit Knopf zum direkten Updaten
#[component]
fn UpdateDialog() -> Element {
    let app = use_app();
    let Some(r) = app.update_avail.read().clone() else {
        return rsx! {};
    };
    let state = app.update_state.read().clone();
    let kind = crate::updater::InstallKind::detect();
    let can = kind.can_update() && crate::updater::pick_asset(&kind, &r.assets).is_some();
    let version = r.version.clone();
    let current = crate::sys::VERSION;
    let notes = r.notes.clone();
    let page = r.page.clone();
    let page2 = r.page.clone();
    let busy = matches!(state, UpdateState::Downloading(..) | UpdateState::Installing | UpdateState::Checking);
    let pct = match state {
        UpdateState::Downloading(got, total) if total > 0 => (got as f64 / total as f64 * 100.0).min(100.0),
        UpdateState::Installing => 100.0,
        _ => 0.0,
    };
    let pct_s = format!("{:.0} %", pct);
    let how = if kind.label().is_empty() { String::new() } else { tf("update.how", &[("kind", kind.label())]) };
    rsx! {
        Dialog { on_close: move |_| if !busy { app.close_modal() }, class: "max-w-xl",
            div { class: "p-8",
                div { class: "flex items-center gap-4",
                    div { class: "size-14 shrink-0 rounded-2xl maz-gradient-bg grid place-items-center text-glow-fg",
                        Icon { name: "download", class: "size-7" }
                    }
                    div {
                        h2 { class: "text-2xl font-bold font-display", {tf("update.title", &[("version", &version)])} }
                        if r.prerelease {
                            span { class: "inline-block mt-1 rounded-full border border-amber-500/50 bg-amber-500/10 px-2 py-0.5 text-xs font-medium text-amber-500", {t("update.test_badge")} }
                        }
                        p { class: "text-sm text-muted-foreground", {tf("update.current", &[("version", current)])} }
                    }
                }
                if !notes.trim().is_empty() {
                    div { class: "mt-5 max-h-72 overflow-y-auto rounded-xl bg-muted p-4 select-text",
                        super::manage::MarkdownNotes { text: notes.clone() }
                    }
                }
                {match state.clone() {
                    UpdateState::Downloading(..) | UpdateState::Installing => rsx! {
                        div { class: "mt-6",
                            div { class: "flex justify-between text-sm mb-2",
                                span { {t(if matches!(state, UpdateState::Installing) { "update.installing" } else { "update.downloading" })} }
                                span { class: "text-muted-foreground", "{pct_s}" }
                            }
                            div { class: "h-2 rounded-full bg-muted overflow-hidden",
                                div { class: "h-full maz-gradient-bg transition-all", style: "width: {pct}%" }
                            }
                        }
                    },
                    UpdateState::Ready(path) => rsx! {
                        div { class: "mt-6 rounded-xl border border-success/40 bg-success/10 p-4 text-sm", {t("update.ready")} }
                        div { class: "mt-6 flex justify-end",
                            Button { variant: ButtonVariant::Glow, onclick: move |_| crate::updater::restart(&path),
                                Icon { name: "refresh", class: "size-4" }
                                {t("update.restart")}
                            }
                        }
                    },
                    UpdateState::Done => rsx! {
                        div { class: "mt-6 rounded-xl border border-success/40 bg-success/10 p-4 text-sm", {t("update.done")} }
                    },
                    UpdateState::Manual(file, cmd) => {
                        let c2 = cmd.clone();
                        rsx! {
                            div { class: "mt-6 rounded-xl border border-border bg-muted p-4 text-sm",
                                p { {tf("update.manual", &[("file", &file)])} }
                                pre { class: "mt-3 rounded-lg bg-background p-3 text-xs font-mono whitespace-pre-wrap break-all select-text", "{cmd}" }
                                Button { variant: ButtonVariant::Outline, class: "mt-3", onclick: move |_| { crate::sys::copy_to_clipboard(&c2); app.toast(&t("problem.copied"), "check"); },
                                    Icon { name: "copy", class: "size-4" }
                                    {t("problem.copy")}
                                }
                            }
                        }
                    },
                    UpdateState::Failed(e) => rsx! {
                        div { class: "mt-6 flex items-start gap-2 rounded-xl border border-destructive/40 bg-destructive/10 p-4 text-sm",
                            Icon { name: "alert", class: "size-4 mt-0.5 shrink-0 text-destructive" }
                            span { "{e}" }
                        }
                    },
                    _ => rsx! {},
                }}
                if !matches!(state, UpdateState::Ready(_) | UpdateState::Done) {
                    if !how.is_empty() && can {
                        p { class: "mt-5 text-xs text-muted-foreground", "{how}" }
                    }
                    if !can {
                        p { class: "mt-5 text-sm text-muted-foreground", {t("update.no_auto")} }
                    }
                    div { class: "mt-6 flex flex-wrap justify-end gap-2",
                        Button { variant: ButtonVariant::Ghost, disabled: busy,
                            onclick: move |_| {
                                let v = version.clone();
                                app.update(|s| s.update_skip = v);
                                let mut u = app.update_avail;
                                u.set(None);
                                app.close_modal();
                            },
                            {t("update.skip")}
                        }
                        Button { variant: ButtonVariant::Outline, disabled: busy, onclick: move |_| { let _ = crate::sys::open_url(&page); },
                            Icon { name: "external", class: "size-4" }
                            {t("update.github")}
                        }
                        if can {
                            Button { variant: ButtonVariant::Glow, disabled: busy, onclick: move |_| super::net::start_update(app),
                                if busy { Spinner {} } else { Icon { name: "download", class: "size-4" } }
                                {t("update.now")}
                            }
                        } else {
                            Button { variant: ButtonVariant::Glow, onclick: move |_| { let _ = crate::sys::open_url(&page2); },
                                Icon { name: "download", class: "size-4" }
                                {t("update.download")}
                            }
                        }
                    }
                }
            }
        }
    }
}

fn checklist_for(app: App, id: &str) -> Vec<String> {
    let s = app.settings.read();
    let mut items: Vec<String> = s.effective_checklist().into_iter().filter(|x| !x.trim().is_empty()).collect();
    if let Some(c) = s.custom(id) {
        for line in c.checklist.lines() {
            let l = line.trim();
            if !l.is_empty() {
                items.push(l.to_string());
            }
        }
    }
    items
}

#[component]
fn Checklist(items: Vec<String>) -> Element {
    rsx! {
        ul { class: "space-y-2 text-left",
            {items.into_iter().enumerate().map(|(i, it)| rsx! {
                li { key: "{i}", class: "flex items-start gap-3 rounded-lg bg-muted px-4 py-2.5 text-sm",
                    span { class: "mt-0.5 size-5 shrink-0 rounded-full bg-primary/15 text-primary grid place-items-center",
                        Icon { name: "check", class: "size-3" }
                    }
                    span { class: "leading-relaxed", "{it}" }
                }
            })}
        }
    }
}

// ----------------------------------------------------------------------
// Sicherheitsfrage vor jedem Start eines eigenen Spiels
// ----------------------------------------------------------------------

#[component]
fn ConfirmDialog(id: String) -> Element {
    let app = use_app();
    let name = app.settings.read().custom(&id).map(|c| c.name.clone()).or_else(|| app.game(&id).map(|g| g.name)).unwrap_or_default();
    let items = checklist_for(app, &id);
    let (id_yes, id_no) = (id.clone(), id.clone());
    rsx! {
        Dialog { on_close: move |_| app.close_modal(), close_on_backdrop: false, class: "max-w-lg",
            div { class: "p-8 text-center",
                div { class: "mx-auto size-16 rounded-2xl maz-gradient-bg grid place-items-center text-glow-fg shadow-xl mb-5",
                    Icon { name: "shield", class: "size-8" }
                }
                h2 { class: "text-2xl font-bold font-display", {t("confirm.title")} }
                p { class: "text-muted-foreground mt-2 mb-6 text-sm leading-relaxed",
                    {tf("confirm.text", &[("name", &name)])}
                }
                Checklist { items }
                div { class: "grid grid-cols-2 gap-3 mt-8",
                    Button { variant: ButtonVariant::Outline, size: ButtonSize::Lg, class: "w-full",
                        onclick: move |_| app.open(Modal::NotStarted(id_no.clone())),
                        Icon { name: "x", class: "size-4" }
                        {t("common.no")}
                    }
                    Button { variant: ButtonVariant::Glow, size: ButtonSize::Lg, class: "w-full",
                        onclick: move |_| start_confirmed(app, id_yes.clone()),
                        Icon { name: "check", class: "size-4" }
                        {t("confirm.yes")}
                    }
                }
            }
        }
    }
}

#[component]
fn NotStartedDialog(id: String) -> Element {
    let app = use_app();
    let name = app.settings.read().custom(&id).map(|c| c.name.clone()).or_else(|| app.game(&id).map(|g| g.name)).unwrap_or_default();
    let items = checklist_for(app, &id);
    let id2 = id.clone();
    rsx! {
        Dialog { on_close: move |_| app.close_modal(), class: "max-w-lg",
            div { class: "p-8 text-center",
                div { class: "mx-auto size-16 rounded-2xl bg-warning/15 text-warning grid place-items-center mb-5",
                    Icon { name: "hand", class: "size-8" }
                }
                h2 { class: "text-2xl font-bold font-display", {t("notstarted.title")} }
                p { class: "text-muted-foreground mt-2 mb-6 text-sm leading-relaxed",
                    {tf("notstarted.text", &[("name", &name)])}
                }
                ol { class: "space-y-2 text-left text-sm mb-6",
                    li { class: "flex gap-3", span { class: "font-bold text-primary", "1." } span { {t("notstarted.s1")} } }
                    li { class: "flex gap-3", span { class: "font-bold text-primary", "2." } span { {t("notstarted.s2")} } }
                    li { class: "flex gap-3", span { class: "font-bold text-primary", "3." } span { {t("notstarted.s3")} } }
                }
                Checklist { items }
                div { class: "grid grid-cols-2 gap-3 mt-8",
                    Button { variant: ButtonVariant::Outline, size: ButtonSize::Lg, class: "w-full", onclick: move |_| app.close_modal(), {t("notstarted.ok")} }
                    Button { variant: ButtonVariant::Secondary, size: ButtonSize::Lg, class: "w-full", onclick: move |_| app.open(Modal::Confirm(id2.clone())),
                        {t("notstarted.ready")}
                    }
                }
            }
        }
    }
}

// ----------------------------------------------------------------------
// Fehlererklärung – ausführlich und Schritt für Schritt
// ----------------------------------------------------------------------

fn fix_label(f: &Fix) -> (String, &'static str) {
    match f {
        Fix::RunAsAdmin => (t("fix.admin"), "shield"),
        Fix::EditGame => (t("fix.edit"), "pencil"),
        Fix::OpenUrl { label, url } if url.starts_with("steam://validate") || url.contains("action=verify") => (label.clone(), "wrench"),
        Fix::OpenUrl { label, .. } => (label.clone(), "external"),
        Fix::OpenFolder { label, .. } => (label.clone(), "folder"),
        Fix::Rescan => (t("fix.rescan"), "refresh"),
        Fix::OpenSettings => (t("fix.settings"), "settings"),
        Fix::RemoveGame => (t("fix.remove"), "trash"),
    }
}

#[component]
fn ProblemDialog(problem: Problem, #[props(!optional)] game_id: Option<String>) -> Element {
    let app = use_app();
    let mut copied = use_signal(|| false);
    let (ring, icon) = match problem.severity {
        Severity::Error => ("bg-destructive/15 text-destructive", "x-circle"),
        Severity::Warning => ("bg-warning/15 text-warning", "alert"),
        Severity::Info => ("bg-info/15 text-info", "info"),
    };
    let title = problem.title.clone();
    let what = problem.what_happened.clone();
    let why = problem.why.clone();
    let tech = problem.technical.clone();
    let clip = problem.to_clipboard_text();
    let steps = problem.steps.clone();
    let fixes = problem.fixes.clone();
    let copy_icon = if copied() { "check" } else { "copy" };
    let can_edit = game_id.as_ref().map(|id| app.settings.read().custom(id).is_some()).unwrap_or(false);
    rsx! {
        Dialog { on_close: move |_| app.close_modal(), class: "max-w-2xl",
            div { class: "p-8",
                div { class: "flex items-start gap-4 pr-8",
                    div { class: "size-14 shrink-0 rounded-2xl grid place-items-center {ring}",
                        Icon { name: icon, class: "size-7" }
                    }
                    div {
                        div { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-1", {t("problem.header")} }
                        h2 { class: "text-2xl font-bold leading-tight font-display", "{title}" }
                    }
                }
                div { class: "grid gap-4 mt-6",
                    div { class: "rounded-xl bg-muted p-4",
                        div { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-1.5", {t("problem.what")} }
                        p { class: "text-[15px] leading-relaxed", "{what}" }
                    }
                    div { class: "rounded-xl bg-muted p-4",
                        div { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-1.5", {t("problem.why")} }
                        p { class: "text-[15px] leading-relaxed text-foreground/85", "{why}" }
                    }
                }
                h3 { class: "mt-7 mb-4 text-lg font-bold flex items-center gap-2",
                    Icon { name: "wrench", class: "size-5 text-primary" }
                    {t("problem.steps")}
                }
                ol { class: "space-y-3",
                    {steps.into_iter().enumerate().map(|(i, st)| {
                        let n = i + 1;
                        rsx! {
                            li { key: "{i}", class: "flex gap-4 items-start",
                                span { class: "size-8 shrink-0 rounded-full maz-gradient-bg text-glow-fg font-bold grid place-items-center text-sm shadow", "{n}" }
                                p { class: "pt-1 text-[15px] leading-relaxed select-text", "{st}" }
                            }
                        }
                    })}
                }
                if !fixes.is_empty() {
                    div { class: "mt-7 flex flex-wrap gap-2",
                        {fixes.into_iter().enumerate().filter(|(_, f)| !matches!(f, Fix::EditGame | Fix::RemoveGame | Fix::RunAsAdmin) || can_edit).map(|(i, f)| {
                            let (label, icon) = fix_label(&f);
                            let variant = if i == 0 { ButtonVariant::Glow } else { ButtonVariant::Outline };
                            let gid = game_id.clone();
                            rsx! {
                                Button { key: "{i}", variant, onclick: move |_| run_fix(app, f.clone(), gid.clone()),
                                    Icon { name: icon, class: "size-4" }
                                    "{label}"
                                }
                            }
                        })}
                    }
                }
                details { class: "mt-7 rounded-xl border border-border group",
                    summary { class: "cursor-pointer px-4 py-3 text-sm text-muted-foreground hover:text-foreground select-none",
                        {t("problem.tech")}
                    }
                    pre { class: "px-4 pb-4 text-xs text-muted-foreground whitespace-pre-wrap break-all select-text font-mono", "{tech}" }
                }
                div { class: "mt-6 flex justify-between gap-3",
                    Button { variant: ButtonVariant::Ghost, onclick: move |_| {
                            if sys::copy_to_clipboard(&clip) {
                                copied.set(true);
                            }
                        },
                        Icon { name: copy_icon, class: "size-4" }
                        {t(if copied() { "problem.copied" } else { "problem.copy" })}
                    }
                    Button { variant: ButtonVariant::Secondary, onclick: move |_| app.close_modal(), {t("common.close")} }
                }
            }
        }
    }
}

// ----------------------------------------------------------------------
// Eigenes Spiel hinzufügen / bearbeiten
// ----------------------------------------------------------------------

fn pretty_name(path: &str) -> String {
    let stem = Path::new(path).file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
    let mut out = String::new();
    let mut prev_lower = false;
    for c in stem.chars() {
        if c == '_' || c == '-' || c == '.' {
            out.push(' ');
            prev_lower = false;
            continue;
        }
        if c.is_uppercase() && prev_lower {
            out.push(' ');
        }
        prev_lower = c.is_lowercase();
        out.push(c);
    }
    let out = out
        .replace(" Win64", "")
        .replace(" Shipping", "")
        .replace(" x64", "")
        .replace(" Launcher", "");
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[component]
pub fn Field(#[props(into)] label: String, #[props(into, default)] hint: String, children: Element) -> Element {
    rsx! {
        div { class: "grid gap-2",
            Label { "{label}" }
            {children}
            if !hint.is_empty() {
                p { class: "text-xs text-muted-foreground leading-relaxed", "{hint}" }
            }
        }
    }
}

#[component]
fn EditGameDialog(#[props(!optional)] id: Option<String>) -> Element {
    let app = use_app();
    let existing: Option<CustomGame> = id.as_ref().and_then(|i| app.settings.peek().custom(i).cloned());
    let is_new = existing.is_none();
    let init = existing.clone().unwrap_or_default();
    let mut name = use_signal(|| init.name.clone());
    let mut exe = use_signal(|| init.exe.clone());
    let mut args = use_signal(|| init.args.clone());
    let mut workdir = use_signal(|| init.workdir.clone());
    let mut cover = use_signal(|| init.cover.clone());
    let mut admin = use_signal(|| init.admin);
    let mut checklist = use_signal(|| init.checklist.clone());
    let mut runner = use_signal(|| init.runner.clone());
    // Linux: gefundene Wine-/Proton-Versionen (einmal beim Öffnen suchen)
    let runners = use_hook(crate::runners::list);
    let mut error = use_signal(String::new);

    let save = move |_: MouseEvent| {
        let n = name.peek().trim().to_string();
        let e = exe.peek().trim().trim_matches('"').to_string();
        if n.is_empty() {
            error.set(t("edit.err_name"));
            return;
        }
        if e.is_empty() {
            error.set(t("edit.err_exe"));
            return;
        }
        if !Path::new(&e).exists() {
            error.set(t("edit.err_exe_missing"));
            return;
        }
        let w = workdir.peek().trim().trim_matches('"').to_string();
        if !w.is_empty() && !Path::new(&w).is_dir() {
            error.set(t("edit.err_workdir"));
            return;
        }
        let mut game = existing.clone().unwrap_or_else(|| CustomGame {
            id: config::new_id(),
            added: sys::now_secs(),
            ..Default::default()
        });
        game.name = n.clone();
        game.exe = e;
        game.args = args.peek().trim().to_string();
        game.workdir = w;
        game.cover = cover.peek().trim().trim_matches('"').to_string();
        game.admin = *admin.peek();
        game.checklist = checklist.peek().clone();
        game.runner = runner.peek().clone();
        let gid = game.id.clone();
        app.update(|s| match s.custom_games.iter_mut().find(|c| c.id == gid) {
            Some(c) => *c = game,
            None => s.custom_games.push(game),
        });
        refresh_custom(app, gid);
        app.close_modal();
        app.toast(
            &tf(if is_new { "toast.added" } else { "toast.saved" }, &[("name", &n)]),
            "check-circle",
        );
    };

    let checklist_ph = t("edit.checklist_ph");
    let show_runner = cfg!(target_os = "linux") && {
        let e = exe().to_lowercase();
        e.ends_with(".exe") || e.ends_with(".msi") || e.ends_with(".bat")
    };
    let title = t(if is_new { "lib.add_custom" } else { "fix.edit" });
    let head_icon = if is_new { "plus" } else { "pencil" };
    let err = error();
    rsx! {
        Dialog { on_close: move |_| app.close_modal(), close_on_backdrop: false, class: "max-w-2xl",
            div { class: "p-8",
                div { class: "flex items-center gap-3 mb-1",
                    div { class: "size-10 rounded-xl maz-gradient-bg grid place-items-center text-glow-fg",
                        Icon { name: head_icon, class: "size-5" }
                    }
                    h2 { class: "text-2xl font-bold font-display", "{title}" }
                }
                p { class: "text-sm text-muted-foreground mb-6",
                    {t("edit.intro")}
                }
                div { class: "grid gap-5",
                    Field { label: t("edit.exe"), hint: t("edit.exe_hint"),
                        div { class: "flex gap-2",
                            Input { class: "font-mono text-xs", placeholder: t("edit.exe_ph"), value: exe(),
                                oninput: move |v: String| exe.set(v) }
                            Button { variant: ButtonVariant::Secondary, class: "h-10",
                                onclick: move |_| {
                                    let start = exe.peek().clone();
                                    if let Some(p) = pick_game_file(Some(&start)) {
                                        if name.peek().trim().is_empty() {
                                            name.set(pretty_name(&p));
                                        }
                                        exe.set(p);
                                        error.set(String::new());
                                    }
                                },
                                Icon { name: "folder", class: "size-4" }
                                {t("common.browse")}
                            }
                        }
                    }
                    Field { label: t("edit.name"),
                        Input { placeholder: t("edit.name_ph"), value: name(), oninput: move |v: String| name.set(v) }
                    }
                    div { class: "grid grid-cols-2 gap-4",
                        Field { label: t("edit.args"), hint: t("edit.args_hint"),
                            Input { class: "font-mono text-xs", placeholder: t("edit.empty_ph"), value: args(), oninput: move |v: String| args.set(v) }
                        }
                        Field { label: t("edit.workdir"), hint: t("edit.workdir_hint"),
                            div { class: "flex gap-2",
                                Input { class: "font-mono text-xs", placeholder: t("edit.auto_ph"), value: workdir(), oninput: move |v: String| workdir.set(v) }
                                Button { variant: ButtonVariant::Secondary, size: ButtonSize::Icon, class: "size-10", title: t("edit.pick_folder"),
                                    onclick: move |_| {
                                        if let Some(p) = pick_folder(&t("edit.workdir")) {
                                            workdir.set(p);
                                        }
                                    },
                                    Icon { name: "folder", class: "size-4" }
                                }
                            }
                        }
                    }
                    Field { label: t("edit.cover"), hint: t("edit.cover_hint"),
                        div { class: "flex gap-2",
                            Input { class: "font-mono text-xs", placeholder: t("edit.auto_ph"), value: cover(), oninput: move |v: String| cover.set(v) }
                            Button { variant: ButtonVariant::Secondary, class: "h-10",
                                onclick: move |_| {
                                    if let Some(p) = pick_image_file() {
                                        cover.set(p);
                                    }
                                },
                                Icon { name: "image", class: "size-4" }
                                {t("edit.pick_image")}
                            }
                        }
                    }
                    if show_runner {
                        Field { label: t("edit.runner"), hint: t("edit.runner_hint"),
                            select {
                                class: "h-10 w-full rounded-md border border-input bg-background/60 px-3 text-sm outline-none cursor-pointer",
                                onchange: move |e| runner.set(e.value()),
                                option { value: "", selected: runner().is_empty(), {t("edit.runner_default")} }
                                {runners.iter().map(|r| {
                                    let path = r.path.clone();
                                    let label = format!("{} {}", if r.kind == crate::runners::Kind::Proton { "Proton ·" } else { "Wine ·" }, r.name);
                                    let sel = runner() == path;
                                    rsx! { option { key: "{path}", value: "{path}", selected: sel, "{label}" } }
                                })}
                            }
                            if runners.is_empty() {
                                p { class: "text-xs text-warning", {t("edit.runner_none")} }
                            }
                        }
                    }
                    div { class: "rounded-xl border border-border px-4",
                        SwitchRow {
                            title: t("edit.admin"),
                            description: t("edit.admin_hint"),
                            checked: admin(),
                            onchange: move |v: bool| admin.set(v),
                        }
                    }
                    Field { label: t("edit.checklist"), hint: t("edit.checklist_hint"),
                        textarea {
                            class: "min-h-20 w-full rounded-md border border-input bg-background/60 px-3 py-2 text-sm outline-none focus-visible:ring-[3px] focus-visible:ring-ring/40",
                            placeholder: "{checklist_ph}",
                            value: "{checklist}",
                            oninput: move |e| checklist.set(e.value()),
                        }
                    }
                }
                if !err.is_empty() {
                    div { class: "mt-5 flex items-start gap-3 rounded-xl border border-destructive/40 bg-destructive/10 px-4 py-3 text-sm animate-slide-up",
                        Icon { name: "alert", class: "size-5 text-destructive shrink-0" }
                        span { "{err}" }
                    }
                }
                div { class: "mt-7 flex justify-end gap-3",
                    Button { variant: ButtonVariant::Ghost, onclick: move |_| app.close_modal(), {t("common.cancel")} }
                    Button { variant: ButtonVariant::Glow, size: ButtonSize::Lg, onclick: save,
                        Icon { name: "check", class: "size-4" }
                        {t(if is_new { "edit.add" } else { "edit.save" })}
                    }
                }
            }
        }
    }
}

#[component]
fn DeleteDialog(id: String) -> Element {
    let app = use_app();
    let name = app.settings.read().custom(&id).map(|c| c.name.clone()).unwrap_or_default();
    let id2 = id.clone();
    rsx! {
        Dialog { on_close: move |_| app.close_modal(), class: "max-w-md",
            div { class: "p-8",
                h2 { class: "text-xl font-bold", {tf("delete.title", &[("name", &name)])} }
                p { class: "text-sm text-muted-foreground mt-3 leading-relaxed",
                    {t("delete.text")}
                }
                div { class: "mt-7 flex justify-end gap-3",
                    Button { variant: ButtonVariant::Ghost, onclick: move |_| app.close_modal(), {t("common.cancel")} }
                    Button { variant: ButtonVariant::Destructive,
                        onclick: move |_| {
                            let gid = id2.clone();
                            app.update(|s| {
                                s.custom_games.retain(|c| c.id != gid);
                                s.favorites.retain(|f| *f != gid);
                            });
                            refresh_custom(app, id2.clone());
                            app.close_modal();
                            app.toast(&t("toast.removed"), "trash");
                        },
                        Icon { name: "trash", class: "size-4" }
                        {t("delete.button")}
                    }
                }
            }
        }
    }
}

// ----------------------------------------------------------------------
// Willkommen (erster Start)
// ----------------------------------------------------------------------

#[component]
fn WelcomeDialog() -> Element {
    let app = use_app();
    let finish = move || {
        app.update(|s| s.welcome_done = true);
        app.close_modal();
    };
    rsx! {
        Dialog { on_close: move |_| finish(), close_on_backdrop: false, class: "max-w-3xl",
            div { class: "p-10",
                div { class: "flex items-center gap-4",
                    MazLogo { class: "size-14 text-base" }
                    div {
                        h2 { class: "text-3xl font-black font-display", {t("welcome.title")} " " span { class: "maz-gradient-text", "MAZ Launcher" } }
                        p { class: "text-muted-foreground mt-1", {t("welcome.sub")} }
                    }
                }
                div { class: "grid grid-cols-3 gap-4 mt-8",
                    WelcomeStep { icon: "plug", title: t("welcome.s1t"), text: t("welcome.s1") }
                    WelcomeStep { icon: "plus", title: t("welcome.s2t"), text: t("welcome.s2") }
                    WelcomeStep { icon: "palette", title: t("welcome.s3t"), text: t("welcome.s3") }
                }
                h3 { class: "mt-8 mb-3 font-semibold", {t("welcome.layout")} }
                LanguagePicker {}
                h3 { class: "mt-6 mb-3 font-semibold", {t("welcome.look")} }
                LayoutPicker {}
                div { class: "mt-8 flex justify-end",
                    Button { variant: ButtonVariant::Glow, size: ButtonSize::Lg, onclick: move |_| finish(),
                        {t("welcome.go")}
                        Icon { name: "chevron-right", class: "size-4" }
                    }
                }
            }
        }
    }
}

#[component]
fn WelcomeStep(icon: &'static str, #[props(into)] title: String, #[props(into)] text: String) -> Element {
    rsx! {
        div { class: "rounded-xl bg-muted p-5",
            div { class: "size-10 rounded-lg bg-primary/15 text-primary grid place-items-center mb-3",
                Icon { name: icon, class: "size-5" }
            }
            div { class: "font-semibold text-sm", "{title}" }
            p { class: "text-xs text-muted-foreground mt-1.5 leading-relaxed", "{text}" }
        }
    }
}
