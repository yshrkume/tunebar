mod commands;
mod media;
mod tray;

use media::MediaState;
use tauri::Manager;

#[derive(Clone, Copy)]
enum RemoteCommand {
    Toggle,
    Play,
    Pause,
    Next,
    Previous,
}

impl RemoteCommand {
    fn from_str(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "toggle" => Some(Self::Toggle),
            "play" => Some(Self::Play),
            "pause" => Some(Self::Pause),
            "next" => Some(Self::Next),
            "previous" | "prev" => Some(Self::Previous),
            _ => None,
        }
    }

    fn as_bridge_command(self) -> &'static str {
        match self {
            Self::Toggle => "toggle",
            Self::Play => "play",
            Self::Pause => "pause",
            Self::Next => "next",
            Self::Previous => "previous",
        }
    }
}

fn parse_remote_command(args: &[String]) -> Option<RemoteCommand> {
    let mut args_iter = args.iter();
    while let Some(arg) = args_iter.next() {
        if arg == "--remote" {
            return args_iter.next().and_then(|command| RemoteCommand::from_str(command));
        }
    }
    None
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if let Some(command) = parse_remote_command(&args) {
                tray::run_remote_command(app, command.as_bridge_command());
                return;
            }

            // When a second instance is launched without remote command, show the popover.
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

            let startup_args: Vec<String> = std::env::args().collect();
            if let Some(command) = parse_remote_command(&startup_args) {
                tray::run_remote_command(app.handle(), command.as_bridge_command());
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running TuneBar");
}
