# Maintenance ablation and policy evaluation

GPU: Quadro RTX 5000 with Max-Q Design; backend: Vulkan; driver: NVIDIA 595.84

Matched checkpoints; finite inventory and unchanged coefficients. All four policies run on every seed/reserve combination. Differences below are first policy minus second.

| Seed | Tool access | Closed mines | Contrast | Month | Settled population Δ | Harvest Δ kg cumulative | Unmet rations Δ kg cumulative | Tool multiplier Δ | Tools made Δ kg cumulative |
|---|---|---|---|---:|---:|---:|---:|---:|---:|
| 7307 | 1 | False | food-maintenance − adaptive | 60 | 35.210 | 29235.18 | -9224.86 | 0.000174 | 10.782 |
| 7307 | 1 | False | food-maintenance − adaptive | 120 | 26.268 | 55057.58 | -6557.89 | 0.000143 | 30.231 |
| 7307 | 1 | True | food-maintenance − adaptive | 60 | 0.050 | -1410.78 | -17.05 | 0.000085 | 8.960 |
| 7307 | 1 | True | food-maintenance − adaptive | 120 | 7.216 | 2612.33 | -1233.32 | 0.000091 | 0.446 |
| 7307 | 1 | False | food-only − adaptive | 60 | 35.238 | 29250.38 | -9235.25 | 0.000174 | 10.805 |
| 7307 | 1 | False | food-only − adaptive | 120 | 26.300 | 55179.21 | -6567.97 | 0.000282 | 28.591 |
| 7307 | 1 | True | food-only − adaptive | 60 | 0.078 | 101.56 | -27.00 | 0.000009 | 0.006 |
| 7307 | 1 | True | food-only − adaptive | 120 | 7.425 | 4686.26 | -1289.23 | 0.000066 | 1.468 |
| 7307 | 1 | False | food-maintenance − food-only | 60 | -0.028 | -15.20 | 10.39 | -0.000000 | -0.023 |
| 7307 | 1 | False | food-maintenance − food-only | 120 | -0.031 | -121.63 | 10.09 | -0.000139 | 1.640 |
| 7307 | 1 | True | food-maintenance − food-only | 60 | -0.028 | -1512.35 | 9.95 | 0.000077 | 8.954 |
| 7307 | 1 | True | food-maintenance − food-only | 120 | -0.209 | -2073.94 | 55.92 | 0.000026 | -1.021 |
| 7307 | 0 | False | food-maintenance − adaptive | 60 | 53.995 | 45651.36 | -17282.85 | 0.001443 | 20.733 |
| 7307 | 0 | False | food-maintenance − adaptive | 120 | 47.568 | 91182.53 | -13353.27 | 0.001107 | 46.772 |
| 7307 | 0 | True | food-maintenance − adaptive | 60 | 1.615 | 903.45 | -618.29 | 0.000352 | 1.795 |
| 7307 | 0 | True | food-maintenance − adaptive | 120 | 15.772 | 19550.29 | -6548.76 | 0.000102 | 17.538 |
| 7307 | 0 | False | food-only − adaptive | 60 | 56.101 | 48543.80 | -18025.19 | 0.001506 | 8.330 |
| 7307 | 0 | False | food-only − adaptive | 120 | 47.468 | 92819.89 | -13263.69 | 0.001104 | 40.789 |
| 7307 | 0 | True | food-only − adaptive | 60 | 2.084 | 1307.78 | -779.16 | -0.000116 | 0.047 |
| 7307 | 0 | True | food-only − adaptive | 120 | 11.732 | 17355.86 | -7058.78 | 0.000197 | 15.317 |
| 7307 | 0 | False | food-maintenance − food-only | 60 | -2.106 | -2892.44 | 742.33 | -0.000063 | 12.404 |
| 7307 | 0 | False | food-maintenance − food-only | 120 | 0.100 | -1637.35 | -89.58 | 0.000003 | 5.982 |
| 7307 | 0 | True | food-maintenance − food-only | 60 | -0.469 | -404.32 | 160.87 | 0.000468 | 1.747 |
| 7307 | 0 | True | food-maintenance − food-only | 120 | 4.040 | 2194.42 | 510.02 | -0.000095 | 2.221 |
| 12011 | 1 | False | food-maintenance − adaptive | 60 | 25.749 | 23119.60 | -9644.31 | 0.000000 | 18.465 |
| 12011 | 1 | False | food-maintenance − adaptive | 120 | 25.611 | 39733.76 | -8953.90 | 0.000000 | 20.621 |
| 12011 | 1 | True | food-maintenance − adaptive | 60 | 2.424 | 2526.05 | -893.12 | 0.001357 | -1.654 |
| 12011 | 1 | True | food-maintenance − adaptive | 120 | 5.150 | 9499.54 | -1746.11 | 0.000000 | -12.580 |
| 12011 | 1 | False | food-only − adaptive | 60 | 25.749 | 23119.60 | -9644.31 | 0.000000 | 18.465 |
| 12011 | 1 | False | food-only − adaptive | 120 | 25.611 | 39733.76 | -8953.90 | 0.000000 | 20.621 |
| 12011 | 1 | True | food-only − adaptive | 60 | 2.457 | 3170.39 | -904.98 | 0.001429 | -3.770 |
| 12011 | 1 | True | food-only − adaptive | 120 | 7.173 | 4598.48 | -730.67 | 0.000000 | -13.658 |
| 12011 | 1 | False | food-maintenance − food-only | 60 | 0.000 | 0.00 | 0.00 | 0.000000 | 0.000 |
| 12011 | 1 | False | food-maintenance − food-only | 120 | 0.000 | 0.00 | 0.00 | 0.000000 | 0.000 |
| 12011 | 1 | True | food-maintenance − food-only | 60 | -0.033 | -644.34 | 11.87 | -0.000072 | 2.116 |
| 12011 | 1 | True | food-maintenance − food-only | 120 | -2.022 | 4901.06 | -1015.44 | 0.000000 | 1.077 |
| 12011 | 0 | False | food-maintenance − adaptive | 60 | 60.364 | 55237.10 | -21647.70 | 0.000000 | 29.932 |
| 12011 | 0 | False | food-maintenance − adaptive | 120 | 63.643 | 103535.60 | -21099.95 | 0.000000 | 53.323 |
| 12011 | 0 | True | food-maintenance − adaptive | 60 | 3.515 | 1692.70 | -1371.39 | -0.000299 | -6.037 |
| 12011 | 0 | True | food-maintenance − adaptive | 120 | 16.771 | 18328.89 | -5156.00 | 0.000000 | 18.457 |
| 12011 | 0 | False | food-only − adaptive | 60 | 54.331 | 58040.60 | -21850.43 | 0.000000 | 32.773 |
| 12011 | 0 | False | food-only − adaptive | 120 | 64.457 | 107113.37 | -21366.55 | 0.000000 | 53.047 |
| 12011 | 0 | True | food-only − adaptive | 60 | 4.522 | 3593.03 | -1723.71 | 0.000140 | -2.648 |
| 12011 | 0 | True | food-only − adaptive | 120 | 13.676 | 17525.05 | -4818.28 | 0.000000 | 17.254 |
| 12011 | 0 | False | food-maintenance − food-only | 60 | 6.033 | -2803.49 | 202.74 | 0.000000 | -2.841 |
| 12011 | 0 | False | food-maintenance − food-only | 120 | -0.814 | -3577.77 | 266.61 | 0.000000 | 0.275 |
| 12011 | 0 | True | food-maintenance − food-only | 60 | -1.007 | -1900.32 | 352.32 | -0.000440 | -3.389 |
| 12011 | 0 | True | food-maintenance − food-only | 120 | 3.096 | 803.84 | -337.72 | 0.000000 | 1.204 |
| 19001 | 1 | False | food-maintenance − adaptive | 60 | 35.513 | 28382.42 | -13010.83 | 0.000000 | 8.716 |
| 19001 | 1 | False | food-maintenance − adaptive | 120 | 17.169 | 44142.08 | -8296.47 | 0.000000 | 19.542 |
| 19001 | 1 | True | food-maintenance − adaptive | 60 | -0.007 | -245.22 | 2.43 | 0.000002 | 0.729 |
| 19001 | 1 | True | food-maintenance − adaptive | 120 | -2.469 | 6554.68 | -4781.04 | 0.000000 | -4.937 |
| 19001 | 1 | False | food-only − adaptive | 60 | 35.513 | 28382.42 | -13010.83 | 0.000000 | 8.716 |
| 19001 | 1 | False | food-only − adaptive | 120 | 17.169 | 44142.08 | -8296.47 | 0.000000 | 19.542 |
| 19001 | 1 | True | food-only − adaptive | 60 | 0.000 | 0.00 | -0.00 | 0.000000 | -0.000 |
| 19001 | 1 | True | food-only − adaptive | 120 | 7.532 | 7143.31 | -4514.08 | 0.000000 | 1.253 |
| 19001 | 1 | False | food-maintenance − food-only | 60 | 0.000 | 0.00 | 0.00 | 0.000000 | 0.000 |
| 19001 | 1 | False | food-maintenance − food-only | 120 | 0.000 | 0.00 | 0.00 | 0.000000 | 0.000 |
| 19001 | 1 | True | food-maintenance − food-only | 60 | -0.007 | -245.22 | 2.44 | 0.000002 | 0.729 |
| 19001 | 1 | True | food-maintenance − food-only | 120 | -10.002 | -588.63 | -266.96 | 0.000000 | -6.190 |
| 19001 | 0 | False | food-maintenance − adaptive | 60 | 70.299 | 55383.22 | -23839.69 | 0.000000 | 32.203 |
| 19001 | 0 | False | food-maintenance − adaptive | 120 | 48.958 | 104157.56 | -18885.59 | 0.000000 | 51.113 |
| 19001 | 0 | True | food-maintenance − adaptive | 60 | -0.201 | -469.27 | 71.22 | 0.001486 | 5.601 |
| 19001 | 0 | True | food-maintenance − adaptive | 120 | 30.862 | 23878.55 | -13753.99 | 0.000000 | 5.301 |
| 19001 | 0 | False | food-only − adaptive | 60 | 70.555 | 55403.82 | -23981.39 | 0.000000 | 31.952 |
| 19001 | 0 | False | food-only − adaptive | 120 | 45.281 | 100833.54 | -18093.58 | 0.000000 | 45.745 |
| 19001 | 0 | True | food-only − adaptive | 60 | 0.054 | 55.37 | -18.37 | 0.000001 | 0.021 |
| 19001 | 0 | True | food-only − adaptive | 120 | 30.697 | 25969.17 | -12814.39 | 0.000000 | 5.513 |
| 19001 | 0 | False | food-maintenance − food-only | 60 | -0.256 | -20.60 | 141.70 | 0.000000 | 0.250 |
| 19001 | 0 | False | food-maintenance − food-only | 120 | 3.677 | 3324.02 | -792.01 | 0.000000 | 5.368 |
| 19001 | 0 | True | food-maintenance − food-only | 60 | -0.255 | -524.64 | 89.59 | 0.001485 | 5.581 |
| 19001 | 0 | True | food-maintenance − food-only | 120 | 0.165 | -2090.62 | -939.59 | 0.000000 | -0.212 |

