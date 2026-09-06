import { SITE } from "./config.js";

chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.removeAll(() => {
    chrome.contextMenus.create({
      id: "underkb-compress",
      title: "Compress with UnderKb",
      contexts: ["image"],
    });
  });
});

chrome.contextMenus.onClicked.addListener((info) => {
  if (info.menuItemId !== "underkb-compress") return;
  chrome.tabs.create({ url: `${SITE}/comprimir-imagen-kb` });
});
