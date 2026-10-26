//! Bibliothek: führt Steam, Epic und eigene Spiele zusammen.

use crate::model::{normalize_name, Art, CustomGame, Game, LaunchTarget, Settings, Source, StoreStatus};
use crate::problem::Problem;
use crate::stores::StoreLibrary;
use crate::{emulators, epic, icons, roblox, steam, stores, sys};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ScanResult {
    pub games: Vec<Game>,
    pub steam: StoreStatus,
    pub epic: StoreStatus,
    pub roblox: StoreStatus,
    /// Weitere Stores (GOG, EA …)
    pub stores: Vec<(Source, StoreStatus)>,
    pub warnings: Vec<Problem>,
    /// Weitere Versionen desselben Spiels: ID des angezeigten Spiels → andere Stores
    pub duplicates: BTreeMap<String, Vec<Game>>,
}

/// Komplette Bibliothek einlesen (läuft im Hintergrund-Thread).
pub fn scan(settings: &Settings) -> ScanResult {
    let mut result = ScanResult::default();

    if settings.steam_enabled {
        let s = steam::scan(&settings.steam_api_key, &settings.steam_id, settings.steam_show_uninstalled);
        result.games.extend(s.games);
        result.steam = s.status;
        if let Some(w) = s.warning {
            result.warnings.push(w);
        }
    } else {
        result.steam.note = Some(crate::i18n::t("store.steam_off"));
    }

    if settings.epic_enabled {
        let e = epic::scan(settings.epic_show_uninstalled);
        result.games.extend(e.games);
        result.epic = e.status;
    } else {
        result.epic.note = Some(crate::i18n::t("store.epic_off"));
    }

    if settings.roblox_enabled {
        let (games, status) = roblox::scan(&settings.roblox_games);
        result.games.extend(games);
        result.roblox = status;
    } else {
        result.roblox.note = Some(crate::i18n::t("roblox.off"));
    }

    for (src, sc) in stores::scan_all(settings) {
        // Gleiche ID nicht doppelt (z. B. GOG über Galaxy und Heroic)
        for g in sc.games {
            if !result.games.iter().any(|o| o.id == g.id) {
                result.games.push(g);
            }
        }
        result.stores.push((src, sc.status));
    }

    // Erweiterungen (nur wenn erlaubt; jede läuft als eigenes Programm)
    crate::extensions::refresh(settings);
    if settings.extensions_allowed {
        for lib in crate::extensions::libraries() {
            let sc = lib.scan(settings.stores_show_uninstalled);
            for g in sc.games {
                if !result.games.iter().any(|o| o.id == g.id) {
                    result.games.push(g);
                }
            }
            result.stores.push((lib.source(), sc.status));
        }
    }

    if settings.store_on(Source::Minecraft) {
        let (games, status) = crate::minecraft::scan();
        result.games.extend(games);
        result.stores.push((Source::Minecraft, status));
    }

    if settings.store_on(Source::Emulator) {
        result.games.extend(emulators::scan(&settings.emulators));
    }

    for g in result.games.iter_mut() {
        match g.source {
            Source::Epic => {
                if let Some(exe) = &g.exe {
                    if let Some(icon) = icons::cached_icon(Path::new(exe)) {
                        g.art.icon.insert(0, sys::file_url(&icon));
                    }
                }
                apply_lookup(g, &settings.art_lookup);
            }
            Source::Steam => {
                // Kein Icon im Steam-Cache? Dann aus der Spiel-EXE holen.
                if g.art.icon.is_empty() && g.installed {
                    if let Some(dir) = &g.install_dir {
                        if let Some(exe) = icons::guess_main_exe(Path::new(dir), &g.name) {
                            if let Some(icon) = icons::cached_icon(&exe) {
                                g.art.icon.push(sys::file_url(&icon));
                            }
                        }
                    }
                }
            }
            Source::Gog | Source::Ea | Source::Ubisoft | Source::BattleNet | Source::Amazon | Source::Itch | Source::Xbox => {
                if g.art.icon.is_empty() {
                    let exe = g.exe.clone().map(std::path::PathBuf::from).or_else(|| {
                        g.install_dir.as_ref().and_then(|d| icons::guess_main_exe(Path::new(d), &g.name))
                    });
                    if let Some(exe) = exe {
                        if let Some(icon) = icons::cached_icon(&exe) {
                            g.art.icon.push(sys::file_url(&icon));
                        }
                    }
                }
                apply_lookup(g, &settings.art_lookup);
            }
            Source::Emulator => apply_lookup(g, &settings.art_lookup),
            // Minecraft-Cover aus dem Steam-Store gibt es nicht → eigenes Bild in der Oberfläche
            Source::Minecraft => {}
            Source::Extension(_) => {
                if g.art.cover.is_empty() {
                    apply_lookup(g, &settings.art_lookup);
                }
            }
            Source::Roblox | Source::Custom => {}
        }
    }

    for c in &settings.custom_games {
        result.games.push(custom_game(c, &settings.art_lookup, true));
    }
    if settings.merge_duplicates {
        let (games, dupes) = merge_duplicates(std::mem::take(&mut result.games));
        result.games = games;
        result.duplicates = dupes;
    }
    for g in result.games.iter_mut().chain(result.duplicates.values_mut().flatten()) {
        apply_meta(g, settings);
    }
    result
}

