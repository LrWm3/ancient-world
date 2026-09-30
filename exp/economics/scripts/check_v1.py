#!/usr/bin/env python3
"""Run the frozen v1 gates; keep generated evidence under ignored output/."""
import argparse
import datetime as dt
import json
from pathlib import Path
import re
import shlex
import subprocess
import sys
import time

CRATE = Path(__file__).resolve().parents[1]
ROOT = CRATE.parents[1]
CARGO = ["cargo", "+1.92.0"]


def test(target, name=None, ignored=False):
    command = CARGO + ["test", "--locked", "--test", target]
    if name:
        command.append(name)
    command += ["--", "--nocapture"]
    if name:
        command.append("--exact")
    if ignored:
        command.append("--ignored")
    return command


SCENARIOS = [
    ("S1-offers", test("household_offers")),
    ("S1-shared-payment", test("household_accounting", "household_support_funds_member_dues_in_native_goods_or_coins_once")),
    ("S2-continuing", test("household_income", "voluntary_surplus_closes_the_loop_for_ten_years_on_cpu_with_separate_books")),
    ("S2-no-support", test("household_income", "finite_private_work_target_eventually_stops_collective_income")),
    ("S2-interruption", test("household_income", "coordinated_income_recovers_after_a_temporary_market_demand_loss")),
    ("S2-observer", test("household_income", "support_observer_explains_transfers_without_changing_execution")),
    ("S3-farm-finance", test("household_farm_finance", "repeated_household_harvests_service_mortgage_rent_and_forwards_through_actual_sales")),
    ("S3-buffer", test("household_farm_finance", "fixed_food_buffer_can_pay_financial_claims_while_missing_a_meal")),
    ("S4-mint-finance", test("mint_finance")),
    ("S5-mortgage", test("mortgage_receivables")),
    ("S5-native", test("native_receivables")),
    ("S5-crop-recovery", test("mortgage_recovery", "financed_land_and_attached_crop_use_the_authorized_estate_lifecycle")),
    ("S5-claim-budget", test("estate_receivables", "receivable_and_inventory_lots_compete_for_one_opening_cash_budget")),
    ("S5-exit", test("household_dissolution")),
    ("S5-observer", test("recovery", "recovery_observers_report_actual_guarantees_distributions_and_writeoffs")),
    ("S6-population", test("household_accounting", "specialist_households_reconcile_production_trading_and_annual_dues", ignored=True)),
]
CHECKS = [
    ("full-suite", CARGO + ["test", "--locked"]),
    ("clippy", CARGO + ["clippy", "--locked", "--all-targets", "--", "-D", "warnings"]),
    ("format", CARGO + ["fmt", "--all", "--", "--check"]),
    ("artifacts", ["python3", str(ROOT / "scripts/check_repository_artifacts.py")]),
    ("diff", ["git", "diff", "--check"]),
]


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True).strip()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--list", action="store_true", help="print commands without running")
    parser.add_argument("--scenarios-only", action="store_true", help="diagnostic subset; not full certification")
    args = parser.parse_args()
    steps = SCENARIOS + ([] if args.scenarios_only else CHECKS)
    if args.list:
        for label, command in steps:
            print(f"{label}: {shlex.join(command)}")
        return 0
    revision = git("rev-parse", "HEAD")
    if git("status", "--porcelain", "--untracked-files=normal"):
        parser.error("commit source changes first: release evidence requires a clean checkout")
    stamp = dt.datetime.now(dt.timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")
    output = ROOT / "output/economics" / f"v1-{revision[:8]}-{stamp}"
    output.mkdir(parents=True)
    metadata = {
        "revision": revision, "utc": stamp, "scenarios_only": args.scenarios_only,
        "fixture_manifest": "exp/economics/V1-SCENARIOS.md",
        "toolchain": subprocess.check_output(CARGO + ["--version"], text=True).strip(),
        "steps": steps,
    }
    (output / "run.json").write_text(json.dumps(metadata, indent=2) + "\n")
    report = output / "report.md"
    report.write_text(f"# Economics v1 verification\n\nRevision: `{revision}`.\n\n"
                      f"Scenarios only: {args.scenarios_only}. See V1-SCENARIOS.md for fixed settings.\n\n"
                      "| Gate | Exit | Seconds | Passed / failed / ignored |\n| --- | --- | --- | --- |\n")
    evidence = []
    print(f"Evidence: {output}", flush=True)
    for label, command in steps:
        if git("rev-parse", "HEAD") != revision or git("status", "--porcelain"):
            print("Checkout changed during verification; evidence is invalid.", file=sys.stderr)
            return 1
        start = time.monotonic()
        log = output / f"{label}.log"
        print(f"Starting {label}", flush=True)
        with log.open("w") as stream:
            process = subprocess.Popen(command, cwd=CRATE if label != "artifacts" else ROOT,
                                       stdout=stream, stderr=subprocess.STDOUT)
            next_update = start + 30
            try:
                while process.poll() is None:
                    time.sleep(1)
                    if time.monotonic() >= next_update:
                        print(f"{label}: still running ({time.monotonic() - start:.0f}s)", flush=True)
                        next_update += 30
            except KeyboardInterrupt:
                process.terminate()
                process.wait()
                raise
        seconds = time.monotonic() - start
        text = log.read_text()
        results = re.findall(r"test result: .*? (\d+) passed; (\d+) failed; (\d+) ignored;", text)
        totals = [sum(int(row[i]) for row in results) for i in range(3)]
        code = process.returncode
        if "test" in command and (not results or totals[0] == 0):
            code = code or 1  # A stale --exact name must not silently pass zero tests.
        if git("rev-parse", "HEAD") != revision or git("status", "--porcelain"):
            code = code or 1
        with report.open("a") as stream:
            stream.write(f"| {label} | {code} | {seconds:.2f} | {' / '.join(map(str, totals))} |\n")
        for line in text.splitlines():
            if line.startswith("V1_EVIDENCE "):
                evidence.append(json.loads(line.removeprefix("V1_EVIDENCE ")))
        print(f"{label}: {'PASS' if code == 0 else 'FAIL'} ({seconds:.1f}s), log {log.name}", flush=True)
        if code:
            print(text[-8000:], file=sys.stderr)
            return code
    with report.open("a") as stream:
        stream.write("\n## Committed-outcome observations\n\n"
                     "Resource/definition IDs refer to each fixture; receipt units are not labor-hours. "
                     "Unused capacity includes expired and final remaining capacity. "
                     "These observations supplement the asserted outcomes and controls.\n\n")
        for row in evidence:
            stream.write(f"### {row['case']} ({row['backend']})\n\n```json\n{json.dumps(row, indent=2)}\n```\n\n")
        stream.write("All requested gates passed. " +
                     ("Diagnostic scenario subset only.\n" if args.scenarios_only else "Full v1 candidate verification completed.\n"))
    print(f"PASS: {report}", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
