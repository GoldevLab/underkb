//! Image tool UIs — dropzone, options, result.

use resuma::prelude::*;

fn dropzone(title: &'static str, hint: &'static str) -> View {
    view! {
        <label class="drop" data-drop="">
            <input
                class="drop-input"
                data-file=""
                name="file"
                type="file"
                accept="image/jpeg,image/png,image/webp,image/gif,image/heic,image/heif,.heic,.heif"
            />
            <img class="drop-preview" data-drop-preview="" alt="" hidden="" />
            <span class="drop-title">{title}</span>
            <span class="drop-hint" data-filename="">{hint}</span>
        </label>
        <p class="hint" data-gif="" hidden="">"Animated GIFs: we keep the first frame only."</p>
        <p class="hint" data-format-hint="" hidden="">"JPG or WebP usually hits the budget on photos. PNG can stay huge."</p>
    }
}

fn result_block(before: &'static str, after: &'static str, download: &'static str) -> View {
    view! {
        <div class="result" data-result="" hidden="">
            <div class="compare">
                <figure>
                    <img data-before="" alt={before} />
                    <figcaption>{before}</figcaption>
                </figure>
                <figure>
                    <img data-preview="" alt={after} />
                    <figcaption>{after}</figcaption>
                </figure>
            </div>
            <p class="result-name" data-out-name=""></p>
            <p class="result-stats" data-stats=""></p>
            <p class="result-warn" data-warn="" hidden="" role="status"></p>
            <div class="swatches" data-swatches="" hidden=""></div>
            <div class="result-actions">
                <a class="btn btn-primary" data-download="" download="">{download}</a>
                <button type="button" class="btn btn-ghost" data-share="" hidden="">"Share"</button>
            </div>
        </div>
    }
}

pub fn compressor() -> View {
    view! {
        <div id="ukb-app" data-tool="compress" data-endpoint="/api/compress" data-idle="Compress" data-busy="Compressing…" data-empty="Drop or choose an image first.">
            <form class="hero-form" data-form="" method="post" action="/api/compress" enctype="multipart/form-data">
                <label class="drop" data-drop="">
                    <input
                        class="drop-input"
                        data-file=""
                        id="ukb-file"
                        name="file"
                        type="file"
                        multiple=""
                        accept="image/jpeg,image/png,image/webp,image/gif,image/heic,image/heif,.heic,.heif"
                    />
                    <img class="drop-preview" data-drop-preview="" alt="" hidden="" />
                    <span class="drop-title">"Drop an image, or click to choose"</span>
                    <span class="drop-hint" data-filename="">"JPG, PNG, WebP, GIF, HEIC · free 20 MB · Pro: 50 MB or ZIP of 20"</span>
                </label>
                <p class="hint" data-gif="" hidden="">"Animated GIFs: we keep the first frame only."</p>
                <p class="hint" data-format-hint="" hidden="">"JPG or WebP usually hits the budget on photos. PNG can stay huge or time out."</p>
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
                            <option value="webp">"WebP"</option>
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
            {result_block("Original", "Compressed", "Download")}
        </div>
    }
}

pub fn converter() -> View {
    view! {
        <div id="ukb-app" data-tool="convert" data-endpoint="/api/convert" data-idle="Convert" data-busy="Converting…" data-empty="Drop or choose an image first.">
            <form class="hero-form" data-form="" method="post" action="/api/convert" enctype="multipart/form-data">
                {dropzone(
                    "Drop a JPG or PNG",
                    "Default output: WebP · max 20 MB",
                )}
                <div class="controls">
                    <label>
                        "Format"
                        <select data-format="" name="format">
                            <option value="webp" selected=true>"WebP"</option>
                            <option value="jpeg">"JPG"</option>
                            <option value="png">"PNG"</option>
                        </select>
                    </label>
                    <label>
                        "Quality"
                        <input
                            data-quality=""
                            name="quality"
                            type="number"
                            min="20"
                            max="100"
                            value="80"
                            inputmode="numeric"
                        />
                    </label>
                </div>
                <p class="hint">"80 is a solid WebP for photos. 98 or higher uses lossless WebP."</p>
                <p class="hint form-error" data-error="" hidden="" role="alert" aria-live="polite"></p>
                <button type="submit" class="btn btn-primary btn-wide" data-submit="">"Convert"</button>
            </form>
            {result_block("Original", "Converted", "Download")}
        </div>
    }
}

