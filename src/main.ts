
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

type Settings = {
  hotkey: { ctrl: boolean; shift: boolean; alt: boolean; code: string };
};

const codeSelect = document.getElementById("hk-code") as HTMLSelectElement;
const saveBtn = document.getElementById("save-hk") as HTMLButtonElement;
const okMsg = document.getElementById("save-ok") as HTMLDivElement;

async function loadSettings() {
  const s = (await invoke("get_settings")) as Settings;
  // 根據 code 選中對應選項
  codeSelect.value = s.hotkey.code;
}

async function saveSettings() {
  const s: Settings = {
    hotkey: { ctrl: true, shift: false, alt: false, code: codeSelect.value },
  };

  try {
    await invoke("set_settings", { newSettings: s });
    okMsg.style.display = "block";          // ✅ 顯示成功訊息
    okMsg.textContent = "✅ 已成功儲存設定！";
    setTimeout(() => (okMsg.style.display = "none"), 2000);
  } catch (err) {
    okMsg.style.display = "block";
    okMsg.style.color = "#ff7b7b";
    okMsg.textContent = "❌ 儲存失敗，請重試";
    setTimeout(() => (okMsg.style.display = "none"), 2500);
  }
}

saveBtn.addEventListener("click", saveSettings);
loadSettings();