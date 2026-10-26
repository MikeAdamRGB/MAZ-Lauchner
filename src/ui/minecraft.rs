//! Minecraft-Inhalte: Mods, Texturenpakete, Shader und Datenpakete von Modrinth und
//! CurseForge suchen und mit einem Klick installieren.

use super::art::ArtImage;
use super::components::*;
use super::icons::Icon;
use super::state::*;
use crate::i18n::{t, tf};
use crate::mccontent::{self as mc, Kind, Project, Provider, Target};
use crate::sys;
use dioxus::prelude::*;
use std::collections::BTreeMap;

#[derive(Clone, PartialEq)]
enum InstallState {
    Busy,
    Done(String),
    Failed(String),
}

fn kind_label(k: Kind) -> String {
    t(match k {
        Kind::Mod => "mc.kind.mod",
        Kind::ResourcePack => "mc.kind.resourcepack",
        Kind::Shader => "mc.kind.shader",
        Kind::DataPack => "mc.kind.datapack",
    })
}

fn format_count(n: u64) -> String {
    match n {
        0..=999 => n.to_string(),
        1_000..=999_999 => format!("{:.1}k", n as f64 / 1000.0),
        _ => format!("{:.1}M", n as f64 / 1_000_000.0),
    }
}

#[component]
pub fn MinecraftPage() -> Element {
    let app = use_app();
    let mut targets = use_signal(Vec::<Target>::new);
    let mut versions = use_signal(Vec::<String>::new);
    let mut target_id = use_signal(|| app.mc_target.peek().clone());
    let mut provider = use_signal(|| Provider::Modrinth);
    let mut kind = use_signal(|| Kind::Mod);
    let mut query = use_signal(String::new);
    let mut version = use_signal(String::new);
    let mut loader = use_signal(|| "fabric".to_string());
    let mut world = use_signal(String::new);
    let mut results = use_signal(Vec::<Project>::new);
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut offset = use_signal(|| 0usize);
    let mut states = use_signal(BTreeMap::<String, InstallState>::new);
    let mut fabric_busy = use_signal(|| false);
    let mut show_guide = use_signal(|| false);
    let mut key_input = use_signal(String::new);

    // Ziele (offizieller Launcher + NoRisk-Profile) und Minecraft-Versionen laden
    use_hook(move || {
        spawn(async move {
            let (t, v) = tokio::task::spawn_blocking(|| (mc::targets(), mc::game_versions())).await.unwrap_or_default();
            if version.peek().is_empty() {
                if let Some(first) = v.first() {
                    version.set(first.clone());
                }
            }
            versions.set(v);
            let wanted = target_id.peek().clone();
            if !t.iter().any(|x| x.id == wanted) {
                if let Some(first) = t.first() {
                    target_id.set(first.id.clone());
                }
            }
            targets.set(t);
        });
    });

    let target = targets.read().iter().find(|x| x.id == *target_id.read()).cloned();
    // NoRisk-Profil: Version und Loader stehen fest
    let eff_version = target.as_ref().and_then(|t| t.game_version.clone()).unwrap_or_else(|| version());
    let eff_loader = target.as_ref().and_then(|t| t.loader.clone()).unwrap_or_else(|| loader());
    let fixed = target.as_ref().map(|t| t.game_version.is_some()).unwrap_or(false);
    let cf_key = app.settings.read().curseforge_key.clone();
    let worlds = target.as_ref().map(|t| mc::worlds(&t.dir)).unwrap_or_default();
    // Ohne Auswahl: erste Welt
    let eff_world = if world().is_empty() { worlds.first().cloned().unwrap_or_default() } else { world() };

    let mut run_search = move |reset: bool| {
        let Some(_) = targets.peek().iter().find(|x| x.id == *target_id.peek()) else { return };
        if reset {
            offset.set(0);
            results.set(Vec::new());
        }
        let (p, k, q) = (*provider.peek(), *kind.peek(), query.peek().clone());
        let tgt = targets.peek().iter().find(|x| x.id == *target_id.peek()).cloned();
        let gv = tgt.as_ref().and_then(|t| t.game_version.clone()).unwrap_or_else(|| version.peek().clone());
        let lo = tgt.as_ref().and_then(|t| t.loader.clone()).unwrap_or_else(|| loader.peek().clone());
        let off = *offset.peek();
        let key = app.settings.peek().curseforge_key.clone();
        loading.set(true);
        error.set(None);
        spawn(async move {
            let res = tokio::task::spawn_blocking(move || {
                let q = mc::Query { text: &q, kind: k, game_version: &gv, loader: &lo, offset: off };
                mc::search(p, &q, &key)
            })
            .await
            .unwrap_or_else(|_| Err(t("common.err_unknown")));
            loading.set(false);
            match res {
                Ok(list) => {
                    let n = list.len();
                    results.write().extend(list);
                    offset.set(off + n);
                }
                Err(e) => error.set(Some(e)),
            }
        });
    };

    // Erste Suche, sobald die Ziele da sind
    use_effect(move || {
        if !targets.read().is_empty() && results.peek().is_empty() && !*loading.peek() {
            run_search(true);
        }
    });

    let mut install = move |p: Project| {
        let Some(tgt) = targets.peek().iter().find(|x| x.id == *target_id.peek()).cloned() else { return };
        let gv = tgt.game_version.clone().unwrap_or_else(|| version.peek().clone());
        let lo = tgt.loader.clone().unwrap_or_else(|| loader.peek().clone());
        let w = if world.peek().is_empty() { mc::worlds(&tgt.dir).into_iter().next().unwrap_or_default() } else { world.peek().clone() };
        let (k, key) = (*kind.peek(), app.settings.peek().curseforge_key.clone());
        let prov = p.provider.unwrap_or(Provider::Modrinth);
        let pid = p.id.clone();
        let title = p.title.clone();
        states.write().insert(pid.clone(), InstallState::Busy);
        spawn(async move {
            let id2 = pid.clone();
            let res = tokio::task::spawn_blocking(move || {
                mc::install(&mc::InstallRequest {
                    provider: prov,
                    project: &id2,
                    kind: k,
                    game_version: &gv,
                    loader: &lo,
                    game_dir: &tgt.dir,
                    world: &w,
                    cf_key: &key,
                })
            })
            .await
            .unwrap_or_else(|_| Err(t("common.err_unknown")));
            match res {
                Ok(done) => {
                    let n = done.files.len();
                    states.write().insert(pid, InstallState::Done(done.files.join(", ")));
                    app.toast(&tf("mc.installed_toast", &[("name", &title), ("n", &n.to_string())]), "check-circle");
                }
                Err(e) => {
                    states.write().insert(pid, InstallState::Failed(e));
                }
            }
        });
    };

    let tlist = targets.read().clone();
    let vlist = versions.read().clone();
    let list = results.read().clone();
    let st = states.read().clone();
    let busy = loading();
    let err = error.read().clone();
    let need_key = provider() == Provider::CurseForge && cf_key.trim().is_empty();
    let fabric_missing = target.as_ref().map(|t| t.official).unwrap_or(false)
        && kind() == Kind::Mod
        && eff_loader == "fabric"
        && mc::valid_version(&eff_version)
        && target.as_ref().map(|t| !mc::fabric_installed(&t.dir, &eff_version)).unwrap_or(false);
    let target_dir = target.as_ref().map(|t| t.dir.to_string_lossy().to_string()).unwrap_or_default();
    let has_more = !list.is_empty() && list.len() % 24 == 0;

    rsx! {
        div { class: "h-full overflow-y-auto maz-ambient",
            div { class: "max-w-[1600px] mx-auto px-8 py-7",
                div { class: "flex items-center gap-4 mb-6",
                    div { class: "size-12 rounded-2xl bg-[#3c8527] text-white grid place-items-center shadow-lg",
                        Icon { name: "cube", class: "size-6" }
                    }
                    div { class: "mr-auto",
                        h1 { class: "text-2xl font-bold tracking-tight font-display", {t("mc.title")} }
                        p { class: "text-sm text-muted-foreground", {t("mc.sub")} }
                    }
                    if !target_dir.is_empty() {
                        Button { variant: ButtonVariant::Outline, onclick: move |_| sys::open_folder(&target_dir),
                            Icon { name: "folder", class: "size-4" }
                            {t("mc.open_folder")}
                        }
                    }
                }

                if tlist.is_empty() {
                    div { class: "rounded-xl border border-border bg-card/60 p-6 text-sm",
                        div { class: "font-semibold mb-1", {t("mc.no_target_title")} }
                        p { class: "text-muted-foreground mb-4", {t("mc.no_target_text")} }
                        div { class: "flex flex-wrap gap-2",
                            Button { variant: ButtonVariant::Outline, onclick: move |_| { let _ = sys::open_url(crate::minecraft::MC_DOWNLOAD); },
                                Icon { name: "download", class: "size-4" } "Minecraft Launcher"
                            }
                            Button { variant: ButtonVariant::Outline, onclick: move |_| { let _ = sys::open_url(crate::minecraft::NORISK_DOWNLOAD); },
                                Icon { name: "download", class: "size-4" } "NoRisk Client"
                            }
                        }
                    }
                } else {
                    // Ziel + Quelle
                    div { class: "flex flex-wrap items-center gap-3 mb-4",
                        span { class: "text-sm text-muted-foreground", {t("mc.target")} }
                        select {
                            class: "h-9 rounded-md border border-input bg-background/60 px-3 text-sm outline-none cursor-pointer",
                            onchange: move |e| { target_id.set(e.value()); world.set(String::new()); run_search(true); },
                            {tlist.iter().map(|x| {
                                let sel = *target_id.read() == x.id;
                                let (id, label) = (x.id.clone(), x.label.clone());
                                rsx! { option { key: "{id}", value: "{id}", selected: sel, "{label}" } }
                            })}
                        }
                        div { class: "flex-1" }
                        Segmented {
                            options: vec![("modrinth".to_string(), "Modrinth".to_string()), ("curseforge".to_string(), "CurseForge".to_string())],
                            value: if provider() == Provider::Modrinth { "modrinth".to_string() } else { "curseforge".to_string() },
                            onchange: move |v: String| {
                                provider.set(if v == "curseforge" { Provider::CurseForge } else { Provider::Modrinth });
                                run_search(true);
                            },
                        }
                    }
                    // Art
                    div { class: "flex flex-wrap gap-2 mb-4",
                        {Kind::ALL.into_iter().map(|k| {
                            let active = kind() == k;
                            let cls = if active {
                                "h-9 px-4 rounded-full text-sm font-medium bg-primary text-primary-foreground cursor-pointer"
                            } else {
                                "h-9 px-4 rounded-full text-sm font-medium bg-secondary text-secondary-foreground hover:bg-accent cursor-pointer"
                            };
                            let label = kind_label(k);
                            rsx! { button { key: "{label}", class: "{cls}", onclick: move |_| { kind.set(k); run_search(true); }, "{label}" } }
                        })}
                    }
                    // Filter + Suche
                    div { class: "flex flex-wrap items-center gap-3 mb-5",
                        div { class: "relative flex-1 min-w-64",
                            span { class: "absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground pointer-events-none",
                                Icon { name: "search", class: "size-4" }
                            }
                            Input { class: "pl-9 h-9", placeholder: t("mc.search_ph"), value: query(),
                                oninput: move |v: String| query.set(v),
                                onenter: move |_| run_search(true),
                            }
                        }
                        if fixed {
                            span { class: "h-9 px-3 rounded-md border border-border text-sm grid place-items-center text-muted-foreground",
                                {format!("{eff_version} · {}", crate::minecraft::loader_label(&eff_loader))}
                            }
                        } else {
                            select {
                                class: "h-9 rounded-md border border-input bg-background/60 px-3 text-sm outline-none cursor-pointer",
                                title: t("mc.version"),
                                onchange: move |e| { version.set(e.value()); run_search(true); },
                                {vlist.iter().map(|v| {
                                    let sel = *v == eff_version;
                                    let v = v.clone();
                                    rsx! { option { key: "{v}", value: "{v}", selected: sel, "{v}" } }
                                })}
                            }
                            if kind() == Kind::Mod {
                                select {
                                    class: "h-9 rounded-md border border-input bg-background/60 px-3 text-sm outline-none cursor-pointer",
                                    title: t("mc.loader"),
                                    onchange: move |e| { loader.set(e.value()); run_search(true); },
                                    for l in ["fabric", "neoforge", "forge", "quilt"] {
                                        option { key: "{l}", value: "{l}", selected: eff_loader == l, {crate::minecraft::loader_label(l)} }
                                    }
                                }
                            }
                        }
                        if kind() == Kind::DataPack {
                            select {
                                class: "h-9 rounded-md border border-input bg-background/60 px-3 text-sm outline-none cursor-pointer",
                                title: t("mc.world"),
                                onchange: move |e| world.set(e.value()),
                                if worlds.is_empty() {
                                    option { value: "", {t("mc.no_worlds")} }
                                }
                                {worlds.iter().map(|w| {
                                    let sel = *w == eff_world;
                                    let w = w.clone();
                                    rsx! { option { key: "{w}", value: "{w}", selected: sel, "{w}" } }
                                })}
                            }
                        }
                        Button { variant: ButtonVariant::Secondary, class: "h-9", disabled: busy, onclick: move |_| run_search(true),
                            if busy { Spinner {} } else { Icon { name: "search", class: "size-4" } }
                            {t("scan.search")}
                        }
                    }

                    // Hinweise
                    if fabric_missing {
                        div { class: "mb-5 flex flex-wrap items-center gap-3 rounded-xl border border-warning/40 bg-warning/10 px-4 py-3 text-sm",
                            Icon { name: "info", class: "size-4 text-warning" }
                            span { class: "flex-1", {tf("mc.fabric_missing", &[("version", &eff_version)])} }
                            Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm, disabled: fabric_busy(),
                                onclick: move |_| {
                                    let Some(tgt) = targets.peek().iter().find(|x| x.id == *target_id.peek()).cloned() else { return };
                                    let gv = version.peek().clone();
                                    fabric_busy.set(true);
                                    spawn(async move {
                                        let res = tokio::task::spawn_blocking(move || mc::install_fabric(&tgt.dir, &gv)).await.unwrap_or_else(|_| Err(t("common.err_unknown")));
                                        fabric_busy.set(false);
                                        match res {
                                            Ok(id) => app.toast(&tf("mc.fabric_ok", &[("id", &id)]), "check-circle"),
                                            Err(e) => error.set(Some(e)),
                                        }
                                    });
                                },
                                if fabric_busy() { Spinner {} } else { Icon { name: "download", class: "size-3.5" } }
                                {t("mc.fabric_install")}
                            }
                        }
                    }
                    if !fixed && kind() == Kind::Mod && matches!(eff_loader.as_str(), "forge" | "neoforge") {
                        div { class: "mb-5 flex items-center gap-3 rounded-xl border border-border bg-card/60 px-4 py-3 text-sm text-muted-foreground",
                            Icon { name: "info", class: "size-4 text-primary" }
                            span { class: "flex-1", {t("mc.forge_hint")} }
                            Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm,
                                onclick: move |_| { let _ = sys::open_url(if loader.peek().as_str() == "forge" { "https://files.minecraftforge.net/" } else { "https://neoforged.net/" }); },
                                Icon { name: "external", class: "size-3.5" } {t("mc.official_installer")}
                            }
                        }
                    }
                    if kind() == Kind::Shader {
                        div { class: "mb-5 flex items-center gap-3 rounded-xl border border-border bg-card/60 px-4 py-3 text-sm text-muted-foreground",
                            Icon { name: "info", class: "size-4 text-primary" }
                            {t("mc.shader_hint")}
                        }
                    }

                    if need_key {
                        CurseForgeKeyBox { show_guide: show_guide(), on_toggle: move |_| show_guide.set(!show_guide()), key_input: key_input(),
                            on_input: move |v: String| key_input.set(v),
                            on_save: move |_| {
                                let k = key_input.peek().trim().to_string();
                                if k.len() >= 20 && k.len() < 200 && k.chars().all(|c| c.is_ascii_graphic()) {
                                    app.update(|s| s.curseforge_key = k);
                                    key_input.set(String::new());
                                    app.toast(&t("mc.key_saved"), "check-circle");
                                    run_search(true);
                                } else {
                                    app.toast(&t("mc.key_invalid"), "alert");
                                }
                            },
                        }
                    } else {
                        if let Some(e) = err {
                            div { class: "mb-5 flex items-start gap-2 rounded-lg border border-destructive/40 bg-destructive/10 px-3 py-2 text-sm",
                                Icon { name: "alert", class: "size-4 mt-0.5 shrink-0 text-destructive" }
                                span { "{e}" }
                            }
                        }
                        if list.is_empty() && busy {
                            div { class: "grid gap-4", style: "grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));",
                                for i in 0..9 { div { key: "{i}", class: "h-32 rounded-xl skeleton" } }
                            }
                        } else if list.is_empty() {
                            div { class: "py-16 text-center text-sm text-muted-foreground", {t("mc.nothing")} }
                        }
                        div { class: "grid gap-4", style: "grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));",
                            {list.into_iter().map(|p| {
                                let state = st.get(&p.id).cloned();
                                let key = format!("{:?}-{}", p.provider, p.id);
                                let p2 = p.clone();
                                let page = p.page.clone();
                                let page2 = p.page.clone();
                                let icon = if p.icon.is_empty() { Vec::new() } else { vec![p.icon.clone()] };
                                let downloads = format_count(p.downloads);
                                let (title, author, desc) = (p.title.clone(), p.author.clone(), p.description.clone());
                                let allowed = p.distribution_allowed;
                                rsx! {
                                    div { key: "{key}", class: "flex gap-4 rounded-xl border border-border bg-card/60 p-4",
                                        div { class: "relative size-16 shrink-0 rounded-lg overflow-hidden bg-muted",
                                            ArtImage { srcs: icon, class: "absolute inset-0 size-full object-cover",
                                                fallback: rsx! { div { class: "absolute inset-0 grid place-items-center text-muted-foreground", Icon { name: "cube", class: "size-7" } } } }
                                        }
                                        div { class: "min-w-0 flex-1 flex flex-col",
                                            div { class: "font-semibold truncate", title: "{title}", "{title}" }
                                            div { class: "text-xs text-muted-foreground truncate",
                                                if !author.is_empty() { "{author} · " }
                                                {tf("mc.downloads", &[("n", &downloads)])}
                                            }
                                            p { class: "text-xs text-muted-foreground mt-1 line-clamp-2", "{desc}" }
                                            div { class: "mt-auto pt-3 flex flex-wrap items-center gap-2",
                                                {match state {
                                                    Some(InstallState::Busy) => rsx! { Button { size: ButtonSize::Sm, disabled: true, Spinner {} {t("mc.installing")} } },
                                                    Some(InstallState::Done(files)) => rsx! {
                                                        span { class: "inline-flex items-center gap-1 text-xs text-success font-medium", title: "{files}",
                                                            Icon { name: "check-circle", class: "size-4" } {t("mc.installed")}
                                                        }
                                                    },
                                                    Some(InstallState::Failed(e)) => rsx! {
                                                        span { class: "text-xs text-destructive line-clamp-2", title: "{e}", "{e}" }
                                                        Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm, onclick: move |_| install(p2.clone()), {t("mc.retry")} }
                                                    },
                                                    None if allowed => rsx! {
                                                        Button { variant: ButtonVariant::Glow, size: ButtonSize::Sm, onclick: move |_| install(p2.clone()),
                                                            Icon { name: "download", class: "size-3.5" } {t("game.install")}
                                                        }
                                                    },
                                                    None => rsx! {
                                                        Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm, title: t("mc.cf_only_tip"), onclick: move |_| { let _ = sys::open_url(&page2); },
                                                            Icon { name: "external", class: "size-3.5" } {t("mc.cf_only")}
                                                        }
                                                    },
                                                }}
                                                Button { variant: ButtonVariant::Ghost, size: ButtonSize::Sm, onclick: move |_| { let _ = sys::open_url(&page); },
                                                    {t("mc.page")}
                                                }
                                            }
                                        }
                                    }
                                }
                            })}
                        }
                        if has_more {
                            div { class: "flex justify-center mt-6",
                                Button { variant: ButtonVariant::Outline, disabled: busy, onclick: move |_| run_search(false),
                                    if busy { Spinner {} }
                                    {t("mc.more")}
                                }
                            }
                        }
                    }
                    p { class: "mt-8 text-xs text-muted-foreground leading-relaxed", {t("mc.legal")} }
                }
            }
        }
    }
}

