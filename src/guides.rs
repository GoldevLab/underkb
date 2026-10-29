//! Crawlable guides that answer a search and end on the tool.

use resuma::prelude::*;
use serde_json::json;

use crate::landing::canonical_url;

enum Job {
    Compress,
    Convert,
    Resize,
    Background,
}

struct Guide {
    path: &'static str,
    title: &'static str,
    description: &'static str,
    h1: &'static str,
    lead: &'static str,
    job: Job,
    sections: &'static [(&'static str, &'static str)],
    faq: &'static [(&'static str, &'static str)],
    links: &'static [(&'static str, &'static str)],
}

const ALL: &[Guide] = &[COMPRESS, WEBP, RESIZE, BACKGROUND];

const COMPRESS: Guide = Guide {
    path: "/guides/compress-image-to-200kb",
    title: "Compress an Image to 200 KB | UnderKb",
    description: "How to compress a JPG, PNG, or WebP to 200 KB. UnderKb hits a real kilobyte budget, then you download the file. No account.",
    h1: "Compress an image to 200 KB",
    lead: "200 KB is the cap most forms, email hosts, and course sites ask for. Drop the file, leave the target at 200, and download the result. Quality drops first. Pixels shrink only if the budget still is not met.",
    job: Job::Compress,
    sections: &[
        (
            "Set the kilobyte budget, not a quality slider",
            "A quality slider cannot promise 200 KB. The number on this page is the contract. 50 KB fits a thumbnail. 500 KB fits a lecture slide. The smallest target is 8 KB, and a photo aimed that low is scaled down hard.",
        ),
        (
            "What the file can be",
            "JPG, PNG, WebP, GIF, and HEIC when the browser can decode it. An animated GIF keeps the first frame. Free uploads are one file up to 20 MB. A Pro key raises that to 50 MB and a ZIP of up to 20 compress jobs.",
        ),
        (
            "When 200 KB is the wrong target",
            "A screenshot full of text often stays sharp as JPG or WebP and blows up as PNG. If the result still misses the budget, the page says so instead of pretending the file fits. Try JPG, or raise the target.",
        ),
        (
            "What we keep",
            "The result stays in memory about 30 minutes so you can download it, then it expires. There is no account and no gallery.",
        ),
    ],
    faq: &[
        (
            "Can I compress a JPG to 200 KB?",
            "Yes. 200 KB is the default. You can also pick 50, 100, 500, or 1024.",
        ),
        (
            "Will a PNG of a photo hit 200 KB?",
            "Often no. Photos belong in JPG or WebP. PNG is for flat color and transparency.",
        ),
        (
            "Do I need an account?",
            "No. Drop the file and download.",
        ),
    ],
    links: &[
        ("/comprimir-imagen-kb", "Open the compressor"),
        ("/guides/jpg-to-webp", "Convert the result to WebP"),
        ("/guides/resize-image", "Resize before you compress"),
    ],
};

const WEBP: Guide = Guide {
    path: "/guides/jpg-to-webp",
    title: "Convert JPG to WebP | UnderKb",
    description: "Turn a JPG or PNG into WebP, or back into JPG. You set the quality. No account.",
    h1: "Convert a JPG to WebP",
    lead: "WebP at a quality around 70–85 is usually smaller than the same photo as JPG. Drop the file, pick WebP, and download. If a form still wants JPG, convert back on the same page.",
    job: Job::Convert,
    sections: &[
        (
            "When WebP is the right export",
            "Use it for photos on a page you control. Skip it when the host only accepts JPG, which is common on older forms and some printers.",
        ),
        (
            "Quality is yours",
            "You pick the number. Quality 98 or higher uses lossless WebP, which suits flat UI better than a mushy 60. Photos usually want something in the 70s.",
        ),
        (
            "Transparency",
            "PNG and WebP can keep a transparent background. JPG cannot. If you just cut a white backdrop, export PNG or WebP, not JPG.",
        ),
        (
            "Size after the conversion",
            "This page changes the format. If you also need a hard cap such as 200 KB, open Compress to KB with the file you just saved.",
        ),
    ],
    faq: &[
        (
            "Can I turn WebP back into JPG?",
            "Yes. Pick JPG on the same dropzone.",
        ),
        (
            "Does HEIC work?",
            "When the browser can decode it. The upload limit is still 20 MB on the free tool.",
        ),
        (
            "Is WebP always smaller?",
            "On photos, usually. A tiny icon can be smaller as PNG.",
        ),
    ],
    links: &[
        ("/convertir-jpg-a-webp", "Open JPG to WebP"),
        ("/guides/compress-image-to-200kb", "Then compress to 200 KB"),
        ("/guides/remove-white-background", "Cut a flat background first"),
    ],
};

