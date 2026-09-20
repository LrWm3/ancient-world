# Specialized activities, durable production and work contracts

Implemented CPU experiment. Run `cargo +1.92.0 run --locked -- specialized-activities`
from `exp/economics`, or `cargo +1.92.0 run --locked --example activities_audit` for
compact results. The original four-person control and the new 32-person baseline
both run for 36 months. Recipes,
costs and yields are abstract catalog choices, not historical estimates.

## Generic model

There are no miner, farmer, herder or builder classes. The scenario assigns generic
work orders: maintain a stock target, create a target number of durable assets, or
repair assets below a condition threshold. Each order names an agent, process,
priority and target. Existing need-driven requests and these orders enter the same
bounded labor, material, plot and storage resolver. Specialized priorities are
configured, not learned; people can still select subsistence processes.

The new `Activities` catalog extends processes with durable creation/repair outcomes,
mandatory productive assets, optional shared site access, expiring service tickets,
and contract-specific coin payment alternatives. Durables have kind, owner,
remaining service capacity, last-use date and an optional plot attachment. Kind
specifications declare maximum condition and optional monthly decay.

Validated completed process records authorize creation and repairs. Created asset
IDs derive deterministically from process IDs; collisions and arithmetic overflow
reject the batch. Inputs and labor are charged through ordinary ledger effects.
Creation becomes usable next month in this pilot, preventing new equipment from
funding another action in the same production batch. Replay derives exactly the
same assets from the same process records and catalog. Duplicate replay is rejected.

Optional techniques replace a stage's labor requirements while retaining its input,
output and duration rules. Mandatory assets are reserved separately, so a herd and
a labor-saving comb can both be required by the same action. A durable asset can
be used or repaired once per month. Repairs cap condition at its declared maximum;
a grinding stone cannot assist its own repair. Labor-only repair remains available.

## Activity catalog

| Activity | Inputs and output | Equipment |
| --- | --- | --- |
| Stone quarrying and clay digging | Finite shared deposit + labor → stone or clay | Stone-and-stick tool, shovel or pick reduce labor |
| Copper, iron, silver and gold extraction | Separate finite deposits + labor → ores | Same mining-tool techniques |
| Refining each metal | Ore + fuel + labor → refined metal | Separate from extraction; no automatic coin minting |
| Tool making | Stone, wood or metal + labor → durable tool | Hammer stones, soft hammers, punches and knives reduce crafting labor |
| Farming | Existing seed/labor/plot process → grain and replacement seed | Stone hoe assists stages with higher manual labor costs |
| Fishing | Regenerating shared fish supply + labor → food | Spear reduces labor; manual fishing is available |
| Raising livestock | Young stock + hay + labor on a plot → productive herd | Requires a starting animal, not spontaneous animal creation |
| Livestock work | Herd + hay + labor → milk and wool | Comb reduces the bundled collection work |
| Herd upkeep | Hay + labor → restored herd condition | Monthly condition decay applies even when idle |
| Home building | Stone + clay + wood + two stages of labor → attached home | Hammer stones, knives and shovels can assist |
| Tool repair | Labor → restored condition | Grinding stone reduces labor |
| Home maintenance | Clay + one labor → restored home condition | Low maintenance relative to construction |

All eleven requested initial tool types have creation and repair definitions:
hammer stone, soft hammer, punch, knife, stone-and-stick mining tool, shovel, pick,
stone hoe, fishing spear, livestock comb and grinding stone. Tool condition normally
allows 24 assisted actions; repair restores up to 12. Equal-cost techniques use the
existing deterministic tie break, so possessing several tools does not multiply
labor savings for a single action.

Workshop construction and equipment remain **TBC**, as requested. The durable
creation, attachment, required-asset and technique rules can express a workshop or
bench when its recipe and benefits are specified; none is silently required for
this first tool-making catalog.

## Homes and plots

A home belongs to the process beneficiary and attaches to the state-owned plot used
for construction. The plot contract remains separate from ownership of the building.
Ordinary portable-equipment offers cannot transfer an attached home.

