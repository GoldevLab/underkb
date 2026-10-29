//! UnderKb — compress images under a kilobyte budget (Resuma Flow).

mod ads;
mod api;
mod compress;
mod landing;
mod ops;
mod site;
mod pages;
mod stage;
mod stats;
mod tool;

use axum::response::Redirect;
use axum::routing::{get, post};
use pages::PagesRegistry;
use resuma::prelude::*;
use resuma::SeoKit;

fn page_shows_ads() -> bool {
    let path = current_request()
        .map(|r| r.path)
        .unwrap_or_else(|| "/".into());
    let trimmed = path.trim_end_matches('/');
    let path = if trimmed.is_empty() { "/" } else { trimmed };
    !matches!(path, "/privacy" | "/terms" | "/pricing")
}

fn chrome(body: View, ads: bool) -> View {
    let activity = crate::stats::snapshot();
    let n_compress = crate::stats::grouped(activity.compress);
    let n_convert = crate::stats::grouped(activity.convert);
    let n_resize = crate::stats::grouped(activity.resize);
    let n_background = crate::stats::grouped(activity.background);
    let n_colors = crate::stats::grouped(activity.colors);
    let footer_ad = if ads {
        view! {
            <div class="ad-rail ad-rail-end">
                {crate::ads::slot("footer", "leaderboard")}
            </div>
        }
    } else {
        view! { <div hidden=""></div> }
    };
    view! {
        <div class="app">
            <div class="liquid-orbs" aria-hidden="true">
                <div class="liquid-blob liquid-blob-a"></div>
                <div class="liquid-blob liquid-blob-b"></div>
                <div class="liquid-blob liquid-blob-c"></div>
            </div>
            <header class="site-header">
                <div class="header-inner">
                    <NavLink href="/" class="brand" activeClass="is-active" exact=true>
                        <span class="brand-mark" aria-hidden="true">"kB"</span>
                        <span class="brand-name">"UnderKb"</span>
                    </NavLink>
                    <div class="live-stats" data-live-stats="" aria-live="polite" aria-label="Completed jobs">
                        <span class="live-dot" aria-hidden="true"></span>
                        <span class="live-stat">"Compress "<strong data-stat="compress">{n_compress}</strong></span>
                        <span class="live-stat">"Convert "<strong data-stat="convert">{n_convert}</strong></span>
                        <span class="live-stat">"Resize "<strong data-stat="resize">{n_resize}</strong></span>
                        <span class="live-stat">"Background "<strong data-stat="background">{n_background}</strong></span>
                        <span class="live-stat">"Colors "<strong data-stat="colors">{n_colors}</strong></span>
                    </div>
                    <span class="nav-progress" aria-hidden="true"></span>
                </div>
                <nav class="site-nav" aria-label="Tools">
                    <NavLink href="/comprimir-imagen-kb" class="site-nav-link" activeClass="is-active">"Compress to KB"</NavLink>
                    <NavLink href="/convertir-jpg-a-webp" class="site-nav-link" activeClass="is-active">"JPG to WebP"</NavLink>
                    <NavLink href="/redimensionar-imagen" class="site-nav-link" activeClass="is-active">"Resize"</NavLink>
                    <NavLink href="/quitar-fondo" class="site-nav-link" activeClass="is-active">"Remove background"</NavLink>
                    <NavLink href="/extraer-colores-imagen" class="site-nav-link" activeClass="is-active">"Colors"</NavLink>
                </nav>
            </header>
            {body}
            {footer_ad}
            <footer class="site-footer">
                {crate::landing::seo_footer_links()}
                {crate::landing::sister_apps_links()}
                <p>
                    <strong>"UnderKb"</strong>
                    " — compress to a kilobyte budget, convert, resize, cut a flat background, HEX palette. No account."
                </p>
            </footer>
        </div>
    }
}

#[layout("/")]
fn RootLayout() -> View {
    chrome(view! { <Slot /> }, page_shows_ads())
}

fn not_found() -> View {
    chrome(view! {
        <main class="content-section">
            <h1>"Page not found"</h1>
            <p class="hero-lead">"That path does not exist on UnderKb."</p>
            <p>
                <NavLink href="/" class="btn btn-primary">"Go home"</NavLink>
            </p>
        </main>
    }, false)
}

async fn redirect_tool_alias(target: &'static str) -> Redirect {
    Redirect::permanent(target)
}

