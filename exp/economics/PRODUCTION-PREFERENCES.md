# Completed-production preferences

This follows the [input-buffer comparison](INPUT-BUFFER-PREFERENCES.md). A stock
reserve can penalize consuming inputs productively. This experiment instead adds
an explicit monthly completed-process target, without changing execution,
settlement, endowments, prices, permissions or readiness gates. It remains opt-in.

## Meaning and boundaries

`ProcessShortfall { definition, target }` measures missing completed batches of a
named process for each scoped operator in each completed forecast month. Its loss
is the sum of `max(target - completions, 0)` across those operators and months.
Surplus work cannot compensate for another operator's shortage or a missed month.
The target counts process instances, not output quantity, sales, profit or utility.
It is meaningful for a selected recipe; it is not invariant to splitting recipes
into smaller processes and is not a general seasonal production schedule.

Only committed completion transitions count, including employment transactions.
Active, failed and merely planned processes do not count. Attribution follows the
operator, not the beneficiary. Members are frozen at forecast opening so exit or
death cannot improve a forecast by removing a beneficiary. Unknown definitions
are rejected; a valid definition does not confer permission to execute it.

Ordinary and bilateral forecasts use the same dated scoring. Optional path evidence
distinguishes unobserved work from observed zero work. Finalization requires a
committed Close for every month. The live controller observes the previous completed
month; at founding, month one has no prior interval and therefore reports zero.
Restoring a later flow-enabled controller requires its ledger, as well as state
and world. Bare state-only measurement rejects this metric.

Discovery includes the named process and its prerequisites while retaining signed
authority checks. The monthly target does not invent a recurring work order or
reserve labor. Actual resource allocation and dated admission remain authoritative.

## Comparison settings

The 48-month comparison holds workshop cash preferences at monthly exposure and
town preferences at terminal reserve. The workshop targets one REFINE completion
per month, either before or after its cash goal; Off retains cash-only preferences.
The town has no opening ore. Workshop starts retain both ore and metal, metal only,
ore only, or neither, using the preceding stock-ablation quantities. Preferences
are installed at founding, not amended into an existing signed mandate.

The common-opening diagnostic separately compares retaining the month-12 program
against dropping its ore bid. On the same state and peer hypotheses, retaining
projects four completions and dropping the bid projects one. Their monthly
production shortfalls are eight and eleven; their cash losses remain 11,910 and
11,870. Production-first prefers retaining the bid. This local ranking does not
establish startup viability.

## Forty-eight-month results

All rows use the ordinary audit and unchanged readiness checker. Food deficits
are cumulative; wage arrears are outstanding at the end. Readiness additionally
checks historical late payments and all three complete operating years.

| Opening workshop stocks | Priority | Control | Mints | Workshop batches | Town batches | Food deficit | Wage arrears | Forward signed / activated | Ready |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | --- | --- |
| Both | Cash only | Baseline | 41 | 40 | 40 | 0 | 0 | 7 / 9 | No |
| Both | Production first | Baseline | 44 | 43 | 43 | 0 | 0 | — | Yes |
| Both | Cash first, then production | Baseline | 44 | 43 | 43 | 0 | 0 | — | Yes |
| Metal only | Production first | Baseline | 44 | 43 | 44 | 0 | 0 | 3 / 5 | Yes |
| Ore only | Production first | Baseline | 0 | 1 | 2 | 67 | 2 | — | No |
| Neither | Production first | Baseline | 0 | 1 | 3 | 67 | 2 | 1 / 3 | No |
| Metal only | Production first | No ore supply | 2 | 0 | 0 | 52 | 2 | — | No |
| Metal only | Production first | Low metal price | 6 | 4 | 6 | 31 | 2 | 2 / 4 | No |

The both-stock variants close the previous annual production shortfall and
operate without a startup forward. Metal-only uses an accepted forward in month
3, activated in month 5. The earlier cash-only metal-start comparison signed in
month 5, activated in month 7, and completed 40 workshop batches rather than 43.
That historical comparison is recorded in [input-buffer results](INPUT-BUFFER-PREFERENCES.md);
the cash-only both-stock control was rerun here.

Forward authority remains enabled in all these runs. No signed forward in the
both-stock result does not prove that removing the available financing option
would leave planning unchanged.

Cash-first plus production also passes the both-stock case, so this result does
not establish that production must outrank cash. Nor does it establish that two
priority orders have identical decisions merely because their aggregate outcomes
agree. The tested production-first starts without opening metal still fail to
sustain operations. Only the both-stock baseline compares all three priorities;
other stock variants and adverse controls here use production-first alone.
Reducing prices produces some work but leaves food deficits and wage arrears.
Neither finding can be repaired by treating a completion target as finance or
declaring a successful forward equivalent to a sustainable business.

## Reproduction

From `exp/economics`:

```sh
cargo +1.92.0 run --locked --release --example production_preferences -- both production
cargo +1.92.0 run --locked --release --example production_preferences -- metal production low-price
cargo +1.92.0 test --locked --release --test production_preferences -- --nocapture
cargo +1.92.0 test --locked --release --test production_continuation
```

The example reports readiness without treating intentional economic failure as a
simulation error. Tests separately assert scenario outcomes and accounting.

## Limits

Producing more can consume working cash or create unsold inventory. A completed
batch is not evidence of a buyer, income, timely wages or satisfied household
needs. This objective is an explicit policy choice, not a new universal definition
of rational behavior. Any later default change needs founding-to-operation
evidence, including supply and margin failures. Recipes, prices, employment,
constitutions, consent and initial resources remain supplied in this comparison.

## Verification scope

The targeted run covers 65 tests across 13 Cargo targets: the 30 library tests,
20 existing and new forecast/observer/forward regressions, five flow-boundary
tests, eight 48-month outcome comparisons, and two 24-month continuation cases.
The outcome tests pin readiness, actual production, nutrition deficits, wage
arrears and forward signing/activation. Independent review checked implementation,
evidence and interpretation. This is not a full-crate suite run.

The continuation cases use production-first with both stocks and metal only.
Reference and CubeCL CPU execution agree on world, state, ledger, reports and
audit. The CPU simulation is reconstructed after every phase while retaining
ledger and reports. Replay compares state after each phase and final audit using
the live reference world; independent catalog reversal compares final state,
ledger and audit. These controls do not certify durable serialization, bare-ledger
world reconstruction, 48-month CPU parity or GPU execution.

Formatting, strict all-target release Clippy and repository artifact checks pass.
The example command also completes the both-stock production-first comparison.
Generated outputs stay under ignored `output/economics/`.
