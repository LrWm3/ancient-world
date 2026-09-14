#!/usr/bin/env python3
"""Run native history with the agreed nutrient baseline; forward other native options.

This preset changes investigation runs, not game defaults. Food-access policies
remain as selected by the native defaults, archive, or explicit system switches.
"""
import argparse
from pathlib import Path
import shlex
import subprocess

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_NUTRIENT_RETENTION = 0.95
DEFAULT_PHOSPHORUS_RELEASE_MONTHLY_FRACTION = 5e-7


def command(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--binary', type=Path, default=ROOT / 'target/debug/ancient-world')
    parser.add_argument('--farm-nutrient-retention', type=float, default=DEFAULT_NUTRIENT_RETENTION)
    parser.add_argument('--farm-phosphorus-release', type=float,
                        default=DEFAULT_PHOSPHORUS_RELEASE_MONTHLY_FRACTION)
    args, remaining = parser.parse_known_args(argv)
    if not 0 <= args.farm_nutrient_retention <= 1:
        parser.error('nutrient retention must be finite and within 0–1')
    if not 0 <= args.farm_phosphorus_release <= 1e-4:
        parser.error('monthly phosphorus release must be finite and within 0–0.0001')
    return [str(args.binary), *([] if '--headless' in remaining else ['--headless']),
            '--farm-nutrient-retention', str(args.farm_nutrient_retention),
            '--farm-phosphorus-release', str(args.farm_phosphorus_release), *remaining]


def main():
    cmd = command()
    print(shlex.join(cmd), flush=True)
    return subprocess.run(cmd, check=False).returncode


if __name__ == '__main__':
    raise SystemExit(main())
