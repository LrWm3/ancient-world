#!/usr/bin/env python3
"""Run matched integrated histories; keep generated reports and logs under output/."""
import argparse
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--years', type=int, default=100)
    parser.add_argument('--seeds', default='17,81,256,409,1024')
    parser.add_argument('--output', type=Path, default=Path('output/integration-balance'))
    parser.add_argument('--binary', type=Path, default=Path('target/debug/examples/cultural_work_calibrate'))
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    # The first pair changes resolution; the second changes founding access only.
    variants = [
        ('individual', '0.5', ['--individual-demography', '--workshop-refinement']),
        ('aggregate', '0.5', ['--aggregate-resolution']),
        ('scarcity-founding', '0.33', ['--individual-demography', '--workshop-refinement']),
        ('scarcity-static', '0.33', ['--individual-demography', '--workshop-refinement', '--no-founding-access']),
    ]
    for label, crop_yield, flags in variants:
        command = [str(args.binary), '--seeds', args.seeds, '--years', str(args.years),
                   '--resolution', '32', '--crop-yield-scale', crop_yield,
                   '--compare-resolution', '--output', str(args.output / f'{label}.json'), *flags]
        print(f'Starting {label}: {" ".join(command)}', flush=True)
        with (args.output / f'{label}.log').open('w') as log:
            subprocess.run(command, stdout=log, stderr=subprocess.STDOUT, check=True)
        print(f'Completed {label}', flush=True)


if __name__ == '__main__':
    main()
