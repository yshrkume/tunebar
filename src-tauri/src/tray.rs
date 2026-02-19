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
    let autostart_enabled = app
        .autolaunch()
        .is_enabled()
        .unwrap_or(false);

    let launch_login_item = CheckMenuItemBuilder::with_id("launch_login", "Launch at Login")
        .checked(autostart_enabled)
        .build(app)?;
    let pip_item = MenuItem::with_id(
        app,
        "pip",
        "Picture in Picture",
        true,
        None::<&str>,
    )?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit TuneBar", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&launch_login_item, &pip_item, &separator, &quit_item],
    )?;

    let launch_login_clone = launch_login_item.clone();

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("TuneBar")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id().as_ref() {
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
                if currently_enabled {
                    let _ = autolaunch.disable();
                    let _ = launch_login_clone.set_checked(false);
                } else {
                    let _ = autolaunch.enable();
                    let _ = launch_login_clone.set_checked(true);
                }
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

fn toggle_popover(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.move_window(Position::TrayBottomCenter);
            let _ = window.show();
            let _ = window.set_focus();
        }
    } else {
        let _ = create_main_window(app, true);
    }
}

pub fn run_remote_command(app: &AppHandle, command: &str) {
    let window = if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        window
    } else if let Some(window) = create_main_window(app, false) {
        window
    } else {
        return;
    };

    let escaped_command = command.replace('\\', "\\\\").replace('\'', "\\'");
    let command_js = format!(
        r#"
        (function() {{
            const cmd = '{escaped_command}';
            const send = () => {{
                if (window.__MUSIC_BRIDGE__ && typeof window.__MUSIC_BRIDGE__.runCommand === "function") {{
                    return window.__MUSIC_BRIDGE__.runCommand(cmd);
                }}
                return false;
            }};

            if (send()) {{
                return;
            }}

            window.__TUNEBAR_PENDING_REMOTE_COMMAND__ = cmd;
            let attempts = 0;
            const timer = setInterval(() => {{
                attempts += 1;
                if (send() || attempts >= 40) {{
                    clearInterval(timer);
                }}
            }}, 250);
        }})();
        "#
    );
    let _ = window.eval(&command_js);
}

fn create_main_window(app: &AppHandle, show_on_create: bool) -> Option<tauri::WebviewWindow> {
    let ad_blocker = include_str!("../../src/scripts/ad-blocker.js");
    let media_bridge = include_str!("../../src/scripts/media-bridge.js");
    let error_handler = include_str!("../../src/scripts/error-handler.js");
    let init_script = format!(
        r#"
        // Wait for page to load then inject scripts
        (function() {{
            {ad_blocker}
            {media_bridge}
            {error_handler}
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
    .on_navigation(|url| {
        let host = url.host_str().unwrap_or("");
        // YouTube Music uses many Google-owned domains internally
        // (iframes, analytics, APIs, auth, etc.)
        let allowed_suffixes = [
            "youtube.com",
            "google.com",
            "googleapis.com",
            "gstatic.com",
            "googlevideo.com",
            "googletagmanager.com",
            "googleusercontent.com",
            "ytimg.com",
        ];
        if allowed_suffixes
            .iter()
            .any(|s| host == *s || host.ends_with(&format!(".{}", s)))
        {
            true
        } else {
            // Open in system browser
            let _ = std::process::Command::new("open")
                .arg(url.as_str())
                .spawn();
            false
        }
    })
    .initialization_script(&init_script);

    match builder.build() {
        Ok(window) => {
            if show_on_create {
                let _ = window.move_window(Position::TrayBottomCenter);
                let _ = window.show();
                let _ = window.set_focus();
            }

            // Hide on focus loss (popover behavior)
            let win_clone = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Focused(false) = event {
                    let _ = win_clone.hide();
                }
            });
            Some(window)
        }
        Err(e) => {
            log::error!("Failed to create main window: {}", e);
            None
        }
    }
}
