import { invoke } from "@tauri-apps/api/core";

type Settings = {
  hotkey: { ctrl: boolean; shift: boolean; alt: boolean; code: string };
};

const codeSelect = document.getElementById("hk-code") as HTMLSelectElement;
const saveBtn = document.getElementById("save-hk") as HTMLButtonElement;
const okMsg = document.getElementById("save-ok") as HTMLDivElement;
const hotkeyDisplay = document.getElementById("hotkey-display") as HTMLSpanElement;

async function loadHotkeyDisplay() {
  const display = await invoke<string>("get_hotkey_display");
  hotkeyDisplay.textContent = display;
}



async function loadSettings() {
  const s = (await invoke("get_settings")) as Settings;
  codeSelect.value = s.hotkey.code;
  await loadHotkeyDisplay();
}

async function saveSettings() {
  const s: Settings = {
    hotkey: { ctrl: true, shift: false, alt: false, code: codeSelect.value },
  };

  try {
    await invoke("set_settings", { newSettings: s });
    await loadHotkeyDisplay(); //  更新提示
    okMsg.style.display = "block";   // 顯示成功訊息
    okMsg.textContent = "✅ 已成功儲存設定！"+ hotkeyDisplay.textContent;
    setTimeout(() => (okMsg.style.display = "none"), 2000);
  } catch (err) {
    okMsg.style.display = "block";
    okMsg.style.color = "#ff7b7b";
    okMsg.textContent = "⚠️ 熱鍵註冊失敗，可能被其他程式佔用";
    setTimeout(() => (okMsg.style.display = "none"), 2500);
  }
}

saveBtn.addEventListener("click", saveSettings);
loadSettings();