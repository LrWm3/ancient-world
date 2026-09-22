# ZIP pricing in the bilateral marketplace

Implemented opt-in `QuotePolicy::Zip(Config)`, alongside fixed quotes and bounded
concessions. The marketplace remains person-only and facilitates the same
whole-lot grain/coin exchange. ZIP changes quoting and retained pricing state;
it does not grant funds, create goods, choose counterparties or bypass settlement.

Run from `exp/economics`:

```sh
cargo +1.92.0 run --locked --example zip
```

## Learning rule and source

The implementation follows the signed-margin and Widrow–Hoff momentum formulation
in [Dave Cliff's 2008 lecture, slides 9–10](https://archive.cs.st-andrews.ac.uk/dls-archive/files/DCliff_2_050308.pdf).
With private limit `lambda`, margin `mu`, internal price `p`, target `tau`, learning
rate `beta` and momentum `gamma`:

```text
p = lambda * (1 + mu)
delta = beta * (tau - p)
adjustment = gamma * previous_adjustment + (1 - gamma) * delta
next_margin = (p + adjustment) / lambda - 1
```

Targets lie slightly above or below an observed price, using random absolute and
relative perturbations. Rejected bids can make buyers more competitive; rejected
asks can make sellers more competitive. After a trade, traders able to transact
at that price seek greater surplus; less competitive traders move toward it.
Buyer margins are nonpositive and seller margins nonnegative. The historical
formulation is described in Cliff's [1997 report record](https://research-information.bris.ac.uk/en/publications/minimal-intelligence-agents-for-bargaining-behaviors-in-market-ba/).

## Explicit adaptations to this experiment

This is a ZIP margin policy inside our **bilateral paired-quote mechanism**, not
a reproduction of a continuous double auction. Both quotes become public together;
non-crossing pairs emit a rejected bid and rejected ask. Crossing quotes retain
the existing midpoint pricing rule. A completed exchange emits a trade-price
event with no aggressor identity. Consequently, trade responses here compare each
participant's quote directly with that price; there is no accepted-bid versus
accepted-ask distinction or population of inactive observers.

The event adapter is separate from the learner. `zip::Event` carries only public
side/price information and distinguishes `Rejected`, `Trade` and
`SettlementFailed`. The learner receives its own limit, never the other person's
limit. Failed money, stock or storage checks do not emit a successful trade event
and do not trigger trade learning. Earlier rejected public quotes in that session
can still have supplied learning observations.

Prices and balances remain integer payment units. Learning retains millionths
of margins and payment adjustments using fixed-point arithmetic, with wider
intermediate calculations. Published quotes round to the market tick and remain
within private limits; updates clamp to legal bounds and retain the realized
adjustment at a bound. This clipping, finite price range and integer precision
are explicit numerical choices. Internal progress is retained even when a
published quote has not yet moved a whole tick. ZIP therefore does not use the
fixed/concession policy's unchanged-quote early exit; the configured round budget
still bounds execution.

Default beta is 0.30, gamma 0.05, relative target displacement is up to 5%, and
absolute displacement up to one market price tick. These fixture parameters are
fixed, not independently randomized or calibrated. Initial margins derive from
the same opening quotes used by the comparison policies. Seeded target draws are
separate per venue, participant, market and side, with the PRNG state persisted.

## Persistence and monthly timing

Each marketplace pricing record now optionally holds margin, previous adjustment,
random state and update count. It also retains the last publicly quoted price;
that can differ from the next learned quote after processing the final event.

Acquire creates a local learner copy and consumes the current session's public
events. The batch retains those events and both ending learner states. Commit
recomputes them and rejects altered learning or receipts. Pricing memory, event
history and physical transfer effects publish together on staged state. Replay,
resource failure or any other rejected batch cannot partly advance learning.

A later explicitly scheduled session with the same policy resumes its margin and
momentum. A changed reservation limit reprices that margin against the new limit.
Changing policy or configuration resets learning from the supplied opening quote.
Buy and sell records are separate; there is no counterparty-pair key. Checkpoint
cloning preserves the complete learning and random sequence. No new monthly
phase, automatic order renewal or membership flow is introduced.

## Controlled CPU comparisons

Each run has 12 scheduled monthly sessions and at most 64 quote pairs per session.
Buyer starts with 600 coin ticks, seller with 24 grain, and buyer storage holds
24 grain. Every order covers two grain. Initial limits are buyer 50 and seller 30;
opening quotes are 20/60 for every policy. Each policy sees the same dated orders
and initial physical resources. Later holdings and quotes follow actual outcomes.

| Scenario | Policy | Trades / 12 | Total quote rounds | Trade price range | Buyer surplus | Seller surplus |
| --- | --- | --- | --- | --- | --- | --- |
| Stable limits | Fixed | 0 | 12 | — | 0 | 0 |
| Stable limits | Concede | 12 | 16 | 40 | 120 | 120 |
| Stable limits | ZIP seed 7 | 12 | 68 | 35–36 | 169 | 71 |
| Stable limits | ZIP seed 19 | 12 | 67 | 37 | 156 | 84 |
| Limits rise in month 7 | Fixed | 0 | 12 | — | 0 | 0 |
| Limits rise in month 7 | Concede | 12 | 18 | 40–50 | 240 | 60 |
| Limits rise in month 7 | ZIP seed 7 | 12 | 73 | 36–58 | 217 | 83 |
| Limits rise in month 7 | ZIP seed 19 | 12 | 72 | 37–60 | 203 | 97 |
| Incompatible limits | Fixed | 0 | 12 | — | 0 | 0 |
| Incompatible limits | Concede | 0 | 18 | — | 0 | 0 |
| Incompatible limits | ZIP seed 7 | 0 | 768 | — | 0 | 0 |
| Incompatible limits | ZIP seed 19 | 0 | 768 | — | 0 | 0 |

The limit-change control raises buyer/seller limits to 80/50 in month seven.
ZIP seed 7 then trades at 58 for five sessions and 57 in the last; seed 19 trades
at 60 once and 59 thereafter. The incompatible control instead sets the buyer's
limit to 25 throughout, below the seller's minimum 30.

Quote rounds measure negotiation attempts, not months or wall-clock performance.
Surplus sums buyer limit minus price and price minus seller limit for completed
lots; it is not newly created money. The stable profitable cases all realize
240 ticks of combined surplus, and the changed-limit cases 300. ZIP changes who
captures that surplus and uses more rounds than concessions. It does not produce
more trades here. The no-overlap case exposes its continued bounded negotiation
attempts despite private limits that cannot cross.

## Validation and limits

Seven ZIP tests include a hand-calculated two-event momentum update, event
selection, seeded repeatability, retained sub-tick learning, invalid parameters,
private limits, policy reset, changed limits, tamper rejection and failed-payment
controls. Repeated CPU and reference runs agree after checkpoint continuation and
agent/resource table reversal. The existing 18 marketplace, negotiation, finance
and pricing tests also pass (25 total). Clippy with warnings denied, formatting
and repository artifact checks pass. Raw runs stay in ignored `output/economics/`.

These are two-person implementation controls with externally supplied values and
orders. Asymmetric initial margins, midpoint pricing, the tick size, round budget
and two sampled seeds influence the observed surplus split. There is no claim
of competitive equilibrium, calibrated behavior, or superiority over concessions.
Need-derived valuations, multi-buyer/multi-seller matching and a richer event
stream remain separate next experiments.
