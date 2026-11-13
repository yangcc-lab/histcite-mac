"""Build and package HistCite Mac. Add --universal for Apple Silicon + Intel."""
from pathlib import Path
import argparse
import os
import platform
import subprocess
import sys

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--universal', action='store_true')
args = parser.parse_args()
if platform.system() != 'Darwin':
    parser.error('The desktop app must be built on macOS.')
targets = ['aarch64-apple-darwin', 'x86_64-apple-darwin'] if args.universal else [
    'aarch64-apple-darwin' if platform.machine() == 'arm64' else 'x86_64-apple-darwin']
env = os.environ.copy()
target_dir = Path(env.get('CARGO_TARGET_DIR', root / 'desktop/target')).resolve()
env['MACOSX_DEPLOYMENT_TARGET'] = '13.0'
# Keep local home-directory paths out of distributed Rust panic/debug strings.
env['CARGO_ENCODED_RUSTFLAGS'] = '\x1f'.join(filter(None, [
    env.get('CARGO_ENCODED_RUSTFLAGS', ''), f'--remap-path-prefix={Path.home()}=/build']))
for target in targets:
    subprocess.run(['rustup', 'target', 'add', target], check=True)
    subprocess.run(['cargo', 'build', '--release', '--locked', '--manifest-path',
                    str(root / 'desktop/Cargo.toml'), '--target', target], env=env, check=True)
binaries = [target_dir / target / 'release/histcite-desktop' for target in targets]
if args.universal:
    binary = target_dir / 'universal/histcite-desktop'
    binary.parent.mkdir(exist_ok=True)
    subprocess.run(['lipo', '-create', *map(str, binaries), '-output', str(binary)], check=True)
else:
    binary = binaries[0]
subprocess.run([sys.executable, str(root / 'scripts/package_macos.py'), '--binary', str(binary)], check=True)
