"""Package a built macOS executable as an ad-hoc signed .app."""
from pathlib import Path
import argparse
import json
import plistlib
import shutil
import subprocess
import tempfile

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--binary', type=Path, default=root / 'desktop/target/release/histcite-desktop')
args = parser.parse_args()
if not args.binary.is_file():
    parser.error('Build desktop/Cargo.toml in release mode first, or pass --binary.')
config = json.loads((root / 'desktop/tauri.conf.json').read_text())
dist = root / 'dist'
dist.mkdir(exist_ok=True)
with tempfile.TemporaryDirectory(prefix='.histcite-build-', dir=dist) as temporary:
    staging = Path(temporary)
    app = staging / 'HistCite Mac.app'
    contents = app / 'Contents'
    (contents / 'MacOS').mkdir(parents=True)
    resources = contents / 'Resources'
    resources.mkdir()
    shutil.copy2(args.binary, contents / 'MacOS/histcite-desktop')
    for name in ['THIRD-PARTY.txt', 'Graphviz-source.zip']:
        shutil.copy2(root / 'ui' / name, resources / name)
    shutil.copy2(root / 'LICENSE', resources / 'LICENSE.txt')
    shutil.copy2(root / 'licenses/Rust-dependencies.txt', resources / 'Rust-dependencies.txt')
    shutil.copy2(root / 'examples/demo-wos.txt', resources / 'demo-wos.txt')
    iconset = staging / 'AppIcon.iconset'
    iconset.mkdir()
    for size in [16, 32, 128, 256, 512]:
        for scale in [1, 2]:
            name = f'icon_{size}x{size}' + ('@2x' if scale == 2 else '') + '.png'
            subprocess.run(['/usr/bin/sips', '-z', str(size * scale), str(size * scale),
                            str(root / 'desktop/icons/icon.png'), '--out', str(iconset / name)],
                           check=True, stdout=subprocess.DEVNULL)
    subprocess.run(['/usr/bin/iconutil', '-c', 'icns', str(iconset), '-o',
                    str(resources / 'AppIcon.icns')], check=True)
    info = {
        'CFBundleName': config['productName'], 'CFBundleDisplayName': config['productName'],
        'CFBundleExecutable': 'histcite-desktop', 'CFBundleIdentifier': config['identifier'],
        'CFBundlePackageType': 'APPL', 'CFBundleShortVersionString': config['version'],
        'CFBundleVersion': config['version'], 'CFBundleIconFile': 'AppIcon',
        'LSMinimumSystemVersion': '13.0', 'NSHighResolutionCapable': True,
    }
    (contents / 'Info.plist').write_bytes(plistlib.dumps(info))
    subprocess.run(['/usr/bin/codesign', '--force', '--sign', '-', '--timestamp=none', str(app)], check=True)
    subprocess.run(['/usr/bin/codesign', '--verify', '--strict', str(app)], check=True)
    target = dist / app.name
    if target.exists():
        shutil.rmtree(target)
    app.rename(target)
print(target)
