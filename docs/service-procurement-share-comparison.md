# Procurement share comparison

Status: protocol fixed before runs. The previous 50-year screen was mixed;
this follow-up tests the cash envelope, not higher fees or easier credit.

Use debt-free seed 1024 at 32/32 for 50 years, five civilizations, delivery-paid
exports and unchanged yield. Run all four monetary arms at surplus shares 0.10
and 0.25 with procurement enabled. Keep the cash floor, input reserve, service fee,
underwriting and issuance caps unchanged. Both series use one fixed executable.

Compare cumulative completed operator work, fees earned/refunded, outstanding
escrow, actual loans, crop harvest and terminal food access. Check the 0.25 series
against the prior screen for unintended changes. The smaller share is promising
only if it improves completed work and affordability rather than merely reducing
funding or changing terminal population. Mixed results retain the opt-in policy
and do not advance the currency gate.

This remains a single-seed parameter screen, not held-out validation or a causal
account of monthly cash shortages. Boundary cash observations and additional seeds
are still needed. Constants verification may run concurrently, so elapsed times
are not isolated performance measurements. Raw outputs belong under ignored
`output/service-procurement-share/`.

Reproduction (repeat with `0.25` and a distinct output directory):

```sh
python3 scripts/monetary_experiment.py \
  --binary output/service-procurement-share/ancient-world \
  --checkpoint 1024=output/monetary-estates-heldout-founding/1024.world \
  --years 50 --service-order-procurement --service-procurement-share 0.10 \
  --output output/service-procurement-share/low
```
