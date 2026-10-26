use std::sync::Mutex;

use tauri::{
    menu::{CheckMenuItemBuilder, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_positioner::{Position, WindowExt};

const MAIN_WINDOW_LABEL: &str = "main";
const WINDOW_WIDTH: f64 = 400.0;
const WINDOW_HEIGHT: f64 = 600.0;
// Use Safari UA — WKWebView IS WebKit so this is legitimate.
// Chrome UA gets blocked by Google Sign-In ("This browser may not be secure").
// Safari UA works for both Google login and YouTube Music.
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.3 Safari/605.1.15";

pub fn create_tray(app: &AppHandle) -> tauri::Result<()> {
    // Check current autostart state
    let autostart_enabled = app.autolaunch().is_enabled().unwrap_or(false);

    let launch_login_item = CheckMenuItemBuilder::with_id("launch_login", "Launch at Login")
        .checked(autostart_enabled)
        .build(app)?;
    let show_item = MenuItem::with_id(app, "show", "Open TuneBar", true, None::<&str>)?;
    let reload_item = MenuItem::with_id(app, "reload", "Reload YouTube Music", true, None::<&str>)?;
    let pip_item = MenuItem::with_id(app, "pip", "Picture in Picture", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit TuneBar", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &show_item,
            &reload_item,
            &launch_login_item,
            &pip_item,
            &separator,
            &quit_item,
        ],
    )?;

    let launch_login_clone = launch_login_item.clone();

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("TuneBar")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "show" => show_popover(app),
            "reload" => {
                show_popover(app);
                if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
                    if let Err(error) = window.navigate(
                        "https://music.youtube.com"
                            .parse()
                            .expect("valid music URL"),
                    ) {
                        eprintln!("TuneBar: could not reload music: {error}");
                    }
                }
            }
            "quit" => {
                app.exit(0);
            }
            "pip" => {
                if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
                    let _ = window.eval("window.__MUSIC_BRIDGE__?.requestPiP()");
                }
            }
            "launch_login" => {
                let autolaunch = app.autolaunch();
                let currently_enabled = autolaunch.is_enabled().unwrap_or(false);
                let result = if currently_enabled {
                    autolaunch.disable()
                } else {
                    autolaunch.enable()
                };
                if let Err(error) = result {
                    eprintln!("TuneBar: could not change launch at login: {error}");
                }
                let _ = launch_login_clone
                    .set_checked(autolaunch.is_enabled().unwrap_or(currently_enabled));
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                toggle_popover(app);
            }
        })
        .build(app)?;

    Ok(())
}

pub fn show_popover(app: &AppHandle) {
    let window = app
        .get_webview_window(MAIN_WINDOW_LABEL)
        .or_else(|| create_main_window(app));
    if let Some(window) = window {
        if window
            .move_window_constrained(Position::TrayBottomCenter)
            .is_err()
        {
            let _ = window.center();
        }
        if let Err(error) = window.show().and_then(|_| window.set_focus()) {
            eprintln!("TuneBar: could not show player: {error}");
        }
    }
}

fn is_allowed_navigation(url: &tauri::Url) -> bool {
    if url.scheme() != "https"
        || url.port_or_known_default() != Some(443)
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return false;
    }
    let host = url.host_str().unwrap_or("");
    [
        "youtube.com",
        "google.com",
        "googleapis.com",
        "gstatic.com",
        "googlevideo.com",
        "googletagmanager.com",
        "googleusercontent.com",
        "ytimg.com",
    ]
    .iter()
    .any(|suffix| host == *suffix || host.ends_with(&format!(".{suffix}")))
}

fn toggle_popover(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
            return;
        }
    }
    show_popover(app);
}

#[derive(Default)]
pub struct RemoteState {
    ready: bool,
    pending: Vec<String>,
}

pub fn run_remote_command(app: &AppHandle, command: &str) {
    let state = app.state::<Mutex<RemoteState>>();
    let Ok(mut state) = state.lock() else { return };
    if state.ready {
        if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
            send_command(&window, command);
            return;
        }
    }
    // Bound commands while offline or waiting for sign-in.
    if state.pending.len() < 32 {
        state.pending.push(command.to_owned());
    }
    drop(state);
    if app.get_webview_window(MAIN_WINDOW_LABEL).is_none() {
        create_main_window(app);
    }
}

