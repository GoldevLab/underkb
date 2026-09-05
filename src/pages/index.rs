use resuma::prelude::*;

use crate::landing::{canonical_url, Tool};
use crate::tool;

pub fn page(_req: FlowRequest) -> View {
    set_page_title("UnderKb — compress images to a KB budget, plus convert and resize");
    set_page_description(
        "Free image tools. The home dropzone hits a real kilobyte cap (default 200 KB). Convert, resize, flat-background cut, and HEX palette have their own pages. No account.",
    );
    set_page_canonical(canonical_url("/"));

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
            <section class="tool-pick" aria-labelledby="jobs-title">
                <h2 id="jobs-title">"Other jobs"</h2>
                <p class="hint">
                    "Each landing is a different task so search can find it. The compressor on this page is the default tool."
                </p>
                <div class="tool-grid">{cards}</div>
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
                </div>
            </section>
            {crate::ads::slot("home-faq", "infeed")}
        </main>
    }
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
