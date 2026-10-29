//! Google AdSense. Layout is reserved; live units when ADSENSE_CLIENT + slot are set.
//! One ad-unit id is emitted once per page. A repeated id becomes an empty node
//! so the same unit is not stacked (AdSense fills one unit per page).

use std::cell::RefCell;

use resuma::prelude::*;
use resuma::server::CspConfig;

const CLIENT_ENV: &str = "ADSENSE_CLIENT";

const ADSENSE_ORIGINS: &[&str] = &[
    "https://pagead2.googlesyndication.com",
    "https://googleads.g.doubleclick.net",
    "https://tpc.googlesyndication.com",
    "https://www.google.com",
    "https://www.gstatic.com",
    "https://www.googleadservices.com",
    "https://adservice.google.com",
    "https://www.googletagservices.com",
    "https://partner.googleadservices.com",
    "https://ep1.adtrafficquality.google",
    "https://ep2.adtrafficquality.google",
    "https://fundingchoicesmessages.google.com",
];

pub fn client_id() -> Option<String> {
    std::env::var(CLIENT_ENV)
        .ok()
        .as_deref()
        .and_then(sanitize_client)
}

fn sanitize_client(raw: &str) -> Option<String> {
    let s = raw.trim();
    let digits = s.strip_prefix("ca-pub-")?;
    if digits.len() >= 10 && digits.bytes().all(|b| b.is_ascii_digit()) {
        Some(s.to_string())
    } else {
        None
    }
}

fn sanitize_slot(raw: &str) -> Option<String> {
    let s = raw.trim();
    if !s.is_empty() && s.len() <= 22 && s.bytes().all(|b| b.is_ascii_digit()) {
        Some(s.to_string())
    } else {
        None
    }
}

fn env_slot(name: &str) -> Option<String> {
    std::env::var(name).ok().as_deref().and_then(sanitize_slot)
}

fn slot_id(placement: &str, size: &str) -> Option<String> {
    let specific = format!(
        "ADSENSE_SLOT_{}",
        placement.replace('-', "_").to_ascii_uppercase()
    );
    env_slot(&specific)
        .or_else(|| env_slot(&format!("ADSENSE_SLOT_{}", size.trim().to_ascii_uppercase())))
        .or_else(|| env_slot("ADSENSE_SLOT"))
}

pub fn head_snippet() -> String {
    // Tiny deferred loader — AdSense after engagement / long idle (monitor TBT).
    match client_id() {
        Some(id) => format!(
            r#"<link rel="dns-prefetch" href="https://pagead2.googlesyndication.com" />
<link rel="dns-prefetch" href="https://googleads.g.doubleclick.net" />
<meta name="google-adsense-account" content="{id}" />
<meta name="ukb-adsense-client" content="{id}" />
<script>
(function(){{
  function load(){{
    if(window.__ukbAdsJs)return;
    window.__ukbAdsJs=1;
    var s=document.createElement('script');
    s.type='module';
    s.src='/js/underkb-ads.js?v=4';
    s.fetchPriority='low';
    document.head.appendChild(s);
  }}
  function arm(){{
    var start=function(){{load();}};
    ['pointerdown','keydown','touchstart','scroll'].forEach(function(e){{
      window.addEventListener(e,start,{{once:true,passive:true}});
    }});
    if('requestIdleCallback' in window)requestIdleCallback(start,{{timeout:12000}});
    else window.addEventListener('load',function(){{setTimeout(start,8000);}},{{once:true}});
  }}
  if(document.readyState==='loading')document.addEventListener('DOMContentLoaded',arm,{{once:true}});
  else arm();
}})();
</script>"#
        ),
        None => String::new(),
    }
}

pub fn ads_txt() -> Option<String> {
    let client = client_id()?;
    let pub_id = client.strip_prefix("ca-")?;
    Some(format!("google.com, {pub_id}, DIRECT, f08c47fec0942fa0\n"))
}

pub fn apply_csp(csp: &mut CspConfig) {
    // Resuma 2.0 turns eval off and adds frame-src. Ads are injected without a
    // nonce, so strict-dynamic must stay off. The Fly header is the enforcing
    // policy; this one stays report-only so the two headers do not fight.
    let mut policy = csp.clone().allow_google_adsense();
    policy.strict_dynamic = false;
    policy.unsafe_eval = true;
    for origin in ADSENSE_ORIGINS {
        for list in [
            &mut policy.script_src,
            &mut policy.img_src,
            &mut policy.connect_src,
            &mut policy.style_src,
            &mut policy.frame_src,
        ] {
            if !list.iter().any(|s| s == origin) {
                list.push((*origin).into());
            }
        }
    }
    policy.report_only = true;
    *csp = policy;
}

thread_local! {
    static EMITTED_UNITS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Call at the start of the page function, before any [`slot`].
/// The page view is built before the layout, so clearing inside the layout
/// would forget the unit the page already emitted.
pub fn begin_page() {
    EMITTED_UNITS.with(|c| c.borrow_mut().clear());
}

fn claim_unit(unit: &str) -> bool {
    EMITTED_UNITS.with(|c| {
        let mut seen = c.borrow_mut();
        if seen.iter().any(|s| s == unit) {
            return false;
        }
        seen.push(unit.to_string());
        true
    })
}

pub fn slot(placement: &'static str, size: &'static str) -> View {
    let class = format!("ad-slot ad-slot-{size}");
    let live = client_id().zip(slot_id(placement, size));
    if let Some((_, unit)) = &live {
        if !claim_unit(unit) {
            return view! { <div hidden=""></div> };
        }
    }
    let lazy = matches!(size, "rectangle" | "article");
    match live {
        Some((client, unit)) if lazy => {
            let class = format!("{class} is-live");
            view! {
                <aside class={class} data-ad={placement} aria-label="Advertisement" data-ad-lazy="">
                    <div class="ad-slot-frame">
                        <ins
                            class="adsbygoogle"
                            style="display:block"
                            data-ad-client={client}
                            data-ad-slot={unit}
                            data-ad-format="auto"
                            data-full-width-responsive="true"
                        ></ins>
                    </div>
                </aside>
            }
        }
        Some((client, unit)) => {
            let class = format!("{class} is-live");
            view! {
                <aside class={class} data-ad={placement} aria-label="Advertisement">
                    <div class="ad-slot-frame">
                        <ins
                            class="adsbygoogle"
                            style="display:block"
                            data-ad-client={client}
                            data-ad-slot={unit}
                            data-ad-format="auto"
                            data-full-width-responsive="true"
                        ></ins>
                    </div>
                </aside>
            }
        }
        None => view! {
            <aside class={class} data-ad={placement} aria-label="Advertisement">
                <div class="ad-slot-frame">
                    <span class="ad-slot-label">"Ad"</span>
                </div>
            </aside>
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_ca_pub() {
        assert!(sanitize_client("ca-pub-1234567890123456").is_some());
        assert!(sanitize_client("pub-123").is_none());
    }

    #[test]
    fn one_unit_id_per_page() {
        begin_page();
        assert!(claim_unit("4857783384"));
        assert!(!claim_unit("4857783384"));
        assert!(claim_unit("999"));
        begin_page();
        assert!(claim_unit("4857783384"));
    }
}
