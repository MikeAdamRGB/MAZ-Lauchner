//! UI-Bausteine nach dem Vorbild von rust-ui (https://github.com/rust-ui/ui, MIT).
//! Gleiche Varianten und Tailwind-Klassen wie dort, nur ohne `tw_merge`,
//! damit keine zusätzliche Abhängigkeit in die EXE wandert.

use super::icons::Icon;
use dioxus::prelude::*;

// ----------------------------------------------------------------------
// Button
// ----------------------------------------------------------------------

#[derive(Default, Clone, PartialEq, Eq)]
pub enum ButtonVariant {
    #[default]
    Default,
    Destructive,
    Success,
    Outline,
    Secondary,
    Ghost,
    Link,
    Glow,
}

#[allow(dead_code)]
#[derive(Default, Clone, PartialEq, Eq)]
pub enum ButtonSize {
    #[default]
    Default,
    Sm,
    Lg,
    Xl,
    Icon,
    IconSm,
}

impl ButtonVariant {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "bg-primary text-primary-foreground shadow-xs hover:bg-primary/90",
            Self::Destructive => "bg-destructive text-white shadow-xs hover:bg-destructive/90",
            Self::Success => "bg-success text-success-foreground hover:bg-success/90",
            Self::Outline => "border border-border bg-background/40 shadow-xs hover:bg-accent hover:text-accent-foreground",
            Self::Secondary => "bg-secondary text-secondary-foreground shadow-xs hover:bg-secondary/80",
            Self::Ghost => "hover:bg-accent hover:text-accent-foreground",
            Self::Link => "text-primary underline-offset-4 hover:underline",
            Self::Glow => "maz-gradient-bg text-white shadow-lg shadow-black/30 hover:brightness-110",
        }
    }
}

impl ButtonSize {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "h-9 px-4 py-2",
            Self::Sm => "h-8 rounded-md gap-1.5 px-3",
            Self::Lg => "h-11 rounded-lg px-6 text-[15px]",
            Self::Xl => "h-14 rounded-xl px-8 text-lg",
            Self::Icon => "size-9",
            Self::IconSm => "size-8 rounded-md",
        }
    }
}

#[component]
pub fn Button(
    #[props(into, optional)] class: Option<String>,
    #[props(default = ButtonVariant::default())] variant: ButtonVariant,
    #[props(default = ButtonSize::default())] size: ButtonSize,
    #[props(optional)] disabled: bool,
    #[props(into, optional)] title: Option<String>,
    #[props(optional)] onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let cls = format!(
        "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-all disabled:pointer-events-none disabled:opacity-50 shrink-0 outline-none focus-visible:ring-ring/50 focus-visible:ring-[3px] cursor-pointer active:scale-[0.98] select-none {} {} {}",
        variant.as_str(),
        size.as_str(),
        class.as_deref().unwrap_or("")
    );
    rsx! {
        button {
            r#type: "button",
            class: "{cls}",
            disabled,
            title: title.as_deref(),
            onclick: move |e| {
                if let Some(h) = &onclick {
                    h.call(e);
                }
            },
            {children}
        }
    }
}

// ----------------------------------------------------------------------
// Card
// ----------------------------------------------------------------------

#[component]
pub fn Card(#[props(into, optional)] class: Option<String>, children: Element) -> Element {
    let cls = format!(
        "bg-card text-card-foreground flex flex-col rounded-xl border border-border shadow-sm {}",
        class.as_deref().unwrap_or("")
    );
    rsx! { div { "data-name": "Card", class: "{cls}", {children} } }
}

#[component]
pub fn CardHeader(#[props(into, optional)] class: Option<String>, children: Element) -> Element {
    let cls = format!("flex flex-col gap-1.5 px-6 pt-6 {}", class.as_deref().unwrap_or(""));
    rsx! { div { class: "{cls}", {children} } }
}

#[component]
pub fn CardTitle(#[props(into, optional)] class: Option<String>, children: Element) -> Element {
    let cls = format!("leading-none font-semibold text-[15px] {}", class.as_deref().unwrap_or(""));
    rsx! { h2 { class: "{cls}", {children} } }
}

