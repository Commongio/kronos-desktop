// updater.rs — how an installed KRONOS gets a newer KRONOS.
//
// ── WHAT NEEDS THIS, AND WHAT DOES NOT ────────────────────────────────────
//
// Almost nothing. The app is a window on kronosterminal.online; a deploy of
// the Terminal reaches every installed app on its next load, like a browser
// tab. This code is only for changes to the SHELL — tray, notification
// bridge, installer, Tauri itself — which happen a few times a year.
//
// ── THE FLOW ──────────────────────────────────────────────────────────────
//
//   check     shortly after launch, then once a day while running. Fetches
//             latest.json from the GitHub release and compares versions.
//   offer     a small KRONOS-styled window: version, notes, Install / Later.
//             Never a system dialog: the point of an installed app is that
//             it looks like the product, including here.
//   download  the plugin reports bytes as they arrive; the bar is real.
//   verify    the plugin checks the release's minisign signature against the
//             public key compiled into tauri.conf.json BEFORE installing, so
//             a compromised download host cannot push an update. This is the
//             one thing that makes auto-update safe to have at all.
//   install   Windows: the NSIS installer runs in passive mode and relaunches
//             the app. macOS: the .app is replaced in place and we restart.
//
// "Later" snoozes for a day, not forever. An installed base that can decline
// updates permanently is an installed base that stays on a broken build by
// accident.
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_updater::{Update, UpdaterExt};

const FIRST_CHECK_AFTER: Duration = Duration::from_secs(15);
const CHECK_EVERY: Duration = Duration::from_secs(24 * 3600);
const SNOOZE_FOR: Duration = Duration::from_secs(24 * 3600);
const WINDOW_LABEL: &str = "update";

/// The update we most recently offered, kept so Install does not re-fetch.
#[derive(Default)]
pub struct Pending(pub Mutex<Option<Update>>);

#[derive(Clone, serde::Serialize)]
struct Progress {
    downloaded: u64,
    total: Option<u64>,
    phase: &'static str,
}

fn snooze_path(app: &AppHandle) -> Option<std::path::PathBuf> {
    app.path().app_data_dir().ok().map(|d| d.join("update-snoozed-until"))
}

fn snoozed(app: &AppHandle) -> bool {
    let Some(p) = snooze_path(app) else { return false };
    let Ok(s) = std::fs::read_to_string(p) else { return false };
    let until = s.trim().parse::<u64>().unwrap_or(0);
    now_secs() < until
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// Start the background check loop. Called once from setup.
pub fn start(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(FIRST_CHECK_AFTER).await;
        loop {
            check_and_offer(&app).await;
            tokio::time::sleep(CHECK_EVERY).await;
        }
    });
}

async fn check_and_offer(app: &AppHandle) {
    if snoozed(app) {
        return;
    }
    // A failed check is silent. The user did not ask for anything, the app
    // works, and a dialog about GitHub being unreachable helps nobody.
    let Ok(updater) = app.updater() else { return };
    let Ok(Some(update)) = updater.check().await else { return };

    let version = update.version.clone();
    let notes = update.body.clone().unwrap_or_default();
    if let Some(state) = app.try_state::<Pending>() {
        *state.0.lock().unwrap() = Some(update);
    }
    open_window(app, &version, &notes);
}

fn open_window(app: &AppHandle, version: &str, notes: &str) {
    if let Some(w) = app.get_webview_window(WINDOW_LABEL) {
        let _ = w.set_focus();
        return;
    }
    let url = format!(
        "update.html?version={}&notes={}",
        urlencoding::encode(version),
        urlencoding::encode(notes)
    );
    let _ = WebviewWindowBuilder::new(app, WINDOW_LABEL, WebviewUrl::App(url.into()))
        .title("KRONOS update")
        .inner_size(440.0, 320.0)
        .resizable(false)
        .minimizable(false)
        .center()
        .background_color(tauri::window::Color(0, 0, 0, 255))
        .build();
}

/// Install / Later are the only two things the update window can ask for.

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    let update = app
        .state::<Pending>()
        .0
        .lock()
        .unwrap()
        .clone()
        .ok_or("no update pending")?;

    let emit = |p: Progress| { let _ = app.emit_to(WINDOW_LABEL, "update-progress", p); };
    // Shared between the two callbacks, which the borrow checker otherwise
    // sees as one mutable and one immutable capture of the same counter.
    let downloaded = AtomicU64::new(0);

    let bytes = update
        .download(
            |chunk, total| {
                let d = downloaded.fetch_add(chunk as u64, Ordering::Relaxed) + chunk as u64;
                emit(Progress { downloaded: d, total, phase: "downloading" });
            },
            || {
                let d = downloaded.load(Ordering::Relaxed);
                emit(Progress { downloaded: d, total: Some(d), phase: "verifying" });
            },
        )
        .await
        .map_err(|e| e.to_string())?;

    let d = downloaded.load(Ordering::Relaxed);
    emit(Progress { downloaded: d, total: Some(d), phase: "installing" });
    // On Windows this launches the installer and exits the process; the
    // installer relaunches the app. On macOS it returns and we restart.
    update.install(bytes).map_err(|e| e.to_string())?;
    app.restart();
}

#[tauri::command]
pub fn snooze_update(app: AppHandle) {
    if let Some(p) = snooze_path(&app) {
        if let Some(dir) = p.parent() { let _ = std::fs::create_dir_all(dir); }
        let _ = std::fs::write(p, (now_secs() + SNOOZE_FOR.as_secs()).to_string());
    }
    if let Some(w) = app.get_webview_window(WINDOW_LABEL) {
        let _ = w.close();
    }
}
