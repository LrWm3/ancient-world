# Purchasing, public cash and institutional service follow-up

This continues the [nutrient growth screen](growth-nutrient-screen.md). Improved
nutrients remain an explicit investigation setting, not an AGENTS.md instruction
or an application default. No simulation parameters or funding rules changed in
this follow-up; the new source is a reusable native-history report and its tests.

## Comparisons

Fresh seeds 1024 and 409, terrain/ecology 32/32, one geological epoch, five
foundings (600 people), 200 years of living aggregate history. Retention is 0.95
for initial towns and monthly finite geological phosphorus release is 5e-7.
Starting food, storage, crop yield, birth/death rules and daughter thresholds are
unchanged. These are game-policy experiments, not empirical calibration.

The purchasing circulation arm enables estate inheritance, estate reclamation,
household wealth tax, council welfare reserves and named office service. It uses
ordinary household food purchasing. The previous funded communal-food arm differs
only by enabling needs-based food. Comparing the purchasing arm with the earlier
nutrient-only arm tests a bundle of circulation policies, not any one tax rate.
The solidarity arm adds only food solidarity to purchasing circulation.

Reproduce either new purchasing run:

```sh
python3 scripts/run_growth_investigation.py --seed 1024 \
  --resolution 32 --ecology-resolution 32 --epochs 1 --civilizations 5 \
  --history-years 200 \
  --enable-system demographic-audit,household-estate-inheritance,household-estate-reclamation,household-wealth-tax,council-welfare-reserves,named-office-service \
  --history-export output/purchasing-1024.json
```

Repeat with seed 409. Add `--enable-system food-solidarity` for the second arm.
All raw exports, checkpoints and logs remain ignored under output/growth-rounds.

## Purchasing circulation results

| Seed | Nutrient-only population | Purchasing circulation population | Final-decade births / deaths | Purchasing gap, whole run | Physical gap, whole run | Active / total towns |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1024 | 360.8 | 499.0 | 119.72 / 144.32 | 2.076% | 0.092% | 5 / 5 |
| 409 | 511.1 | 824.0 | 191.56 / 187.43 | 1.414% | 0.253% | 4 / 5 |

The nutrient-only purchasing gaps were 2.428% and 1.898%. Funded circulation helps
both endpoints but does not establish sustained growth in both seeds or new
settlement formation. Seed 409 has only a small positive final-decade balance;
seed 1024 still declines. Final-50-year purchasing gaps remain 1.929% / 1.628%.

| Seed | Town cash | Council cash | Household cash | Institutional cash | Cumulative wealth tax | Operational / active institutions |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1024 | 15,742.84 | 308.57 | 33,937.15 | 11.44 | 24,994.89 | 2 / 10 |
| 409 | 28,170.89 | 3,962.24 | 17,474.57 | 67.33 | 17,242.78 | 3 / 8 |

Cash columns are selected closing accounts, not an independent all-account money
conservation proof. Tax is cumulative turnover of existing money; it is not new
issuance. Native validation ran during both completed histories. Elapsed times
were 119.5 / 117.8 seconds with concurrent GPU runs, not isolated benchmarks.

## What inspection changes about the diagnosis

In the communal-food growth case, town cash ended near zero because households
no longer pay for food. The purchasing comparison instead leaves substantial cash
in towns, and institutional dues reach 8,912 / 8,629 over two centuries. Neither
"there is no money" nor "councils hold all the money" describes both regimes.
A subsidy designed for communal food should not automatically apply to a town
already earning food-sale revenue.

`social_year()` permits emergency town support only above its hunger threshold.
The request helper repeats that gate. Neither legacy nor CashGap support is a
costed institutional-service subsidy. In the funded communal-food worlds this
can leave councils with cash while food-secure town operating accounts are empty.
In the purchasing world, seed 409 requests no emergency town support at all, yet
has 28,171 in town cash. Removing the hunger gate alone would miss this distinction.

