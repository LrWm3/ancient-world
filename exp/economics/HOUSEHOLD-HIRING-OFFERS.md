# Household acceptance of labor offers

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
member tie-breaking, process feasibility and productive-work allocator. Only the
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
visible in employment receipts (`NoUsefulWork`, `UsefulWorkLimit`, or the existing
budget/permission/unavailability reasons). No second wage or accounting book exists.

## Evidence and limits

Focused tests demonstrate buying two useful hours from a three-hour offer; no
hiring without work, enough budget, constitutional permission or positive modeled
benefit; no extra hire when own labor suffices; and no duplicated demand across two
workers. They check actual output, cash and paid-capacity basis, replay, altered
receipt rejection, checkpoint continuation and CPU/reference equality.

The first adapter supports plain or town acquisition, direct loans and direct
prepaid deliveries. It does not add ZIP wage negotiation, universal labor-market
participation, multi-month staffing optimization or internal member employment.
Worker consent and offered terms are supplied; household acceptance is agentic.
Person self-directed policy changes remain deferred.
