use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, sync::Mutex};
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotkey {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub code: String, // 例如: "Semicolon", "KeyJ", "KeyK", "KeyL"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub hotkey: Hotkey,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey: Hotkey {
                ctrl: true,
                shift: false,
                alt: false,
                code: "Semicolon".into(),
            },
        }
    }
}

//  設定檔位置
static SETTINGS_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let mut dir = dirs::config_dir().unwrap_or(std::env::current_dir().unwrap());
    dir.push("fix-zhuyin");
    fs::create_dir_all(&dir).ok();
    dir.push("settings.json");
    dir
});

//  快取設定，避免重複讀檔
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

pub fn get_cached() -> Settings {
    CACHED.lock().unwrap().clone()
}

pub fn set_cached(s: Settings) {
    *CACHED.lock().unwrap() = s;
}