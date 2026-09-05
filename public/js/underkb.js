(() => {
  let disposeTool = null;
  const boot = () => {
    const root = document.getElementById("ukb-app");
    if (!root) {
      disposeTool?.();
      return;
    }
    if (root.dataset.ready === "1") return;
    disposeTool?.();
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
    let picked = [];
    const shareBtn = result?.querySelector("[data-share]");
    let beforeUrl = "";
    let origW = 0;
    let origH = 0;
    let abortCtl = null;
    let previewGen = 0;
    let submitGen = 0;
    let lastHex = [];

    const csrfHeaders = () => {
      const headers = {};
      try {
        const tok = JSON.parse(document.getElementById("resuma-state")?.textContent || "{}").csrf_token;
        if (tok) headers["x-resuma-csrf"] = tok;
      } catch (_) {}
      return headers;
    };
    const captureProKey = () => {
      try {
        const u = new URL(location.href);
        const key = (u.searchParams.get("key") || "").trim();
        if (key.length >= 16) {
          localStorage.setItem("ukb_pro", key);
          document.cookie = "ukb_pro=" + encodeURIComponent(key) + "; Path=/; Max-Age=31536000; SameSite=Lax";
          u.searchParams.delete("key");
          history.replaceState({}, "", u.pathname + u.search + u.hash);
        }
      } catch (_) {}
    };
    captureProKey();
    const proKey = () => {
      try {
        return (localStorage.getItem("ukb_pro") || "").trim();
      } catch (_) {
        return "";
      }
    };
    const authHeaders = () => {
      const headers = csrfHeaders();
      const k = proKey();
      if (k) headers["X-Api-Key"] = k;
      return headers;
    };
    const maxUpload = () => (proKey() ? 50 : 20) * 1024 * 1024;
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
      formatHint.hidden = !(tool === "compress" && photo && fmt === "png");
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
      if (f.size > maxUpload()) throw new Error("That file is over " + (maxUpload() / (1024 * 1024)) + " MB.");
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
      const needReencode = mustCanvas
        || tooBigForServer
        || (tool !== "colors" && wantScale && oriented);
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
      const keepAlpha = outFmt === "png" || outFmt === "webp" || tool === "removebg" || tool === "colors";
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
      if (abortCtl) {
        abortCtl.abort();
        abortCtl = null;
        submitGen += 1;
        busy(false, idleLabel);
      }
      const kind = kindOf(f);
      if (kind === "svg") { setError("SVG is not supported. Export a PNG or JPG first."); return; }
      if (kind === "unsupported") { setError("Use JPG, PNG, WebP, GIF, or HEIC."); return; }
      if (kind === "no") { setError("Choose a JPG, PNG, WebP, GIF, or HEIC."); return; }
      if (f.size > maxUpload()) { setError("That file is over " + (maxUpload() / (1024 * 1024)) + " MB."); return; }
      file = f;
      picked = [f];
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
        btn.title = "Copy " + c.hex;
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
    const pickList = (list) => {
      const arr = Array.from(list || []).filter(Boolean);
      if (!arr.length) return;
      if (tool === "compress" && arr.length > 1) {
        if (!proKey()) {
          setError("Free is one file. A Pro key on /pricing allows a ZIP of up to 20.");
          pick(arr[0]);
          return;
        }
        if (arr.length > 20) {
          setError("Max 20 files per ZIP.");
          return;
        }
        const cap = maxUpload();
        const heavy = arr.find((f) => f.size > cap);
        if (heavy) {
          setError(heavy.name + " is over " + (cap / (1024 * 1024)) + " MB.");
          return;
        }
        const total = arr.reduce((s, f) => s + f.size, 0);
        if (total > 50 * 1024 * 1024) {
          setError("Batch total is over 50 MB.");
          return;
        }
        pick(arr[0]);
        picked = arr;
        if (nameEl) nameEl.textContent = arr.length + " files · " + human(total);
        if (gifNote) gifNote.hidden = !arr.some((f) => kindOf(f) === "gif");
        return;
      }
      pick(arr[0]);
    };
    drop?.addEventListener("drop", (e) => {
      e.preventDefault();
      drop.classList.remove("is-over");
      pickList(e.dataTransfer?.files);
    });
    input?.addEventListener("change", () => pickList(input.files));
    if (input?.files?.length) pickList(input.files);
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

    const offerShare = async (url, filename, mime) => {
      if (!shareBtn) return;
      shareBtn.hidden = true;
      shareBtn.onclick = null;
      if (tool === "colors" || !url || !navigator.canShare || !navigator.share) return;
      try {
        const res = await fetch(url);
        const blob = await res.blob();
        const f = new File([blob], filename || "underkb.bin", { type: mime || blob.type || "application/octet-stream" });
        if (!navigator.canShare({ files: [f] })) return;
        shareBtn.hidden = false;
        shareBtn.onclick = async (ev) => {
          ev.preventDefault();
          try { await navigator.share({ files: [f], title: filename }); } catch (_) {}
        };
      } catch (_) {}
    };

    form?.addEventListener("submit", async (e) => {
      e.preventDefault();
      const batch = tool === "compress" && picked.length > 1;
      const src = file || input?.files?.[0] || null;
      if (!batch && !src) { setError(emptyMsg); return; }
      if (src) file = src;
      const fmt = formatEl?.value || (tool === "convert" ? "webp" : tool === "removebg" ? "png" : "jpeg");
      const gen = ++submitGen;
      abortCtl?.abort();
      abortCtl = new AbortController();
      busy(true, busyText);
      setError("");
      if (result) result.hidden = true;
      if (warn) { warn.hidden = true; warn.textContent = ""; }
      if (swatches) { swatches.hidden = true; swatches.innerHTML = ""; }
      if (shareBtn) { shareBtn.hidden = true; shareBtn.onclick = null; }
      try {
        const body = new FormData();
        let postTo = endpoint;
        if (batch) {
          if (!proKey()) throw new Error("Free is one file. A Pro key on /pricing allows a ZIP of up to 20.");
          postTo = "/api/compress-batch";
          let target = Number(kb?.value || 200);
          if (!Number.isFinite(target)) target = 200;
          if (target < 8) target = 8;
          if (target > 5120) target = 5120;
          if (kb && Number(kb.value) !== target) kb.value = String(target);
          for (const f of picked) {
            const ready = await prepare(f, fmt, abortCtl.signal);
            body.append("file", ready);
          }
          body.append("target_kb", String(target));
          body.append("format", fmt);
        } else {
          const ready = await prepare(src, fmt, abortCtl.signal);
          body.append("file", ready);
          body.append("orig_bytes", String(src.size));
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
            if (!w && !h) throw new Error("Enter a width, a height, or both.");
            body.append("mode", root.querySelector("[data-mode]")?.value || "fit");
            body.append("format", fmt);
          } else if (tool === "removebg") {
            body.append("tolerance", String(root.querySelector("[data-tolerance]")?.value || "32"));
            body.append("format", fmt);
          } else if (tool === "colors") {
            body.append("count", String(root.querySelector("[data-count]")?.value || "6"));
          }
        }
        const r = await fetch(postTo, {
          method: "POST",
          body,
          signal: abortCtl.signal,
          headers: authHeaders(),
        });
        const text = await r.text();
        let data;
        try { data = JSON.parse(text); }
        catch (_) {
          throw new Error(r.status === 413 ? "File is too large for the server." : r.status === 402 ? "A Pro key is required for batch ZIP. See /pricing." : r.status === 429 ? "Too many requests. Wait a minute." : "Request failed.");
        }
        if (!data.ok) throw new Error(data.error || "Request failed.");
        const before = result?.querySelector("[data-before]");
        const img = result?.querySelector("[data-preview]");
        const link = download;
        const isZip = data.format === "zip";
        if (before && beforeUrl && !isZip) {
          before.src = beforeUrl;
          before.width = data.original_width || origW || 1;
          before.height = data.original_height || origH || 1;
          before.alt = "Original";
        } else if (before && isZip) {
          before.removeAttribute("src");
        }
        if (tool === "colors") {
          if (!(data.colors || []).length) throw new Error("No colors found.");
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
        } else if (isZip) {
          if (img) img.removeAttribute("src");
          if (link) {
            link.href = data.url + (data.url.includes("?") ? "&" : "?") + "dl=1";
            link.setAttribute("download", data.filename);
          }
          const nameOut = result?.querySelector("[data-out-name]");
          const stats = result?.querySelector("[data-stats]");
          if (nameOut) nameOut.textContent = data.filename;
          if (stats) stats.textContent = human(data.original_bytes) + " → " + human(data.result_bytes) + " ZIP";
          if (warn) {
            if (data.over_budget) {
              warn.textContent = "At least one file missed " + (data.target_kb || "") + " KB. The ZIP still has the closest sizes.";
              warn.hidden = false;
            } else {
              warn.hidden = true;
            }
          }
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
        if (data.url && tool !== "colors") {
          await offerShare(data.url, data.filename, data.mime);
        }
      } catch (ex) {
        if (ex.name === "AbortError" || gen !== submitGen) return;
        setError(ex.message || "Request failed.");
      } finally {
        if (gen === submitGen) busy(false, idleLabel);
      }
    });

    disposeTool = () => {
      abortCtl?.abort();
      abortCtl = null;
      submitGen += 1;
      previewGen += 1;
      if (beforeUrl) URL.revokeObjectURL(beforeUrl);
      beforeUrl = "";
      delete root.dataset.ready;
      disposeTool = null;
    };
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

  let stopHero = null;
  const mountHeroParticles = () => {
    if (typeof stopHero === "function") {
      stopHero();
      stopHero = null;
    }
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    const root = document.querySelector("[data-hero-particles]");
    if (!root) return;
    const canvas = document.createElement("canvas");
    canvas.className = "hero-particles-canvas";
    canvas.setAttribute("aria-hidden", "true");
    root.replaceChildren(canvas);
    const ctx = canvas.getContext("2d", { alpha: true });
    if (!ctx) return;
    const COUNT = 160;
    const dots = Array.from({ length: COUNT }, () => ({
      x: Math.random(),
      y: Math.random(),
      z: Math.random(),
      s: 0.4 + Math.random() * 1.4,
      p: Math.random() * Math.PI * 2,
    }));
    let w = 0;
    let h = 0;
    let pointerX = 0;
    let pointerY = 0;
    let raf = 0;
    let accent = "#4ade80";
    const readAccent = () => {
      const cs = getComputedStyle(document.documentElement);
      accent = (cs.getPropertyValue("--accent") || "#4ade80").trim() || "#4ade80";
    };
    const resize = () => {
      const dpr = Math.min(window.devicePixelRatio || 1, 2);
      w = root.clientWidth;
      h = root.clientHeight;
      if (!w || !h) return;
      canvas.width = Math.floor(w * dpr);
      canvas.height = Math.floor(h * dpr);
      canvas.style.width = w + "px";
      canvas.style.height = h + "px";
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    };
    const onMove = (e) => {
      const r = root.getBoundingClientRect();
      if (!r.width || !r.height) return;
      pointerX = (e.clientX - r.left) / r.width - 0.5;
      pointerY = (e.clientY - r.top) / r.height - 0.5;
    };
    const tick = (t) => {
      if (!root.isConnected) {
        if (typeof stopHero === "function") stopHero();
        return;
      }
      if (document.hidden) {
        raf = 0;
        return;
      }
      const time = t * 0.001;
      ctx.clearRect(0, 0, w, h);
      ctx.fillStyle = accent;
      for (const d of dots) {
        d.p += 0.002 * d.s;
        const x = (d.x + Math.sin(time * d.s + d.p) * 0.03 + pointerX * 0.04 * d.z) * w;
        const y = (d.y + Math.cos(time * 0.7 * d.s + d.p) * 0.025 + pointerY * 0.03 * d.z) * h;
        ctx.globalAlpha = 0.18 + d.z * 0.35;
        ctx.beginPath();
        ctx.arc(x, y, 0.6 + d.z * 1.6, 0, Math.PI * 2);
        ctx.fill();
      }
      raf = requestAnimationFrame(tick);
    };
    const onVis = () => {
      if (!document.hidden && !raf && root.isConnected) raf = requestAnimationFrame(tick);
    };
    const themeWatch = new MutationObserver(readAccent);
    themeWatch.observe(document.documentElement, { attributes: true, attributeFilter: ["data-theme"] });
    readAccent();
    resize();
    window.addEventListener("resize", resize, { passive: true });
    window.addEventListener("pointermove", onMove, { passive: true });
    document.addEventListener("visibilitychange", onVis);
    raf = requestAnimationFrame(tick);
    stopHero = () => {
      cancelAnimationFrame(raf);
      raf = 0;
      themeWatch.disconnect();
      window.removeEventListener("resize", resize);
      window.removeEventListener("pointermove", onMove);
      document.removeEventListener("visibilitychange", onVis);
      canvas.remove();
      stopHero = null;
    };
  };
  const tryHero = () => mountHeroParticles();
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", tryHero, { once: true });
  } else {
    tryHero();
  }
  document.addEventListener("resuma:navigate", () => requestAnimationFrame(tryHero));
})();
