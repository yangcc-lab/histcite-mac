# Analyze a Web of Science export with HistCite Mac

[繁體中文](citation-analysis.zh-TW.md)

Use the records from your own Web of Science search to find which papers cite a particular work and draw a graph of those relationships. The steps use the English menus in HistCite Mac v0.3.5.

In a [direct-import check](compatibility.md#direct-import-check), the shared analysis core read two original Web of Science exports without external cleaning. It processed **1,217 records** and **62,116 cited-reference entries**, producing **954 links within the collection**.

## Export and open your collection

1. In the Web of Science results, choose **Export → Plain Text File** and select **Full Record and Cited References** for the record content. Keep the original `.txt` files and record your search terms, database selection and export date. [Web of Science export options](https://webofscience.zendesk.com/hc/en-us/articles/20135124532113-Search-Results).
2. Follow the [Mac installation instructions](../README.md#1-download-and-install). Choose **File → Open collection…** and select the file or files you exported.
3. Open **Tools → Data checks** to review import warnings and record merges before interpreting the results.

## Trace a citation back to its source

1. Click the **LCS** column and choose a record with a positive score. LCS counts records in your collection that cite the selected work. **GCS** reflects the citation count supplied by Web of Science at export.
2. Click the record number beside the title. Under **Citing records**, choose one of the entries and expand its **Match evidence**. You can see the matching method and the reference text from the citing record.
3. If all local scores are zero, check for missing reference lists in **Data checks**. The papers in a collection may also have no direct citation links to each other.

## Draw the citation graph

Return to the main window and choose **Tools → Graph Maker…**. The graph arranges its selected records by publication year. Hover over a node to see its details or click it to open the record. Choose **SVG** in the graph window to save the figure. If you use the graph in a report, record the app version, the search behind the collection and the settings you used to select the plotted records.

## Research and further reading

- Eugene Garfield's 2004 paper describes how the original HistCite used Web of Science exports to map literature in chronological order. [Historiographic Mapping of Knowledge Domains Literature](https://journals.sagepub.com/doi/10.1177/0165551504042802).
- The University of Pennsylvania archive collects papers about the original HistCite and related applications. [Papers on Algorithmic Historiography](https://garfield.library.upenn.edu/algorithmichistoriographyhistcite.html).
- For the independent Mac implementation, see [verification results and compatibility limits](compatibility.md) and [how citation matching works](citation-matching.md). HistCite Mac is developed by `yangcc-lab` and follows workflows from the original HistCite by Eugene Garfield and collaborators.
