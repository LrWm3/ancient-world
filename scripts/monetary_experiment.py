#!/usr/bin/env python3
"""Run matched credit/issuance arms; retain raw artifacts only under output/."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import time

DEFAULT_HISTORY_YEARS = 200

ARMS = (
    ("baseline", False, False),
    ("credit", True, False),
    ("issuance", False, True),
    ("combined", True, True),
)


def digest(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=Path("target/debug/ancient-world"))
    parser.add_argument("--checkpoint", action="append", required=True, metavar="LABEL=PATH")
    parser.add_argument("--years", type=int, default=DEFAULT_HISTORY_YEARS)
    parser.add_argument("--output", type=Path, default=Path("output/monetary-experiment"))
    args = parser.parse_args()
    repo = Path(__file__).resolve().parents[1]
    if args.years <= 0:
        parser.error("years must be positive")
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
        "common_payment_policy": "delivery",
    }
    (out / "metadata.json").write_text(json.dumps(metadata, indent=2) + "\n")
    results = []
    for label, checkpoint in cases.items():
        for arm, credit, issuance in ARMS:
            name = f"{label}-{arm}"
            archive = out / f"{name}.json"
            command = [str(fixed), "--headless", "--load", str(checkpoint), "--epochs", "0",
                       "--history-years", str(args.years), "--delivery-paid-exports",
                       f"--commercial-credit={str(credit).lower()}",
                       f"--council-credit={str(credit).lower()}",
                       f"--shared-issuance={str(issuance).lower()}",
                       "--history-export", str(archive)]
            started = time.monotonic()
            with (out / f"{name}.log").open("w") as log:
                status = subprocess.run(command, cwd=repo, stdout=log, stderr=subprocess.STDOUT, check=False)
            result = {"run": name, "seconds": time.monotonic() - started, "exit_code": status.returncode}
            if status.returncode == 0:
                history = json.loads(archive.read_text())
                loans = history["credit"]["loans"]
                result.update(population=sum(s["stocks"]["stock"][0] for s in history["sites"]),
                              loans=len(loans), defaults=sum(l["status"] == "Defaulted" for l in loans),
                              issued=sum(r["issued"] for r in history["credit"]["issuance"]["receipts"]),
                              history_sha256=digest(archive))
            results.append(result)
            (out / "results.json").write_text(json.dumps(results, indent=2) + "\n")
            print(json.dumps(result), flush=True)
            if status.returncode:
                raise SystemExit(f"{name} failed; inspect {out / (name + '.log')}")


if __name__ == "__main__":
    main()
