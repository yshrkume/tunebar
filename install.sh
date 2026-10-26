#!/bin/sh
set -eu

PROJECT_DIR=$(CDPATH= cd "$(dirname "$0")" && pwd)
SOURCE="$PROJECT_DIR/src-tauri/target/release/bundle/macos/TuneBar.app"
DESTINATION=/Applications/TuneBar.app
BACKUP_ROOT="$HOME/Library/Application Support/TuneBar/Backups"

case "${1-}" in
  --dry-run)
    printf '%s\n' 'Would verify the built app, back up /Applications/TuneBar.app, and install the new bundle.'
    exit 0
    ;;
  '') ;;
  *) printf '%s\n' 'Usage: ./install.sh [--dry-run]' >&2; exit 2 ;;
esac

[ -d "$SOURCE" ] || { printf '%s\n' 'Build first: npm run tauri build -- --bundles app' >&2; exit 1; }
codesign --verify --deep --strict "$SOURCE"
if [ -d "$DESTINATION" ] && diff -qr "$SOURCE" "$DESTINATION" >/dev/null 2>&1; then
  printf '%s\n' 'TuneBar is already up to date.'
  exit 0
fi
if pgrep -x tunebar >/dev/null; then
  printf '%s\n' 'Quit TuneBar before installing.' >&2
  exit 1
fi

STAGING_DIR=$(mktemp -d /Applications/.tunebar-install.XXXXXX)
BACKUP=''
cleanup() {
  if [ ! -e "$DESTINATION" ] && [ -n "$BACKUP" ] && [ -d "$BACKUP" ]; then
    mv "$BACKUP" "$DESTINATION"
  fi
  rm -rf "$STAGING_DIR"
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM

ditto "$SOURCE" "$STAGING_DIR/TuneBar.app"
codesign --verify --deep --strict "$STAGING_DIR/TuneBar.app"
if [ -e "$DESTINATION" ]; then
  mkdir -p "$BACKUP_ROOT"
  BACKUP="$BACKUP_ROOT/TuneBar-$(date +%Y%m%d-%H%M%S).app"
  [ ! -e "$BACKUP" ] || { printf '%s\n' 'Backup already exists; retry in a moment.' >&2; exit 1; }
  mv "$DESTINATION" "$BACKUP"
fi
mv "$STAGING_DIR/TuneBar.app" "$DESTINATION"
printf '%s\n' 'Installed /Applications/TuneBar.app. The previous bundle is retained in ~/Library/Application Support/TuneBar/Backups.'
