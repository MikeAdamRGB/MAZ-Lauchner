//! Statistik: Spielzeit pro Woche, Monat und Store, meistgespielte und ungespielte Spiele.
//! Rechnet nur mit lokalen Daten.

use crate::model::{status, Game, Settings, Source};

const DAY: u64 = 86_400;
const WEEK: u64 = 7 * DAY;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct StoreRow {
    pub source: Option<Source>,
    pub label: String,
    pub games: usize,
    pub minutes: u64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Stats {
    /// Gesamte Spielzeit aller Spiele (Store-Angaben + im Launcher gemessen)
    pub total_min: u64,
    /// Im Launcher gemessen: letzte 7 bzw. 30 Tage
    pub week_min: u64,
    pub month_min: u64,
    /// Letzte 12 Wochen, älteste zuerst (Minuten je Woche)
    pub weeks: Vec<u64>,
    pub per_store: Vec<StoreRow>,
    /// Meistgespielt: (ID, Name, Minuten)
    pub top: Vec<(String, String, u64)>,
    pub games: usize,
    pub installed: usize,
    /// Nie gespielt (keine Spielzeit, nie gestartet, nicht durchgespielt)
    pub unplayed: usize,
    /// Anzahl je Status in der Reihenfolge von `status::ALL`
    pub by_status: Vec<(&'static str, usize)>,
}

/// Wurde dieses Spiel schon einmal gespielt?
pub fn played(g: &Game, s: &Settings) -> bool {
    s.playtime(g) > 0 || s.launch_count.get(&g.id).copied().unwrap_or(0) > 0 || s.last_played(&g.id) > 0
}

pub fn is_unplayed(g: &Game, s: &Settings) -> bool {
    !played(g, s) && !matches!(s.status_of(&g.id), status::FINISHED | status::PLAYING | status::DROPPED)
}

pub fn compute(games: &[Game], s: &Settings, now: u64) -> Stats {
    let visible: Vec<&Game> = games.iter().filter(|g| !s.is_hidden(&g.id)).collect();
    let mut st = Stats { games: visible.len(), ..Default::default() };
    st.installed = visible.iter().filter(|g| g.installed).count();
    st.unplayed = visible.iter().filter(|g| is_unplayed(g, s)).count();
    st.total_min = visible.iter().map(|g| s.playtime(g)).sum();

    // Sitzungen (nur im Launcher gemessen)
    let mut weeks = vec![0u64; 12];
    for ses in &s.sessions {
        if ses.start > now {
            continue;
        }
        let age = now - ses.start;
        if age < WEEK {
            st.week_min += ses.minutes;
        }
        if age < 30 * DAY {
            st.month_min += ses.minutes;
        }
        let w = (age / WEEK) as usize;
        if w < 12 {
            weeks[11 - w] += ses.minutes;
        }
    }
    st.weeks = weeks;

    // Pro Store
    let mut rows: Vec<StoreRow> = Vec::new();
    for g in &visible {
        let label = g.source.label();
        let i = match rows.iter().position(|r| r.label == label) {
            Some(i) => i,
            None => {
                rows.push(StoreRow { source: Some(g.source), label, ..Default::default() });
                rows.len() - 1
            }
        };
        rows[i].games += 1;
        rows[i].minutes += s.playtime(g);
    }
    rows.sort_by(|a, b| b.minutes.cmp(&a.minutes).then(b.games.cmp(&a.games)).then(a.label.cmp(&b.label)));
    st.per_store = rows;

    // Meistgespielt
    let mut top: Vec<(String, String, u64)> =
        visible.iter().map(|g| (g.id.clone(), g.name.clone(), s.playtime(g))).filter(|t| t.2 > 0).collect();
    top.sort_by(|a, b| b.2.cmp(&a.2).then(a.1.to_lowercase().cmp(&b.1.to_lowercase())));
    top.truncate(10);
    st.top = top;

    st.by_status = status::ALL.iter().map(|k| (*k, visible.iter().filter(|g| s.status_of(&g.id) == *k).count())).collect();
    st
}

/// Minuten lesbar: "45 min", "3 h 20 min", "1.234 h"
pub fn fmt_minutes(m: u64) -> String {
    if m < 60 {
        format!("{m} min")
    } else if m < 100 * 60 {
        let (h, r) = (m / 60, m % 60);
        if r == 0 {
            format!("{h} h")
        } else {
            format!("{h} h {r} min")
        }
    } else {
        let h = m / 60;
        let s = h.to_string();
        // Tausenderpunkte
        let mut out = String::new();
        for (i, c) in s.chars().enumerate() {
            if i > 0 && (s.len() - i) % 3 == 0 {
                out.push('.');
            }
            out.push(c);
        }
        format!("{out} h")
    }
}

/// Zufallsspiel: Filter + Auswahl. `seed` sorgt für Abwechslung.
#[derive(Clone, Debug, PartialEq)]
pub struct RandomFilter {
    pub installed_only: bool,
    /// Leer = alle Stores, sonst Source::key()
    pub store: String,
    pub skip_done: bool,
    pub unplayed_only: bool,
}

pub fn random_candidates<'a>(games: &'a [Game], s: &Settings, f: &RandomFilter) -> Vec<&'a Game> {
    games
        .iter()
        .filter(|g| !s.is_hidden(&g.id))
        .filter(|g| !f.installed_only || g.installed)
        .filter(|g| f.store.is_empty() || g.source.key() == f.store)
        .filter(|g| !f.skip_done || !matches!(s.status_of(&g.id), status::FINISHED | status::DROPPED))
        .filter(|g| !f.unplayed_only || is_unplayed(g, s))
        .collect()
}

