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
        <p class="hint" data-gif="" hidden="">"Los GIF animados: solo el primer fotograma."</p>
        <p class="hint" data-format-hint="" hidden="">"En fotos, JPG suele cumplir el límite. WebP/PNG sin pérdida pueden quedar grandes."</p>
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
            <a class="btn btn-primary" data-download="" download="">{download}</a>
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
            {result_block("Original", "Compressed", "Download")}
        </div>
    }
}

pub fn compressor_es() -> View {
    view! {
        <div id="ukb-app" data-tool="compress" data-endpoint="/api/compress" data-idle="Comprimir" data-busy="Comprimiendo…" data-empty="Suelta o elige una imagen primero.">
            <form class="hero-form" data-form="" method="post" action="/api/compress" enctype="multipart/form-data">
                {dropzone(
                    "Suelta una imagen, o pulsa para elegir",
                    "JPG, PNG, WebP, GIF, HEIC · máx. 20 MB",
                )}
                <div class="controls">
                    <label>
                        "Tamaño objetivo (KB)"
                        <input
                            data-kb=""
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
                        "Salida"
                        <select data-format="" name="format">
                            <option value="jpeg" selected=true>"JPG"</option>
                            <option value="webp">"WebP (sin pérdida)"</option>
                            <option value="png">"PNG"</option>
                        </select>
                    </label>
                </div>
                <div class="presets" role="group" aria-label="Tamaños">
                    <button type="button" class="btn btn-ghost" data-preset="50">"50 KB"</button>
                    <button type="button" class="btn btn-ghost" data-preset="100">"100 KB"</button>
                    <button type="button" class="btn btn-ghost is-on" data-preset="200">"200 KB"</button>
                    <button type="button" class="btn btn-ghost" data-preset="500">"500 KB"</button>
                    <button type="button" class="btn btn-ghost" data-preset="1024">"1 MB"</button>
                </div>
                <p class="hint form-error" data-error="" hidden="" role="alert" aria-live="polite"></p>
                <button type="submit" class="btn btn-primary btn-wide" data-submit="">"Comprimir"</button>
            </form>
            {result_block("Original", "Comprimida", "Descargar")}
        </div>
    }
}

pub fn converter() -> View {
    view! {
        <div id="ukb-app" data-tool="convert" data-endpoint="/api/convert" data-idle="Convertir" data-busy="Convirtiendo…" data-empty="Suelta o elige una imagen primero.">
            <form class="hero-form" data-form="" method="post" action="/api/convert" enctype="multipart/form-data">
                {dropzone(
                    "Suelta un JPG o PNG",
                    "Salida por defecto: WebP · máx. 20 MB",
                )}
                <div class="controls">
                    <label>
                        "Formato"
                        <select data-format="" name="format">
                            <option value="webp" selected=true>"WebP"</option>
                            <option value="jpeg">"JPG"</option>
                            <option value="png">"PNG"</option>
                        </select>
                    </label>
                    <label>
                        "Calidad"
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
                <p class="hint">"80 es un buen WebP para fotos. 98 o más usa WebP sin pérdida."</p>
                <p class="hint form-error" data-error="" hidden="" role="alert" aria-live="polite"></p>
                <button type="submit" class="btn btn-primary btn-wide" data-submit="">"Convertir"</button>
            </form>
            {result_block("Original", "Convertida", "Descargar")}
        </div>
    }
}

