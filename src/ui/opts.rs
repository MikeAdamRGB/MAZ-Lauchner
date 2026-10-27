//! Dialog „Starthilfen“: Programm vor/nach dem Spiel, GameMode, MangoHud, DXVK/VKD3D.

use super::components::*;
use super::icons::Icon;
use super::state::*;
use crate::i18n::{t, tf};
use crate::launchopts::{tool_available, LaunchOpts};
use crate::model::{LaunchTarget, Source};
use dioxus::prelude::*;

#[component]
pub fn LaunchOptsDialog(id: String) -> Element {
    let app = use_app();
    let mut o = use_signal(|| launch_opts(app, &id));
    let game = app.game(&id);
    let name = game.as_ref().map(|g| g.name.clone()).unwrap_or_default();
    // GameMode/MangoHud wirken nur bei Spielen, die der Launcher selbst startet
    let direct = game.as_ref().map(|g| matches!(g.launch, LaunchTarget::Exe { .. } | LaunchTarget::Custom)).unwrap_or(false);
    let is_wine = cfg!(target_os = "linux")
        && game.as_ref().map(|g| g.source == Source::Custom).unwrap_or(false)
        && app.settings.read().custom(&id).map(|c| c.exe.to_lowercase().ends_with(".exe")).unwrap_or(false);
    let linux = cfg!(target_os = "linux");
    let (has_gm, has_mh) = if linux { (tool_available("gamemoderun"), tool_available("mangohud")) } else { (false, false) };
    let pick = move |which: u8| {
        if let Some(p) = rfd::FileDialog::new().set_title(t("opts.pick")).pick_file() {
            let s = p.to_string_lossy().to_string();
            let quoted = if s.contains(' ') { format!("\"{s}\"") } else { s };
            o.with_mut(|x| if which == 0 { x.pre = quoted } else { x.post = quoted });
        }
    };
    let gid = id.clone();
    rsx! {
        Dialog { on_close: move |_| app.close_modal(), class: "max-w-xl",
            div { class: "p-8",
                div { class: "flex items-center gap-3 mb-1",
                    div { class: "size-10 rounded-xl bg-primary/15 text-primary grid place-items-center", Icon { name: "wrench", class: "size-5" } }
                    h2 { class: "text-lg font-bold", {tf("opts.title", &[("game", &name)])} }
                }
                p { class: "text-sm text-muted-foreground mb-5", {t("opts.text")} }

                div { class: "grid gap-4",
                    div { class: "grid gap-1.5",
                        label { class: "text-sm font-medium", r#for: "opt-pre", {t("opts.pre")} }
                        div { class: "flex gap-2",
                            input { id: "opt-pre", class: "flex-1 h-10 rounded-md border border-input bg-background px-3 text-sm font-mono outline-none focus-visible:ring-2 focus-visible:ring-ring",
                                value: "{o.read().pre}", placeholder: t("opts.ph"),
                                oninput: move |e| o.with_mut(|x| x.pre = e.value()) }
                            Button { variant: ButtonVariant::Outline, size: ButtonSize::Icon, title: t("opts.pick"), onclick: move |_| { let mut p = pick; p(0) },
                                Icon { name: "folder", class: "size-4" } }
                        }
                    }
                    div { class: "grid gap-1.5",
                        label { class: "text-sm font-medium", r#for: "opt-post", {t("opts.post")} }
                        div { class: "flex gap-2",
                            input { id: "opt-post", class: "flex-1 h-10 rounded-md border border-input bg-background px-3 text-sm font-mono outline-none focus-visible:ring-2 focus-visible:ring-ring",
                                value: "{o.read().post}", placeholder: t("opts.ph"),
                                oninput: move |e| o.with_mut(|x| x.post = e.value()) }
                            Button { variant: ButtonVariant::Outline, size: ButtonSize::Icon, title: t("opts.pick"), onclick: move |_| { let mut p = pick; p(1) },
                                Icon { name: "folder", class: "size-4" } }
                        }
                        p { class: "text-xs text-muted-foreground", {t("opts.post_hint")} }
                    }
                }

                if linux {
                    div { class: "mt-5 rounded-xl border border-border px-4",
                        SwitchRow { title: "GameMode",
                            description: if !has_gm { t("opts.not_installed") } else if !direct { t("opts.direct_only") } else { t("opts.gamemode") },
                            checked: o.read().gamemode, onchange: move |v: bool| o.with_mut(|x| x.gamemode = v) }
                        SwitchRow { title: "MangoHud",
                            description: if !has_mh { t("opts.not_installed") } else if !direct { t("opts.direct_only") } else { t("opts.mangohud") },
                            checked: o.read().mangohud, onchange: move |v: bool| o.with_mut(|x| x.mangohud = v) }
                    }
                    if is_wine {
                        div { class: "mt-4 rounded-xl border border-border px-4",
                            div { class: "pt-3 text-xs font-semibold uppercase tracking-wider text-muted-foreground", {t("opts.wine")} }
                            SwitchRow { title: "DXVK", description: t("opts.dxvk"), checked: o.read().dxvk, onchange: move |v: bool| o.with_mut(|x| x.dxvk = v) }
                            SwitchRow { title: "VKD3D-Proton", description: t("opts.vkd3d"), checked: o.read().vkd3d, onchange: move |v: bool| o.with_mut(|x| x.vkd3d = v) }
                            SwitchRow { title: t("opts.hud"), checked: o.read().dxvk_hud, onchange: move |v: bool| o.with_mut(|x| x.dxvk_hud = v) }
                            SwitchRow { title: t("opts.nvapi"), description: t("opts.nvapi_text"), checked: o.read().nvapi, onchange: move |v: bool| o.with_mut(|x| x.nvapi = v) }
                        }
                    }
                }

                div { class: "flex justify-between gap-2 mt-6",
                    Button { variant: ButtonVariant::Ghost, onclick: move |_| o.set(LaunchOpts::default()), {t("opts.reset")} }
                    div { class: "flex gap-2",
                        Button { variant: ButtonVariant::Ghost, onclick: move |_| app.close_modal(), {t("common.cancel")} }
                        Button { variant: ButtonVariant::Glow,
                            onclick: move |_| {
                                let v = o.peek().clone();
                                let gid = gid.clone();
                                app.update(move |s| {
                                    if v.is_default() {
                                        s.launch_opts.remove(&gid);
                                    } else {
                                        s.launch_opts.insert(gid, v);
                                    }
                                });
                                app.close_modal();
                                app.toast(&t("opts.saved"), "check-circle");
                            },
                            Icon { name: "check", class: "size-4" }
                            {t("opts.save")}
                        }
                    }
                }
            }
        }
    }
}

/// „Zu Steam hinzufügen“: Spiel (oder mit leerer ID den Launcher selbst) als Nicht-Steam-Spiel
#[component]
pub fn AddToSteamDialog(id: String) -> Element {
    let app = use_app();
    let mut check = use_signal(|| 0u32);
    let mut result = use_signal(|| None::<Result<crate::steamshortcut::AddResult, String>>);
    let _ = check();
    let file = crate::steamshortcut::shortcuts_file();
    let running = crate::steamshortcut::steam_running();
    let is_self = id.is_empty();
    let name = if is_self { crate::sys::APP_NAME.to_string() } else { app.game(&id).map(|g| g.name).unwrap_or_default() };
    let icon = app.settings.read().custom(&id).map(|c| c.exe.clone()).filter(|e| e.to_lowercase().ends_with(".exe")).unwrap_or_default();
    let (gid, n2) = (id.clone(), name.clone());
    rsx! {
        Dialog { on_close: move |_| app.close_modal(), class: "max-w-lg",
            div { class: "p-8",
                div { class: "flex items-center gap-3 mb-3",
                    div { class: "size-10 rounded-xl bg-primary/15 text-primary grid place-items-center", Icon { name: "gamepad", class: "size-5" } }
                    h2 { class: "text-lg font-bold", {tf("steamsc.title", &[("game", &name)])} }
                }
                p { class: "text-sm leading-relaxed", {t(if is_self { "steamsc.text_self" } else { "steamsc.text" })} }
                {match (&file, result()) {
                    (None, _) => rsx! {
                        div { class: "mt-4 rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm", role: "alert", {t("steamsc.no_steam")} }
                    },
                    (_, Some(Ok(crate::steamshortcut::AddResult::Added))) => rsx! {
                        div { class: "mt-4 rounded-lg border border-success/40 bg-success/10 px-3 py-2 text-sm flex gap-2", role: "status",
                            Icon { name: "check-circle", class: "size-4 text-success mt-0.5" } {t("steamsc.added")}
                        }
                    },
                    (_, Some(Ok(crate::steamshortcut::AddResult::AlreadyThere))) => rsx! {
                        div { class: "mt-4 rounded-lg border border-border px-3 py-2 text-sm", role: "status", {t("steamsc.exists")} }
                    },
                    (_, Some(Err(e))) => rsx! {
                        div { class: "mt-4 rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm", role: "alert", "{e}" }
                    },
                    (Some(_), None) if running => rsx! {
                        div { class: "mt-4 rounded-lg border border-warning/40 bg-warning/10 px-3 py-3 text-sm", role: "alert",
                            p { {t("steamsc.running")} }
                            div { class: "flex gap-2 mt-3",
                                Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm, onclick: move |_| { let _ = crate::sys::open_url("steam://exit"); },
                                    {t("steamsc.quit_steam")} }
                                Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, onclick: move |_| check += 1,
                                    Icon { name: "refresh", class: "size-3.5" } {t("steamsc.recheck")} }
                            }
                        }
                    },
                    (Some(_), None) => rsx! {},
                }}
                div { class: "flex justify-end gap-2 mt-6",
                    Button { variant: ButtonVariant::Ghost, onclick: move |_| app.close_modal(), {t("common.close")} }
                    if file.is_some() && result().is_none() {
                        Button { variant: ButtonVariant::Glow, disabled: running,
                            onclick: move |_| {
                                let Some(f) = crate::steamshortcut::shortcuts_file() else { return };
                                if crate::steamshortcut::steam_running() {
                                    check += 1;
                                    return;
                                }
                                let sc = if gid.is_empty() {
                                    crate::steamshortcut::self_command().map(|(exe, dir, pre)| crate::steamshortcut::Shortcut {
                                        name: crate::sys::APP_NAME.to_string(), exe, start_dir: dir, icon: String::new(), launch_options: pre,
                                    })
                                } else {
                                    crate::steamshortcut::for_game(&gid, &n2, &icon)
                                };
                                let Some(sc) = sc else { return };
                                result.set(Some(crate::steamshortcut::add_to_file(&f, &sc)));
                            },
                            Icon { name: "plus", class: "size-4" }
                            {t("steamsc.add")}
                        }
                    }
                }
            }
        }
    }
}

