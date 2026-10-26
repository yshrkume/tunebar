use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub struct MediaState {
    controls: Mutex<Option<MediaControls>>,
}

impl MediaState {
    pub fn new() -> Self {
        Self {
            controls: Mutex::new(None),
        }
    }
}

pub fn setup_media_controls(app: &AppHandle) {
    let config = PlatformConfig {
        dbus_name: "tunebar",
        display_name: "TuneBar",
        hwnd: None,
    };

    match MediaControls::new(config) {
        Ok(mut controls) => {
            let app_handle = app.clone();
            if let Err(error) = controls.attach(move |event: MediaControlEvent| {
                let command = match event {
                    MediaControlEvent::Play => "play",
                    MediaControlEvent::Pause => "pause",
                    MediaControlEvent::Toggle => "toggle",
                    MediaControlEvent::Next => "next",
                    MediaControlEvent::Previous => "previous",
                    MediaControlEvent::Stop => "pause",
                    _ => return,
                };

                crate::tray::run_remote_command(&app_handle, command);
            }) {
                eprintln!("TuneBar: failed to attach media controls: {error:?}");
                return;
            }

            if let Some(state) = app.try_state::<MediaState>() {
                if let Ok(mut lock) = state.controls.lock() {
                    *lock = Some(controls);
                }
            }
        }
        Err(e) => {
            eprintln!("TuneBar: failed to create media controls: {e:?}");
        }
    }
}

pub fn update_metadata(app: &AppHandle, title: &str, artist: &str) {
    if let Some(state) = app.try_state::<MediaState>() {
        if let Ok(mut lock) = state.controls.lock() {
            if let Some(controls) = lock.as_mut() {
                if let Err(error) = controls.set_metadata(MediaMetadata {
                    title: Some(title),
                    artist: Some(artist),
                    album: None,
                    ..Default::default()
                }) {
                    eprintln!("TuneBar: failed to update media metadata: {error:?}");
                }
            }
        }
    }
}

pub fn update_playback(app: &AppHandle, playing: bool) {
    if let Some(state) = app.try_state::<MediaState>() {
        if let Ok(mut lock) = state.controls.lock() {
            if let Some(controls) = lock.as_mut() {
                let playback = if playing {
                    MediaPlayback::Playing { progress: None }
                } else {
                    MediaPlayback::Paused { progress: None }
                };
                if let Err(error) = controls.set_playback(playback) {
                    eprintln!("TuneBar: failed to update playback state: {error:?}");
                }
            }
        }
    }
}
