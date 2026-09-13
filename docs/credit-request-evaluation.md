# Recorded credit demand and rejection evidence

The monetary runner now reports the recorded proposal stages separately:
requested principal, eligible principal, numerical grants and actual committed
principal. It also counts decisions, repayment-source types, binding lender/
borrower/source capacity receipts, incomplete rounds and loans outside rounds.

These are cumulative request observations, not distinct projects or outstanding
debt. Repeated applications can count the same underlying need more than once.
The pilots do not record months in which they construct no requests. Zero recorded
requests cannot distinguish no funding gap, missing contact, absent repayment
evidence or a request discarded before underwriting.

Actual principal is looked up by stable loan ID rather than inferred from a
numerical grant. Request and grant ordering can differ. Missing capacity receipts
in older archives are explicitly counted; an empty binding-count map alone is not
evidence of unconstrained credit. Missing round records are also distinct from a
present but empty round list.

## Reanalysis of retained experiments

Applied the reporter to all 20 history exports in the existing crop-control,
crop-scarcity and held-out estate ensembles. This reuses previous runs; it is not
a rerun of current source or evidence for the recent commercial request fix.

| Saved run | Recorded requests | Recorded result | Committed principal |
| --- | ---: | --- | ---: |
| Crop control, seed 409, credit | 0 | No recorded request | 0 |
| Crop control, seed 409, combined | 1 export | Approved | 2.023681640625 |
| Crop scarcity, seed 409, credit and combined | 0 each | No recorded request | 0 |
| Held-out, seed 256, credit and combined | 0 each | No recorded request | 0 |
| Held-out, seed 409, credit | 0 | No recorded request | 0 |
| Held-out, seed 409, combined | 1 export | Approved | 2.023681640625 |
| Held-out, seed 1024, credit | 6 (1 tax, 5 export) | All NoCapacity | 0 |
| Held-out, seed 1024, combined | 58 (53 tax, 5 export) | All NoCapacity | 0 |

All baseline and issuance-only arms have no recorded requests, as expected with
new credit disabled. All inspected rounds are complete and every recorded loan
belongs to a round. The one seed-409 grant was 2.0238299714692403; the lower
committed amount is the actual representable cash transfer. It must not be
reported as the grant amount.

All 64 seed-1024 grants predate detailed capacity receipts. Inspecting their
archived underwriting inputs independently shows expected receipts at or below
recorded operating costs in every request. Five export requests in each arm also
have no offered cash above reserves. This supports examining receipt/cost
forecasts and real feasible spending needs next; simply loosening exposure caps
would not address those recorded nonpositive net receipts.

The crop-scarcity results therefore do not demonstrate a functioning credit
intervention failing to save towns: in those arms the lending path never reached
a recorded request. Nor do seed-1024 rejection counts demonstrate that credit
would have helped if approved. Useful-work counterfactuals remain necessary.

## Verification and reproduction

Run `python3 -m unittest discover -s scripts -p test_monetary_experiment.py`.
Six tests pass, covering stage separation, tied constraints, absent legacy records,
incomplete rounds, explicit loans, reordered requests, rejection counts and
invalid/dangling receipts. No Rust or shader behavior changes in this increment.

New `scripts/monetary_experiment.py` runs include these fields in their normal
results. To inspect a retained history export without advancing it:

```sh
PYTHONPATH=scripts python3 - output/monetary-crop-control/409-combined.json <<'PY'
import json
import sys
from pathlib import Path
from monetary_experiment import credit_funnel
print(json.dumps(credit_funnel(json.loads(Path(sys.argv[1]).read_text())), indent=2))
PY
```

Raw histories and analysis output remain in ignored `output/`. Read this with the
original ensemble summaries for settings, revisions, population outcomes and
their limits.
