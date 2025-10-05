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

#[tauri::command]
fn get_hotkey_display() -> String {
    use crate::settings::get_cached;
    let s = get_cached();
    let mut parts = Vec::new();
    if s.hotkey.ctrl { parts.push("Ctrl"); }
    if s.hotkey.shift { parts.push("Shift"); }
    if s.hotkey.alt { parts.push("Alt"); }

    let key = match s.hotkey.code.as_str() {
        "Semicolon" => ";",
        "KeyJ" => "J",
        "KeyK" => "K",
        "KeyL" => "L",
        _ => "?",
    };

    parts.push(key);
    format!("{}", parts.join(" + "))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn main() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        //  掛上全域熱鍵插件
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
        //  前端可以呼叫的指令
        .invoke_handler(tauri::generate_handler![
            get_settings,
            set_settings,
            replace_with,
            get_hotkey_display
        ])
        //  初始化（建立 Tray、視窗、熱鍵等）
        .setup(|app| {
            #[cfg(desktop)]
            {
                use tauri::menu::{MenuBuilder, MenuItem};
                use tauri::tray::TrayIconBuilder;
                use tauri::Manager;

                //  建立「設定」菜單項目
                let open = MenuItem::with_id(app, "open_settings", "設定", true, None::<&str>)?;
                let quit = MenuItem::with_id(app, "quit_app", "關閉程式", true, None::<&str>)?;
                let menu = MenuBuilder::new(app).item(&open).item(&quit).build()?;
                use std::fs;
                use tauri::image::Image;
                let icon_bytes = fs::read("icons/icon.ico").unwrap_or_default();
                let icon = Image::new_owned(icon_bytes, 0, 0); // 讓 Tauri 自行解析大小

                //  建立系統匣圖示
                TrayIconBuilder::new()
                    .icon(icon)
                    .menu(&menu)
                    .on_menu_event(|app, ev| match ev.id().as_ref() {
                                        "open_settings" => {
                                            if let Some(win) = app.get_webview_window("main") {
                                                let _ = win.show();
                                                let _ = win.set_focus();
                                            }
                                        }
                                        "quit_app" => {
                                            std::process::exit(0);
                                        }
                                        _ => {}
                                    })
                                    .build(app)?;
                // --- 攔截關閉事件，只隱藏 ---
                if let Some(window) = app.get_webview_window("main") {
                    let app_handle = app.handle().clone(); // ✅ 這裡 clone()
                    window.on_window_event(move |event| {
                        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                            api.prevent_close();
                            if let Some(win) = app_handle.get_webview_window("main") {
                                let _ = win.hide();
                                let _ = win.set_skip_taskbar(true);
                            }
                            if let Some(tray) = app_handle.tray_by_id("main") {
                                let _ = tray.set_visible(true);
                            }
                        }
                    });
                }
                use tauri_plugin_autostart::ManagerExt;
                let autostart = app.autolaunch();
                if !autostart.is_enabled().unwrap_or(false) {
                    let _ = autostart.enable();
                    println!("✅ 自動啟動已啟用");
                }
            }

            //  啟動時載入設定並註冊熱鍵
            let s = settings::get_cached();
            hotkey::register_from_settings(&app.handle(), &s)?;

            Ok(())
        });

    builder
        .run(tauri::generate_context!())
        .expect(" error while running Fix-Zhuyin app");
}

//  熱鍵觸發後的處理邏輯
async fn handle_hotkey(_app: tauri::AppHandle) -> Result<()> {
    // 使用者必須手動切換到中文注音
    let selected = os::get_selected_text()?;
    if selected.trim().is_empty() {
        return Ok(());
    }
    os::retype_key_sequence(&selected)?;
    Ok(())
}