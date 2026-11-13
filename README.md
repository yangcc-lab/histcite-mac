# HistCite Mac

**Explore a literature collection, follow its citations, and draw its history — on your Mac.**

HistCite Mac brings familiar HistCite tables, bibliometric analyses and Graph Maker workflows to a standalone Mac app. Import a Web of Science export and start exploring. Your data is processed locally.

**[Download for Mac](https://github.com/yangnim21029/histcite-mac/releases/latest/download/HistCite-Mac-0.3.2-universal.dmg)** · [繁體中文](README.zh-TW.md) · [Releases](https://github.com/yangnim21029/histcite-mac/releases)

macOS 13 Ventura or later · Apple Silicon and Intel · Free and open source

## Start in three steps

1. **Download and open the DMG.** Drag **HistCite Mac** into **Applications**.
2. **Open HistCite Mac.** Choose **File → Open collection…** and select your WoS `.txt` export. You can select several files together.
3. **Explore.** Click **LCS** to sort by citations within your collection, open a paper's details, or choose **Tools → Graph Maker…** for a citation timeline.

When exporting from Web of Science, choose **Plain Text** and **Full Record and Cited References**. The app is ready to use: no terminal, Rust, Python, Windows or Wine installation is needed.

Want to try it first? The DMG includes **demo-wos.txt**, a small fictional collection. Copy it to your Mac and open it in the app.

> **First launch:** this community release is not Apple-notarized. If macOS blocks it, open **System Settings → Privacy & Security → Open Anyway**, then reopen HistCite Mac. This is a one-time approval for the app. [Apple's instructions](https://support.apple.com/en-us/102445).

![Literature list with local and global citation counts, using fictional demo records](docs/images/records.png)

## From a collection to a citation map

- **Read and compare:** independent paper windows, abstracts, references, related records and citation counts.
- **Analyze:** authors, journals, cited references, words, years, document types, languages, institutions, subdivisions and countries.
- **Organize:** search, mark, tag, edit and merge records, with undo and a duplicate-merge history.
- **Draw:** citation timelines with node sizing, direction, labels and merged links. Hover for a citation; click to open the paper.
- **Save and share:** native `.hcm` collections, HistCite `.hci`, CSV, offline HTML, SVG, PNG, PostScript, DOT and Pajek.

![Graph Maker showing the citation history of the fictional demo collection](docs/images/graph.png)

## Everyday questions

**Where is my data?** On your Mac. Analysis runs locally and does not upload your collection. DOI and WoS links open your browser when you choose them. Original import files are preserved; use **File → Save as…** to save your working collection as `.hcm`.

**Which files can I open?** Web of Science tagged plain-text exports (`.txt`), HistCite collections (`.hci`) and HistCite Mac collections (`.hcm`). For Excel or Scopus exports, export from WoS in the format above or convert to a compatible tagged format first.

**Can I keep working later?** Yes. Save an `.hcm` collection to keep records, tags, settings and saved graphs together. The app also restores your last working collection on launch.

**Is this the original HistCite?** This is an independent Rust implementation following HistCite 12.03.17 workflows, with no affiliation to the original authors. Citation counts, analytical groups, merging and graph layout have been compared with the original on reference cases. See [compatibility and validation](docs/compatibility.md) for the tested scope.

## Contributing

[Report a bug or request a feature](https://github.com/yangnim21029/histcite-mac/issues). Include your macOS version and the steps to reproduce it. A small synthetic example is ideal; please remove private data from attachments.

For source builds, tests and architecture, see the [developer guide](docs/development.md). Building from source is optional; the download above is the normal installation.

## License

HistCite Mac's original code is [MIT licensed](LICENSE). The bundled Graphviz 2.2.1 layout engine retains **CPL 1.0**; its sorting adapter retains **BSD-3-Clause**. Third-party notices and complete Graphviz source are included in the app and in [the repository](ui/THIRD-PARTY.txt).
