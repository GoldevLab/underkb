//! Tool landings (English UI; Spanish URL slugs for search).

use resuma::prelude::*;
use serde_json::json;

use crate::tool;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Compress,
    Convert,
    Resize,
    RemoveBg,
    Colors,
}

impl Tool {
    pub fn path(self) -> &'static str {
        match self {
            Self::Compress => "/comprimir-imagen-kb",
            Self::Convert => "/convertir-jpg-a-webp",
            Self::Resize => "/redimensionar-imagen",
            Self::RemoveBg => "/quitar-fondo",
            Self::Colors => "/extraer-colores-imagen",
        }
    }

    fn title(self) -> &'static str {
        match self {
            Self::Compress => "Compress image to KB — JPG, PNG and WebP | UnderKb",
            Self::Convert => "Convert JPG to WebP online, free | UnderKb",
            Self::Resize => "Resize an image — width, height, and format | UnderKb",
            Self::RemoveBg => "Remove a flat image background | UnderKb",
            Self::Colors => "Extract colors from an image — HEX palette | UnderKb",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Compress => {
                "Compress a photo to 50, 200, or 500 KB. JPG, PNG, or WebP. No account. Free 20 MB; Pro 50 MB or a ZIP of 20."
            }
            Self::Convert => {
                "Turn JPG or PNG into WebP with adjustable quality. PNG and JPG export too. No account, no watermark."
            }
            Self::Resize => {
                "Change width and height. Fit, crop, or stretch. JPG, WebP, or PNG."
            }
            Self::RemoveBg => {
                "Cut a flat backdrop (white or studio) and download a transparent PNG. Not an AI portrait matte."
            }
            Self::Colors => {
                "Pull a HEX palette from a photo, with approximate share. Copy codes for design or CSS."
            }
        }
    }

    fn eyebrow(self) -> &'static str {
        match self {
            Self::Compress => "Compress to a size",
            Self::Convert => "Convert format",
            Self::Resize => "Resize",
            Self::RemoveBg => "Remove background",
            Self::Colors => "Color palette",
        }
    }

    fn h1(self) -> &'static str {
        match self {
            Self::Compress => "Compress an image to KB",
            Self::Convert => "Convert JPG to WebP",
            Self::Resize => "Resize an image",
            Self::RemoveBg => "Remove a flat background",
            Self::Colors => "Extract colors from an image",
        }
    }

    fn lead_es(self) -> &'static str {
        match self {
            Self::Compress => {
                "Comprime una foto a 50, 200 o 500 KB. JPG, PNG o WebP. Sin cuenta."
            }
            Self::Convert => {
                "Pasa JPG o PNG a WebP (o al revés). Calidad ajustable. Sin marca de agua."
            }
            Self::Resize => {
                "Cambia el ancho y el alto. Encajar, recortar o estirar. JPG, WebP o PNG."
            }
            Self::RemoveBg => {
                "Quita un fondo plano (blanco o estudio) y descarga un PNG transparente. No es un recorte de retrato con IA."
            }
            Self::Colors => {
                "Saca una paleta HEX de la foto, con un porcentaje aproximado."
            }
        }
    }

    fn lead(self) -> &'static str {
        match self {
            Self::Compress => {
                "Set a kilobyte budget. We drop quality first, then scale pixels if needed. No account."
            }
            Self::Convert => {
                "WebP is usually smaller than JPG on the web. Tune quality, or export PNG if you need transparency."
            }
            Self::Resize => {
                "Enter width, height, or both. Fit keeps aspect; fill crops; stretch distorts."
            }
            Self::RemoveBg => {
                "Works on product shots and screenshots with a uniform backdrop. If the subject touches the edge, it can nibble the outline."
            }
            Self::Colors => {
                "We sample the photo and merge similar tones. You get HEX codes ready to copy."
            }
        }
    }

    fn howto(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::Compress => &[
                ("Drop the file", "JPG, PNG, WebP, or GIF (first frame). Free 20 MB. Pro: 50 MB or a ZIP of 20."),
                ("Set the budget", "200 KB is solid for web and email. 50 KB for thumbnails."),
                ("Download", "Quality first; if it still will not fit, we shrink the long edge."),
            ],
            Self::Convert => &[
                ("Drop a JPG or PNG", "WebP and GIF work too. HEIC converts in the browser when it can."),
                ("Pick WebP and quality", "80 is a good photo default. 98+ uses lossless WebP."),
                ("Download the .webp", "Compare size and dimensions under the preview."),
            ],
            Self::Resize => &[
                ("Drop the image", "Same 20 MB limit as the rest of UnderKb."),
                ("Width and height", "One value keeps aspect. Both together use the mode you pick."),
                ("Download", "JPG for photos, PNG for transparency, WebP for a smaller file."),
            ],
            Self::RemoveBg => &[
                ("Flat backdrop", "Studio white or a solid color. A landscape will not work."),
                ("Tune tolerance", "Raise it if you still see a halo. Lower it if it eats the subject."),
                ("Transparent PNG", "Preview uses a checkerboard. Download and drop it into a layout."),
            ],
            Self::Colors => &[
                ("Drop the photo", "Logos, UI, or photos. Very transparent pixels are ignored."),
                ("How many colors", "Between 3 and 12. Near-identical tones are merged."),
                ("Copy HEX", "Each swatch shows the code and an approximate share."),
            ],
        }
    }

    pub fn card_mark(self) -> &'static str {
        match self {
            Self::Compress => "kB",
            Self::Convert => "W",
            Self::Resize => "↔",
            Self::RemoveBg => "✂",
            Self::Colors => "#",
        }
    }

    pub fn card_title(self) -> &'static str {
        match self {
            Self::Compress => "Compress to KB",
            Self::Convert => "JPG to WebP",
            Self::Resize => "Resize",
            Self::RemoveBg => "Remove background",
            Self::Colors => "Extract colors",
        }
    }

    pub fn card_blurb(self) -> &'static str {
        match self {
            Self::Compress => "Hit 50, 200, or 500 KB. Quality first, then scale.",
            Self::Convert => "Turn JPG or PNG into WebP (or back). You set the quality.",
            Self::Resize => "Change width and height. Fit, crop, or stretch.",
            Self::RemoveBg => "White or flat backdrop → transparent PNG. Not a portrait cutout.",
            Self::Colors => "HEX palette with an approximate share. Copy the codes.",
        }
    }

    pub fn card_cta(self) -> &'static str {
        "Open tool"
    }

    pub fn all() -> [Tool; 5] {
        [
            Self::Compress,
            Self::Convert,
            Self::Resize,
            Self::RemoveBg,
            Self::Colors,
        ]
    }

    /// English aliases → Spanish canonicals (not in the sitemap).
    pub fn en_alias(self) -> &'static str {
        match self {
            Self::Compress => "/compress-image-kb",
            Self::Convert => "/jpg-to-webp",
            Self::Resize => "/resize-image",
            Self::RemoveBg => "/remove-background",
            Self::Colors => "/extract-image-colors",
        }
    }

    fn why_title(self) -> &'static str {
        match self {
            Self::Compress => "Why a kilobyte budget beats a quality slider",
            Self::Convert => "When WebP is the right export",
            Self::Resize => "What resize actually changes",
            Self::RemoveBg => "What this cutout is (and is not)",
            Self::Colors => "A palette you can check against the photo",
        }
    }

    fn why(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::Compress => &[
                ("The number is the contract", "Email, LMS, and forms reject files over a cap. A quality slider cannot promise 200 KB."),
                ("Quality first, then pixels", "We lower JPEG/WebP quality before shrinking the long edge, so you keep resolution until the budget forces a scale."),
                ("Honest miss", "If a PNG still will not fit, we say so. We do not pretend every screenshot becomes 50 KB."),
                ("No account", "Upload, download, gone in about 30 minutes. We do not keep a gallery."),
            ],
            Self::Convert => &[
                ("Smaller photos on the web", "WebP at 70–85 is usually smaller than the same JPG. That is the job of this page."),
                ("You pick the format", "Need a JPG for an old form? PNG for transparency? Same dropzone."),
                ("Lossless when you ask", "Quality 98+ uses lossless WebP. Flat UI often prefers that over a mushy 60."),
                ("Same 20 MB limit", "HEIC converts in the browser when the device can decode it."),
            ],
            Self::Resize => &[
                ("One side keeps aspect", "Set only width or only height and we compute the other. No stretch unless you pick stretch."),
                ("Fit vs fill", "Fit letterboxes. Fill crops. Stretch distorts. The labels match what you get."),
                ("Then pick a format", "JPG for photos, PNG for alpha, WebP if you also want it smaller."),
                ("Server cap", "The long edge on the server is 4096 px. Huge camera files are scaled in the browser first."),
            ],
            Self::RemoveBg => &[
                ("Flat color only", "Studio white, a solid backdrop, a screenshot on one color. Not a person-segmentation model."),
                ("Tolerance is the lever", "Raise it to eat a halo. Lower it if it bites the subject."),
                ("PNG out", "You need alpha. JPG cannot hold transparency."),
                ("Honest failures", "Hair on a busy wall, or a subject touching the frame edge, will look rough. We do not advertise portraits."),
            ],
            Self::Colors => &[
                ("Clusters, not eyedropper", "Nearby tones merge. Good for a design start, not for press matching."),
                ("Share is approximate", "Each swatch shows a rough percent so you see dominant vs accent."),
                ("Copy HEX", "Codes are ready for CSS or a brand doc."),
                ("Transparent pixels skipped", "A logo on alpha does not pollute the palette with checkerboard."),
            ],
        }
    }

    fn examples_title(self) -> &'static str {
        match self {
            Self::Compress => "Jobs this compressor is for",
            Self::Convert => "Typical convert jobs",
            Self::Resize => "Typical resize jobs",
            Self::RemoveBg => "When a flat cutout is enough",
            Self::Colors => "When a HEX pull is enough",
        }
    }

    fn examples(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::Compress => &[
                ("Job application form", "The portal says max 200 KB. Default target. Download and attach."),
                ("Course upload", "A 12 MP phone photo will not pass. 500 KB is usually enough for a lecture slide."),
                ("Thumbnail", "50 KB for a card image. Expect a smaller long edge."),
            ],
            Self::Convert => &[
                ("Replace a fat JPG on a page", "Export WebP at 80 and compare bytes under the preview."),
                ("Need JPG after all", "Some government forms still reject WebP. Switch the format, same file."),
                ("Keep a transparent logo", "PNG or lossless WebP. Do not export JPG."),
            ],
            Self::Resize => &[
                ("Avatar 256", "Set width 256, leave height empty, Fit."),
                ("Crop to a banner", "Set both sides and Fill so the subject stays in frame."),
                ("Email inline", "Resize first, then compress to KB if the host still rejects the file."),
            ],
            Self::RemoveBg => &[
                ("Product on white", "Cut the sweep, drop the PNG on a colored landing."),
                ("App screenshot", "Solid desktop wallpaper, then a transparent PNG for the docs."),
                ("Not a selfie", "Use a portrait tool if you need hair detail. This page will not invent one."),
            ],
            Self::Colors => &[
                ("Moodboard", "Drop a reference photo, copy 6 HEX codes into Figma."),
                ("Brand from a logo", "A simple mark usually returns 3–5 tones."),
                ("UI screenshot", "Expect the greys and one accent — that is the point."),
            ],
        }
    }

    fn limits(self) -> &'static str {
        match self {
            Self::Compress => {
                "Free uploads up to 20 MB, one file. A Pro key allows 50 MB and a ZIP of up to 20 images (sequential on this 512 MB machine). Animated GIFs keep the first frame. A huge PNG can still miss a tiny budget or time out. Results live in memory about 30 minutes."
            }
            Self::Convert => {
                "Free 20 MB, Pro 50 MB. HEIC depends on the browser. Batch ZIP is only on Compress. Animated GIFs become a still."
            }
            Self::Resize => {
                "Long edge 4096 px on the server. Stretch will look ugly — that is the mode doing what you asked. We do not upscale a 32 px icon into a sharp poster."
            }
            Self::RemoveBg => {
                "No people model, no hair matte, no busy backgrounds. Tolerance is a color distance, not magic. Output is PNG (or WebP with alpha)."
            }
            Self::Colors => {
                "Not a spectrophotometer. Counts from 3 to 12. Very similar tones collapse. Do not use this for print-critical brand matching."
            }
        }
    }

    fn faq(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::Compress => &[
                ("Can I compress to 200 KB?", "Yes. 200 KB is the default. You can also pick 50, 100, 500, or 1024."),
                ("Do you store my photos?", "The result stays in memory about 30 minutes so you can download it, then it expires."),
                ("JPG, PNG, or WebP?", "Photos usually hit the budget as JPG or WebP. PNG can stay huge."),
                ("Is there an account?", "No. Ads may appear around the tool. The file itself is not paywalled."),
                ("Can I compress a folder?", "Free is one file. A Pro key on /pricing lets you drop up to 20 images and download a ZIP."),
                ("What if it still will not fit?", "We already dropped quality and scaled the long edge. A dense PNG may miss a tiny cap — try JPG or a larger budget."),
            ],
            Self::Convert => &[
                ("Is WebP smaller than JPG?", "For photos, usually, at quality 70–85. Flat graphics sometimes prefer PNG or lossless WebP."),
                ("Is transparency lost?", "If you export JPG, yes (white background). WebP and PNG keep it."),
                ("What quality should I use?", "80 for photos. 98+ for lossless WebP on UI and logos."),
                ("Do you store the file?", "About 30 minutes in memory for the download, then it expires."),
                ("HEIC?", "If the browser can decode it, we convert. If not, export JPG/PNG from the phone first."),
            ],
            Self::Resize => &[
                ("Can I set only the width?", "Yes. Height is computed so it is not stretched. Same the other way."),
                ("Is there a maximum?", "The long edge on the server is 4096 px. Huge camera files are scaled in the browser first."),
                ("Fit, fill, or stretch?", "Fit keeps the whole image. Fill crops to the box. Stretch distorts."),
                ("Can I compress after?", "Yes — open Compress to KB with the resized file if a host still rejects the size."),
                ("Do you store the file?", "About 30 minutes in memory, then it expires."),
            ],
            Self::RemoveBg => &[
                ("Does it cut out a portrait?", "No people model. If the backdrop is not flat or hair hits the edge, the cut will be rough."),
                ("Why PNG?", "You need an alpha channel. JPG has no transparency."),
                ("What is tolerance?", "How far a pixel can be from the backdrop color and still be removed."),
                ("Will it work on a landscape?", "Almost never. Use a studio or solid-color shot."),
                ("Do you store the file?", "About 30 minutes in memory, then it expires."),
            ],
            Self::Colors => &[
                ("Are these exact pixel colors?", "No. We cluster nearby tones. Fine for palettes, not for press matching."),
                ("How many colors come back?", "Up to the count you ask for, if the photo has them. A gray screenshot may return fewer."),
                ("Can I copy all HEX at once?", "Copy each swatch. The share percent is approximate."),
                ("Does it use the preview or the original?", "The uploaded pixels, ignoring very transparent ones."),
                ("Do you store the photo?", "The working file expires after about 30 minutes. We do not keep a palette history."),
            ],
        }
    }
}

