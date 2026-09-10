# Test summaries and artifact retention

This repository retains Markdown summaries of tests and experiments, not raw
experimental artifacts. Compressed trajectories, binaries, generated screenshots,
logs, JSON reports and run manifests were removed during the September 2026 size
cleanup, and the prior Git history was replaced with one source snapshot.

Existing reports describe historical runs. Statements that raw artifacts were
"retained", old commit hashes and historical artifact paths refer to the earlier
repository and are no longer available here. Numerical summaries are preserved;
they are not a substitute for independently reproducing those runs. Comparisons
requiring removed baseline data must regenerate a suitable baseline or use local
copies. Existing source scripts remain available for new experiments.

For new work, generate results under ignored `output/`. Commit only concise
Markdown summaries recording configuration, seeds, duration, test outcomes,
important failures, numerical tolerances and limitations. Do not add binary or
compressed experimental results to Git.
