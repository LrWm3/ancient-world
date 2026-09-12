#!/usr/bin/env python3
"""Run the six continuity additions' controlled checks on an available GPU.

Raw output stays under ignored output/. This does not run a balance calibration.
"""
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[1]
OUT = ROOT / "output" / "continuity-checks"
FILTERS = [
    "institution_relocation::",
    "artifact_petitions::",
    "route_warnings::",
    "relocation_conserves_and_reserves_capacity_and_preserves_identity",
    "contagion::",
    "peace::tests",
    "siege::tests",
]


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    commands = [("library", ["cargo", "test", "--lib"])]
    commands += [
        (name.replace(":", "_"), ["cargo", "test", "--lib", name,
                                  "--", "--include-ignored", "--test-threads=1"])
        for name in FILTERS
    ]
    for name, command in commands:
        log = OUT / f"{name}.log"
        with log.open("w") as stream:
            result = subprocess.run(command, cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT)
        print(f"{name}: {'PASS' if result.returncode == 0 else 'FAIL'} ({log})", flush=True)
        if result.returncode:
            raise SystemExit(result.returncode)


if __name__ == "__main__":
    main()
