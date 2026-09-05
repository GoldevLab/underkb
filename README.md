# UnderKb

Compress a JPG, PNG, or WebP **under a kilobyte budget** (default 200 KB). Also: JPG→WebP, resize, flat-background cutout, color palette. No account. Free uploads 20 MB; a Pro key allows 50 MB and a ZIP of up to 20 compress jobs.

```bash
cd underkb
RESUMA_CSP=0 RESUMA_BODY_LIMIT=54525952 cargo run
# http://127.0.0.1:3000
```

Home (`/`) is the compressor (default 200 KB). Other jobs have their own indexable pages with the same form. English aliases 308 to the Spanish canonicals (not in the sitemap). Spanish-slug pages add a one-line `lang=es` lead under the English H1.

| Path | Tool |
|---|---|
| `/` | Compress to KB |
| `/comprimir-imagen-kb` | Compress to KB (SEO) |
| `/convertir-jpg-a-webp` | Convert to WebP/JPG/PNG |
| `/redimensionar-imagen` | Resize |
| `/quitar-fondo` | Remove a flat background |
| `/extraer-colores-imagen` | HEX palette |
| `/privacy` | Privacy / ads |
| `/terms` | Terms |
| `/pricing` | Pro key / 50 MB / batch ZIP |

Custom domain later: set `SITE_URL`.

APIs (multipart `file`): `POST /api/compress` (`target_kb`, `format`), `POST /api/compress-batch` (repeat `file`, Pro), `/api/convert` (`format`, `quality`), `/api/resize` (`width`, `height`, `mode`, `format`), `/api/remove-bg` (`tolerance`, `format`), `/api/colors` (`count`).

On a phone, Share appears after a successful file when the browser can share that file.

Live: https://underkb.fly.dev — Fly org Gravitad, region `dfw`. Push to `main` deploys via GitHub Actions.

## Google AdSense

Same pattern as YouTubeForge. Keep units to `home-faq` and `landing-mid`. No ads on 404. Do not commit publisher IDs.

```bash
export ADSENSE_CLIENT=ca-pub-xxxxxxxxxxxxxxxx
export ADSENSE_SLOT=1234567890
# Fly
fly secrets set ADSENSE_CLIENT=ca-pub-xxxxxxxxxxxxxxxx ADSENSE_SLOT=1234567890
```

After deploy, `/ads.txt` is served when `ADSENSE_CLIENT` is set. CSP stays report-only so AdSense iframes are not blocked.

## Optional env (do not invent values)

| Env | Effect |
|---|---|
| `SITE_URL` | Canonical origin |
| `ADSENSE_CLIENT` / `ADSENSE_SLOT*` | Live ads + `/ads.txt` |
| `UNDERKB_PRO_KEYS` or `API_KEY` | 50 MB + batch ZIP (comma-separated, each ≥16 chars) |
| `CONTACT_EMAIL` | Shown on `/privacy` and `/pricing` |
| `GSC_VERIFICATION` | Search Console meta |
| `GA4_ID` (`G-…`) or `PLAUSIBLE_DOMAIN` | Analytics |

Search Console sitemap submit and AdSense approval stay in your Google accounts — the app only exposes `/sitemap.xml` and `/ads.txt`. Open `/?key=YOUR_KEY` once to store a Pro key in this browser.
