use anyhow::{anyhow, Result};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::settings::Settings;

static CURRENT: Lazy<Mutex<Option<Shortcut>>> = Lazy::new(|| Mutex::new(None));

fn parse_code(code: &str) -> Option<Code> {
  use Code::*;
  Some(match code {
    "Semicolon" => Semicolon,
    "Backquote" => Backquote,
    "Backslash" => Backslash,
    "J" => KeyJ,
    "K" => KeyK,
    "Space" => Space,
    // 可自行擴充：A~Z、數字鍵等
    _ => return None,
  })
}

pub fn register_from_settings(app: &tauri::AppHandle, s: &Settings) -> Result<()> {
  let mut mods = Modifiers::empty();
  if s.hotkey.ctrl { mods |= Modifiers::CONTROL; }
  if s.hotkey.shift { mods |= Modifiers::SHIFT; }
  if s.hotkey.alt { mods |= Modifiers::ALT; }
  let code = parse_code(&s.hotkey.code).ok_or_else(|| anyhow!("unsupported code"))?;
  let shortcut = Shortcut::new(Some(mods), code);

  // 先解除舊的
  if let Some(old) = CURRENT.lock().unwrap().take() {
    let _ = app.global_shortcut().unregister(old);
  }
  app.global_shortcut().register(shortcut)?;
  *CURRENT.lock().unwrap() = Some(shortcut);
  Ok(())
}