fn send_command(window: &tauri::WebviewWindow, command: &str) {
    let command_json = serde_json::to_string(command).expect("serialize command");
    if let Err(error) = window.eval(format!(
        "window.__MUSIC_BRIDGE__?.runCommand({command_json})"
    )) {
        eprintln!("TuneBar: failed to send playback command: {error}");
    }
}

fn create_main_window(app: &AppHandle) -> Option<tauri::WebviewWindow> {
    let ad_blocker = include_str!("../../src/scripts/ad-blocker.js");
    let media_bridge = include_str!("../../src/scripts/media-bridge.js");
    let error_handler = include_str!("../../src/scripts/error-handler.js");
    let init_script = format!(
        r#"
        // Wait for page to load then inject scripts
        (function() {{
            if (location.origin !== "https://music.youtube.com" || window.top !== window) return;
            const initialize = () => {{
                {ad_blocker}
                {media_bridge}
                {error_handler}
            }};
            if (document.readyState === "loading") {{
                document.addEventListener("DOMContentLoaded", initialize, {{ once: true }});
            }} else {{
                initialize();
            }}
        }})();
        "#
    );

    let builder = WebviewWindowBuilder::new(
        app,
        MAIN_WINDOW_LABEL,
        WebviewUrl::External("https://music.youtube.com".parse().unwrap()),
    )
    .title("TuneBar")
    .inner_size(WINDOW_WIDTH, WINDOW_HEIGHT)
    .decorations(false)
    .always_on_top(true)
    .visible(false)
    .skip_taskbar(true)
    .user_agent(USER_AGENT)
    .on_page_load(|window, payload| {
        let state = window.state::<Mutex<RemoteState>>();
        let Ok(mut state) = state.lock() else { return };
        state.ready = matches!(payload.event(), tauri::webview::PageLoadEvent::Finished)
            && payload.url().origin().ascii_serialization() == "https://music.youtube.com";
        if state.ready {
            for command in state.pending.drain(..) {
                send_command(&window, &command);
            }
        }
    })
    .on_navigation(is_allowed_navigation)
    .on_new_window(|_, _| tauri::webview::NewWindowResponse::Deny)
    .initialization_script(&init_script);

    match builder.build() {
        Ok(window) => {
            // A newly created macOS window may report an initial unfocused
            // event. Only dismiss after it has actually acquired focus.
            let win_clone = window.clone();
            let has_focus = std::sync::atomic::AtomicBool::new(false);
            window.on_window_event(move |event| {
                use std::sync::atomic::Ordering;
                match event {
                    tauri::WindowEvent::Focused(true) => {
                        has_focus.store(true, Ordering::Relaxed);
                    }
                    tauri::WindowEvent::Focused(false)
                        if has_focus.swap(false, Ordering::Relaxed) =>
                    {
                        let _ = win_clone.hide();
                    }
                    tauri::WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();
                        let _ = win_clone.hide();
                    }
                    _ => {}
                }
            });
            Some(window)
        }
        Err(e) => {
            eprintln!("TuneBar: failed to create main window: {e}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::is_allowed_navigation;

    #[test]
    fn navigation_requires_https_and_real_google_domain_boundaries() {
        for url in [
            "https://music.youtube.com/",
            "https://accounts.google.com/signin",
            "https://www.youtube.com/",
        ] {
            assert!(is_allowed_navigation(&url.parse().unwrap()), "{url}");
        }
        for url in [
            "http://music.youtube.com/",
            "file:///etc/passwd",
            "javascript:alert(1)",
            "https://youtube.com.evil.example/",
            "https://evilyoutube.com/",
            "https://music.youtube.com:8443/",
            "https://user:password@music.youtube.com/",
            "mailto:test@example.com",
        ] {
            assert!(!is_allowed_navigation(&url.parse().unwrap()), "{url}");
        }
    }
}
