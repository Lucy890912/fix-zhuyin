#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use anyhow::Result;
use tauri::menu::{MenuBuilder, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use tauri::image::Image;
use image::{ImageReader, RgbaImage, Rgba};
use std::io::Cursor;
use image::GenericImageView;

mod os;
mod conv;
mod rime;
mod settings;
mod hotkey;


/// 若載入失敗，自動使用灰色圓形 fallback 圖示
fn load_tray_icon() -> Image<'static> {
    // 11. 內嵌你的圖示檔案 (.ico 或 .png 都可)
    let bytes = include_bytes!("../icons/icon.ico");

    // 2️. 嘗試解析圖片格式
    let decoded_result = match ImageReader::new(Cursor::new(bytes)).with_guessed_format() {
        Ok(reader) => reader.decode(), // 這裡 decode() 會回傳 Result<DynamicImage, ImageError>
        Err(e) => Err(image::ImageError::IoError(e)), // 統一成 ImageError 類型
    };

    match decoded_result {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (width, height) = img.dimensions();
            //println!("🖼️ 圖示載入成功 ({width}x{height})");
            Image::new_owned(rgba.into_raw(), width, height)
        }
        Err(e) => {
            println!("⚠️ 圖示載入失敗，使用 fallback icon: {e}");

            // 3️⃣ 建立灰色圓形 fallback 圖示 (32x32)
            let size = 32;
            let mut rgba = RgbaImage::new(size, size);

            for y in 0..size {
                for x in 0..size {
                    let dx = x as f32 - (size as f32 / 2.0);
                    let dy = y as f32 - (size as f32 / 2.0);
                    let dist = (dx * dx + dy * dy).sqrt();
                    let color = if dist <= size as f32 / 2.5 {
                        Rgba([150, 150, 150, 255]) // 灰色圓點
                    } else {
                        Rgba([0, 0, 0, 0]) // 透明背景
                    };
                    rgba.put_pixel(x, y, color);
                }
            }

            Image::new_owned(rgba.into_raw(), size, size)
        }
    }
}

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
                //  建立「設定」菜單項目
                let open = MenuItem::with_id(app, "open_settings", "設定", true, None::<&str>)?;
                let quit = MenuItem::with_id(app, "quit_app", "關閉程式", true, None::<&str>)?;
                let menu = MenuBuilder::new(app).item(&open).item(&quit).build()?;

                let icon = load_tray_icon();

                let tray = TrayIconBuilder::new()
                    .icon(icon)
                    .tooltip("Zhuyin Fixer")
                    .menu(&menu)
                    .on_menu_event(|app, ev| match ev.id().as_ref() {
                        "open_settings" => {
                            if let Some(win) = app.get_webview_window("main") {
                                // 若視窗已存在，就顯示出來並聚焦
                                let _ = win.show();
                                let _ = win.set_skip_taskbar(false);
                                let _ = win.set_focus();
                            } else {
                                use tauri::{WebviewUrl, WebviewWindowBuilder};

                                // 根據模式選擇載入來源
                                let url = if cfg!(debug_assertions) {
                                    //  開發模式：載入 Vite dev server
                                    WebviewUrl::External("http://localhost:1420".parse().unwrap())
                                } else {
                                    //  打包後：載入內嵌前端
                                    WebviewUrl::App("index.html".into())
                                };

                                println!(" 開啟設定視窗：{:?}", url);

                                let _ = WebviewWindowBuilder::new(app, "main", url)
                                    .title("Fix Zhuyin")
                                    .resizable(true)
                                    .fullscreen(false)
                                    .build();
                            }
                        }
                        "quit_app" => std::process::exit(0),
                        _ => {}
                    })
                    .build(app)?;

                //println!(" 系統匣建立成功: {:?}", tray.id());
                // --- 攔截關閉事件，只隱藏 ---
                if let Some(window) = app.get_webview_window("main") {
                    let app_handle = app.handle().clone(); //  這裡 clone()
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
                    println!(" 自動啟動已啟用");
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