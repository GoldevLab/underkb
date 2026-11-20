//! Public contact page — clear ownership for visitors and AdSense review.

use resuma::prelude::*;

use crate::landing::{canonical_url, public_origin};
use crate::site;

pub fn page(_req: FlowRequest) -> View {
    set_page_title("Contact UnderKb — Support & Policy");
    set_page_description(
        "Email or GitHub contact for UnderKb (under200kb.com). Questions about image compression, privacy, copyright, Pro keys, or ads on this site.",
    );
    set_page_canonical(canonical_url("/contact"));
    set_page_json_ld(
        serde_json::json!({
            "@context": "https://schema.org",
            "@type": "ContactPage",
            "name": "Contact UnderKb",
            "url": canonical_url("/contact"),
            "isPartOf": {"@type": "WebSite", "name": "UnderKb", "url": public_origin()}
        })
        .to_string(),
    );

    let email_block = match site::contact_email() {
        Some(email) => {
            let href = format!("mailto:{email}?subject=UnderKb");
            view! {
                <section aria-labelledby="email-title">
                    <h2 id="email-title">"Email"</h2>
                    <p>
                        "Write to "
                        <a href={href}>{email.clone()}</a>
                        ". Use this for privacy questions, copyright concerns, broken pages, Pro key requests, or AdSense / advertising issues on under200kb.com."
                    </p>
                    <p class="hint">
                        "We read messages about this site only. We cannot recover files after the ~30 minute download window expires."
                    </p>
                </section>
            }
        }
        None => view! {
            <section aria-labelledby="email-title">
                <h2 id="email-title">"Email"</h2>
                <p>
                    "A direct mailbox is being configured. Until then, use GitHub issues below and include under200kb.com in the title."
                </p>
            </section>
        },
    };

    view! {
        <main class="content-section privacy-page">
            <p class="eyebrow">"Support"</p>
            <h1>"Contact UnderKb"</h1>
            <p class="hero-lead">
                "This page is the public contact point for under200kb.com — free image compress, convert, resize, and related tools."
            </p>

            {email_block}

            <section aria-labelledby="gh-title">
                <h2 id="gh-title">"GitHub"</h2>
                <p>
                    "Bug reports and feature ideas: "
                    <a href="https://github.com/GoldevLab/underkb/issues" rel="noopener">
                        "github.com/GoldevLab/underkb/issues"
                    </a>
                    ". Prefer issues for reproducible product bugs; prefer email for privacy or legal requests."
                </p>
            </section>

            <section aria-labelledby="topics-title">
                <h2 id="topics-title">"What to include"</h2>
                <ul>
                    <li>"The page URL (for example /compress-image-kb or a guide path)."</li>
                    <li>"File type and approximate size if a compress job failed — never paste private keys in public issues."</li>
                    <li>"For copyright: the work you claim, your relationship to it, and the URLs involved."</li>
                </ul>
            </section>

            <section aria-labelledby="policy-title">
                <h2 id="policy-title">"Policies"</h2>
                <p>
                    <NavLink href="/privacy">"Privacy"</NavLink>
                    " · "
                    <NavLink href="/terms">"Terms"</NavLink>
                    " · "
                    <NavLink href="/about">"About"</NavLink>
                    " · "
                    <NavLink href="/pricing">"Pro"</NavLink>
                    " · "
                    <a href="/ads.txt">"ads.txt"</a>
                </p>
            </section>

            <p class="error-actions">
                <NavLink href="/" class="btn btn-ghost">"Back to the tool"</NavLink>
                <NavLink href="/guides" class="btn btn-ghost">"Guides"</NavLink>
            </p>
        </main>
    }
}