Occupying a home requires the holder's current plot right and an available home on
that plot. It consumes one of 120 service months and produces a shelter ticket.
Consuming that ticket satisfies a new generic shelter need; unused tickets expire
at the next Open and cannot accumulate. Fuel-based warmth remains a separate need.
There is no climatic heat-loss model or automatic fuel saving from housing yet.

Occupancy is shared plot use: living in a house does not reserve the whole plot
against farming. Construction retains exclusive work-site use. Home maintenance
is shared plot use but reserves the home itself, so repair and occupancy cannot
use the same home that month. The configured maintenance order takes priority over
occupancy when condition falls below 24; one clay and one labor restore up to 24
service months. No maintenance is needed within the ordinary 36-month run.

Herd condition is an aggregate productive-asset abstraction, not an animal demographic
model. In this pilot the herd is attached to its managed plot. Condition loses one
unit each month plus one per production action; tending restores up to 12 for one
hay and one labor. Births, herd size, individual death, disease and grazing ecology
are not modeled. Zero condition disables production but is recoverable through
upkeep; it does not mean dead animals are revived.

## Annual work contracts and coins

The state still supplies plot use through the existing accepted agreements. This
fixture sets annual terms by configured work focus:

| Focus | Annual payment |
| --- | --- |
| Food production | Two grain |
| Extraction | Two stone |
| Tool making | Two coins |
| Livestock | Two wool |

Commodity contracts also accept one coin per native payment unit. The quote is
catalog data and can differ by contract; it is not a market valuation. Bills first
fall due in month 13, then every twelve months. This is a fixed access charge,
not a percentage of completed production. Switching work orders does not silently
rewrite an existing contract.

Settlement first transfers available native output that fits in the creditor's
store, then uses opening coin balances for remaining whole payment units. Coin
alternatives take no storage. A denomination that cannot cover a complete native
unit remains unpaid; settlement does not overpay or borrow. Agents do not yet choose
a strategic native-versus-coin payment mix.

Obligations record both units settled and native units actually delivered. Only
native grain collected under the grain issuance rule authorizes new tokens; paying
tax in coins transfers existing currency and cannot trigger issuance. The scenario
starts with four coins per person, explicitly endowed, and keeps grain-linked
issuance at one token per two collected grain. Gold and silver remain materials.

Coin fallback also works when the state store is full. Without sufficient native
receiving space or coins, arrears remain and the existing new-work restriction
applies, including starting a new occupancy process. No eviction/foreclosure policy
or exception for creditor-caused collection failure has been added. Candidate claim
forecasts still conservatively count the native obligation rather than optimize
its payment denomination.

## CPU results

All four people build homes by the end of month 2 and use them in months 3–36.
Each incurs two units of unmet shelter during construction, then recovers; nobody
has a food or warmth deficit or reaches a terminal condition. Each home finishes
with 86 service months remaining. Starting food, fuel, materials, six labor per
person/month and enlarged stores deliberately support this integration test.

| Activity | Completed in 36 months |
| --- | ---: |
| Stone / clay extraction | 5 / 3 |
| Copper / iron / silver / gold ore extraction | 7 / 8 / 8 / 1 |
| Copper / iron / silver / gold refining | 2 / 3 / 4 / 1 |
| Tool creation | 15 assets across all eleven types |
| Pick repair | 1 |
| Home construction / occupancy | 4 / 136 |
| Herd creation / upkeep / milk-and-wool collection | 1 / 3 / 16 |
| Fishing / hay cutting | 2 / 11 |

All eight annual bills are fully settled across two due dates. The state collects
four grain, four stone, four coins and four wool. Grain collection issues two coins;
total currency rises from 16 to 18, with six coins in the treasury. Commodity-paid
contracts do not use coin fallback in this ordinary run; focused controls exercise
full stores, missing goods, partial payment and different coin quotes.

