"""Include all source needed to reproduce the bundled Graphviz WASM module."""
from pathlib import Path
from zipfile import ZipFile, ZIP_DEFLATED

root = Path(__file__).resolve().parents[1]
original = root / '.runtime/graphviz-2.2.1.tar.gz'
if original.exists():
    upstream = original.read_bytes()
else:
    with ZipFile(root / 'ui/Graphviz-source.zip') as previous:
        upstream = previous.read('graphviz-2.2.1.tar.gz')
with ZipFile(root / 'ui/Graphviz-source.zip', 'w', ZIP_DEFLATED) as archive:
    archive.writestr('graphviz-2.2.1.tar.gz', upstream)
    for name in ['vendor/graphviz-port/config.h', 'vendor/graphviz-port/legacy-sort.c',
                 'scripts/build_graphviz.py', 'ui/THIRD-PARTY.txt']:
        archive.write(root / name, name)
    archive.writestr('README-LICENSE.txt', '''Graphviz 2.2.1, CPL 1.0; sanos sorting adapter, BSD-3-Clause.
Full licenses and source attribution are in ui/THIRD-PARTY.txt.

Rebuild:
  mkdir -p vendor
  tar -xzf graphviz-2.2.1.tar.gz -C vendor
  EMCC=/path/to/emsdk/upstream/emscripten/emcc python3 scripts/build_graphviz.py

Use Emscripten 4.0.10. Output: ui/graphviz.js (self-contained WASM/JS).
The build substitutes hc_legacy_qsort for C library qsort. The upstream
Graphviz source is unchanged; the complete portable config, sort source
and build command are included here. No HistCite executable or code is included.
''')
print(root / 'ui/Graphviz-source.zip')
