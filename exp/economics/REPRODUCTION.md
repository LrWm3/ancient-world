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
storage. Existing children have stable child-ID priority. Then eligible adults
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

Household children, child-specific storage/pooling and growing-up admission remain
separate work. Existing member/worker objective scopes include known dependents
when comparing harm, but that does not make every household or person planner
family-aware. Enabled endogenous discovery is explicitly rejected with this
configuration until dependent-aware registration/admission exists. The comparison
uses an unlicensed foraging world; it does not demonstrate citizenship, recruitment
or land acquisition by grown children.

The next useful comparison is support-aware person/household planning against the
same opening resources, separating a shortage of food from food being produced
or transferred too late. Demographic realism can follow once that economic loop
works.
