use resuma::prelude::*;

use crate::landing::canonical_url;
use crate::site;

pub fn page(_req: FlowRequest) -> View {
    crate::ads::begin_page();
    set_page_title("KB extension for Chrome — compress images | UnderKb");
    set_page_description(
        "UnderKb KB extension for Chrome. Open compress to 200 KB, JPG to WebP, resize or stretch, remove a flat background, or copy a HEX palette from the toolbar.",
    );
    set_page_canonical(canonical_url("/extension"));
    set_page_json_ld(extension_json_ld());
    let store_block = match site::chrome_store_url() {
        Some(url) => view! {
            <p>
                <a class="btn btn-primary" href={url} rel="noopener">"Add from Chrome Web Store"</a>
            </p>
        },
        None => view! {
            <p class="hint">
                "The Web Store listing is not public yet. In chrome://extensions turn on Developer mode and Load unpacked from the GitHub folder extension/."
            </p>
            <p>
                <a class="btn btn-primary" href="https://github.com/GoldevLab/underkb/tree/main/extension" rel="noopener">
                    "extension/ on GitHub"
                </a>
            </p>
        },
    };
    view! {
        <main class="content-section privacy-page">
            <p class="eyebrow">"Browser"</p>
            <h1>"KB extension for Chrome"</h1>
            <p class="hero-lead">
                "Toolbar shortcuts to every UnderKb job. Right-click an image to open Compress. The photo still drops on the website — ads stay there, which Google AdSense allows."
            </p>
            {store_block}
            <h2>"What it does"</h2>
            <ol class="howto-grid">
                <li>
                    <h3>"Toolbar"</h3>
                    <p>"Compress, WebP, resize, remove background, palette — each button opens the matching page."</p>
                </li>
                <li>
                    <h3>"Right-click"</h3>
                    <p>"On an image, Compress with UnderKb."</p>
                </li>
            </ol>
            <h2>"Open a tool"</h2>
            <ul class="feature-grid">
                <li>
                    <h3><NavLink href="/comprimir-imagen-kb">"Compress to KB"</NavLink></h3>
                    <p>"Hit 50, 200, or 500 KB."</p>
                </li>
                <li>
                    <h3><NavLink href="/convertir-jpg-a-webp">"JPG to WebP"</NavLink></h3>
                    <p>"Smaller photos for the web."</p>
                </li>
                <li>
                    <h3><NavLink href="/redimensionar-imagen">"Resize"</NavLink></h3>
                    <p>"Fit, crop, or stretch."</p>
                </li>
                <li>
                    <h3><NavLink href="/quitar-fondo">"Remove background"</NavLink></h3>
                    <p>"Flat backdrop to a transparent PNG."</p>
                </li>
                <li>
                    <h3><NavLink href="/extraer-colores-imagen">"Colors"</NavLink></h3>
                    <p>"HEX palette from the photo."</p>
                </li>
            </ul>
            <h2>"FAQ"</h2>
            <div class="faq-list">
                <details>
                    <summary>"What does the KB extension do?"</summary>
                    <p>"It opens UnderKb from the Chrome toolbar: compress to a KB budget, JPG to WebP, resize or stretch, remove a flat background, or copy HEX colors. The image is processed on the website."</p>
                </details>
                <details>
                    <summary>"Does the extension compress inside Chrome?"</summary>
                    <p>"No. Each button opens the matching page on UnderKb. Ads stay on the site, which is what AdSense allows."</p>
                </details>
            </div>
            <p>
                <NavLink href="/" class="btn btn-ghost">"Use the website"</NavLink>
            </p>
            {crate::ads::slot("extension-end", "article")}
        </main>
    }
}

fn extension_json_ld() -> String {
    serde_json::json!({
        "@context": "https://schema.org",
        "@graph": [
            {
                "@type": "WebPage",
                "name": "KB extension for Chrome — compress images | UnderKb",
                "description": "UnderKb KB extension for Chrome. Open compress to 200 KB, JPG to WebP, resize or stretch, remove a flat background, or copy a HEX palette from the toolbar.",
                "url": canonical_url("/extension"),
                "inLanguage": "en"
            },
            {
                "@type": "FAQPage",
                "mainEntity": [
                    {
                        "@type": "Question",
                        "name": "What does the KB extension do?",
                        "acceptedAnswer": {"@type": "Answer", "text": "It opens UnderKb from the Chrome toolbar: compress to a KB budget, JPG to WebP, resize or stretch, remove a flat background, or copy HEX colors. The image is processed on the website."}
                    },
                    {
                        "@type": "Question",
                        "name": "Does the extension compress inside Chrome?",
                        "acceptedAnswer": {"@type": "Answer", "text": "No. Each button opens the matching page on UnderKb. Ads stay on the site, which is what AdSense allows."}
                    }
                ]
            }
        ]
    })
    .to_string()
}