pub fn resizer() -> View {
    view! {
        <div id="ukb-app" data-tool="resize" data-endpoint="/api/resize" data-idle="Redimensionar" data-busy="Redimensionando…" data-empty="Suelta o elige una imagen primero.">
            <form class="hero-form" data-form="" method="post" action="/api/resize" enctype="multipart/form-data">
                {dropzone(
                    "Suelta una imagen para cambiar el tamaño",
                    "Un solo lado mantiene la proporción · máx. 20 MB",
                )}
                <p class="hint" data-dims="" hidden=""></p>
                <div class="controls">
                    <label>
                        "Ancho (px)"
                        <input data-width="" name="width" type="number" min="1" max="4096" inputmode="numeric" placeholder="p. ej. 1200" />
                    </label>
                    <label>
                        "Alto (px)"
                        <input data-height="" name="height" type="number" min="1" max="4096" inputmode="numeric" placeholder="auto" />
                    </label>
                </div>
                <div class="controls">
                    <label>
                        "Modo"
                        <select data-mode="" name="mode">
                            <option value="fit" selected=true>"Encajar (proporción)"</option>
                            <option value="fill">"Rellenar y recortar"</option>
                            <option value="stretch">"Estirar"</option>
                        </select>
                    </label>
                    <label>
                        "Salida"
                        <select data-format="" name="format">
                            <option value="jpeg" selected=true>"JPG"</option>
                            <option value="webp">"WebP"</option>
                            <option value="png">"PNG"</option>
                        </select>
                    </label>
                </div>
                <p class="hint form-error" data-error="" hidden="" role="alert" aria-live="polite"></p>
                <button type="submit" class="btn btn-primary btn-wide" data-submit="">"Redimensionar"</button>
            </form>
            {result_block("Original", "Redimensionada", "Descargar")}
        </div>
    }
}

pub fn remover() -> View {
    view! {
        <div id="ukb-app" data-tool="removebg" data-endpoint="/api/remove-bg" data-idle="Quitar fondo" data-busy="Recortando…" data-empty="Suelta o elige una imagen primero.">
            <form class="hero-form" data-form="" method="post" action="/api/remove-bg" enctype="multipart/form-data">
                {dropzone(
                    "Suelta una foto con fondo liso",
                    "Producto o captura sobre blanco o color uniforme",
                )}
                <div class="controls">
                    <label>
                        "Tolerancia"
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
                        "Salida"
                        <select data-format="" name="format">
                            <option value="png" selected=true>"PNG (transparencia)"</option>
                            <option value="webp">"WebP"</option>
                        </select>
                    </label>
                </div>
                <p class="hint">"Si queda halo, sube la tolerancia. Si se come el objeto, bájala."</p>
                <p class="hint form-error" data-error="" hidden="" role="alert" aria-live="polite"></p>
                <button type="submit" class="btn btn-primary btn-wide" data-submit="">"Quitar fondo"</button>
            </form>
            {result_block("Original", "Sin fondo", "Descargar PNG")}
        </div>
    }
}

pub fn palette() -> View {
    view! {
        <div id="ukb-app" data-tool="colors" data-endpoint="/api/colors" data-idle="Extraer colores" data-busy="Analizando…" data-empty="Suelta o elige una imagen primero.">
            <form class="hero-form" data-form="" method="post" action="/api/colors" enctype="multipart/form-data">
                {dropzone(
                    "Suelta una imagen para ver su paleta",
                    "JPG, PNG, WebP · máx. 20 MB",
                )}
                <div class="controls">
                    <label>
                        "Colores"
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
                <button type="submit" class="btn btn-primary btn-wide" data-submit="">"Extraer colores"</button>
            </form>
            {result_block("Original", "Paleta", "Copiar HEX")}
        </div>
    }
}

pub fn tools_nav() -> View {
    view! {
        <nav class="tool-nav" aria-label="Herramientas">
            <NavLink href="/" class="tool-nav-link" activeClass="is-active" exact=true>"200 KB"</NavLink>
            <NavLink href="/comprimir-imagen-kb" class="tool-nav-link" activeClass="is-active">"Comprimir"</NavLink>
            <NavLink href="/convertir-jpg-a-webp" class="tool-nav-link" activeClass="is-active">"JPG→WebP"</NavLink>
            <NavLink href="/redimensionar-imagen" class="tool-nav-link" activeClass="is-active">"Redimensionar"</NavLink>
            <NavLink href="/quitar-fondo" class="tool-nav-link" activeClass="is-active">"Quitar fondo"</NavLink>
            <NavLink href="/extraer-colores-imagen" class="tool-nav-link" activeClass="is-active">"Colores"</NavLink>
        </nav>
    }
}
