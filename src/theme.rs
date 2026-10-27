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
    ("oled", ["#000000", "#0b0b0b", "#000000", "#f5f5f5", "#8f8f8f", "#ffffff", "#d4d4d4", "#1f1f1f"]),
    ("light", ["#f4f5fa", "#ffffff", "#ffffff", "#14142b", "#34364a", "#6d28d9", "#0ea5e9", "#dfe1ee"]),
    ("mint", ["#f3fbf7", "#ffffff", "#ffffff", "#0f2a1f", "#2f4a3e", "#10b981", "#06b6d4", "#d6ebe1"]),
    // Pixel-Designs: kantige Ecken (wie Blöcke) und ein Pixel-Streifen in den Designfarben
    ("pixel_violet", ["#11001d", "#1d0331", "#160027", "#f7edff", "#c8a9e0", "#9a00dc", "#d07bff", "#3b0b5e"]),
    ("pixel_flame", ["#0c0505", "#1a0a07", "#130705", "#fff2e8", "#d6a592", "#ff3b00", "#ffc400", "#401510"]),
    ("pixel_rainbow", ["#090909", "#151515", "#0f0f0f", "#f3f3f3", "#a9a9a9", "#00d9ff", "#ff9100", "#2b2b2b"]),
    ("pixel_jungle", ["#04140f", "#0b241a", "#071b13", "#ecfff5", "#9ccbb3", "#35d65a", "#ffd21f", "#174633"]),
];

/// Pixel-Streifen für die Pixel-Designs – harte Farbübergänge wie Pixel
const SKIN_STRIPES: &[(&str, &[&str])] = &[
    ("pixel_violet", &["#8b00c9", "#1a0030", "#8b00c9", "#b44dff", "#8b00c9", "#1a0030", "#8b00c9", "#b44dff"]),
    ("pixel_flame", &["#2a0a05", "#ff1a00", "#ff5a00", "#ff9a00", "#ffd000", "#ff9a00", "#ff5a00", "#ff1a00", "#2a0a05"]),
    ("pixel_rainbow", &["#ff1f1f", "#ff8a00", "#ffd400", "#a6e22e", "#22c55e", "#00e0c6", "#00b7ff", "#1d1d1d"]),
    ("pixel_jungle", &["#2e9e47", "#35d65a", "#ffd21f", "#ff8a1a", "#ff5a1f", "#22d3ee", "#7ee0ff", "#2e9e47"]),
];

/// Frühere Kennungen (2.1.0) → neue, neutrale Kennungen
pub fn renamed(id: &str) -> Option<&'static str> {
    match id {
        "wemmbu" => Some("pixel_violet"),
        "flamefrags" => Some("pixel_flame"),
        "spoke" => Some("pixel_rainbow"),
        "parrot" => Some("pixel_jungle"),
        _ => None,
    }
}

/// CSS für den Pixel-Streifen eines Designs (nur Pixel-Designs)
pub fn stripe_css(id: &str) -> Option<String> {
    let colors = SKIN_STRIPES.iter().find(|(k, _)| *k == id)?.1;
    let n = colors.len() as f32;
    let stops: Vec<String> = colors
        .iter()
        .enumerate()
        .map(|(i, c)| format!("{c} {:.2}% {:.2}%", i as f32 / n * 100.0, (i + 1) as f32 / n * 100.0))
        .collect();
    Some(format!("linear-gradient(90deg,{})", stops.join(",")))
}

/// Pixel-Designs sind kantig
fn preset_radius(id: &str) -> u8 {
    if SKIN_STRIPES.iter().any(|(k, _)| *k == id) {
        3
    } else {
        12
    }
}

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
        radius: preset_radius(p),
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

