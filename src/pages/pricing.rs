use resuma::prelude::*;

use crate::landing::canonical_url;
use crate::site;

pub fn page(_req: FlowRequest) -> View {
    set_page_title("Pro limits | UnderKb");
    set_page_description(
        "Free image tools with a 20 MB cap. Request a Pro key for 50 MB uploads and a ZIP of up to 20 compress jobs.",
    );
    set_page_canonical(canonical_url("/pricing"));
    let contact = site::contact_email();
    let mail = contact
        .as_ref()
        .map(|e| format!("mailto:{e}?subject=UnderKb%20Pro%20key"))
        .unwrap_or_else(|| "https://github.com/GoldevLab/underkb/issues".into());
    let mail_label = if contact.is_some() {
        "Email for a Pro key"
    } else {
        "Request a key on GitHub"
    };
    view! {
        <main class="content-section privacy-page">
            <p class="eyebrow">"Limits"</p>
            <h1>"Pro uploads and batch ZIP"</h1>
            <p class="hero-lead">
                "The website stays free. A hand-issued key raises the file cap so a 512 MB machine is not an open hopper for 50 MB dumps from every IP."
            </p>
            <h2>"Free (no key)"</h2>
            <p>"One image at a time, up to 20 MB. Compress, convert, resize, flat cutout, palette. Ads may appear."</p>
            <h2>"Pro (UNDERKB_PRO_KEYS)"</h2>
            <p>
                "Send X-Api-Key or Authorization: Bearer, or open the site with ?key= and we store it in this browser. Then: 50 MB per file, and on Compress a ZIP of up to 20 images (processed one after another)."
            </p>
            <pre class="recap-body">{"POST /api/compress-batch\nX-Api-Key: YOUR_KEY\n# multipart fields: file (repeat), target_kb, format"}</pre>
            <p>"No card form on this page. We issue keys by hand."</p>
            <p>
                <a class="btn btn-primary" href={mail}>{mail_label}</a>
                " "
                <NavLink href="/" class="btn btn-ghost">"Open compressor"</NavLink>
            </p>
            <p class="hint">
                "A key is not permission to ignore copyright. See "
                <NavLink href="/terms">"Terms"</NavLink>
                "."
            </p>
        </main>
    }
}
