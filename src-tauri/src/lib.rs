mod commands;
mod media;
mod tray;

use media::MediaState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // When a second instance is launched, show the popover of the existing instance
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_opener::init())
        .manage(MediaState::new())
        .invoke_handler(tauri::generate_handler![
            commands::update_track_info,
            commands::update_playback_state,
        ])
        .setup(|app| {
            // Hide from Dock (Accessory app — menu bar only)
            #[cfg(target_os = "macos")]
            {
                use tauri::ActivationPolicy;
                let _ = app.handle().set_activation_policy(ActivationPolicy::Accessory);
            }

            tray::create_tray(app.handle())?;
            media::setup_media_controls(app.handle());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running TuneBar");
}
