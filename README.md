# fix-zhuyin

一個專為台灣使用者設計的跨平台桌面應用程式。  
解決常見問題：在輸入中文注音時忘記切換回中文輸入法，結果打出一串英文。  
此程式能將英文鍵序重新轉換為注音，並**模擬重新輸入**，讓使用者的注音輸入法正常彈出候選字選單，自行選字。

---

## 功能特色

- **鍵序修復**：將誤打的英文字母轉回注音鍵序。
- **重新輸入**：模擬鍵盤輸入，把注音送回目前的中文輸入法，讓使用者在熟悉的候選窗選字。
- **全域熱鍵**：預設為 `Ctrl + ;`，可在任何應用程式中使用。
- **支援多種應用**：Word、Google 文件、記事本、瀏覽器輸入框等。
- **安全**：不修改使用者詞庫、不依賴雲端。

---

## 🛠 開發環境

開發環境準備（Windows 10/11）
```bash
#安裝 Visual Studio 2022 Build Tools（選擇 C++ toolset）
#PowerShell
winget install --id Microsoft.VisualStudio.2022.BuildTools -e
### 重開機

#安裝 Rust（rustup，Stable）
winget install Rustlang.Rustup
rustup default stable-x86_64-pc-windows-msvc
rustup update

#安裝 Node.js LTS（給前端與 Tauri CLI）
winget install OpenJS.NodeJS.LTS

#安裝 CMake（用來編 librime）
winget install Kitware.CMake

#Git（之後抓專案 會用）
winget install Git.Git


---

## 開始使用

### 1. Clone 專案
git clone https://github.com/Lucy890912/fix-zhuyin.git
cd fix-zhuyin
### 2. 安裝依賴
npm install

### 3. 開發模式執行
npm run tauri dev
### 4. 打包應用程式
npm run tauri build

🎹 使用方式

1.在任何地方誤打英文鍵序
2.切換到 中文注音輸入法
3.反白該字串，按下 Ctrl + ;。
4.程式會自動幫你逐鍵「重打」注音
5.使用者照平常習慣（空白/數字鍵/方向鍵）選字即可。