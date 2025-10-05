use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, Modifiers, Code};
use crate::settings::Settings;

// 儲存當前註冊中的快捷鍵，以便後續取消或覆蓋
static CURRENT: Lazy<Mutex<Option<Shortcut>>> = Lazy::new(|| Mutex::new(None));

/// 將字串轉換成 tauri 的 Code
fn parse_code(code: &str) -> Option<Code> {
    use Code::*;
    Some(match code {
        "Semicolon" => Semicolon,
        "KeyJ" => KeyJ,
        "KeyK" => KeyK,
        "KeyL" => KeyL,
        _ => return None,
    })
}

/// 根據設定註冊熱鍵
pub fn register_from_settings(app: &tauri::AppHandle, s: &Settings) -> Result<()> {
    let mut mods = Modifiers::empty();
    if s.hotkey.ctrl {
        mods |= Modifiers::CONTROL;
    }
    if s.hotkey.shift {
        mods |= Modifiers::SHIFT;
    }
    if s.hotkey.alt {
        mods |= Modifiers::ALT;
    }

    let code = parse_code(&s.hotkey.code)
        .ok_or_else(|| anyhow!("unsupported code"))?;
    let shortcut = Shortcut::new(Some(mods), code);

    // 先解除舊熱鍵
    if let Some(old) = CURRENT.lock().unwrap().take() {
        let _ = app.global_shortcut().unregister(old);
    }

    // 註冊新熱鍵
    match app.global_shortcut().register(shortcut) {
        Ok(_) => {
            println!("✅ 快捷鍵註冊成功: {:?}+{:?}", mods, code);
            *CURRENT.lock().unwrap() = Some(shortcut);
            Ok(())
        }
        Err(e) => {
            println!("⚠️ 快捷鍵註冊失敗: {:?}+{:?} ({:?})", mods, code, e);
            Err(anyhow!("快捷鍵註冊失敗，可能被其他程式佔用"))
        }
    }
}