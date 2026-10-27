//! Online-Funktionen der Oberfläche: Gratis-Spiele und Epic-Bibliothek (ohne eigenes Konto).
//! Alle Netzwerkaufrufe laufen im Hintergrund, die Oberfläche bleibt flüssig.

use super::state::*;
use crate::freegames::FreeGame;
use crate::i18n::{t, tf};
use crate::sys;
use dioxus::core::spawn_forever;
use dioxus::prelude::*;

// ----------------------------------------------------------------------
// Epic-Konto: komplette Epic-Bibliothek
// ----------------------------------------------------------------------

/// Bibliothek mit dem gespeicherten Epic-Konto neu laden.
/// `force` = false → höchstens alle 6 Stunden.
pub fn sync_epic(app: App, force: bool) {
    let Some(auth) = app.settings.peek().epic_auth.clone() else { return };
    let last = app.settings.peek().epic_synced;
    if *app.epic_busy.peek() || (!force && sys::now_secs().saturating_sub(last) < 6 * 3600) {
        return;
    }
    run_epic(app, move |p| {
        let fresh = crate::epicacct::ensure_fresh(&auth)?;
        let list = crate::epicacct::fetch_library(&fresh, &crate::epicacct::load_cache(), &crate::i18n::current_code(), p)?;
        Ok((fresh, list))
    });
}

/// Code aus dem Browser (oder die ganze kopierte Seite) → Konto verbinden
pub fn epic_connect(app: App, pasted: String) {
    if *app.epic_busy.peek() {
        return;
    }
    let Some(code) = crate::epicacct::extract_code(&pasted) else {
        let mut err = app.epic_error;
        err.set(Some(t("epic.err_code")));
        return;
    };
    run_epic(app, move |p| {
        let auth = crate::epicacct::exchange(&code)?;
        // Anmeldung hat geklappt → ab jetzt zählt der Fortschritt
        p.total.store(1, std::sync::atomic::Ordering::Relaxed);
        let list = crate::epicacct::fetch_library(&auth, &crate::epicacct::load_cache(), &crate::i18n::current_code(), p)?;
        Ok((auth, list))
    });
}

