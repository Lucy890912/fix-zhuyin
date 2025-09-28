use anyhow::Result;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct LayoutFile {
    name: String,
    #[allow(dead_code)]
    notes: Option<String>,
    map: HashMap<String, String>,
}

// 直接把 JSON 內嵌，路徑以 src-tauri 為基準
static LAYOUT: Lazy<LayoutFile> = Lazy::new(|| {
    // 確保這個路徑存在：src-tauri/resources/layouts/standard_bopomofo.json
    const JSON: &str = include_str!("../resources/layouts/standard_bopomofo.json");
    serde_json::from_str(JSON).unwrap_or_else(|e| {
        eprintln!("[conv] parse embedded layout failed: {e}");
        LayoutFile { name: "empty".into(), notes: None, map: HashMap::new() }
    })
});

/// 將原始英文鍵序轉成注音符號序列（依鍵位映射）
fn to_bopomofo_symbols(raw: &str) -> Vec<String> {
    let mut out = Vec::new();
    for ch in raw.chars() {
        let key = ch.to_ascii_lowercase().to_string();
        if let Some(sym) = LAYOUT.map.get(&key) {
            out.push(sym.clone());
        } else {
            // 不在映射表的字元先忽略（或改成保留）
        }
    }
    out
}

/// 簡易音節切分：遇到聲調就結束一個音節；句尾若沒聲調，最後一段也視為一個音節
fn segment_syllables(symbols: &[String]) -> Vec<String> {
    const TONES: [&str; 5] = ["˙","ˊ","ˇ","ˋ","ˮ"];
    let mut res = Vec::new();
    let mut buf: Vec<&str> = Vec::new();
    for s in symbols {
        buf.push(s.as_str());
        if TONES.contains(&s.as_str()) {
            res.push(buf.join(""));
            buf.clear();
        }
    }
    if !buf.is_empty() {
        res.push(buf.join(""));
    }
    res
}

/// 將 raw 英文鍵序 → 注音音節；下一步會接 RIME
pub fn convert_raw_english_to_candidates(raw: &str) -> Result<Vec<String>> {
    let symbols = to_bopomofo_symbols(raw);
    if symbols.is_empty() {
        return Ok(vec![]);
    }
    let syllables = segment_syllables(&symbols);
    // 目前先把注音當預覽輸出，驗證鍵位是否正確
    Ok(vec![format!("〔{}〕", syllables.join(" "))])
}