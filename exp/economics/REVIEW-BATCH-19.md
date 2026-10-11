# Nineteen expansion iterations

Start: `40b4236`. Independent review selects and checks each bounded change;
source-only commits retain focused test evidence. Generated results remain under
ignored `output/`. Individual self-directed policy changes remain deferred.

## 1. Plan existing-child support before production

Opt-in `demography::Config.plan_support` forecasts known children's finite stocks,
permitted consumption and exact maturity cutoff. Support is due at Open, before
hypothetical production; hypothetical births are excluded. The parent reserve is
a reusable stock floor, not recurring consumption or a new financial liability.

Independent review identified the larger-reserve case; the forecast now preserves
that floor. Audited CPU/reference controls compare identical food with planning
off/on and reserves of one/two grain. Enabled planning produces in month one and
prevents the child's month-two missed meal. The thirteen reproduction tests pass,
including the unchanged 36-month legacy population controls.

## 2. Keep alternative-food planning and consumption consistent

The substitution planner now applies the same dated support debits. Upcoming
support is earmarked in both the forecast and actual consumption, so a parent can
eat an alternative food while preserving the child's support stock. Earmarks are
released if necessary to meet the parent's own need; they are not escrow.

Independent review caught forecast-only protection; live consumption now shares
it. CPU/reference audited tests cover an unavailable alternative recipe (which
must not disable planning), existing alternative child food, and a parent choosing
lower-yield berries to retain grain for support. All fifteen reproduction tests pass.

## 3. Produce for dependents without inventing personal needs

Known support gaps can generate ordinary productive requests with no personal
need goal. The parent's need catalog and reports remain unchanged. Candidate
benefit uses retained outputs and their completion dates; existing work is counted
and duplicate requests are suppressed. Ordinary resolution still enforces inputs,
rights, labor and storage.

Independent review caught an infeasible first-choice producer and accidental
activation for unrelated debts. Selection now prefers feasible candidates and
requires actual known-dependent demand. Sixteen reproduction tests pass, including
audited CPU/reference controls for no personal food demand, disabled planning,
disabled work, and a lower-ID recipe requiring too much labor.

## 4. Retain family support before voluntary market sales

Market stock protection now includes optional dated support plus one donor floor,
in addition to accepted claims and personal consumption. This intentionally
conservative budget is not a minimal reserve, a legal debt or escrow. No future
request means no extra floor protection.

Independent review checked timing and recommended the conservative bound.
Seventeen reproduction and eleven need-order tests pass. Audited CPU/reference
controls show unprotected sales can cause next-month child hunger, protection
prevents that sale, and additional genuine surplus still trades successfully.

## 5. Buy support even when the parent has no personal food demand

Both bilateral need orders and bounded lot selection include known support
shortfalls. Material objectives are merged before comparing them with the shared
stock budget; a purchase cannot count the same grain against two obligations.
Stock shortfalls remain distinct from fulfillment-resource deficits in receipts.

Independent review checked both buyer entrypoints and their zero-demand exits.
Audited CPU/reference controls cover funded purchase, no support policy, no cash
and insufficient storage, checking subsequent child support rather than bids alone.
A bounded-lot test selects the minimum useful quantity for a support-only buyer.

## 6. Observe committed births, maturation and support

The external settlement observer projects lifecycle receipts into `person_birth`,
`person_matured` and `dependent_support` logs. Birth records include names for
people absent from the initial catalog snapshot. Support distinguishes birth
buffers from later top-ups and reports requested, delivered and missing amounts.
Filtering on either parent or child retains both counterparties.

Audited CPU controls compare observed and unobserved worlds, states, ledgers,
reports and financial books exactly, test parent/child filters and disabled
settlement logs, and reconcile emitted quantities with receipts. Nineteen
reproduction and ten telemetry tests pass. No forecast or planner logging hooks
were introduced.

## 7. Feed dependents through explicit household policy

Static `Charter.support_dependents` allows living members' children to receive
ordinary need allocations. This does not change the adult roster, rights, votes,
labor, contributions or shared storage. Children retain their private storage cap.
Requests for child labor, inputs or debt support remain invalid.

Independent review caught an unintended change to adult allocation order. Existing
adult order is now retained, with sorted children appended. Twenty-one reproduction
and twelve household-integration tests pass, including audited CPU/reference
feeding with policy on/off and empty stocks, full child storage, unchanged adult
membership, forged request rejection and reversed-adult-order scarce food.

## 8. Include dependents in collective market demand

Collective consumption projections include charter-supported children and their
private stocks. Their demand ends at the known maturity boundary, while the
opening adult roster and permission/policy context remain frozen. The projection
assumes neither new births nor future parental transfers.

Independent review checked the maturity boundary and requested direct trade
evidence. Twenty-two reproduction, twelve household-market and one forecast
boundary test pass. CPU/reference audited controls show an actual collective coin
payment and food purchase feeds a child when adults have no food demand; disabled
support or absent cash does not. Stocked-child and cross-maturity forecasts avoid
charging the same child food twice or after dependent eligibility ends.

## 9. Govern contributed labor for dependent welfare

Needs-first household governance includes charter-supported dependents in its
baseline and candidate welfare projections. Workers, donors, work rights and
retained commitments remain adult-scoped. The same finite labor pool can therefore
choose food for a hungry child over a larger unweighted craft output.

Independent review passed. Twenty-three reproduction and twenty-four household
income tests pass. Audited CPU/reference controls distinguish needs-first from
net-output policy, disabled dependent support and unavailable food production;
they check actual child deprivation and adult-only donor receipts.
