# UnderKb

Compress a JPG, PNG, or WebP **under a kilobyte budget** (default 200 KB). Also: JPG→WebP, resize, flat-background cutout, color palette. No account. Uploads up to 20 MB.

```bash
cd underkb
RESUMA_CSP=0 RESUMA_BODY_LIMIT=26214400 cargo run
# http://127.0.0.1:3000
```

| Path | Tool |
|---|---|
| `/` | Compress to KB (English) |
| `/comprimir-imagen-kb` | Comprimir a KB |
| `/convertir-jpg-a-webp` | Convert to WebP/JPG/PNG |
| `/redimensionar-imagen` | Resize |
| `/quitar-fondo` | Remove a flat background |
| `/extraer-colores-imagen` | HEX palette |

APIs (multipart `file`): `POST /api/compress` (`target_kb`, `format`), `/api/convert` (`format`, `quality`), `/api/resize` (`width`, `height`, `mode`, `format`), `/api/remove-bg` (`tolerance`, `format`), `/api/colors` (`count`).

Live: https://underkb.fly.dev — Fly org Gravitad, region `dfw`. Push to `main` deploys via GitHub Actions.
