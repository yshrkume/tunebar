fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&["update_track_info", "update_playback_state"]),
    ))
    .expect("failed to build TuneBar");
}
