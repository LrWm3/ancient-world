# Wildlife assembly evaluation

This records the initial pass. The [trophic stability follow-up](wildlife-trophic-stability.md)
adds rare-prey refuges and evaluates revised intake settings against these failures.

This pass establishes explicit animal founders, habitat-connected dispersal, neutral
origin tracking and an economy connection. It does **not** establish stable,
centuries-long food webs or independently evolved island species. Removing implicit
animal recruitment exposed weaknesses that previous nonzero animal stocks concealed.

## Evidence and protocol

[Artifact retention policy](evidence/README.md) and
[complete summary tables](evidence/wildlife/tables.md) retain successful and rejected
experiments. Regenerate tables with:

```sh
python3 scripts/wildlife_report.py > docs/evidence/wildlife/tables.md
```

Trials use terrain/ecology edge resolutions 32/16 and 64/32, seeds 17, 81, 256 and
held-out seed 409, one terrain epoch, then ecology-only evolution. Each matched pair
starts after the same initial ecological year; only the open-barrier flag changes.
Three early trials listed in the manifest used different barrier settings during that
initial year and are preliminary rather than strict counterfactuals.

The evaluator arguments are additional months, terrain resolution, predator/bird food
half-saturation, aquatic grazer/river half-saturation, producer-feeding guild intake,
river intake override, terrestrial intake override, and optional seed. Omit trailing
arguments to use bundled settings and seeds 17/81/256. For example:

```sh
# Matched default comparison, 100 additional years.
mise exec rust@1.89.0 -- cargo run --release --example wildlife_evaluate -- 1200 32
# Experimental aquatic intake; NOT the bundled default.
mise exec rust@1.89.0 -- cargo run --release --example wildlife_evaluate -- 1200 32 0.000005 0.001 2.4 2.4 0.8
# Held-out larger grid and long-run stress test of that same experimental setting.
mise exec rust@1.89.0 -- cargo run --release --example wildlife_evaluate -- 1200 64 0.000005 0.001 2.4 2.4 0.8 409
mise exec rust@1.89.0 -- cargo run --release --example wildlife_evaluate -- 6000 32 0.000005 0.001 2.4 2.4 0.8 17
```

Outputs include before/after inventories, occupancy, ancestry, feeding rates, config,
budget residuals and stage timings. These small grids test mechanisms; they are not
resolution convergence evidence or full-resolution performance benchmarks.

## What worked

- Zero populations stay absent without an occupied source, including after a global
  removal/restoration scenario. Restore enables recolonization; it does not create founders.
- Fine-edge fixtures block false coastal bridges, permit actual connected immigration,
  match CPU conductance references across cube seams and preserve mass-weighted ancestry.
- At 100 years in the aquatic-intake experiment, central terrestrial guilds have no
  outer-founder ancestry, while central waterbirds have approximately 85–98%. Movement
  opportunity, rather than an island productivity multiplier, creates that distinction.
- The experimental aquatic intake yields lake predator shares of 4.19%, 3.23% and
  2.70% for seeds 17/81/256; held-out seed 409 gives 4.46%. The denominator includes
  both aquatic grazer and migratory river prey, not just the small grazer guild.
- Maximum C/N/P relative residual is below 9e-6 in those century trials. The 500-year
  pair remains below 5.7e-5. Conservation does not imply a healthy food web.

## What failed, and the resulting decision

Lowering predator half-saturation alone did not restore a substantial food web.
Increasing every producer-feeding guild's intake favored a few dominant guilds;
reducing river-animal intake instead produced a predator-heavy endpoint after prey
decline. Raising only aquatic producer-feeding intake looked better at 100 years.

However, at 500 years that last trial loses all central terrestrial guild occupancy
above the reporting threshold. Small aquatic grazers disappear while river animals
dominate; predator share falls to roughly 0.00046%, despite about 186 million kg of
predator carbon remaining. Both absolute stocks and shares matter. **No experimental
intake or half-saturation changes were promoted to the bundled catalog.** These runs
are evidence of unresolved trophic calibration, not accepted ecological targets.

One actual units error was fixed: `feeding` is maximum kg food C per kg consumer C
per year, not a 0–1 efficiency. Validation now permits a finite 0–12 annual rate
(an explicit model bound), leaving assimilation constrained to 0–1. A GPU analytical
fixture explains why this matters. For food P:C=0.001 and consumer P:C=0.015, gross
P-limited growth cannot exceed `feeding × 0.001/0.015` per year. At intake 0.8 this
cannot cover maintenance 0.1 plus mortality 0.02, even with abundant food. The fixture
matches the actual monthly update:

```text
C_next = (C + C × feeding/12 × 0.001/0.015)
         × (1 - 0.1/12) × (1 - 0.02/12)
```

It verifies decline at 0.8 and growth at 2.4 without changing the nutrient ledger.
This licenses meaningful future rate calibration; it does not justify applying 2.4
to every guild or claiming it is a measured biological rate.

## Economic feedback

Fisheries previously recognized only aquatic grazers. They now draw sequentially
from grazers, migratory river animals and aquatic predators, with catchability factors
1, 0.25 and 0.1 and **one shared** worker catch budget. These factors are game-design
assumptions. Catch transfers existing C/N/P through the existing harvest/exchange
ledgers; ancestry is retained proportionally in the remaining stock. A controlled GPU
fixture with no aquatic grazers confirms a river-animal fishery works and balances.

## Next calibration gate

Before adding species names or body-size evolution, resolve intake, prey stoichiometry,
maintenance and low-density predation together using isolated producer–consumer and
multi-prey fixtures. Then require century and 500-year held-out runs. Habitat refuges
or density-dependent prey switching are candidates, not universal fixes: research on
[marine predator functional responses](https://pmc.ncbi.nlm.nih.gov/articles/PMC7013479/)
and [optimal prey switching](https://pubmed.ncbi.nlm.nih.gov/36416056/) supports dependence
on predator traits and context, rather than applying a sigmoid to every animal.

Persistent populations within guilds and connected components inside coarse cells
remain necessary before claiming per-island endemism. Current ancestry is a neutral
source tracer, not speciation, adaptation or a richness measure.