/// CurseForge-Schlüssel eintragen – mit Schritt-für-Schritt-Anleitung
#[component]
pub fn CurseForgeKeyBox(show_guide: bool, on_toggle: EventHandler<()>, key_input: String, on_input: EventHandler<String>, on_save: EventHandler<()>) -> Element {
    let steps = crate::i18n::tl("mc.key_steps");
    rsx! {
        div { class: "rounded-xl border border-primary/30 bg-primary/5 p-5",
            div { class: "flex items-start gap-3",
                div { class: "size-10 shrink-0 rounded-xl bg-[#f16436] text-white grid place-items-center",
                    Icon { name: "key", class: "size-5" }
                }
                div { class: "flex-1 min-w-0",
                    div { class: "font-semibold", {t("mc.key_title")} }
                    p { class: "text-sm text-muted-foreground mt-1 leading-relaxed", {t("mc.key_text")} }
                }
            }
            button { class: "mt-4 text-sm font-medium text-primary hover:underline cursor-pointer", onclick: move |_| on_toggle.call(()),
                {t(if show_guide { "mc.key_guide_hide" } else { "mc.key_guide" })}
            }
            if show_guide {
                ol { class: "mt-3 space-y-2",
                    {steps.into_iter().enumerate().map(|(i, st)| {
                        let n = i + 1;
                        rsx! {
                            li { key: "{i}", class: "flex gap-3 items-start text-sm",
                                span { class: "size-6 shrink-0 rounded-full maz-gradient-bg text-white text-xs font-bold grid place-items-center", "{n}" }
                                span { class: "pt-0.5 leading-relaxed", "{st}" }
                            }
                        }
                    })}
                }
                Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm, class: "mt-3",
                    onclick: move |_| { let _ = sys::open_url("https://console.curseforge.com/"); },
                    Icon { name: "external", class: "size-3.5" } {t("mc.key_open_console")}
                }
            }
            div { class: "flex gap-2 mt-4",
                Input { kind: "password", class: "font-mono text-xs", placeholder: t("mc.key_ph"), value: key_input, oninput: move |v: String| on_input.call(v), onenter: move |_| on_save.call(()) }
                Button { variant: ButtonVariant::Glow, class: "h-10", onclick: move |_| on_save.call(()),
                    Icon { name: "check", class: "size-4" } {t("mc.key_save")}
                }
            }
            p { class: "mt-2 text-xs text-muted-foreground flex gap-1.5",
                Icon { name: "shield", class: "size-3.5 mt-0.5 shrink-0" }
                {t("mc.key_privacy")}
            }
        }
    }
}
