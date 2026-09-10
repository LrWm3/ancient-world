# Repository artifact policy

Keep commits source-only: source code, editable catalogs, tests, documentation and
human-readable Markdown test summaries. Do not commit experiment binaries,
compressed results, raw logs, trajectories, generated images, or run manifests.
Write generated experiment outputs under ignored `output/` and summarize settings,
results, failures and limitations in Markdown. Existing experiment scripts can
produce local artifacts but those artifacts must not be added to Git.

Run `python3 scripts/check_repository_artifacts.py` before committing.
Continue committing and pushing normal work as requested by the user. The
single-commit history rewrite was a one-time cleanup; do not routinely rewrite
published history.