| Seed | Tool access | Closed mines | Maintenance: first differing labor month | Identical complete monthly observations |
|---|---|---|---:|---:|
| 7307 | 1 | False | 13 | 12/120 |
| 7307 | 1 | True | 30 | 29/120 |
| 7307 | 0 | False | 1 | 0/120 |
| 7307 | 0 | True | 1 | 0/120 |
| 12011 | 1 | False | none | 120/120 |
| 12011 | 1 | True | 33 | 32/120 |
| 12011 | 0 | False | 1 | 0/120 |
| 12011 | 0 | True | 1 | 0/120 |
| 19001 | 1 | False | none | 120/120 |
| 19001 | 1 | True | 30 | 29/120 |
| 19001 | 0 | False | 1 | 0/120 |
| 19001 | 0 | True | 1 | 0/120 |

Maximum economy_residuals: 3.201437e-06.

Maximum source_residual: 0.000000e+00.

Maximum population_residual: 1.605587e-07.

Maximum ecology_relative_error: 3.654122e-06.

Maximum ecology_water_relative_error: 2.782312e-05.

5760 monthly observations. Unmet rations are measured kg calorie equivalents; population and therefore ration demand can change. Later differences do not isolate every intermediate mechanism. Identical recorded observations do not prove all GPU state identical.
These are legacy shared-resource economies at one terrain/ecology resolution, not empirical calibration or cross-hardware validation.
