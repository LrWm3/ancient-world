# Persistent site assets: dry storage

The first site-asset increment separates dry-storage capacity from current
population. It uses two fixed vec4 records in the existing GPU Economy buffer;
there is no second inventory of buildings or their materials.

## Yards and warehouses

At the first enabled production boundary, a site records its existing abstract
yard allowance: current population times `storage_kg_per_person` (default 100 kg).
This baseline is open staging space, not a material-backed warehouse or an
arrival shipment. It is recorded once and never resized after deaths, migration,
abandonment or reoccupation. New daughter settlements establish their own
baseline. The baseline event describes a recorded allowance, not witnessed
construction. Baseline yards do not weather in this increment.

Additional warehouse capacity requires 0.02 kg timber and 0.03 kg bricks per kg
of capacity, plus 0.002 worker-months per kg. These are regional game-scale
coefficients, not architectural dimensions. Building and repair share at most
10% of available monthly craft work after cultural/research reservations.
Work used here is unavailable to industrial workshops and ordinary recipes.

The planner targets 25% headroom over actual dry inventory plus incoming cargo.
The target is capped at twice the population-based planning allowance, or the
original yard allowance if larger. This cap limits requests; it does not delete
existing warehouses when population falls. Speculative production orders alone
cannot justify expansion. Materials are requested through the existing recipe
planner and purchased through existing markets.

Installed timber and brick each wear at 0.1% per month plus up to 2% from
waterlogging/cleanup exposure. Warehouse capacity is limited by the scarcer
remaining structural material. Replacement restores useful capacity only when
stock pressure still justifies it; idle warehouses may decay without repair.
Decay also runs for empty settlements. Embodied material becomes recorded waste
and C/N/P detritus. Construction transfers stocked goods into installed material;
it does not mark them consumed a second time.

The resulting capacity feeds existing extraction, craft expansion and cargo
reservation limits. Shrinking capacity does not erase goods already present:
an overfull town stops accepting additional dry inventory under existing rules.
There is no new exposure-loss mechanism for dry goods yet. Food granaries and
their population-based spoilage model are unchanged.

## History, inspection and compatibility

Sites record a baseline, establishment at 100 kg of warehouse capacity, and
subsequent changes exceeding both 25% and 100 kg relative to the last report.
Deterioration and expansion events link to the previous storage event. These
thresholds avoid monthly construction spam.

The explorer shows baseline yards, warehouse capacity, installed timber/bricks,
and cumulative wear. History evaluation samples include per-site materials,
capacity and construction work.

The archived catalog's `production.persistent_storage` flag defaults false
when absent; bundled new-world settings enable it. Older economy records default
to zero storage assets. Explicitly enabling the flag through
`Generator::configure_economy` records the baseline on the next monthly boundary
without inventing embodied materials. Existing material assets remain conserved
if the policy is disabled. The ordinary archive includes both GPU records and
the last reported event threshold.

Evaluator controls:

- `--no-persistent-storage`: original population-scaled capacity.
- `--storage-kg-per-person 10`: a tighter initial yard allowance.
- These controls should be set before history begins for paired comparisons.

## Scope

Housing, food-storage assets, waterworks, fortifications and unified ownership
remain later increments. Existing industrial workshops and institutional meeting
places keep their own inventories. This does not add streets or local geometry.

## Validation

The GPU fixture checks material and labor limits, construction blocked by missing
bricks, aggregate goods/CNP conservation, save continuation versus separate monthly
calls, decay without residents, and surviving assets during reoccupation.
Legacy-field defaults are covered separately. The existing economy suite checks
stock limits, production, wear and recipes with the enlarged GPU layout.

## First calibration

Seeds 17, 81 and 256 ran for 50 years at terrain resolution 64, one epoch,
16 founding civilizations, crop yield 0.33, and frozen environmental history.
Default 100 kg/person yards needed no warehouses: population outcomes matched
the preceding meeting-place evaluation. This avoids unnecessary construction.

At a tighter 10 kg/person allowance, the enabled model built warehouses.
The paired control used the same allowance with population-scaled storage.

| Seed | Warehouse sites | Final warehouse kg | Worn materials kg | Population, persistent / control |
|---|---:|---:|---:|---:|
| 17 | 20 | 15922 | 362.5 | 1848 / 2050 |
| 81 | 17 | 13882 | 322.4 | 1722 / 1844 |
| 256 | 18 | 17246 | 366.2 | 2372 / 2479 |

Persistent storage roughly doubled completed deliveries in these pressure runs,
but final population was lower in all three comparisons. This is not calibrated
as a prosperity bonus: retained and constructed capacity changes production,
trade, labor competition and subsequent political history. Further tuning should
examine these coupled effects rather than require a population increase.

