# 用 HistCite Mac 分析 Web of Science 匯出資料

[English](citation-analysis.md)

用自己從 Web of Science（WoS）匯出的資料，找出哪些文獻引用了同一篇文章，再把引用關係畫成圖。以下沿用 HistCite Mac v0.3.4 的英文選單名稱。

我們曾將兩份 Web of Science 原始匯出檔直接交給分析核心，未經外部清洗。這次驗證讀入 **1,217 筆文獻**和 **62,116 筆參考文獻條目**，建立 **954 條集合內引用連線**。[直接匯入驗證](compatibility.md#direct-import-check)。

## 匯出並開啟文獻集合

1. 在 Web of Science 的檢索結果頁選擇 **Export → Plain Text File**，紀錄內容選 **Full Record and Cited References**。保留原始 `.txt` 檔案，記下檢索詞、所選資料庫和匯出日期。[Web of Science 匯出選項](https://webofscience.zendesk.com/hc/en-us/articles/20135124532113-Search-Results)。
2. 依照 [Mac 安裝說明](../README.zh-TW.md#1-下載並安裝)裝好程式，選擇 **File → Open collection…**，開啟一個或多個匯出檔。
3. 解讀結果前，先到 **Tools → Data checks** 查看匯入警告和文獻合併紀錄。

## 找出一條引用的來源

1. 點選 **LCS** 欄位排序，找一筆 LCS 大於 0 的文獻。LCS 代表目前集合裡有幾篇文獻引用它。**GCS** 則沿用 Web of Science 匯出當時所記錄的被引用次數。
2. 點標題旁的文獻編號，開啟文獻詳情。從 **Citing records** 找到一篇引用它的文獻，展開該筆下方的 **Match evidence**，查看比對方式與參考文獻原文。
3. 如果所有文獻的 LCS 都是 0，可回到 **Data checks** 查看是否缺少參考文獻清單。集合中的文獻也可能沒有彼此引用。

## 畫出引用關係圖

回到主視窗，選擇 **Tools → Graph Maker…**。圖上會依出版年排列納入圖中的文獻。把游標移到節點上可查看資訊，點選節點可開啟文獻詳情。要儲存圖片，在圖形視窗按 **SVG**。若要把圖用在研究報告，請記下 App 版本、建立集合的檢索條件，以及圖上文獻的篩選設定。

## 研究文獻與延伸閱讀

- Eugene Garfield 在 2004 年的論文中說明，原版 HistCite 如何以 Web of Science 匯出資料繪製按時間排列的引文圖譜。[Historiographic Mapping of Knowledge Domains Literature](https://journals.sagepub.com/doi/10.1177/0165551504042802)。
- 賓州大學的典藏收錄了原版 HistCite 的方法論文及應用案例。[Papers on Algorithmic Historiography](https://garfield.library.upenn.edu/algorithmichistoriographyhistcite.html)。
- 想進一步了解 Mac 版，可看[檢查結果與相容性限制](compatibility.md)，以及[引用比對的實作說明](citation-matching.md#繁體中文)。HistCite Mac 由 `yangnim21029` 獨立開發，參照 Eugene Garfield 與合作團隊開發的原版 HistCite 工作流程。
