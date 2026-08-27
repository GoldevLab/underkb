//! Home compressor island — dropzone, target KB, download.

use resuma::prelude::*;

#[island]
pub fn compressor() -> View {
    visible_task!(
        r##"
(async (_state, _resuma) => {
    const root = document.getElementById("ukb-app");
    if (!root || root.dataset.ready === "1") return;
    root.dataset.ready = "1";
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
    let file = null;
    let beforeUrl = "";
    let origW = 0;
    let origH = 0;
    let abortCtl = null;
    let previewGen = 0;

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
        const show = !!(photo && (fmt === "png" || fmt === "webp"));
        formatHint.hidden = !show;
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
        if (outFmt !== "png") {
            ctx.fillStyle = "#fff";
            ctx.fillRect(0, 0, cw, ch);
        }
        ctx.drawImage(bitmap, 0, 0, cw, ch);
        bitmap.close?.();
        throwIfAborted(signal);
        let mime = "image/jpeg";
        let ext = ".jpg";
        if (outFmt === "png") { mime = "image/png"; ext = ".png"; }
        else if (outFmt === "webp") { mime = "image/webp"; ext = ".webp"; }
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
        const gen = ++previewGen;
        (async () => {
            try {
                const { bitmap } = await bitmapFrom(f);
                if (gen !== previewGen) { bitmap.close?.(); return; }
                origW = bitmap.width;
                origH = bitmap.height;
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
    const syncPresets = () => {
        const v = String(kb?.value || "200");
        root.querySelectorAll("[data-preset]").forEach((b) => {
            b.classList.toggle("is-on", b.getAttribute("data-preset") === v);
        });
    };
    const busy = (on, label) => {
        form?.setAttribute("aria-busy", on ? "true" : "false");
        if (submit) {
            submit.disabled = on;
            if (label) submit.textContent = label;
        }
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
            if (kb) kb.value = btn.getAttribute("data-preset") || "200";
            syncPresets();
        });
    });

    form?.addEventListener("submit", async (e) => {
        e.preventDefault();
        if (!file) { setError("Drop or choose an image first."); return; }
        let target = Number(kb?.value || 200);
        if (!Number.isFinite(target)) target = 200;
        if (target < 8) target = 8;
        if (target > 5120) target = 5120;
        if (kb && Number(kb.value) !== target) kb.value = String(target);
        syncPresets();
        const fmt = root.querySelector("[data-format]")?.value || "jpeg";
        abortCtl?.abort();
        abortCtl = new AbortController();
        busy(true, "Preparing…");
        setError("");
        result.hidden = true;
        if (warn) { warn.hidden = true; warn.textContent = ""; }
        try {
            const ready = await prepare(file, fmt, abortCtl.signal);
            const body = new FormData();
            body.append("file", ready);
            body.append("target_kb", String(target));
            body.append("format", fmt);
            body.append("orig_bytes", String(file.size));
            if (origW) body.append("orig_width", String(origW));
            if (origH) body.append("orig_height", String(origH));
            if (submit) submit.textContent = "Compressing…";
            const r = await fetch("/api/compress", { method: "POST", body, signal: abortCtl.signal });
            const text = await r.text();
            let data;
            try { data = JSON.parse(text); }
            catch (_) {
                throw new Error(r.status === 413 ? "File is too large for the server." : r.status === 429 ? "Too many compresses. Wait a minute." : "Compress failed.");
            }
            if (!data.ok) throw new Error(data.error || "Compress failed.");
            const before = result.querySelector("[data-before]");
            const img = result.querySelector("[data-preview]");
            const link = result.querySelector("[data-download]");
            if (before && beforeUrl) {
                before.src = beforeUrl;
                before.width = data.original_width || origW || 1;
                before.height = data.original_height || origH || 1;
                before.alt = "Original";
            }
            if (img) {
                img.src = data.url;
                img.width = data.width || 1;
                img.height = data.height || 1;
                img.alt = data.filename || "Compressed preview";
            }
            if (link) {
                link.href = data.url + (data.url.includes("?") ? "&" : "?") + "dl=1";
                link.setAttribute("download", data.filename);
            }
            const nameOut = result.querySelector("[data-out-name]");
            const stats = result.querySelector("[data-stats]");
            if (nameOut) nameOut.textContent = data.filename;
            if (stats) stats.textContent =
                human(data.original_bytes) + " → " + human(data.result_bytes)
                + " · " + data.original_width + "×" + data.original_height
                + " → " + data.width + "×" + data.height;
            if (warn) {
                if (data.over_budget) {
                    warn.textContent = "Could not reach " + (data.target_kb || target) + " KB. This is the closest size — try JPG or a larger budget.";
                    warn.hidden = false;
                } else {
                    warn.hidden = true;
                }
            }
            result.hidden = false;
        } catch (ex) {
            if (ex.name === "AbortError") return;
            setError(ex.message || "Compress failed.");
        } finally {
            busy(false, "Compress");
        }
    });
})
"##
    );

    view! {
        <div id="ukb-app">
            <form class="hero-form" data-form="" method="post" action="/api/compress" enctype="multipart/form-data">
                <label class="drop" data-drop="">
                    <input
                        class="drop-input"
                        data-file=""
                        id="ukb-file"
                        name="file"
                        type="file"
                        accept="image/jpeg,image/png,image/webp,image/gif,image/heic,image/heif,.heic,.heif"
                    />
                    <img class="drop-preview" data-drop-preview="" alt="" hidden="" />
                    <span class="drop-title">"Drop an image, or click to choose"</span>
                    <span class="drop-hint" data-filename="">"JPG, PNG, WebP, GIF, HEIC · max 20 MB"</span>
                </label>
                <p class="hint" data-gif="" hidden="">"Animated GIFs: we keep the first frame only."</p>
                <p class="hint" data-format-hint="" hidden="">"JPG usually hits the budget on photos. Lossless WebP/PNG can stay huge or time out."</p>
                <div class="controls">
                    <label>
                        "Target size (KB)"
                        <input
                            data-kb=""
                            id="ukb-kb"
                            name="target_kb"
                            type="number"
                            min="8"
                            max="5120"
                            value="200"
                            inputmode="numeric"
                            enterkeyhint="done"
                        />
                    </label>
                    <label>
                        "Output"
                        <select data-format="" id="ukb-format" name="format">
                            <option value="jpeg" selected=true>"JPG"</option>
                            <option value="webp">"WebP (lossless)"</option>
                            <option value="png">"PNG"</option>
                        </select>
                    </label>
                </div>
                <div class="presets" role="group" aria-label="Size presets">
                    <button type="button" class="btn btn-ghost" data-preset="50">"50 KB"</button>
                    <button type="button" class="btn btn-ghost" data-preset="100">"100 KB"</button>
                    <button type="button" class="btn btn-ghost is-on" data-preset="200">"200 KB"</button>
                    <button type="button" class="btn btn-ghost" data-preset="500">"500 KB"</button>
                    <button type="button" class="btn btn-ghost" data-preset="1024">"1 MB"</button>
                </div>
                <p class="hint form-error" data-error="" hidden="" role="alert" aria-live="polite"></p>
                <button type="submit" class="btn btn-primary btn-wide" data-submit="">"Compress"</button>
            </form>
            <div class="result" data-result="" hidden="">
                <div class="compare">
                    <figure>
                        <img data-before="" alt="Original" />
                        <figcaption>"Original"</figcaption>
                    </figure>
                    <figure>
                        <img data-preview="" alt="Compressed preview" />
                        <figcaption>"Compressed"</figcaption>
                    </figure>
                </div>
                <p class="result-name" data-out-name=""></p>
                <p class="result-stats" data-stats=""></p>
                <p class="result-warn" data-warn="" hidden="" role="status"></p>
                <a class="btn btn-primary" data-download="" download="">"Download"</a>
            </div>
        </div>
    }
}
