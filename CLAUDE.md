# CLAUDE.md — TuneBar Project Instructions

## Project Overview

TuneBar is a macOS menu bar YouTube Music player built with Tauri v2 + Vanilla TypeScript + Vite + Rust.

## Commands

- `npm install` — Install dependencies
- `npm run tauri dev` — Run in development mode
- `npm run tauri build` — Build release `.app` and `.dmg`
- Build output: `src-tauri/target/release/bundle/macos/TuneBar.app`

## Architecture

```
Tray Icon (menu bar)
  ├─ Left click  → Toggle popover window (YouTube Music in WebView)
  └─ Right click → Context menu (Launch at Login / PiP / Quit)

Popover Window (WebviewWindow)
  ├─ Loads https://music.youtube.com with Safari User-Agent
  ├─ Injected scripts via include_str! in tray.rs:
  │   ├─ ad-blocker.js  — CSS ad hiding + JS video ad skipping
  │   └─ media-bridge.js — window.__MUSIC_BRIDGE__ API + track change detection
  ├─ Hides on focus loss (popover behavior)
  └─ No window decorations, always on top

Rust Backend
  ├─ lib.rs      — Tauri setup, plugins, ActivationPolicy::Accessory (no Dock icon)
  ├─ tray.rs     — Tray icon, popover window creation/toggle, right-click menu
  ├─ media.rs    — souvlaki media key integration, Now Playing metadata
  └─ commands.rs — IPC commands (update_track_info, update_playback_state)
```

## Key Design Decisions

- **Safari User-Agent**: WKWebView IS WebKit, so Safari UA is legitimate. Chrome UA causes Google Sign-In to block login ("This browser may not be secure"). Safari UA works for both Google login and YouTube Music.
- **`windows: []` in tauri.conf.json**: Window is created programmatically in `tray.rs`, not declaratively.
- **`include_str!`**: JS scripts are embedded at compile time. Path is relative to the Rust source file (`src-tauri/src/tray.rs`), so `../../src/scripts/` reaches the frontend scripts directory.
- **Single-instance**: `tauri-plugin-single-instance` prevents duplicate tray icons from multiple launches.
- **Launch at Login**: `tauri-plugin-autostart` manages macOS LaunchAgent plist. `CheckMenuItem` in tray menu with state synced via `ManagerExt::autolaunch().is_enabled()` on startup.
- **`ActivationPolicy::Accessory`**: Hides the app from the Dock — menu bar only.

## File Conventions

- Rust source: `src-tauri/src/*.rs` — standard Rust style
- Frontend scripts: `src/scripts/*.js` — vanilla JS (injected into YouTube Music's context, not bundled by Vite)
- Config: `src-tauri/tauri.conf.json` — Tauri v2 config schema

## Known Constraints

- CSP in `tauri.conf.json` must allow all YouTube/Google domains for music playback and sign-in.
- `souvlaki` v0.7 — media key crate. PlatformConfig `dbus_name`/`display_name` are Linux-only but required by the struct. `hwnd` is Windows-only. macOS needs no special config.
- Ad blocker selectors may need updating if YouTube Music changes its DOM structure.
- The app is macOS-only (uses WKWebView, macOS-specific activation policy).
