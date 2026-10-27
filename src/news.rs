//! Spiel-News und Patch-Notes von Steam (öffentliche Schnittstelle, ohne Schlüssel).
//!
//! SICHERHEIT: Die Texte kommen aus dem Internet. Sie werden hier in reinen Text umgewandelt
//! (HTML und BBCode entfernt) und in der Oberfläche nur als Text angezeigt – nie als HTML.
//! Links werden nur übernommen, wenn sie mit https:// beginnen.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Debug, PartialEq)]
pub struct NewsItem {
    pub title: String,
    pub url: Option<String>,
    pub date: u64,
    pub feed: String,
    pub patch: bool,
    pub text: String,
}

const MAX_TEXT: usize = 1500;

/// HTML-Zeichen wie &amp; zurückwandeln (nur die üblichen)
fn entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('&') {
        out.push_str(&rest[..i]);
        let tail = &rest[i..];
        let end = tail.find(';').filter(|e| *e <= 10);
        let Some(e) = end else {
            out.push('&');
            rest = &tail[1..];
            continue;
        };
        let ent = &tail[1..e];
        let rep = match ent {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" | "#39" => Some('\''),
            "nbsp" => Some(' '),
            _ => ent
                .strip_prefix("#x")
                .and_then(|h| u32::from_str_radix(h, 16).ok())
                .or_else(|| ent.strip_prefix('#').and_then(|d| d.parse().ok()))
                .and_then(char::from_u32)
                .filter(|c| !c.is_control() || *c == '\n'),
        };
        match rep {
            Some(c) => {
                out.push(c);
                rest = &tail[e + 1..];
            }
            None => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// HTML/BBCode → lesbarer Text (Absätze und Listenpunkte bleiben erhalten)
pub fn to_plain(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == '<' || c == '[' {
            let close = if c == '<' { '>' } else { ']' };
            // Tag nur, wenn er kurz ist und geschlossen wird
            if let Some(len) = input[i + 1..].find(close).filter(|l| *l <= 200) {
                let raw = &input[i + 1..i + 1 + len];
                let tag = raw.to_ascii_lowercase();
                let body = tag.strip_prefix('/').unwrap_or(&tag);
                let name: String = body.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '*').collect();
                // Ein Tag beginnt direkt mit dem Namen („<p>“, „[/list]“) – „a < b“ ist kein Tag
                let after = body[name.len()..].chars().next();
                let shaped = !name.is_empty() && matches!(after, None | Some(' ' | '=' | '/' | '\t'));
                let known = shaped && matches!(
                    name.as_str(),
                    "p" | "br" | "b" | "i" | "u" | "s" | "h1" | "h2" | "h3" | "h4" | "h5" | "list" | "olist" | "ul" | "ol" | "li" | "*" | "url" | "a"
                        | "img" | "strong" | "em" | "strike" | "spoiler" | "noparse" | "hr" | "quote" | "code" | "table" | "tr" | "td" | "th"
                        | "div" | "span" | "previewyoutube" | "video" | "expand" | "emoticon"
                );
                if known {
                    match name.as_str() {
                        "p" | "br" | "h1" | "h2" | "h3" | "h4" | "h5" | "div" | "hr" | "tr" | "quote" => out.push('\n'),
                        "li" | "*" if !tag.starts_with('/') => out.push_str("\n• "),
                        _ => {}
                    }
                    // Bis zum Ende des Tags springen
                    let stop = i + 1 + len;
                    while let Some((j, _)) = chars.peek() {
                        if *j > stop {
                            break;
                        }
                        chars.next();
                    }
                    continue;
                }
            }
        }
        out.push(c);
    }
    let text = entities(&out).replace("{STEAM_CLAN_IMAGE}", "");
    // Leerraum aufräumen: höchstens eine Leerzeile, keine Steuerzeichen
    let mut lines: Vec<String> = Vec::new();
    for l in text.lines() {
        let clean: String = l.chars().filter(|c| !c.is_control()).collect::<String>().split_whitespace().collect::<Vec<_>>().join(" ");
        if clean.is_empty() && lines.last().map(|x| x.is_empty()).unwrap_or(true) {
            continue;
        }
        lines.push(clean);
    }
    while lines.last().map(|l| l.is_empty()).unwrap_or(false) {
        lines.pop();
    }
    let joined = lines.join("\n");
    if joined.chars().count() > MAX_TEXT {
        let cut: String = joined.chars().take(MAX_TEXT).collect();
        format!("{}…", cut.trim_end())
    } else {
        joined
    }
}

