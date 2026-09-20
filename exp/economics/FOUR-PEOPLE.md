# Four people sharing a state and wild-food supply

Implemented CPU extension: three additional people, four people total. The original
one-person fixtures remain controls. People 88–91 each receive five grain, one seed,
one fuel, forty raw wood, two labor per month, nutrition/warmth requirements and a
15-unit store. Each has a separate state-owned plot, use right and two-grain annual
tax agreement. Issuance remains one treasury token per two grain actually collected
on each annual installment; people can sell grain for existing treasury tokens.

## Allocation and execution

The monthly boundaries are unchanged. Open renews each person's capacity and the
shared wild supply once. Due collects taxes into the shared state store. Acquire
considers eligible seller subsets and validates their combined demand against the
same opening treasury tokens and receiving space. Each selected seller transfers
one grain for one token. Trades commit together; one dated productive plan then
executes against private labor/stocks and the shared foraging pool. ClearArrears,
consumption and Close retain their existing timing.

The bounded forecaster now accepts up to four participants, four needs each and
four posted offers. It compares a common priority, deferral and producer preference
for the group, scoring summed consequences across participants. It enumerates
seller subsets, not independent full planning policies for every person. It still
assumes no further acquisitions during a forecast rollout. This is a small
coordinated planning experiment, not yet four independently optimizing economies.

Scarce requests retain stable-ID tie breaking. Seller subsets likewise use sorted
IDs, with no trade preferred when outcomes tie. Reordering catalog records leaves
outcomes unchanged, but changing IDs can change which otherwise identical person
wins a scarce resource. Determinism does not establish fairness.

## Run and compare

From `exp/economics`:

```sh
cargo +1.92.0 run --locked -- four-person-exchange
cargo +1.92.0 run --locked -- four-person-scaled
cargo +1.92.0 run --locked --example multi_person_audit
cargo +1.92.0 test --locked --test multi_person
```

Both scenarios run 60 months on CubeCL CPU. `four-person-exchange` keeps the original
32-unit state store, six-unit wild-supply pool and one-unit monthly regeneration.
`four-person-scaled` multiplies those shared capacities, opening wild supply and
regeneration by four; private endowments, tax rates and planning remain identical.
The scaled arm is a combined shared-capacity control, not a separate estimate of
each resource's effect.

The unchanged-capacity run produces identical totals for each person: seven
harvests, five forages, six of eight grain paid in tax, two tokens held, two units
of unmet nutrition and no unmet warmth. Nobody reaches the terminal condition.
The state finishes with 32 grain and four of twelve issued tokens; eight grain
were purchased from people.

The state store fills before the month-49 tax collection. With no room to receive
payment, the annual bills remain unpaid and the existing arrears rule blocks new
planting. This exposes a policy limitation: creditor storage failure penalizes the
debtor. No special exemption, discard rule or automatic expansion has been added.

The scaled-capacity control completes nine harvests and one forage per person,
pays all eight grain of tax per person, and has no food or warmth deficits. Each
person holds two tokens. The state finishes with forty grain and eight tokens:
thirty-two grain collected in tax, sixteen tokens issued, and eight grain purchased.
Both 60-month runs match full Rust-reference state, ledgers and reports exactly.

| Shared capacities | Harvests per person | Food / warmth deficit per person | Total tax collected | Total tokens issued | State grain |
| --- | ---: | ---: | ---: | ---: | ---: |
| Original | 7 | 2 / 0 | 24 | 12 | 32 |
| Scaled fourfold | 9 | 0 / 0 | 32 | 16 | 40 |

These symmetric fixtures do not establish fairness with unequal needs, stocks or
rights. The immediate modeling question exposed by the original-capacity run is
how to treat a due payment that the creditor cannot receive.

## Verification and limits

Four focused tests cover separate endowments and rights, rejection beyond the
four-participant forecast bound, joint treasury/storage limits, stable tie winners,
shared-pool exhaustion and exactly-once regeneration. Both scenarios also run for
14 months with CPU/reference comparison, reversed catalogs, monthly versus batched
execution, full ledger replay and resumption from the month-13 dated production
plan. Selected current-month forecasts match every participant's actual outcomes.
The 83 existing tests also pass, for 87 total; formatting and strict all-target
Clippy pass.

The harvest-pressure control offers four equally eligible sellers: two treasury
tokens admit two sales and two harvests; a single free treasury storage slot admits
one sale and one harvest; no treasury tokens admit no sales. Losing harvests retain
the existing abort-on-failure semantics. Shared foraging separately allows only
one of four requests to consume a single remaining wild-supply unit.

No new agent-specific production logic, scheduler phases, person-to-person trade,
independent strategic policies or storage expansion is introduced. The group
forecast enumerates more alternatives and is intended for this bounded experiment;
no throughput or large-population scalability claim is made.
