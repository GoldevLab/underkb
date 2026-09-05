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
                "Compress a photo to 50, 200, or 500 KB. JPG, PNG, or WebP. No account. Uploads up to 20 MB."
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
                ("Drop the file", "JPG, PNG, WebP, or GIF (first frame). Up to 20 MB."),
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

    fn faq(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::Compress => &[
                ("Can I compress to 200 KB?", "Yes. 200 KB is the default. You can also pick 50, 100, 500, or 1024."),
                ("Do you store my photos?", "The result stays in memory about 30 minutes so you can download it, then it expires."),
            ],
            Self::Convert => &[
                ("Is WebP smaller than JPG?", "For photos, usually, at quality 70–85. Flat graphics sometimes prefer PNG or lossless WebP."),
                ("Is transparency lost?", "If you export JPG, yes (white background). WebP and PNG keep it."),
            ],
            Self::Resize => &[
                ("Can I set only the width?", "Yes. Height is computed so it is not stretched. Same the other way."),
                ("Is there a maximum?", "The long edge on the server is 4096 px. Huge camera files are scaled in the browser first."),
            ],
            Self::RemoveBg => &[
                ("Does it cut out a portrait?", "No people model. If the backdrop is not flat or hair hits the edge, the cut will be rough."),
                ("Why PNG?", "You need an alpha channel. JPG has no transparency."),
            ],
            Self::Colors => &[
                ("Are these exact pixel colors?", "No. We cluster nearby tones. Fine for palettes, not for press matching."),
                ("How many colors come back?", "Up to the count you ask for, if the photo has them. A gray screenshot may return fewer."),
            ],
        }
    }
}

pub fn page(tool: Tool) -> View {
    // Every tool page used to ship the homepage <title>/description: fatal for a
    // keyword-targeted multi-page site. Stage per-page SEO on this response.
    set_page_title(tool.title());
    set_page_description(tool.description());

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
        "url": format!("https://underkb.fly.dev{}", tool.path())
    });

    let howto: Vec<View> = tool
        .howto()
        .iter()
        .map(|(h, p)| {
            view! {
                <li>
                    <h3>{*h}</h3>
                    <p>{*p}</p>
                </li>
            }
        })
        .collect();
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
                    {form}
                </section>
            </div>
            {crate::ads::slot("landing-hero", "infeed")}
            <section class="howto" aria-labelledby="howto-title">
                <h2 id="howto-title">"How to use it"</h2>
                <ol class="howto-grid">{howto}</ol>
            </section>
            {crate::ads::slot("landing-mid", "infeed")}
            <section class="faq" aria-labelledby="faq-title">
                <h2 id="faq-title">"FAQ"</h2>
                <div class="faq-list">{faq}</div>
            </section>
            {crate::ads::slot("landing-faq", "infeed")}
            <section class="features" aria-labelledby="more-title">
                <h2 id="more-title">"Other tools"</h2>
                <ul class="feature-grid">{more}</ul>
            </section>
        </main>
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
