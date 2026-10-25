//! Einfache Markdown-Auswertung für Release-Notizen (Überschriften, Listen, **fett**, `Code`).
//! Links werden nur als Text gezeigt – im Update-Fenster wird nichts angeklickt oder geladen.

#[derive(Clone, Debug, PartialEq)]
pub enum Span {
    Text(String),
    Bold(String),
    Code(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Block {
    Heading(u8, String),
    Bullet(Vec<Span>),
    Para(Vec<Span>),
}

/// "[Text](https://…)" → "Text", "<br>" weg
fn strip_links(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(start) = rest.find('[') {
        let after = &rest[start + 1..];
        match (after.find("]("), after.find(']')) {
            (Some(mid), Some(close)) if mid == close => {
                let tail = &after[mid + 2..];
                if let Some(end) = tail.find(')') {
                    out.push_str(&rest[..start]);
                    out.push_str(&after[..mid]);
                    rest = &tail[end + 1..];
                    continue;
                }
                out.push_str(&rest[..=start]);
                rest = after;
            }
            _ => {
                out.push_str(&rest[..=start]);
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out.replace("<br>", " ")
}

pub fn inline(s: &str) -> Vec<Span> {
    let s = strip_links(s);
    let mut out = Vec::new();
    let mut buf = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '*' && chars.get(i + 1) == Some(&'*') {
            if let Some(end) = (i + 2..chars.len().saturating_sub(1)).find(|&j| chars[j] == '*' && chars[j + 1] == '*') {
                if !buf.is_empty() {
                    out.push(Span::Text(std::mem::take(&mut buf)));
                }
                out.push(Span::Bold(chars[i + 2..end].iter().collect()));
                i = end + 2;
                continue;
            }
        }
        if chars[i] == '`' {
            if let Some(end) = (i + 1..chars.len()).find(|&j| chars[j] == '`') {
                if !buf.is_empty() {
                    out.push(Span::Text(std::mem::take(&mut buf)));
                }
                out.push(Span::Code(chars[i + 1..end].iter().collect()));
                i = end + 1;
                continue;
            }
        }
        buf.push(chars[i]);
        i += 1;
    }
    if !buf.is_empty() {
        out.push(Span::Text(buf));
    }
    out
}

pub fn parse(md: &str) -> Vec<Block> {
    let mut out = Vec::new();
    let mut para = String::new();
    let flush = |para: &mut String, out: &mut Vec<Block>| {
        if !para.trim().is_empty() {
            out.push(Block::Para(inline(para.trim())));
        }
        para.clear();
    };
    for line in md.lines() {
        let l = line.trim();
        if l.is_empty() || l.starts_with("---") || l.starts_with("<!--") {
            flush(&mut para, &mut out);
            continue;
        }
        let hashes = l.chars().take_while(|c| *c == '#').count();
        if hashes > 0 && l.chars().nth(hashes) == Some(' ') {
            flush(&mut para, &mut out);
            out.push(Block::Heading(hashes.min(6) as u8, strip_links(l[hashes..].trim())));
        } else if let Some(item) = l.strip_prefix("- ").or_else(|| l.strip_prefix("* ")).or_else(|| l.strip_prefix("• ")) {
            flush(&mut para, &mut out);
            out.push(Block::Bullet(inline(item.trim())));
        } else {
            if !para.is_empty() {
                para.push(' ');
            }
            para.push_str(l);
        }
    }
    flush(&mut para, &mut out);
    out
}

/// Art eines Abschnitts (für Farbe/Icon): "Neu", "Verbessert", "Behoben", "Sicherheit"
pub fn section_kind(title: &str) -> &'static str {
    let t = title.to_lowercase();
    if ["neu", "new", "nouveau", "nuevo", "novo", "nuovo", "nowe", "yeni", "нов"].iter().any(|k| t.contains(k)) {
        "new"
    } else if ["verbessert", "improv", "amélior", "mejor", "melhor", "miglior", "ulepsz", "iyileş", "улучш"].iter().any(|k| t.contains(k)) {
        "improved"
    } else if ["behoben", "fix", "corrig", "correg", "corri", "poprawk", "düzelt", "исправ"].iter().any(|k| t.contains(k)) {
        "fixed"
    } else if ["sicher", "secur", "sécur", "segur", "bezpiecz", "güvenl", "безопас"].iter().any(|k| t.contains(k)) {
        "security"
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown() {
        let md = "## MAZ Launcher 1.5.0\n\n### Neu\n- **GOG** & EA werden erkannt\n- Siehe [Seite](https://x.y)\n\nText über\nzwei Zeilen `code`\n---\n";
        let b = parse(md);
        assert_eq!(b[0], Block::Heading(2, "MAZ Launcher 1.5.0".into()));
        assert_eq!(b[1], Block::Heading(3, "Neu".into()));
        assert_eq!(b[2], Block::Bullet(vec![Span::Bold("GOG".into()), Span::Text(" & EA werden erkannt".into())]));
        assert_eq!(b[3], Block::Bullet(vec![Span::Text("Siehe Seite".into())]));
        assert_eq!(b[4], Block::Para(vec![Span::Text("Text über zwei Zeilen ".into()), Span::Code("code".into())]));
        assert_eq!(b.len(), 5);
        assert_eq!(section_kind("Behoben"), "fixed");
        assert_eq!(section_kind("✨ Neu"), "new");
        assert_eq!(inline("a ** b"), vec![Span::Text("a ** b".into())]);
    }
}
