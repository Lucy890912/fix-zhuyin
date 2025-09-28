#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use tauri::Emitter;
use anyhow::Result;

mod os;
mod conv;
mod rime; // 目前可留空

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn main() {
  tauri::Builder::default()
    .setup(|app| {
      #[cfg(desktop)]
      {
        use tauri::{AppHandle};
        use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

        let app_handle: AppHandle = app.handle().clone();

        app.handle().plugin(
          tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |_app, _shortcut, event| {
              if event.state() == ShortcutState::Pressed {
                let ah = app_handle.clone();
                tauri::async_runtime::spawn(async move {
                  if let Err(e) = handle_hotkey(ah).await {
                    eprintln!("[fix-zhuyin] convert error: {e:?}");
                  }
                });
              }
            })
            .build()
        )?;

        // 註冊 Ctrl + ;
        let hotkey = Shortcut::new(Some(Modifiers::CONTROL), Code::Semicolon);
        app.global_shortcut().register(hotkey)?;
      }

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![replace_with])
    .plugin(tauri_plugin_opener::init())
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[tauri::command]
fn replace_with(text: String) -> Result<(), String> {
  os::replace_selection_with(&text).map_err(|e| e.to_string())
}

async fn handle_hotkey(_app: tauri::AppHandle) -> anyhow::Result<()> {
  // 1) 讀取被選取的英數鍵序
  let selected = os::get_selected_text()?;
  if selected.trim().is_empty() {
    return Ok(());
  }

  // 2) 不自動切輸入法 — 讓使用者自行切到「中文注音」
  // os::toggle_ime_once();  // ← 移除這行

  // 3) 直接「逐鍵重打」：不送 Enter/空白，喚出使用者當前 IME 的候選窗
  os::retype_key_sequence(&selected)?;
  Ok(())
}