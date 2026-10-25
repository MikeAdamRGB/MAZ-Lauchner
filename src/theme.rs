//! Farbschemata: fertige Designs und eigene Designs aus dem Design-Editor.
//!
//! Jedes Design besteht aus 8 Grundfarben. Alle weiteren Farben (Hover,
//! Eingabefelder, Rahmen …) werden daraus berechnet – deshalb sieht jede
//! Kombination stimmig aus.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CustomTheme {
    pub id: String,
    pub name: String,
    pub bg: String,
    pub card: String,
    pub sidebar: String,
    pub text: String,
    pub muted: String,
    pub accent: String,
    pub accent2: String,
    pub border: String,
    /// Eckenrundung in Pixeln (0–24)
    pub radius: u8,
    /// Schrift: "standard", "rund", "modern", "mono", "serif"
    pub font: String,
    /// Abstände: 1 = kompakt, 2 = normal, 3 = großzügig
    pub density: u8,
    /// Ansicht, die das Design mitbringt ("" = nicht ändern, "maz", "steam", "epic")
    pub layout: String,
}

/// Erlaubte Schriften (Name → CSS). Nur System-Schriften, nichts wird geladen.
pub const FONTS: &[(&str, &str)] = &[
    ("standard", "\"Segoe UI Variable Text\", \"Segoe UI\", Inter, system-ui, sans-serif"),
    ("rund", "\"Nunito\", \"Varela Round\", \"Segoe UI\", \"Ubuntu\", system-ui, sans-serif"),
    ("modern", "Inter, \"SF Pro Text\", \"Helvetica Neue\", Arial, system-ui, sans-serif"),
    ("mono", "\"Cascadia Code\", \"JetBrains Mono\", \"SF Mono\", Consolas, \"DejaVu Sans Mono\", monospace"),
    ("serif", "Georgia, \"Iowan Old Style\", \"Times New Roman\", serif"),
];

pub fn font_css(key: &str) -> Option<&'static str> {
    FONTS.iter().find(|(k, _)| *k == key && *k != "standard").map(|(_, css)| *css)
}

/// Zoom-Faktor für die Abstände
pub fn density_zoom(d: u8) -> Option<f32> {
    match d {
        1 => Some(0.9),
        3 => Some(1.1),
        _ => None,
    }
}

impl Default for CustomTheme {
    fn default() -> Self {
        preset("maz").unwrap_or_else(|| CustomTheme {
            id: String::new(),
            name: String::new(),
            bg: "#0a0a12".into(),
            card: "#13131f".into(),
            sidebar: "#0e0e18".into(),
            text: "#ecebf5".into(),
            muted: "#9895b0".into(),
            accent: "#8b5cf6".into(),
            accent2: "#22d3ee".into(),
            border: "#26263b".into(),
            radius: 12,
            font: String::new(),
            density: 2,
            layout: String::new(),
        })
    }
}

/// Fertige Designs: (id, Hintergrund, Karten, Seitenleiste, Text, Text gedimmt, Akzent, Akzent 2, Rahmen)
pub const PRESETS: &[(&str, [&str; 8])] = &[
    ("maz", ["#0a0a12", "#13131f", "#0e0e18", "#ecebf5", "#9895b0", "#8b5cf6", "#22d3ee", "#26263b"]),
    ("midnight", ["#070d1a", "#0f1828", "#0a1222", "#e6edf7", "#8b9bb6", "#3b82f6", "#06b6d4", "#1d2b45"]),
    ("neon", ["#0b0613", "#150b22", "#0f0819", "#fbe9ff", "#b18fc4", "#ff2bd6", "#00f0ff", "#33184f"]),
    ("synthwave", ["#12061f", "#1d0b31", "#170827", "#ffe9fb", "#b48ccf", "#f72585", "#ffb703", "#3a1a5c"]),
    ("ocean", ["#04121a", "#0a1f2b", "#061823", "#e2f4fb", "#86a9b8", "#0ea5e9", "#2dd4bf", "#143444"]),
    ("forest", ["#07110c", "#0e1c15", "#0a160f", "#e7f5ec", "#8bb09a", "#22c55e", "#a3e635", "#1b3527"]),
    ("ember", ["#120a07", "#1d120c", "#160d08", "#fbeee6", "#bb9a85", "#f97316", "#facc15", "#3a2416"]),
    ("crimson", ["#120607", "#1f0b0d", "#18080a", "#fbe9ea", "#b98a8e", "#e11d48", "#fb7185", "#3d1519"]),
    ("gold", ["#0f0d07", "#1a160c", "#13100a", "#f7f1e1", "#b3a684", "#eab308", "#f59e0b", "#3a321b"]),
    ("sakura", ["#140a10", "#1f1019", "#180c13", "#fdeef5", "#c497ad", "#f472b6", "#c084fc", "#3b1f2f"]),
    ("dracula", ["#1e1f29", "#282a36", "#21222c", "#f8f8f2", "#9ea3c2", "#bd93f9", "#ff79c6", "#44475a"]),
    ("nord", ["#2e3440", "#3b4252", "#2b303b", "#eceff4", "#a3acbd", "#88c0d0", "#b48ead", "#4c566a"]),
    ("coffee", ["#16110d", "#221a14", "#1b1410", "#f3e9df", "#b09a86", "#c08457", "#e0b084", "#3d2f24"]),
    ("oled", ["#000000", "#0b0b0b", "#000000", "#f5f5f5", "#8f8f8f", "#f5f5f5", "#8f8f8f", "#1f1f1f"]),
    ("light", ["#f4f5fa", "#ffffff", "#ffffff", "#14142b", "#62647d", "#6d28d9", "#0ea5e9", "#dfe1ee"]),
    ("mint", ["#f3fbf7", "#ffffff", "#ffffff", "#0f2a1f", "#5b7a6c", "#10b981", "#06b6d4", "#d6ebe1"]),
];

