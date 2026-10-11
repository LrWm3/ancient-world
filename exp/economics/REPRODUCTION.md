# Abstract reproduction and population pressure

Implemented as an opt-in person lifecycle pilot, outside the historical bounded v1
release. The purpose is to let new people enter the existing economy and expose
resource/support constraints. It is not a calibrated human demographic model.

## Rules and boundaries

`World.demography` holds the configuration; `State.demography` records births,
parent links, maturation and birth cooldowns. Each child is a distinct agent and
participant with ordinary needs, condition rules, stock accounts and finite
storage. A child starts with zero labor. At the configured maturity date, Open
activates the adult capacity from the template. Deprivation and terminal outcomes
use the existing condition model; dead children do not mature or restart.

At Open, the resolver first supports existing living dependents from their
parent's stored resources, retaining the configured parent reserve. Requests are
capped by the child's current stock deficit, the donor's available stock and
storage. `SupportOrder::OldestFirst` preserves stable child-ID priority by default;
`Rotating` rotates each parent’s sibling order by month. Receipts retain the policy,
opening spendable stock and child order. Unfillable or zero-demand requests leave
food for the next sibling. This changes allocation, not monthly timing. Then eligible adults
may reproduce, one birth per parent per month, in stable parent-ID order:

- The parent is living, mature, outside the cooldown and has no recorded deprivation.
- Its remaining stock covers the birth buffer plus its own reserve.
- The configured lifetime birth ceiling has not been reached.

The birth buffer is an actual transfer from parent to child. Nothing mints food,
coins, land or rights. Births do assign a configurable personal storage capacity,
just as the initial person template does; this pilot does not model building a
child's storage. The configuration can admit mature descendants as parents, from
the month after maturation. Turning births off retains support and maturation of
existing children.

Household opening transfers settle before the demographic boundary. Opening
credit outflows reduce the stock available for support; credit inflows cannot be
spent again on support at that same boundary. Birth/maturity registrations are
staged with the Open batch and published atomically. Support appears as paired
resource transfers and donor transfer expense/recipient transfer income in the
existing separate double-entry books, using transferred carrying cost.

Children can consume through ordinary processes. The shared permission check
rejects their productive, contracting and formation actions, including productive
recipes that happen to require no labor. They do not become adult household
signatories, governors, citizens or marketplace participants automatically.
Maturity supplies labor capacity, not institutional rights or consent.

`demography::replay` replays each batch with its chronological World and State,
staging household opening transfers and catalog registrations before settlement.
Using the final population catalog as the initial historical catalog is incorrect.
Checkpoints must retain World as well as State. Forecasts preserve known children,
support and maturity but suppress hypothetical future births; a forecast cannot
silently enlarge the eventual live population.

## Controlled CPU comparison

Run from `exp/economics`:

```sh
cargo +1.92.0 run --locked --release --example reproduction
cargo +1.92.0 test --locked --release --test demography
```

The comparison starts with two adults and six grain each. Each living person
needs one nutrition per month. A one-month foraging process uses one labor and one
unit from a shared wild-food pool to produce three grain. Personal storage is
12 grain. The pool holds at most eight units and regenerates either eight units
per month (abundant) or one (scarce). Both cases use the same ordinary planner,
initial stocks and allocation rules.

Reproduction transfers two grain at birth and subsequently tops the child's
stock up to one grain per Open, retaining one grain for the parent. Birth cooldown
and childhood each last six months. These are deliberately accelerated test
settings, not human ages. The lifetime ceiling is twelve births, including people
who later die. Each unmet nutrition unit adds one deprivation, a met month
recovers one, and six accumulated deprivation causes death. Impairment does not
reduce labor in this control.

Results after 36 completed months:

| Supply | Births enabled | Births | Living people | Deaths | Unmet nutrition units |
| --- | --- | ---: | ---: | ---: | ---: |
| Abundant | No | 0 | 2 | 0 | 0 |
| Abundant | Yes | 12 | 14 | 0 | 14 |
| Scarce | No | 0 | 2 | 0 | 0 |
| Scarce | Yes | 10 | 5 | 7 | 59 |

Twelve of the unmet nutrition units in each reproduction run occur during
childhood; the remaining two (abundant) and forty-seven (scarce) occur in adults.
At month 36 Close, the abundant run has fourteen adults; the scarce run has four
adults and one child.

The abundant population reaches the configured birth ceiling; it is not evidence
of a natural equilibrium. Scarcity changes births, survival and accumulated need
shortfall. The unchanged two-adult controls survive both supply settings.

Abundant supply does not guarantee access for children. Support uses stored food
at Open, before this month's production, and retains the parent's own reserve.
The legacy policy used in this table does not jointly plan the family's future
food/support commitments. This comparison therefore exercises a deliberately
simple support policy, not reliable family provisioning. Stable allocation order
also matters under scarcity; these runs do not establish fair access.

## Optional support-aware planning

`Config.plan_support` opts into known-child provisioning. Dated forecasts debit
support at Open before hypothetical production, respect the reusable parent
reserve and stop at maturity. They offset children's actual stocks and permitted
food substitutions; no future birth is assumed. A parent can also request ordinary
productive work specifically for support without acquiring a fictitious personal
need. Rights, labor, inputs and storage still determine actual completion.

