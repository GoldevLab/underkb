(() => {
  const boot = () => {
    const root = document.getElementById("ukb-app");
    if (!root || root.dataset.ready === "1") return;
    root.dataset.ready = "1";
    const tool = root.dataset.tool || "compress";
    const endpoint = root.dataset.endpoint || "/api/compress";
    const idleLabel = root.dataset.idle || "Compress";
    const busyText = root.dataset.busy || "Working…";
    const emptyMsg = root.dataset.empty || "Drop or choose an image first.";
    const form = root.querySelector("[data-form]");
    const drop = root.querySelector("[data-drop]");
    const input = root.querySelector("[data-file]");
    const nameEl = root.querySelector("[data-filename]");
    const kb = root.querySelector("[data-kb]");
    const err = root.querySelector("[data-error]");
    const result = root.querySelector("[data-result]");
    const submit = root.querySelector("[data-submit]");
    const warn = result?.querySelector("[data-warn]");
    const gifNote = root.querySelector("[data-gif]");
    const formatHint = root.querySelector("[data-format-hint]");
    const formatEl = root.querySelector("[data-format]");
    const dropPrev = root.querySelector("[data-drop-preview]");
    const dimsEl = root.querySelector("[data-dims]");
    const swatches = result?.querySelector("[data-swatches]");
    const download = result?.querySelector("[data-download]");
    let file = null;
    let beforeUrl = "";
    let origW = 0;
    let origH = 0;
    let abortCtl = null;
    let previewGen = 0;
    let lastHex = [];

    const csrfHeaders = () => {
      const headers = {};
      try {
        const tok = JSON.parse(document.getElementById("resuma-state")?.textContent || "{}").csrf_token;
        if (tok) headers["x-resuma-csrf"] = tok;
      } catch (_) {}
      return headers;
    };
    const setError = (t) => {
      if (!err) return;
      err.textContent = t || "";
      err.hidden = !t;
    };
    const human = (n) => {
      if (n < 1024) return n + " B";
      if (n < 1024 * 1024) return Math.round(n / 1024) + " KB";
      return (n / (1024 * 1024)).toFixed(2) + " MB";
    };
    const kindOf = (f) => {
      const t = (f.type || "").toLowerCase();
      const name = f.name || "";
      if (t === "image/svg+xml" || t === "image/svg" || /\.svg$/i.test(name)) return "svg";
      if (t === "image/heic" || t === "image/heif" || /\.hei[cf]$/i.test(name)) return "heic";
      if (t === "image/avif" || /\.avif$/i.test(name)) return "avif";
      if (t === "image/gif" || /\.gif$/i.test(name)) return "gif";
      if (t === "image/jpeg" || t === "image/jpg" || t === "image/pjpeg" || /\.jpe?g$/i.test(name)) return "jpeg";
      if (t === "image/png" || /\.png$/i.test(name)) return "png";
      if (t === "image/webp" || /\.webp$/i.test(name)) return "webp";
      if (t.startsWith("image/")) return "unsupported";
      if (/\.jpe?g$/i.test(name)) return "jpeg";
      if (/\.png$/i.test(name)) return "png";
      if (/\.webp$/i.test(name)) return "webp";
      if (/\.gif$/i.test(name)) return "gif";
      return "no";
    };
    const isPhotoKind = (kind) => kind === "jpeg" || kind === "heic" || kind === "avif";
    const syncFormatHint = () => {
      if (!formatHint) return;
      const fmt = formatEl?.value || "jpeg";
      const photo = file && isPhotoKind(kindOf(file));
      formatHint.hidden = !(tool === "compress" && photo && (fmt === "png" || fmt === "webp"));
    };
    const throwIfAborted = (signal) => {
      if (signal?.aborted) throw new DOMException("Aborted", "AbortError");
    };
    const bitmapFrom = async (f) => {
      try {
        return { bitmap: await createImageBitmap(f, { imageOrientation: "from-image" }), oriented: true };
      } catch (_) {
        return { bitmap: await createImageBitmap(f), oriented: false };
      }
    };
    const toBlob = (canvas, mime, q) => new Promise((res) => canvas.toBlob(res, mime, q));
    const canvasPreviewBlob = async (bitmap) => {
      const c = document.createElement("canvas");
      c.width = Math.max(1, bitmap.width);
      c.height = Math.max(1, bitmap.height);
      const ctx = c.getContext("2d");
      if (!ctx) return null;
      ctx.fillStyle = "#fff";
      ctx.fillRect(0, 0, c.width, c.height);
      ctx.drawImage(bitmap, 0, 0);
      return toBlob(c, "image/jpeg", 0.85);
    };
    const prepare = async (f, outFmt, signal) => {
      throwIfAborted(signal);
      const kind = kindOf(f);
      if (kind === "svg") throw new Error("SVG is not supported. Export a PNG or JPG first.");
      if (kind === "unsupported") throw new Error("Use JPG, PNG, WebP, GIF, or HEIC.");
      if (kind === "no") throw new Error("Choose a JPG, PNG, WebP, GIF, or HEIC.");
      if (f.size > 20 * 1024 * 1024) throw new Error("That file is over 20 MB.");
      let loaded;
      try {
        loaded = await bitmapFrom(f);
      } catch (_) {
        if (kind === "heic" || kind === "avif") {
          throw new Error("This browser cannot read HEIC/AVIF. Share the photo as JPG, or try Safari.");
        }
        origW = 0;
        origH = 0;
        return f;
      }
      throwIfAborted(signal);
      const { bitmap, oriented } = loaded;
      origW = bitmap.width;
      origH = bitmap.height;
      const MAX_EDGE = 2560;
      const MAX_PX = 12_000_000;
      const SERVER_EDGE = 4096;
      const SERVER_PX = 12_000_000;
      const w = bitmap.width;
      const h = bitmap.height;
      const scale = Math.min(1, MAX_EDGE / Math.max(w, h), Math.sqrt(MAX_PX / Math.max(1, w * h)));
      const needScale = scale < 0.999;
      const tooBigForServer = Math.max(w, h) > SERVER_EDGE || w * h > SERVER_PX;
      const wantScale = needScale || f.size > 6 * 1024 * 1024;
      const mustCanvas = kind === "heic" || kind === "avif";
      const needReencode = mustCanvas || (wantScale && (oriented || tooBigForServer));
      if (!needReencode) {
        bitmap.close?.();
        return f;
      }
      await new Promise((r) => setTimeout(r, 0));
      throwIfAborted(signal);
      const cw = Math.max(1, Math.round(w * scale));
      const ch = Math.max(1, Math.round(h * scale));
      const canvas = document.createElement("canvas");
      canvas.width = cw;
      canvas.height = ch;
      const ctx = canvas.getContext("2d");
      if (!ctx) {
        bitmap.close?.();
        return f;
      }
      const keepAlpha = outFmt === "png" || outFmt === "webp" || tool === "removebg";
      if (!keepAlpha) {
        ctx.fillStyle = "#fff";
        ctx.fillRect(0, 0, cw, ch);
      }
      ctx.drawImage(bitmap, 0, 0, cw, ch);
      bitmap.close?.();
      throwIfAborted(signal);
      let mime = "image/jpeg";
      let ext = ".jpg";
      if (keepAlpha && outFmt === "png") { mime = "image/png"; ext = ".png"; }
      else if (keepAlpha) { mime = "image/webp"; ext = ".webp"; }
      let blob = await toBlob(canvas, mime, 0.92);
      if ((!blob || !blob.size) && mime !== "image/jpeg") {
        blob = await toBlob(canvas, "image/jpeg", 0.92);
        ext = ".jpg";
        mime = "image/jpeg";
      }
      if (!blob) return f;
      const stem = (f.name || "image").replace(/\.[^.]+$/, "");
      return new File([blob], stem + ext, { type: mime });
    };
    const pick = (f) => {
      if (!f) return;
      const kind = kindOf(f);
      if (kind === "svg") { setError("SVG is not supported. Export a PNG or JPG first."); return; }
      if (kind === "unsupported") { setError("Use JPG, PNG, WebP, GIF, or HEIC."); return; }
      if (kind === "no") { setError("Choose a JPG, PNG, WebP, GIF, or HEIC."); return; }
      if (f.size > 20 * 1024 * 1024) { setError("That file is over 20 MB."); return; }
      file = f;
      if (beforeUrl) { URL.revokeObjectURL(beforeUrl); beforeUrl = ""; }
      if (dropPrev) { dropPrev.removeAttribute("src"); dropPrev.hidden = true; }
      if (gifNote) gifNote.hidden = kind !== "gif";
      if (input && f !== input.files?.[0]) {
        try {
          const dt = new DataTransfer();
          dt.items.add(f);
          input.files = dt.files;
        } catch (_) {}
      }
      if (nameEl) nameEl.textContent = f.name + " · " + human(f.size);
      drop?.classList.add("has-file");
      setError("");
      syncFormatHint();
      syncPresets();
      const gen = ++previewGen;
      (async () => {
        try {
          const { bitmap } = await bitmapFrom(f);
          if (gen !== previewGen) { bitmap.close?.(); return; }
          origW = bitmap.width;
          origH = bitmap.height;
          if (dimsEl) {
            dimsEl.hidden = false;
            dimsEl.textContent = origW + " × " + origH + " px";
          }
          let url = "";
          if (kind === "heic" || kind === "avif") {
            const blob = await canvasPreviewBlob(bitmap);
            if (blob) url = URL.createObjectURL(blob);
          }
          bitmap.close?.();
          if (gen !== previewGen) {
            if (url) URL.revokeObjectURL(url);
            return;
          }
          if (!url) url = URL.createObjectURL(f);
          if (beforeUrl) URL.revokeObjectURL(beforeUrl);
          beforeUrl = url;
          if (dropPrev && url) {
            dropPrev.src = url;
            dropPrev.hidden = false;
          }
        } catch (_) {
          if (gen !== previewGen) return;
          try { beforeUrl = URL.createObjectURL(f); } catch (__) { beforeUrl = ""; }
          if (dropPrev && beforeUrl) {
            dropPrev.src = beforeUrl;
            dropPrev.hidden = false;
          }
        }
      })();
    };
    const maxTargetKb = () => {
      if (!file) return 5120;
      return Math.max(8, Math.floor((file.size - 1) / 1024));
    };
    const syncPresets = () => {
      const cap = maxTargetKb();
      const buttons = root.querySelectorAll("[data-preset]");
      buttons.forEach((b) => {
        const n = Number(b.getAttribute("data-preset"));
        const tooBig = !!file && Number.isFinite(n) && n * 1024 >= file.size;
        b.hidden = tooBig;
      });
      if (kb) {
        kb.max = String(cap);
        let v = Number(kb.value || 200);
        if (!Number.isFinite(v) || v > cap) {
          const allowed = [...buttons]
            .filter((b) => !b.hidden)
            .map((b) => Number(b.getAttribute("data-preset")))
            .filter((n) => Number.isFinite(n) && n <= cap);
          kb.value = String(allowed.length ? Math.max(...allowed) : cap);
        }
      }
      const on = String(kb?.value || "200");
      buttons.forEach((b) => {
        b.classList.toggle("is-on", !b.hidden && b.getAttribute("data-preset") === on);
      });
    };
    const busy = (on, label) => {
      form?.setAttribute("aria-busy", on ? "true" : "false");
      if (submit) {
        submit.disabled = on;
        if (label) submit.textContent = label;
      }
    };
    const paintSwatches = (colors) => {
      if (!swatches) return;
      swatches.innerHTML = "";
      lastHex = (colors || []).map((c) => c.hex).filter(Boolean);
      if (!lastHex.length) {
        swatches.hidden = true;
        return;
      }
      swatches.hidden = false;
      colors.forEach((c) => {
        const btn = document.createElement("button");
        btn.type = "button";
        btn.className = "swatch";
        btn.title = "Copiar " + c.hex;
        const chip = document.createElement("span");
        chip.className = "swatch-chip";
        chip.style.background = c.hex;
        const meta = document.createElement("span");
        meta.className = "swatch-meta";
        const strong = document.createElement("strong");
        strong.textContent = c.hex;
        meta.append(strong, document.createElement("br"), document.createTextNode(
          typeof c.pct === "number" ? c.pct.toFixed(1) + "%" : ""
        ));
        btn.append(chip, meta);
        btn.addEventListener("click", async () => {
          try { await navigator.clipboard.writeText(c.hex); } catch (_) {}
        });
        swatches.appendChild(btn);
      });
    };

    ["dragenter", "dragover"].forEach((ev) => drop?.addEventListener(ev, (e) => {
      e.preventDefault();
      drop.classList.add("is-over");
    }));
    drop?.addEventListener("dragleave", (e) => {
      e.preventDefault();
      if (!drop.contains(e.relatedTarget)) drop.classList.remove("is-over");
    });
    drop?.addEventListener("drop", (e) => {
      e.preventDefault();
      drop.classList.remove("is-over");
      pick(e.dataTransfer?.files?.[0]);
    });
    input?.addEventListener("change", () => pick(input.files?.[0]));
    if (input?.files?.[0]) pick(input.files[0]);
    kb?.addEventListener("input", syncPresets);
    formatEl?.addEventListener("change", syncFormatHint);
    root.querySelectorAll("[data-preset]").forEach((btn) => {
      btn.addEventListener("click", () => {
        if (btn.hidden) return;
        if (kb) kb.value = btn.getAttribute("data-preset") || "200";
        syncPresets();
      });
    });
    download?.addEventListener("click", async (e) => {
      if (tool !== "colors") return;
      e.preventDefault();
      if (!lastHex.length) return;
      try { await navigator.clipboard.writeText(lastHex.join("\n")); } catch (_) {}
    });

    form?.addEventListener("submit", async (e) => {
      e.preventDefault();
      file = file || input?.files?.[0] || null;
      if (!file) { setError(emptyMsg); return; }
      const fmt = formatEl?.value || (tool === "convert" ? "webp" : tool === "removebg" ? "png" : "jpeg");
      abortCtl?.abort();
      abortCtl = new AbortController();
      busy(true, busyText);
      setError("");
      if (result) result.hidden = true;
      if (warn) { warn.hidden = true; warn.textContent = ""; }
      if (swatches) { swatches.hidden = true; swatches.innerHTML = ""; }
      try {
        const ready = await prepare(file, fmt, abortCtl.signal);
        const body = new FormData();
        body.append("file", ready);
        body.append("orig_bytes", String(file.size));
        if (origW) body.append("orig_width", String(origW));
        if (origH) body.append("orig_height", String(origH));
        if (tool === "compress") {
          let target = Number(kb?.value || 200);
          if (!Number.isFinite(target)) target = 200;
          if (target < 8) target = 8;
          if (target > 5120) target = 5120;
          const cap = maxTargetKb();
          if (target > cap) target = cap;
          if (kb && Number(kb.value) !== target) kb.value = String(target);
          syncPresets();
          body.append("target_kb", String(target));
          body.append("format", fmt);
        } else if (tool === "convert") {
          body.append("format", fmt);
          body.append("quality", String(root.querySelector("[data-quality]")?.value || "80"));
        } else if (tool === "resize") {
          const w = (root.querySelector("[data-width]")?.value || "").trim();
          const h = (root.querySelector("[data-height]")?.value || "").trim();
          if (w) body.append("width", w);
          if (h) body.append("height", h);
          if (!w && !h) throw new Error("Pon un ancho, un alto, o ambos.");
          body.append("mode", root.querySelector("[data-mode]")?.value || "fit");
          body.append("format", fmt);
        } else if (tool === "removebg") {
          body.append("tolerance", String(root.querySelector("[data-tolerance]")?.value || "32"));
          body.append("format", fmt);
        } else if (tool === "colors") {
          body.append("count", String(root.querySelector("[data-count]")?.value || "6"));
        }
        const r = await fetch(endpoint, {
          method: "POST",
          body,
          signal: abortCtl.signal,
          headers: csrfHeaders(),
        });
        const text = await r.text();
        let data;
        try { data = JSON.parse(text); }
        catch (_) {
          throw new Error(r.status === 413 ? "File is too large for the server." : r.status === 429 ? "Too many requests. Wait a minute." : "Request failed.");
        }
        if (!data.ok) throw new Error(data.error || "Request failed.");
        const before = result?.querySelector("[data-before]");
        const img = result?.querySelector("[data-preview]");
        const link = download;
        if (before && beforeUrl) {
          before.src = beforeUrl;
          before.width = data.original_width || origW || 1;
          before.height = data.original_height || origH || 1;
          before.alt = "Original";
        }
        if (tool === "colors") {
          if (!(data.colors || []).length) throw new Error("No se encontraron colores.");
          paintSwatches(data.colors || []);
          if (img) img.removeAttribute("src");
          if (link) {
            link.removeAttribute("href");
            link.setAttribute("download", "");
          }
          const nameOut = result?.querySelector("[data-out-name]");
          const stats = result?.querySelector("[data-stats]");
          if (nameOut) nameOut.textContent = (data.colors || []).map((c) => c.hex).join("  ");
          if (stats) stats.textContent = (data.width || origW) + "×" + (data.height || origH);
        } else {
          if (img) {
            img.src = data.url;
            img.width = data.width || 1;
            img.height = data.height || 1;
            img.alt = data.filename || "Preview";
            img.classList.toggle("is-alpha", tool === "removebg" || data.format === "png" || data.format === "webp");
          }
          if (link) {
            link.href = data.url + (data.url.includes("?") ? "&" : "?") + "dl=1";
            link.setAttribute("download", data.filename);
          }
          const nameOut = result?.querySelector("[data-out-name]");
          const stats = result?.querySelector("[data-stats]");
          if (nameOut) nameOut.textContent = data.filename;
          if (stats) stats.textContent =
            human(data.original_bytes) + " → " + human(data.result_bytes)
            + " · " + data.original_width + "×" + data.original_height
            + " → " + data.width + "×" + data.height;
          if (warn) {
            if (data.over_budget) {
              warn.textContent = "Could not reach " + (data.target_kb || "") + " KB. This is the closest size — try JPG or a larger budget.";
              warn.hidden = false;
            } else {
              warn.hidden = true;
            }
          }
        }
        if (result) result.hidden = false;
      } catch (ex) {
        if (ex.name === "AbortError") return;
        setError(ex.message || "Request failed.");
      } finally {
        busy(false, idleLabel);
      }
    });
  };

  const tryBoot = () => boot();
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", tryBoot, { once: true });
  } else {
    tryBoot();
  }
  document.addEventListener("resuma:navigate", () => {
    queueMicrotask(tryBoot);
    requestAnimationFrame(tryBoot);
  });
  new MutationObserver(tryBoot).observe(document.documentElement, { childList: true, subtree: true });
})();
