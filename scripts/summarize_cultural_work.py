#!/usr/bin/env python3
"""Summarize completed cultural work runs; rates are game diagnostics, not empirical targets."""
import argparse
import json
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('reports', nargs='+', type=Path)
args = parser.parse_args()
print('| Run | Seed | Years | Cancelled funded bundles | Cancelled grant | Work used / granted | Population | Institutions |')
print('|---|---:|---:|---:|---:|---:|---:|---:|')
for path in args.reports:
    report = json.loads(path.read_text())
    if not report['complete']:
        raise SystemExit(f'{path}: incomplete; refusing to summarize as completed')
    for run in report['runs']:
        s = run['samples'][-1]
        def percent(n, d):
            return f'{100*n/d:.2f}%' if d else 'n/a'
        print(f"| {path.stem} | {run['seed']} | {s['year']} | "
              f"{s['cancelled_bundles']}/{s['funded_bundles']} ({percent(s['cancelled_bundles'],s['funded_bundles'])}) | "
              f"{percent(s['cancelled_work'],s['granted'])} | {percent(s['used'],s['granted'])} | "
              f"{s['population']:.0f} | {s['institutions']} |")
