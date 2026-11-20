//! About UnderKb — indexable page for visitors and AdSense reviewers.

use resuma::prelude::*;

use crate::landing::{canonical_url, public_origin};

pub fn page(_req: FlowRequest) -> View {
    set_page_title("About UnderKb — Compress Images Under a KB Budget");
    set_page_description(
        "Who builds UnderKb (under200kb.com), what the free image tools do, and how we handle uploads, downloads, and ads. Independent site — not a stock library.",
    );
    set_page_canonical(canonical_url("/about"));
    set_page_json_ld(
        serde_json::json!({
            "@context": "https://schema.org",
            "@type": "AboutPage",
            "name": "About UnderKb",
            "url": canonical_url("/about"),
            "description": "Independent free tools to compress images under a kilobyte budget, convert to WebP, resize, cut flat backgrounds, and extract HEX palettes.",
            "isPartOf": {"@type": "WebSite", "name": "UnderKb", "url": public_origin()},
            "about": {
                "@type": "WebApplication",
                "name": "UnderKb",
                "applicationCategory": "UtilitiesApplication",
                "operatingSystem": "Web",
                "offers": {"@type": "Offer", "price": "0", "priceCurrency": "USD"}
            }
        })
        .to_string(),
    );
    view! {
        <main class="content-section privacy-page">
            <p class="eyebrow">"UnderKb"</p>
            <h1>"About UnderKb"</h1>
            <p class="hero-lead">
                "UnderKb (under200kb.com) is a small, independent website that compresses and transforms images you upload — under a kilobyte budget you choose — without an account."
            </p>

            <h2>"What we build"</h2>
            <p>
                "The main job is compressing a JPG, PNG, WebP, or GIF (first frame) so it fits a size cap. The default is 200 KB for forms, email, and LMS uploads; you can aim lower or higher. Quality drops before we shrink pixels. Related tools on their own pages: convert to WebP, resize or stretch, remove a flat studio-style background (not an AI people matte), and copy a HEX color palette."
            </p>
            <p>
                "We also publish short guides — compress to 200 KB, JPG to WebP, resize, and white-background cut — so people who arrive from search get clear instructions next to the tools."
            </p>

            <h2>"What we are not"</h2>
            <p>
                "We are not a stock photo library, a permanent file host, or an official Google product. Uploaded results live in memory for about thirty minutes so you can download them, then they expire. We do not keep a gallery of other people’s pictures."
            </p>

            <h2>"Editorial approach"</h2>
            <p>
                "Each landing covers one job with complete sentences and how-to steps: compress, convert, resize, background, or palette. Ads, when enabled, sit around the tools and reading pages — not on Privacy, Terms, Pricing, About, or Contact."
            </p>

            <h2>"Copyright and responsible use"</h2>
            <p>
                "Only upload images you have the right to process. Do not use UnderKb to strip watermarks you do not own or to store illegal material. Full rules are on "
                <NavLink href="/terms">"Terms"</NavLink>
                "; data practices are on "
                <NavLink href="/privacy">"Privacy"</NavLink>
                "."
            </p>

            <h2>"Who runs the site"</h2>
            <p>
                "UnderKb is maintained by GoldevLab as a free utility site. Optional Pro keys raise upload limits for heavier jobs — see "
                <NavLink href="/pricing">"Pricing"</NavLink>
                ". Questions and policy requests go through the "
                <NavLink href="/contact">"contact page"</NavLink>
                "."
            </p>

            <h2>"Advertising"</h2>
            <p>
                "If Google AdSense is active, Google and its partners may show ads and use cookies or similar identifiers as described in Privacy and Google’s advertising settings. Publisher "
                <a href="/ads.txt">"ads.txt"</a>
                " lists our publisher id when ads are configured."
            </p>

            <p class="error-actions">
                <NavLink href="/" class="btn btn-primary">"Open the compressor"</NavLink>
                <NavLink href="/guides" class="btn btn-ghost">"Read the guides"</NavLink>
                <NavLink href="/contact" class="btn btn-ghost">"Contact"</NavLink>
            </p>
        </main>
    }
}
