use resuma::prelude::*;

use crate::landing::canonical_url;
use crate::site;

pub fn page(_req: FlowRequest) -> View {
    set_page_title("Chrome extension | UnderKb");
    set_page_description(
        "Open UnderKb from the toolbar: compress to KB, JPG to WebP, resize, remove a flat background, palette.",
    );
    set_page_canonical(canonical_url("/extension"));
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
            <h1>"UnderKb extension"</h1>
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
            <p>
                <NavLink href="/" class="btn btn-ghost">"Use the website"</NavLink>
            </p>
        </main>
    }
}
