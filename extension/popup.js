import { SITE } from "./config.js";

document.querySelectorAll("[data-path]").forEach((btn) => {
  btn.addEventListener("click", () => {
    chrome.tabs.create({ url: SITE + btn.dataset.path });
  });
});