#[component]
pub fn CardDescription(#[props(into, optional)] class: Option<String>, children: Element) -> Element {
    let cls = format!("text-muted-foreground text-sm {}", class.as_deref().unwrap_or(""));
    rsx! { p { class: "{cls}", {children} } }
}

#[component]
pub fn CardContent(#[props(into, optional)] class: Option<String>, children: Element) -> Element {
    let cls = format!("px-6 py-5 {}", class.as_deref().unwrap_or(""));
    rsx! { div { class: "{cls}", {children} } }
}

// ----------------------------------------------------------------------
// Badge
// ----------------------------------------------------------------------

#[allow(dead_code)]
#[derive(Default, Clone, PartialEq, Eq)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Outline,
    Success,
    Warning,
    Destructive,
    Steam,
    Epic,
    Roblox,
    Custom,
    /// Weitere Stores (GOG, EA, Ubisoft …)
    Store,
}

impl BadgeVariant {
    const fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "border-transparent bg-primary text-primary-foreground",
            Self::Secondary => "border-transparent bg-secondary text-secondary-foreground",
            Self::Outline => "border-border text-foreground",
            Self::Success => "border-transparent bg-success/15 text-success",
            Self::Warning => "border-transparent bg-warning/15 text-warning",
            Self::Destructive => "border-transparent bg-destructive/15 text-destructive",
            Self::Steam => "border-transparent bg-[#1b2838] text-[#66c0f4]",
            Self::Epic => "border-transparent bg-[#2a2a2a] text-white",
            Self::Roblox => "border-transparent bg-[#e2231a] text-white",
            Self::Custom => "border-transparent bg-primary/20 text-primary",
            Self::Store => "border-transparent bg-black/70 text-white",
        }
    }
}

#[component]
pub fn Badge(
    #[props(into, optional)] class: Option<String>,
    #[props(default = BadgeVariant::default())] variant: BadgeVariant,
    children: Element,
) -> Element {
    let cls = format!(
        "inline-flex items-center gap-1 rounded-md border px-2 py-0.5 text-[11px] font-semibold tracking-wide whitespace-nowrap {} {}",
        variant.as_str(),
        class.as_deref().unwrap_or("")
    );
    rsx! { span { class: "{cls}", {children} } }
}

// ----------------------------------------------------------------------
// Input / Label
// ----------------------------------------------------------------------

#[component]
pub fn Input(
    #[props(into, optional)] class: Option<String>,
    #[props(into, default = "text".to_string())] kind: String,
    #[props(into, optional)] placeholder: Option<String>,
    #[props(into)] value: String,
    #[props(optional)] oninput: Option<EventHandler<String>>,
    #[props(optional)] onenter: Option<EventHandler<()>>,
) -> Element {
    let cls = format!(
        "flex h-10 w-full min-w-0 rounded-md border border-input bg-background/60 px-3 py-1 text-sm shadow-xs transition-[color,box-shadow] outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-ring/40 focus-visible:ring-[3px] {}",
        class.as_deref().unwrap_or("")
    );
    rsx! {
        input {
            r#type: "{kind}",
            class: "{cls}",
            placeholder: placeholder.as_deref(),
            value: "{value}",
            spellcheck: "false",
            oninput: move |e| {
                if let Some(h) = &oninput {
                    h.call(e.value());
                }
            },
            onkeydown: move |e: KeyboardEvent| {
                if e.key() == Key::Enter {
                    if let Some(h) = &onenter {
                        h.call(());
                    }
                }
            },
        }
    }
}

#[component]
pub fn Label(#[props(into, optional)] class: Option<String>, children: Element) -> Element {
    let cls = format!("flex items-center gap-2 text-sm leading-none font-medium select-none {}", class.as_deref().unwrap_or(""));
    rsx! { label { class: "{cls}", {children} } }
}

// ----------------------------------------------------------------------
// Switch (kontrolliert)
// ----------------------------------------------------------------------

