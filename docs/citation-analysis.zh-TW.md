# 用 HistCite Mac 做第一次引文分析

[English](citation-analysis.md)

先用小型文獻集合找出哪些文獻引用了同一篇文章，再把這些關係畫成圖。本教學使用 **12 筆虛構文獻**，並保留 HistCite Mac v0.3.4 的英文選單名稱。跟著操作不需要 Web of Science 帳號。

## 開啟範例集合

依照 [Mac 安裝說明](../README.zh-TW.md#1-下載並安裝)裝好程式後，從磁碟映像檔複製 `demo-wos.txt`，或[直接下載同一份範例](https://github.com/yangnim21029/histcite-mac/releases/download/v0.3.4/demo-wos.txt)。在程式中選擇 **File → Open collection…**，開啟範例。集合會顯示 **12 筆文獻、31 條引用連線**。

## 找出一條引用的來源

1. 點選 **LCS** 欄位，按文獻在集合內的被引用次數排序。找到 Avery 在 2011 年發表的範例文獻 *Starting a literature map*，它的 **LCS 為 5**、**GCS 為 120**。集合中有五筆文獻引用 Avery。範例的 GCS 則是模擬資料庫所記錄的引用次數，計算範圍不同。
2. 點標題旁的文獻編號，開啟 Avery 的文獻詳情。從 **Citing records** 找到 Bennett 在 2012 年發表的範例文獻 *Finding connections between papers*。
3. 展開 Bennett 下方的 **Match evidence**，應會看到 `histcite_key: Avery A, 2011, DEMO J RES, V1, P1`。這一行列出比對方式，以及匯入 Bennett 那筆資料時讀到的參考文獻原文。

## 畫出引用關係圖

回到主視窗，選擇 **Tools → Graph Maker…**。圖上會依出版年排列範例文獻。把游標移到節點上可查看文獻資訊，點選節點可開啟文獻詳情。試著在時間軸較早的位置找出 Avery 和 Bennett。如果要儲存圖片，在圖形視窗按 **SVG**。

![Graph Maker 繪出的虛構範例文獻引用圖](images/graph.png)

## 換成自己的研究資料

從 Web of Science 匯出檢索結果時，選擇 **Plain Text** 和 **Full Record and Cited References**。保留原始檔案，並記下檢索詞、所選資料庫、匯出日期和 App 版本。解讀引用圖前，可以先到 **Tools → Data checks** 查看匯入警告和合併紀錄。LCS 會隨納入集合的文獻改變，GCS 則沿用原始匯出檔的數值。想了解一篇文獻為什麼引用另一篇，仍要回到原文閱讀。

## 研究文獻與延伸閱讀

- Eugene Garfield 在 2004 年的論文中說明，原版 HistCite 如何以 Web of Science 匯出資料繪製按時間排列的引文圖譜。[Historiographic Mapping of Knowledge Domains Literature](https://journals.sagepub.com/doi/10.1177/0165551504042802)。
- 賓州大學的典藏收錄了原版 HistCite 的方法論文及應用案例。[Papers on Algorithmic Historiography](https://garfield.library.upenn.edu/algorithmichistoriographyhistcite.html)。
- 想進一步了解 Mac 版，可看[檢查結果與相容性限制](compatibility.md)，以及[引用比對的實作說明](citation-matching.md#繁體中文)。HistCite Mac 由 `yangnim21029` 獨立開發，參照 Eugene Garfield 與合作團隊開發的原版 HistCite 工作流程。
