import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";

type Settings = { hotkey: { ctrl: boolean; shift: boolean; alt: boolean; code: string } };

const el = {
  panel: document.getElementById("settings") as HTMLDivElement,
  ctrl: document.getElementById("hk-ctrl") as HTMLInputElement,
  shift: document.getElementById("hk-shift") as HTMLInputElement,
  alt: document.getElementById("hk-alt") as HTMLInputElement,
  code: document.getElementById("hk-code") as HTMLSelectElement,
  save: document.getElementById("save-hk") as HTMLButtonElement,
  close: document.getElementById("close-settings") as HTMLButtonElement,
  ok: document.getElementById("save-ok") as HTMLDivElement,
};

function openPanel() { el.panel.style.display = "block"; el.ok.style.display = "none"; }
function closePanel() { el.panel.style.display = "none"; }

async function loadSettings() {
  const s = (await invoke("get_settings")) as Settings;
  el.ctrl.checked = s.hotkey.ctrl;
  el.shift.checked = s.hotkey.shift;
  el.alt.checked = s.hotkey.alt;
  el.code.value = s.hotkey.code;
}
async function saveSettings() {
  const s: Settings = {
    hotkey: { ctrl: el.ctrl.checked, shift: el.shift.checked, alt: el.alt.checked, code: el.code.value }
  };
  await invoke("set_settings", { newSettings: s });
  el.ok.style.display = "block";
}

el.save?.addEventListener("click", saveSettings);
el.close?.addEventListener("click", closePanel);

// 系統匣「設定」→ 打開面板
listen("open-settings", () => { openPanel(); loadSettings(); });

// 你也可以加個鍵盤快捷鍵打開面板（例如 Ctrl+,）
window.addEventListener("keydown", (e) => {
  if (e.ctrlKey && e.key === ",") { openPanel(); loadSettings(); }
});

// 啟動時讀一次（可選）
loadSettings();