# Model evidence run

Profile: focused; status: passed
Commit: `60189b231f7f064a9c9cde7ab838eba11aaeec9f`

Completion applies only to the selected profile. This is verification and model intervention evidence, not empirical calibration.

| Stage | Status | Seconds | Passed / failed / ignored |
|---|---|---:|---|
| runner-tests | passed | 0.1 | not a test count |
| format | passed | 0.6 | not a test count |
| clippy | passed | 0.2 | not a test count |
| cpu | passed | 22.4 | 42 / 0 / 98 |
| gpu-coupling | passed | 21.5 | 19 / 0 / 0 |
| gpu-muster | passed | 13.0 | 1 / 0 / 0 |
| gpu-drainage | passed | 0.5 | 1 / 0 / 0 |
| gpu-exchange | passed | 0.9 | 1 / 0 / 0 |
| mine-intervention | passed | 274.7 | not a test count |
| mine-report | passed | 5.1 | not a test count |

## Coverage limits

- CPU profile skips hardware tests; focused profile runs selected GPU boundaries; full profile executes all Rust tests including ignored cases.
- Test counts describe executions, not unique requirements or code coverage.
- Other GPUs/backends, global sensitivity, empirical fitting and resolution-response convergence remain unverified by this run.
- See docs/model-evidence.md for traceability, equations and interpretation limits.
