//! UnderKb — compress images under a kilobyte budget (Resuma Flow).

mod ads;
mod api;
mod compress;
mod landing;
mod ops;
mod site;
mod pages;
mod stage;
mod tool;

use axum::response::Redirect;
use axum::routing::{get, post};
use pages::PagesRegistry;
use resuma::prelude::*;
use resuma::SeoKit;
use serde_json::json;

fn chrome(body: View) -> View {
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
                    <span class="nav-progress" aria-hidden="true"></span>
                </div>
            </header>
            {body}
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
    chrome(view! { <Slot /> })
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
    })
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
        .with_default_json_ld()
        .push_json_ld(json!({
            "@context": "https://schema.org",
            "@type": "WebApplication",
            "name": "UnderKb",
            "alternateName": ["compress image to 200kb", "image compressor"],
            "url": crate::landing::public_origin(),
            "applicationCategory": "UtilitiesApplication",
            "operatingSystem": "Web",
            "offers": {"@type": "Offer", "price": "0", "priceCurrency": "USD"},
            "description": "Free image compressor that hits a real KB target. JPG, WebP, PNG. No account."
        }))
        .push_json_ld(json!({
            "@context": "https://schema.org",
            "@type": "FAQPage",
            "mainEntity": [
                {
                    "@type": "Question",
                    "name": "Can I compress a JPG to 200 KB?",
                    "acceptedAnswer": {
                        "@type": "Answer",
                        "text": "Yes. 200 KB is the default target. You can set 50, 100, 500, or 1024 KB."
                    }
                },
                {
                    "@type": "Question",
                    "name": "Does UnderKb store my images?",
                    "acceptedAnswer": {
                        "@type": "Answer",
                        "text": "The compressed file is kept in memory for about 30 minutes so you can download it, then it expires."
                    }
                },
                {
                    "@type": "Question",
                    "name": "Do I need an account?",
                    "acceptedAnswer": {
                        "@type": "Answer",
                        "text": "No. Drop a file, download. Ads may appear around the tool."
                    }
                },
                {
                    "@type": "Question",
                    "name": "Does remove background work on portraits?",
                    "acceptedAnswer": {
                        "@type": "Answer",
                        "text": "Only flat backdrops (studio, white). It is not an AI people matte."
                    }
                }
            ]
        }));
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
            "POST /api/compress file,target_kb,format. POST /api/compress-batch (Pro: ZIP of up to 20). /api/convert format,quality. /api/resize width,height,mode. /api/remove-bg tolerance. /api/colors count. Optional X-Api-Key (UNDERKB_PRO_KEYS) for 50 MB and batch.".into(),
        ),
    ];
    kit
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // `with_seo_kit` owns keywords/author/theme-color meta, JSON-LD, and the
    // `/robots.txt` + `/llms.txt` routes (AI crawler policy included).
    let head = format!(
        "<link rel=\"icon\" href=\"/icon.svg\" type=\"image/svg+xml\" /><script type=\"module\" src=\"/js/underkb.js?v=10\"></script>{}{}",
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
        .with_html_theme(
            HtmlTheme::new(["forest"])
                .dark(["forest"])
                .cookie("underkb_theme")
                .storage_key("underkb-theme"),
        )
        .with_stylesheet("/css/underkb.css")
        .static_asset("/icon.svg", ICON, "image/svg+xml");
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
            cache_version: "ukb-7".into(),
            display: "standalone".into(),
            orientation: "any".into(),
            lang: "en".into(),
            icon_char: Some("k".into()),
            // Must match the URL the page requests (`?v=10`), or the SW precache
            // never hits offline.
            precache_paths: vec![
                "/themes.css".into(),
                "/css/underkb.css".into(),
                "/js/underkb.js?v=10".into(),
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
