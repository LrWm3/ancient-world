#!/usr/bin/env python3
"""Run matched credit/issuance arms; retain raw artifacts only under output/."""
import argparse
from collections import Counter
import hashlib
import json
import math
from pathlib import Path
import shutil
import subprocess
import time

BINDING_RATIO_TOLERANCE = 1e-12
DEFAULT_HISTORY_YEARS = 200

ARMS = (
    ("baseline", False, False),
    ("credit", True, False),
    ("issuance", False, True),
    ("combined", True, True),
)


def procurement_share(value):
    share = float(value)
    if not math.isfinite(share) or not 0 <= share <= 1:
        raise argparse.ArgumentTypeError("procurement share must be finite and within 0–1")
    return share


def comparison_arms(export_recovery=False, institution_lenders=False, institution_reserves=False):
    arms = [(name, credit, issuance, False) for name, credit, issuance in ARMS]
    if export_recovery:
        arms.extend((name + "-recovery", credit, issuance, True)
                    for name, credit, issuance in ARMS if credit)
    if institution_lenders or institution_reserves:
        arms.extend((name + "-institution-lenders", credit, issuance, False)
                    for name, credit, issuance in ARMS if credit)
    if institution_reserves:
        arms.extend((name + "-institution-operating-reserve", credit, issuance, False)
                    for name, credit, issuance in ARMS if credit)
    return arms


def institution_arm_settings(name):
    operating = name.endswith("-institution-operating-reserve")
    return name.endswith("-institution-lenders") or operating, operating


