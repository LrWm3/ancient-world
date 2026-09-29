# Initial person–household economic loop

Status: implemented and verified on CubeCL CPU and the reference backend,
2026-09-29. Completion covers the bounded adult household/town-market loop below,
not the full institutional and financial roadmap.

## What completes the loop

Persons retain individual needs, holdings, commitments, productive permissions and
work targets. A lawfully formed household has its own account, a static constitution
and charter, a human governor, and a selectable allocation policy. It pools agreed
resources and storage, reserves bounded member labor, buys for member needs and
sells actual surplus through a permitted local marketplace. Settlement records real
outcomes and maintains separate person and household financial statements.

The last missing connection was private surplus. The controlled income fixture
previously stopped producing when the worker accumulated its private target of 24
fuel, even though the household still needed sales to buy food. Raising that target
would only delay the failure. The same work target, technology, opening cash and
prices now work with an explicit voluntary member support mandate.

| Part of the initial loop | Evidence |
| --- | --- |
| Formation, legal recognition, constitution, charter, governor and policy | [Governance checklist](HOUSEHOLD-BASICS.md), including election, authority and CPU/accounting controls |
| Individual needs, collective purchases and member distribution | [Town-market adapter](HOUSEHOLD-MARKET.md); private stocks reduce orders without becoming household property |
| Reserve member labor and decide whether productive work helps | [Income policy](HOUSEHOLD-INCOME.md); current needs precede expected next-book cash |
| Coordinate private surplus with collective income | Dated member mandates, protected personal reserves, actual transfer and independent acceptance |
| Consume, trade, record outcomes and repeat with finite resources | 120-month CPU/reference state, ledger, reports and accounting equality |
| Stop unaffordable or unhelpful actions; respond when access returns | Closed-market, no-demand, private-need, commitment, consent and recovery controls |
| Membership changes, solvent wind-down and retained financial history | [Membership](HOUSEHOLD-MEMBERSHIP.md) and [wind-down](HOUSEHOLD-WIND-DOWN.md) checks; these remain distinct from death estates |

## Voluntary surplus support

`households::support::authorize` accepts a member-signed instruction specifying the
household, commodity, start/end month, monthly transfer limit, minimum private stock,
consumption reserve horizon and collective stock target. A governor cannot sign for
another member. This is an additional voluntary transfer; the ordinary half-output
sharing rule remains unchanged. The fixture's worker authorizes at most one fuel
per month, keeps at least two privately, and fills only a one-fuel collective target.

`support::revoke` lets the member withdraw an instruction from the next month.
Original terms remain recorded, and prior receipts still replay. Expired or withdrawn
instructions, inactive membership, wind-down and absence of governor authority do
not permit new transfers. Later authorization can use a nonoverlapping effective
interval. Consent is explicit scenario/API input; the model does not yet generate
personal support terms or infer consent from a household's needs.

At Productive, after existing resource reservations and before contributed-labor
allocation, the support resolver:

1. Protects known obligations and unpaid active-process inputs using the existing
   claim projection, then personal consumption over the authorized horizon and the
   explicit private stock floor. Existing claim-projection limits still apply;
   this is not a forecast of all future loan interest or discretionary purchases.
2. Caps the offer by opening private surplus, the member's monthly limit and the
   remaining collective target. Incoming transfers at the same reservation boundary
   cannot be recycled into a new offer. Shared storage must permit the transfer.
3. Compares actual process/consumption previews and the existing next-book income
   forecast. No member need may worsen for the donor. Under `NeedsThenIncome`, an
   offer must improve collective current needs, or improve expected cash with equal
   current needs. No useful buyer, funds, permissions or crossed quote means no
   presumed sale. This first adapter is inactive under the other operating policies.
4. Commits the accepted stock transfer through the existing household boundary,
   then evaluates contributed labor against that updated stock. Equal income from
   another job does not justify extra contributed work.

Offers follow the configured member tie-break and resource ID order; accepted stock
is removed before the next offer is considered. Whole offers that do not fit storage
are rejected. This is bounded sequential allocation, not global portfolio optimization.

