# Stand-alone agent-based economics experiment

The goal is a consistent economy built from generic agents, explicit agreements
and planning from needs and available opportunities. Persons remain individuals;
organizations coordinate real members and delegated resources without duplicating
population, labor or wealth. Start with small scenarios and rebuild broader
capabilities on these shared primitives.

The [project goals](GOALS.md) set the direction: lawful formation, immutable
constitution templates, initially static charter parameters, governance by persons,
swappable decision policies,
contributed labor, local bid/ask marketplaces and explicit estates. They distinguish
ownership, membership, governance and valuation. These are goals, not implemented
features; the [integration matrix](INTEGRATION-STATUS.md) records current support.
Physical-world expansion and autonomous state planning remain later priorities.

Optional [external telemetry](TELEMETRY.md) exports metrics and committed-event
logs from scenario runners without instrumenting agent or settlement code.

Status: the process-based simulation is implemented and runs on CPU.
The individual [opportunity marketplace](OPPORTUNITIES.md) now links food/warmth
needs to cultivation, state land agreements and wood collection, with state-defined
transaction permissions per agent type and [citizenship membership](CITIZENSHIP.md).
The state posts citizenship; accepting it enables land agreements and cultivation. Run `cargo +1.92.0 run --locked -- opportunity-farming`
from this directory.
An opt-in household scenario adds agreement-formed collective agents, pooled
income/storage, shared shelter, member tax/forward support, and spare-labor decisions. This is the existing pilot; the target redesign uses
explicit labor contributions and constitutional governance, as described in the goals.
A controlled specialization fixture adds mining/refining, tool creation and repair,
fishing, livestock, plot-attached housing and annual commodity-or-coin taxes.
One-person and four-person scenarios plan repeated harvests and meet nutrition/warmth needs using generic
process definitions, state-owned plots and dated use rights. The broader architecture remains a
proposal; general markets, general-purpose finance and population demographics are not implemented.
The 32-person trading fixture now uses upfront coin tool purchases and state
advances against projected future commodity production, with finite treasury funding.
A finite state offer now exchanges grain for a durable labor-saving tool.
The storage experiment adds finite shared stores, a two-grain annual tax, treasury
token issuance and voluntary grain sales to the state. Tokens require no storage.
Foraging now provides an alternative food through finite shared supply; consumption
and planning support substitutable foods.
Generic requirement consequences now connect deprivation to capacity and terminal
lifecycle states, with a separate institution-upkeep fixture using the same machinery.

The central question is: **can the same kind of decision-maker produce, consume,
trade and coordinate resources without separate simulation code for every social
role?** A farmer, craft worker or trader should emerge from needs, resources,
rights, capabilities and selected processes, rather than require a different agent
class. Processes describe possible transformations; transactions record outcomes.

