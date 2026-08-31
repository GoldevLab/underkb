//! UnderKb — compress images under a kilobyte budget (Resuma Flow).

mod api;
mod compress;
mod landing;
mod ops;
mod pages;
mod stage;
mod tool;

use axum::routing::{get, post};
use pages::PagesRegistry;
use resuma::prelude::*;
use resuma::SeoKit;
use serde_json::json;

fn chrome(body: View) -> View {
    view! {
        <div class="app">
            <header class="site-header">
                <div class="header-inner">
                    <NavLink href="/" class="brand" activeClass="is-active" exact=true>
                        <span class="brand-mark" aria-hidden="true">"kB"</span>
                        <span class="brand-name">"UnderKb"</span>
                    </NavLink>
                    {tool::tools_nav()}
                    <span class="nav-progress" aria-hidden="true"></span>
                </div>
            </header>
            {body}
            <footer class="site-footer">
                <p>
                    <strong>"UnderKb"</strong>
                    " — image tools: compress to a KB budget, JPG→WebP, resize, flat background, color palette. No account."
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
                <NavLink href="/" class="btn btn-primary">"Compress an image"</NavLink>
            </p>
        </main>
    })
}

fn seo_kit() -> SeoKit {
    let mut kit = SeoKit::new("UnderKb", "https://underkb.fly.dev")
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
            "url": "https://underkb.fly.dev",
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
                }
            ]
        }));
    kit.theme_color = Some("#0d9488".into());
    kit.author = "UnderKb".into();
    kit.llms_sections = vec![(
        "How to use".into(),
        "POST /api/compress file,target_kb,format. /api/convert format,quality. /api/resize width,height,mode. /api/remove-bg tolerance. /api/colors count.".into(),
    )];
    kit
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let kit = seo_kit();
    let head = format!(
        "{}<link rel=\"icon\" href=\"/icon.svg\" type=\"image/svg+xml\" /><script type=\"module\" src=\"/js/underkb.js?v=5\"></script>",
        kit.head_extras()
    );
    let json_ld = serde_json::to_string(&kit.json_ld_blocks).unwrap_or_else(|_| "[]".into());
    let llms: &'static [u8] = Box::leak(kit.llms_txt().into_bytes().into_boxed_slice());
    const ICON: &[u8] = include_bytes!("icon.svg");
    let public = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("public");

    FlowApp::new()
        .with_title("Compress Image to 200KB — Free JPG & WebP Under Any Size | UnderKb")
        .with_description(
            "Compress a photo under 200 KB — or any size you set. JPG, WebP, PNG. Up to 20 MB. No account, no watermark.",
        )
        .with_site_url("https://underkb.fly.dev")
        .with_og_image("/cover.png")
        .with_json_ld(json_ld)
        .with_head(head)
        .with_stylesheet("/css/underkb.css")
        .static_asset("/llms.txt", llms, "text/plain; charset=utf-8")
        .static_asset("/icon.svg", ICON, "image/svg+xml")
        .with_public_dir(public)
        .without_pwa()
        .route("/api/compress", post(api::compress_upload).options(api::preflight))
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
        .serve(FlowServeOptions::default())
        .await
}
