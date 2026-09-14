# Municipal food needs before surplus retention

Follow-up to [food-connection planning](food-connection-horizon.md). Its seed 1024
had successful transport and no late physical food deficits, yet purchasing gaps
persisted. This experiment changes allocation of existing municipal cash, not food
production, lending, issuance or transport.

## Opening evidence

At month 2400, the fifteen food-consuming households at site 1 collectively held
about 0.005 currency after retail. The twenty-six at site 4 held about 0.456.
Some households missed roughly 3–4% of their monthly ration despite local food.
World household cash (19,395) concealed this geographic concentration.

After other relief and before retail, municipal assistance at site 1 requested
288.62 currency, backed by available food. It paid 183.13: the 5% allowance on
surplus above protected working cash was binding. Site 4 requested 383.34 and paid
231.43 for the same reason. They retained protected cash plus unspent surplus.
These are one-boundary observations, not proof that the same restriction dominated
every year of decline.

## Allocation policy

Enable `municipal-welfare-reserves` alongside `municipal-food-relief` to allow
remaining food-backed purchasing gaps to draw on all cash **above** the existing
10-currency-per-resident working reserve, rather than only 5% of that surplus.
Both options are registered and independently inspectable. The former selects an
allocation rule; it does not activate the latter's transfer mechanism by itself.
Disabling the new option restores the 5% rule. Existing archives default to that
rule. Receipts retain the rule used for each dated allocation.

Transfers remain bounded by household need, available local food after already
funded entitlement, and actual town cash above its reserve. Funds are credited
once to household wallets and relief ledgers. The existing GPU consumption and
retail settlement then purchase actual food; payments return to the town. Nothing
creates food, money, labor or permanent credit. The Reserve→Execute/settle timing
is unchanged and repeated preparation cannot pay twice in one month.

This can recycle municipal purchasing money through food sales. It also competes
with later town spending: a working reserve is not a guarantee that every industry,
ship or public service remains funded. No policy default changes are implied.

## Comparison

Start each control and treatment from the same saved year-200 forward-connection
world (seeds 1024/409), and advance five living years. This is a short intervention
on already-diverged worlds, not a new 200-year world-generation comparison.

```sh
target/debug/ancient-world --headless --epochs 0 \
  --load output/growth-rounds/19-forward-connections-1024-200.world \
  --history-years 5 --enable-system municipal-welfare-reserves \
  --history-export output/municipal-needs-1024.json
```

Use `--disable-system municipal-welfare-reserves` for the control. Municipal relief
is already enabled in these checkpoints. Count only years 201–205 for food and
demographic changes; retained earlier observations are not intervention results.

## Results

The immediate mediator changed as intended. Over years 201–205 there were no
physical food shortfalls in either arm. Purchasing shortfalls changed from 1.186%
to approximately 0.00001% in seed 1024, and 1.421% to approximately 0.00001% in
seed 409 (small floating-point residuals remain). Populations at year 205 were
753 / 777 and 1,279 / 1,329 for control / treatment respectively.

The same year-200 checkpoints were then advanced for thirty years:

| Seed | Municipal rule | Year-230 population | Years 201–230 births / deaths | Purchasing shortfall | Town cash | Operational / active institutions |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1024 | 5% surplus | 668 | 510 / 582 | 2.437% | 28,075 | 8 / 16 |
| 1024 | Needs first | 838 | 565 / 467 | 0.989% | 30,043 | 5 / 16 |
| 409 | 5% surplus | 1,295 | 892 / 866 | 1.596% | 38,212 | 16 / 20 |
| 409 | Needs first | 1,538 | 979 / 710 | 0.513% | 35,527 | 20 / 22 |

All four thirty-year arms had zero measured physical shortfalls in the intervention
window. The new rule improved population in both cases without exhausting town
cash. Relief money can return through existing retail payments; larger gross
relief is not equivalent to permanently removing that cash from municipal use.
At the last boundary seed 1024 paid 1,950 rather than 379 currency in municipal
relief; seed 409 paid 4,275 rather than 867. These are latest payments, not thirty-year
cumulative totals.

The result is still mixed across systems. Seed 1024's operational institution count
fell relative to control, while seed 409's increased and it gained an active town.
The disappearance of purchasing gaps over five years did not persist for thirty.
Protected working cash, physically backed relief and changed later demands remain
real constraints; this comparison does not attribute each later shortfall to a
specific one. Nor does it establish that every town should prioritize food over
all subsequent expenditure. The rule remains opt-in.

## Verification and limitations

- Four focused municipal transfer tests pass, including food/cash limits,
  conservation, old defaults, monthly guard, ordering and serialization. The new
  controlled fixture changes the binding allowance without changing stocks or need.
- Registry enable/disable/archive/resume GPU fixture passes with the new option.
- Regular library suite: 218 passed, 158 ignored. Clippy passes for library/native
  binary with warnings denied.
- Eight native comparison runs completed: two seeds × two policies × five/thirty
  years. Five-year pairs took about five seconds per run; thirty-year pairs about
  22 seconds on this machine. These concurrent wall times are not benchmarks.
- Seed 1024 treatment run as five years → save/load → twenty-five years produced
  **identical complete exported history JSON** to the uninterrupted thirty-year
  treatment. This does not claim byte equality of all GPU world buffers.
- Reporting now accepts `summarize_growth_funding.py --after-year 200` so preexisting
  years are excluded from intervention food/death totals. Endpoint cash and
  cumulative account counters remain endpoint/lifetime quantities; they are not
  mislabeled as interval flows. Nine growth-focused Python tests pass.
- Initial runner attempts omitted `--epochs 0` and correctly failed because
  geological generation is locked after founding. No comparison results came from
  those attempts; the command above uses the corrected invocation.

These are two selected diagnostic-resolution worlds with a specific policy and
nutrient bundle, not held-out calibration or a new default recommendation. All
raw logs/worlds/exports stay under ignored output; only source and this summary
are committed.