`collect_institution_funding()` still uses a small fraction of town cash in legacy
mode, requires completed cultural administration, and pays after other claims.
Institutional capacity also requires staff, usable space and upkeep. A repaired
building is not sufficient: purchasing seed 409 institution 2 ends with building
condition near 1 and treasury 26.62 but readiness zero. Conversely, several later
institutions have near-zero condition despite town cash being available. Before
changing all institution budgets, inspect service grants, operating-space demand
and actual collection separately.

## Reporting and verification

```sh
python3 scripts/summarize_growth_funding.py output/purchasing-1024.json
python3 -m unittest discover -s scripts -p 'test_*growth*.py'
```

The report shows annual audit coverage, weights food gaps by actual food demand,
and separates final-decade births/deaths from whole-history totals. Operational
counts follow the existing institution predicate, including leadership vacancy,
readiness and building conditions, rather than counting an active identity as a
working institution. Tests cover these distinctions and missing audit data.
Comparisons support only the recorded seeds, resolution and policies. They do not
justify a blanket default change or claim that institutional funding is solved.

### Jurisdiction and timing matter

At the final purchasing relief boundary, seed 409 council 1 has 3,415.29 cash and
no household request or administrative forecast. The other four councils each
have positive requests (37.16–162.88) and zero opening treasury. Closing balances
also include later annual collections, so they cannot substitute for these dated
Reserve receipts. Vacant household accounts retain 5,005.83. Neither observation
justifies taking those stocks without a transfer rule, but both prune the claim
that all apparent reserves are readily available to the needy households.
Seed 1024 has no vacant-household cash; that mechanism cannot explain both seeds.

Institutional support is the minimum of completed basic work, usable-space
coverage, paid fee coverage and local membership coverage. Readiness then loses
additional support under disruption. Existing operating-core and essential-work
pilots address some of these constraints, but were not enabled in this study.
These outcomes do not measure those pilots. A future service comparison should
use the existing institutional state report and work receipts before changing
funding ceilings or assuming every inactive service needs a new subsidy.

## Solidarity added to purchasing circulation

| Seed | People | Final-decade births / deaths | Purchasing gap | Physical gap | Town cash | Operational / active institutions |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 1024 | 887.5 | 207.72 / 211.30 | 1.229% | 0.364% | 28,669.44 | 1 / 11 |
| 409 | 979.0 | 228.63 / 232.85 | 1.236% | 0.251% | 32,067.41 | 2 / 8 |

Still five sites total, with five/four active. Both endpoints improve over the
same funded purchasing bundle without solidarity, but final-decade natural growth
is slightly negative in both. This is not a sustained expansion solution. Later
trajectories and institutional outcomes also differ; more food access did not
uniformly improve service readiness. Concurrent run times were 121.4 / 116.4 s.

Actual cumulative household transfers are 246,254.41 / 60,822.07. Summed donor and
recipient counters agree within 2e-10 currency units. This is one transfer-channel
check, not a replacement for native validation or the complete monetary ledger.

The last monthly receipts distinguish food backing from donor availability. In
seed 409, requested purchases sum to 416.23 and opening food supports approximately
the same amount, but donors provide only 8.04. Every active town is donor-limited.
In seed 1024, one town has 108.14 of food-backed requests and zero donor allowance;
another has a surplus allowance above its own request. These local pools do not
share across towns. Solidarity protects three months of full food cost and spends
at most 5% of the excess each month. Those are policy limits, not missing food.

Closing cash shifts toward municipal accounts, reaching 28,669 / 32,067. The next
bounded investigation should therefore inspect actual town-to-household earnings,
common shares, payroll/dividends and remaining local relief needs. Avoid simply
raising nutrient supply, increasing a population cap, or pooling all council
reserves without jurisdiction. A municipal distribution comparison is a stronger
next candidate than another generic grant to already cash-rich towns.

Separately, a communal-food policy still needs a sustainable service-payment path:
its disappearance of food-sale revenue is real. That question should remain a
separate comparison rather than bundling a purchasing reform, institution subsidy
and new tax into one change. No default was changed based on these four runs.
