use resuma::prelude::*;

use crate::tool::compressor;

pub fn page(_req: FlowRequest) -> View {
    view! {
        <main class="home-page">
            <section class="hero">
                <p class="eyebrow">"Image compressor"</p>
                <h1>"Compress an image under 200 KB"</h1>
                <p class="hero-lead">
                    "Set any size — 50 KB, 200 KB, 1 MB. JPG, WebP, or PNG. Photos up to 20 MB. No account, no watermark."
                </p>
                {compressor()}
            </section>

            <section class="howto" aria-labelledby="howto-title">
                <h2 id="howto-title">"How to compress an image to a size"</h2>
                <ol class="howto-grid">
                    <li>
                        <h3>"Drop the file"</h3>
                        <p>"JPG, PNG, WebP, GIF, or HEIC (Safari converts HEIC). Up to 20 MB. Phone photos and screenshots both work."</p>
                    </li>
                    <li>
                        <h3>"Pick a budget"</h3>
                        <p>"200 KB is the default for web and email. 50 KB for thumbnails. 500 KB if you still want print-ish detail."</p>
                    </li>
                    <li>
                        <h3>"Download"</h3>
                        <p>"JPEG quality first, then scale if needed. WebP here is lossless (good for graphics, often larger than JPG for photos). You get a before/after preview and a file that aims at your budget."</p>
                    </li>
                </ol>
            </section>

            <section class="features" aria-labelledby="why-title">
                <h2 id="why-title">"Why UnderKb instead of a generic compressor"</h2>
                <ul class="feature-grid">
                    <li>
                        <h3>"A real byte target"</h3>
                        <p>"Most tools only expose a quality slider. UnderKb keeps going until the file is under the KB you asked for — and tells you if it cannot quite get there."</p>
                    </li>
                    <li>
                        <h3>"Web-ready formats"</h3>
                        <p>"JPEG for photos, lossless WebP for graphics, PNG when you still need transparency (it may need a smaller canvas)."</p>
                    </li>
                    <li>
                        <h3>"No sign-up"</h3>
                        <p>"Compress, download, done. We do not keep a gallery of your pictures."</p>
                    </li>
                    <li>
                        <h3>"Built for Fly"</h3>
                        <p>"One Rust binary. No Node image farm. Same stack as a serious micro SaaS, not a PHP upload form from 2012."</p>
                    </li>
                </ul>
            </section>

            <section class="faq" aria-labelledby="faq-title">
                <h2 id="faq-title">"FAQ"</h2>
                <div class="faq-list">
                    <details>
                        <summary>"Can I compress a JPG to 200 KB?"</summary>
                        <p>"Yes. 200 KB is the default. Change the box or tap a chip (50, 100, 200, 500, 1024)."</p>
                    </details>
                    <details>
                        <summary>"Does this work on PNG and WebP too?"</summary>
                        <p>"Yes. Input: JPG, PNG, WebP, GIF (first frame), and HEIC in browsers that can decode it. Output: JPG, lossless WebP, or PNG."</p>
                    </details>
                    <details>
                        <summary>"Will you store my image?"</summary>
                        <p>"We keep the result in memory for about 30 minutes so you can download it, then it expires. We do not build a user library."</p>
                    </details>
                    <details>
                        <summary>"Why did the image get smaller in pixels?"</summary>
                        <p>"If quality alone cannot hit the budget, we scale the longest edge down. Tiny budgets on huge phone photos need that."</p>
                    </details>
                    <details>
                        <summary>"Is there a file size limit?"</summary>
                        <p>"20 MB upload. Huge photos are resized in the browser first. Very large megapixel images are capped on the server so it stays fast."</p>
                    </details>
                </div>
            </section>
        </main>
    }
}
