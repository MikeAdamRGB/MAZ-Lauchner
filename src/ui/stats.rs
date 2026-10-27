//! Statistik-Seite: Spielzeit pro Woche/Monat/Store, meistgespielt, ungespielt, Status.

use super::art::GameIcon;
use super::components::*;
use super::icons::Icon;
use super::state::*;
use crate::i18n::{t, tf};
use crate::stats::{compute, fmt_minutes};
use dioxus::prelude::*;

#[component]
fn Tile(#[props(into)] label: String, #[props(into)] value: String, #[props(into, default)] hint: String, icon: &'static str) -> Element {
    rsx! {
        div { class: "rounded-xl border border-border bg-card/70 p-5",
            div { class: "flex items-center gap-2 text-sm text-foreground/80",
                Icon { name: icon, class: "size-4 text-primary" }
                "{label}"
            }
            div { class: "mt-2 text-3xl font-bold font-display tabular-nums", "{value}" }
            if !hint.is_empty() {
                div { class: "mt-1 text-xs text-muted-foreground", "{hint}" }
            }
        }
    }
}

#[component]
pub fn StatsPage() -> Element {
    let app = use_app();
    let st = {
        let s = app.settings.read();
        let games = app.games.read();
        compute(&games, &s, crate::sys::now_secs())
    };
    let max_week = st.weeks.iter().copied().max().unwrap_or(0).max(1);
    let max_store = st.per_store.iter().map(|r| r.minutes).max().unwrap_or(0).max(1);
    let weeks = st.weeks.clone();
    let n_weeks = weeks.len();
    rsx! {
        div { class: "h-full overflow-y-auto maz-ambient",
            div { class: "max-w-[1200px] mx-auto px-8 py-7",
                div { class: "flex items-center gap-4 mb-6",
                    div { class: "size-12 rounded-2xl maz-gradient-bg text-glow-fg grid place-items-center shadow-lg",
                        Icon { name: "chart", class: "size-6" }
                    }
                    div { class: "mr-auto",
                        h1 { class: "text-2xl font-bold tracking-tight font-display", {t("stats.title")} }
                        p { class: "text-sm text-muted-foreground", {t("stats.sub")} }
                    }
                    Button { variant: ButtonVariant::Outline, onclick: move |_| app.open(Modal::Random),
                        Icon { name: "dice", class: "size-4" }
                        {t("random.button")}
                    }
                }

                div { class: "grid gap-4 grid-cols-2 lg:grid-cols-4",
                    Tile { label: t("stats.total"), value: fmt_minutes(st.total_min), icon: "clock" }
                    Tile { label: t("stats.week"), value: fmt_minutes(st.week_min), hint: t("stats.measured"), icon: "timer" }
                    Tile { label: t("stats.month"), value: fmt_minutes(st.month_min), hint: t("stats.measured"), icon: "timer" }
                    Tile { label: t("stats.unplayed"), value: st.unplayed.to_string(), hint: tf("stats.of_games", &[("n", &st.games.to_string())]), icon: "flag" }
                }

                // Letzte 12 Wochen
                div { class: "mt-6 rounded-xl border border-border bg-card/70 p-5",
                    div { class: "flex items-baseline justify-between mb-4",
                        h2 { class: "font-semibold", {t("stats.weeks")} }
                        span { class: "text-xs text-muted-foreground", {t("stats.measured")} }
                    }
                    div { class: "flex items-end gap-2 h-40", role: "img", "aria-label": t("stats.weeks"),
                        {weeks.into_iter().enumerate().map(|(i, m)| {
                            let h = (m as f64 / max_week as f64 * 100.0).max(if m > 0 { 3.0 } else { 0.0 });
                            let label = if i + 1 == n_weeks { t("stats.this_week") } else { tf("stats.weeks_ago", &[("n", &(n_weeks - 1 - i).to_string())]) };
                            let tip = format!("{label}: {}", fmt_minutes(m));
                            rsx! {
                                div { key: "{i}", class: "flex-1 h-full flex flex-col justify-end items-center gap-1", title: "{tip}",
                                    div { class: "w-full rounded-t-md maz-gradient-bg", style: "height: {h}%;" }
                                }
                            }
                        })}
                    }
                    div { class: "flex justify-between mt-2 text-[11px] text-muted-foreground",
                        span { {tf("stats.weeks_ago", &[("n", &(n_weeks.saturating_sub(1)).to_string())])} }
                        span { {t("stats.this_week")} }
                    }
                }

                div { class: "mt-6 grid gap-6 lg:grid-cols-2",
                    // Pro Store
                    div { class: "rounded-xl border border-border bg-card/70 p-5",
                        h2 { class: "font-semibold mb-4", {t("stats.per_store")} }
                        div { class: "space-y-3",
                            {st.per_store.iter().map(|r| {
                                let w = r.minutes as f64 / max_store as f64 * 100.0;
                                let label = r.label.clone();
                                let right = format!("{} · {}", fmt_minutes(r.minutes), tf("lib.count", &[("n", &r.games.to_string())]));
                                rsx! {
                                    div { key: "{label}",
                                        div { class: "flex justify-between text-sm mb-1",
                                            span { class: "font-medium", "{label}" }
                                            span { class: "text-muted-foreground tabular-nums", "{right}" }
                                        }
                                        div { class: "h-2 rounded-full bg-muted overflow-hidden",
                                            div { class: "h-full rounded-full maz-gradient-bg", style: "width: {w}%;" }
                                        }
                                    }
                                }
                            })}
                        }
                    }
                    // Meistgespielt
                    div { class: "rounded-xl border border-border bg-card/70 p-5",
                        h2 { class: "font-semibold mb-4", {t("stats.top")} }
                        if st.top.is_empty() {
                            p { class: "text-sm text-muted-foreground", {t("stats.no_playtime")} }
                        }
                        ol { class: "space-y-1",
                            {st.top.iter().enumerate().map(|(i, (id, name, m))| {
                                let (gid, gid2) = (id.clone(), id.clone());
                                let n = i + 1;
                                let time = fmt_minutes(*m);
                                let name = name.clone();
                                let game = app.game(&gid2);
                                rsx! {
                                    li { key: "{gid}",
                                        button { class: "w-full flex items-center gap-3 rounded-lg px-2 py-1.5 hover:bg-accent text-left cursor-pointer",
                                            onclick: move |_| app.open(Modal::Detail(gid.clone())),
                                            span { class: "w-5 text-right text-sm font-bold tabular-nums text-muted-foreground", "{n}" }
                                            {game.map(|g| rsx! { GameIcon { game: g, class: "size-8 rounded-md" } })}
                                            span { class: "flex-1 truncate text-sm font-medium", "{name}" }
                                            span { class: "text-sm tabular-nums text-foreground/80", "{time}" }
                                        }
                                    }
                                }
                            })}
                        }
                    }
                }

                // Status
                div { class: "mt-6 rounded-xl border border-border bg-card/70 p-5",
                    h2 { class: "font-semibold mb-4", {t("nav.status")} }
                    div { class: "grid gap-3 grid-cols-2 md:grid-cols-4",
                        {st.by_status.iter().map(|(k, n)| {
                            let k = *k;
                            let n = *n;
                            rsx! {
                                button { key: "{k}", class: "rounded-lg border border-border p-4 text-left hover:border-primary cursor-pointer",
                                    onclick: move |_| app.set_filter(Filter::Status, k),
                                    div { class: "flex items-center gap-2 text-sm text-foreground/80",
                                        Icon { name: crate::model::status::icon(k), class: "size-4" }
                                        {t(&format!("status.{k}"))}
                                    }
                                    div { class: "mt-1 text-2xl font-bold tabular-nums", "{n}" }
                                }
                            }
                        })}
                    }
                    p { class: "mt-3 text-xs text-muted-foreground", {t("stats.status_hint")} }
                }
                p { class: "mt-6 text-xs text-muted-foreground leading-relaxed", {t("stats.note")} }
            }
        }
    }
}
