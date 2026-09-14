# Food through the first shortage

This corrects the emphasis in [the founding farm screen](founding-farm-balance.md):
low first-year production is not sufficient evidence for an early famine. Imported
provisions must be included, and a yearly total hides the months between harvests.
No food, staffing, trade or demographic rules changed in this investigation.

## What actually happens

Three identical continuations of seeds 1024, 256 and 409 were traced for 36 months,
using the previous screen's frozen 32/32 founding archives and settings. All start
with five towns of 120 people. Declared provisions are **25,920 kg per town**;
society initialization reserves 120 kg in the separate seed-grain compartment,
leaving **25,800 kg per town** at the opening of month one. That small transfer
explains the difference between declared provisions and this trace's opening food.
It is not a mysterious food loss or the explanation for famine.

No town experiences a measured dietary shortfall in year one. First shortfalls
appear at these months (more than 0.01 kg unmet dietary need):

| Seed | Town 0 | Town 1 | Town 2 | Town 3 | Town 4 |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1024 | 31 | 22 | 21 | 18 | 16 |
| 256 | None through 36 | 19 | 18 | 21 | None through 36 |
| 409 | None through 36 | None through 36 | 19 | 18 | 22 |

Selected balances from opening month one **through the first shortage month**:

| Seed / town | Through month | Opening food | Edible production | Recorded consumption | Spoilage | Ending food |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1024 / 4 | 16 | 25,800 | 5,233.52 | 29,475.45 | 1,558.07 | 0 |
| 1024 / 3 | 18 | 25,800 | 7,787.58 | 32,166.55 | 1,421.04 | 0 |
| 256 / 1 | 19 | 25,800 | 9,700.57 | 33,989.79 | 1,510.78 | 0 |
| 1024 / 0 | 31 | 25,800 | 33,739.98 | 57,286.02 | 2,253.95 | 0 |

Values are calorie-equivalent kg, rounded. Opening + production − consumption −
spoilage explains the empty stores; starting supplies were not omitted. There are
no measured town-food changes before or after the GPU step in these runs. The
largest per-town/month GPU balance remainder is 0.00953 kg, consistent with the
float32 cumulative ledgers. This remainder is reported, not treated as an independent
conservation proof. The native ledgers separately pass validation.

The failure is sometimes a bridge to the next harvest:

- Seed 256 town 1 reaches month 19 with 782 kg against approximately 1,869 kg of
  need. In month 20 it produces 7,917 kg, eats about 1,859 kg and ends with 6,058 kg.
- Seed 1024 town 0 first runs short in month 31; month 32 produces 13,078 kg.
- Seed 1024 town 4 starts running short in month 16 and receives its next
  substantial production in month 20. Its output is also much too small to replace
  continuing consumption; this is more than a one-month harvest mismatch.

No sampled GPU step in any of the 540 town-months reports a crop N/P/water cap.
This strengthens the earlier endpoint finding for this **particular early window**,
not for longer runs. Crop suitability, cultivated labor and calendar remain upstream
constraints. Raw standing crops are not edible stock before harvest and processing.

## Why food elsewhere does not arrive

All three runs record **zero market food dispatches** through month 36, despite
requests. These are the recorded mutually exclusive failure counts:

| Seed | No eligible seller surplus | Surplus exists, but no usable route |
| --- | ---: | ---: |
| 1024 | 46 | 0 |
| 256 | 23 | 10 |
| 409 | 8 | 26 |

These counts are repeated decision observations, not quantities of missing food.
An eligible seller must exceed `population × 18 kg × 12 months` in stored food and
permit exports. Thus positive stock in another town is not necessarily offered
for sale. Where eligible surplus exists, the next observed barrier is usable
transport. No food flows appear in the pre/post-GPU stock observations either;
these runs do not receive a hidden net relief or relocation-food transfer.

Reducing the seller reserve alone would not fix the route barrier and could make
the seller miss its own harvest. Increasing local farm work alone would not make
food already elsewhere travel. The next comparisons should separate:

1. A conservative forecast of food needed until the seller's next harvest, versus
   the current fixed twelve-month export reserve, with transport held fixed.
2. The same reserve policies with explicitly available, finite transport, to test
   whether redistribution actually closes the recipient's gap without exporting
   famine to the donor.
3. Earlier farm staffing under the existing work budget, measuring its harvest
   effect and displaced industrial/service work separately.

This is a revised experimental priority, not a claim that those changes will solve
all decline. Retain real food, payment, travel time and labor constraints.

## Optional observation contract

`--demographic-audit=true` now also retains the latest 60 months of managed-town
food observations within `demographic_audit.food`. It remains an aggregate-mode
observer; individual/resolution paths do not supply these rows. Old audits load
with an empty trace, without invented earlier months.

For each active managed town that actually ran production, rows contain:

- Food and cumulative production/consumption/spoilage at schedule opening,
  immediately before GPU production, immediately after GPU readback, and schedule
  close. Missing opening/closing sites are null, not zero.
- Actual GPU dietary need, food eaten and physical food availability, before
  CPU response systems can change demographic state.
- That month's cultivation, tool multiplier, crop constraint and cumulative raw
  crop harvest, so a harvest can be identified without confusing mass and calories.

Readback reuses the existing town buffers. No terrain snapshot, GPU allocation,
random draw or decision rule is added. Duplicate observed months are skipped by
the existing audit guard. Opening/closing values use stable site IDs. Observations
are part of the scheduler transaction and rollback with an unsuccessful month.
Schedule close precedes the separate living-environment return boundary; this
screen is frozen and makes no claim to trace those later living returns.

`report_monthly_food.py` calculates each phase's ledger changes and unclassified
net flow. That remainder can include cargo, relief, expedition provisions or other
transfers; the reporter never labels it automatically as trade. Equal opposing
transfers could cancel, so this is not a gross transaction journal. A nonzero
remainder warrants a narrower subsystem audit, not an automatic conservation pass.

## Validation and reproduction

Use the full flags and founding archives from the preceding screen, with
`--history-years 3 --demographic-audit=true`. Local commands, exports and logs are
under ignored `output/monthly-food-trace/`; exact archived inputs are not committed.

```
python3 scripts/report_monthly_food.py output/monthly-food-trace/{1024,256,409}.json
python3 -m unittest discover -s scripts -p 'test_report_*.py'
CARGO_INCREMENTAL=0 cargo test --lib
```

Strict all-target Clippy passed.

All three complete resulting history JSON objects match the preceding runs exactly
after removing only the added food observations. Native validation passed in each.
The library suite passed 209 tests (156 extended/hardware tests remain ignored).
Twelve Python reporter tests passed, including independent import/consumption/
spoilage/export arithmetic, physical versus access shortfall, missing observations
and detection of an unclassified withdrawal. Rust fixtures check ID-based missing
boundaries, retention, serialization continuation and old-audit loading. Full-world
checkpoint continuation and cross-GPU behavior were not newly tested here.
