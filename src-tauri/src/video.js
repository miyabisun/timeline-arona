// Timeline Arona's helper in the YouTube page of the video window. browser.rs
// calls these through ExecuteScript and polls what takes time; the page gets no
// way to call the app, and nothing here sends anything anywhere.
(() => {
  if (window.__arona) return;

  const player = () => document.querySelector("#movie_player");
  const video = () => document.querySelector("video.html5-main-video") ?? document.querySelector("video");
  const ad = () => Boolean(player()?.classList.contains("ad-showing"));

  /** Gets on past an ad: muted, playing, skipped as soon as it can be. */
  function passAd() {
    const v = video();
    if (v) {
      v.muted = true;
      if (v.paused) v.play().catch(() => {});
    }
    document.querySelector(".ytp-skip-ad-button, .ytp-ad-skip-button-modern, .ytp-ad-skip-button")?.click();
  }

  /** Whether the video can be read; once it can, what the video is. */
  function info() {
    if (ad()) {
      passAd();
      return { ready: false, ad: true };
    }
    const v = video();
    if (!v || v.readyState < 1 || !Number.isFinite(v.duration)) return { ready: false, ad: false };
    v.muted = true;
    const p = player();
    const best = p?.getAvailableQualityLevels?.()[0];
    if (best && best !== "auto") p.setPlaybackQualityRange?.(best, best);
    const details = p?.getPlayerResponse?.()?.videoDetails ?? window.ytInitialPlayerResponse?.videoDetails ?? {};
    return {
      ready: true,
      ad: false,
      page: location.href,
      title: details.title ?? document.title,
      channel: details.author ?? "",
      duration: v.duration,
      description: details.shortDescription ?? "",
    };
  }

  let wanted = 0;

  /** Starts going to `time` (seconds); `settled` says when the picture is there. */
  function seek(time) {
    const v = video();
    if (!v) return false;
    wanted = time;
    v.pause();
    v.currentTime = time;
    return true;
  }

  function settled() {
    if (ad()) {
      passAd();
      return false;
    }
    const v = video();
    if (!v || v.seeking || v.readyState < 2) return false;
    // An ad or the player itself may have moved the video meanwhile.
    if (Math.abs(v.currentTime - wanted) > 0.5) {
      seek(wanted);
      return false;
    }
    if (!v.paused) v.pause();
    return true;
  }

  const THUMB = { w: 64, h: 36 };

  /** The picture now shown, cut to `crop` (fractions), as a PNG data URL and a grey thumbnail. */
  function grab(crop) {
    try {
      const v = video();
      const sx = Math.round(crop.x * v.videoWidth);
      const sy = Math.round(crop.y * v.videoHeight);
      const sw = Math.max(1, Math.round(crop.w * v.videoWidth));
      const sh = Math.max(1, Math.round(crop.h * v.videoHeight));
      const canvas = document.createElement("canvas");
      canvas.width = sw;
      canvas.height = sh;
      canvas.getContext("2d").drawImage(v, sx, sy, sw, sh, 0, 0, sw, sh);
      const small = document.createElement("canvas");
      small.width = THUMB.w;
      small.height = THUMB.h;
      const context = small.getContext("2d", { willReadFrequently: true });
      context.drawImage(canvas, 0, 0, THUMB.w, THUMB.h);
      const rgba = context.getImageData(0, 0, THUMB.w, THUMB.h).data;
      const thumb = [];
      for (let i = 0; i < rgba.length; i += 4) {
        thumb.push(Math.round((rgba[i] * 299 + rgba[i + 1] * 587 + rgba[i + 2] * 114) / 1000));
      }
      return {
        time: v.currentTime,
        width: v.videoWidth,
        height: v.videoHeight,
        png: canvas.toDataURL("image/png"),
        thumb,
      };
    } catch (error) {
      return { error: String(error) };
    }
  }

  window.__arona = { info, seek, settled, grab };
})();
