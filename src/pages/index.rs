use resuma::prelude::*;

use crate::landing::Tool;

pub fn page(_req: FlowRequest) -> View {
    let cards: Vec<View> = [
        Tool::Compress,
        Tool::Convert,
        Tool::Resize,
        Tool::RemoveBg,
        Tool::Colors,
    ]
    .into_iter()
    .map(tool_card)
    .collect();

    view! {
        <main class="home-page home-hub" lang="en">
            <section class="hero">
                <p class="eyebrow">"Image tools"</p>
                <h1>"What do you want to do with your photo?"</h1>
                <p class="hero-lead">
                    "Pick a tool, then drop the file there. No account, no watermark. Up to 20 MB."
                </p>
            </section>
            {crate::ads::slot("home-hero", "infeed")}
            <section class="tool-pick" aria-labelledby="tools-title">
                <h2 id="tools-title">"Tools"</h2>
                <div class="tool-grid">{cards}</div>
            </section>
            {crate::ads::slot("home-mid", "infeed")}
            <section class="howto" aria-labelledby="howto-title">
                <h2 id="howto-title">"How it works"</h2>
                <ol class="howto-grid">
                    <li>
                        <h3>"Pick a job"</h3>
                        <p>"Compress to a size, convert to WebP, resize, cut a flat background, or pull a color palette."</p>
                    </li>
                    <li>
                        <h3>"Drop the image"</h3>
                        <p>"JPG, PNG, WebP, or GIF. On a phone you can pick from the camera roll."</p>
                    </li>
                    <li>
                        <h3>"Download"</h3>
                        <p>"The file stays available for about 30 minutes. We do not keep a gallery of your pictures."</p>
                    </li>
                </ol>
            </section>
            {crate::ads::slot("home-faq", "infeed")}
            <section class="faq" aria-labelledby="faq-title">
                <h2 id="faq-title">"FAQ"</h2>
                <div class="faq-list">
                    <details>
                        <summary>"Do I need an account?"</summary>
                        <p>"No. Open a tool, upload, download."</p>
                    </details>
                    <details>
                        <summary>"Where do I compress to 200 KB?"</summary>
                        <p>"Compress to KB. 200 KB is the default target."</p>
                    </details>
                    <details>
                        <summary>"Does remove background work on portraits?"</summary>
                        <p>"Only flat backdrops (studio, white). It is not an AI people matte."</p>
                    </details>
                </div>
            </section>
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