#[component]
pub fn Switch(checked: bool, onchange: EventHandler<bool>, #[props(into, optional)] label: Option<String>) -> Element {
    let state = if checked { "checked" } else { "unchecked" };
    rsx! {
        button {
            r#type: "button",
            role: "switch",
            "aria-checked": "{checked}",
            "aria-label": label.as_deref(),
            "data-state": "{state}",
            class: "inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring data-[state=checked]:bg-primary data-[state=unchecked]:bg-input",
            onclick: move |_| onchange.call(!checked),
            span {
                "data-state": "{state}",
                class: "block rounded-full shadow-lg transition-transform pointer-events-none size-5 bg-white data-[state=checked]:translate-x-5 data-[state=unchecked]:translate-x-0",
            }
        }
    }
}

/// Zeile mit Titel, Beschreibung und Schalter
#[component]
pub fn SwitchRow(
    #[props(into)] title: String,
    #[props(into, default)] description: String,
    checked: bool,
    onchange: EventHandler<bool>,
) -> Element {
    rsx! {
        div { class: "flex items-center justify-between gap-6 py-3",
            div { class: "min-w-0",
                div { class: "text-sm font-medium", "{title}" }
                if !description.is_empty() {
                    div { class: "text-xs text-muted-foreground mt-1 leading-relaxed", "{description}" }
                }
            }
            Switch { checked, onchange, label: title.clone() }
        }
    }
}

#[component]
pub fn Separator(#[props(into, optional)] class: Option<String>) -> Element {
    let cls = format!("shrink-0 bg-border h-px w-full {}", class.as_deref().unwrap_or(""));
    rsx! { div { role: "separator", class: "{cls}" } }
}

// ----------------------------------------------------------------------
// Dialog (über ein Signal gesteuert, ohne JavaScript)
// ----------------------------------------------------------------------

#[component]
pub fn Dialog(
    on_close: EventHandler<()>,
    #[props(into, optional)] class: Option<String>,
    #[props(default = true)] close_on_backdrop: bool,
    children: Element,
) -> Element {
    let cls = format!(
        "relative bg-popover text-popover-foreground border border-border rounded-2xl shadow-2xl shadow-black/50 w-full max-h-[88vh] overflow-y-auto animate-pop-in {}",
        class.as_deref().unwrap_or("max-w-lg")
    );
    rsx! {
        div { class: "fixed inset-0 z-50 grid place-items-center p-6",
            div {
                class: "absolute inset-0 bg-black/60 backdrop-blur-sm animate-fade-in",
                onclick: move |_| {
                    if close_on_backdrop {
                        on_close.call(());
                    }
                },
            }
            div { class: "{cls}", "data-dialog": "1",
                onkeydown: move |e| {
                    if e.key() == Key::Escape {
                        on_close.call(());
                    }
                },
                button {
                    r#type: "button",
                    "data-dialog-close": "1",
                    class: "absolute top-4 right-4 z-10 p-1.5 rounded-md text-muted-foreground hover:bg-accent hover:text-foreground cursor-pointer",
                    title: crate::i18n::t("common.close"),
                    onclick: move |_| on_close.call(()),
                    Icon { name: "x", class: "size-4" }
                }
                {children}
            }
        }
    }
}

// ----------------------------------------------------------------------
// Kleinkram
// ----------------------------------------------------------------------

#[component]
pub fn Spinner(#[props(into, default)] class: String) -> Element {
    let cls = if class.is_empty() { "size-4".to_string() } else { class };
    rsx! { Icon { name: "loader", class: "animate-spin {cls}" } }
}

/// Segmentierte Auswahl (z. B. Kartengröße)
#[component]
pub fn Segmented(options: Vec<(String, String)>, value: String, onchange: EventHandler<String>) -> Element {
    rsx! {
        div { class: "inline-flex rounded-lg bg-muted p-1 gap-1",
            {options.into_iter().map(|(val, label)| {
                let active = val == value;
                let cls = if active {
                    "px-3 h-8 rounded-md text-sm font-medium bg-background text-foreground shadow cursor-pointer"
                } else {
                    "px-3 h-8 rounded-md text-sm font-medium text-muted-foreground hover:text-foreground cursor-pointer"
                };
                let v = val.clone();
                rsx! {
                    button { key: "{val}", r#type: "button", class: "{cls}", onclick: move |_| onchange.call(v.clone()), "{label}" }
                }
            })}
        }
    }
}