/// Public origin for canonicals / JSON-LD. Override with `SITE_URL` when a custom domain is live.
pub fn public_origin() -> String {
    std::env::var("SITE_URL")
        .ok()
        .map(|s| s.trim().trim_end_matches('/').to_string())
        .filter(|s| s.starts_with("http://") || s.starts_with("https://"))
        .unwrap_or_else(|| "https://underkb.fly.dev".into())
}

pub fn canonical_url(path: &str) -> String {
    format!("{}{path}", public_origin())
}

pub fn page(tool: Tool) -> View {
    set_page_title(tool.title());
    set_page_description(tool.description());
    set_page_canonical(canonical_url(tool.path()));

    let faq_ld = json!({
        "@context": "https://schema.org",
        "@type": "FAQPage",
        "mainEntity": tool.faq().iter().map(|(q, a)| json!({
            "@type": "Question",
            "name": q,
            "acceptedAnswer": { "@type": "Answer", "text": a }
        })).collect::<Vec<_>>()
    });
    let page_ld = json!({
        "@context": "https://schema.org",
        "@type": "WebPage",
        "name": tool.title(),
        "description": tool.description(),
        "url": canonical_url(tool.path())
    });

    let howto: Vec<View> = pairs_to_items(tool.howto());
    let why: Vec<View> = pairs_to_items(tool.why());
    let examples: Vec<View> = pairs_to_items(tool.examples());
    let faq: Vec<View> = tool
        .faq()
        .iter()
        .map(|(q, a)| {
            view! {
                <details>
                    <summary>{*q}</summary>
                    <p>{*a}</p>
                </details>
            }
        })
        .collect();

    let form = match tool {
        Tool::Compress => tool::compressor(),
        Tool::Convert => tool::converter(),
        Tool::Resize => tool::resizer(),
        Tool::RemoveBg => tool::remover(),
        Tool::Colors => tool::palette(),
    };

    let more = more_tools(tool);

    // Replace the site-wide kit JSON-LD (WebApplication + homepage FAQ) with this
    // page's WebPage + FAQ, emitted in <head> with the CSP nonce — instead of a
    // raw <script> inside <main> next to a second FAQPage block.
    set_page_json_ld(json!([page_ld, faq_ld]).to_string());

    view! {
        <main class="home-page" lang="en">
            <div class="hero-wrap">
                <div class="hero-particles" data-hero-particles="" aria-hidden="true"></div>
                <section class="hero">
                    <p class="eyebrow">{tool.eyebrow()}</p>
                    <h1>{tool.h1()}</h1>
                    <p class="hero-lead">{tool.lead()}</p>
                    <p class="hero-lead-es" lang="es">{tool.lead_es()}</p>
                    {form}
                </section>
            </div>
            <section class="howto" aria-labelledby="howto-title">
                <h2 id="howto-title">"How to use it"</h2>
                <ol class="howto-grid">{howto}</ol>
            </section>
            <section class="features" aria-labelledby="why-title">
                <h2 id="why-title">{tool.why_title()}</h2>
                <ul class="feature-grid">{why}</ul>
            </section>
            <section class="features" aria-labelledby="ex-title">
                <h2 id="ex-title">{tool.examples_title()}</h2>
                <ul class="feature-grid">{examples}</ul>
            </section>
            <section class="content-section limits">
                <h2>"Limits"</h2>
                <p>{tool.limits()}</p>
            </section>
            {crate::ads::slot("landing-mid", "infeed")}
            <section class="faq" aria-labelledby="faq-title">
                <h2 id="faq-title">"FAQ"</h2>
                <div class="faq-list">{faq}</div>
            </section>
            <section class="features" aria-labelledby="more-title">
                <h2 id="more-title">"Other tools"</h2>
                <ul class="feature-grid">{more}</ul>
            </section>
        </main>
    }
}

