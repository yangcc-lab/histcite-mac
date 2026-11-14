"""Pointer/keyboard tests with real Rust data and explicitly mocked desktop IPC."""
from pathlib import Path
from http.server import ThreadingHTTPServer, SimpleHTTPRequestHandler
from functools import partial
from threading import Thread
import argparse
import json
import os
import shutil
import subprocess
from playwright.sync_api import sync_playwright, expect

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--fixture', type=Path, help='Optional precomputed Rust analysis JSON')
parser.add_argument('--output', type=Path, default=root / '.runtime/interaction-checks')
args = parser.parse_args()
out = args.output
out.mkdir(parents=True, exist_ok=True)
if args.fixture:
    fixture = args.fixture
else:
    fixture = out / 'demo-analysis.json'
    subprocess.run([os.environ.get('CARGO', 'cargo'), 'run', '--locked', '--quiet', '--',
                    '--output', str(fixture), str(root / 'examples/demo-wos.txt')], cwd=root, check=True)
data = json.loads(fixture.read_text())
data['collection'] = {'title': 'Interaction checks', 'marks': [], 'tags': {}, 'graphs': [], 'revision': 0, 'undo_count': 0}
data['settings'] = {'view': 'standard', 'record_page_size': 100, 'index_page_size': 200, 'author_limit': 5, 'save_graph_settings': False}
payload = json.dumps(data)

class Quiet(SimpleHTTPRequestHandler):
    def log_message(self, *_): pass

