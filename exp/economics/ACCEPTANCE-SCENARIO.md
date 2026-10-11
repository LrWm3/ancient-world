# Mixed-organization acceptance scenario

Status: six-pass refinement in progress. This is one opt-in fixture built on the
integrated commitment economy, not a replacement scheduler or a claim of arbitrary
organization support. Run from this directory:

```sh
cargo +1.92.0 run --locked --release --example acceptance
```

The acceptance target is people, a household, a workshop, a town organization, a
state and a venue interacting through finite resources, lawful agreements and
separate accounting. Success requires actual outcomes, not merely named agents.

## Pass 1 — establish the existing chain

`acceptance::Run` reuses the elected state/household commitment fixture and normal
Audit stepping. It reports actual nutrition deficits, completed mint work, forward
deliveries, annual land payments, issuance and reconciled separate agent books.
It rejects evidence requests before a completed month and reconciles the entire
coin supply against issuance. The 14-month reference control has two opening
nutrition deficits, four completed mints, 40 issued coins, two delivered wheat and
one paid annual land due. A final delivered total alone does not prove timeliness;
the underlying commitment tests retain the month-four deadline assertion.

## Boundaries to close or retain explicitly

- Workshop: no distinct operating company yet in this initial pass.
- Town: no governed municipality yet; locality is not governance.
- Marketplace: the existing mint venue supports scoped posted quotes; it is not a
  universal market for all contracts. Do not combine incompatible acquisition drivers.
- Founding consent, constitutional menus, quotes and initial endowments are supplied.
- No automatic company ownership/dividends, municipal formation or dissolution,
  multiple jurisdictions or population/discovery integration is implied.

## Pass 2 — make metal supply an operating dependency

The workshop is a distinct permitted type with its own ore, coins, storage and
zero endowed hours. A four-month employment agreement buys two smith-hours each
month; ordinary productive work converts two ore into two metal. The workshop
replaces the original seller in the existing venue's metal quote. Output from
Productive can first be offered at the following Acquire. Opening metal is removed,
so the state cannot issue coins unless the workshop actually produces and sells it.

Employment is an explicit accepted commitment, not an optimizer that automatically
stops wages when no useful job exists. Missing ore can leave paid but unused hours;
missing labor cannot create output. Company founding, ownership and governance
remain open. The runner starts from this month-one fixture; restart verification
must preserve its Audit and opening-supply basis rather than construct a new book.

Measured reference outcome: four workshop batches, eight paid wages and four mint
batches; eight distinct agent books reconcile. Removing ore or smith hours stops
both refining and minting. Workshop cash returns to eight, but this is **not
profitability**: the supplied metal price covers wages and consumes the opening ore
endowment without replacing its carrying value. Financial sustainability remains a
separate acceptance gate.

## Pass 3 — municipal treasury and competing employment

A distinct municipal type owns the plot and receives its existing one-coin annual
lease payment. It has ten opening coins, no endowed hours, and an accepted contract
for one smith-hour each month to collect one unit of retained municipal fuel.
The workshop's employment rank precedes the municipality's rank. Both requests use
the same ordinary employment reservation boundary; timing is unchanged.

The first four months leave no town hours because the workshop buys both available
smith-hours. After its four-month contract expires, the town can buy one hour per
month. This is an operating mandate and real treasury, asset and output accounting,
not a municipality with implemented founding, elections or autonomous spending.
Retained fuel is not yet distributed public benefit, and annual rent does not cover
recurring payroll. Neither this treasury nor the workshop receives synthetic labor.

## Pass 4 — paid people must also eat

The smith and existing mint worker each require one wheat per month, start with
one wheat, and retain their own accounts. The smith starts with one coin; employment
income arrives through normal wage settlement. Wheat quotes use three-unit lots at
three coins. Public sales and explicitly authorized household/grower surplus asks
share the existing mint venue. The grower retains two private wheat; household
membership does not authorize sale of private inventory. Sale/claim protection
uses one month. Quote limits and stock targets are supplied, not ZIP learning.

Calibration is explicit: the broad fixture uses six wheat and two seed per harvest,
four hours, and a private grain work target of twelve. The original integrated
fixture remains unchanged. `TightFood` changes only the broad fixture's wheat yield
to four; `NoSalesTarget` removes only its commercial work target. Neither the work
target nor the quotes are autonomously discovered business plans.

The diagnostic run found that increasing yield alone did not feed the smith.
Storage blocked repeated harvests, and private wheat was not on the market. Adding
a protected private ask reduces the full-run food deficit to five (four for the
smith) and creates four household food sales and three smith purchases. This is
still a **failed full-nutrition gate**, not sustainable autonomous coordination.
Each sale/purchase count is a completed whole-lot deal, not a wheat-unit count.
The two-wheat forward delivers by month four, the state loan is actually accepted
and repaid, and annual rent is paid. Missing loans report `None`, not zero debt.

The household and grower feed different buyers: aggregate household sales do not
prove household-to-smith exchange. The test checks a committed private-seller route
to the smith explicitly. Crop accounting retains the inherited 4:2 wheat/seed cost
weights as a supplied carrying-cost policy; those weights are not output quantities
or market valuations for the calibrated 6:2 recipe.