const RESIZE: Guide = Guide {
    path: "/guides/resize-image",
    title: "Resize an Image for a Form or Email | UnderKb",
    description: "Change width and height. Fit keeps the whole photo, fill crops, stretch distorts. No account.",
    h1: "Resize an image to the pixels a form asks for",
    lead: "Set the width, the height, or both. One side alone keeps the aspect ratio. Fit shows the whole photo. Fill crops to cover the box. Stretch distorts, and the label says so.",
    job: Job::Resize,
    sections: &[
        (
            "Width or height, not both, keeps the shape",
            "A passport-style box wants both numbers and Fill or Fit. A banner that must stay wide wants one side and the ratio left alone.",
        ),
        (
            "The server cap",
            "The long edge on the server is 4096 pixels. A camera file larger than that is scaled in the browser before upload.",
        ),
        (
            "Pixels are not kilobytes",
            "A resized photo can still be too heavy for email. Save it, then open Compress to KB if the host rejects the file.",
        ),
        (
            "Format after the resize",
            "JPG for photos, PNG when you need a transparent edge, WebP when the page can take it.",
        ),
    ],
    faq: &[
        (
            "Will this stretch my photo?",
            "Only if you choose stretch. Fit and a single side do not.",
        ),
        (
            "Can I resize to a kilobyte size?",
            "This page changes pixels. The kilobyte budget is the compress tool.",
        ),
        (
            "What is the largest edge?",
            "4096 pixels on the server.",
        ),
    ],
    links: &[
        ("/redimensionar-imagen", "Open resize"),
        ("/guides/compress-image-to-200kb", "Compress after you resize"),
        ("/guides/jpg-to-webp", "Export as WebP"),
    ],
};

const BACKGROUND: Guide = Guide {
    path: "/guides/remove-white-background",
    title: "Remove a White Background from an Image | UnderKb",
    description: "Cut a flat white or solid backdrop to a transparent PNG. Not a portrait cutout. No account.",
    h1: "Remove a white background",
    lead: "This cuts a flat color: studio white, a solid backdrop, a screenshot on one color. It is not a person-segmentation model. Hair on a busy wall will look rough, and the page does not claim portraits.",
    job: Job::Background,
    sections: &[
        (
            "Tolerance is the lever",
            "Raise it to eat a gray halo around the subject. Lower it if the cut starts eating the object. Start near the default and move once.",
        ),
        (
            "The file has to be PNG or WebP",
            "JPG cannot hold transparency. The download is a PNG so the checkerboard stays empty where the white was.",
        ),
        (
            "What fails honestly",
            "A subject that touches the frame edge, a shadow that is not the backdrop color, or a photo of a person. Use a real cutout tool for those.",
        ),
        (
            "After the cut",
            "If the PNG is too large for the form, compress it or export WebP. A photo-sized PNG often misses a 200 KB cap.",
        ),
    ],
    faq: &[
        (
            "Does this remove the background from a portrait?",
            "No. Flat color only.",
        ),
        (
            "Why is the edge fringed?",
            "Raise the tolerance a little. If the subject is close to white, a high tolerance will bite it.",
        ),
        (
            "Can I download a JPG?",
            "A JPG would paint the transparent pixels a color. Keep PNG or WebP.",
        ),
    ],
    links: &[
        ("/quitar-fondo", "Open remove background"),
        ("/guides/jpg-to-webp", "Export the cut as WebP"),
        ("/guides/compress-image-to-200kb", "Compress if the PNG is heavy"),
    ],
};

