use tauri::{command, AppHandle};

use crate::media;

#[command]
pub fn update_track_info(app: AppHandle, title: String, artist: String) -> Result<(), String> {
    if title.len() > 4096 || artist.len() > 4096 {
        return Err("track metadata exceeds size limit".into());
    }
    media::update_metadata(&app, &title, &artist);
    Ok(())
}

#[command]
pub fn update_playback_state(app: AppHandle, playing: bool) {
    media::update_playback(&app, playing);
}
