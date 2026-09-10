# Model evidence run

Profile: focused; status: passed
Commit: `de8bbf795a21c1905a72612bccfdffb148afeb5d`

Completion applies only to the selected profile. This is verification and model intervention evidence, not empirical calibration.

| Stage | Status | Seconds | Passed / failed / ignored |
|---|---|---:|---|
| runner-tests | passed | 0.0 | not a test count |
| format | passed | 0.5 | not a test count |
| clippy | passed | 0.3 | not a test count |
| cpu | passed | 39.5 | 42 / 0 / 99 |
| gpu-coupling | passed | 60.0 | 20 / 0 / 0 |
| gpu-muster | passed | 13.8 | 1 / 0 / 0 |
| gpu-drainage | passed | 0.6 | 1 / 0 / 0 |
| gpu-exchange | passed | 1.0 | 1 / 0 / 0 |
| mine-intervention | passed | 383.9 | not a test count |
| mine-report | passed | 3.1 | not a test count |

## Coverage limits

- CPU profile skips hardware tests; focused profile runs selected GPU boundaries; full profile executes all Rust tests including ignored cases.
- Test counts describe executions, not unique requirements or code coverage.
- Other GPUs/backends, global sensitivity, empirical fitting and resolution-response convergence remain unverified by this run.
- See docs/model-evidence.md for traceability, equations and interpretation limits.