pub fn preset(id: &str) -> Option<CustomTheme> {
    PRESETS.iter().find(|(p, _)| *p == id).map(|(p, c)| CustomTheme {
        id: p.to_string(),
        name: String::new(),
        bg: c[0].into(),
        card: c[1].into(),
        sidebar: c[2].into(),
        text: c[3].into(),
        muted: c[4].into(),
        accent: c[5].into(),
        accent2: c[6].into(),
        border: c[7].into(),
        radius: 12,
        font: String::new(),
        density: 2,
        layout: String::new(),
    })
}

pub fn parse_hex(s: &str) -> Option<(u8, u8, u8)> {
    let h = s.trim().trim_start_matches('#');
    let h = if h.len() == 3 {
        h.chars().flat_map(|c| [c, c]).collect::<String>()
    } else {
        h.to_string()
    };
    if h.len() != 6 {
        return None;
    }
    let v = u32::from_str_radix(&h, 16).ok()?;
    Some(((v >> 16) as u8, (v >> 8) as u8, v as u8))
}

pub fn to_hex(c: (u8, u8, u8)) -> String {
    format!("#{:02x}{:02x}{:02x}", c.0, c.1, c.2)
}

/// Mischt Farbe `a` mit `b` (t = Anteil von b, 0.0–1.0)
pub fn mix(a: &str, b: &str, t: f32) -> String {
    let (Some(x), Some(y)) = (parse_hex(a), parse_hex(b)) else {
        return a.to_string();
    };
    let m = |p: u8, q: u8| (p as f32 + (q as f32 - p as f32) * t).round().clamp(0.0, 255.0) as u8;
    to_hex((m(x.0, y.0), m(x.1, y.1), m(x.2, y.2)))
}

fn luminance(c: &str) -> f32 {
    let Some((r, g, b)) = parse_hex(c) else { return 0.0 };
    (0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32) / 255.0
}

pub fn is_light(c: &str) -> bool {
    luminance(c) > 0.55
}

/// Lesbare Schriftfarbe auf einer Hintergrundfarbe
pub fn contrast_text(bg: &str) -> &'static str {
    if luminance(bg) > 0.6 {
        "#0b0b10"
    } else {
        "#ffffff"
    }
}

fn valid(c: &str, fallback: &str) -> String {
    if parse_hex(c).is_some() {
        c.trim().to_string()
    } else {
        fallback.to_string()
    }
}

impl CustomTheme {
    /// Bereinigt ungültige Farben (z. B. aus importierten Codes)
    pub fn sanitized(&self) -> CustomTheme {
        let d = preset("maz").unwrap_or_default();
        CustomTheme {
            id: self.id.clone(),
            name: self.name.chars().take(40).collect(),
            bg: valid(&self.bg, &d.bg),
            card: valid(&self.card, &d.card),
            sidebar: valid(&self.sidebar, &d.sidebar),
            text: valid(&self.text, &d.text),
            muted: valid(&self.muted, &d.muted),
            accent: valid(&self.accent, &d.accent),
            accent2: valid(&self.accent2, &d.accent2),
            border: valid(&self.border, &d.border),
            radius: self.radius.min(24),
            font: if FONTS.iter().any(|(k, _)| *k == self.font) { self.font.clone() } else { String::new() },
            density: if (1..=3).contains(&self.density) { self.density } else { 2 },
            layout: if ["maz", "steam", "epic"].contains(&self.layout.as_str()) { self.layout.clone() } else { String::new() },
        }
    }

    /// Zusätzliche Stile (Schrift, Abstände) für das Hauptfenster
    pub fn extra_style(&self) -> String {
        let t = self.sanitized();
        let mut out = String::new();
        if let Some(f) = font_css(&t.font) {
            out.push_str(&format!("--font-sans:{f};--font-display:{f};font-family:{f};"));
        }
        if let Some(z) = density_zoom(t.density) {
            out.push_str(&format!("zoom:{z};"));
        }
        out
    }

