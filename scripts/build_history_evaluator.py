"""Build an immutable evaluator with source provenance, rejecting edits during compilation."""
import argparse
import json
import shutil
import subprocess
from pathlib import Path
from integrated_history import digest


def sources():
    files = [p for root in ('src','shaders','assets','examples')
             for p in sorted(Path(root).rglob('*')) if p.is_file()]
    files += [Path(p) for p in ('Cargo.toml','Cargo.lock','rust-toolchain.toml') if Path(p).exists()]
    return {str(p):digest(p) for p in files}


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output', type=Path, required=True)
    a=p.parse_args()
    a.output.mkdir(parents=True, exist_ok=False)
    before=sources()
    command=['mise','exec','rust@1.89.0','--','cargo','build','--example','history_evaluate']
    with (a.output/'build.log').open('w') as log:
        subprocess.run(command, stdout=log, stderr=subprocess.STDOUT, check=True)
    if sources() != before:
        raise RuntimeError('Source changed during build; no verified executable published')
    binary=a.output/'evaluator.bin'
    shutil.copy2('target/debug/examples/history_evaluate', binary)
    manifest={'binary_sha256':digest(binary),'source_sha256':before,'command':command,
              'base_commit':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()}
    (a.output/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')


if __name__ == '__main__':
    main()
