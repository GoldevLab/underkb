# Chrome extension (UnderKb)

Load unpacked: chrome://extensions → Developer mode → Load unpacked → this folder.

Publish: zip this folder. Listing copy: STORE.txt. Privacy: https://underkb.fly.dev/privacy

After you buy a domain:
1. Fly `SITE_URL=https://YOUR.DOMAIN`
2. Edit `config.js` and `manifest.json` (`homepage_url`, `host_permissions`)
3. Fly `CHROME_STORE_URL=…` when the listing is live
4. AdSense stays on the website (`ADSENSE_CLIENT` / `ADSENSE_SLOT*`) — never in this extension