server = ThreadingHTTPServer(('127.0.0.1', 0), partial(Quiet, directory=str(root / 'ui')))
Thread(target=server.serve_forever, daemon=True).start()
checks = []
with sync_playwright() as pw:
    chrome = os.environ.get('CHROME') or shutil.which('google-chrome')
    if not chrome and Path('/Applications/Google Chrome.app/Contents/MacOS/Google Chrome').exists():
        chrome = '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome'
    browser = pw.chromium.launch(**({'executable_path': chrome} if chrome else {}))
    context = browser.new_context(viewport={'width': 1360, 'height': 880})
    context.route('**/fixture.json', lambda route: route.fulfill(body=payload, content_type='application/json'))
    context.add_init_script('''window.calls=[];window.__TAURI__={core:{invoke:async(name,args={})=>{
      calls.push({name,args});window.fixture??=await(await fetch('/fixture.json')).json();
      if(name==='mutate_collection'){if(args.operation.action!=='settings')throw Error('Unexpected mutation');fixture.settings=args.operation.settings;}
      if(name==='current_revision')return fixture.collection.revision;
      if(['initial_analysis','current_analysis','mutate_collection'].includes(name))return structuredClone(fixture);
      if(['open_record','open_graph','save_export','print_window'].includes(name))return null;
      throw Error('Unexpected IPC '+name);
    }}};''')
    page = context.new_page()
    errors = []
    page.on('pageerror', lambda e: errors.append(str(e)))
    url = f'http://127.0.0.1:{server.server_port}'
    page.goto(url)
    page.wait_for_selector('#rows .paper')
    menus = page.locator('.menu')
    file = menus.nth(0)
    tools = menus.nth(3)
    file.locator('summary').hover()
    expect(file).to_have_attribute('open', '')
    # A subsequent click must not toggle closed just because hover opened it.
    file.locator('summary').click()
    expect(page.locator('#import')).to_be_visible()
    page.mouse.move(1100, 700)
    page.wait_for_timeout(180)
    expect(page.locator('#import')).to_be_visible()
    page.locator('#import').hover()
    page.wait_for_timeout(600)
    expect(page.locator('#import')).to_be_visible()
    tools.locator('summary').hover()
    expect(page.locator('#import')).to_be_hidden()
    expect(page.locator('#graph-open')).to_be_visible()
    page.mouse.move(1100, 700)
    expect(page.locator('#graph-open')).to_be_hidden(timeout=1500)
    checks.append('hover opens; click stays open; 500 ms exit grace; re-entry cancels close; sibling switching')
    tools.locator('summary').hover()
    page.locator('#view-title').click()
    expect(page.locator('#graph-open')).to_be_hidden()
    file.locator('summary').focus()
    page.keyboard.press('ArrowDown')
    expect(page.locator('#import')).to_be_focused()
    page.keyboard.press('ArrowRight')
    expect(page.locator('#analysis-menu button').first).to_be_focused()
    page.keyboard.press('End')
    expect(page.locator('#analysis-menu button').last).to_be_focused()
    page.keyboard.press('Home')
    expect(page.locator('#analysis-menu button').first).to_be_focused()
    page.keyboard.press('Escape')
    expect(menus.nth(1).locator('summary')).to_be_focused()
    expect(page.locator('#analysis-menu button').first).to_be_hidden()
    tools.locator('summary').focus()
    page.keyboard.press('Enter')
    page.keyboard.press('ArrowDown')  # Skip unavailable Historiographs.
    expect(page.locator('#search-open')).to_be_focused()
    page.keyboard.press('Enter')
    expect(page.locator('#search-author')).to_be_focused()
    expect(page.locator('#graph-open')).to_be_hidden()
    checks.append('keyboard arrows/Home/End/Enter; disabled items skipped; Escape returns focus; action retains form focus')
    tools.locator('summary').hover()
    expect(page.locator('[data-action=undo]')).to_be_disabled()
    expect(page.locator('[data-action=edit]')).to_be_disabled()
    assert 'no changes' in page.locator('[data-action=undo]').get_attribute('title')
    page.screenshot(path=str(out / 'menu-hover.png'))
    page.keyboard.press('Escape')
    for label in ['LCS', 'GCS', 'LCR', 'CR']:
        heading = page.locator('#record-headings').get_by_role('button', name=label, exact=True)
        assert len(heading.get_attribute('title')) > 20
        assert heading.get_attribute('aria-description') == heading.get_attribute('title')
    assert 'records, TLCS' in page.locator('.author-link').first.get_attribute('title')
    assert len(page.locator('.record-link').first.get_attribute('title')) > 40
    menus.nth(2).locator('summary').hover()
    page.locator('[data-view-mode=bibliometric]').click()
    expect(page.locator('#record-headings th')).to_have_count(14)
    assert all(page.locator('#record-headings button').evaluate_all('(es)=>es.map(e=>!!e.title)'))
    page.locator('#analysis-index [data-view=authors]').click()
    assert all(page.locator('#group-headings button').evaluate_all('(es)=>es.map(e=>!!e.title)'))
    assert 'Total across' in page.locator('#group-headings').get_by_role('button', name='TLCS', exact=True).get_attribute('title')
    checks.append('all record metrics, analytical totals, authors and record links have explanatory hints')
    detail = context.new_page()
    detail.on('pageerror', lambda e: errors.append(str(e)))
    detail.goto(url + '/detail.html?index=0')
    detail.wait_for_selector('.score-row span')
    assert all(detail.locator('.score-row span').evaluate_all('(es)=>es.map(e=>!!e.title)'))
    checks.append('independent record detail has all 12 metric hints')
    graph = context.new_page()
    graph.on('pageerror', lambda e: errors.append(str(e)))
    graph.goto(url + '/?window=graph')
    graph.wait_for_selector('.graph-node ellipse')
    for shape, selector, prop in [('circle', 'ellipse', 'stroke'), ('box', 'polygon', 'stroke'), ('none', 'text', 'fill')]:
        if shape != 'circle':
            graph.locator('#node-shape').select_option(shape)
            graph.get_by_role('button', name='Make graph', exact=True).click()
        graph.wait_for_selector('.graph-node ' + selector)
        node = graph.locator('.graph-node').first
        visible = node.locator(selector).first
        graph.mouse.move(0, 0)
        visible.hover()
        expect(visible).to_have_css(prop, 'rgb(0, 0, 238)')
        assert len(node.locator('title').text_content()) > 40
        graph.screenshot(path=str(out / f'graph-hover-{shape}.png'))
        graph.evaluate("Graph.exportGraph('svg')")
        svg=graph.evaluate("calls.filter(c=>c.name==='save_export').at(-1).args.content")
        assert 'stroke:#00e' not in svg and 'stroke: rgb(0, 0, 238)' not in svg
        graph.mouse.move(0, 0)
        expect(visible).to_have_css(prop, 'rgb(0, 0, 0)')
        # Reach the node by keyboard; focus must remain visibly indicated.
        node.focus()
        graph.keyboard.press('Tab')
        graph.keyboard.press('Shift+Tab')
        expect(node).to_be_focused()
        expect(visible).to_have_css(prop, 'rgb(0, 0, 238)')
        before = graph.evaluate("calls.filter(c=>c.name==='open_record').length")
        scroll = graph.locator('#graph-pane').evaluate('(e)=>e.scrollTop')
        graph.keyboard.press('Space')
        assert graph.evaluate("calls.filter(c=>c.name==='open_record').length") == before + 1
        assert graph.locator('#graph-pane').evaluate('(e)=>e.scrollTop') == scroll
        graph.keyboard.press('Enter')
        assert graph.evaluate("calls.filter(c=>c.name==='open_record').length") == before + 2
        node.click()
        assert graph.evaluate("calls.filter(c=>c.name==='open_record').length") == before + 3
        graph.get_by_role('button', name='Make graph', exact=True).focus()
    checks.append('circle/square/text-only: hover and leave, keyboard focus, tooltip, click, Enter and Space without scrolling')
    # Deliberately drop notifications: focus must discover the changed revision.
    page.evaluate("fixture.collection.title='Recovered on focus';fixture.collection.revision++;window.dispatchEvent(new Event('focus'))")
    expect(page.locator('#collection-title')).to_have_text('Recovered on focus')
    before = page.evaluate("calls.filter(c=>c.name==='current_analysis').length")
    page.evaluate("window.dispatchEvent(new Event('focus'))")
    page.wait_for_timeout(150)
    assert page.evaluate("calls.filter(c=>c.name==='current_analysis').length") == before
    detail = context.new_page()
    detail.on('pageerror', lambda e: errors.append(str(e)))
    detail.goto(url + '/detail.html?index=0')
    detail.wait_for_selector('.detail-title')
    detail.evaluate("fixture.records[0].title='Detail synchronized';fixture.collection.revision++;window.dispatchEvent(new Event('focus'))")
    expect(detail.locator('.detail-title')).to_have_text('Detail synchronized')
    # The graph request arrives before the collection containing it is refreshed.
    graph.evaluate("""fixture.collection.graphs=[{id:'late-graph',records:fixture.records.slice(0,2).map(r=>r.id),settings:{layoutVersion:4,shape:'circle'}}];
      fixture.collection.revision++;localStorage.setItem('histcite-load-graph','late-graph');
      window.dispatchEvent(new Event('focus'));""")
    expect(graph.locator('#graph-selection')).to_have_value('saved')
    expect(graph.locator('#graph-canvas .graph-node')).to_have_count(2)
    assert graph.evaluate("localStorage.getItem('histcite-load-graph')") is None
    checks.append('lost notification: focus repairs main/detail/graph; unchanged revision skips reload; pending saved graph survives stale data')
    engine = graph.evaluate("""async()=>{
      for(const formats of [[],null,['png'],['svg:other'],['svg','ps','svg']]){
        let rejected=false;try{await GraphLayout.layout('digraph { a -> b }',formats);}catch(e){rejected=String(e).includes('Unsupported graph output format');}
        if(!rejected)throw Error('Unexpected renderer accepted');
      }
      const out=await GraphLayout.layout('digraph { a -> b }',['svg','ps']);
      return out.svg.includes('<svg')&&out.ps.startsWith('%!PS');
    }""")
    assert engine
    checks.append('graph worker rejects unsupported renderers and renders SVG/PostScript')
    # Touch must still activate the menus without a preliminary hover.
    touch = browser.new_context(has_touch=True, viewport={'width': 1100, 'height': 800})
    touch.route('**/fixture.json', lambda r: r.fulfill(body=payload, content_type='application/json'))
    touch.add_init_script("window.__TAURI__={core:{invoke:async()=>{window.d??=await(await fetch('/fixture.json')).json();return d;}}}")
    tp = touch.new_page()
    tp.goto(url)
    tp.wait_for_selector('#rows .paper')
    tp.locator('.menu summary').first.tap()
    expect(tp.locator('#import')).to_be_visible()
    tp.touchscreen.tap(1000, 700)
    expect(tp.locator('#import')).to_be_hidden()
    checks.append('touch opens menu; outside tap dismisses')
    empty=context.new_page()
    empty.add_init_script("window.__TAURI__={core:{invoke:async()=>null}};")
    empty.goto(url)
    empty.wait_for_selector('#empty-import')
    empty.locator('.menu summary').first.hover()
    expect(empty.locator('#import')).to_be_enabled()
    expect(empty.locator('[data-action=save]')).to_be_disabled()
    expect(empty.locator('[data-action=new-record]')).to_be_enabled()
    checks.append('empty collection keeps Open/New enabled and disables data-dependent commands')
    assert not errors, errors
    (out / 'checks.json').write_text(json.dumps({'passed': True, 'transport': 'Chrome with real Rust fixture and explicit IPC mock', 'checks': checks, 'javascript_errors': errors}, indent=2) + '\n')
    browser.close()
server.shutdown()
print('Interaction checks passed: ' + str(len(checks)), flush=True)