Support receipts explain protected, offered and accepted amounts, rejection reasons
and baseline/alternative cash forecasts. The settlement observer exports them as
`household_support`. Replay recomputes both consent and economic feasibility and
rejects altered transfers or receipts before publication. Existing inventory pooling
recognizes the transfer at carrying cost on separate books. It creates no wage,
receivable, new money or consolidation relationship, and is not pooled a second time.

## Controlled results

Two adults share the household; two independent grain producers participate in the
same permitted town venue. Household members contribute 20% of five labor units.
One six-labor job produces two fuel, shared equally with the household. The household
buys a two-grain lot for 40 coin ticks, while an external producer buys one fuel for
40. Grain production consumes and replaces seed. Prices and work opportunities are
supplied, deliberately isolating coordination from price discovery.

| Run | Result |
| --- | --- |
| Original policy, no support mandate | Private fuel reaches 24; productive income stops, then household food deficits start in month 27 |
| Coordinated 36-month CPU example | All household food needs met; 35 fuel sold; closing household cash 60 after each month |
| Coordinated 120-month CPU/reference comparison | All 240 member-month food requirements met; 119 fuel sold; total coins conserved; separate statements reconcile |
| Temporary fuel-market closure in months 11–13 | No support transfers or extra contributed work during closure; real food shortages occur; members are fed again from month 15 after trading resumes |
| No demand, protected private need/input, expired/withdrawn consent | No unpermitted private-stock capture or invented income |

Private fuel is at most three units and settles into a two/three cycle. From month
four the household alternates voluntary stock support with productive labor. The
initial buyer's first-month warmth deficit remains: no fuel has yet reached a market.
The household cannot use a same-book sale to fund a purchase against insufficient
opening cash, so recovery includes a real liquidity delay. Ten-year viability here
is evidence for this calibration, not a guarantee under every price or shock.

## Reproduce and verification scope

Run from `exp/economics`:

```sh
cargo +1.92.0 run --locked --example household_income
cargo +1.92.0 run --locked --example household_income -- 120
cargo +1.92.0 test --locked --test household_income
```

The example now defaults to 36 months and uses `income::scenario::coordinated()`.
`income::scenario::scenario()` retains the original failure control. Tests also
cover reordered input tables, checkpoint continuation across the old failure date,
atomic rejection of forged receipts, immutable observer behavior, and member
withdrawal without changing historical settlement.

Verification: **215 distinct tests passed** across 15 selected household, market,
accounting, law and telemetry suites. The broad run passed 212 tests, including the
120-month audit; three additional boundary controls then passed, and the final
23-test income control run passed (the already-passed long audit was excluded
from that repeat). Strict all-target Clippy, formatting, the 36-month CPU example
and the repository artifact check passed. One slow 32-person annual accounting
test remained ignored; the full crate suite was not run.

Generated logs stay under ignored `output/economics/household-loop-*.log`.

## Remaining extensions

The initial loop is complete within this scope. Available jobs, private stock
targets, support terms, governance instructions and reservation prices can still be
supplied. Autonomous business discovery, personal policy/consent selection,
long-horizon joint planning, speculative input purchases, household hiring remain subsequent work.
[Five consolidation passes](INTEGRATION-PASSES.md) now connect outside wages,
private surplus sales, static cash buffers and direct loans with town purchases.
Remaining town-market composition exclusions stay enforced. Person self-directed
policy changes are deferred by request. Recruitment, children, automatic death
estates, nested institutions and autonomous state governance also remain extensions.

[Five further passes](INTEGRATION-PASSES-2.md) connect physical barter, partial
support, charter-delegated consumption buying, membership-aware order gates and
collective funding of active member-process inputs. These preserve separate
identities, opening budgets and financial books.


[Passes 11–15](INTEGRATION-PASSES-3.md) connect physical payroll, pooled inventory
costs, opt-in current member loan assistance, collective acquisition of loan-payment
stock and explicit scarce-support policy. CPU/reference checks combine wages or
market exchange with later collection and separate financial statements. These
are current accepted obligations, not autonomous borrowing or long-horizon
financing plans. Person self-directed policy changes remain excluded.
