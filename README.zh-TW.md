# HistCite Mac

**這是`yangnim21029`獨立開發的 macOS 專案，並非原版 HistCite 的官方發行版。** 原版 HistCite 由 Eugene Garfield 與合作團隊開發；本專案與原團隊沒有隸屬關係。[專案身分與原版貢獻說明](docs/project-identity.md#繁體中文)。

**在 Mac 上整理 Web of Science 文獻、分析引用關係，從檢索結果中尋找研究領域的發展線索。**

當研究者從 Web of Science（WoS）取得數百筆、甚至數千筆文獻後，除了逐篇閱讀，也可以透過引用關係了解這些研究如何相互連結：哪些文獻在資料集中經常被引用？不同時期的研究如何延續先前的成果？哪些作者、期刊或研究主題值得進一步關注？

HistCite Mac 是一款以 Rust 獨立開發分析核心的 macOS 書目計量工具，參照 HistCite 12.03.17 的主要工作流程，讓研究者能在 Mac 上整理文獻、比較引用指標，並使用 Graph Maker 繪製文獻之間的引用關係圖。

**從文獻匯入、資料整理到引用分析，主要操作均可在 Mac 本機完成，無須安裝 Windows、Wine 或額外的程式開發環境。**

[**下載 HistCite Mac**](https://github.com/yangnim21029/histcite-mac/releases/latest) · [60 秒操作示範](#60-秒看-histcite-mac-實際操作) · [English](README.md) · [简体中文安装指南](docs/mac-install.zh-CN.md) · [版本紀錄](https://github.com/yangnim21029/histcite-mac/releases)

macOS 13 Ventura 以上 · 支援 Apple Silicon 與 Intel Mac · 免費開源

[![Security checks](https://github.com/yangnim21029/histcite-mac/actions/workflows/security.yml/badge.svg?branch=main)](https://github.com/yangnim21029/histcite-mac/actions/workflows/security.yml) [![OpenSSF Scorecard](https://api.scorecard.dev/projects/github.com/yangnim21029/histcite-mac/badge)](https://scorecard.dev/viewer/?uri=github.com/yangnim21029/histcite-mac)

徽章顯示自動檢查結果與專案維護評分，並非資安認證。[檢查範圍與已知限制](docs/security-review.md)包含舊版 Graphviz 尚待完成的審查。

## 60 秒看 HistCite Mac 實際操作

在 macOS 原生程式中，示範匯入 WoS 純文字檔、依 LCS／GCS 排序、查看作者索引、引用關係圖與文獻詳情。影片使用 12 筆虛構文獻、31 條本地引用連線，附英文字幕。

影片用來展示介面操作，不能證明所有目前的 WoS 匯出皆相容。另以兩份原始研究匯出檔、未經外部清洗進行的驗證，請見[直接匯入檢查與限制](docs/compatibility.md#direct-import-check)。

https://github.com/user-attachments/assets/0c809917-e373-4628-b55f-fc354c1838c0

[下載 MP4](https://github.com/yangnim21029/histcite-mac/releases/download/v0.3.4/HistCite-Mac-demo-en.mp4) · [英文字幕（SRT）](https://github.com/yangnim21029/histcite-mac/releases/download/v0.3.4/HistCite-Mac-demo-en.srt)

HistCite Mac 由yangnim21029獨立實作；原版 HistCite 的成果歸於 Eugene Garfield 與合作團隊，本專案並非原作者的官方發行版。

## 從文獻資料到引用關係：三步開始

### 1. 下載並安裝

前往[最新版本頁面](https://github.com/yangnim21029/histcite-mac/releases/latest)，下載 **Universal DMG**。Apple Silicon 與 Intel Mac 都使用同一份安裝檔。

開啟 DMG 後，將 **HistCite Mac** 拖入 **Applications（應用程式）**資料夾，再從應用程式開啟。

> **第一次開啟：**目前的社群版本尚未經 Apple 公證。如果 macOS 阻擋開啟，確認檔案來自本專案的正式發布頁面後，可到 **系統設定 → 隱私權與安全性 → 強制打開（Open Anyway）**。這會為此 App 建立例外，並不表示 Apple 已驗證其安全性。[Apple 官方說明](https://support.apple.com/zh-tw/102445)。

### 2. 準備 Web of Science 文獻資料

在 Web of Science 完成檢索後，匯出文獻紀錄，並選擇：

- **檔案格式**：Plain Text（純文字）
- **紀錄內容**：Full Record and Cited References（完整紀錄與被引參考文獻）

HistCite Mac 可直接讀取匯出的 `.txt` 檔案。如果文獻分成多批匯出，也可以一次選取多個檔案。

**建議保留原始匯出檔案及檢索條件。** 當分析結果出現疑問時，才能回頭確認資料來源、匯出範圍與後續處理方式。

### 3. 匯入文獻並開始分析

開啟 HistCite Mac，選擇 **File → Open collection…**，載入準備好的文獻檔案。可以先從三項操作認識資料：

1. **查看文獻清單**：點選 **LCS** 欄位，依文獻在目前集合內的被引用次數排序。
2. **開啟文獻詳情**：在獨立視窗查看摘要、參考文獻及相關書目資訊。
3. **繪製引用關係圖**：選擇 **Tools → Graph Maker…**，觀察集合中的文獻如何透過引用建立關聯。

**還沒有準備自己的研究資料？** 安裝檔附有 `demo-wos.txt`，包含 12 筆虛構文獻。將範例檔複製到 Mac 後，即可匯入，熟悉文獻清單、引用指標與 Graph Maker 的操作。

![文獻列表與引用計量；畫面使用虛構示範資料](docs/images/records.png)

## 可以做什麼？

### 1. 從引用指標辨認值得深入閱讀的文獻

面對大量檢索結果時，研究者往往無法立即判斷應從哪些文獻開始閱讀。HistCite Mac 提供引用指標及文獻詳情，協助使用者從資料集中尋找可能具有參考價值的研究。

其中，兩項常用指標反映不同的引用範圍：

- **LCS（Local Citation Score）**：一篇文獻在目前匯入集合內，被其他文獻引用的次數。
- **GCS（Global Citation Score）**：WoS 匯出當時記錄的被引用次數，匯入後不會自動更新。

例如，一篇研究的 GCS 可能很高，但 LCS 不一定同樣突出。它在來源資料庫中可能廣受引用，在本次建立的文獻集合中卻較少被引用。比較兩項指標，有助於區分這些引用情境，作為安排後續閱讀的線索。

### 2. 分析文獻集合的組成與分布

除了閱讀個別文獻，也可以依不同欄位整理資料，觀察研究產出的分布情形。目前支援的分析項目包括：

- 作者與期刊
- 被引參考文獻
- 詞彙與發表年份
- 文獻類型與語言
- 機構、部門與國家

例如，年度分布可以協助觀察文獻產出的時間變化；作者與期刊分析則有助於辨認資料集中發表較多的研究者及出版來源。

這些分類提供不同的閱讀入口，讓使用者可以從集合的整體分布，進一步查看形成這些分布的個別文獻。

### 3. 使用 Graph Maker 追蹤文獻之間的引用關係

引用次數可以顯示一篇文獻被引用的情形。Graph Maker 則將引用關係轉化為圖形，讓研究者觀察它與哪些研究相連，以及這些連結在時間上的分布。

例如，哪些較早發表的研究被後續文獻引用？哪些研究之間存在可追查的引用路徑？使用者可以：

- 調整時間軸選項與節點大小。
- 設定箭頭方向及標籤。
- 合併引用連線的共同區段。
- 將滑鼠移至節點，查看文獻提示。
- 點擊節點，開啟對應的文獻詳情。

Graph Maker 讓原本分散於多筆參考文獻清單中的引用資訊，成為可以觀察、並回到個別文獻追查的關係。

![Graph Maker；畫面使用虛構示範資料](docs/images/graph.png)

### 4. 整理、修正與合併書目資料

書目計量分析也需要處理原始紀錄中的重複、缺漏或名稱不一致等問題。搜尋、標記、標籤及編輯紀錄等功能，讓研究者可以在集合內整理資料，並依研究目的調整分析範圍。集合的修改支援復原。

**匯入 TXT／HCI 與新增檔案時，會自動套用重複合併規則。** 預設規則參照原版 HistCite 的書目判定方式，可能將識別碼不同的紀錄合併。可在 **Tools → Data checks** 查看合併紀錄，並保留原始匯出檔，方便核對篇名、作者、出版資訊與識別碼。

既有 `.hcm` 集合會依儲存內容重新開啟；若要另行整併，可使用 **Tools → Merge duplicates**，這項操作支援復原。合併歷程會保留匯入紀錄的欄位，供後續檢查。

### 5. 儲存工作並匯出分析結果

完成分析後，可以依照後續工作需求，選擇不同的儲存與匯出格式。

| 格式 | 主要用途 |
| --- | --- |
| `.hcm` | 儲存 HistCite Mac 工作集合，供下次繼續編輯與分析 |
| `.hci` | 以 HistCite 集合格式交換文獻紀錄 |
| CSV | 匯出書目與計量資料，供試算表或其他工具使用 |
| 離線 HTML | 將分析內容整理為可在瀏覽器開啟的互動形式 |
| SVG、PNG | 將引用圖放入報告、簡報或其他文件 |
| PostScript、DOT、Pajek | 供其他繪圖或網絡分析工具使用 |

其中，`.hcm` 適合保存持續進行的研究工作，可保留文獻紀錄、標籤、設定與已儲存的引用圖。資料與圖形匯出則便於後續整理、分享及呈現結果。

### 如何解讀分析結果

所有分析都以目前匯入的文獻集合為基礎。檢索式、資料庫涵蓋範圍、文獻納入條件，以及參考文獻資訊的缺漏，都會影響計量結果與引用圖；引用次數也受到發表時間與文獻類型等因素影響。

**可以先用分析結果辨認問題與閱讀線索，再回到文獻內容及研究脈絡加以判斷。** 單憑引用連線，還無法說明一篇文獻的學術貢獻，或另一篇研究引用它的原因。

## 常見問題

### HistCite 有 Mac 版嗎？

**目前已有獨立開發的 HistCite Mac，可直接在 macOS 上執行。** 本專案參照 HistCite 12.03.17 的工作流程，提供本機引用分析與 Graph Maker，支援 macOS 13 以上；Apple Silicon 與 Intel Mac 使用同一份 Universal DMG，無須 Windows、Wine、CrossOver 或虛擬機。

舊教學中的平台限制可能指原版 Windows HistCite 或 HistCite Pro。本專案是獨立實作，並非原作者推出的官方 Mac 版本。請從[最新版本頁面](https://github.com/yangnim21029/histcite-mac/releases/latest)下載，依前面的安裝步驟操作；目前發行版尚未經 Apple 公證，首次開啟方式也列在安裝說明中。

### 支援哪些輸入格式？

目前可以直接開啟：

- **Web of Science 標籤式純文字檔**（`.txt`）
- **HistCite 集合檔**（`.hci`）
- **HistCite Mac 工作集合**（`.hcm`）

Scopus 原生匯出檔或一般 Excel 檔案，需要先轉換為相容的 WoS 標籤式格式。若相關文獻可以從 WoS 取得，也可以依照前述設定重新匯出。

直接將副檔名改成 `.txt`，不會轉換檔案內部的欄位與資料結構。

### 文獻資料會上傳到伺服器嗎？

文獻匯入、整理與分析功能都在 Mac 本機執行，HistCite Mac 不會將文獻集合上傳至遠端服務。

如果主動點選 DOI 或 WoS 連結，App 會開啟瀏覽器，前往相應網站。原始匯入檔會保留，編輯操作作用於工作集合。

### 下次開啟可以繼續分析嗎？

可以。選擇 **File → Save as…**，將工作集合儲存為 `.hcm`，即可保留文獻紀錄、標籤、設定及已儲存的引用圖。App 也提供上次工作集合的自動恢復功能。

若研究需要多次清理或調整資料，建議在適當階段儲存工作集合，同時保留最初匯出的資料，以便比較不同處理階段的結果。

### HistCite Mac 是原版 HistCite 嗎？

HistCite Mac 是使用 Rust 獨立開發的專案，參照 HistCite 12.03.17 的操作流程，不包含原版 HistCite 的程式碼或執行檔，也與原作者沒有隸屬關係。

開發過程已針對特定測試資料與操作情境，比較引用計分、分析群組、重複紀錄合併及圖形排版等結果。這些驗證確認的是已測試條件下的相容性，尚不能據此認定所有資料與功能都和原版完全一致。

測試項目、比較方式與已知限制，請參考[相容性與驗證說明](docs/compatibility.md)。

## 回報問題與參與開發

如果在文獻匯入、資料整理或圖形分析時遇到錯誤，歡迎透過 [GitHub Issues](https://github.com/yangnim21029/histcite-mac/issues) 回報。建議提供：

- macOS 版本與 Mac 晶片類型。
- 使用的檔案格式與 App 版本。
- 發生問題前的操作步驟。
- 錯誤訊息或畫面截圖。
- 可重現問題的少量範例資料。

如果錯誤只在特定集合中發生，可以嘗試縮小資料範圍，找出仍能觸發問題的少量紀錄。公開附件前，請先移除私人資訊及不宜公開的研究資料；若能使用虛構資料重現，也有助於公開討論與後續測試。

有興趣閱讀原始碼、參與測試或自行編譯，可參考[開發說明](docs/development.md)。

**一般使用者只需下載 Mac App，不必自行編譯。**

## 授權與第三方元件

HistCite Mac 的原創程式碼採用 [MIT 授權](LICENSE)。第三方元件保留各自的授權，包括：

- **Graphviz 2.2.1**：CPL 1.0
- **排序程式**：BSD-3-Clause

第三方授權聲明及完整 Graphviz 原始碼已隨 App 提供，也可在倉庫中查閱 [Graphviz 與排序程式聲明](ui/THIRD-PARTY.txt)，以及 [Rust 相依元件聲明](licenses/Rust-dependencies.txt)。

[**下載 HistCite Mac，開始分析文獻**](https://github.com/yangnim21029/histcite-mac/releases/latest)
