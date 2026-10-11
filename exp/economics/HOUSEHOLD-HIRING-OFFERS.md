# Household acceptance of labor offers

The [common offer adapter](ACQUISITION-ADAPTERS.md#posted-household-labor-through-common-offers)
now exposes these terms to the named employer and prepares the ordinary
policy-selected monthly fill, including financial commitments and household work.

Implemented as the second one-item Fibonacci batch, after direct-forward recovery.

`World.employment` remains the shared terms catalog. IDs in
`World.employment_offers` are posted, worker-consented offers to a named household;
other entries remain preaccepted employment. Offer dates bound availability, not
promised future employment. Each month's exercised quantity creates the ordinary
earned-wage claim. Unaccepted offers do not appear as accepted agreements or block
household dissolution. Historical earned claims remain payable.

The existing static charter `hiring_budget` authorizes bounded acceptance. At
Acquire, already accepted acquisition spending and preaccepted jobs reserve first.
Offers follow explicit `(rank, id)` priority. Each offer must satisfy permissions,
living/active counterparties, available worker hours after membership commitments,
charter budget, opening liquidity and prior unpaid wages. Active recovery blocks
new acceptance. Incoming cash cannot fund another acquisition at this boundary.

The household compares its existing contributed-labor plan with a plan that adds
the offered hours. The comparison uses the existing constitution, governor policy,
member tie-breaking, process feasibility and productive-work allocator. The
preview uses acquired-stock pooling, collective material allocation and consented
support before comparing work. Only the
incremental hours actually directed to work are considered; the trimmed quantity
is checked again. Earlier accepted hires enter later comparisons, so two offers
cannot independently count the same work benefit.

Needs-first policies may spend their bounded budget to reduce current deficits.
Otherwise the net-output policy requires incremental output score above the wage
cost, using the allocator's documented par/quoted-resource proxy. NeedsThenIncome
compares next-book household cash including actual earned payroll. Its forecasts
do not assume another round of hires. This remains a bounded forecast, not proof
of future sales or solvency. Execution rechecks its real Productive boundary.

Accepted work uses the existing capacity transfer, payroll, arrears, pooling,
service-cost and financial-statement paths. Rejections and trimmed offers are
visible in employment receipts (`NoUsefulWork`, `UsefulWorkLimit`, `WorkerProtection`, or the existing
budget/permission/unavailability reasons). No second wage or accounting book exists.

## Evidence and limits

Focused tests demonstrate buying two useful hours from a three-hour offer; no
hiring without work, enough budget, constitutional permission or positive modeled
benefit; no extra hire when own labor suffices; and no duplicated demand across two
workers. They check actual output, cash and paid-capacity basis, replay, altered
receipt rejection, checkpoint continuation and CPU/reference equality.

The adapter supports plain or town acquisition, direct loans, direct
prepaid deliveries and physical minting. Mint input packages reserve worker hours
before optional household hiring; an unfunded package leaves those hours available. It does not add ZIP wage negotiation, universal labor-market
participation, multi-month staffing optimization or internal member employment.
Offered terms remain supplied. Worker consent is supplied by default; the opt-in
assessment below can restrict hours. Household acceptance is agentic.
Person self-directed policy changes remain deferred.

Integration follow-up also checks prepayment at month one, affordable hiring at
month two and real delivery at month three, with separate balances throughout.
A town-income case hires two hours per month for three months: household cash
increases from four to seven, the worker earns six and the food buyer spends nine.
Increasing the wage above the predicted sale proceeds rejects the hire. Buyer
liquidity is finite; this does not establish indefinite economic sustainability.

## Optional worker assessment

`World.employment_supply` assigns a bounded forecast horizon (1–24 months) to a
participant. For that participant's posted jobs, acceptance must also preserve projected
survival, need satisfaction and process completion for the worker and current
household members. Keeping the hours is compared with consuming them; hypothetical
wages and employer outputs are not credited. The approved quantity is capped at 64 whole
hours. Each final quantity must pass both worker and employer comparisons. This is
conservative opportunity-cost protection, not wage bargaining or a claim that the
best-paid job is selected. Accepted forward deliveries and loan arrears are also compared in their own units.
Forward shortfalls are fixed at their deadlines; loan losses use worst recorded
arrears per accepted loan. This does not yet cover every obligation kind.
An empty policy map preserves supplied worker consent; preaccepted jobs are unchanged.

The shared post-Acquire preview stages validated citizenship/access, actual
acquisitions, pooling and prior hires before Productive. Opening labor permissions
still govern employment. A binding dated production plan prevents optional hiring;
this slice does not amend committed plans. Discovery can compose with posted hires
when discovered land is disabled. Land's prerequisite reservations still need a
handoff before that restriction can be lifted.

## Repeated circulation and remaining work

The [sixteen-iteration review](REVIEW-BATCH-16.md) checks competing employers,
other household members, active processes, accepted delivery deadlines and loan
arrears, with CPU/reference and continuation controls. Planning observers expose
`worker_supply` comparisons without rerunning the forecast; final delivered hours
remain distinct from an assessment's approved quantity.

In its eight-month town-market comparison, both worker policies sustain 16 coins
of paid wages and 14 grain of sales with no food deficit. Closing the market in
month three causes a two-unit deficit under supplied consent; protected workers
retain hours for food and avoid it. Both interrupted cases still end with no
employer cash or sustained later hiring. This control has a preformed household,
supplied jobs, fixed prices and a two-grain opening food buffer. It does not resolve
the [forty-month discovered-income shortfall](PRIVATE-CIRCULATION.md).

Next integration boundaries are explicit discovered-land reservation handoff,
autonomous job posting/terms, and broader obligation performance coverage (including
land dues, wages and guarantees). Current objectives compare cumulative needs and
worst recorded loan arrears within a bounded horizon; they do not prove that every
future need or payment will be met. Person self-directed policy changes remain
deferred.