Nine focused tests cover craft prerequisites and delayed availability, assisted
and manual repairs, repair exclusivity and caps, plot attachment and expiry,
concurrent housing/farming, service expiration, upkeep and feed requirements,
depletable deposits, refining without issuance, coin settlement, tamper rejection,
replay, reversed catalog order, month batching and mid-construction continuation.
The complete 36-month CPU state, ledger and reports match the Rust reference.

## Limits and next experiments

This is not a self-financing specialized economy. Specialists consume starting food
and fuel; produced tools remain with their owner until an explicit equipment offer
is configured. There is no automatic goods/tool marketplace or sales-based work
choice. Deposits are finite shared pools; geology, transport and local mineral
access restrictions are absent. Work uses generic plot rights, not a location-aware
mining concession system. Gold is reached late because of the configured order
priorities, not because agents discover a profitable opportunity.

Storage still covers configured stock resources; durable tool volumes and building
footprints are not counted in that store. Houses age by occupied service months,
not idle calendar time. Prices, nutrition yields and maintenance rates are abstract.
A useful next controlled extension would be finite sales of produced tools and
food, giving specialists a way to replenish their starting supplies before adding
independent specialization decisions or workshop recipes.

## 32-person baseline

Run from `exp/economics`:

```sh
cargo +1.92.0 run --locked -- specialized-32
cargo +1.92.0 run --locked --example activities_audit -- specialized-32
```

There are 32 people (IDs 88–119) plus the state: eight repetitions of the four
configured work profiles. Each person has private stocks, storage, labor, needs,
one state-owned plot and a separate annual contract. Profiles retain the same
two-unit annual taxes: grain, stone, coins or wool respectively. Non-coin contracts
retain their coin payment alternative. Only native grain tax receipts fund issuance.

Opening private endowments and work targets are unchanged per person. Shared
mineral deposits, wild food, fish and pasture supply, regeneration and state
storage scale eightfold against the four-person control. Supplies remain shared,
finite accounts, rather than copies available independently to every person.
State storage is 4,096 units; personal storage remains 256. This is an abundant
population baseline, not a fixed-land scarcity or allocation-fairness experiment.

The policy is still NeedFirst with configured work priorities. The bounded
ConsequenceAware planner retains its four-participant limit; this scenario does
not enlarge its joint search or give 32 agents independent forward planning.
There is no change to monthly timing, reservation or settlement.

### Observed CPU results

The 36-month CubeCL CPU run matches reference state, ledger and need reports exactly.

| Measure | Result |
| --- | --- |
| Food / warmth deficits | Zero for every person |
| Shelter deficits | Two units per person during initial construction |
| Homes | 32, each with 86 service months remaining |
| Terminal agents / overdue unpaid tax | Zero / zero |
| Tools created | 120 across all eleven kinds |
| Herds / husbandry completions | 8 / 128 |
| Fishing completions | 16 |
| Tax paid per contract | Four units, across the two due annual bills |
| Coins | 128 initially; 16 issued; 144 finally, including 48 held by state |

Every group reproduces the four-person control's ending stocks: the farming/fishing
profile has 40 grain; each other profile has four grain. Toolmakers have spent all
four starting coins on taxes; other people retain four coins. All mining and refining
families activate. Stable-ID allocation remains in place; equal outcomes here do
not demonstrate fairness under scarcity.

Starting food, fuel and materials still carry much of this run. Specialists neither
sell their output nor purchase each other's tools. The useful next experiment is a
small exchange between these existing people, with receipts tracking who can earn
enough to replace food and pay the next tax. This run alone does not establish
long-term viability or emergent specialization.

The regression checks both populations for CPU/reference parity, reversed catalog
and participant ordering, monthly versus batched execution, ledger replay and
continuation from a mid-construction boundary. Raw audit output stays under ignored
`output/economics/`.

The next implemented experiment is [specialist trading](TRADING.md): tools are
delivered against a share of realized output, using fractional stock quantities
and a measured provider-count comparison.
