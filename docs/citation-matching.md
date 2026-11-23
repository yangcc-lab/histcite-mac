# How a reference becomes a citation link

[繁體中文](#繁體中文) · [Try it in the app](citation-analysis.md)

A reference in a Web of Science export is a line of text. To draw a citation graph, HistCite Mac needs to identify which record in the collection that line refers to. The default `Histcite` mode follows matching behavior observed in the original HistCite 12.03.17. [Compatibility checks](compatibility.md).

In the [fictional example](../examples/demo-wos.txt), Bennett's record contains `CR Avery A, 2011, DEMO J RES, V1, P1`. The parser reads this as a cited reference. The Rust core builds a lookup key from the first author's surname and first initial, the year, volume and first page. In this mode, the journal and DOI do not enter that key. Exactly one collection record, Avery's, matches the sample reference. [Matching code](../src/lib.rs).

The core stores one directed link from Bennett to Avery and keeps the original reference alongside the matching method. That link adds one to Avery's **LCS**, the collection's count of records citing Avery. The exported **GCS** stays as supplied. If more than one distinct record matches a reference, the program records the ambiguity for review and leaves that link unresolved. [Record details](../ui/detail.js).

This is one of the differences that matter when working on compatibility. To propose a change, identify which input or behavior should change and include a small example with the expected result, using fictional or shareable data. The [developer guide](development.md#contribute-a-fix-or-improvement) points to the matching fixtures and test command.

## 繁體中文

[跟著教學操作](citation-analysis.zh-TW.md)

Web of Science 匯出檔中的參考文獻是一行文字。要畫出引用圖，HistCite Mac 得先找出這行文字指的是集合裡哪一筆文獻。預設的 `Histcite` 模式依照實測觀察到的原版 HistCite 12.03.17 行為進行比對。[相容性檢查](compatibility.md)。

[虛構範例](../examples/demo-wos.txt)中，Bennett 的紀錄包含 `CR Avery A, 2011, DEMO J RES, V1, P1`。程式把這一行讀成參考文獻，再由 Rust 分析核心用第一作者的姓氏、名字首字母、年份、卷號和起始頁建立比對鍵。這個模式的比對鍵不含期刊名稱與 DOI。上述參考文獻在集合中只對應到一筆文獻，也就是 Avery 那筆。[比對程式碼](../src/lib.rs)。

分析核心會建立一條從 Bennett 指向 Avery 的連線，並保留參考文獻原文和比對方式。這條連線計入 Avery 的 **LCS**，也就是集合裡引用 Avery 的文獻數。匯出檔提供的 **GCS** 維持原值。如果一條參考文獻同時符合多筆不同的紀錄，程式會標記供人工檢查，暫時不建立連線。[文獻詳情程式碼](../ui/detail.js)。

這類比對行為會影響和原版的相容性。若想提議改善，請說明是哪種輸入或行為需要改變，附一份虛構或可公開的小範例，以及預期結果。[開發指南](development.md#contribute-a-fix-or-improvement)列出比對測試的資料位置與執行方式。
