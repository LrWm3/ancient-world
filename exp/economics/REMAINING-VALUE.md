# Choosing work after acquiring an ongoing agreement

Implemented in `src/work_choice.rs` as the opt-in **remaining-value** policy.
An agent compares continuing an active process with other discoverable processes
and waiting. The CPU fixture applies it to the state after repossessing a crop;
the evaluator has no state, crop or wood-specific decision branch.

## Discovery, forecast and execution

`World.work_choice` configures one agent, a monthly horizon and explicit stock
values. These values express preferences; they are neither market quotes nor coin
income. The agent in this experiment has no urgent needs. Its objective is to
increase the value of its holdings within the horizon.

The policy discovers alternatives through the existing `offers::discover`
interface. Candidate plans are:

- Wait, declining any ongoing work.
- Repeat an available one-month process.
- Continue the active agreement, then wait.
- Continue it while trying an alternative after its monthly work. Unused capacity
  can serve that alternative; after completion the alternative can use the full
  monthly allowance.

Each alternative may run at most once per month. The bounded search supports at
most one active agreement, four one-month alternatives and a twelve-month horizon.
New multi-period starts and arbitrary combinations of alternatives are outside
this search. Every productive input/output stock requires an explicit valuation;
zero is allowed, but missing values are rejected.

Each candidate runs in an isolated reference forecast through the ordinary offer
resolver and settlement code. Rights, stocks, minimum service requirements and
storage still limit what actually completes. The score is:

```text
sum(resource value × (ending stock − opening stock))
```

Future inputs lower the score; completed output raises it. Prior seed and labor
are already spent and are not charged again or refunded. Remaining labor affects
feasibility and displaces alternative production. Higher stock value wins, then
less completed work, then stable candidate order. There is no additional assumed
wage or market sale of outputs.

The policy replans each month. Its forecasts assume the currently observed
capacity remains available and use known process terms. They do not read future
fixture capacity shocks, future scheduled starts or discretionary cash transfers.
Candidate plans are fixed during each forecast rather than recursively searching.
This is an approximation, not a promise that future outcomes will match.

## Monthly boundary

Open establishes capacity. Due can transfer title and the active crop. At
Productive, the new controller evaluates that committed state, chooses requests,
and the existing resolver reserves resources and executes the granted work.
Timing and allocation remain separate; no scheduler phase was moved.

Choosing not to continue applies the existing missed-work failure rule at
Productive. Transfer itself leaves progress intact. Other agents still emit their
normal work requests. The current policy supports independent work budgets, not
competition for a shared labor pool. Forecasts do not reserve real resources.

Each productive batch stores all candidate scores, first-work receipts, projected
stocks and the selected plan. Settlement recomputes the decision and resulting
batch before publishing it, rejecting altered choices or work atomically. CPU
execution uses the same gathered update path as the other experiment scenarios.

## Controlled CPU outcomes

The state values grain, seed and wood at one unit each. It has two labor units
per month. Wood collection requires two units and produces four wood immediately.
The inherited crop produces eight grain and one seed. It needs one unit per growth
month and two at harvest; remaining capacity during growth cannot complete the
indivisible wood task. Repossession occurs in month three. The horizon is four
months, covering months three through six.

| Control | Remaining crop time | Best continuing plan | Wood-only plan | Choice |
| --- | --- | ---: | ---: | --- |
| Nearly mature | 2 months | 17 | 16 | Maintain, then collect wood |
| More work remaining | 4 months | 9 | 16 | Collect wood; crop fails |
| No capacity | 4 months | 0 | 0 | Wait; crop fails |

The mature plan produces nine units of crop/seed value, then eight wood. Choosing
only continuation would score nine, so evaluating the subsequent alternative
matters. In the longer case, five labor units spread over four months displace
four wood collections worth sixteen. It is feasible to maintain the crop, but
that is not the agent's preferred outcome at these values.

By month six, the mature case holds eight grain, one seed and sixteen wood; the
longer case holds twenty-four wood; the zero-capacity case holds none. The wood
totals include eight collected before repossession in the two funded controls.
All three retain the same 21.60-coin deficiency: crop decisions do not reprice
the fixed collateral settlement.

Additional controls require two seed at harvest. Without those inputs the crop
cannot complete. With them, continuation is feasible but its net value falls from
seventeen to fifteen, below wood's sixteen. Increasing the grain preference to
two makes maintaining it preferable again.

## Verification and limits

Run from `exp/economics`:

```sh
cargo +1.92.0 run --locked --example work_choice
cargo +1.92.0 test --locked --test work_choice
```

Five focused tests cover the choices, remaining inputs, changed preferences,
atomic rejection of forged/missing receipts, reference/CPU agreement, table
reordering, monthly/batched execution and checkpoint continuation. A future
capacity shock leaves the earlier forecast unchanged but causes actual failure
when it arrives. A one-month horizon abandons the two-month crop: delayed outputs
outside the horizon receive no terminal value. This is a demonstrated limitation.

The focused suite plus collateral, credit, economics, conditions, search,
planning and process-offer regressions passed **60 tests**. All-target Clippy with
warnings denied and formatting checks passed. CPU example output and raw logs
remain under ignored `output/economics/`.

This policy is not yet a replacement for the consequence-aware needs planner.
The [collateral resale pilot](COLLATERAL-RESALE.md) reuses its forecasts to compare
a prospective buyer's work with and without the asset and derive a bid premium.
It does not value survival, deprivation, uncertainty, resale, hired labor or
contractual penalties beyond the existing crop-failure consequence. It also does
not make borrowing autonomous or fund loan payments through sales. The next
valuation improvements should be tested against the same physical requests and
budgets rather than changing execution order or granting extra capacity.
