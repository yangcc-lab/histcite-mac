# Trace citations and draw citation graphs with HistCite Mac

[繁體中文](https://github.com/yangcc-lab/histcite-mac/blob/main/docs/citation-analysis.zh-TW.md)

A large collection of records from Web of Science (WoS) takes time to read. Finding which works are cited repeatedly and how studies from different years connect can help you decide where to start. HistCite Mac combines citation counts, the reference text behind each matched citation, and citation graphs to help you investigate those relationships.

This guide uses the English menus in HistCite Mac v0.3.5 to import records, analyze citations, and draw a citation graph.

## 1. Import records and check the data

1. On the WoS results page, choose **Export → Plain Text File** and select **Full Record and Cited References** for the record content. Keep the original `.txt` files and record the search criteria.
2. Follow the [Mac installation instructions](https://github.com/yangcc-lab/histcite-mac/blob/main/README.md#1-download-and-install). Choose **File → Open collection…** and select one or more WoS export files.
3. Open **Tools → Data checks** to see which records lack cited references and review how duplicate records were merged.

The reliability of the analysis depends on the imported data. HistCite Mac identifies the works cited in each reference list to establish links within the collection. Check the completeness of those lists first so you can judge whether the data support a finding of no local citations.

## 2. Compare LCS and GCS to find papers to read

After importing the records, sort by **LCS** to see which papers are cited most often within the current collection.

- **LCS (Local Citation Score)** counts the distinct records in the current collection that cite a work. An LCS of 8 means eight records in the collection cite it.
- **GCS (Global Citation Score)** is the citation count supplied by WoS at the time of export. It stays fixed after import.

The two scores help guide your reading in different ways. For example, a work with a high GCS and a low LCS may warrant a closer look at its relevance to your current topic. For a work with a high LCS, you can check which records in your collection cite it.

Search scope, inclusion criteria, and data completeness all affect citation counts. **Use the scores to prioritize your reading, then read the papers to assess their contributions.**

## 3. Trace a citation and inspect the match evidence

LCS tells you how many records cite a work. Open the record details to see which papers cited it and how the citations were matched.

1. Click the record number beside the title to open the record details.
2. Under **Citing records**, find a paper that cites the work.
3. Expand **Match evidence** to check the original reference text and the matching method.

HistCite Mac compares cited references with bibliographic records in the collection. The default Histcite mode builds a matching key primarily from the first author's surname and first initial, the publication year, the volume, and the starting page. A unique match establishes a citation link. When several records match, the program records the ambiguity for review.

**Match evidence lets you trace a citation count back to the records that contribute to it.** If every LCS is zero, use **Data checks** to look for missing data before judging whether the collection contains any direct citations.

## 4. Draw a citation graph to explore relationships between studies

Choose **Tools → Graph Maker…** to plot the records and their citation links. The graph arranges the selected papers by publication year. Hover over a node for information about the paper or click it to open the record details.

LCS summarizes how often a paper is cited within the collection. The graph shows which works cite which others and when they were published. When several later papers cite an early study, you can identify them in the graph and read them to see how they used, extended, or revised the earlier work.

**You can trace citation relationships in the graph and read the papers themselves to understand how their ideas are connected.**

Choose **SVG** in the graph window to export the figure. If you use it in a research report, keep the original WoS files and record the search criteria, export date, app version, and graph filter settings so you can revisit the analysis.

## 5. Validation and limitations

On November 16, 2026, HistCite Mac processed two original WoS exports without external cleaning. In the default Histcite mode, it read **1,217 records and 62,116 cited-reference entries**, established **954 citation links within the collection**, and reported no ambiguous matches.

This check used the command-line interface (CLI) and the shared Rust analysis core. The native Mac app was tested separately. The reader requires UTF-8 text and a valid WoS field structure. The results apply to the files and conditions tested.

Treat data import, citation matching, and research interpretation as separate steps. The first two depend on the data and the software. To understand what a citation means for your research, consider the search criteria and read the papers themselves.

See the [compatibility and validation record](https://github.com/yangcc-lab/histcite-mac/blob/main/docs/compatibility.md) and [how citation matching works](https://github.com/yangcc-lab/histcite-mac/blob/main/docs/citation-matching.md) for more detail.

## Further reading

- Eugene Garfield (2004), [Historiographic Mapping of Knowledge Domains Literature](https://journals.sagepub.com/doi/10.1177/0165551504042802).
- University of Pennsylvania archive, [Papers on Algorithmic Historiography](https://garfield.library.upenn.edu/algorithmichistoriographyhistcite.html).

HistCite Mac is an unofficial, independent implementation by `yangcc-lab`. It follows workflows from the original HistCite, developed by Eugene Garfield and his collaborators.
