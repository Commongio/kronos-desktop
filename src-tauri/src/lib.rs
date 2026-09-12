// kronos-desktop — a window on kronosterminal.online.
//
// ── WHAT THIS IS, AND IS NOT ──────────────────────────────────────────────
//
// This app contains no Terminal code and no secrets. Everything that decides
// anything — Stripe, the crons, the Lab contract, the model calls — stays on
// Cloudflare, and this is one more way to look at it, alongside the browser
// and the phone. Auth is a Bearer token the page keeps in localStorage, which
// is why loading the live origin directly costs nothing: login, checkout,
// the SSE trade stream and every API route work exactly as in a browser.
//
// The alternative — bundling the Next frontend into the binary — would put
// the page on tauri://localhost and make every call cross-origin: CORS,
// cookie SameSite, and a second deploy pipeline whose version could drift
// from the site's. Rejected. See the vault note "Desktop app is a shell, not
// a port".
//
// ── WHAT THE SHELL ACTUALLY DOES ──────────────────────────────────────────
//
// Two things a plain webview would get wrong:
//
//   OUTSIDE LINKS GO OUTSIDE. Stripe checkout, article links, the Lab —
//   anything not on kronosterminal.online opens in the system browser. In a
//   webview with no back button, an in-place navigation to stripe.com is a
//   trap: the user finishes paying and is looking at a Stripe page with no
//   way home. Both navigation and window.open are intercepted.
//
//   THE PAGE KNOWS WHERE IT IS. `window.__KRONOS_DESKTOP__` is set before
//   any page script runs, so the Terminal can adapt — PushAlerts can say
//   alerts come from the app instead of reporting Web Push unsupported,
//   checkout can return via a deep link instead of a browser tab.
//
//   IT UPDATES ITSELF, RARELY. Terminal changes need nothing — the window
//   shows the live site. Shell changes go through src/updater.rs: a signed
//   release, a KRONOS-styled offer, a real progress bar, and a relaunch.
//
//   ALERTS SURVIVE THE CLOSE BUTTON. Native webviews have no Web Push, so the
//   Terminal's DesktopAlerts.jsx polls once a minute and calls
//   `new Notification()` — which tauri-plugin-notification bridges to the OS
//   notification centre. That only works while the page is alive, so the
//   close button hides to the tray instead of quitting. Quit is on the tray
//   menu, where it is a decision rather than a reflex.
mod updater;

use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    webview::{NewWindowResponse, WebviewWindowBuilder},
    AppHandle, Manager, WindowEvent,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt as _};
use tauri_plugin_opener::OpenerExt;

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

/// The one origin the window may navigate within. Everything else is handed
/// to the system browser.
const HOME_HOST: &str = "kronosterminal.online";

/// Hosts that must stay IN the window even though they are not ours: the
/// auth provider round-trips through here during sign-in and password reset,
/// and bouncing that to an external browser would strand the session there.
const IN_WINDOW_HOSTS: &[&str] = &["supabase.co"];

fn stays_in_window(url: &url::Url) -> bool {
    let Some(host) = url.host_str() else { return true }; // about:blank and friends
    if host == HOME_HOST || host.ends_with(&format!(".{HOME_HOST}")) {
        return true;
    }
    IN_WINDOW_HOSTS.iter().any(|h| host == *h || host.ends_with(&format!(".{h}")))
}

/// Runs before any page script. Kept tiny: this is the only code the shell
/// injects into the site, and the site should need to trust as little of it
/// as possible.
const INIT_SCRIPT: &str = r#"
  Object.defineProperty(window, "__KRONOS_DESKTOP__", { value: true, writable: false, configurable: false });
"#;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(updater::Pending::default())
        .invoke_handler(tauri::generate_handler![updater::install_update, updater::snooze_update])
        // Registered but OFF by default. Launching at login is the user's call
        // from the tray menu, not something an installer decides for them.
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // Hide, don't close: the page keeps polling for signals.
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(|app| {
            let open = MenuItem::with_id(app, "open", "Open KRONOS", true, None::<&str>)?;
            let at_login = CheckMenuItem::with_id(
                app, "autostart", "Launch at login", true,
                app.autolaunch().is_enabled().unwrap_or(false), None::<&str>,
            )?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &at_login, &quit])?;
            TrayIconBuilder::new()
                .icon(app.default_window_icon().cloned().expect("bundle icon"))
                .tooltip("KRONOS")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, e| match e.id.as_ref() {
                    "open" => show_main(app),
                    "autostart" => {
                        // The check state has already flipped by the time
                        // this runs; make the OS agree with it.
                        let want = at_login.is_checked().unwrap_or(false);
                        let r = if want { app.autolaunch().enable() } else { app.autolaunch().disable() };
                        if r.is_err() { let _ = at_login.set_checked(!want); }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, e| {
                    if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = e {
                        show_main(tray.app_handle());
                    }
                })
                .build(app)?;

            let cfg = app
                .config()
                .app
                .windows
                .iter()
                .find(|w| w.label == "main")
                .cloned()
                .expect("tauri.conf.json must define the main window");

            let handle_nav = app.handle().clone();
            let handle_new = app.handle().clone();

            WebviewWindowBuilder::from_config(app, &cfg)?
                .initialization_script(INIT_SCRIPT)
                .on_navigation(move |url| {
                    if stays_in_window(url) {
                        return true;
                    }
                    // Cancelled in the window; opened where a back button exists.
                    let _ = handle_nav.opener().open_url(url.as_str(), None::<&str>);
                    false
                })
                .on_new_window(move |url, _features| {
                    // window.open / target=_blank. Same rule, and there is
                    // never a reason to spawn a second shell window for it.
                    if stays_in_window(&url) {
                        return NewWindowResponse::Allow;
                    }
                    let _ = handle_new.opener().open_url(url.as_str(), None::<&str>);
                    NewWindowResponse::Deny
                })
                .build()?;

            updater::start(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running KRONOS");
}

#[cfg(test)]
mod tests {
    use super::stays_in_window;
    use url::Url;

    #[test]
    fn home_and_subdomains_stay() {
        assert!(stays_in_window(&Url::parse("https://kronosterminal.online/terminal").unwrap()));
        assert!(stays_in_window(&Url::parse("https://www.kronosterminal.online/").unwrap()));
    }

    #[test]
    fn stripe_and_news_go_outside() {
        // The trap this guards: a payment page in a window with no back button.
        assert!(!stays_in_window(&Url::parse("https://checkout.stripe.com/c/pay/abc").unwrap()));
        assert!(!stays_in_window(&Url::parse("https://www.reuters.com/markets/").unwrap()));
        // The Lab is a different product with its own login; it gets a real browser.
        assert!(!stays_in_window(&Url::parse("https://www.kronoslab.online/").unwrap()));
    }

    #[test]
    fn a_lookalike_host_does_not_pass() {
        // ends_with(".kronosterminal.online") — not contains(). This is the
        // difference between a subdomain and a domain someone else owns.
        assert!(!stays_in_window(&Url::parse("https://kronosterminal.online.evil.com/").unwrap()));
        assert!(!stays_in_window(&Url::parse("https://notkronosterminal.online/").unwrap()));
    }

    #[test]
    fn auth_round_trip_stays() {
        assert!(stays_in_window(&Url::parse("https://abcdefgh.supabase.co/auth/v1/verify?x=1").unwrap()));
    }
}
