"""Build Graphviz 2.2.1 as a self-contained offline WebAssembly module.

Requires Emscripten 4.0.10 (EMCC or .runtime/emsdk/upstream/emscripten/emcc).
Upstream layout code is unmodified; the port reproduces Windows CRT sorting
for equal spline segments and disables optional rendering libraries.
Each layout gets a fresh WASM instance, isolating the legacy engine's globals.
"""
from pathlib import Path
import os,re,subprocess,shutil
root=Path(__file__).resolve().parents[1]
(root/'ui').mkdir(exist_ok=True)
original=root/'vendor/graphviz-2.2.1'
src=root/'.runtime/graphviz-wasm-source'
shutil.copytree(original,src,dirs_exist_ok=True,ignore=shutil.ignore_patterns('.libs','*.o','*.lo','*.a','*.la','dot','config.log'))
shutil.copy2(root/'vendor/graphviz-port/config.h',src/'config.h')
emcc=os.environ.get('EMCC',str(root/'.runtime/emsdk/upstream/emscripten/emcc'))
dirs=['cdt','graph','pathplan','gd','dotneato/common','dotneato/gvrender','dotneato/dotgen']
sources=[]
for d in dirs:
    make=(src/d/'Makefile.am').read_text().replace('\\\n',' ')
    line=re.search(r'^lib\w+_la_SOURCES\s*=\s*(.+)',make,re.M).group(1)
    sources.extend(str(src/d/f.replace('.y','.c')) for f in line.split() if f.endswith(('.c','.y')))
sources.append(str(src/'dotneato/dot.c'))
sources.append(str(root/'vendor/graphviz-port/legacy-sort.c'))
args=[emcc,'-O2','-std=gnu89','-DHAVE_CONFIG_H','-Dqsort=hc_legacy_qsort','-Wno-implicit-function-declaration','-Wno-int-conversion','-Wno-incompatible-function-pointer-types','-Wno-deprecated-non-prototype','-Wno-pointer-sign','-I'+str(root/'vendor/graphviz-port')]
args+=['-I'+str(src/d) for d in dirs]+['-I'+str(src)]
args+=sources+['-sMODULARIZE=1','-sEXPORT_NAME=HistCiteDot','-sSINGLE_FILE=1','-sENVIRONMENT=web,worker,node','-sALLOW_MEMORY_GROWTH=1','-sMAXIMUM_MEMORY=268435456','-sSTACK_SIZE=8388608','-sEXPORTED_RUNTIME_METHODS=FS,callMain','-sINVOKE_RUN=0','-sEXIT_RUNTIME=1','-sEMULATE_FUNCTION_POINTER_CASTS=1','-o',str(root/'ui/graphviz.js')]
subprocess.run(args,check=True)
print(root/'ui/graphviz.js')