fn pairs_to_items(rows: &[(&'static str, &'static str)]) -> Vec<View> {
    rows.iter()
        .map(|(h, p)| {
            view! {
                <li>
                    <h3>{*h}</h3>
                    <p>{*p}</p>
                </li>
            }
        })
        .collect()
}

pub fn seo_footer_links() -> View {
    view! {
        <nav class="seo-links" aria-label="UnderKb tools">
            <NavLink href="/comprimir-imagen-kb">"Compress"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/convertir-jpg-a-webp">"JPG → WebP"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/redimensionar-imagen">"Resize"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/quitar-fondo">"Remove bg"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/extraer-colores-imagen">"Colors"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/privacy">"Privacy"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/terms">"Terms"</NavLink>
            <span aria-hidden="true">" · "</span>
            <NavLink href="/pricing">"Pro"</NavLink>
        </nav>
    }
}

/// Canonical family order. Each app skips itself in the footer and home cards.
const FAMILY: &[(&str, &str, &str)] = &[
    (
        "YouTubeForge",
        "YouTube transcript, MP3, SRT, and translation.",
        "https://youtubetotext.fly.dev",
    ),
    (
        "UnderKb",
        "Compress images to a real KB target. JPG, WebP, PNG.",
        "https://underkb.fly.dev",
    ),
    (
        "PDFForge",
        "Merge, split, compress PDFs. JPG ↔ PDF and extract text.",
        "https://pdfforge.fly.dev",
    ),
    (
        "PlacaQR",
        "3D-printable QR — stand, tile, keychain, or plaque.",
        "https://placaqr.fly.dev",
    ),
    (
        "Billloom",
        "Invoice, quote, and receipt PDFs. No account, no watermark.",
        "https://billloom.fly.dev",
    ),
];

