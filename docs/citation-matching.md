# How a reference becomes a citation link

[繁體中文](#繁體中文) · [Try it in the app](citation-analysis.md)

A reference in a Web of Science export is a line of text. To draw a citation graph, HistCite Mac needs to identify which record in the collection that line refers to. The default `Histcite` mode follows matching behavior observed in the original HistCite 12.03.17. [Compatibility checks](compatibility.md).

The parser reads cited references from the `CR` field. The Rust core uses the first author's surname and first initial, the year, volume and first page to build a lookup key. In this mode, the journal and DOI do not enter that key. [Matching code](../src/lib.rs).

When exactly one local record matches, the core stores a link from the citing record to that record and keeps the original reference alongside the matching method. Each distinct citing record counts once toward the matched record's **LCS**. The exported **GCS** stays as supplied. If several local records match the reference, the program records the ambiguity for review and leaves that link unresolved. [Record details](../ui/detail.js).

In a check on two original WoS exports, the core processed 62,116 cited-reference entries and produced 954 links among 1,217 records. See the [import results and scope](compatibility.md#direct-import-check). To propose a matching change, describe the behavior and include a small example you can share with the expected result. The [developer guide](development.md#contribute-a-fix-or-improvement) links to the fixtures and test command.

## 繁體中文

[跟著教學操作](citation-analysis.zh-TW.md)

Web of Science 匯出檔中的參考文獻是一行文字。要畫出引用圖，HistCite Mac 得先找出這行文字指的是集合裡哪一筆文獻。預設的 `Histcite` 模式依照實測觀察到的原版 HistCite 12.03.17 行為進行比對。[相容性檢查](compatibility.md)。

程式從 `CR` 欄位讀取參考文獻。Rust 分析核心用第一作者的姓氏、名字首字母、年份、卷號和起始頁建立比對鍵。這個模式的比對鍵不含期刊名稱與 DOI。[比對程式碼](../src/lib.rs)。

比對結果只有一筆時，分析核心會從引用者建立一條連線指向被引文獻，並保留參考文獻原文和比對方式。每篇引用同一目標的文獻只計入該目標的 **LCS** 一次。匯出檔提供的 **GCS** 維持原值。如果一條參考文獻同時符合多筆文獻，程式會標記供人工檢查，暫時不建立連線。[文獻詳情程式碼](../ui/detail.js)。

曾用兩份 WoS 原始匯出檔進行驗證。分析核心讀取 62,116 筆參考文獻條目，在 1,217 筆文獻之間建立 954 條連線。[匯入結果與驗證範圍](compatibility.md#direct-import-check)。若想提議改善比對方式，請說明需要改變的行為，附上可公開的小範例和預期結果。[開發指南](development.md#contribute-a-fix-or-improvement)列出測試資料位置與執行方式。