- [Verification stress test](VERIFICATION-STRESS-TEST.md): staged evidence from minting and markets through institutional finance and alternative civilizations; experimental targets, not final-game content.
- [Reciprocal grain and wood markets](RECIPROCAL-MARKET.md): two books share money/storage; per-good purchase policies and observations.
- [Production and market planning](PRODUCTION-MARKET.md): four people compare work, buying and waiting; repeated crops, observed demand and trade/no-trade controls.
- [Four-person town market](TOWN-MARKET.md): local admission, competing bids/asks, monthly prices and volume, with fixed or ZIP quotes.
- [Need-generated orders](NEED-ORDERS.md): consumption deficits and protected surplus create recurring bilateral orders, with concession or ZIP pricing.
- [Integration status](INTEGRATION-STATUS.md): shared credit/exchange reservations, permission checks, planning contracts and an explicit compatibility matrix.
- [Joint production and sale planning](JOINT-PLANNING.md): bounded multi-cycle work/sale alternatives with dated execution and explicit scarcity fallback.
- [Bounded sale planning](SALE-PLANNING.md): forecast sale quantities against dated production and needs, with explicit horizon limitations.
- [Repeated-credit audit](REPEATED-CREDIT.md): cultivation-right expiry explains missed meals; six harvests over 60 months after correcting the fixture.
- [Production-funded credit](PRODUCTION-FUNDED-CREDIT.md): finite grain bids, protected food reserves and harvest-funded mortgage installments.
- [Borrowing decisions](BORROWING-DECISIONS.md): accept/decline forecasts over needs, labor and actual coin payments.
- [Shared forecast context](FORECAST-CONTEXT.md): common observation rules for search, work choices and resale valuation.
- [Structure review](STRUCTURE-REVIEW.md): historical findings and links to implemented follow-ups.
- [Collateral resale](COLLATERAL-RESALE.md): pending sales, buyer valuation, actual proceeds and a possible borrower-redemption extension.
- [Remaining-value work choices](REMAINING-VALUE.md): maintain an inherited crop, choose wood collection or wait using bounded forecasts.
- [Secured credit](SECURED-CREDIT.md): financed plots, monthly interest, balance sheets and fixed-value repossession with attached crops on CPU.
- [ZIP pricing](ZIP.md): persistent margin learning and repeated CPU comparisons with fixed/concession policies.
- [Marketplace agent](MARKETPLACE.md): person-only access, explicit grain/coin catalog and persistent participant pricing records.
- [Negotiated pricing](NEGOTIATED-PRICING.md): bilateral reservation prices, bounded quote concessions and atomic CPU exchange.
- [Consequence-based allocation](CONSEQUENCE-PRIORITY.md): urgent warmth versus tool investment, with controlled CPU outcomes.
- [Shared-access learning](ACCESS-LEARNING.md): realized-access estimates and optimistic/learned CPU comparisons.
- [Intermediary resource experiment](INTERMEDIARY.md): competing tool projections, atomic inputs, fallback work and CPU harvest outcomes.
- [Resource resolution](RESOLUTION.md): separate ranking from immediate or atomic bundle acceptance.
- [Monthly wood market](WOOD-MARKET.md): quantity allocation, urgency versus lottery, and scarcity consequences on CPU.
- [Contested offers](CONTESTED-OFFERS.md): generic allocation policies, open land applications and two-person CPU controls.
- [Production agreements and common offers](MARKET-AGREEMENTS.md): farming terms, atomic acceptance and planning across monthly commitments.
- [Shared agreements and consequences](AGREEMENTS.md): citizenship, land and production plus read-only loan terms, claims and enforcement views.
- [Swappable opportunity search](SEARCH.md): named strategies, common candidate plans, shared evaluation and search budgets.
- [Households](HOUSEHOLDS.md): adult agreements, pooled resources, shared dwelling services and household decisions.
- [State prices and tool economics](STATE-PRICING.md): 1.50 resale, 0.75 spot purchases, 0.50 forwards and productivity calibration.
- [Additional plots](ADDITIONAL-PLOTS.md): productivity-tested requests, finite land and per-plot annual taxes.
- [Upfront tool purchases and forwards](FORWARD-TRADING.md): coin prices, production projections and state financing.
- [Output-share control](TRADING.md): earlier fractional royalties and provider-count tuning.
- [Specialized activities](ACTIVITIES.md): catalogs, work targets, durable outcomes and CPU results.
- [Four people](FOUR-PEOPLE.md): three additional people and shared-resource limits.
- [Storage and currency](STORAGE-CURRENCY.md): capacity, annual issuance and grain sales.
- [Foraging](FORAGING.md): food substitution, immediate relief, and crop-work conflict on CPU.
- [Design](DESIGN.md): generic agents and persons, rights, multi-period processes,
  bounded planning, transaction effects and monthly settlement barriers.
- [Production audit](PRODUCTION-AUDIT.md): combined rent/tool/experience comparison,
  idle-crop diagnosis and a feasible surplus-producing work calendar.
- [Experiments](EXPERIMENTS.md): small scenarios, controls, observations and criteria
  for retaining or rejecting the design.

