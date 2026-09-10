"""Reject generated artifacts in the Git index; local output/ remains unrestricted."""
from pathlib import PurePosixPath
import subprocess
import sys

names = subprocess.check_output(['git', 'ls-files', '-z']).decode().split('\0')
blocked = {'.gz', '.zip', '.tar', '.tgz', '.bz2', '.xz', '.7z', '.bin', '.exe',
           '.log', '.jsonl', '.world', '.png', '.jpg', '.jpeg', '.webp', '.pdf'}
bad = [n for n in names if n and (
    PurePosixPath(n).suffix.lower() in blocked
    or n.startswith(('output/', 'target/'))
    or (n.startswith('docs/evidence/') and not n.endswith('.md')))]
if bad:
    print('Keep generated artifacts out of Git:\n' + '\n'.join(bad), file=sys.stderr)
    sys.exit(1)
print('Repository artifact policy passed.')