    /// Design als Datei (.maztheme.json) zum Teilen
    pub fn to_file(&self) -> String {
        let mut t = self.sanitized();
        t.id = String::new();
        serde_json::to_string_pretty(&ThemeFile { format: "maz-theme".into(), version: 1, theme: t }).unwrap_or_default()
    }

    pub fn from_file(text: &str) -> Option<CustomTheme> {
        if let Ok(f) = serde_json::from_str::<ThemeFile>(text) {
            if f.format == "maz-theme" {
                return Some(f.theme.sanitized());
            }
        }
        CustomTheme::from_code(text)
    }

    /// Alle CSS-Variablen für dieses Design
    pub fn css_vars(&self) -> String {
        let t = self.sanitized();
        let light = is_light(&t.bg);
        let popover = mix(&t.card, &t.text, if light { 0.0 } else { 0.03 });
        let secondary = mix(&t.card, &t.text, 0.08);
        let muted_bg = mix(&t.bg, &t.text, 0.05);
        let accent_bg = mix(&t.card, &t.text, 0.11);
        let input = mix(&t.border, &t.text, 0.08);
        let sidenav_accent = mix(&t.sidebar, &t.text, 0.09);
        let sidenav_fg = mix(&t.text, &t.sidebar, 0.15);
        format!(
            "--background:{bg};--foreground:{fg};--card:{card};--card-foreground:{fg};--popover:{pop};--popover-foreground:{fg};\
--primary:{acc};--primary-foreground:{accfg};--secondary:{sec};--secondary-foreground:{fg};--muted:{mut};--muted-foreground:{mfg};\
--accent:{abg};--accent-foreground:{fg};--border:{border};--input:{input};--ring:{acc};--sidenav:{side};--sidenav-foreground:{sfg};\
--sidenav-accent:{sacc};--glow:{acc};--glow-2:{acc2};--radius:{r}px;",
            bg = t.bg,
            fg = t.text,
            card = t.card,
            pop = popover,
            acc = t.accent,
            accfg = contrast_text(&t.accent),
            sec = secondary,
            mut = muted_bg,
            mfg = t.muted,
            abg = accent_bg,
            border = t.border,
            input = input,
            side = t.sidebar,
            sfg = sidenav_fg,
            sacc = sidenav_accent,
            acc2 = t.accent2,
            r = t.radius,
        )
    }

    /// Design als teilbarer Code (eine Zeile)
    pub fn to_code(&self) -> String {
        let mut t = self.sanitized();
        t.id = String::new();
        format!("MAZTHEME:{}", serde_json::to_string(&t).unwrap_or_default())
    }

    pub fn from_code(code: &str) -> Option<CustomTheme> {
        let json = code.trim().strip_prefix("MAZTHEME:").unwrap_or(code.trim());
        let t: CustomTheme = serde_json::from_str(json).ok()?;
        Some(t.sanitized())
    }
}

/// Aufbau einer Design-Datei
#[derive(Serialize, Deserialize)]
struct ThemeFile {
    format: String,
    version: u32,
    theme: CustomTheme,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors() {
        assert_eq!(parse_hex("#fff"), Some((255, 255, 255)));
        assert_eq!(mix("#000000", "#ffffff", 0.5), "#808080");
        assert!(is_light("#f4f5fa"));
        assert!(!is_light("#0a0a12"));
        assert_eq!(contrast_text("#ffffff"), "#0b0b10");
    }

    #[test]
    fn presets_and_codes() {
        for (id, _) in PRESETS {
            let t = preset(id).unwrap();
            assert!(t.css_vars().contains("--background:"));
        }
        let mut t = preset("neon").unwrap();
        t.name = "Mein Design".into();
        let code = t.to_code();
        let back = CustomTheme::from_code(&code).unwrap();
        assert_eq!(back.accent, t.accent);
        assert_eq!(back.name, "Mein Design");
        let broken = CustomTheme::from_code(r#"{"bg":"lila","radius":99}"#).unwrap();
        assert_eq!(broken.bg, "#0a0a12");
        assert_eq!(broken.radius, 24);
        assert!(CustomTheme::from_code("Quatsch").is_none());
        // Datei hin und zurück, ungültige Schrift wird verworfen
        let mut t = preset("ocean").unwrap();
        t.font = "mono".into();
        t.density = 1;
        let back = CustomTheme::from_file(&t.to_file()).unwrap();
        assert_eq!(back.font, "mono");
        assert_eq!(back.density, 1);
        assert!(back.extra_style().contains("zoom:0.9"));
        let bad = CustomTheme::from_file(r#"{"format":"maz-theme","version":1,"theme":{"font":"url(evil)","density":9}}"#).unwrap();
        assert_eq!(bad.font, "");
        assert_eq!(bad.density, 2);
    }
}