This is an independent Rust crate with its own manifest, lockfile and toolchain.
It does not depend on the main world generator, GPU, terrain, history or catalogs.
The only direct library dependency is [CubeCL 0.10.0](https://docs.rs/cubecl/0.10.0/),
with its CPU backend enabled. The test kernel and launch helper are generic over
CubeCL's runtime; CUDA and other backends can be selected in later work. They are
not enabled or verified by this CPU smoke test.

## Run the simulation

The new specialization fixture runs 36 months:

```sh
cd exp/economics
cargo +1.92.0 run --locked -- specialized-activities
cargo +1.92.0 run --locked -- specialized-32
cargo +1.92.0 run --locked -- households-32
cargo +1.92.0 run --locked -- trading-32
cargo +1.92.0 run --locked --example activities_audit
```

From the repository root:

```sh
cd exp/economics
cargo +1.92.0 run --locked -- four-person-exchange
cargo +1.92.0 run --locked -- four-person-scaled
cargo +1.92.0 run --locked -- storage-exchange
cargo +1.92.0 run --locked -- repeated-harvests
cargo +1.92.0 run --locked -- warmth-food-first
cargo +1.92.0 run --locked -- warmth-first
cargo +1.92.0 run --locked -- conditions-warmth-first
cargo +1.92.0 run --locked -- institution-recovery
cargo +1.92.0 run --locked -- forecast-harvest
cargo +1.92.0 run --locked -- forecast-cold
cargo +1.92.0 run --locked -- tool-beneficial
cargo +1.92.0 run --locked -- tool-food-risk
cargo +1.92.0 run --locked -- experience-manual
cargo +1.92.0 run --locked -- experience-tool
cargo +1.92.0 run --locked -- annual-access
cargo +1.92.0 run --locked -- annual-arrears
cargo +1.92.0 run --locked -- offer-short
cargo +1.92.0 run --locked -- offer-long
cargo +1.92.0 run --locked -- payment-protected
cargo +1.92.0 run --locked -- payment-trap-protected
```

These commands execute 60 modeled months using **CubeCL's CPU runtime** and
print a Markdown balance table plus need-specific decision/transaction trace.
The original `baseline` and its controls still run nine months. Use `--help` for
all 68 scenarios. Redirect traces to `../../output/economics/` when keeping
local runs.

Foraging controls run nine months:

```sh
cargo +1.92.0 run --locked -- forage-bridge
cargo +1.92.0 run --locked -- forage-conflict
cargo +1.92.0 run --locked -- forage-conflict-long
cargo +1.92.0 run --locked --example foraging_audit
```

Repeated harvests consume seed at planting and return seed at harvest. Seven crops
complete in 60 months, with no food shortage and an eighth crop underway. Adding
warmth with food-first priority also meets both needs throughout. Warmth-first
instead prioritizes fuel buffers over critical crop work and loses its harvest.
These original controls omit consequences. The `conditions-` variants add
persistent deprivation, gradual recovery, capacity penalties and terminal states.
`institution-upkeep`, `institution-recovery` and `institution-supplied` exercise the
same rules with upkeep and administrative capacity instead of food and labor.
The legacy policies do not anticipate condition effects. The optional
`ConsequenceAware` policy compares bounded forecasts using the same process and
condition evaluator, then commits only the selected current-month work.
`forecast-harvest`, `forecast-cold`, `forecast-resupply` and `forecast-scarcity`
exercise that policy; matched controls retain fixed priorities.
The cold fixture starts at month six and runs 60 months from that boundary. See [the CPU results](RESULTS.md)
for the paired-policy evidence and controls.

The six tool fixtures compare beneficial purchase, a matched no-offer control,
unsafe food spending, unavailable/exhausted equipment and last-use fallback. A
three-grain purchase buys six assisted harvests, each saving one labor unit. Wear
occurs only on completed use. These are fixed barter offers from a passive state,
not negotiated prices or an autonomous seller.

Experience fixtures award practice for completed harvests. Four points unlock
a one-labor manual technique; the fourth harvest still uses its original technique.
An experienced operator preserves the tool when manual work costs the same.
Forecasts include these gains. Practice belongs to the operator and cannot be
traded; missing inputs still prevent learning.

Annual-access fixtures start with an accepted agreement: one grain per year for
plot access, first due twelve months after activation. Due payments precede new
work. Arrears block new planting but allow existing crops to finish; committed
harvest output can clear debt before consumption. The fixed payment has priority
over consumption, so these fixtures explicitly measure resulting food shortages.
The separate `offer-` fixtures compare accepting posted terms with declining.
Acceptance activates access and dates the first bill twelve months later.
The horizon comparison holds the configured buffer at six months and extends
decision forecasts from six to eighteen months. Contract-linked candidates also
look ahead through production lead time. Negotiation remains unimplemented.

Payment fixtures compare default DebtFirst with opt-in ProtectEssentials under
identical opening resources and terms. Protection preserves current essential
consumption inputs from rent collection, leaving arrears that still block planting.
With dated production candidates, both normal runs meet all food provisions and
payments. In the scarce pair protection delays death by one month without restoring
production. See [current results](DATED-CANDIDATES.md) before treating a protected
payment budget as a sustainable solution.

Planning, rights checks, joint reservation, grouping and structural transitions
run in ordinary Rust. A runtime-generic CubeCL kernel gathers integer effects
into candidate account balances, which the host validates before publication.
The CLI always selects the CPU runtime; tests also use a Rust reference gather
and a separate raw-effect balance audit. This is a correctness experiment,
not a performance benchmark or a fully device-resident simulation.

## Validate the simulation and compute setup

From the repository root:

```sh
cd exp/economics
cargo +1.92.0 test --locked
cargo +1.92.0 fmt --check
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
```

Rust 1.92.0 is pinned locally because CubeCL's dependency graph requires a newer
compiler than the main application's Rust 1.89.0. The explicit toolchain in these
commands also works when a shell sets `RUSTUP_TOOLCHAIN=1.89.0`, which overrides
directory toolchain files. Install it if needed:

```sh
rustup toolchain install 1.92.0 --profile minimal --component rustfmt --component clippy
```

The first build downloads Rust dependencies and the CPU backend's LLVM/MLIR bundle
through CubeCL's `tracel-llvm` dependencies. Their bundle cache is outside the
repository under `~/.cache/tracel`. A working native compiler/linker and network
access are needed for initial setup; no CUDA toolkit or GPU is needed for this test.

The test uploads five `f32` values, dispatches an add-one kernel across eight units,
reads the output back through the CPU runtime, and compares all results exactly.
Negative, fractional and zero inputs exercise arithmetic; the excess units exercise
the tail guard. This checks macro compilation, CPU kernel compilation/execution,
buffer transfer and readback rather than only importing the dependency.

Verified on Linux x86_64 with Rust 1.92.0: the full regression run passed 102
tests, followed by all six final exchange tests (including one additional
provider-selection test). Formatting and strict all-target Clippy passed.
The new 72-month trading scenario matches CPU/reference state, ledger and reports
exactly; see [the provider-count comparison](TRADING.md#observed-72-month-comparison).
Earlier controls remain covered by their regression tests. CUDA and other backends
remain unverified.

## Design and artifact boundaries

The architecture is transaction-first: agents read committed state and emit
intents; resolution stages reserve resources and produce transactions; grouped
effects are validated and committed at explicit monthly boundaries. The implemented
boundaries are Open, productive process execution (including planning/reservation),
consumption and Close. Worlds with equipment offers or stock bids add Acquire after Open: barter
settles before the dated productive plan executes. Agreement worlds add Due after
Open and ClearArrears after Productive. General market stages remain empty.

The two-link planner connects each need to consumption and a producing process,
scores dated shortfall reduction and requests at most one new productive instance
per need per participant per month. Duplicate producers selected by multiple needs are
collapsed. Explicit need ranks or continuation priority allocate the shared budget;
this heuristic does not jointly optimize all need chains. The forecast policy
compares continuation-first and each-need-first rollouts, with optional deferral
of new work this month. Equipment offers add buy-now versus keep-food alternatives,
with physical wear and manual fallback included in each forecast. It supports one
participant, at most four requirements and four posted offers;
it does not search all possible action sequences. Future fixture shocks are
excluded from its observations. Processes execute from catalog data; there is no
occupation switch or farming-specific phase. The catalog and initial scenarios
are editable in [scenario.rs](src/scenario.rs). Process failure currently means
abort with sunk inputs retained and occupancy released. More failure policies,
weather, multi-need optimization and file-based checkpoints are future work.

Committed records retain effects, before/after process transitions, dated provision
receipts, condition changes and terminal transitions. Close evaluates consequences;
next Open applies the most restrictive capacity modifier. Parameters are abstract
fixture values, not calibrated human or institutional survival thresholds. They support
replay from the same initial world, plus in-memory continuation at every barrier.
They are trusted internal operation records, not an external transaction API.

Use the main project as a source of lessons, not as a required runtime dependency.
In particular, preserve finite resources, explicit allocation, dated decisions and
requested-versus-completed accounting. The experiment does not promise numerical
parity with Ancient World or replace its monthly coordinator.

Keep source, editable scenarios, documentation and curated Markdown findings here.
Generated runs, checkpoints, traces and build artifacts should go under the
repository's ignored `output/economics/`, outside this source directory. The local
Cargo configuration directs build output there when run from this directory.
If invoking Cargo elsewhere with `--manifest-path`, explicitly pass
`--target-dir output/economics/target` from the repository root because Cargo does
not discover the manifest directory's configuration from that invocation. This
follows the [repository artifact policy](../../AGENTS.md).

Dated payments now inform production candidates; see [the CPU comparison](DATED-CANDIDATES.md).

Shared settlement primitives and migration scope: [FINANCE.md](FINANCE.md).

Production and reciprocal-market scenarios now use a shared six-month buying and
selling horizon by default. See the [horizon comparison](ORDER-HORIZONS.md);
`ORDER_HORIZON=legacy` retains the old 6/2 diagnostic control.

The [two-person calibration](CALIBRATION.md) compares autarky, directed exchange,
and optional counterparty-expectation / plan-persistence variants under the same
6/6 rules. Run `cargo +1.92.0 run --locked --example calibration` with
`TELEMETRY_DIR` set to a fresh directory under ignored `output/`.

[Harvest-funded loan stress controls](CREDIT-STRESS.md) compare normal repayment,
a lost harvest followed by recovery, and repeated harvest failure with crop-preserving
repossession. The CPU runner exposes monthly balance sheets and settlement receipts.