/// Selbst bearbeiteten Namen / Hintergrund übernehmen
pub fn apply_meta(g: &mut Game, settings: &Settings) {
    if let Some(m) = settings.meta.get(&g.id) {
        if !m.name.trim().is_empty() {
            g.name = m.name.trim().to_string();
        }
        if !m.hero.trim().is_empty() {
            g.art.hero.insert(0, sys::file_url(Path::new(m.hero.trim())));
        }
    }
}

/// Dasselbe Spiel aus mehreren Stores nur einmal anzeigen (installierte Version bevorzugt).
/// Eigene Spiele, Emulator-ROMs und Roblox bleiben immer einzeln.
pub fn merge_duplicates(games: Vec<Game>) -> (Vec<Game>, BTreeMap<String, Vec<Game>>) {
    let mergeable = |g: &Game| !matches!(g.source, Source::Custom | Source::Emulator | Source::Roblox);
    let mut groups: BTreeMap<String, Vec<Game>> = BTreeMap::new();
    let mut out = Vec::new();
    for g in games {
        let key = normalize_name(&g.name);
        if mergeable(&g) && key.len() >= 2 {
            groups.entry(key).or_default().push(g);
        } else {
            out.push(g);
        }
    }
    let mut dupes = BTreeMap::new();
    for (_, mut list) in groups {
        list.sort_by_key(|g| (!g.installed, g.source.rank()));
        let main = list.remove(0);
        // Nur zusammenfassen, wenn es wirklich verschiedene Stores sind
        list.retain(|g| g.source != main.source);
        if !list.is_empty() {
            dupes.insert(main.id.clone(), list);
        }
        out.push(main);
    }
    (out, dupes)
}

/// Wandelt ein eigenes Spiel in einen Bibliothekseintrag um.
pub fn custom_game(c: &CustomGame, lookup: &BTreeMap<String, u32>, with_icon: bool) -> Game {
    let mut art = Art::default();
    if !c.cover.trim().is_empty() {
        art.cover.push(sys::file_url(Path::new(c.cover.trim())));
    }
    let exe_path = Path::new(&c.exe);
    if with_icon {
        if let Some(icon) = icons::cached_icon(exe_path) {
            art.icon.push(sys::file_url(&icon));
        }
    }
    let mut g = Game {
        id: c.id.clone(),
        name: c.name.clone(),
        source: Source::Custom,
        installed: true,
        install_dir: exe_path.parent().map(|p| p.to_string_lossy().to_string()),
        exe: Some(c.exe.clone()),
        launch: LaunchTarget::Custom,
        art,
        size_bytes: None,
        playtime_min: None,
        steam_appid: None,
    };
    apply_lookup(&mut g, lookup);
    g
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(id: &str, name: &str, source: Source, installed: bool) -> Game {
        Game {
            id: id.into(),
            name: name.into(),
            source,
            installed,
            install_dir: None,
            exe: None,
            launch: LaunchTarget::Custom,
            art: Art::default(),
            size_bytes: None,
            playtime_min: None,
            steam_appid: None,
        }
    }

    #[test]
    fn duplicates_prefer_installed() {
        let (list, d) = merge_duplicates(vec![
            g("s1", "The Witcher 3: Wild Hunt", Source::Steam, false),
            g("g1", "The Witcher® 3 Wild Hunt", Source::Gog, true),
            g("c1", "The Witcher 3 Wild Hunt", Source::Custom, true),
            g("e1", "Fortnite", Source::Epic, true),
        ]);
        assert_eq!(list.len(), 3);
        assert!(list.iter().any(|x| x.id == "g1"));
        assert!(list.iter().any(|x| x.id == "c1"));
        assert_eq!(d.get("g1").map(|v| v[0].id.as_str()), Some("s1"));
    }
}

/// Hängt Steam-Store-Bilder an, wenn wir das Spiel dort gefunden haben.
pub fn apply_lookup(g: &mut Game, lookup: &BTreeMap<String, u32>) {
    if let Some(&appid) = lookup.get(&normalize_name(&g.name)) {
        if appid != 0 {
            let cdn = steam::cdn_art(appid);
            g.art.cover.extend(cdn.cover);
            g.art.hero.extend(cdn.hero);
            g.art.logo.extend(cdn.logo);
            g.art.header.extend(cdn.header);
        }
    }
}

/// Sucht für Epic- und eigene Spiele passende Cover im Steam-Store (Hintergrund).
/// Gibt neue Einträge für den Cache zurück: normalisierter Name → AppID (0 = nichts gefunden).
pub fn lookup_missing_art(names: Vec<String>, known: &BTreeMap<String, u32>) -> BTreeMap<String, u32> {
    let mut found = BTreeMap::new();
    let todo: Vec<String> = names
        .into_iter()
        .filter(|n| !known.contains_key(&normalize_name(n)))
        .take(40)
        .collect();
    if todo.is_empty() {
        return found;
    }
    let agent = steam::http_agent();
    for name in todo {
        match steam::store_lookup(&agent, &name) {
            Some(appid) => {
                found.insert(normalize_name(&name), appid);
            }
            None => break, // Offline o. Ä. → später erneut versuchen
        }
    }
    found
}
