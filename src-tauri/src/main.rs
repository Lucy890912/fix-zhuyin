#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use anyhow::Result;

mod os;
mod conv;
mod rime;
mod settings;
mod hotkey;

#[tauri::command]
fn get_settings() -> settings::Settings {
    settings::get_cached()
}

#[tauri::command]
fn set_settings(app: tauri::AppHandle, new_settings: settings::Settings) -> Result<(), String> {
    settings::save(&new_settings).map_err(|e| e.to_string())?;
    settings::set_cached(new_settings.clone());
    hotkey::register_from_settings(&app, &new_settings).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn replace_with(text: String) -> Result<(), String> {
    os::replace_selection_with(&text).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn main() {
    let builder = tauri::Builder::default()
        // 🧩 掛上全域熱鍵插件
        .plugin({
            use tauri_plugin_global_shortcut::ShortcutState;
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, ev| {
                    if ev.state() == ShortcutState::Pressed {
                        let ah = app.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Err(e) = handle_hotkey(ah).await {
                                eprintln!("[fix-zhuyin hotkey error] {e:?}");
                            }
                        });
                    }
                })
                .build()
        })
        // 🧩 其他插件（例如打開外部檔案）
        .plugin(tauri_plugin_opener::init())
        // 🧩 前端可以呼叫的指令
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            replace_with
        ])
        // 🧩 初始化（建立 Tray、視窗、熱鍵等）
        .setup(|app| {
            #[cfg(desktop)]
            {
                use tauri::menu::{MenuBuilder, MenuItem};
                use tauri::tray::TrayIconBuilder;
                use tauri::{Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};

                // 🔹 建立「設定」菜單項目
                let open = MenuItem::with_id(app, "open_settings", "設定", true, None::<&str>)?;
                let menu = MenuBuilder::new(app).item(&open).build()?;

                // 🔹 建立系統匣圖示
                TrayIconBuilder::new()
                    .menu(&menu)
                    .on_menu_event(|app, event| {
                        if event.id().as_ref() == "open_settings" {
                            // 已存在 → 顯示
                            if let Some(win) = app.get_webview_window("settings") {
                                let _ = win.show();
                                let _ = win.set_focus();
                                return;
                            }

                            // 否則建立新設定視窗
                            let win = WebviewWindowBuilder::new(
                                app,
                                "settings",
                                WebviewUrl::App("settings.html".into()),
                            )
                            .title("Fix Zhuyin 設定")
                            .inner_size(360.0, 420.0)
                            .resizable(false)
                            .closable(true)
                            .visible(true)
                            .build()
                            .expect("Failed to create settings window");

                            //  當按下 X 時不要退出，只隱藏
                            let win2 = win.clone();
                            win.on_window_event(move |event| {
                                if let tauri::WindowEvent::CloseRequested { api, .. } = event {              
                                  api.prevent_close();
                                  let _ = win2.hide();
                                }
                            });
                        }
                    })
                    .build(app)?;
            }

            // 🔹 啟動時載入設定並註冊熱鍵
            let s = settings::get_cached();
            hotkey::register_from_settings(&app.handle(), &s)?;

            Ok(())
        });

    builder
        .run(tauri::generate_context!())
        .expect(" error while running Fix-Zhuyin app");
}

// 🧩 熱鍵觸發後的處理邏輯
async fn handle_hotkey(_app: tauri::AppHandle) -> Result<()> {
    // 使用者必須手動切換到中文注音
    let selected = os::get_selected_text()?;
    if selected.trim().is_empty() {
        return Ok(());
    }
    os::retype_key_sequence(&selected)?;
    Ok(())
}