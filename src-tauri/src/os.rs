use anyhow::{anyhow, Result};
use arboard::Clipboard;
use enigo::{Enigo, KeyboardControllable, Key};
use std::{thread, time::Duration};

/// 模擬 Ctrl+C，讀取目前選取文字；同時備份/還原剪貼簿
pub fn get_selected_text() -> Result<String> {
    // 1) 備份剪貼簿
    let mut cb = Clipboard::new().map_err(|e| anyhow!("clipboard open: {e}"))?;
    let backup = cb.get_text().ok();

    // 2) 模擬 Ctrl+C
    let mut enigo = Enigo::new();
    enigo.key_down(Key::Control);
    enigo.key_click(Key::Layout('c'));
    enigo.key_up(Key::Control);

    // 3) 等待系統把內容放入剪貼簿
    thread::sleep(Duration::from_millis(80));

    // 4) 讀取選取的文字
    let copied = Clipboard::new()
        .and_then(|mut c| c.get_text())
        .map_err(|e| anyhow!("clipboard read: {e}"))?;

    // ⚠ 關鍵：如果與備份相同，視為「沒有選到任何新內容」→ 回傳空字串避免後續動作
    if let Some(b) = &backup {
        if b == &copied {
            return Ok(String::new());
        }
    }

    // 5) 還原剪貼簿
    if let Some(b) = backup {
        let _ = Clipboard::new().and_then(|mut c| c.set_text(b));
    }
    Ok(copied)
}

/// 逐鍵「重打」一段鍵序，不送 Enter/空白（不提交），讓 IME 彈出候選
/// 注意：通常直接輸入就會覆蓋目前的「選取文字」，不需先退格
pub fn retype_key_sequence(raw_ascii: &str) -> Result<()> {
    let mut enigo = Enigo::new();

    // 視需求可先清掉選取：大多數應用程式在有選取時，直接輸入就會覆寫，因此可省略
    // enigo.key_click(Key::Backspace);

    // 逐字緩打，讓 IME 有時間處理組字
    for ch in raw_ascii.chars() {
        // 僅允許 ASCII 可見字元與常見標點；其他略過
        if ch.is_ascii() {
            enigo.key_click(Key::Layout(ch));
            thread::sleep(Duration::from_millis(16)); // 10~20ms 體感最穩
        }
    }
    Ok(())
}

pub fn replace_selection_with(s: &str) -> Result<()> {
    // 備份剪貼簿
    let mut cb = Clipboard::new().map_err(|e| anyhow!("clipboard open: {e}"))?;
    let backup = cb.get_text().ok();

    // 放入要替換的文字
    cb.set_text(s.to_string())
        .map_err(|e| anyhow!("set clipboard: {e}"))?;

    // Ctrl+V 貼上
    let mut enigo = Enigo::new();
    enigo.key_down(Key::Control);
    enigo.key_click(Key::Layout('v'));
    enigo.key_up(Key::Control);

    // 給目標應用程式時間取出剪貼簿內容
    thread::sleep(Duration::from_millis(180));

    // 還原剪貼簿
    if let Some(b) = backup {
        let _ = Clipboard::new().and_then(|mut c| c.set_text(b));
    }
    Ok(())
}