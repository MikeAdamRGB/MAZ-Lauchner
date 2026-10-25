//! Bilder mit automatischem Ersatz: Wenn ein Bild nicht lädt, wird das nächste
//! probiert – und am Ende ein schönes, generiertes Cover angezeigt.

use super::icons::Icon;
use crate::model::Game;
use crate::sys;
use dioxus::prelude::*;

/// Bild mit Ersatzliste. Ändert sich die Liste (z. B. Cover nachträglich gefunden),
/// wird die innere Komponente über den Schlüssel neu aufgebaut.
pub fn safe_img(u: &str) -> bool {
    (u.starts_with("https://") || u.starts_with("/maz/f/")) && !u.contains('"')
}

#[component]
pub fn ArtImage(
    srcs: Vec<String>,
    #[props(into, default)] class: String,
    #[props(into, default)] alt: String,
    fallback: Element,
) -> Element {
    // SICHERHEIT: Nur sichere Bildquellen (https:// oder lokale Bilder über /maz/f/)
    let srcs: Vec<String> = srcs.into_iter().filter(|u| safe_img(u)).collect();
    let k = art_key(&srcs);
    rsx! {
        {std::iter::once(rsx! {
            ArtImageInner { key: "{k}", srcs, class, alt, fallback }
        })}
    }
}

#[component]
fn ArtImageInner(srcs: Vec<String>, class: String, alt: String, fallback: Element) -> Element {
    let mut idx = use_signal(|| 0usize);
    let i = idx();
    match srcs.get(i) {
        Some(src) => {
            let src = src.clone();
            rsx! {
                img {
                    src: "{src}",
                    class: "{class}",
                    alt: "{alt}",
                    "loading": "lazy",
                    "draggable": "false",
                    onerror: move |_| idx.set(i + 1),
                }
            }
        }
        None => fallback,
    }
}

const GRADIENTS: &[&str] = &[
    "bg-linear-to-br from-violet-600 via-indigo-700 to-slate-950",
    "bg-linear-to-br from-cyan-500 via-blue-700 to-slate-950",
    "bg-linear-to-br from-rose-500 via-purple-700 to-slate-950",
    "bg-linear-to-br from-amber-500 via-red-700 to-slate-950",
    "bg-linear-to-br from-emerald-500 via-teal-700 to-slate-950",
    "bg-linear-to-br from-fuchsia-500 via-violet-700 to-slate-950",
    "bg-linear-to-br from-sky-500 via-indigo-700 to-slate-950",
    "bg-linear-to-br from-lime-500 via-emerald-700 to-slate-950",
];

pub fn gradient_for(name: &str) -> &'static str {
    GRADIENTS[(sys::fnv1a(name) % GRADIENTS.len() as u64) as usize]
}

pub fn initials(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().filter(|w| w.chars().next().map(|c| c.is_alphanumeric()).unwrap_or(false)).collect();
    let mut s: String = words.iter().take(2).filter_map(|w| w.chars().next()).collect();
    if s.is_empty() {
        s = "?".into();
    }
    s.to_uppercase()
}

/// Generiertes Cover: Farbverlauf + Original-Icon + Name
#[component]
pub fn CoverFallback(game: Game, #[props(default = true)] show_name: bool) -> Element {
    let grad = gradient_for(&game.name);
    let name = game.name.clone();
    let ini = initials(&game.name);
    let icons = game.art.icon.clone();
    let src_icon = game.source.icon();
    rsx! {
        div { class: "absolute inset-0 {grad} flex flex-col items-center justify-center gap-3 p-4 text-center overflow-hidden",
            div { class: "absolute -right-6 -bottom-6 opacity-10 text-white",
                Icon { name: src_icon, class: "size-40" }
            }
            div { class: "relative w-[42%] min-w-12 max-w-28 aspect-square rounded-2xl bg-white/10 backdrop-blur ring-1 ring-white/20 grid place-items-center shadow-xl overflow-hidden",
                ArtImage {
                    srcs: icons,
                    class: "size-[78%] object-contain drop-shadow-lg",
                    fallback: rsx! {
                        span { class: "text-white/90 font-black text-2xl tracking-tight", "{ini}" }
                    },
                }
            }
            if show_name {
                div { class: "relative text-white font-bold leading-tight text-sm line-clamp-3 drop-shadow", "{name}" }
            }
        }
    }
}

/// Kleines Original-Icon des Spiels (für Listen)
#[component]
pub fn GameIcon(game: Game, #[props(into, default)] class: String) -> Element {
    let cls = if class.is_empty() { "size-5".to_string() } else { class };
    let grad = gradient_for(&game.name);
    let ini = initials(&game.name);
    let icons = game.art.icon.clone();
    rsx! {
        div { class: "relative shrink-0 overflow-hidden rounded {cls}",
            ArtImage {
                srcs: icons,
                class: "absolute inset-0 size-full object-cover",
                fallback: rsx! {
                    div { class: "absolute inset-0 {grad} grid place-items-center text-[9px] font-bold text-white", "{ini}" }
                },
            }
        }
    }
}

/// Schlüssel, damit ein Bild neu geladen wird, wenn sich die Bildliste ändert
pub fn art_key(srcs: &[String]) -> String {
    format!("{}:{}", srcs.len(), srcs.first().map(|s| s.as_str()).unwrap_or(""))
}