/// Relative Helligkeit nach WCAG
fn rel_lum(c: &str) -> f32 {
    let Some((r, g, b)) = parse_hex(c) else { return 0.0 };
    let ch = |v: u8| {
        let v = v as f32 / 255.0;
        if v <= 0.03928 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * ch(r) + 0.7152 * ch(g) + 0.0722 * ch(b)
}

/// Kontrast-Verhältnis (1 … 21) zwischen zwei Farben
pub fn contrast(a: &str, b: &str) -> f32 {
    let (x, y) = (rel_lum(a), rel_lum(b));
    let (hi, lo) = if x > y { (x, y) } else { (y, x) };
    (hi + 0.05) / (lo + 0.05)
}

/// Schriftfarbe so lange Richtung `strong` schieben, bis sie auf `bg` gut lesbar ist.
/// Helle Hintergründe brauchen mehr Kontrast (graue Schrift auf Weiß ist schwer zu lesen).
pub fn readable(fg: &str, bg: &str, strong: &str) -> String {
    let min = if is_light(bg) { 7.0 } else { 4.5 };
    let mut c = fg.to_string();
    for i in 1..=10 {
        if contrast(&c, bg) >= min {
            return c;
        }
        c = mix(fg, strong, i as f32 / 10.0);
    }
    c
}

/// Schrift auf einem Farbverlauf (z. B. „Installieren“-Knopf): Schwarz oder Weiß – je nachdem,
/// was an *beiden* Enden des Verlaufs besser lesbar ist. Nie Weiß auf Hellgrau.
pub fn gradient_text(a: &str, b: &str) -> &'static str {
    let w = contrast("#ffffff", a).min(contrast("#ffffff", b));
    let k = contrast("#0b0b10", a).min(contrast("#0b0b10", b));
    if w >= k {
        "#ffffff"
    } else {
        "#0b0b10"
    }
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
        let sidenav_fg = readable(&mix(&t.text, &t.sidebar, 0.15), &t.sidebar, &t.text);
        format!(
            "--background:{bg};--foreground:{fg};--card:{card};--card-foreground:{fg};--popover:{pop};--popover-foreground:{fg};\
--primary:{acc};--primary-foreground:{accfg};--secondary:{sec};--secondary-foreground:{fg};--muted:{mut};--muted-foreground:{mfg};\
--accent:{abg};--accent-foreground:{fg};--border:{border};--input:{input};--ring:{acc};--sidenav:{side};--sidenav-foreground:{sfg};\
--sidenav-accent:{sacc};--glow:{acc};--glow-2:{acc2};--glow-fg:{gfg};--radius:{r}px;",
            bg = t.bg,
            fg = t.text,
            card = t.card,
            pop = popover,
            acc = t.accent,
            accfg = contrast_text(&t.accent),
            sec = secondary,
            mut = muted_bg,
            mfg = readable(&t.muted, &t.bg, &t.text),
            abg = accent_bg,
            border = t.border,
            input = input,
            side = t.sidebar,
            sfg = sidenav_fg,
            sacc = sidenav_accent,
            acc2 = t.accent2,
            gfg = gradient_text(&t.accent, &t.accent2),
            r = t.radius,
        )
    }

    /// Design als teilbarer Code (eine Zeile)
    pub fn to_code(&self) -> String {
        let mut t = self.sanitized();
        t.id = String::new();
        format!("OGTHEME:{}", serde_json::to_string(&t).unwrap_or_default())
    }

    pub fn from_code(code: &str) -> Option<CustomTheme> {
        // Alte Codes (bis 1.7: „MAZTHEME:“) funktionieren weiter
        let c = code.trim();
        let json = c.strip_prefix("OGTHEME:").or_else(|| c.strip_prefix("MAZTHEME:")).unwrap_or(c);
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
        // Grau auf Weiß wird automatisch dunkler gemacht
        let fixed = readable("#9a9a9a", "#ffffff", "#111111");
        assert!(contrast(&fixed, "#ffffff") >= 7.0, "{fixed}");
        assert!(contrast("#000000", "#ffffff") > 20.0);
        // Weiß auf Hellgrau wird nie gewählt
        assert_eq!(gradient_text("#ffffff", "#d4d4d4"), "#0b0b10");
        assert_eq!(gradient_text("#1e1b4b", "#312e81"), "#ffffff");
        // Alle mitgelieferten Designs: Nebentext gut lesbar
        for (id, c) in PRESETS.iter() {
            let need = if is_light(c[0]) { 7.0 } else { 4.5 };
            assert!(contrast(c[4], c[0]) >= need, "{id}: {} auf {}", c[4], c[0]);
        }
        assert!(!is_light("#0a0a12"));
        assert_eq!(contrast_text("#ffffff"), "#0b0b10");
    }

    #[test]
    fn presets_and_codes() {
        for (id, _) in PRESETS {
            let t = preset(id).unwrap();
            assert!(t.css_vars().contains("--background:"));
        }
        // Pixel-Designs: gut lesbar, kantig, mit Pixel-Streifen
        for id in ["pixel_violet", "pixel_flame", "pixel_rainbow", "pixel_jungle"] {
            let t = preset(id).unwrap();
            assert!(contrast(&t.text, &t.bg) >= 7.0, "{id}");
            assert!(contrast(&t.text, &t.card) >= 7.0, "{id}");
            assert_eq!(t.radius, 3);
            assert!(stripe_css(id).unwrap().starts_with("linear-gradient(90deg,"));
        }
        assert!(stripe_css("neon").is_none());
        assert_eq!(renamed("wemmbu"), Some("pixel_violet"));
        assert!(preset(renamed("parrot").unwrap()).is_some());
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
