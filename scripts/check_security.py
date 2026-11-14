"""Run local scanners and retain their evidence; any failed check fails the command."""
import argparse
import json
import os
from pathlib import Path
import subprocess
from datetime import datetime, timezone

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('group', choices=['rust', 'source', 'secrets', 'all'])
parser.add_argument('--output', type=Path, default=root / '.runtime/security')
args = parser.parse_args()
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=True)
cargo = os.environ.get('CARGO', 'cargo')
checks = []


def run(name, command, stdout=None):
    print(f'Checking {name}…', flush=True)
    with (out / (name + '.log')).open('w') as log:
        if stdout:
            with (out / stdout).open('w') as result:
                status = subprocess.run(command, cwd=root, stdout=result, stderr=log).returncode
        else:
            status = subprocess.run(command, cwd=root, stdout=log, stderr=log).returncode
    checks.append({'check': name, 'exit_code': status, 'command': command})
    print(f'{name}: {"passed" if status == 0 else "FAILED"} ({out / (name + ".log")})', flush=True)


if args.group in ('rust', 'all'):
    for label, manifest, lock in [('core', 'Cargo.toml', 'Cargo.lock'),
                                  ('desktop', 'desktop/Cargo.toml', 'desktop/Cargo.lock')]:
        run('audit-' + label, [cargo, 'audit', '--file', lock, '--json'], 'audit-' + label + '.json')
        run('deny-' + label, [cargo, 'deny', '--locked', '--manifest-path', manifest,
                             '--config', 'deny.toml', 'check'])
if args.group in ('source', 'all'):
    run('semgrep', ['semgrep', 'scan', '--config', 'p/security-audit', '--config', 'p/trailofbits',
                    '--metrics', 'off', '--disable-version-check', '--strict', '--error',
                    '--exclude', 'ui/graphviz.js', '--json', '--output', str(out / 'semgrep.json'),
                    '--sarif-output', str(out / 'semgrep.sarif'),
                    'src', 'desktop/src', 'ui', 'scripts', 'vendor/graphviz-port'])
if args.group in ('secrets', 'all'):
    run('gitleaks', ['gitleaks', 'git', '--redact', '--log-opts=--all', '--report-format', 'json',
                     '--report-path', str(out / 'gitleaks.json'), '.'])

summary = {'timestamp': datetime.now(timezone.utc).isoformat(),
           'commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip(),
           'group': args.group, 'checks': checks, 'passed': all(c['exit_code'] == 0 for c in checks)}
(out / ('summary-' + args.group + '.json')).write_text(json.dumps(summary, indent=2) + '\n')
raise SystemExit(0 if summary['passed'] else 1)
