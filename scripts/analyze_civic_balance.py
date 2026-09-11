#!/usr/bin/env python3
"""Summarize local civic_balance outputs; no outcome quotas or causal claims."""
import argparse
import json
from collections import Counter

parser = argparse.ArgumentParser()
parser.add_argument("report")
args = parser.parse_args()
with open(args.report) as handle:
    report = json.load(handle)
print("| Seed | Enabled | Requests (relief/learning/autonomy) | Honored | Paid | Population | Active towns | Peak grants / administration pay | Max residual |")
print("|---|---|---|---:|---:|---:|---:|---:|---:|")
for run in report["runs"]:
    requests = run["petitions"]
    counts = Counter(p["demand"] for p in requests)
    final = run["samples"][-1]
    ratio = max(0., max(s["paid"] / max(s["administrative_pay"], 1) for s in run["samples"]))
    print(f'| {run["seed"]} | {run["enabled"]} | {counts["Relief"]}/{counts["Learning"]}/{counts["Autonomy"]} | '
          f'{final["honored"]}/{len(requests)} | {max(0., final["paid"]):.1f} | {final["population"]:.0f} | '
          f'{final["active_sites"]} | {ratio:.3%} | {run["max_residual"]:.2e} |')
    resolved = [p for p in requests if p["resolved"] is not None]
    if resolved:
        waits = [p["resolved"] - p["opened"] for p in resolved]
        assert all(3 <= w <= 12 for w in waits), "unexpected petition duration"
    assert final["paid"] >= 0 and all(0 <= p["paid"] <= p["requested"] for p in requests)
print("\nMatched final population changes (downstream associations, not isolated mediators):")
for seed in sorted({r["seed"] for r in report["runs"]}):
    pair = {r["enabled"]: r for r in report["runs"] if r["seed"] == seed}
    if len(pair) == 2:
        base = pair[False]["samples"][-1]["population"]
        treated = pair[True]["samples"][-1]["population"]
        print(f"- {seed}: {treated-base:+.1f} people ({(treated/base-1)*100:+.2f}%)")

print("\nRecorded outcomes:")
for run in report["runs"]:
    if run["enabled"]:
        print(run["seed"], dict(Counter(p.get("resolution_reason") or "unrecorded/pending" for p in run["petitions"])))