Alternative-food planning and live consumption share support earmarks. Voluntary
sales conservatively protect projected top-ups plus the parent floor once, in
addition to personal consumption and accepted claims. This may retain more than a
minimal dated budget. Support is not a new financial liability or escrow. Need
orders and bounded lot selection may buy missing support stocks, even for a parent
with no personal demand for that resource; money, market access and storage remain
necessary. These changes do not alter the legacy comparison above.

## Household and institutional integration

A static household `Charter.support_dependents` may include living members' children
in need reservations, collective purchases, contributed-labor welfare comparisons
and voluntary-surplus decisions. It defaults off. Children remain outside the adult
roster: they donate no labor, do not vote or sign, and do not contribute shared
storage. Their own finite storage still limits delivery. Parent support planning
also lets household inputs fund ordinary production for children. Early voluntary
payment funding checks dependent needs without changing existing creditor rights.

At maturity, household accession still requires an explicit agreement and consent;
citizenship similarly requires accepting the state's offer. Historical admission
checks reject pre-maturity signatures even after a person grows up. Mature labor
alone grants neither institutional membership nor land. Worker protection with a
two-month horizon can retain time for the next Open's child support; the one-month
control misses that consequence. Available spare hours still permit paid work.

## Support-planning comparisons

```sh
cargo +1.92.0 run --locked --release --example reproduction -- --support-matrix
cargo +1.92.0 run --locked --release --example reproduction -- --support-matrix --overlapping-cohorts
```

Both matrices run 36 months on CPU with audited books. They hold initial resources
and rules fixed within each comparison. The second fixture changes childhood from
six to twelve months, while retaining the six-month birth interval; it is a separate
overlapping-cohort stress test. Support totals include actual birth buffers.

| Childhood | Supply | Support planning | Births | Deaths | Living | Nutrition deficit | Childhood deficit | Support delivered/requested |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 6 months | Abundant | Off | 12 | 0 | 14 | 14 | 12 | 60/72 |
| 6 months | Abundant | On | 12 | 0 | 14 | 0 | 0 | 72/72 |
| 6 months | Scarce | Off | 10 | 7 | 5 | 59 | 12 | 44/56 |
| 6 months | Scarce | On | 8 | 7 | 3 | 45 | 2 | 46/48 |
| 12 months | Abundant | Off | 12 | 0 | 14 | 38 | 38 | 86/124 |
| 12 months | Abundant | On | 12 | 0 | 14 | 0 | 0 | 144/144 |
| 12 months | Scarce | Off | 6 | 4 | 4 | 45 | 22 | 42/64 |
| 12 months | Scarce | On | 6 | 4 | 4 | 36 | 5 | 61/66 |

Oldest-first and rotating support produce identical aggregate outcomes in these
fixtures. Focused identical-request scarcity tests separately verify that rotation
changes the recipient and alternates by month. Neither observation establishes a
survival benefit from rotation. Every no-birth control retains two living adults
with zero nutrition deficits under either planning or allocation policy.

Planning removes the abundant-world deficit and reduces missed child support under
scarcity. Death counts under scarcity do not improve; birth counts and timing can
change, so total deficits are not a per-child welfare comparison. In the overlapping
abundant fixture, planning brings births forward enough that all descendants mature
by month 36, whereas six remain children without it. The fixed birth ceiling still
precludes any claim of natural equilibrium. Raw CSVs are local ignored outputs;
this table is the source-controlled result summary.

## Verification and limits

On 2026-10-11, the full release suite passed **1,255 tests, zero failed, one
ignored**. All twelve focused reproduction tests passed, as did strict all-target
release Clippy, formatting and repository artifact checks. The ignored 32-person
annual accounting integration test also passed separately.

Focused controls cover CPU/reference equality, conserved support, separate
financial statements, exact maturity, deprivation/death, insufficient birth funds,
cooldowns, zero-labor productive recipes, catalog ordering, tampered receipts and
checkpoints, atomic ID-collision failure, chronological replay, checkpoint/batch
continuation, loan repayment, and household wind-down funding a birth at the same
Open. An opening credit-transfer control checks that funds cannot be spent twice.

This first slice does **not** implement mating, sex, genetics, gestation, fertility
preferences, birth labor costs, aging deaths, multiple guardians, adoption,
inheritance or automatic estates. Terminal agents retain their remaining property;
the example’s remaining-food total includes those accounts. It is a deterministic
material gate, not a
reproductive decision made by the person's planner. One configured support stock
and one parent are supported. Births do not reserve future support and the parent's
reserve is not a general priority rule protecting all future debts.

The household adapters above cover known children, not adoption, guardianship after
parental death, automatic grown-child accession, or a separate child pooling policy.
Existing worker/private support protections do not make every planner family-aware.
Enabled endogenous discovery remains explicitly rejected until its dynamic
registration/admission adapters handle dependent rosters. The standalone comparison
uses unlicensed foraging; focused tests separately exercise explicit citizenship
and household admission, not autonomous recruitment or land acquisition.

The next useful work is dependent-aware endogenous discovery and explicit guardian
policy, then demographic choice/realism if needed. Individual self-directed policy
changes remain deferred.
