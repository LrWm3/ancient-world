# Model evidence run

Profile: full; status: passed
Commit: `38c66d4517f94454a62ec497fe1bfbace9ab7993`

Completion applies only to the selected profile. This is verification and model intervention evidence, not empirical calibration.

| Stage | Status | Seconds | Passed / failed / ignored |
|---|---|---:|---|
| runner-tests | passed | 0.0 | not a test count |
| format | passed | 0.5 | not a test count |
| clippy | passed | 0.2 | not a test count |
| cpu | passed | 37.8 | 41 / 0 / 97 |
| gpu-full | passed | 432.6 | 97 / 0 / 0 |
| mine-intervention | passed | 277.6 | not a test count |
| mine-report | passed | 2.2 | not a test count |

## Coverage limits

- CPU profile skips hardware tests; focused profile runs selected GPU boundaries; full profile executes all Rust tests including ignored cases.
- Test counts describe executions, not unique requirements or code coverage.
- Other GPUs/backends, global sensitivity, empirical fitting and resolution-response convergence remain unverified by this run.
- See docs/model-evidence.md for traceability, equations and interpretation limits.
