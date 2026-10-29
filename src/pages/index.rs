use resuma::prelude::*;
use serde_json::json;

use crate::landing::{canonical_url, Tool};
use crate::tool;

pub fn page(_req: FlowRequest) -> View {
    crate::ads::begin_page();
    set_page_title("Compress an image to 200 KB — free | UnderKb");
    set_page_description(
        "Free image compressor. The home dropzone hits a real kilobyte cap (default 200 KB, minimum 8 KB). Convert, resize or stretch, flat-background cut, HEX palette, and a Chrome KB extension. No account.",
    );
    set_page_canonical(canonical_url("/"));
    set_page_json_ld(home_json_ld());

    let cards: Vec<View> = Tool::all().into_iter().map(tool_card).collect();

    view! {
        <main class="home-page" lang="en">
            <div class="hero-wrap">
                <div class="hero-particles" data-hero-particles="" aria-hidden="true"></div>
                <section class="hero">
                    <p class="eyebrow">"Free image compressor"</p>
                    <h1>"Compress an image to KB"</h1>
                    <p class="hero-lead">
                        "Set a kilobyte budget. We drop quality first, then scale pixels if needed. No account. Convert, resize, and the other jobs live on their own pages — same dropzone idea."
                    </p>
                    {tool::compressor()}
                </section>
            </div>
            <section class="howto" aria-labelledby="howto-title">
                <h2 id="howto-title">"How it works"</h2>
                <ol class="howto-grid">
                    <li>
                        <h3>"Drop the image"</h3>
                        <p>"JPG, PNG, WebP, or GIF (first frame). Free 20 MB, one file. A Pro key on /pricing allows 50 MB or a ZIP of 20. On a phone you can pick from the camera roll and Share the result."</p>
                    </li>
                    <li>
                        <h3>"Set the budget"</h3>
                        <p>"200 KB is the default — forms, email, and LMS caps. 50 KB for thumbnails. 500 KB for a lecture slide."</p>
                    </li>
                    <li>
                        <h3>"Download"</h3>
                        <p>"The file stays available for about 30 minutes. We do not keep a gallery of your pictures."</p>
                    </li>
                </ol>
            </section>
            {crate::ads::slot("home-howto", "article")}
            <section class="tool-pick" aria-labelledby="jobs-title">
                <h2 id="jobs-title">"Other jobs"</h2>
                <p class="hint">
                    "Convert to WebP, resize or stretch, cut a flat background, or copy a HEX palette. The compressor on this page is the default."
                </p>
                <div class="tool-grid">{cards}</div>
            </section>
            <section class="tool-pick" aria-labelledby="guides-title">
                <h2 id="guides-title">"Guides"</h2>
                <p class="hint">"The same jobs, written out: 200 KB, WebP, resize, and a white background."</p>
                {crate::guides::home_strip()}
            </section>
            <section class="faq" aria-labelledby="faq-title">
                <h2 id="faq-title">"FAQ"</h2>
                <div class="faq-list">
                    <details>
                        <summary>"Can I compress a JPG to 200 KB?"</summary>
                        <p>"Yes. 200 KB is the default target. You can set 50, 100, 500, or 1024 KB."</p>
                    </details>
                    <details>
                        <summary>"Does UnderKb store my images?"</summary>
                        <p>"The compressed file is kept in memory for about 30 minutes so you can download it, then it expires."</p>
                    </details>
                    <details>
                        <summary>"Do I need an account?"</summary>
                        <p>"No. Drop a file, download. Ads may appear around the tool."</p>
                    </details>
                    <details>
                        <summary>"Does remove background work on portraits?"</summary>
                        <p>"Only flat backdrops (studio, white). It is not an AI people matte. Open Remove background for that job."</p>
                    </details>
                    <details>
                        <summary>"Is there a KB extension?"</summary>
                        <p>
                            "Yes. The "
                            <NavLink href="/extension">"Chrome extension"</NavLink>
                            " opens compress, WebP, resize, a flat-background cut, and the palette. The file is still processed on this site."
                        </p>
                    </details>
                    <details>
                        <summary>"Can I stretch an image?"</summary>
                        <p>
                            "Open "
                            <NavLink href="/redimensionar-imagen">"Resize"</NavLink>
                            " and choose Stretch. Fit keeps the aspect ratio. Fill crops to the box."
                        </p>
                    </details>
                    <details>
                        <summary>"Can I compress to 2 KB?"</summary>
                        <p>"The smallest target is 8 KB. A photo that small is scaled down hard. 200 KB is the default for forms and email."</p>
                    </details>
                </div>
            </section>
            {crate::landing::sister_apps()}
        </main>
    }
}

