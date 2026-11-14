"""Collect source license notices for the resolved macOS Rust dependency graph."""
from pathlib import Path
import json
import subprocess
from urllib.request import urlopen

root = Path(__file__).resolve().parents[1]
metadata = json.loads(subprocess.check_output([
    'cargo', 'metadata', '--locked', '--format-version', '1', '--manifest-path',
    str(root / 'desktop/Cargo.toml'), '--filter-platform', 'aarch64-apple-darwin'], text=True))
resolved = {node['id'] for node in metadata['resolve']['nodes']}
cache = {}

def fetch(url):
    if url not in cache:
        # URLs are constructed below from fixed upstream HTTPS hosts, never user input.
        # nosemgrep: python.lang.security.audit.dynamic-urllib-use-detected.dynamic-urllib-use-detected
        with urlopen(url, timeout=30) as response:
            cache[url] = response.read().decode('utf-8')
    return cache[url]

sections = ['Rust dependency notices for HistCite Mac\n'
            'Includes build-time and runtime crates resolved for macOS.\n'
            'Each package retains its upstream license; the project MIT license does not replace it.\n'
            'Exact package sources are available at the source URLs below.\n']
for package in sorted(metadata['packages'], key=lambda p: (p['name'], p['version'])):
    if package['id'] not in resolved or not package['source']:
        continue
    directory = Path(package['manifest_path']).parent
    files = [p for p in directory.iterdir() if p.is_file() and
             p.name.upper().startswith(('LICENSE', 'LICENCE', 'COPYING', 'COPYRIGHT', 'NOTICE'))]
    if package.get('license_file'):
        files.append(directory / package['license_file'])
    notices = [(p.name, p.read_text(errors='replace')) for p in sorted(set(files))]
    if not notices:
        vcs = json.loads((directory / '.cargo_vcs_info.json').read_text())
        commit = vcs['git']['sha1']
        repo = package.get('repository', '')
        if repo == 'https://github.com/madsmtm/objc2':
            # These releases state MIT terms in LICENSE.md but omit the text.
            # Preserve that release's notice and upstream's complete MIT text.
            url = f'https://raw.githubusercontent.com/madsmtm/objc2/{commit}/LICENSE.md'
            mit = 'https://raw.githubusercontent.com/madsmtm/objc2/8a5abacd90d5403fc0463459a35826f892b9142e/LICENSE-MIT.txt'
            notices.append((mit, fetch(mit)))
        elif package['name'] == 'alloc-stdlib':
            url = f'https://raw.githubusercontent.com/dropbox/rust-alloc-no-stdlib/{commit}/LICENSE'
        elif package['name'] == 'tauri-plugin':
            url = f'https://raw.githubusercontent.com/tauri-apps/tauri/{commit}/LICENSE-MIT'
        elif package['name'] == 'selectors':
            url = 'https://www.mozilla.org/media/MPL/2.0/index.txt'
        else:
            raise SystemExit(f'Missing license notice: {package["name"]} {package["version"]}')
        notices.append((url, fetch(url)))
    name, version = package['name'], package['version']
    section = (f'\n{"=" * 72}\n{name} {version}\nLicense: {package["license"]}\n'
               f'Authors: {"; ".join(package["authors"])}\n'
               f'Source: https://crates.io/api/v1/crates/{name}/{version}/download\n')
    for label, text in notices:
        section += f'\n--- {label} ---\n{text.rstrip()}\n'
    sections.append(section)
output = root / 'licenses/Rust-dependencies.txt'
output.parent.mkdir(exist_ok=True)
output.write_text('\n'.join(sections))
print(output)
