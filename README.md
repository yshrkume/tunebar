# TuneBar

macOS menu bar player for YouTube Music, built with [Tauri v2](https://v2.tauri.app/).

## Features

- Menu bar (tray) icon with popover window
- YouTube Music loaded directly in a WebView
- Media key support (play/pause/next/previous) via macOS Now Playing
- Picture-in-Picture mode
- Ad blocker (CSS-based ad hiding + video ad skipping)
- Launch at Login (auto-start on macOS login)
- Single-instance enforcement
- No Dock icon (menu bar only app)

## Usage

| Action | Behavior |
|--------|----------|
| Launch / reopen app | Open YouTube Music (also works before the first tray click) |
| Left-click tray icon | Toggle YouTube Music popover |
| Right-click tray icon | Context menu (Open / Reload / Launch at Login / PiP / Quit) |
| Click outside popover | Auto-hide |
| Media keys | Play/Pause/Next/Previous |

### Remote Commands

You can control playback from external tools (for example, Raycast):

```bash
/Applications/TuneBar.app/Contents/MacOS/tunebar --remote toggle
```

Supported commands:

- `toggle`
- `play`
- `pause`
- `next`
- `previous`

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) (LTS)
- [Rust](https://www.rust-lang.org/tools/install)
- Xcode Command Line Tools (`xcode-select --install`)

### Setup

```bash
npm ci
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

Quit TuneBar, then install to `/Applications` (the previous bundle is backed up):

```bash
./install.sh --dry-run
./install.sh
```

## Tech Stack

- **Tauri v2** - Application framework
- **WKWebView** - Renders YouTube Music (Safari User-Agent)
- **Vanilla TypeScript + Vite** - Frontend tooling
- **Rust** - Backend (tray management, media controls)
- **souvlaki** - macOS media key integration (MPRemoteCommandCenter)
- **tauri-plugin-positioner** - Window positioning near tray icon
- **tauri-plugin-single-instance** - Prevents multiple instances
- **tauri-plugin-autostart** - Launch at Login support

## License

MIT

## Reliability and security (2026-10-26)

Local builds use an ad-hoc signature for the complete app bundle (not Developer ID notarization).
Verify with `codesign --verify --deep --strict src-tauri/target/release/bundle/macos/TuneBar.app`.

Validated on macOS 27.0.1 (Apple Silicon). Older macOS versions are not part of the validation target.
Normal launch and Finder reopen show the player; `--background` keeps login launches hidden.
If loading fails, use **Reload YouTube Music** in the tray menu. Cookies and login data are preserved.

The remote YouTube Music page can only invoke the two media metadata/state commands. It has
no native window, filesystem, shell, opener, or autostart permissions. Scripts only initialize in
the top-level `https://music.youtube.com` document after the DOM exists. HTTPS Google/YouTube
navigation is allowed for authentication and playback; other schemes, unsolicited external
navigation and new windows are blocked. The configured CSP protects bundled local content;
it does not replace the remote website's CSP or the native IPC capability boundary.

Dependencies are locked in `package-lock.json` and `src-tauri/Cargo.lock`:

- Tauri 2.12.5 includes the fix for [GHSA-w28w-mhc8-qvjv](https://github.com/tauri-apps/tauri/security/advisories/GHSA-w28w-mhc8-qvjv) (channel response access control; fixed in 2.11.6).
- It also includes [GHSA-7gmj-67g7-phm9](https://github.com/tauri-apps/tauri/security/advisories/GHSA-7gmj-67g7-phm9) (Windows/Android origin confusion; fixed in 2.11.1). The platform-specific exploit is not a macOS issue.
- Vite 8.3.5 includes October's [GHSA-rq7h-c2jc-7f22](https://github.com/vitejs/vite/security/advisories/GHSA-rq7h-c2jc-7f22) fix. The development server binds to loopback only.

`npm audit` and `cargo audit` report zero known vulnerabilities with the reviewed locks.
Cargo audit also reports informational warnings for Linux dependencies (`glib`, `proc-macro-error`,
`zbus_names`, `zvariant`, `zvariant_derive`); none is present in the `aarch64-apple-darwin` build tree.
The macOS media integration's transitive `block` 0.1.6 emits a Rust future-compatibility warning;
current builds pass, but this needs upstream replacement before a future incompatible Rust release.
Audit results are a dated check of known issues, not a guarantee against unknown vulnerabilities.

### Checks

```sh
npm ci
npm test
npm run build
npm audit
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
cargo audit --file src-tauri/Cargo.lock
npm run tauri build -- --bundles app
```

The regression tests cover navigation scheme/domain boundaries, deferred commands executing once,
command ordering and queue limits, state notification deduplication, infinite ad duration, and slow
page loading. Native launch, reopening, playback and remote pause must also be checked on macOS.
