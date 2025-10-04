use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::Mutex};
use once_cell::sync::Lazy;
use tauri_plugin_global_shortcut::Code;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotkey {
  pub ctrl: bool,
  pub shift: bool,
  pub alt: bool,
  pub code: String, // 例如: "Semicolon", "J", "Backquote", "Backslash"
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
  pub hotkey: Hotkey,
}
impl Default for Settings {
  fn default() -> Self {
    Self { hotkey: Hotkey { ctrl: true, shift: false, alt: false, code: "Semicolon".into() } }
  }
}
fn parse_code(code: &str) -> Option<Code> {
    Some(match code {
        "Semicolon" => Code::Semicolon,
        "KeyJ" => Code::KeyJ,
        "KeyK" => Code::KeyK,
        _ => return None,
    })
}
static SETTINGS_PATH: Lazy<PathBuf> = Lazy::new(|| {
  // %APPDATA%\fix-zhuyin\settings.json (Windows)；其他平台對應使用者設定夾
  let mut dir = dirs::config_dir().unwrap_or(std::env::current_dir().unwrap());
  dir.push("fix-zhuyin");
  fs::create_dir_all(&dir).ok();
  dir.push("settings.json");
  dir
});

static CACHED: Lazy<Mutex<Settings>> = Lazy::new(|| Mutex::new(load().unwrap_or_default()));

pub fn load() -> Option<Settings> {
  let p = SETTINGS_PATH.clone();
  let txt = fs::read_to_string(p).ok()?;
  serde_json::from_str(&txt).ok()
}
pub fn save(s: &Settings) -> anyhow::Result<()> {
  let p = SETTINGS_PATH.clone();
  let txt = serde_json::to_string_pretty(s)?;
  fs::write(p, txt)?;
  Ok(())
}
pub fn get_cached() -> Settings { CACHED.lock().unwrap().clone() }
pub fn set_cached(s: Settings) { *CACHED.lock().unwrap() = s; }