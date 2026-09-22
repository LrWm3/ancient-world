# Monthly environmental collection market

Status: implemented CPU pilot. The adapter is configured by process, consumer,
shared input account, seed and allocation policy; it contains no person/farmer
special case. Wood is the first controlled example.

## Offers, requests and commitment

The existing offer discovery interface exposes collection process terms together
with dated shared-stock availability and input units per lot. An agent's ordinary
need-directed search can request collection to replenish its configured stock
buffer. Quantity is the projected deficit divided by output per lot, rounded up,
then bounded by opening private service capacity. In this fixture the buffer is
six months. Quantity is divisible across whole process lots, not fractional labor
or arbitrary fractions of a process.

The resolver collects all submitted quantities before spending the shared pool.
It reserves non-collection work first, including continuing crops and any selected
new production. It then checks each collection request against remaining private
labor, permissions, inputs and storage. This feasibility preview temporarily removes
public-stock scarcity; its shadow stock is never committed. Actual reservations
always use the real finite pool and the ordinary process resolver.

The shared `allocation::resolve` policy receives private-feasible lot quantities,
a minimum useful grant of one lot, and a dated pool/seed context. Two configurations
are compared:

- `PriorityLottery`: agents whose opening fuel cannot cover this month's warmth
  need precede buffer replenishment; seeded lottery breaks ties.
- `Lottery`: seeded lottery without that urgency class.

A reservation must jointly afford the granted wood and labor. Failed reservations
leave the grant available to the next claimant. Unused stock, including a remainder
smaller than one lot, stays in the pool. Future Open phases add regeneration only
up to the pool's capacity. Grants are bounded by demand and available lots, not an
entitlement to the entire forest or workforce.

Each productive batch records opening supply, input units per lot, original work
requests, requested quantities, private-feasible quantities and allocation receipts.
Normal process receipts record completed work. Settlement recomputes the round and
rejects altered or missing allocation receipts for collection transactions.
Actual withdrawals and outputs are still validated and gathered through CubeCL CPU
settlement. The CLI prints a monthly request/reservation/completion summary.

## Timing and consequences

Open still regenerates stock and labor once per month. Acquire forecasts and stores
a dated Productive plan. The actual Productive phase executes that plan; it does not
rerun allocation after another claimant spends resources. Consumption can use the
collected fuel that month. Close evaluates unmet needs and applies existing
condition rules. The next planning boundary reads actual remaining stocks and
conditions, so losing a collection allocation can change subsequent choices and
available labor. It does not invent a debt or a contract breach.

Collection cannot take hours already reserved by the other selected work. Prolonged
cold can nevertheless reduce next month's labor capacity below a crop commitment's
minimum, causing its existing abort-without-refund consequence. This is an explicit
condition effect, not collection retroactively displacing completed crop work.

## Controlled CPU runs

Run from `exp/economics`:

```sh
cargo +1.92.0 run --locked -- wood-market-ample
cargo +1.92.0 run --locked -- wood-market-sufficient
cargo +1.92.0 run --locked -- wood-market-scarce
cargo +1.92.0 run --locked -- wood-market-sufficient-lottery
cargo +1.92.0 test --locked --test pool_market
```

Append `-lottery` to any supply scenario for the policy control. Initialization
uses the reference backend to admit both people to plots and start their crops in
month 1. Measurement runs for 36 months, months 2–37, using CubeCL CPU settlement.
Both agents start that comparison with one fuel unit. Land, crops, initial grain,
seed commitments, three monthly labor units, annual two-grain rent, condition rules
and seed 7 are otherwise identical across the controls.

Collection produces two fuel units for one labor unit, as before. Its input is now
denominated as **two raw wood units per lot** in these opt-in scenarios, so integer
regeneration can represent a supply below one collection lot per month. The pool
starts empty and holds at most eight wood units. Regeneration is 4, 2 or 1 wood
units per month; two people together require two fuel units monthly. Existing
opportunity scenarios retain their original wood terms.

| Supply | Policy | Unmet warmth | Unmet nutrition | Harvests | Aborted crops | Deaths |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Ample, 4 wood/month | Urgency + lottery | 0 | 0 | 10 | 0 | 0 |
| Ample, 4 wood/month | Lottery | 0 | 0 | 10 | 0 | 0 |
| Sufficient, 2 wood/month | Urgency + lottery | 0 | 0 | 10 | 0 | 0 |
| Sufficient, 2 wood/month | Lottery | 4 | 0 | 10 | 0 | 0 |
| Scarce, 1 wood/month | Urgency + lottery | 10 | 6 | 5 | 1 | 1 |
| Scarce, 1 wood/month | Lottery | 10 | 6 | 5 | 1 | 1 |

Under scarcity, person 88's warmth deprivation reduces labor capacity before the
crop aborts. The person subsequently dies from nutrition deprivation in month 11.
Once only one person remains, the same supply can meet that person's warmth needs;
ending shortfalls alone would hide the earlier failure. Under sufficient supply,
urgency improves delivery timing even though both policies complete the same total
number of collection lots. This is evidence for this seed and scenario, not a
claim of universal policy superiority or calibrated human physiology.

## Tests and limits

Focused tests compare policies against identical opening requests and stock, exercise
partial quantities, unavailable labor, inactive requests and sub-lot stock, and check
atomic rollback for tampered or missing receipts and duplicate settlement. CPU and
reference results agree with reversed participant order and monthly/checkpoint
continuation. The scarcity test links prior warmth impairment to the crop's
insufficient-capacity failure and reconciles every productive round's grants,
completed lots and actual wood withdrawal.

This remains one configured collection process and one simple consumption chain.
It is not a universal allocation policy for all environmental resources. Requests
still come from the existing bounded planner, which can choose to wait; urgency
only ranks submitted requests. The recurring planner still evaluates participants
jointly and has no learned model of rivals or probabilities of fulfillment. This
slice has no collection prices, payments, continuous quantities, resource-access
leases, bargaining or ZIP pricing. It deliberately protects selected non-collection
work before this allocation window rather than optimizing every use of labor at once.

Generated run and test logs stay under ignored `output/economics/`; this Markdown
records settings, outcomes and limitations.
