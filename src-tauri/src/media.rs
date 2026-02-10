use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig,
};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

pub struct MediaState {
    controls: Arc<Mutex<Option<MediaControls>>>,
}

impl MediaState {
    pub fn new() -> Self {
        Self {
            controls: Arc::new(Mutex::new(None)),
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
            controls
                .attach(move |event: MediaControlEvent| {
                    let command = match event {
                        MediaControlEvent::Play => "play",
                        MediaControlEvent::Pause => "pause",
                        MediaControlEvent::Toggle => "togglePlay",
                        MediaControlEvent::Next => "next",
                        MediaControlEvent::Previous => "previous",
                        MediaControlEvent::Stop => "pause",
                        _ => return,
                    };

                    if let Some(window) = app_handle.get_webview_window("main") {
                        let js = format!("window.__MUSIC_BRIDGE__?.{}()", command);
                        let _ = window.eval(&js);
                    }
                })
                .ok();

            if let Some(state) = app.try_state::<MediaState>() {
                if let Ok(mut lock) = state.controls.lock() {
                    *lock = Some(controls);
                }
            }
        }
        Err(e) => {
            log::error!("Failed to create media controls: {:?}", e);
        }
    }
}

pub fn update_metadata(app: &AppHandle, title: &str, artist: &str) {
    if let Some(state) = app.try_state::<MediaState>() {
        if let Ok(mut lock) = state.controls.lock() {
            if let Some(controls) = lock.as_mut() {
                controls
                    .set_metadata(MediaMetadata {
                        title: Some(title),
                        artist: Some(artist),
                        album: None,
                        ..Default::default()
                    })
                    .ok();
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
                controls.set_playback(playback).ok();
            }
        }
    }
}
