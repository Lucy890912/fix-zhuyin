import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/tauri";

type Payload = {
  origin: string;
  items: string[];
  position: { x: number; y: number } | null;
};

let popup = document.getElementById("candidate-popup")!;
let candTitle = document.getElementById("cand-title")!;
let candList = document.getElementById("cand-list")!;
let visible = false;
let selIndex = 0;
let items: string[] = [];

function render() {
  candList!.innerHTML = items
    .map((it, i) => {
      const active = i === selIndex ? "background:#2a85ff;color:#fff;" : "background:#222;";
      return `<div data-idx="${i}" style="padding:6px 8px;border-radius:8px;margin:2px 0;cursor:pointer;${active}">
        <b style="opacity:.8;margin-right:6px">${i + 1}.</b> ${it}
      </div>`;
    })
    .join("");
  popup.style.display = "block";
  visible = true;
}

function hide() {
  popup.style.display = "none";
  visible = false;
}

listen<Payload>("show-candidates", (ev) => {
  items = ev.payload.items?.slice(0, 5) || [];
  selIndex = 0;
  candTitle!.textContent = `修復：${ev.payload.origin}`;
  // TODO: 利用 ev.payload.position 定位到游標附近（暫時固定位置）
  render();
});

// 鍵盤操作
window.addEventListener("keydown", (e) => {
  if (!visible) return;

  if (e.key === "Escape") {
    hide();
  } else if (e.key === "ArrowDown") {
    selIndex = (selIndex + 1) % items.length;
    render();
  } else if (e.key === "ArrowUp") {
    selIndex = (selIndex - 1 + items.length) % items.length;
    render();
  } else if (/^[1-5]$/.test(e.key)) {
    const idx = parseInt(e.key, 10) - 1;
    if (idx < items.length) {
      selIndex = idx;
      // 這裡可以發 RPC 要求 Rust 上屏該候選
      chooseCurrent();
    }
  } else if (e.key === "Enter") {
    chooseCurrent();
  }
});

async function chooseCurrent() {
  const chosen = items[selIndex];
  hide();
  await invoke("replace_with", { text: chosen });
}

// ===== IME 提醒：啟動顯示一次、可關閉 =====
const imeHintEl = document.getElementById("ime-hint") as HTMLDivElement | null;
const imeHintCloseBtn = document.getElementById("ime-hint-close") as HTMLButtonElement | null;

function showImeHint() {
  if (!imeHintEl) return;
  imeHintEl.style.display = "block";
}
function hideImeHint(permanent = true) {
  if (!imeHintEl) return;
  imeHintEl.style.display = "none";
  if (permanent) localStorage.setItem("hideImeHint", "1");
}

// 啟動時：若未被關閉過 → 顯示
if (localStorage.getItem("hideImeHint") !== "1") {
  showImeHint();
}
imeHintCloseBtn?.addEventListener("click", () => hideImeHint(true));

// （可選）當後端在按下熱鍵時想再提醒 2 秒：監聽事件
listen("show-ime-hint", () => {
  if (!imeHintEl) return;
  // 即使曾經「永久關閉」，這個提示也只閃一次（不寫入 localStorage）
  showImeHint();
  setTimeout(() => hideImeHint(false), 2000);
});