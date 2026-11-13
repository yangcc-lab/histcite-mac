"""Test the portable demo presentation and capture README images (no IPC mock)."""
from pathlib import Path
import json
import os
import shutil
import subprocess
from playwright.sync_api import sync_playwright

root = Path(__file__).resolve().parents[1]
out = root / '.runtime/demo-presentation'
out.mkdir(parents=True, exist_ok=True)
fixture = out / 'analysis.json'
subprocess.run([os.environ.get('CARGO', 'cargo'), 'run', '--locked', '--quiet', '--',
                '--output', str(fixture), str(root / 'examples/demo-wos.txt')], cwd=root, check=True)
data = json.loads(fixture.read_text())
assert len(data['records']) == 12 and len(data['edges']) == 31
data['collection'] = {'title': 'Literature mapping — demonstration', 'marks': [], 'tags': {},
                      'graphs': [], 'revision': 0, 'undo_count': 0, 'log': []}
data['settings'] = {'view': 'standard', 'record_page_size': 100, 'index_page_size': 200,
                    'author_limit': 5, 'font_size': 10, 'columns': ['lcs', 'gcs', 'lcr', 'cr'],
                    'record_header_interval': 10, 'html_limits': False, 'save_graph_settings': False}
for name in ['index.html', 'detail.html', 'style.css', 'common.js', 'app.js', 'detail.js',
             'graph.js', 'graph-layout.js', 'graphviz.js', 'THIRD-PARTY.txt', 'Graphviz-source.zip']:
    if name.endswith('.html'):
        text = (root / 'ui' / name).read_text().replace(
            '<script src="common.js">', '<script src="data.js"></script><script src="common.js">')
        (out / name).write_text(text)
    else:
        shutil.copy2(root / 'ui' / name, out / name)
(out / 'data.js').write_text('window.__HISTCITE_EXPORT__=' + json.dumps(data) + ';')
images = root / 'docs/images'
images.mkdir(parents=True, exist_ok=True)
with sync_playwright() as pw:
    chrome = os.environ.get('CHROME') or shutil.which('google-chrome')
    if not chrome and Path('/Applications/Google Chrome.app/Contents/MacOS/Google Chrome').exists():
        chrome = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome'
    browser = pw.chromium.launch(**({'executable_path': chrome} if chrome else {}))
    context = browser.new_context(viewport={'width': 1240, 'height': 820}, accept_downloads=True)
    errors = []
    context.on('page', lambda p: p.on('pageerror', lambda e: errors.append(str(e))))
    page = context.new_page()
    page.goto((out / 'index.html').as_uri())
    page.wait_for_selector('#rows .paper')
    assert page.locator('#record-count').inner_text() == '12'
    assert page.locator('#edge-count').inner_text() == '31'
    page.screenshot(path=str(images / 'records.png'))
    for kind in ['authors', 'journals', 'references', 'words', 'years', 'types', 'languages',
                 'institutions', 'subdivisions', 'countries']:
        page.locator(f'#analysis-index [data-view={kind}]').click()
        assert page.locator('#group-rows tr').count() > 0, kind
    page.locator('#analysis-index [data-view=records]').click()
    with page.expect_popup() as popup:
        page.locator('#rows .record-link').first.click()
    detail = popup.value
    detail.wait_for_selector('.detail-title')
    assert 'Starting a literature map' in detail.locator('.detail-title').inner_text()
    detail.close()
    page.locator('.menu summary').filter(has_text='Tools').click()
    with page.expect_popup() as popup:
        page.locator('#graph-open').click()
    graph = popup.value
    graph.wait_for_selector('.graph-node')
    assert graph.locator('.graph-node').count() == 12
    graph.set_viewport_size({'width': 1240, 'height': 820})
    graph.screenshot(path=str(images / 'graph.png'))
    with graph.expect_popup() as popup:
        graph.locator('.graph-node').first.click()
    popup.value.wait_for_selector('.detail-title')
    popup.value.close()
    for label, filename in [('SVG', 'demo.svg'), ('PNG', 'demo.png')]:
        with graph.expect_download() as download:
            graph.get_by_role('button', name=label, exact=True).click()
        download.value.save_as(str(out / filename))
    assert '<svg' in (out / 'demo.svg').read_text()
    assert (out / 'demo.png').read_bytes().startswith(b'\x89PNG\r\n\x1a\n')
    assert not errors, errors
    browser.close()
(out / 'checks.json').write_text(json.dumps({
    'passed': True, 'transport': 'file://, actual Rust demo analysis; no Tauri or IPC mock',
    'records': 12, 'citation_links': 31,
    'checks': ['ten analytical indexes', 'independent record window', 'independent graph window',
               'graph node opens record', 'SVG download', 'PNG download'],
    'javascript_errors': errors,
}, indent=2) + '\n')
print('Demo presentation passed; README images captured from synthetic data.')