/// Absturz beim letzten Start: Bericht ansehen, kopieren oder als GitHub-Issue öffnen
#[component]
pub fn CrashReportDialog(text: String) -> Element {
    let app = use_app();
    let shown = crate::crashreport::redact(&text);
    let (t1, t2) = (shown.clone(), text.clone());
    let close = move || {
        crate::crashreport::dismiss();
        app.close_modal();
    };
    rsx! {
        Dialog { on_close: move |_| close(), class: "max-w-2xl",
            div { class: "p-8",
                div { class: "flex items-center gap-3 mb-3",
                    div { class: "size-10 rounded-xl bg-destructive/15 text-destructive grid place-items-center", Icon { name: "alert", class: "size-5" } }
                    h2 { class: "text-lg font-bold", {t("crash.title")} }
                }
                p { class: "text-sm leading-relaxed", {t("crash.text")} }
                pre { class: "mt-4 max-h-64 overflow-auto rounded-lg bg-muted p-3 text-xs font-mono whitespace-pre-wrap select-text", "aria-label": t("crash.report"), "{shown}" }
                p { class: "mt-3 text-xs text-muted-foreground leading-relaxed", {t("crash.privacy")} }
                div { class: "flex flex-wrap justify-end gap-2 mt-6",
                    Button { variant: ButtonVariant::Ghost, onclick: move |_| close(), {t("crash.dismiss")} }
                    Button { variant: ButtonVariant::Outline,
                        onclick: move |_| { crate::sys::copy_to_clipboard(&t1); app.toast(&t("crash.copied"), "check-circle"); },
                        Icon { name: "copy", class: "size-4" } {t("crash.copy")}
                    }
                    Button { variant: ButtonVariant::Glow,
                        onclick: move |_| {
                            let first = t2.lines().nth(1).unwrap_or("").chars().take(80).collect::<String>();
                            let _ = crate::sys::open_url(&crate::crashreport::issue_url(&format!("Absturz: {first}"), &t2));
                            close();
                        },
                        Icon { name: "external", class: "size-4" } {t("crash.issue")}
                    }
                }
            }
        }
    }
}

