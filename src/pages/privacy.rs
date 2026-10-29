use resuma::prelude::*;

use crate::landing::canonical_url;
use crate::site;

pub fn contact_line() -> View {
    match site::contact_email() {
        Some(email) => {
            let href = format!("mailto:{email}");
            view! {
                <p>
                    "Questions: "
                    <a href={href}>{email}</a>
                    " or "
                    <a href="https://github.com/GoldevLab/underkb/issues" rel="noopener">"GitHub"</a>
                    ". "
                    <NavLink href="/terms">"Terms"</NavLink>
                    " · "
                    <NavLink href="/pricing">"Pro"</NavLink>
                    "."
                </p>
            }
        }
        None => view! {
            <p>
                "Questions: "
                <a href="https://github.com/GoldevLab/underkb/issues" rel="noopener">"open an issue on GitHub"</a>
                ". "
                <NavLink href="/terms">"Terms"</NavLink>
                " · "
                <NavLink href="/pricing">"Pro"</NavLink>
                "."
            </p>
        },
    }
}

pub fn page(_req: FlowRequest) -> View {
    set_page_title("Privacy | UnderKb");
    set_page_description(
        "How UnderKb handles the images you upload, short-lived downloads, ads, and analytics.",
    );
    set_page_canonical(canonical_url("/privacy"));
    view! {
        <main class="content-section privacy-page">
            <p class="eyebrow">"Legal"</p>
            <h1>"Privacy"</h1>
            <p class="hero-lead">
                "UnderKb is a no-account image tool. We do not create user profiles, and we do not keep a gallery of the photos you drop."
            </p>

            <h2>"What we process"</h2>
            <p>
                "When you upload a file we decode it on this server, run the job you asked for (compress, convert, resize, flat-background cut, or palette), and keep the result in memory about 30 minutes so you can download it. Then it expires."
            </p>
            <p>
                "Rate limits use your IP for a short window so the 512 MB machine is not flooded. Optional Pro keys are checked from a header or a cookie you stored after opening ?key=. We do not sell a list of your files."
            </p>

            <h2>"What we do not do"</h2>
            <p>
                "No sign-up, no mailing list, no stored library. Use the tools on images you are allowed to process. We are not affiliated with Google, Adobe, or the camera brands."
            </p>

            <h2>"Advertising"</h2>
            <p>
                "Google AdSense may appear around the tools. This page, Terms, and Pro do not show ads, and we do not ask you to click them. Google and its partners may set cookies or use similar identifiers. See "
                <a href="https://policies.google.com/technologies/ads" rel="noopener">"Google’s advertising policies"</a>
                " and "
                <a href="https://adssettings.google.com/" rel="noopener">"Ad Settings"</a>
                ". Publisher ads.txt is served at "
                <a href="/ads.txt">"/ads.txt"</a>
                " when a publisher id is configured."
            </p>
            <h2>"Browser extension"</h2>
            <p>
                "The optional Chrome extension only opens this website. It does not show Google AdSense inside the popup. Ads stay on these pages."
            </p>

            <h2>"Analytics"</h2>
            <p>
                "Page views may be measured so we can see which tools people open. There is still no account on this site."
            </p>
            <h2>"Contact"</h2>
            {contact_line()}
        </main>
    }
}
