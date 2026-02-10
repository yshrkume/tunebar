// TuneBar Error Handler - Network error detection & retry UI
(function () {
  "use strict";

  const OVERLAY_ID = "tunebar-error-overlay";

  function createOverlay() {
    if (document.getElementById(OVERLAY_ID)) return;

    const overlay = document.createElement("div");
    overlay.id = OVERLAY_ID;
    overlay.innerHTML = `
      <div style="
        position: fixed;
        top: 0; left: 0; right: 0; bottom: 0;
        z-index: 999999;
        background: #030303;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        color: #fff;
      ">
        <div style="font-size: 48px; margin-bottom: 16px;">🎵</div>
        <h2 style="margin: 0 0 8px 0; font-size: 18px; font-weight: 600;">
          ネットワークに接続できません
        </h2>
        <p style="margin: 0 0 24px 0; font-size: 14px; color: #aaa;">
          インターネット接続を確認してください
        </p>
        <button id="tunebar-retry-btn" style="
          background: #fff;
          color: #030303;
          border: none;
          border-radius: 20px;
          padding: 10px 32px;
          font-size: 14px;
          font-weight: 600;
          cursor: pointer;
          transition: opacity 0.2s;
        ">再読み込み</button>
      </div>
    `;
    document.documentElement.appendChild(overlay);

    document.getElementById("tunebar-retry-btn").addEventListener("click", () => {
      location.reload();
    });
  }

  function removeOverlay() {
    const overlay = document.getElementById(OVERLAY_ID);
    if (overlay) overlay.remove();
  }

  // Offline/online events
  window.addEventListener("offline", () => {
    createOverlay();
  });

  window.addEventListener("online", () => {
    removeOverlay();
    location.reload();
  });

  // Check on load — if offline or page failed to load YouTube Music app
  function checkOnLoad() {
    if (!navigator.onLine) {
      createOverlay();
      return;
    }

    // Wait for YouTube Music SPA to initialize (give it a few seconds)
    setTimeout(() => {
      const ytApp = document.querySelector("ytmusic-app");
      if (!ytApp && !document.querySelector("video")) {
        // Page likely failed to load properly
        createOverlay();
      }
    }, 5000);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", checkOnLoad);
  } else {
    checkOnLoad();
  }

  console.log("TuneBar: Error handler initialized");
})();
