// TuneBar Ad Blocker - CSS-based ad hiding + JS video ad skipping
(function () {
  "use strict";

  // CSS-based ad element hiding (more stable than JS DOM manipulation)
  const style = document.createElement("style");
  style.textContent = `
    /* Banner ads */
    ytmusic-mealbar-promo-renderer,
    ytmusic-statement-banner-renderer,
    ytmusic-brand-promo-renderer,
    /* Video ads overlay */
    .ytp-ad-module,
    .ytp-ad-overlay-container,
    .ytp-ad-text-overlay,
    .ytp-ad-skip-button-container,
    .ytp-ad-player-overlay,
    .ytp-ad-image-overlay,
    /* Ad badges */
    .ytd-ad-slot-renderer,
    .ytmusic-ad-badge,
    /* Premium upsell */
    ytmusic-popup-container[dialog-type="PREMIUM_UPSELL"],
    tp-yt-paper-dialog:has(ytmusic-premium-upsell-dialog-renderer),
    /* Masthead ads */
    #masthead-ad,
    .ytd-primetime-promo-renderer,
    /* "Browser not supported" banner (shown for Safari UA) */
    ytmusic-you-there-renderer,
    [class*="browser-not-supported"],
    [class*="unsupported-browser"],
    .ytmusic-mealbar-promo-renderer[data-type="BROWSER_UNSUPPORTED"],
    paper-dialog:has([icon="yt-icons:chrome"]),
    tp-yt-paper-dialog:has(.promo-body) {
      display: none !important;
    }
  `;
  document.documentElement.appendChild(style);

  // Video ad skipper - detects and skips video ads
  function skipVideoAds() {
    const video = document.querySelector("video");
    if (!video) return;

    // Check for ad indicators
    const adPlaying =
      document.querySelector(".ad-showing") ||
      document.querySelector(".ytp-ad-player-overlay") ||
      document.querySelector('[class*="ad-interrupting"]');

    if (adPlaying) {
      // Skip to end of ad
      video.currentTime = video.duration || 0;
      video.playbackRate = 16;

      // Click skip button if available
      const skipBtn =
        document.querySelector(".ytp-ad-skip-button") ||
        document.querySelector(".ytp-ad-skip-button-modern") ||
        document.querySelector('[class*="skip-button"]');
      if (skipBtn) {
        skipBtn.click();
      }
    }
  }

  // Run ad skipper periodically
  setInterval(skipVideoAds, 500);

  // Also observe DOM changes for ad insertion
  const observer = new MutationObserver(() => {
    skipVideoAds();
  });

  // Start observing once the player is ready
  function observePlayer() {
    const player = document.querySelector("#movie_player") || document.body;
    observer.observe(player, {
      childList: true,
      subtree: true,
      attributes: true,
      attributeFilter: ["class"],
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", observePlayer);
  } else {
    observePlayer();
  }
})();
