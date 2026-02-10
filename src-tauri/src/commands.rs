use tauri::{AppHandle, command};

use crate::media;

#[command]
pub fn update_track_info(app: AppHandle, title: String, artist: String) {
    media::update_metadata(&app, &title, &artist);
}

#[command]
pub fn update_playback_state(app: AppHandle, playing: bool) {
    media::update_playback(&app, playing);
}