/// Einstellungen → Allgemein: Taskleisten-Symbol, Discord, Steam
#[component]
pub fn IntegrationSection() -> Element {
    let app = use_app();
    let s = app.settings.read().clone();
    let steps = crate::i18n::tl("discord.steps");
    let id_ok = crate::discord::valid_app_id(s.discord_app_id.trim());
    rsx! {
        super::settings::Section { title: t("integ.title"), description: t("integ.text"),
            SwitchRow { title: t("integ.tray"), description: t("integ.tray_text"), checked: s.tray_icon,
                onchange: move |v: bool| app.update(|s| s.tray_icon = v) }
            Separator {}
            SwitchRow { title: t("integ.discord"), description: t("integ.discord_text"), checked: s.discord_enabled,
                onchange: move |v: bool| { app.update(|s| s.discord_enabled = v); if !v { crate::discord::clear(); } } }
            if s.discord_enabled {
                div { class: "pb-3",
                    ol { class: "text-sm space-y-1 mb-3 list-decimal pl-5 leading-relaxed",
                        {steps.into_iter().enumerate().map(|(i, st)| rsx! { li { key: "{i}", "{st}" } })}
                    }
                    div { class: "flex gap-2 items-center",
                        Input { class: "font-mono text-xs", placeholder: t("discord.ph"), value: s.discord_app_id.clone(),
                            oninput: move |v: String| { let v: String = v.chars().filter(|c| c.is_ascii_digit()).take(21).collect(); app.update(|s| s.discord_app_id = v); } }
                        Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm,
                            onclick: move |_| { let _ = crate::sys::open_url("https://discord.com/developers/applications"); },
                            Icon { name: "external", class: "size-3.5" } {t("discord.portal")} }
                    }
                    if !s.discord_app_id.is_empty() && !id_ok {
                        p { class: "mt-2 text-xs text-destructive", role: "alert", {t("discord.err_id")} }
                    }
                }
            }
            Separator {}
            div { class: "py-3 flex flex-wrap items-center justify-between gap-3",
                div { class: "min-w-0",
                    div { class: "text-sm font-medium", {t("integ.steam_self")} }
                    div { class: "text-xs text-muted-foreground mt-1 leading-relaxed", {t("integ.steam_self_text")} }
                }
                Button { variant: ButtonVariant::Outline, onclick: move |_| app.open(Modal::AddToSteam(String::new())),
                    Icon { name: "gamepad", class: "size-4" } {t("steamsc.add")}
                }
            }
        }
    }
}