pub fn render(path: &str) -> View {
    let guide = ALL.iter().find(|g| g.path == path).unwrap_or(&COMPRESS);
    crate::ads::begin_page();
    set_page_title(guide.title);
    set_page_description(guide.description);
    set_page_canonical(canonical_url(guide.path));
    set_page_json_ld(guide_json_ld(guide));

    let sections: Vec<View> = guide
        .sections
        .iter()
        .map(|(h, p)| {
            view! {
                <section class="guide-section">
                    <h2>{*h}</h2>
                    <p>{*p}</p>
                </section>
            }
        })
        .collect();
    let faq: Vec<View> = guide
        .faq
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
    let related: Vec<View> = guide
        .links
        .iter()
        .map(|(href, label)| {
            let href = (*href).to_string();
            view! {
                <li>
                    <NavLink href={href}>{*label}</NavLink>
                </li>
            }
        })
        .collect();
    let tool = tool_view(&guide.job);

    view! {
        <main class="home-page guide-page" lang="en">
            <div class="hero-wrap">
                <div class="hero-particles" data-hero-particles="" aria-hidden="true"></div>
                <section class="hero">
                    <p class="eyebrow">"Guide"</p>
                    <h1>{guide.h1}</h1>
                    <p class="hero-lead">{guide.lead}</p>
                    {tool}
                </section>
            </div>
            <article class="content-section guide-article">
                {sections}
                <h2>"Related"</h2>
                <ul class="guide-links">{related}</ul>
                <p class="guide-actions">
                    <NavLink href="/guides" class="btn btn-ghost">"All guides"</NavLink>
                </p>
            </article>
            <section class="faq" aria-labelledby="guide-faq">
                <h2 id="guide-faq">"FAQ"</h2>
                <div class="faq-list">{faq}</div>
            </section>
            {crate::ads::slot("guide-end", "article")}
        </main>
    }
}

pub fn index_page() -> View {
    crate::ads::begin_page();
    set_page_title("Image guides — 200 KB, WebP, resize, white background | UnderKb");
    set_page_description(
        "How to compress an image to 200 KB, convert JPG to WebP, resize for a form, and remove a white background. Each guide includes the tool.",
    );
    set_page_canonical(canonical_url("/guides"));
    set_page_json_ld(index_json_ld());

    view! {
        <main class="content-section guide-page">
            <p class="eyebrow">"Learn"</p>
            <h1>"UnderKb guides"</h1>
            <p class="hero-lead">
                "Short pages for the jobs people search for. Each one includes the tool."
            </p>
            {home_strip()}
            {crate::ads::slot("guides-index", "article")}
            <p class="guide-actions">
                <NavLink href="/" class="btn btn-primary">"Compress to KB"</NavLink>
                <NavLink href="/extension" class="btn btn-ghost">"Chrome extension"</NavLink>
            </p>
        </main>
    }
}

pub fn home_strip() -> View {
    let cards: Vec<View> = ALL
        .iter()
        .map(|g| {
            let href = g.path.to_string();
            view! {
                <li>
                    <a href={href} class="related-card" data-r-nav="true">
                        <strong>{g.h1}</strong>
                        <span>{g.description}</span>
                    </a>
                </li>
            }
        })
        .collect();
    view! { <ul class="related-grid">{cards}</ul> }
}

fn index_json_ld() -> String {
    let origin = crate::landing::public_origin();
    let items: Vec<_> = ALL
        .iter()
        .enumerate()
        .map(|(i, g)| {
            json!({
                "@type": "ListItem",
                "position": i + 1,
                "name": g.h1,
                "url": format!("{origin}{}", g.path)
            })
        })
        .collect();
    json!({
        "@context": "https://schema.org",
        "@type": "ItemList",
        "name": "UnderKb guides",
        "itemListElement": items
    })
    .to_string()
}

fn tool_view(job: &Job) -> View {
    match job {
        Job::Compress => crate::tool::compressor(),
        Job::Convert => crate::tool::converter(),
        Job::Resize => crate::tool::resizer(),
        Job::Background => crate::tool::remover(),
    }
}

fn guide_json_ld(g: &Guide) -> String {
    let origin = crate::landing::public_origin();
    let faq_entities: Vec<_> = g
        .faq
        .iter()
        .map(|(q, a)| {
            json!({
                "@type": "Question",
                "name": q,
                "acceptedAnswer": {"@type": "Answer", "text": a}
            })
        })
        .collect();
    let steps: Vec<_> = g
        .sections
        .iter()
        .take(3)
        .enumerate()
        .map(|(i, (name, text))| {
            json!({
                "@type": "HowToStep",
                "position": i + 1,
                "name": name,
                "text": text
            })
        })
        .collect();
    let doc = json!({
        "@context": "https://schema.org",
        "@graph": [
            {
                "@type": "Article",
                "headline": g.h1,
                "description": g.description,
                "mainEntityOfPage": format!("{origin}{}", g.path),
                "author": {"@type": "Organization", "name": "UnderKb"},
                "publisher": {"@type": "Organization", "name": "UnderKb", "url": origin}
            },
            {
                "@type": "HowTo",
                "name": g.h1,
                "description": g.description,
                "step": steps
            },
            {
                "@type": "FAQPage",
                "mainEntity": faq_entities
            }
        ]
    });
    serde_json::to_string(&doc).unwrap_or_else(|_| "{}".into())
}
