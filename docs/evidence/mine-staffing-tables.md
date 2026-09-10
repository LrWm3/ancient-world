# Mine-access intervention evidence

GPU: Quadro RTX 5000 with Max-Q Design; backend: Vulkan; driver: NVIDIA 595.84

Treatment minus its matched baseline. Fixed staffing is a diagnostic 62/8/10/20 percent allocation; it does not freeze worker headcount or provide free inputs.

| Seed | Fixed shares | Month | Settled population Δ | Farm workers Δ | Food produced Δ kg/month | Food eaten Δ kg/month | Tool stock Δ kg |
|---|---|---|---:|---:|---:|---:|---:|
| 17 | False | 1 | 0.00 | 31.20 | 0.00 | 0.00 | 0.00 |
| 17 | False | 12 | 1.73 | 269.48 | 0.00 | 178.68 | 34.90 |
| 17 | False | 60 | 168.12 | 82.03 | -2201.25 | 718.51 | -371.21 |
| 17 | False | 120 | -32.51 | 1.60 | -9183.35 | -243.77 | -26.11 |
| 17 | True | 1 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| 17 | True | 12 | -0.09 | 0.02 | 0.00 | -81.00 | 8.14 |
| 17 | True | 60 | -35.61 | -7.63 | -0.10 | -449.16 | -196.61 |
| 17 | True | 120 | -43.93 | -11.47 | -0.14 | -754.23 | -83.57 |
| 81 | False | 1 | 0.00 | 31.08 | 0.00 | 0.00 | 0.00 |
| 81 | False | 12 | 2.68 | 207.24 | 0.00 | 389.11 | 25.51 |
| 81 | False | 60 | 170.59 | 60.48 | -2142.19 | 2395.59 | -141.15 |
| 81 | False | 120 | -35.45 | 7.64 | -6661.75 | -419.77 | 36.75 |
| 81 | True | 1 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| 81 | True | 12 | -0.28 | -0.05 | -0.00 | -8.12 | 26.11 |
| 81 | True | 60 | -16.94 | -3.13 | -0.04 | -332.45 | -70.84 |
| 81 | True | 120 | -22.87 | -5.56 | -0.08 | -184.95 | -32.85 |
| 256 | False | 1 | 0.00 | 30.76 | 0.00 | 0.00 | 0.00 |
| 256 | False | 12 | 1.25 | 271.08 | 0.00 | 225.12 | 21.80 |
| 256 | False | 60 | 196.37 | 87.40 | -3693.00 | 2731.08 | -345.43 |
| 256 | False | 120 | 58.00 | 42.35 | -8852.13 | 1043.47 | -88.77 |
| 256 | True | 1 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| 256 | True | 12 | 0.02 | 0.04 | 0.00 | -70.44 | 12.67 |
| 256 | True | 60 | -26.34 | -5.75 | -0.08 | -1846.73 | -346.44 |
| 256 | True | 120 | -56.31 | -13.04 | -0.19 | -771.31 | -148.00 |

| Seed | Closure settled population effect, adaptive | Closure settled population effect, fixed | Fixed minus adaptive effect |
|---|---:|---:|---:|
| 17 | 168.12 | -35.61 | -203.73 |
| 81 | 170.59 | -16.94 | -187.53 |
| 256 | 196.37 | -26.34 | -222.70 |

| Seed | Fixed shares | First-year farm share Δ percentage points | Closure harvest Δ kg | Closure rations eaten Δ kg |
|---|---|---:|---:|---:|
| 17 | False | 18.118 | 397063.0 | 129767.1 |
| 17 | True | -0.000 | -16035.4 | -21739.9 |
| 81 | False | 15.676 | 309774.6 | 129830.0 |
| 81 | True | 0.000 | -1200.8 | -10455.2 |
| 256 | False | 18.472 | 422137.5 | 140711.1 |
| 256 | True | 0.000 | -13550.1 | -17253.8 |

| Seed | Fixed shares | Closure total living population Δ including travelers |
|---|---|---:|
| 17 | False | 159.32 |
| 17 | True | -35.97 |
| 81 | False | 170.72 |
| 81 | True | -17.22 |
| 256 | False | 190.09 |
| 256 | True | -32.19 |

Maximum monthly normalized economy residual: 2.589e-06; absolute source residual: 0.000e+00 kg; ecology relative C/N/P error: 3.680e-06; population relative error: 1.943e-07.

No-op checks compare complete serialized social history and measured environmental budgets after one month. They do not prove all GPU buffers identical.
Monthly records include site identity, source inventories, births/deaths/migration, rations and all labor pools. Positive long-run population differences alone do not identify a mechanism.
These runs test legacy shared ore/clay extraction, matching the original experiment; mineral-specific/alloy processing is not enabled.
No fitted parameters, confidence intervals, held-out validation or empirical calibration are claimed.
