# UnderKb

Compress a JPG, PNG, or WebP **under a kilobyte budget** (default 200 KB). Quality first, then scale. No account. Uploads up to 20 MB.

```bash
cd underkb
RESUMA_CSP=0 RESUMA_BODY_LIMIT=26214400 cargo run
# http://127.0.0.1:3000
```

`POST /api/compress` multipart fields: `file`, `target_kb` (default 200), `format` (`jpeg` | `webp` | `png`).

Live: https://underkb.fly.dev — Fly org Gravitad, region `dfw`. Push to `main` deploys via GitHub Actions.
