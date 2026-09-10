"""Publish complete employer suites with deterministic compression and retained source hashes."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path
import re
from evaluate_enterprises import summarize, write_report


def sha(data): return hashlib.sha256(data).hexdigest()


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--suite', action='append', required=True, help='label=completed-output-directory')
    p.add_argument('--log', action='append', default=[], help='label=verification-log')
    p.add_argument('--output', type=Path, required=True)
    a=p.parse_args()
    planned={}
    suites=[]
    def add(name,data):
        if name in planned: raise ValueError('Duplicate artifact destination')
        planned[name]=data
    for value in a.suite:
        label,folder=value.split('=',1)
        if not re.fullmatch(r'[a-z0-9_-]+',label): raise ValueError('Invalid suite label')
        directory=Path(folder)
        raw=(directory/'manifest.json').read_bytes()
        manifest=json.loads(raw)
        if not manifest.get('complete') or any(r['status']!='complete' for r in manifest['runs']):
            raise ValueError('Refusing to publish incomplete suite')
        add(f'{label}/run-manifest.json.gz',gzip.compress(raw,mtime=0))
        rows=[]
        for record in manifest['runs']:
            name=record['name']
            raw=(directory/(name+'.json')).read_bytes()
            if sha(raw)!=record['json_sha256']: raise ValueError('Trajectory hash mismatch')
            data=json.loads(raw);row=summarize(data)
            row['mode']=name.rsplit('-',1)[0];rows.append(row)
            add(f'{label}/{name}.json.gz',gzip.compress(raw,mtime=0))
            add(f'{label}/{name}.log.gz',gzip.compress((directory/(name+'.log')).read_bytes(),mtime=0))
        # Regenerate comparisons from verified raw data using the checked-in analysis code.
        write_report(directory,rows)
        for filename in ('summary.json','summary.md','effects.json'):
            add(f'{label}/{filename}',(directory/filename).read_bytes())
        suites.append(dict(label=label,runs=len(rows),source_manifest=f'{label}/run-manifest.json.gz'))
    for value in a.log:
        label,filename=value.split('=',1)
        if not re.fullmatch(r'[a-z0-9_-]+',label): raise ValueError('Invalid log label')
        add(f'verification/{label}.log.gz',gzip.compress(Path(filename).read_bytes(),mtime=0))
    a.output.mkdir(parents=True,exist_ok=False)
    for filename,data in planned.items():
        target=a.output/filename;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(data)
    manifest=dict(schema=1,suites=suites,artifacts_sha256={name:sha(data) for name,data in sorted(planned.items())},
        analysis_sha256={name:sha(Path('scripts',name).read_bytes()) for name in
            ('evaluate_enterprises.py','publish_enterprise_evidence.py')},
        scope='Matched game-balance experiments on one Vulkan GPU at terrain/ecology resolution 64; no empirical calibration or cross-hardware claim. Run manifests retain exact executable/source hashes. Verification logs retain their actual test counts; ignored fixtures are not passes.')
    (a.output/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print(f'Published {len(planned)} artifacts, {sum(s["runs"] for s in suites)} trajectories to {a.output}')

if __name__=='__main__': main()
