"""Create the drag-to-Applications download and its SHA-256 checksum."""
from pathlib import Path
import hashlib
import json
import shutil
import subprocess
import tempfile

root = Path(__file__).resolve().parents[1]
app = root / 'dist/HistCite Mac.app'
if not app.is_dir():
    raise SystemExit('Run python3 scripts/build_macos.py --universal first.')
binary = app / 'Contents/MacOS/histcite-desktop'
architectures = subprocess.check_output(['lipo', '-archs', str(binary)], text=True).split()
if set(architectures) != {'arm64', 'x86_64'}:
    raise SystemExit('The public DMG requires a universal binary. Use build_macos.py --universal.')
version = json.loads((root / 'desktop/tauri.conf.json').read_text())['version']
output = root / f'dist/HistCite-Mac-{version}-universal.dmg'
with tempfile.TemporaryDirectory(prefix='histcite-dmg-') as temporary:
    folder = Path(temporary)
    shutil.copytree(app, folder / app.name)
    (folder / 'Applications').symlink_to('/Applications', target_is_directory=True)
    shutil.copy2(root / 'examples/demo-wos.txt', folder / 'demo-wos.txt')
    (folder / 'Start here.txt').write_text(
        'HistCite Mac\n\n'
        '1. Drag HistCite Mac to Applications.\n'
        '2. Open HistCite Mac from Applications.\n'
        '3. Choose File > Open collection and select a Web of Science plain-text export.\n'
        '   Select Full Record and Cited References when exporting from Web of Science.\n'
        '   To try the app first, copy demo-wos.txt to your Mac and open it.\n\n'
        'First launch: this community build is not Apple-notarized. If macOS blocks it,\n'
        'open System Settings > Privacy & Security > Open Anyway, then reopen the app.\n\n'
        '安裝：將 HistCite Mac 拖進 Applications，再從「應用程式」開啟。\n'
        '首次開啟若被 macOS 阻擋：系統設定 > 隱私權與安全性 > 強制打開。\n'
        '選 File > Open collection，開啟 WoS 純文字匯出檔即可開始。\n\n'
        'Guide: https://github.com/yangnim21029/histcite-mac\n', encoding='utf-8')
    subprocess.run(['hdiutil', 'create', '-ov', '-volname', 'HistCite Mac', '-srcfolder',
                    str(folder), '-format', 'UDZO', str(output)], check=True)
subprocess.run(['hdiutil', 'verify', str(output)], check=True)
checksum = hashlib.sha256(output.read_bytes()).hexdigest()
(root / 'dist/SHA256SUMS.txt').write_text(f'{checksum}  {output.name}\n')
print(output)