fn clean_line(s: &str, max: usize) -> String {
    let t: String = s.chars().filter(|c| !c.is_control()).take(max).collect();
    t.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn safe_url(u: &str) -> Option<String> {
    let u = u.trim();
    let ok = u.starts_with("https://") && u.len() < 2000 && !u.chars().any(|c| c.is_whitespace() || c.is_control() || c == '"' || c == '<' || c == '>');
    ok.then(|| u.to_string())
}

fn is_patch(title: &str, tags: &[String]) -> bool {
    if tags.iter().any(|t| t.eq_ignore_ascii_case("patchnotes")) {
        return true;
    }
    let t = title.to_lowercase();
    ["patch", "hotfix", "update", "changelog", "release notes", "bugfix"].iter().any(|w| t.contains(w))
}

/// Antwort von GetNewsForApp auswerten
pub fn parse(json: &str) -> Vec<NewsItem> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(json) else { return Vec::new() };
    let Some(items) = v.pointer("/appnews/newsitems").and_then(|x| x.as_array()) else { return Vec::new() };
    items
        .iter()
        .take(30)
        .filter_map(|it| {
            let title = clean_line(it.get("title")?.as_str()?, 200);
            if title.is_empty() {
                return None;
            }
            let tags: Vec<String> = it.get("tags").and_then(|t| t.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
            Some(NewsItem {
                patch: is_patch(&title, &tags),
                url: it.get("url").and_then(|u| u.as_str()).and_then(safe_url),
                date: it.get("date").and_then(|d| d.as_u64()).unwrap_or(0),
                feed: clean_line(it.get("feedlabel").and_then(|f| f.as_str()).unwrap_or(""), 60),
                text: to_plain(it.get("contents").and_then(|c| c.as_str()).unwrap_or("")),
                title,
            })
        })
        .collect()
}

type Cache = Mutex<HashMap<u32, (u64, Vec<NewsItem>)>>;

fn cache() -> &'static Cache {
    static C: OnceLock<Cache> = OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

/// News eines Steam-Spiels laden (30 Minuten zwischengespeichert)
pub fn steam(appid: u32) -> Result<Vec<NewsItem>, String> {
    let now = crate::sys::now_secs();
    if let Ok(c) = cache().lock() {
        if let Some((t, v)) = c.get(&appid) {
            if now.saturating_sub(*t) < 1800 {
                return Ok(v.clone());
            }
        }
    }
    let url = format!("https://api.steampowered.com/ISteamNews/GetNewsForApp/v2/?appid={appid}&count=20&maxlength=3000&format=json");
    let mut r = crate::steam::http_agent().get(&url).call().map_err(|e| e.to_string())?;
    let body = r.body_mut().with_config().limit(4 * 1024 * 1024).read_to_string().map_err(|e| e.to_string())?;
    let items = parse(&body);
    if let Ok(mut c) = cache().lock() {
        c.insert(appid, (now, items.clone()));
    }
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_is_safe() {
        let t = to_plain("[h1]Patch 1.2[/h1][p]Fixed &amp; improved[/p][list][*]Crash fix[*]New map[/list]<script>alert(1)</script>{STEAM_CLAN_IMAGE}/x.png");
        assert!(t.starts_with("Patch 1.2\n"));
        assert!(t.contains("Fixed & improved"));
        assert!(t.contains("• Crash fix\n• New map"));
        // Unbekannte Tags bleiben als Text sichtbar – werden aber nie als HTML ausgeführt
        assert!(!t.contains("STEAM_CLAN_IMAGE"));
        assert_eq!(to_plain("a < b and c > d"), "a < b and c > d");
        assert_eq!(entities("&#x41;&#66;&bogus;&"), "AB&bogus;&");
        assert_eq!(entities("&#0;x"), "&#0;x");
        let long = "x".repeat(5000);
        assert!(to_plain(&long).chars().count() <= MAX_TEXT + 1);
    }

    #[test]
    fn parse_items() {
        let j = r#"{"appnews":{"appid":440,"newsitems":[
            {"title":"Team Fortress 2 Update Released","url":"https://store.steampowered.com/news/1","date":1700000000,"feedlabel":"Community Announcements","contents":"[list][*]Fixed a bug[/list]","tags":["patchnotes"]},
            {"title":"Big sale\u0007","url":"javascript:alert(1)","date":1,"feedlabel":"PC Gamer","contents":"<p>Hello</p>"},
            {"title":"","url":"https://x","date":1}
        ]}}"#;
        let n = parse(j);
        assert_eq!(n.len(), 2);
        assert!(n[0].patch);
        assert_eq!(n[0].text, "• Fixed a bug");
        assert_eq!(n[1].url, None);
        assert_eq!(n[1].title, "Big sale");
        assert!(!n[1].patch);
        assert!(parse("kaputt").is_empty());
    }
}
