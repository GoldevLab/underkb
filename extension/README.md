Publish: zip this folder. Listing copy: STORE.txt. Privacy: https://under200kb.com/privacy

Domain is `under200kb.com`. Keep `config.js` (`SITE`) and `manifest.json` host permissions in sync with Fly `SITE_URL`.

After domain changes:
1. Fly `SITE_URL=https://under200kb.com`
2. Edit `config.js` and `manifest.json` (`homepage_url`, `host_permissions`)
3. Fly `CHROME_STORE_URL=…` when the listing is live
4. AdSense stays on the website (`ADSENSE_CLIENT` / `ADSENSE_SLOT*`) — never in this extension