pub fn pick<'a>(list: &[&'a Game], seed: u64, avoid: &str) -> Option<&'a Game> {
    if list.is_empty() {
        return None;
    }
    // xorshift für eine gleichmäßige Auswahl
    let mut x = seed ^ 0x9E37_79B9_7F4A_7C15;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    let mut i = (x % list.len() as u64) as usize;
    if list.len() > 1 && list[i].id == avoid {
        i = (i + 1) % list.len();
    }
    list.get(i).copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Art, LaunchTarget};

    fn g(id: &str, src: Source, installed: bool, playtime: Option<u64>) -> Game {
        Game {
            id: id.into(),
            name: id.to_uppercase(),
            source: src,
            installed,
            install_dir: None,
            exe: None,
            launch: LaunchTarget::Custom,
            art: Art::default(),
            size_bytes: None,
            playtime_min: playtime,
            steam_appid: None,
        }
    }

    #[test]
    fn numbers() {
        let now = 100 * WEEK;
        let mut s = Settings::default();
        let games = vec![
            g("a", Source::Steam, true, Some(600)),
            g("b", Source::Steam, false, None),
            g("c", Source::Epic, true, None),
            g("d", Source::Gog, true, None),
        ];
        s.add_session("c", now - DAY, 90);
        s.add_session("c", now - 10 * DAY, 30);
        s.add_session("c", now - 40 * DAY, 15);
        s.playtime_local.insert("c".into(), 135);
        s.hidden.push("d".into());
        s.status.insert("b".into(), status::DROPPED.into());
        let st = compute(&games, &s, now);
        assert_eq!(st.games, 3);
        assert_eq!(st.installed, 2);
        assert_eq!(st.week_min, 90);
        assert_eq!(st.month_min, 120);
        assert_eq!(st.weeks.len(), 12);
        assert_eq!(st.weeks[11], 90);
        assert_eq!(st.total_min, 735);
        assert_eq!(st.top[0].0, "a");
        assert_eq!(st.unplayed, 0, "b ist abgebrochen, zählt nicht als ungespielt");
        assert_eq!(st.per_store[0].label, "Steam");
        assert_eq!(st.per_store[0].games, 2);
        assert_eq!(st.by_status.iter().find(|x| x.0 == status::PLAYING).map(|x| x.1), Some(1));
    }

    #[test]
    fn random() {
        let s = Settings::default();
        let games = vec![g("a", Source::Steam, true, None), g("b", Source::Epic, false, None)];
        let f = RandomFilter { installed_only: true, store: String::new(), skip_done: true, unplayed_only: false };
        let c = random_candidates(&games, &s, &f);
        assert_eq!(c.len(), 1);
        assert_eq!(pick(&c, 42, "").map(|g| g.id.as_str()), Some("a"));
        let f2 = RandomFilter { installed_only: false, store: "epic".into(), ..f };
        assert_eq!(random_candidates(&games, &s, &f2).len(), 1);
        let all: Vec<&Game> = games.iter().collect();
        for seed in 0..20 {
            assert_ne!(pick(&all, seed, "a").map(|g| g.id.as_str()), Some("a"));
        }
        assert!(pick(&[], 1, "").is_none());
    }

    #[test]
    fn format() {
        assert_eq!(fmt_minutes(45), "45 min");
        assert_eq!(fmt_minutes(200), "3 h 20 min");
        assert_eq!(fmt_minutes(120), "2 h");
        assert_eq!(fmt_minutes(1234 * 60), "1.234 h");
    }
}