const SELF: &str = "UnderKb";

pub fn sister_apps() -> View {
    let cards = FAMILY
        .iter()
        .copied()
        .filter(|(name, _, _)| *name != SELF)
        .map(|(name, blurb, href)| {
            let name = name.to_string();
            let blurb = blurb.to_string();
            let href = href.to_string();
            view! {
                <li>
                    <a href={href} class="tool-card sister-card" rel="noopener">
                        <h3 class="tool-card-title">{name}</h3>
                        <p class="tool-card-blurb">{blurb}</p>
                        <span class="tool-card-cta">"Open"</span>
                    </a>
                </li>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <nav class="sister-apps" aria-label="Other apps from us">
            <p class="eyebrow">"Also from us"</p>
            <h2>"Free tools, same idea"</h2>
            <p class="hint">
                "No account. Paste, convert, download. Transcripts, PDFs, 3D QR, and invoices."
            </p>
            <ul class="tool-grid">{cards}</ul>
        </nav>
    }
}

pub fn sister_apps_links() -> View {
    let items = FAMILY
        .iter()
        .copied()
        .filter(|(name, _, _)| *name != SELF)
        .enumerate()
        .map(|(i, (name, _, href))| {
            let name = name.to_string();
            let href = href.to_string();
            if i == 0 {
                view! { <a href={href} rel="noopener">{name}</a> }
            } else {
                view! {
                    <span aria-hidden="true">" · "</span>
                    <a href={href} rel="noopener">{name}</a>
                }
            }
        })
        .collect::<Vec<_>>();

    view! {
        <nav class="sister-apps-links" aria-label="Also from us">
            <span>"Also from us:"</span>
            " "
            {items}
        </nav>
    }
}

fn more_tools(current: Tool) -> Vec<View> {
    let all = [
        (Tool::Compress, "Compress to KB", "A real kilobyte cap."),
        (Tool::Convert, "JPG → WebP", "Adjustable quality."),
        (Tool::Resize, "Resize", "Width, height, or crop."),
        (Tool::RemoveBg, "Remove background", "Flat backdrops to PNG."),
        (Tool::Colors, "Extract colors", "HEX palette."),
    ];
    all.into_iter()
        .filter(|(t, _, _)| *t != current)
        .map(|(t, title, blurb)| {
            let href = t.path();
            view! {
                <li>
                    <h3>
                        <NavLink href={href}>{title}</NavLink>
                    </h3>
                    <p>{blurb}</p>
                </li>
            }
        })
        .collect()
}
