# Automatic procurement: 50-year screening protocol

Status: running; results have not yet been accepted or summarized.

Use the existing debt-free seed-1024 founding checkpoint at terrain/ecology
resolution 32/32, five civilizations, for 50 years. Hold delivery-paid exports,
starting inventories and crop yield settings constant. Run the four standard
monetary arms with procurement enabled, then the same four with it disabled.
Service-order lending follows the credit switch only in the procurement series.
Both series explicitly set procurement and service-lending flags so the checkpoint
cannot silently choose them.

The executable is a copy of the native build for `02b74c3`, taken before the next
constants pass. The runner records its SHA-256 and the checkpoint checksum locally.
Commands:

```sh
python3 scripts/monetary_experiment.py \
  --binary output/service-procurement-screen/ancient-world \
  --checkpoint 1024=output/monetary-estates-heldout-founding/1024.world \
  --years 50 --service-order-procurement \
  --output output/service-procurement-screen/with

python3 scripts/monetary_experiment.py \
  --binary output/service-procurement-screen/ancient-world \
  --checkpoint 1024=output/monetary-estates-heldout-founding/1024.world \
  --years 50 --output output/service-procurement-screen/without
```

Assess the immediate mediators first: orders funded, fees earned versus refunded,
remaining escrow, operator completed work and actual loans. Then compare food
harvest separately from terminal household affordability, council/institutional
shortfalls, population and money residuals. Endpoint measures must not be described
as monthly maxima. More fee commitments alone are not a benefit.

This is an initial integration screen, not a held-out multi-seed benefit gate.
Failures or regressions should trigger a specific investigation, not immediate
fee increases or reserve reductions. Concurrent constants verification shares the
GPU, so elapsed times here cannot support isolated performance claims. Generated
archives/logs stay under ignored `output/service-procurement-screen/`.
