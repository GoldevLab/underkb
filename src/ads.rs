//! Reserved AdSense slots — empty frames until units are wired.
//!
//! Each `data-ad` value is a stable placement id for a future AdSense unit:
//! footer and anchor live in chrome; in-page units use home-* and landing-*.

use resuma::prelude::*;

pub fn slot(placement: &'static str, size: &'static str) -> View {
    let class = format!("ad-slot ad-slot-{size}");
    view! {
        <aside class={class} data-ad={placement} aria-label="Advertisement">
            <div class="ad-slot-frame">
                <span class="ad-slot-label">"Ad"</span>
            </div>
        </aside>
    }
}
