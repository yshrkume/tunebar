// TuneBar Media Bridge - YouTube Music ↔ Rust IPC bridge
(function () {
  "use strict";

  // Utility: get the video element
  function getVideo() {
    return document.querySelector("video");
  }

  // Utility: get the player API
  function getPlayerApi() {
    const player = document.querySelector("#movie_player");
    if (player && typeof player.getPlayerState === "function") {
      return player;
    }
    return null;
  }

  // Utility: simulate click on a button by selector
  function clickButton(selector) {
    const btn = document.querySelector(selector);
    if (btn) {
      btn.click();
      return true;
    }
    return false;
  }

  let pendingRemoteCommand =
    typeof window.__TUNEBAR_PENDING_REMOTE_COMMAND__ === "string"
      ? window.__TUNEBAR_PENDING_REMOTE_COMMAND__
      : null;

  function normalizeRemoteCommand(command) {
    if (typeof command !== "string") return "";
    return command.trim().toLowerCase();
  }

  // Public bridge API
  window.__MUSIC_BRIDGE__ = {
    play() {
      const video = getVideo();
      if (video) video.play();
    },

    pause() {
      const video = getVideo();
      if (video) video.pause();
    },

    togglePlay() {
      const video = getVideo();
      if (!video) return;
      if (video.paused) {
        video.play();
      } else {
        video.pause();
      }
    },

    next() {
      // Use YouTube Music's next button
      clickButton(".next-button, .ytmusic-player-bar .next-button, tp-yt-paper-icon-button.next-button") ||
        clickButton('[aria-label="Next"]') ||
        clickButton(".ytp-next-button");
    },

    previous() {
      // Use YouTube Music's previous button
      clickButton(".previous-button, .ytmusic-player-bar .previous-button, tp-yt-paper-icon-button.previous-button") ||
        clickButton('[aria-label="Previous"]') ||
        clickButton(".ytp-prev-button");
    },

    getState() {
      const video = getVideo();
      const titleEl = document.querySelector(
        ".title.ytmusic-player-bar, .content-info-wrapper .title"
      );
      const artistEl = document.querySelector(
        ".byline.ytmusic-player-bar a, .content-info-wrapper .byline a, .subtitle .byline a"
      );

      return {
        playing: video ? !video.paused : false,
        title: titleEl?.textContent?.trim() || "",
        artist: artistEl?.textContent?.trim() || "",
        duration: video?.duration || 0,
        currentTime: video?.currentTime || 0,
      };
    },

    requestPiP() {
      const video = getVideo();
      if (video && document.pictureInPictureEnabled) {
        video.requestPictureInPicture().catch((e) => {
          console.warn("TuneBar: PiP request failed:", e);
        });
      }
    },

    exitPiP() {
      if (document.pictureInPictureElement) {
        document.exitPictureInPicture().catch(() => {});
      }
    },

    runCommand(command) {
      const normalized = normalizeRemoteCommand(command);
      if (!normalized) return true;

      const hasVideo = !!getVideo();
      const hasPlayer = !!getPlayerApi();
      if (!hasVideo && !hasPlayer) {
        pendingRemoteCommand = normalized;
        window.__TUNEBAR_PENDING_REMOTE_COMMAND__ = normalized;
        return false;
      }

      switch (normalized) {
        case "toggle":
          this.togglePlay();
          return true;
        case "play":
          this.play();
          return true;
        case "pause":
          this.pause();
          return true;
        case "next":
          this.next();
          return true;
        case "previous":
        case "prev":
          this.previous();
          return true;
        default:
          console.warn("TuneBar: Unknown remote command:", normalized);
          return true;
      }
    },
  };

  function flushPendingRemoteCommand() {
    if (!pendingRemoteCommand) return;
    const command = pendingRemoteCommand;
    pendingRemoteCommand = null;
    if (window.__MUSIC_BRIDGE__.runCommand(command)) {
      window.__TUNEBAR_PENDING_REMOTE_COMMAND__ = null;
      return;
    }
    pendingRemoteCommand = command;
  }

  // Track change detection via MutationObserver
  let lastTitle = "";
  let lastArtist = "";
  let lastPlaying = null;

  function checkTrackChange() {
    flushPendingRemoteCommand();
    const state = window.__MUSIC_BRIDGE__.getState();

    // Notify Rust on track change
    if (state.title && (state.title !== lastTitle || state.artist !== lastArtist)) {
      lastTitle = state.title;
      lastArtist = state.artist;

      if (window.__TAURI_INTERNALS__) {
        window.__TAURI_INTERNALS__.invoke("update_track_info", {
          title: state.title,
          artist: state.artist,
        });
      }
    }

    // Notify Rust on playback state change
    if (lastPlaying !== state.playing) {
      lastPlaying = state.playing;

      if (window.__TAURI_INTERNALS__) {
        window.__TAURI_INTERNALS__.invoke("update_playback_state", {
          playing: state.playing,
        });
      }
    }
  }

  // Observe player bar for track changes
  function startObserving() {
    flushPendingRemoteCommand();
    const playerBar =
      document.querySelector("ytmusic-player-bar") || document.body;

    const observer = new MutationObserver(() => {
      checkTrackChange();
    });

    observer.observe(playerBar, {
      childList: true,
      subtree: true,
      characterData: true,
    });

    // Also listen to video events
    const video = getVideo();
    if (video) {
      video.addEventListener("play", checkTrackChange);
      video.addEventListener("pause", checkTrackChange);
      video.addEventListener("loadedmetadata", checkTrackChange);
    }

    // Poll periodically as a fallback
    setInterval(checkTrackChange, 2000);
  }

  // Wait for the player bar to be ready
  function waitForPlayer() {
    const playerBar = document.querySelector("ytmusic-player-bar");
    if (playerBar) {
      startObserving();
      return;
    }

    const bodyObserver = new MutationObserver(() => {
      if (document.querySelector("ytmusic-player-bar")) {
        bodyObserver.disconnect();
        startObserving();
      }
    });

    bodyObserver.observe(document.body, {
      childList: true,
      subtree: true,
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", waitForPlayer);
  } else {
    waitForPlayer();
  }

  console.log("TuneBar: Media bridge initialized");
})();