fn run_epic<F>(app: App, job: F)
where
    F: FnOnce(&crate::epicacct::Progress) -> Result<(crate::epicacct::EpicAuth, Vec<crate::epicacct::OwnedEpic>), String> + Send + 'static,
{
    let (mut busy, mut err, mut prog) = (app.epic_busy, app.epic_error, app.epic_progress);
    busy.set(true);
    err.set(None);
    prog.set((0, 0));
    let progress = std::sync::Arc::new(crate::epicacct::Progress::default());
    // Fortschritt regelmäßig in die Oberfläche übernehmen
    let watch = progress.clone();
    spawn_forever(async move {
        use std::sync::atomic::Ordering;
        while *app.epic_busy.peek() {
            let mut prog = app.epic_progress;
            prog.set((watch.done.load(Ordering::Relaxed), watch.total.load(Ordering::Relaxed)));
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
    });
    spawn_forever(async move {
        let p2 = progress.clone();
        let res = tokio::task::spawn_blocking(move || job(&p2)).await;
        let (mut busy, mut err) = (app.epic_busy, app.epic_error);
        busy.set(false);
        match res {
            Ok(Ok((auth, list))) => {
                let n = list.len();
                crate::epicacct::save_cache(&list);
                app.update(|s| {
                    s.epic_auth = Some(auth);
                    s.epic_synced = sys::now_secs();
                    s.epic_show_uninstalled = true;
                });
                app.toast(&tf("epic.synced_toast", &[("n", &n.to_string())]), "check");
                rescan(app);
            }
            Ok(Err(e)) => {
                if e == t("epic.err_expired") {
                    // Anmeldung abgelaufen: Liste bleibt (Cache), nur neu verbinden nötig
                    app.update(|s| s.epic_auth = None);
                }
                err.set(Some(e));
            }
            Err(_) => err.set(Some(t("common.err_unknown"))),
        }
    });
}

pub fn epic_disconnect(app: App) {
    crate::epicacct::clear_cache();
    app.update(|s| {
        s.epic_auth = None;
        s.epic_synced = 0;
    });
    let mut err = app.epic_error;
    err.set(None);
    rescan(app);
}

// ----------------------------------------------------------------------
// Gratis-Spiele
// ----------------------------------------------------------------------

pub fn load_free(app: App, force: bool) {
    let state = *app.free_state.peek();
    if state == LoadState::Loading || (state == LoadState::Done && !force) {
        return;
    }
    let mut st = app.free_state;
    st.set(LoadState::Loading);
    let lang = crate::i18n::current_code();
    spawn_forever(async move {
        let res = tokio::task::spawn_blocking(move || crate::freegames::fetch(&lang)).await;
        let mut free = app.free;
        let mut st = app.free_state;
        match res {
            Ok((games, errors)) => {
                let failed = games.is_empty() && !errors.is_empty();
                free.set(games);
                st.set(if failed { LoadState::Failed } else { LoadState::Done });
            }
            Err(_) => st.set(LoadState::Failed),
        }
    });
}

/// Ist das Gratis-Spiel schon in der Bibliothek?
pub fn owned_game(app: App, fg: &FreeGame) -> Option<crate::model::Game> {
    let source = match fg.store {
        crate::freegames::FreeStore::Epic => crate::model::Source::Epic,
        crate::freegames::FreeStore::Steam => crate::model::Source::Steam,
    };
    app.games
        .read()
        .iter()
        .find(|g| g.source == source && fg.matches(&g.name, g.steam_appid))
        .cloned()
}

/// "Zur Bibliothek hinzufügen": öffnet das Spiel direkt im Epic Launcher bzw. in Steam.
pub fn claim_free(app: App, fg: &FreeGame) {
    let launcher = fg.launcher_url();
    let opened = match fg.store {
        crate::freegames::FreeStore::Epic if crate::epic::launcher_installed() => sys::open_url(&launcher).is_ok(),
        crate::freegames::FreeStore::Steam if sys::protocol_registered("steam") => sys::open_url(&launcher).is_ok(),
        _ => false,
    };
    if !opened {
        let _ = sys::open_url(&fg.url);
    }
    let store = match fg.store {
        crate::freegames::FreeStore::Epic => "Epic Games",
        crate::freegames::FreeStore::Steam => "Steam",
    };
    app.toast(&tf("free.claim_toast", &[("store", store)]), "download");
    // Sobald es im Store hinzugefügt ist, taucht es automatisch in der Bibliothek auf
    rescan_later(app, &[30, 90, 240, 600]);
}

/// Restzeit eines Gratis-Angebots
pub fn remaining(fg: &FreeGame) -> String {
    let now = sys::now_secs();
    let (key_d, key_h, key_m, target) = if fg.upcoming {
        ("free.in_days", "free.in_hours", "free.in_min", fg.starts)
    } else {
        ("free.left_days", "free.left_hours", "free.left_min", fg.ends)
    };
    let Some(target) = target else {
        return if fg.upcoming { t("free.soon") } else { t("free.now") };
    };
    let secs = target.saturating_sub(now);
    let n;
    let key = if secs >= 2 * 86_400 {
        n = secs / 86_400;
        key_d
    } else if secs >= 3600 {
        n = secs / 3600;
        key_h
    } else {
        n = (secs / 60).max(1);
        key_m
    };
    tf(key, &[("n", &n.to_string())])
}

// ----------------------------------------------------------------------
// Updates
// ----------------------------------------------------------------------

static UPDATE_LOOP: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Kurz nach dem Start und danach alle 6 Stunden nach Updates suchen
pub fn update_loop(app: App) {
    if UPDATE_LOOP.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    spawn_forever(async move {
        tokio::time::sleep(std::time::Duration::from_secs(8)).await;
        loop {
            if app.settings.peek().update_check {
                check_update(app, false);
            }
            tokio::time::sleep(std::time::Duration::from_secs(6 * 3600)).await;
        }
    });
}

/// Nach Updates suchen. `manual` = vom Nutzer angestoßen (dann auch "kein Update" melden).
pub fn check_update(app: App, manual: bool) {
    if matches!(*app.update_state.peek(), UpdateState::Checking | UpdateState::Downloading(..) | UpdateState::Installing) {
        return;
    }
    let mut st = app.update_state;
    st.set(UpdateState::Checking);
    spawn_forever(async move {
        let test = app.settings.peek().update_test;
        let res = tokio::task::spawn_blocking(move || crate::updater::check(test)).await;
        let mut st = app.update_state;
        let mut upd = app.update_avail;
        match res {
            Ok(Ok(Some(r))) => {
                let skipped = app.settings.peek().update_skip == r.version;
                st.set(UpdateState::Idle);
                if manual || !skipped {
                    app.toast(&tf("update.available", &[("version", &r.version)]), "download");
                    upd.set(Some(r));
                    if manual {
                        app.open(Modal::Update);
                    }
                }
            }
            Ok(Ok(None)) => {
                st.set(UpdateState::Idle);
                upd.set(None);
                if manual {
                    app.toast(&tf("update.latest", &[("version", crate::sys::VERSION)]), "check-circle");
                }
            }
            Ok(Err(e)) => st.set(if manual { UpdateState::Failed(e) } else { UpdateState::Idle }),
            Err(_) => st.set(UpdateState::Idle),
        }
    });
}

/// Update herunterladen und installieren
pub fn start_update(app: App) {
    let Some(release) = app.update_avail.peek().clone() else { return };
    let kind = crate::updater::InstallKind::detect();
    let total = crate::updater::pick_asset(&kind, &release.assets).map(|a| a.size).unwrap_or(0);
    let progress = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));
    let mut st = app.update_state;
    st.set(UpdateState::Downloading(0, total));
    let p2 = progress.clone();
    let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let done2 = done.clone();
    // Fortschritt anzeigen
    spawn_forever(async move {
        while !done2.load(std::sync::atomic::Ordering::Relaxed) {
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            let got = p2.load(std::sync::atomic::Ordering::Relaxed);
            let mut st = app.update_state;
            if matches!(*st.peek(), UpdateState::Downloading(..)) {
                st.set(if total > 0 && got >= total { UpdateState::Installing } else { UpdateState::Downloading(got, total) });
            }
        }
    });
    spawn_forever(async move {
        let res = tokio::task::spawn_blocking(move || crate::updater::install(&release, &kind, progress)).await;
        done.store(true, std::sync::atomic::Ordering::Relaxed);
        let mut st = app.update_state;
        match res {
            Ok(Ok(crate::updater::Outcome::Restart(p))) => st.set(UpdateState::Ready(p)),
            Ok(Ok(crate::updater::Outcome::Installed)) => st.set(UpdateState::Done),
            Ok(Ok(crate::updater::Outcome::Manual { file, command })) => {
                st.set(UpdateState::Manual(file.to_string_lossy().to_string(), command))
            }
            Ok(Err(e)) => st.set(UpdateState::Failed(e)),
            Err(_) => st.set(UpdateState::Failed(t("common.err_unknown"))),
        }
    });
}