The enabled pressure runs recorded 5–9 deterioration events per world alongside
establishment and expansion. Maximum relative conservation residual stayed below
1.43e-5 across the default and pressure runs. Reports are
`output/persistent-storage.json`, `output/storage-pressure.json`, and
`output/storage-pressure-control.json`.

49 tests passed across library, markets, economy and storage fixtures.
Formatting, whitespace and Clippy checks passed. The GPU fixture includes
same-backend checkpoint continuation with different advancement batch sizes.

## Housing and crowding

Housing uses another pair of fixed GPU vec4 records: installed timber/bricks,
baseline places and cumulative work; target places, worn mass, monthly additions
and policy. New worlds enable `production.persistent_housing`; old catalogs
without the flag preserve unrestricted housing behavior.

The first enabled month records basic shelter at
`initial_housing_per_person` times the resident population (default 1.1, validated
range 0.25–2). This is an explicit starting shelter baseline, not a material
shipment or fabricated construction event. Baseline shelter is retained after
population decline and does not weather yet. Later housing uses two kg timber,
three kg bricks and 0.2 worker-months per additional place. These are abstract
regional project costs.

The planner aims for 10% headroom over residents plus inbound relocating cohorts.
Housing has priority within the same 10% craft-work allocation as warehouses;
warehouses receive the remainder, and ordinary industry retains the rest of
craft labor. Both materials must be available. Installed housing decays at the
warehouse rate, including at empty sites, with the same goods/waste/CNP ledger.
Unneeded housing can decay instead of consuming repair materials indefinitely.

Crowding is the share of residents exceeding shelter capacity, bounded 0–1.
The existing social observations retain its memory: 18% adjustment toward
worsening exposure each month, 8% toward improving exposure. Above 15% remembered
crowding, an episode is recorded and the existing relocation departure gate can
open even without famine; below 5% the recovery event closes the episode.
Household preferences, leadership obligations, provisions, annual departure
limits, route access and food/land admission requirements still apply. Crowding
does not directly kill residents or multiply disease in this increment.

Destinations must have places for current residents, already committed inbound
households, and the proposed new cohort. These are reservations against real
capacity, not new population. If births or damage exhaust capacity before
arrival, the household follows the existing return journey, paying travel time
and provisions. Returning home is permitted even when crowded. No cohort is
deleted or stranded outside the existing transit system.

Housing baseline and capacity changes have site-linked events. Expansion and
deterioration need both a 10% change and two places relative to the last report.
The explorer exposes capacity, current crowding and materials; social inspection
exposes remembered crowding. Evaluation summaries include capacity, materials,
work, wear and crowding per site. Fixed records and event thresholds persist at
the existing monthly archive boundary.

Evaluator controls: `--no-persistent-housing` provides a legacy admission
control, and `--initial-housing-per-person 0.7` starts a deliberately crowded
scenario. Starting allowances are recorded once; changing this setting later
does not resize existing towns.

This does not introduce individual houses, property rents, homelessness health
effects, baseline shelter weathering or new eligibility for settling empty
ruins. Existing reoccupation paths retain surviving shelter when they operate.

### Housing evaluation

Seeds 17, 81 and 256 each ran 50 years at resolution 64, one terrain epoch,
16 founding civilizations and crop yield 0.33. Environmental history was frozen.

| Seed | Added places, default | Added places, crowded start | Crowding episodes / recoveries, crowded start |
|---|---:|---:|---:|
| 17 | 843 | 1481 | 16 / 16 |
| 81 | 631 | 1154 | 15 / 15 |
| 256 | 1019 | 1604 | 17 / 17 |

Default allowances produced expansion without sustained crowding episodes.
Starting at 0.7 places/person produced temporary crowding, construction and
eventual recovery in all three worlds; no site ended above 5% crowding.
These runs establish recoverability, not a calibrated disaster-housing model.
Maximum relative conservation residual stayed below 1.40e-5.

Reports: `output/housing-default.json`, `output/housing-pressure.json`.
Housing fixture coverage includes missing-brick construction failure, material
and work accounting, old archive defaults, persistent crowding events, exact
checkpoint/batch continuation, empty-site decay and reoccupation. Relocation
fixtures cover full destinations, reserved inbound places, crowding-only
departure eligibility and loss of shelter en route.

## Waterworks, domestic water and sanitation

New worlds enable `production.waterworks`; absent legacy flags default false.
Three fixed vec4 records track installed timber/bricks, construction and
operating work, target/service capacity, wear, actual coverage and domestic
water use. There is no initial free waterworks inventory.