def digest(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def activity_and_access(history):
    """Terminal access and cumulative activity; neither is a causal estimate."""
    accounts = history["society"]["household_economy"]["accounts"]
    councils = history["society"]["councils"]
    total_need = sum(a["need"] for a in accounts)
    return {
        "operator_completed_work": sum(f["completed_work"] for f in history["enterprises"]["firms"]),
        "managed_crop_harvest_kg": sum(c[3] for s in history["sites"] for c in s["economy"]["crops"]),
        "reported_food_production": sum(s["stocks"]["ledger"][0] for s in history["sites"]),
        "terminal_food_need": total_need,
        "terminal_need_weighted_hunger": (
            sum(a["hunger"] * a["need"] for a in accounts) / total_need if total_need > 0 else None
        ),
        "council_town_support": sum(c["relief_paid"] for c in councils),
        "ending_council_cash": sum(c["treasury"] for c in councils),
        "ending_household_cash": sum(a["cash"] for a in accounts),
        "ending_town_cash": sum(s["economy"]["finance"][0] for s in history["sites"]),
    }



def institution_outcomes(history):
    """Endpoint accounts and recorded cumulative capacity work, not closure counts."""
    culture = history.get("culture")
    if culture is None:
        return {"institution_records_available": False}
    institutions = culture["institutions"]
    capacities = [n["capacity"] for n in institutions if n.get("capacity") is not None]
    buildings = [c["building"] for c in capacities if c.get("building") is not None]
    return {
        "institution_records_available": True,
        "institution_count": len(institutions),
        "active_institutions": sum(n["active"] for n in institutions),
        "ending_institution_cash": sum(n["treasury"] for n in institutions),
        "ending_inactive_institution_cash": sum(n["treasury"] for n in institutions if not n["active"]),
        "institution_expenses": sum(n["expenses"] for n in institutions),
        "institution_capacity_records": len(capacities),
        "recorded_institution_upkeep_paid": sum(c["paid"] for c in capacities),
        "recorded_institution_upkeep_work": sum(c["work"] for c in capacities),
        "recorded_institution_repair_paid": sum(b["repair_paid"] for b in buildings),
    }


def service_order_outcomes(history):
    """Durable cumulative payments, plus explicitly terminal procurement claims."""
    enterprises = history.get("enterprises") or {}
    orders = enterprises.get("orders", [])
    claims = (enterprises.get("procurement") or {}).get("claims", [])
    observed = [o["execution"] for o in orders if o.get("execution") is not None]
    return {
        "service_orders": len(orders),
        "service_orders_observed": len(observed),
        "service_orders_no_requested_labor": sum(o["requested_labor"] == 0 for o in observed),
        "service_orders_no_funded_labor": sum(o["funded_labor"] == 0 for o in observed),
        "service_orders_funded_without_completion": sum(
            o["funded_labor"] > 0 and o["completed_labor"] == 0 for o in observed
        ),
        "service_orders_pending": sum(o["settled"] is None for o in orders),
        "service_order_funding": sum(o["funded"] for o in orders),
        "service_order_paid": sum(o["paid"] for o in orders),
        "service_order_refunded": sum(o["refunded"] for o in orders),
        "service_order_escrow": sum(o["escrow"] for o in orders),
        "terminal_procurement_requested": sum(c["requested_fee"] for c in claims),
        "terminal_procurement_funded": sum(c["funded"] for c in claims),
    }


def credit_funnel(history):
    """Recorded requests, numerical grants and actual principal are different stocks.

    Empty rounds do not prove there were no opportunities: the pilots omit months
    with no constructed requests. Explicit API loans need not belong to a round.
    """
    credit = history["credit"]
    if "rounds" not in credit:
        return {"credit_request_records_available": False}
    rounds = credit["rounds"]
    loans = {loan["id"]: loan for loan in credit["loans"]}
    if len(loans) != len(credit["loans"]):
        raise ValueError("duplicate loan identity")
    decisions, sources, binding = Counter(), Counter(), Counter()
    requested = eligible = granted = funded = 0.0
    linked = set()
    request_count = 0
    missing_capacity_records = 0
    for round_ in rounds:
        requests = {r["id"]: r for r in round_["requests"]}
        grants = round_["grants"]
        if (len(requests) != len(round_["requests"])
                or len(grants) != len(round_["loan_ids"])
                or len({g["request"] for g in grants}) != len(grants)
                or set(requests) != {g["request"] for g in grants}):
            raise ValueError("inconsistent credit round references")
        for grant, loan_id in zip(grants, round_["loan_ids"]):
            request = requests[grant["request"]]
            amounts = [request["principal"], grant["eligible"], grant["granted"]]
            if any(not math.isfinite(v) or v < 0 for v in amounts):
                raise ValueError("invalid request amounts")
            if not amounts[2] <= amounts[1] <= amounts[0]:
                raise ValueError("credit grant exceeds request")
            requested += amounts[0]
            eligible += amounts[1]
            granted += amounts[2]
            request_count += 1
            decisions[grant["decision"]] += 1
            source = request["terms"]["source"]
            if not isinstance(source, dict) or len(source) != 1:
                raise ValueError("invalid repayment source")
            sources[next(iter(source))] += 1
            capacity = grant.get("capacity")
            if capacity is None and amounts[1] > 0:
                missing_capacity_records += 1
            if capacity:
                ratios = {}
                for pool, stock, demand in (
                    ("lender", "lender_principal", "lender_demand"),
                    ("borrower", "borrower_principal", "borrower_demand"),
                    ("source", "source_receipts", "source_demand"),
                ):
                    if (not math.isfinite(capacity[stock]) or capacity[stock] < 0
                            or not math.isfinite(capacity[demand]) or capacity[demand] <= 0):
                        raise ValueError("invalid capacity receipt")
                    ratios[pool] = capacity[stock] / capacity[demand]
                scale = min(1.0, *ratios.values())
                if scale < 1:
                    for pool, ratio in ratios.items():
                        if math.isclose(ratio, scale, rel_tol=BINDING_RATIO_TOLERANCE,
                                        abs_tol=BINDING_RATIO_TOLERANCE):
                            binding[pool] += 1
            if loan_id is not None:
                if loan_id not in loans or loan_id in linked:
                    raise ValueError("missing or multiply funded round loan")
                principal = loans[loan_id]["original_principal"]
                if not math.isfinite(principal) or not 0 < principal <= amounts[2]:
                    raise ValueError("invalid committed principal")
                funded += principal
                linked.add(loan_id)
    return {
        "credit_request_records_available": True,
        "credit_rounds": len(rounds),
        "credit_incomplete_rounds": sum(not r["complete"] for r in rounds),
        "credit_recorded_requests": request_count,
        "credit_request_sources": dict(sorted(sources.items())),
        "credit_decisions": dict(sorted(decisions.items())),
        "credit_binding_capacity_counts": dict(sorted(binding.items())),
        "credit_eligible_requests_without_capacity_records": missing_capacity_records,
        "credit_requested_principal": requested,
        "credit_eligible_principal": eligible,
        "credit_granted_principal": granted,
        "credit_committed_round_principal": funded,
        "credit_loans_outside_rounds": len(loans) - len(linked),
    }


def council_construction(history):
    """Council-month decisions since diagnostics began, not counts of loans."""
    credit = history["credit"]
    counts = credit.get("council_review_counts")
    if counts is None:
        return {"council_construction_records_available": False}
    if any(type(n) is not int or n < 0 for n in counts.values()):
        raise ValueError("invalid council review count")
    return {
        "council_construction_records_available": True,
        "council_months_reviewed": sum(counts.values()),
        "council_construction_outcomes": dict(sorted(counts.items())),
        "council_latest_review_month": max(
            (r["month"] for r in credit.get("council_reviews", [])), default=None),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=Path("target/debug/ancient-world"))
    parser.add_argument("--checkpoint", action="append", required=True, metavar="LABEL=PATH")
    parser.add_argument("--years", type=int, default=DEFAULT_HISTORY_YEARS)
    parser.add_argument("--output", type=Path, default=Path("output/monetary-experiment"))
    parser.add_argument("--compare-institution-reserves", action="store_true",
                        help="compare council-floor and annual-operating institutional reserves")
    parser.add_argument("--compare-institution-lenders", action="store_true",
                        help="add credit/combined arms with local institutional council lenders")
    parser.add_argument("--compare-export-recovery", action="store_true",
                        help="add credit and combined arms with late-export recovery enabled")
    parser.add_argument("--service-order-procurement", action="store_true",
                        help="hold funded procurement on in every arm; add service lending in credit arms")
    parser.add_argument("--service-procurement-share", type=procurement_share,
                        help="same surplus budget fraction (0–1) in every arm; requires procurement")
    parser.add_argument("--crop-yield-scale", type=float,
                        help="same explicit crop-yield intervention in every arm (0.1–1)")
    args = parser.parse_args()
    if args.service_procurement_share is not None and not args.service_order_procurement:
        parser.error("service procurement share requires --service-order-procurement")
    repo = Path(__file__).resolve().parents[1]
    if args.years <= 0:
        parser.error("years must be positive")
    if args.crop_yield_scale is not None and not 0.1 <= args.crop_yield_scale <= 1.0:
        parser.error("crop yield scale must be finite and within 0.1–1")
    out = (repo / args.output).resolve()
    if not out.is_relative_to((repo / "output").resolve()):
        parser.error("experiment artifacts must be written under repository output/")
    cases = {}
    for entry in args.checkpoint:
        label, separator, name = entry.partition("=")
        if not separator or not label or not all(c.isalnum() or c in "-_" for c in label):
            parser.error("each checkpoint must use a simple LABEL=PATH")
        if label in cases:
            parser.error("checkpoint labels must be unique")
        path = (repo / name).resolve()
        if not path.is_file():
            parser.error(f"missing checkpoint: {path}")
        cases[label] = path
    binary = (repo / args.binary).resolve()
    if not binary.is_file():
        parser.error(f"missing executable: {binary}")
    # Never overwrite an earlier run or replace its executable during compilation.
    out.mkdir(parents=True, exist_ok=False)
    fixed = out / "ancient-world"
    shutil.copy2(binary, fixed)
    metadata = {
        "revision": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip(),
        "tracked_changes": subprocess.check_output(["git", "diff", "HEAD", "--stat"], cwd=repo, text=True),
        "binary_sha256": digest(fixed),
        "checkpoints": {label: {"path": str(path), "sha256": digest(path)} for label, path in cases.items()},
        "years": args.years,
        "crop_yield_scale_override": args.crop_yield_scale,
        "common_payment_policy": "delivery",
        "service_order_procurement": args.service_order_procurement,
        "service_procurement_share_override": args.service_procurement_share,
        "compare_export_recovery": args.compare_export_recovery,
        "compare_institution_lenders": args.compare_institution_lenders,
        "compare_institution_reserves": args.compare_institution_reserves,
    }
    (out / "metadata.json").write_text(json.dumps(metadata, indent=2) + "\n")
    arms = comparison_arms(args.compare_export_recovery, args.compare_institution_lenders,
                           args.compare_institution_reserves)
    results = []
    for label, checkpoint in cases.items():
        for arm, credit, issuance, recovery in arms:
            name = f"{label}-{arm}"
            archive = out / f"{name}.json"
            command = [str(fixed), "--headless", "--load", str(checkpoint), "--epochs", "0",
                       "--history-years", str(args.years), "--delivery-paid-exports",
                       f"--commercial-credit={str(credit).lower()}",
                       f"--service-order-procurement={str(args.service_order_procurement).lower()}",
                       f"--service-order-credit={str(credit and args.service_order_procurement).lower()}",
                       f"--council-credit={str(credit).lower()}",
                       f"--shared-issuance={str(issuance).lower()}",
                       "--history-export", str(archive)]
            if args.compare_institution_lenders or args.compare_institution_reserves:
                enabled, operating = institution_arm_settings(name)
                command.append(f"--institution-credit-lenders={str(enabled).lower()}")
                if args.compare_institution_reserves:
                    command.append(f"--institution-credit-operating-reserve={str(operating).lower()}")
            if args.service_procurement_share is not None:
                command.extend(("--service-procurement-share", str(args.service_procurement_share)))
            if args.crop_yield_scale is not None:
                command.extend(("--crop-yield-scale", str(args.crop_yield_scale)))
            if args.compare_export_recovery:
                command.append(f"--export-default-recovery={str(recovery).lower()}")
            started = time.monotonic()
            with (out / f"{name}.log").open("w") as log:
                status = subprocess.run(command, cwd=repo, stdout=log, stderr=subprocess.STDOUT, check=False)
            result = {"run": name, "seconds": time.monotonic() - started, "exit_code": status.returncode}
            if status.returncode == 0:
                history = json.loads(archive.read_text())
                result.update(activity_and_access(history))
                result.update(service_order_outcomes(history))
                result.update(institution_outcomes(history))
                result.update(credit_funnel(history))
                result.update(council_construction(history))
                loans = history["credit"]["loans"]
                result["loans_by_original_lender_kind"] = dict(sorted(Counter(
                    next(iter(l["terms"]["lender"])) for l in loans).items()))
                result.update(population=sum(s["stocks"]["stock"][0] for s in history["sites"]),
                              loans=len(loans), defaults=sum(l["status"] == "Defaulted" for l in loans),
                              precision_blocked=sum(r.get("precision_blocked", False)
                                                    for r in history["credit"]["service_receipts"]),
                              default_loss=sum(e["principal"] + e["interest"] for l in loans
                                               for e in l["entries"] if e["kind"] == "WriteOff"),
                              outstanding_debt=sum(l["outstanding_principal"] + l["interest_due"] for l in loans),
                              precision_settled=sum(l["status"] == "PrecisionSettled" for l in loans),
                              precision_writeoff=sum(e["principal"] + e["interest"] for l in loans
                                                     for e in l["entries"] if e["kind"] == "PrecisionWriteOff"),
                              recovered_principal=sum(r["transfer"]["principal"]
                                  for r in history["credit"].get("recoveries", [])),
                              recovered_interest=sum(r["transfer"]["interest"]
                                  for r in history["credit"].get("recoveries", [])),
                              issued=sum(r["issued"] for r in history["credit"]["issuance"]["receipts"]),
                              history_sha256=digest(archive))
            results.append(result)
            (out / "results.json").write_text(json.dumps(results, indent=2) + "\n")
            print(json.dumps(result), flush=True)
            if status.returncode:
                raise SystemExit(f"{name} failed; inspect {out / (name + '.log')}")


if __name__ == "__main__":
    main()
