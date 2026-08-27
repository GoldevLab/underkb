//! Home compressor UI — dropzone, target KB, download.

use resuma::prelude::*;

pub fn compressor() -> View {
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
