use resuma::prelude::*;

use crate::landing::canonical_url;

pub fn page(_req: FlowRequest) -> View {
    set_page_title("Terms | UnderKb");
    set_page_description(
        "What UnderKb processes, size limits, Pro keys, and what we do not host.",
    );
    set_page_canonical(canonical_url("/terms"));
    view! {
        <main class="content-section privacy-page">
            <p class="eyebrow">"Legal"</p>
            <h1>"Terms of use"</h1>
            <p class="hero-lead">
                "UnderKb compresses, converts, resizes, or samples images you upload. We are not a stock library or a file host."
            </p>
            <h2>"What you get"</h2>
            <p>
                "You drop a file you are allowed to process. We return a smaller or converted copy. Results stay in memory about 30 minutes, then they expire. We do not keep a gallery."
            </p>
            <h2>"Your responsibility"</h2>
            <p>
                "Only upload images you have the right to change. Do not use this app to strip watermarks you do not own, to flood the 512 MB machine, or to store illegal material."
            </p>
            <h2>"Limits and Pro keys"</h2>
            <p>
                "Free is one file up to 20 MB. A key in UNDERKB_PRO_KEYS (header X-Api-Key, Authorization: Bearer, or cookie ukb_pro) raises that to 50 MB and a ZIP of up to 20 compress jobs. Keys are issued by hand — see "
                <NavLink href="/pricing">"/pricing"</NavLink>
                ". A key is not a license to ignore copyright."
            </p>
            <h2>"Ads and availability"</h2>
            <p>
                "The tools are free. Ads may appear. We do not promise every PNG will hit a tiny kilobyte cap, or that the machine is always idle."
            </p>
            <p>
                <NavLink href="/privacy">"Privacy"</NavLink>
                " · "
                <NavLink href="/pricing">"Pro"</NavLink>
            </p>
        </main>
    }
}
