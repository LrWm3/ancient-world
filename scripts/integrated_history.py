"""Run reproducible integrated history diagnostics; never reuse an output directory."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import shutil
import time


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--binary', type=Path, default=Path('target/debug/examples/history_evaluate'))
    p.add_argument('--seeds', default='17,81,256')
    p.add_argument('--build-manifest', type=Path)
    p.add_argument('--years', type=int, default=100)
    p.add_argument('--resolution', type=int, default=64)
    p.add_argument('--profiles', default='baseline,harsh')
    p.add_argument('--no-negotiated-autonomy', action='store_true')
    p.add_argument('--offices', action='store_true')
    p.add_argument('--occupation-months',type=int)
    p.add_argument('--land-freight-kg-per-person', type=float)
    a = p.parse_args()
    seeds = [int(s) for s in a.seeds.split(',')]
    if not seeds or len(seeds) != len(set(seeds)) or a.years < 1:
        p.error('seeds must be unique and years positive')
    profiles = a.profiles.split(',')
    if not profiles or len(profiles) != len(set(profiles)) or any(v not in ('baseline','harsh') for v in profiles):
        p.error('profiles must be unique baseline/harsh names')
    if a.land_freight_kg_per_person is not None and not 1 <= a.land_freight_kg_per_person <= 1000:
        p.error('land freight capacity must be between 1 and 1000 kg/person')
    a.output.mkdir(parents=True, exist_ok=False)
    binary = (a.output/'evaluator.bin').resolve()
    shutil.copy2(a.binary.resolve(), binary)
    manifest = {'complete': False, 'base_commit': subprocess.check_output(
        ['git', 'rev-parse', 'HEAD'], text=True).strip(), 'binary_sha256': digest(binary),
        'seeds': seeds, 'profiles': profiles, 'years': a.years, 'resolution': a.resolution, 'runs': [],
        'launch_source_sha256': {str(f): digest(f) for root in ('src', 'shaders', 'assets', 'examples')
                          for f in sorted(Path(root).rglob('*')) if f.is_file()}}
    if a.build_manifest is not None:
        build = json.loads(a.build_manifest.read_text())
        if build['binary_sha256'] != manifest['binary_sha256']:
            raise ValueError('Build manifest does not describe this executable')
        manifest['build'] = build
    def persist():
        (a.output/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
    persist()
    for profile in profiles:
        crop_yield = {'baseline':'0.5', 'harsh':'0.33'}[profile]
        for seed in seeds:
            name = f'{profile}-{seed}'
            output = a.output/name
            cmd = [str(binary), '--seeds', str(seed), '--years', str(a.years),
                   '--resolution', str(a.resolution), '--civilizations', '16', '--epochs', '1',
                   '--crop-yield-scale', crop_yield, '--discoveries', '--living-world',
                   '--waterworks-repair-priority', '--output', str(output), '--label', name]
            if a.occupation_months is not None:
                cmd.extend(['--occupation-months',str(a.occupation_months)])
            if a.offices:
                cmd.append('--offices')
            if a.no_negotiated_autonomy:
                cmd.append('--no-negotiated-autonomy')
            if a.land_freight_kg_per_person is not None:
                cmd.extend(['--land-freight-kg-per-person', str(a.land_freight_kg_per_person)])
            record = {'name': name, 'command': cmd, 'status': 'running'}
            manifest['runs'].append(record)
            persist()
            print(f'Starting {name}', flush=True)
            start = time.monotonic()
            with output.with_suffix('.log').open('w') as log:
                result = subprocess.run(cmd, stdout=log, stderr=subprocess.STDOUT)
            record.update(seconds=time.monotonic()-start, exit_code=result.returncode)
            if result.returncode:
                record['status'] = 'failed'
                persist()
                raise SystemExit(f'{name} failed; see {output.with_suffix(".log")}')
            data = json.loads(output.with_suffix('.json').read_text())
            if not data.get('complete') or len(data['runs']) != 1 or len(data['runs'][0]['samples']) != a.years or data['runs'][0]['seed'] != seed or data['resolution'] != a.resolution:
                record['status'] = 'invalid_output'
                persist()
                raise SystemExit('Incomplete trajectory')
            record['status'] = 'complete'
            record['json_sha256'] = digest(output.with_suffix('.json'))
            persist()
            print(f'Completed {name} ({record["seconds"]:.1f}s)', flush=True)
    manifest['complete'] = True
    persist()


if __name__ == '__main__':
    main()
