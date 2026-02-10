# TuneBar

macOS menu bar player for YouTube Music, built with [Tauri v2](https://v2.tauri.app/).

## Features

- Menu bar (tray) icon with popover window
- YouTube Music loaded directly in a WebView
- Media key support (play/pause/next/previous) via macOS Now Playing
- Picture-in-Picture mode
- Ad blocker (CSS-based ad hiding + video ad skipping)
- Single-instance enforcement
- No Dock icon (menu bar only app)

## Usage

| Action | Behavior |
|--------|----------|
| Left-click tray icon | Toggle YouTube Music popover |
| Right-click tray icon | Context menu (PiP / Quit) |
| Click outside popover | Auto-hide |
| Media keys | Play/Pause/Next/Previous |

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) (LTS)
- [Rust](https://www.rust-lang.org/tools/install)
- Xcode Command Line Tools (`xcode-select --install`)

### Setup

```bash
npm install
```

### Dev

```bash
npm run tauri dev
```

### Build

```bash
npm run tauri build
```

The `.app` bundle is generated at `src-tauri/target/release/bundle/macos/TuneBar.app`.

To install, copy to `/Applications`:

```bash
cp -r src-tauri/target/release/bundle/macos/TuneBar.app /Applications/
```

## Tech Stack

- **Tauri v2** - Application framework
- **WKWebView** - Renders YouTube Music (Safari User-Agent)
- **Vanilla TypeScript + Vite** - Frontend tooling
- **Rust** - Backend (tray management, media controls)
- **souvlaki** - macOS media key integration (MPRemoteCommandCenter)
- **tauri-plugin-positioner** - Window positioning near tray icon
- **tauri-plugin-single-instance** - Prevents multiple instances

## License

MIT