fn seo_kit() -> SeoKit {
    let origin = crate::landing::public_origin();
    let mut kit = SeoKit::new("UnderKb", &origin)
        .with_locale("en_US")
        .with_keywords(
            "compress image to 200kb, comprimir imagen kb, convertir jpg a webp, redimensionar imagen, \
             quitar fondo, extraer colores imagen, image compressor, compress jpg to 200kb",
        )
        .with_llms_summary(
            "UnderKb compresses a JPG, PNG, WebP, or GIF under a kilobyte budget (default 200 KB). \
             Also: convert JPG to WebP, resize, remove a flat background, extract a color palette. No account.",
        )
        .with_default_json_ld();
    kit.theme_color = Some("#0c1410".into());
    kit.author = "UnderKb".into();
    kit.llms_sections = vec![
        (
            "How to use".into(),
            "Open / and drop an image to compress to a KB budget (default 200). Other jobs have their own pages.".into(),
        ),
        (
            "SEO landings".into(),
            "/comprimir-imagen-kb, /convertir-jpg-a-webp, /redimensionar-imagen, /quitar-fondo, /extraer-colores-imagen. English aliases redirect to those. /privacy /terms /pricing.".into(),
        ),
        (
            "API".into(),
            "POST /api/compress file,target_kb,format. POST /api/compress-batch (Pro: ZIP of up to 20). /api/convert format,quality. /api/resize width,height,mode. /api/remove-bg tolerance. /api/colors count. Optional X-Api-Key for 50 MB and batch. Keys are issued by hand.".into(),
        ),
    ];
    // Temporary downloads and JSON APIs are not pages. Pricing stays crawlable
    // so its noindex can be seen; it is already out of the sitemap.
    kit.ai.disallow = vec!["/api/".into(), "/d/".into()];
    kit
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    crate::stats::init().await;
    // `with_seo_kit` owns keywords/author/theme-color meta, JSON-LD, and the
    // `/robots.txt` + `/llms.txt` routes (AI crawler policy included).
    let head = format!(
        "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1, viewport-fit=cover\" /><link rel=\"icon\" href=\"/icon.svg\" type=\"image/svg+xml\" /><link rel=\"icon\" href=\"/icons/favicon-32.png\" type=\"image/png\" sizes=\"32x32\" /><link rel=\"apple-touch-icon\" href=\"/icons/apple-touch-icon.png\" sizes=\"180x180\" /><script type=\"module\" src=\"/js/underkb.js?v=12\"></script>{}{}",
        ads::head_snippet(),
        crate::site::head_extras()
    );
    let ads_txt = ads::ads_txt().map(|s| -> &'static [u8] {
        Box::leak(s.into_bytes().into_boxed_slice())
    });
    const ICON: &[u8] = include_bytes!("icon.svg");
    let public = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("public");

    let mut serve = FlowServeOptions::default();
    ads::apply_csp(&mut serve.security.csp);

    let mut app = FlowApp::new()
        .with_title("UnderKb — compress, convert, resize, and more")
        .with_description(
            "Free image tools: compress to KB, JPG to WebP, resize, remove a flat background, HEX palette. No account.",
        )
        .with_site_url(crate::landing::public_origin())
        .with_og_image("/cover.png")
        .with_head(head)
        .with_seo_kit(seo_kit())
        .with_sitemap_exclude(["/pricing"])
        .with_html_theme(
            HtmlTheme::new(["forest"])
                .dark(["forest"])
                .cookie("underkb_theme")
                .storage_key("underkb-theme"),
        )
        .with_stylesheet("/css/underkb.css?v=r5")
        .static_asset("/icon.svg", ICON, "image/svg+xml");
    {
        let contact = crate::site::contact_email()
            .map(|e| format!("Contact: mailto:{e}\n"))
            .unwrap_or_default();
        let body = format!(
            "{contact}Canonical: https://under200kb.com/.well-known/security.txt\nPreferred-Languages: en\nExpires: 2027-12-31T23:59:59Z\n"
        );
        let leaked: &'static [u8] = Box::leak(body.into_bytes().into_boxed_slice());
        app = app.static_asset(
            "/.well-known/security.txt",
            leaked,
            "text/plain; charset=utf-8",
        );
    }

    if let Some(body) = ads_txt {
        app = app.static_asset("/ads.txt", body, "text/plain; charset=utf-8");
    }
    app.with_public_dir(public)
        .with_pwa(FlowPwaConfig {
            name: "UnderKb".into(),
            short_name: "UnderKb".into(),
            description: "Compress to KB, convert, resize, remove a flat background, HEX palette."
                .into(),
            theme_color: "#34d399".into(),
            background_color: "#0c1410".into(),
            start_url: "/".into(),
            scope: "/".into(),
            cache_version: "ukb-12".into(),
            display: "standalone".into(),
            orientation: "any".into(),
            lang: "en".into(),
            icon_char: Some("k".into()),
            // Must match the URL the page requests (`?v=10`), or the SW precache
            // never hits offline.
            precache_paths: vec![
                "/themes.css".into(),
                "/css/underkb.css?v=r5".into(),
                "/js/underkb.js?v=12".into(),
                "/icon.svg".into(),
                "/icons/icon-192.png".into(),
                "/icons/icon-512.png".into(),
                "/icons/apple-touch-icon.png".into(),
            ],
            shortcuts: vec![
                PwaShortcut {
                    name: "Home".into(),
                    short_name: "Home".into(),
                    url: "/".into(),
                },
                PwaShortcut {
                    name: "Compress to KB".into(),
                    short_name: "Compress".into(),
                    url: "/comprimir-imagen-kb".into(),
                },
            ],
            offline_title: "You're offline".into(),
            offline_message:
                "UnderKb needs a connection to process images. Reconnect and try again.".into(),
            manifest_icons: Vec::new(),
        })
        .route(landing::Tool::Compress.en_alias(), get(|| redirect_tool_alias(landing::Tool::Compress.path())))
        .route(landing::Tool::Convert.en_alias(), get(|| redirect_tool_alias(landing::Tool::Convert.path())))
        .route(landing::Tool::Resize.en_alias(), get(|| redirect_tool_alias(landing::Tool::Resize.path())))
        .route(landing::Tool::RemoveBg.en_alias(), get(|| redirect_tool_alias(landing::Tool::RemoveBg.path())))
        .route(landing::Tool::Colors.en_alias(), get(|| redirect_tool_alias(landing::Tool::Colors.path())))
        .route("/api/stats", get(api::stats).options(api::preflight))
        .route("/api/compress", post(api::compress_upload).options(api::preflight))
        .route("/api/compress-batch", post(api::compress_batch).options(api::preflight))
        .route("/api/convert", post(api::convert_upload).options(api::preflight))
        .route("/api/resize", post(api::resize_upload).options(api::preflight))
        .route("/api/remove-bg", post(api::remove_bg_upload).options(api::preflight))
        .route("/api/colors", post(api::colors_upload).options(api::preflight))
        .route("/d/{id}", get(api::download))
        .not_found(not_found)
        .auto_pages(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/pages"),
            PagesRegistry,
        )
        .serve(serve)
        .await
}
