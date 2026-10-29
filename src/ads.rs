//! Google AdSense. Layout is reserved; live units when ADSENSE_CLIENT + slot are set.

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
    csp.strict_dynamic = false;
    for origin in ADSENSE_ORIGINS {
        if !csp.script_src.iter().any(|s| s == origin) {
            csp.script_src.push((*origin).into());
        }
        if !csp.img_src.iter().any(|s| s == origin) {
            csp.img_src.push((*origin).into());
        }
        if !csp.connect_src.iter().any(|s| s == origin) {
            csp.connect_src.push((*origin).into());
        }
        if !csp.style_src.iter().any(|s| s == origin) {
            csp.style_src.push((*origin).into());
        }
    }
    csp.report_only = true;
}

pub fn slot(placement: &'static str, size: &'static str) -> View {
    let class = format!("ad-slot ad-slot-{size}");
    let live = client_id().zip(slot_id(placement, size));
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
}