pub fn resizer() -> View {
    view! {
        <div id="ukb-app" data-tool="resize" data-endpoint="/api/resize" data-idle="Resize" data-busy="Resizing…" data-empty="Drop or choose an image first.">
            <form class="hero-form" data-form="" method="post" action="/api/resize" enctype="multipart/form-data">
                {dropzone(
                    "Drop an image to resize",
                    "One side keeps aspect ratio · max 20 MB",
                )}
                <p class="hint" data-dims="" hidden=""></p>
                <div class="controls">
                    <label>
                        "Width (px)"
                        <input data-width="" name="width" type="number" min="1" max="4096" inputmode="numeric" placeholder="e.g. 1200" />
                    </label>
                    <label>
                        "Height (px)"
                        <input data-height="" name="height" type="number" min="1" max="4096" inputmode="numeric" placeholder="auto" />
                    </label>
                </div>
                <div class="controls">
                    <label>
                        "Mode"
                        <select data-mode="" name="mode">
                            <option value="fit" selected=true>"Fit (keep aspect)"</option>
                            <option value="fill">"Fill and crop"</option>
                            <option value="stretch">"Stretch"</option>
                        </select>
                    </label>
                    <label>
                        "Output"
                        <select data-format="" name="format">
                            <option value="jpeg" selected=true>"JPG"</option>
                            <option value="webp">"WebP"</option>
                            <option value="png">"PNG"</option>
                        </select>
                    </label>
                </div>
                <p class="hint form-error" data-error="" hidden="" role="alert" aria-live="polite"></p>
                <button type="submit" class="btn btn-primary btn-wide" data-submit="">"Resize"</button>
            </form>
            {result_block("Original", "Resized", "Download")}
        </div>
    }
}

pub fn remover() -> View {
    view! {
        <div id="ukb-app" data-tool="removebg" data-endpoint="/api/remove-bg" data-idle="Remove background" data-busy="Cutting…" data-empty="Drop or choose an image first.">
            <form class="hero-form" data-form="" method="post" action="/api/remove-bg" enctype="multipart/form-data">
                {dropzone(
                    "Drop a photo with a flat backdrop",
                    "Product shot or screenshot on white or a solid color",
                )}
                <div class="controls">
                    <label>
                        "Tolerance"
                        <input
                            data-tolerance=""
                            name="tolerance"
                            type="number"
                            min="8"
                            max="90"
                            value="32"
                            inputmode="numeric"
                        />
                    </label>
                    <label>
                        "Output"
                        <select data-format="" name="format">
                            <option value="png" selected=true>"PNG (transparency)"</option>
                            <option value="webp">"WebP"</option>
                        </select>
                    </label>
                </div>
                <p class="hint">"Raise tolerance if you still see a halo. Lower it if it eats the subject."</p>
                <p class="hint form-error" data-error="" hidden="" role="alert" aria-live="polite"></p>
                <button type="submit" class="btn btn-primary btn-wide" data-submit="">"Remove background"</button>
            </form>
            {result_block("Original", "No background", "Download PNG")}
        </div>
    }
}

pub fn palette() -> View {
    view! {
        <div id="ukb-app" data-tool="colors" data-endpoint="/api/colors" data-idle="Extract colors" data-busy="Analyzing…" data-empty="Drop or choose an image first.">
            <form class="hero-form" data-form="" method="post" action="/api/colors" enctype="multipart/form-data">
                {dropzone(
                    "Drop an image to see its palette",
                    "JPG, PNG, WebP · max 20 MB",
                )}
                <div class="controls">
                    <label>
                        "Colors"
                        <input
                            data-count=""
                            name="count"
                            type="number"
                            min="3"
                            max="12"
                            value="6"
                            inputmode="numeric"
                        />
                    </label>
                </div>
                <p class="hint form-error" data-error="" hidden="" role="alert" aria-live="polite"></p>
                <button type="submit" class="btn btn-primary btn-wide" data-submit="">"Extract colors"</button>
            </form>
            {result_block("Original", "Palette", "Copy HEX")}
        </div>
    }
}