fn home_json_ld() -> String {
    let origin = canonical_url("");
    json!({
        "@context": "https://schema.org",
        "@graph": [
            {
                "@type": "Organization",
                "@id": format!("{origin}/#org"),
                "name": "UnderKb",
                "url": origin
            },
            {
                "@type": "WebSite",
                "@id": format!("{origin}/#website"),
                "name": "UnderKb",
                "url": origin,
                "publisher": {"@id": format!("{origin}/#org")}
            },
            {
                "@type": "WebApplication",
                "name": "UnderKb",
                "alternateName": ["compress image to 200kb", "image compressor"],
                "url": origin,
                "applicationCategory": "UtilitiesApplication",
                "operatingSystem": "Web",
                "offers": {"@type": "Offer", "price": "0", "priceCurrency": "USD"},
                "description": "Free image compressor with a real kilobyte cap. Default 200 KB, minimum 8 KB. JPG, WebP, PNG. No account."
            },
            {
                "@type": "HowTo",
                "name": "Compress an image to KB",
                "description": "Set a kilobyte budget. We drop quality first, then scale pixels if needed.",
                "step": [
                    {
                        "@type": "HowToStep",
                        "position": 1,
                        "name": "Drop the image",
                        "text": "JPG, PNG, WebP, or GIF (first frame). Free 20 MB, one file. A Pro key on /pricing allows 50 MB or a ZIP of 20. On a phone you can pick from the camera roll and Share the result."
                    },
                    {
                        "@type": "HowToStep",
                        "position": 2,
                        "name": "Set the budget",
                        "text": "200 KB is the default — forms, email, and LMS caps. 50 KB for thumbnails. 500 KB for a lecture slide."
                    },
                    {
                        "@type": "HowToStep",
                        "position": 3,
                        "name": "Download",
                        "text": "The file stays available for about 30 minutes. We do not keep a gallery of your pictures."
                    }
                ]
            },
            {
                "@type": "FAQPage",
                "mainEntity": [
                    {
                        "@type": "Question",
                        "name": "Can I compress a JPG to 200 KB?",
                        "acceptedAnswer": {"@type": "Answer", "text": "Yes. 200 KB is the default target. You can set 50, 100, 500, or 1024 KB."}
                    },
                    {
                        "@type": "Question",
                        "name": "Does UnderKb store my images?",
                        "acceptedAnswer": {"@type": "Answer", "text": "The compressed file is kept in memory for about 30 minutes so you can download it, then it expires."}
                    },
                    {
                        "@type": "Question",
                        "name": "Do I need an account?",
                        "acceptedAnswer": {"@type": "Answer", "text": "No. Drop a file, download. Ads may appear around the tool."}
                    },
                    {
                        "@type": "Question",
                        "name": "Does remove background work on portraits?",
                        "acceptedAnswer": {"@type": "Answer", "text": "Only flat backdrops (studio, white). It is not an AI people matte. Open Remove background for that job."}
                    },
                    {
                        "@type": "Question",
                        "name": "Is there a KB extension?",
                        "acceptedAnswer": {"@type": "Answer", "text": "Yes. The Chrome extension opens compress, WebP, resize, a flat-background cut, and the palette. The file is still processed on this site."}
                    },
                    {
                        "@type": "Question",
                        "name": "Can I stretch an image?",
                        "acceptedAnswer": {"@type": "Answer", "text": "Open Resize and choose Stretch. Fit keeps the aspect ratio. Fill crops to the box."}
                    },
                    {
                        "@type": "Question",
                        "name": "Can I compress to 2 KB?",
                        "acceptedAnswer": {"@type": "Answer", "text": "The smallest target is 8 KB. A photo that small is scaled down hard. 200 KB is the default for forms and email."}
                    }
                ]
            }
        ]
    })
    .to_string()
}

fn tool_card(tool: Tool) -> View {
    let href = tool.path();
    let mark = tool.card_mark();
    let title = tool.card_title();
    let blurb = tool.card_blurb();
    let cta = tool.card_cta();
    view! {
        <NavLink href={href} class="tool-card">
            <span class="tool-card-mark" aria-hidden="true">{mark}</span>
            <h3 class="tool-card-title">{title}</h3>
            <p class="tool-card-blurb">{blurb}</p>
            <span class="tool-card-cta">{cta}</span>
        </NavLink>
    }
}