One served-resident capacity requires two kg timber, four kg bricks and
0.2 worker-months. The planner targets a configurable fraction of the current
resident population (`production.waterworks_target_fraction`, bounded 0–1).
New bundled worlds target 50%; archives without this setting retain the prior
100% target. A zero target prevents new construction but retains domestic
water demand and health exposure; existing structures remain usable and wear
normally. This is an investment target, not guaranteed operating coverage.
Existing waterworks operate first within available craft labor, at 0.001
worker-months per served resident per month. The remaining craft pool supplies
the shared 10% construction allowance: housing first, then waterworks, then
warehouses. Work is
subtracted once; all remaining industry uses the remaining craft budget.
New waterworks enter service the following month.

Installed structures weather using the existing asset rate, even in empty
towns. Lost material enters the goods-used, waste and C/N/P ledgers.
Capacity is limited by the scarcer structural input and is not deleted when
population leaves.

Each installed resident-capacity adds one m³ to the existing rainwater-storage
limit. It creates no water. Enabled settlements withdraw up to 0.09 m³ per
resident monthly for minimal domestic needs before farming uses the remaining
reservoir. Withdrawals become domestic outflow in the existing water ledger;
the cumulative service counter is descriptive and is not counted again.
Shortfalls remain explicit. This is a regional proxy, not a potable-water
network or a detailed wastewater model.

Operating sanitation coverage is bounded by intact capacity, the fraction of
domestic water supplied and available operating work. The existing flood
contamination term plus a small current-crowding term is reduced by up to 75%
at full coverage. Domestic water shortfalls add a separate illness exposure.
Hunger's illness contribution and normal disease recovery remain unchanged.
No pathogen identities, waterborne transmission network or instant cure are
introduced. These coefficients are game-model assumptions.

Coverage crossing 25% establishes a waterworks event; three consecutive months
below 10% record disruption, and three months at or above 50% record recovery. Events identify measured
coverage, water shortage and installed capacity, linking to earlier waterworks
events. The explorer displays these measures; evaluation summaries include
materials, wear, construction work, operating work and domestic withdrawals.
State and service episode status persist at monthly archive boundaries.

`--no-waterworks` is a legacy control: it disables new construction, domestic
withdrawals and the new sanitation/crowding health pathway together. Therefore
paired worlds measure the whole feature, not sanitation efficacy in isolation.
The controlled GPU fixture keeps the feature enabled in both worlds and varies
installed structures to test sanitation's contribution under matching crowding.


Adaptive labor demand includes recurring waterworks operation even when ordinary
recipe orders are empty. The planner requests actual operating work within its existing workforce and
farm-protection limits. The construction allowance is calculated after
operation, avoiding reservation of ten times the work needed for service.

### Waterworks calibration

Seeds 17, 81 and 256 ran 50 years at resolution 64, one terrain epoch,
16 founding civilizations, crop yield 0.33 and initial housing allowance 0.7.
Environmental history was frozen. Final reports:
`output/waterworks-final.json` and `output/waterworks-control.json`.

| Seed | Installed resident capacity | Mean town coverage | Sustained interruptions | Population: enabled / legacy |
|---|---:|---:|---:|---:|
| 17 | 1934 | 76.2% | 0 | 1837 / 1981 |
| 81 | 1853 | 96.8% | 0 | 1632 / 1690 |
| 256 | 2197 | 92.2% | 0 | 2286 / 2359 |

Coverage is an unweighted mean across sites, not a population-weighted global
coverage estimate. No final site had a domestic water shortage above 1%.
Maximum relative conservation residual stayed below 1.40e-5. Population was
lower than the legacy control in these coupled runs; construction, operation,
crowding exposure, trade and political history all change. These results do
not establish a net demographic benefit or calibrated epidemic model.

Initial evaluation exposed repeated operating interruptions when adaptive labor
ignored service demand after recipe queues emptied. Recurring labor demand and
three-month event thresholds removed that behavior. Operation now requests
actual work, separately from construction, to avoid excessive workforce
reservation. A seed run also caught float roundoff after fully spending a labor
budget; the operation debit is capped at available work and its remainder is
nonnegative.

46 distinct library/economy/asset tests passed during this increment, with the
changed economy and waterworks fixtures rerun after the operating-budget fix.
The waterworks fixture covers four years of operation with empty recipe queues,
matching-crowding health comparisons, missing-water service failure, finite
construction, abandoned assets, old archive defaults, and exact same-backend
checkpoint continuation across different advancement batch sizes. Formatting,
whitespace and Clippy checks passed.

The subsequent [coverage calibration](waterworks-calibration.md) compares 18
paired runs and sets new-world planned coverage to 50%, with legacy targets
preserved and a zero-investment control retaining domestic demand.

The opt-in [recovery priority](waterworks-recovery.md) restores observed lost waterworks
capacity before spare housing when residents have shelter and domestic water.
