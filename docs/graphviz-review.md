# Graphviz warning triage — 14 November 2026

This follow-up classifies the 50 warnings from the additional Semgrep scan described in the [security review](security-review.md). It is a source review of those reported sites, **not a full C audit, fuzzing campaign, or proof that the engine is safe**. The scan's 90 parser/partial-parser entries remain a coverage limitation.

The source is Graphviz 2.2.1 in [the bundled source archive](../ui/Graphviz-source.zip). Compilation units come from the `Makefile.am` source lists selected by [build_graphviz.py](../scripts/build_graphviz.py); preprocessing uses [the port's config.h](../vendor/graphviz-port/config.h). A file appearing in a compilation list does not mean every feature in it is enabled or retained by the linker.

## Scope disposition

| Reported sites | Count | Evidence and disposition for this port |
|---|---:|---|
| `dotneato/gvre_svg/*`, `dotneato/neatogen/solve.c`, `dotneato/pack/ccomps.c`, `gd/annotate.c`, `gd/gdtest.c` | 21 | Not among the selected compilation units. This includes both double-free warnings and all four printf warnings. They are not cleared for other Graphviz builds. |
| `common/cairogen-win.c` | 2 | Excluded by undefined `HAVE_LIBSVG_CAIRO`. |
| `common/htmllex.c`, entity conversion | 1 | Excluded by undefined `HAVE_LIBEXPAT`. |
| `common/input.c`, environment fallback | 2 | Excluded because `HAVE_SETENV` is defined. |
| `common/vrmlgen.c` | 1 | Excluded by undefined `HAVE_GD_PNG`. |
| `gd/gd_jpeg.c` | 1 | Excluded by undefined `HAVE_LIBJPEG`. |
| `graph/parser.c`, obsolete port grammar | 3 | Inside `#ifdef UNUSED`; not part of the compiled grammar. Scanner locations near the trailing grammar are approximate. |
| Remaining sites below | 19 | Compiled source; inspect context rather than treating the presence of a string API as a demonstrated overflow. |

Thus 21 warnings concern omitted files, 10 concern disabled source sections, and 19 concern enabled source sections. This classification does not remove warnings from the original scan or broaden its coverage.

## Enabled-source sites

| Sites | Count | Context and remaining limits |
|---|---:|---|
| `common/emit.c`: layer and multicolor tokenization | 8 | `strtok` operates on duplicated strings or an `agxbuf`. The current DOT builder emits no `layers`/`layer` attributes or colon-separated colors. Each layout uses a new single-threaded WASM instance. This limits exposure to the tokenization state; it is not a general reentrancy review of Graphviz. |
| `common/gdgen.c`: `gd_alternate_fontlist` | 1 | Destination capacity grows to twice `strlen(font)+1` before the copy. The generated graph uses the engine's default font family. Allocation failure handling and the legacy allocation strategy are not certified by this check. |
| `common/picgen.c`: `pic_string` | 1 | The buffer grows with eight bytes of spare capacity before the five-byte copy of a fixed escape sequence. PIC is also not a permitted output format in the worker interface. |
| `gvrender/gvconfig.c`: `gvconfig_list_plugins` | 1 | This enumerates registered renderer names when the requested renderer is unrecognized. The worker now accepts only supported `svg` and `ps`. This does not validate arbitrary plugin configurations. |
| `gd/gd_gif_in.c`: GIF version copy | 1 | A three-byte copy is followed by an explicit terminator in a four-byte array. The app supplies no image files or image attributes to the virtual filesystem. This is not a GIF decoder audit. |
| `graph/parser.c`: `concat`, `concat3` | 5 | Buffer length includes all strings and the terminator. The generated DOT contains no quoted-string concatenation operator or node ports. Legacy signed length arithmetic and allocation failures remain outside the claim; arbitrary external DOT is not an app input format. |
| `graph/refstr.c`: `agstrdup`, `agstrdup_html` | 2 | Allocation includes the structure's one-byte trailing array plus `strlen(s)`, leaving room for the terminator. The builder uses numeric IDs, years/months and fixed option strings, not imported titles or abstracts, in DOT labels. This does not establish correctness under allocation failure or for other engine callers. |

No demonstrated vulnerability in the shipped invocation was established from these 50 patterns. That statement applies to these sites and the current input/output boundary only; it does not clear unscanned code, parser failures, dependencies, or historical Graphviz advisories.

## Changes and validation

Version 0.3.4 adds an explicit SVG/PostScript allowlist at `GraphLayout.layout`, before a worker is started. PNG remains supported by rasterizing the resulting SVG in the browser; DOT and Pajek export do not invoke additional Graphviz renderers. The engine binary and layout algorithm are unchanged.

Browser tests check rejection of unsupported renderer requests, successful SVG/PostScript rendering, and existing interactions. The saved 28 original-HistCite geometry/order comparisons remain compatibility evidence. A dedicated review of the full compiled C paths and controlled fuzzing remain future work.
